//! NMBS GTFS-RT with trip IDs the static feed knows (ADR-027): `overstap rt-fix`.
//!
//! NMBS trip IDs end in the trip's last service date (`…:825:20270930`). The static export
//! clips that date to its own `feed_end_date`; the realtime export doesn't (`…:825:20271210`).
//! MOTIS matches trip updates on the exact `trip_id`, so every train that runs on past the
//! end of the static feed loses its real time (2026-11-24: 244 of 615 updates).
//!
//! This proxy sits between MOTIS and the feed. A trip the static feed doesn't know gets the
//! ID of the static trip that has the same ID up to the last `:`, runs on the update's
//! `start_date`, and isn't already claimed by another update in the feed (NMBS has twins
//! that differ only in that last segment, each with its own update). Of several left, the
//! one whose last segment is the closest date wins.
//!
//! Only `trip_id` bytes change: the feed is rewritten at the protobuf wire level and every
//! other byte is copied as is (`gtfs_rt.rs` covers only part of the schema, so decoding and
//! re-encoding would drop fields MOTIS reads).

use anyhow::{anyhow, Context, Result};
use chrono::{Datelike, NaiveDate, Utc};
use chrono_tz::Europe::Brussels;
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

/// Serve one cached rewrite to everyone asking within this long (MOTIS polls every 60 s).
const CACHE: Duration = Duration::from_secs(15);
/// How often to log how the matching went (and after every static reload).
const LOG_EVERY: Duration = Duration::from_secs(3600);

// --- static side ---------------------------------------------------------------------------

/// The static feed's trips, grouped by ID up to the last `:`, and the days each service runs.
#[derive(Default)]
pub struct StaticTrips {
    ids: HashSet<String>,
    /// Prefix → (trip_id, service), sorted by trip_id so ties break the same way every time.
    by_prefix: HashMap<String, Vec<(String, u32)>>,
    /// calendar.txt: service → (weekday bits, Monday = 1, first day, last day).
    weekly: HashMap<u32, (u8, i32, i32)>,
    /// calendar_dates.txt: (service, day) → runs (exception_type 1) or not (2).
    exceptions: HashMap<(u32, i32), bool>,
}

impl StaticTrips {
    /// Read trips.txt, calendar.txt and calendar_dates.txt from a GTFS zip.
    pub fn load(zip_path: &Path) -> Result<Self> {
        let file =
            std::fs::File::open(zip_path).with_context(|| format!("{}", zip_path.display()))?;
        let mut zip = zip::ZipArchive::new(std::io::BufReader::new(file))
            .with_context(|| format!("{} is not a zip", zip_path.display()))?;
        let mut s = Self::default();
        let mut services = HashMap::new();
        s.read_trips(zip.by_name("trips.txt")?, &mut services)?;
        if let Ok(f) = zip.by_name("calendar.txt") {
            s.read_calendar(f, &services)?;
        }
        if let Ok(f) = zip.by_name("calendar_dates.txt") {
            s.read_calendar_dates(f, &services)?;
        }
        Ok(s)
    }

    /// The same from the three files' contents.
    #[cfg(test)]
    pub fn from_csv(trips: &str, calendar: &str, calendar_dates: &str) -> Result<Self> {
        let mut s = Self::default();
        let mut services = HashMap::new();
        s.read_trips(trips.as_bytes(), &mut services)?;
        s.read_calendar(calendar.as_bytes(), &services)?;
        s.read_calendar_dates(calendar_dates.as_bytes(), &services)?;
        Ok(s)
    }

    pub fn trip_count(&self) -> usize {
        self.ids.len()
    }

    fn read_trips(&mut self, r: impl Read, services: &mut HashMap<Vec<u8>, u32>) -> Result<()> {
        let (mut csv, cols) = table(r, &["trip_id", "service_id"])?;
        let mut rec = csv::ByteRecord::new();
        while csv.read_byte_record(&mut rec)? {
            let (Some(id), Some(svc)) = (rec.get(cols[0]), rec.get(cols[1])) else {
                continue;
            };
            let Ok(id) = std::str::from_utf8(id) else {
                continue;
            };
            let n = services.len() as u32;
            let svc = *services.entry(svc.to_vec()).or_insert(n);
            if let Some((prefix, _)) = id.rsplit_once(':') {
                self.by_prefix
                    .entry(prefix.to_string())
                    .or_default()
                    .push((id.to_string(), svc));
            }
            self.ids.insert(id.to_string());
        }
        for v in self.by_prefix.values_mut() {
            v.sort();
        }
        Ok(())
    }

    fn read_calendar(&mut self, r: impl Read, services: &HashMap<Vec<u8>, u32>) -> Result<()> {
        const DAYS: [&str; 7] = [
            "monday",
            "tuesday",
            "wednesday",
            "thursday",
            "friday",
            "saturday",
            "sunday",
        ];
        let mut names = vec!["service_id", "start_date", "end_date"];
        names.extend(DAYS);
        let (mut csv, cols) = table(r, &names)?;
        let mut rec = csv::ByteRecord::new();
        while csv.read_byte_record(&mut rec)? {
            let Some(&svc) = rec.get(cols[0]).and_then(|s| services.get(s)) else {
                continue;
            };
            let (Some(start), Some(end)) = (
                rec.get(cols[1]).and_then(day),
                rec.get(cols[2]).and_then(day),
            ) else {
                continue;
            };
            let mask = (0..7).fold(0u8, |m, i| {
                m | (u8::from(rec.get(cols[3 + i]) == Some(b"1")) << i)
            });
            self.weekly.insert(svc, (mask, start, end));
        }
        Ok(())
    }

    fn read_calendar_dates(
        &mut self,
        r: impl Read,
        services: &HashMap<Vec<u8>, u32>,
    ) -> Result<()> {
        let (mut csv, cols) = table(r, &["service_id", "date", "exception_type"])?;
        let mut rec = csv::ByteRecord::new();
        while csv.read_byte_record(&mut rec)? {
            let Some(&svc) = rec.get(cols[0]).and_then(|s| services.get(s)) else {
                continue;
            };
            let Some(d) = rec.get(cols[1]).and_then(day) else {
                continue;
            };
            match rec.get(cols[2]) {
                Some(b"1") => self.exceptions.insert((svc, d), true),
                Some(b"2") => self.exceptions.insert((svc, d), false),
                _ => None,
            };
        }
        Ok(())
    }

    fn runs(&self, svc: u32, d: i32) -> bool {
        if let Some(&on) = self.exceptions.get(&(svc, d)) {
            return on;
        }
        let Some(&(mask, start, end)) = self.weekly.get(&svc) else {
            return false;
        };
        let weekday = NaiveDate::from_num_days_from_ce_opt(d)
            .map_or(0, |n| n.weekday().num_days_from_monday());
        start <= d && d <= end && mask & (1 << weekday) != 0
    }

    /// Which static trip each unknown trip in `seen` (feed order) is. `today` (days from CE)
    /// stands in for a missing start_date.
    fn resolve(&self, seen: &[Seen], today: i32) -> (HashMap<Key, String>, Stats) {
        let mut stats = Stats::default();
        let run_day = |s: &Seen| {
            s.start_date
                .as_deref()
                .map_or(Some(today), |d| day(d.as_bytes()))
        };
        let mut keys = HashSet::new();
        let mut claimed: HashSet<(&str, i32)> = HashSet::new();
        for s in seen {
            if self.ids.contains(&s.trip_id) {
                if let Some(d) = run_day(s) {
                    claimed.insert((s.trip_id.as_str(), d));
                }
            }
        }
        let mut map = HashMap::new();
        for s in seen {
            if !keys.insert(s.key()) {
                continue;
            }
            stats.trips += 1;
            if self.ids.contains(&s.trip_id) {
                stats.known += 1;
                continue;
            }
            // Added and unscheduled trips are meant to be missing from the static feed.
            if matches!(s.relationship, Some(1 | 2 | 8)) {
                stats.added += 1;
                continue;
            }
            let pick = (|| {
                let d = run_day(s)?;
                let (prefix, last) = s.trip_id.rsplit_once(':')?;
                let last = day(last.as_bytes());
                let gap = |id: &str| match (
                    last,
                    id.rsplit_once(':').and_then(|(_, l)| day(l.as_bytes())),
                ) {
                    (Some(a), Some(b)) => (a - b).abs(),
                    _ => i32::MAX,
                };
                let best = self
                    .by_prefix
                    .get(prefix)?
                    .iter()
                    .filter(|(id, svc)| self.runs(*svc, d) && !claimed.contains(&(id.as_str(), d)))
                    .min_by_key(|(id, _)| gap(id))?;
                Some((best.0.as_str(), d))
            })();
            match pick {
                Some((id, d)) => {
                    claimed.insert((id, d));
                    map.insert(s.key(), id.to_string());
                    stats.rematched += 1;
                }
                None => stats.unmatched += 1,
            }
        }
        (map, stats)
    }
}

/// A CSV reader over `r` and the positions of the `names` columns (all required).
fn table<R: Read>(r: R, names: &[&str]) -> Result<(csv::Reader<R>, Vec<usize>)> {
    let mut csv = csv::ReaderBuilder::new().flexible(true).from_reader(r);
    let headers = csv.byte_headers()?.clone();
    let find = |name: &str| {
        headers
            .iter()
            .position(|h| {
                let h = std::str::from_utf8(h).unwrap_or("");
                h.trim_start_matches('\u{feff}').trim() == name
            })
            .ok_or_else(|| anyhow!("no column {name}"))
    };
    let cols = names.iter().map(|n| find(n)).collect::<Result<_>>()?;
    Ok((csv, cols))
}

/// "YYYYMMDD" → days from CE.
fn day(s: &[u8]) -> Option<i32> {
    let s = std::str::from_utf8(s).ok()?.trim();
    if s.len() != 8 {
        return None;
    }
    let n = |r: std::ops::Range<usize>| s.get(r)?.parse::<u32>().ok();
    NaiveDate::from_ymd_opt(n(0..4)? as i32, n(4..6)?, n(6..8)?).map(|d| d.num_days_from_ce())
}

// --- the feed --------------------------------------------------------------------------------

/// A trip descriptor as found in the feed.
struct Seen {
    trip_id: String,
    start_date: Option<String>,
    relationship: Option<u64>,
}

type Key = (String, Option<String>);

impl Seen {
    fn key(&self) -> Key {
        (self.trip_id.clone(), self.start_date.clone())
    }
}

/// How the trips in one feed matched, each (trip_id, start_date) counted once.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Stats {
    pub trips: usize,
    /// Already a static trip ID.
    pub known: usize,
    /// Given a static trip's ID.
    pub rematched: usize,
    /// No static trip fits: left as is (MOTIS will drop it).
    pub unmatched: usize,
    /// Added/unscheduled trips: left as is.
    pub added: usize,
}

impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} trips: {} known, {} re-matched, {} unmatched, {} added",
            self.trips, self.known, self.rematched, self.unmatched, self.added
        )
    }
}

/// Give every trip in `feed` (a GTFS-RT FeedMessage) that `trips` doesn't know the ID of the
/// static trip it is. `today` (days from CE) stands in for a missing start_date.
pub fn fix_feed(trips: &StaticTrips, feed: &[u8], today: i32) -> Result<(Vec<u8>, Stats)> {
    let mut seen = Vec::new();
    walk(feed, Msg::Feed, &mut |t: &Trip| {
        if let Some(id) = t.trip_id {
            seen.push(Seen {
                trip_id: id.to_string(),
                start_date: t.start_date.map(str::to_string),
                relationship: t.relationship,
            });
        }
        None
    })?;
    let (map, stats) = trips.resolve(&seen, today);
    if map.is_empty() {
        return Ok((feed.to_vec(), stats));
    }
    let out = walk(feed, Msg::Feed, &mut |t: &Trip| {
        let key = (t.trip_id?.to_string(), t.start_date.map(str::to_string));
        map.get(&key).cloned()
    })?;
    Ok((out.unwrap_or_else(|| feed.to_vec()), stats))
}

/// The messages on the way from a FeedMessage to a TripDescriptor.
#[derive(Clone, Copy, PartialEq)]
enum Msg {
    Feed,
    Entity,
    TripUpdate,
    Vehicle,
    Alert,
    Selector,
    Trip,
}

impl Msg {
    /// The message a length-delimited field `tag` of `self` holds, if it leads to trips.
    fn child(self, tag: u32) -> Option<Msg> {
        use Msg::*;
        match (self, tag) {
            (Feed, 2) => Some(Entity),
            (Entity, 3) => Some(TripUpdate),
            (Entity, 4) => Some(Vehicle),
            (Entity, 5) => Some(Alert),
            (TripUpdate, 1) | (Vehicle, 1) | (Selector, 4) => Some(Trip),
            (Alert, 5) => Some(Selector),
            _ => None,
        }
    }
}

#[derive(Default)]
struct Trip<'a> {
    trip_id: Option<&'a str>,
    start_date: Option<&'a str>,
    relationship: Option<u64>,
}

/// Walk `buf` as a `m`; `fix` sees every TripDescriptor and may return a new trip_id.
/// Returns the rewritten message, or `None` when nothing changed.
fn walk(
    buf: &[u8],
    m: Msg,
    fix: &mut dyn FnMut(&Trip) -> Option<String>,
) -> Result<Option<Vec<u8>>> {
    if m == Msg::Trip {
        return trip(buf, fix);
    }
    let mut out = Vec::new();
    let mut changed = false;
    for f in wire::fields(buf) {
        let f = f?;
        let new = match m.child(f.tag) {
            Some(c) if f.kind == wire::LEN => walk(f.body, c, fix)?,
            _ => None,
        };
        match new {
            Some(body) => {
                changed = true;
                wire::put_len(&mut out, f.tag, &body);
            }
            None => out.extend_from_slice(f.raw),
        }
    }
    Ok(changed.then_some(out))
}

fn trip(buf: &[u8], fix: &mut dyn FnMut(&Trip) -> Option<String>) -> Result<Option<Vec<u8>>> {
    let mut t = Trip::default();
    for f in wire::fields(buf) {
        let f = f?;
        match (f.tag, f.kind) {
            (1, wire::LEN) => t.trip_id = std::str::from_utf8(f.body).ok(),
            (3, wire::LEN) => t.start_date = std::str::from_utf8(f.body).ok(),
            (4, wire::VARINT) => t.relationship = Some(f.value),
            _ => {}
        }
    }
    let Some(new_id) = fix(&t) else {
        return Ok(None);
    };
    let mut out = Vec::with_capacity(buf.len() + 8);
    for f in wire::fields(buf) {
        let f = f?;
        if f.tag == 1 && f.kind == wire::LEN {
            wire::put_len(&mut out, 1, new_id.as_bytes());
        } else {
            out.extend_from_slice(f.raw);
        }
    }
    Ok(Some(out))
}

/// Just enough of the protobuf wire format to copy fields and replace length-delimited ones.
mod wire {
    use anyhow::{bail, Context, Result};

    pub const VARINT: u8 = 0;
    pub const LEN: u8 = 2;

    pub struct Field<'a> {
        pub tag: u32,
        pub kind: u8,
        /// The whole field, key included, to copy it unchanged.
        pub raw: &'a [u8],
        /// A length-delimited field's payload.
        pub body: &'a [u8],
        /// A varint field's value.
        pub value: u64,
    }

    pub fn fields(buf: &[u8]) -> impl Iterator<Item = Result<Field<'_>>> {
        let mut pos = 0;
        std::iter::from_fn(move || {
            if pos >= buf.len() {
                return None;
            }
            let r = field(buf, &mut pos);
            if r.is_err() {
                pos = buf.len();
            }
            Some(r)
        })
    }

    fn field<'a>(buf: &'a [u8], pos: &mut usize) -> Result<Field<'a>> {
        let start = *pos;
        let key = varint(buf, pos)?;
        let (tag, kind) = ((key >> 3) as u32, (key & 7) as u8);
        if tag == 0 {
            bail!("field number 0 at byte {start}");
        }
        let (mut body, mut value) = (&buf[0..0], 0);
        match kind {
            0 => value = varint(buf, pos)?,
            1 => *pos += 8,
            2 => {
                let n = usize::try_from(varint(buf, pos)?)?;
                let end = pos.checked_add(n).context("length overflow")?;
                body = buf.get(*pos..end).context("truncated field")?;
                *pos = end;
            }
            5 => *pos += 4,
            _ => bail!("unsupported wire type {kind} at byte {start}"),
        }
        let raw = buf.get(start..*pos).context("truncated field")?;
        Ok(Field {
            tag,
            kind,
            raw,
            body,
            value,
        })
    }

    fn varint(buf: &[u8], pos: &mut usize) -> Result<u64> {
        let mut v = 0u64;
        for shift in (0..64).step_by(7) {
            let b = *buf.get(*pos).context("truncated varint")?;
            *pos += 1;
            v |= u64::from(b & 0x7f) << shift;
            if b < 0x80 {
                return Ok(v);
            }
        }
        bail!("varint too long")
    }

    pub fn put_varint(out: &mut Vec<u8>, mut v: u64) {
        while v >= 0x80 {
            out.push(v as u8 | 0x80);
            v >>= 7;
        }
        out.push(v as u8);
    }

    pub fn put_len(out: &mut Vec<u8>, tag: u32, body: &[u8]) {
        put_varint(out, u64::from(tag) << 3 | 2);
        put_varint(out, body.len() as u64);
        out.extend_from_slice(body);
    }
}

// --- the proxy -------------------------------------------------------------------------------

pub struct Options {
    pub listen: String,
    pub upstream: String,
    pub gtfs: PathBuf,
}

struct Proxy {
    opts: Options,
    http: reqwest::blocking::Client,
    trips: Option<StaticTrips>,
    gtfs_mtime: Option<SystemTime>,
    cached: Option<(Instant, Vec<u8>, Stats)>,
    logged: Option<Instant>,
}

/// Serve the upstream feed with fixed trip IDs on every GET, until killed.
pub fn serve(opts: Options) -> Result<()> {
    let server = tiny_http::Server::http(&opts.listen)
        .map_err(|e| anyhow!("listening on {}: {e}", opts.listen))?;
    eprintln!(
        "rt-fix: serving {} on http://{} with trip IDs from {}",
        opts.upstream,
        opts.listen,
        opts.gtfs.display()
    );
    let mut p = Proxy {
        http: reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("overstap-rt-fix")
            .build()?,
        opts,
        trips: None,
        gtfs_mtime: None,
        cached: None,
        logged: None,
    };
    for req in server.incoming_requests() {
        let resp = if *req.method() != tiny_http::Method::Get {
            tiny_http::Response::from_string("GET only").with_status_code(405)
        } else {
            match p.feed() {
                Ok((body, stats)) => tiny_http::Response::from_data(body)
                    .with_header(header("Content-Type", "application/x-protobuf"))
                    .with_header(header(
                        "X-Rt-Fix",
                        &format!(
                            "known={} rematched={} unmatched={} added={}",
                            stats.known, stats.rematched, stats.unmatched, stats.added
                        ),
                    )),
                Err(e) => {
                    eprintln!("rt-fix: {e:#}");
                    tiny_http::Response::from_string(format!("{e:#}")).with_status_code(502)
                }
            }
        };
        let _ = req.respond(resp);
    }
    Ok(())
}

fn header(k: &str, v: &str) -> tiny_http::Header {
    tiny_http::Header::from_bytes(k, v).expect("valid header")
}

impl Proxy {
    fn feed(&mut self) -> Result<(Vec<u8>, Stats)> {
        self.reload_static();
        if let Some((at, body, stats)) = &self.cached {
            if at.elapsed() < CACHE {
                return Ok((body.clone(), *stats));
            }
        }
        let raw = self
            .http
            .get(&self.opts.upstream)
            .send()
            .and_then(|r| r.error_for_status())
            .and_then(|r| r.bytes())
            .with_context(|| format!("fetching {}", self.opts.upstream))?
            .to_vec();
        let today = Utc::now()
            .with_timezone(&Brussels)
            .date_naive()
            .num_days_from_ce();
        let (body, stats) = match &self.trips {
            None => (raw, Stats::default()),
            Some(t) => fix_feed(t, &raw, today).unwrap_or_else(|e| {
                eprintln!("rt-fix: feed not rewritten, passed on as is: {e:#}");
                (raw, Stats::default())
            }),
        };
        if self.logged.is_none_or(|t| t.elapsed() >= LOG_EVERY) {
            eprintln!("rt-fix: {stats}");
            self.logged = Some(Instant::now());
        }
        self.cached = Some((Instant::now(), body.clone(), stats));
        Ok((body, stats))
    }

    /// (Re)load the static trips when the GTFS file changed (the nightly update replaces it).
    /// A failed load keeps the trips loaded before; without any, the feed passes unchanged.
    fn reload_static(&mut self) {
        let mtime = std::fs::metadata(&self.opts.gtfs)
            .and_then(|m| m.modified())
            .ok();
        if mtime.is_none() || mtime == self.gtfs_mtime {
            if mtime.is_none() && self.trips.is_none() && self.logged.is_none() {
                eprintln!(
                    "rt-fix: no {} yet: passing the feed on unchanged",
                    self.opts.gtfs.display()
                );
            }
            return;
        }
        self.gtfs_mtime = mtime;
        let t0 = Instant::now();
        match StaticTrips::load(&self.opts.gtfs) {
            Ok(t) => {
                eprintln!(
                    "rt-fix: {} trips from {} in {} ms",
                    t.trip_count(),
                    self.opts.gtfs.display(),
                    t0.elapsed().as_millis()
                );
                self.trips = Some(t);
                self.cached = None;
                self.logged = None;
            }
            Err(e) => eprintln!(
                "rt-fix: can't read {} ({e:#}); keeping the trips read before",
                self.opts.gtfs.display()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::wire::put_len;
    use super::*;
    use crate::gtfs_rt::FeedMessage;
    use prost::Message;

    const TRIPS: &str = "\u{feff}route_id,service_id,trip_id,trip_headsign\n\
        1,s_daily,88____:A72::8821006:8832409:13:825:20270930,Mol\n\
        1,s_daily,88____:007::8821006:8832664:11:842:20261211,Hamont\n\
        1,s_daily,88____:007::8821006:8872009:11:835:20270930,X\n\
        1,s_daily,88____:007::8821006:8872009:11:835:20261207,X\n\
        1,s_never,88____:007::8821006:8872009:11:835:20270101,X\n\
        1,s_weekend,88____:007::1:2:3:900:20270930,Y\n\
        1,s_daily,88____:007::1:2:3:901:20270930,Z\n\
        1,s_daily,88____:007::1:2:3:901:20261201,Z\n";
    const CALENDAR: &str =
        "service_id,monday,tuesday,wednesday,thursday,friday,saturday,sunday,start_date,end_date\n\
        s_daily,0,0,0,0,0,0,0,20261121,20270930\n\
        s_never,0,0,0,0,0,0,0,20261121,20270930\n\
        s_weekend,0,0,0,0,0,1,1,20261121,20270930\n";
    const DATES: &str = "service_id,date,exception_type\n\
        s_daily,20261124,1\n\
        s_daily,20261125,1\n\
        s_never,20261124,2\n";

    fn trips() -> StaticTrips {
        StaticTrips::from_csv(TRIPS, CALENDAR, DATES).unwrap()
    }

    fn today() -> i32 {
        day(b"20261124").unwrap()
    }

    /// A TripDescriptor, plus `extra` raw bytes (fields this code doesn't know).
    fn desc(id: &str, date: &str, rel: Option<u8>, extra: &[u8]) -> Vec<u8> {
        let mut b = Vec::new();
        put_len(&mut b, 1, id.as_bytes());
        put_len(&mut b, 3, date.as_bytes());
        if let Some(r) = rel {
            b.extend([4 << 3, r]);
        }
        b.extend_from_slice(extra);
        b
    }

    /// A FeedMessage with a header and one trip update entity per descriptor; each update
    /// has a stop time update and an unknown field, so both must survive the rewrite.
    fn feed(descs: &[Vec<u8>]) -> Vec<u8> {
        let mut f = Vec::new();
        let mut header = Vec::new();
        put_len(&mut header, 1, b"2.0");
        header.extend([3 << 3, 42]);
        put_len(&mut f, 1, &header);
        for (i, d) in descs.iter().enumerate() {
            let mut stu = Vec::new();
            put_len(&mut stu, 4, b"8821121");
            let mut dep = vec![1 << 3, 0xac, 0x02]; // delay 300 (varint)
            dep.extend([2 << 3, 1]);
            put_len(&mut stu, 3, &dep);
            let mut tu = Vec::new();
            put_len(&mut tu, 1, d);
            put_len(&mut tu, 2, &stu);
            tu.extend([0xb8, 0x06, 7]); // field 103, varint 7: an extension this code doesn't know
            let mut e = Vec::new();
            put_len(&mut e, 1, format!("e{i}").as_bytes());
            put_len(&mut e, 3, &tu);
            put_len(&mut f, 2, &e);
        }
        f
    }

    fn ids(feed: &[u8]) -> Vec<String> {
        FeedMessage::decode(feed)
            .unwrap()
            .entity
            .into_iter()
            .map(|e| e.trip_update.unwrap().trip.unwrap().trip_id.unwrap())
            .collect()
    }

    #[test]
    fn rematches_clipped_end_dates() {
        let f = feed(&[
            desc(
                "88____:A72::8821006:8832409:13:825:20271210",
                "20261124",
                Some(0),
                &[],
            ),
            desc(
                "88____:007::8821006:8832664:11:842:20261211",
                "20261124",
                None,
                &[],
            ),
        ]);
        let (out, stats) = fix_feed(&trips(), &f, today()).unwrap();
        assert_eq!(
            ids(&out),
            [
                "88____:A72::8821006:8832409:13:825:20270930",
                "88____:007::8821006:8832664:11:842:20261211"
            ]
        );
        assert_eq!(
            (stats.trips, stats.known, stats.rematched, stats.unmatched),
            (2, 1, 1, 0)
        );
        // Everything but the trip_id is untouched: delays, the unknown field, the header.
        let m = FeedMessage::decode(&out[..]).unwrap();
        let tu = m.entity[0].trip_update.as_ref().unwrap();
        assert_eq!(
            tu.stop_time_update[0].departure.as_ref().unwrap().delay,
            Some(300)
        );
        assert_eq!(m.header.unwrap().timestamp, Some(42));
        assert_eq!(out.windows(3).filter(|w| w == &[0xb8, 0x06, 7]).count(), 2);
        assert_eq!(out.len(), f.len()); // same-length suffix: only those 8 bytes differ
    }

    #[test]
    fn unchanged_feed_is_byte_identical() {
        let f = feed(&[desc(
            "88____:007::8821006:8832664:11:842:20261211",
            "20261124",
            None,
            &[0x30, 1], // direction_id 1
        )]);
        let (out, stats) = fix_feed(&trips(), &f, today()).unwrap();
        assert_eq!(out, f);
        assert_eq!(stats.known, 1);
    }

    #[test]
    fn unknown_descriptor_fields_survive() {
        let f = feed(&[desc(
            "88____:A72::8821006:8832409:13:825:20271210",
            "20261124",
            None,
            &[0x30, 1],
        )]);
        let (out, _) = fix_feed(&trips(), &f, today()).unwrap();
        let m = FeedMessage::decode(&out[..]).unwrap();
        let t = m.entity[0]
            .trip_update
            .as_ref()
            .unwrap()
            .trip
            .as_ref()
            .unwrap();
        assert_eq!(t.start_date.as_deref(), Some("20261124"));
        assert!(out.windows(2).any(|w| w == [0x30, 1]));
    }

    #[test]
    fn a_twin_with_its_own_update_is_not_taken() {
        // Static has …:835:20270930 and …:835:20261207, both running today; the feed has
        // …:835:20261207 itself and …:835:20271210 → the latter can only be …:20270930.
        // (…:835:20270101 has the same prefix but doesn't run today.)
        let f = feed(&[
            desc(
                "88____:007::8821006:8872009:11:835:20271210",
                "20261124",
                None,
                &[],
            ),
            desc(
                "88____:007::8821006:8872009:11:835:20261207",
                "20261124",
                None,
                &[],
            ),
        ]);
        let (out, stats) = fix_feed(&trips(), &f, today()).unwrap();
        assert_eq!(
            ids(&out),
            [
                "88____:007::8821006:8872009:11:835:20270930",
                "88____:007::8821006:8872009:11:835:20261207"
            ]
        );
        assert_eq!(stats.rematched, 1);
    }

    #[test]
    fn closest_end_date_breaks_ties() {
        let f = feed(&[
            desc("88____:007::1:2:3:901:20271210", "20261124", None, &[]),
            desc("88____:007::1:2:3:901:20261202", "20261124", None, &[]),
        ]);
        let (out, _) = fix_feed(&trips(), &f, today()).unwrap();
        assert_eq!(
            ids(&out),
            [
                "88____:007::1:2:3:901:20270930",
                "88____:007::1:2:3:901:20261201"
            ]
        );
    }

    #[test]
    fn leaves_what_does_not_fit() {
        let f = feed(&[
            // Runs on weekends only; 2026-11-24 is a Tuesday.
            desc("88____:007::1:2:3:900:20271210", "20261124", None, &[]),
            // …but 2026-11-28 is a Saturday (calendar.txt, no exception).
            desc("88____:007::1:2:3:900:20271210", "20261128", None, &[]),
            // No such train.
            desc("88____:007::9:9:9:999:20271210", "20261124", None, &[]),
            // Added: not in the static feed on purpose.
            desc(
                "88____:A72::8821006:8832409:13:825:20271210",
                "20261124",
                Some(1),
                &[],
            ),
        ]);
        let (out, stats) = fix_feed(&trips(), &f, today()).unwrap();
        let got = ids(&out);
        assert_eq!(got[0], "88____:007::1:2:3:900:20271210");
        assert_eq!(got[1], "88____:007::1:2:3:900:20270930");
        assert_eq!(got[2], "88____:007::9:9:9:999:20271210");
        assert_eq!(got[3], "88____:A72::8821006:8832409:13:825:20271210");
        assert_eq!(
            (stats.trips, stats.rematched, stats.unmatched, stats.added),
            (4, 1, 2, 1)
        );
    }

    #[test]
    fn missing_start_date_means_today() {
        let mut d = Vec::new();
        put_len(&mut d, 1, b"88____:A72::8821006:8832409:13:825:20271210");
        let (out, _) = fix_feed(&trips(), &feed(&[d]), today()).unwrap();
        assert_eq!(ids(&out), ["88____:A72::8821006:8832409:13:825:20270930"]);
    }

    #[test]
    fn rejects_garbage() {
        assert!(fix_feed(&trips(), &[0x12, 0x50, 0x01], today()).is_err());
        assert!(fix_feed(&trips(), &[0x0b], today()).is_err()); // group wire type
    }
}
