//! GTFS-RT logger (ADR-006): poll NMBS and De Lijn trip updates and keep their history,
//! for delay distributions and "will this change hold?" scoring later.
//!
//! Storage (SQLite, see `SCHEMA`): one row per *change* in a stop's prediction, not per
//! fetch. NMBS sends ~4k stop predictions a minute that rarely change; De Lijn sends 80k
//! stop entries of which ~90 % are NO_DATA (skipped here). A row is written when a stop
//! first appears, when its status changes, or when a delay moves by `min_change_s` or more.
//! The observation time is the fetch's (`fetches.fetched_at`). Raw feeds can also be kept,
//! gzipped, every `raw_every_min` minutes, so everything can be re-parsed later.

use crate::gtfs_rt::{FeedMessage, StopTimeEvent, STOP_NO_DATA, STOP_SKIPPED};
use anyhow::{Context, Result};
use chrono::{Local, Utc};
use prost::Message;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub const FEEDS: &[(&str, &str)] = &[
    (
        "sncb",
        "https://sncb-opendata.hafas.de/gtfs/realtime/c21ac6758dd25af84cca5b707f3cb3de",
    ),
    (
        "delijn",
        "https://api-management-opendata-production.azure-api.net/api/gtfs/feed/delijn/rt/trip-update?format=protobuf",
    ),
];

const SCHEMA: &str = "
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
CREATE TABLE IF NOT EXISTS fetches (
  id INTEGER PRIMARY KEY, feed TEXT NOT NULL, fetched_at INTEGER NOT NULL,
  header_ts INTEGER, bytes INTEGER, entities INTEGER, rows INTEGER, error TEXT);
CREATE TABLE IF NOT EXISTS trips (
  id INTEGER PRIMARY KEY, feed TEXT NOT NULL, trip_id TEXT NOT NULL, start_date TEXT NOT NULL,
  start_time TEXT, route_id TEXT, first_seen INTEGER NOT NULL, last_seen INTEGER NOT NULL,
  UNIQUE (feed, trip_id, start_date));
CREATE TABLE IF NOT EXISTS stops (
  id INTEGER PRIMARY KEY, feed TEXT NOT NULL, stop_id TEXT NOT NULL, UNIQUE (feed, stop_id));
-- Trip status changes: 0 scheduled, 1 added, 3 canceled, ... (GTFS-RT TripDescriptor).
CREATE TABLE IF NOT EXISTS trip_events (
  trip INTEGER NOT NULL REFERENCES trips, fetch INTEGER NOT NULL REFERENCES fetches, rel INTEGER NOT NULL);
-- Stop prediction changes. Delays in seconds, times in Unix seconds; rel 0 scheduled, 1 skipped.
CREATE TABLE IF NOT EXISTS stop_events (
  trip INTEGER NOT NULL REFERENCES trips, stop INTEGER NOT NULL REFERENCES stops, seq INTEGER,
  fetch INTEGER NOT NULL REFERENCES fetches, rel INTEGER NOT NULL,
  arr_delay INTEGER, arr_time INTEGER, dep_delay INTEGER, dep_time INTEGER);
CREATE INDEX IF NOT EXISTS stop_events_trip ON stop_events (trip, stop);
CREATE INDEX IF NOT EXISTS trip_events_trip ON trip_events (trip);
";

pub struct Options {
    pub db: PathBuf,
    pub interval_s: u64,
    pub raw_dir: Option<PathBuf>,
    pub raw_every_min: i64,
    pub raw_keep_days: u64,
    pub min_change_s: i64,
    pub once: bool,
}

#[derive(Clone, Copy, PartialEq)]
struct Pred {
    rel: i32,
    arr_delay: Option<i64>,
    arr_time: Option<i64>,
    dep_delay: Option<i64>,
    dep_time: Option<i64>,
}

impl Pred {
    fn new(rel: i32, arr: Option<&StopTimeEvent>, dep: Option<&StopTimeEvent>) -> Self {
        Self {
            rel,
            arr_delay: arr.and_then(|e| e.delay).map(i64::from),
            arr_time: arr.and_then(|e| e.time),
            dep_delay: dep.and_then(|e| e.delay).map(i64::from),
            dep_time: dep.and_then(|e| e.time),
        }
    }

    /// Worth a new row compared with `old`? Small drifts (bus GPS) are not.
    fn differs(&self, old: &Pred, min: i64) -> bool {
        let moved = |a: Option<i64>, b: Option<i64>| match (a, b) {
            (Some(a), Some(b)) => (a - b).abs() >= min,
            (None, None) => false,
            _ => true,
        };
        self.rel != old.rel
            || moved(self.arr_delay, old.arr_delay)
            || moved(self.dep_delay, old.dep_delay)
            || (self.arr_delay.is_none() && moved(self.arr_time, old.arr_time))
            || (self.dep_delay.is_none() && moved(self.dep_time, old.dep_time))
    }
}

/// What a poll did.
pub struct PollStats {
    pub bytes: usize,
    pub entities: usize,
    pub rows: usize,
    pub unchanged_feed: bool,
}

pub struct Logger {
    conn: Connection,
    http: reqwest::blocking::Client,
    opts: Options,
    trip_ids: HashMap<(String, String, String), i64>,
    stop_ids: HashMap<(String, String), i64>,
    last_trip: HashMap<i64, i32>,
    last_stop: HashMap<(i64, i64, u32), Pred>,
    last_header: HashMap<String, u64>,
    last_raw: HashMap<String, i64>,
    caches_day: String,
}

impl Logger {
    pub fn open(opts: Options) -> Result<Self> {
        let conn =
            Connection::open(&opts.db).with_context(|| format!("opening {}", opts.db.display()))?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self {
            conn,
            http: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(60))
                .user_agent("overstap-rt-log")
                .build()?,
            opts,
            trip_ids: HashMap::new(),
            stop_ids: HashMap::new(),
            last_trip: HashMap::new(),
            last_stop: HashMap::new(),
            last_header: HashMap::new(),
            last_raw: HashMap::new(),
            caches_day: String::new(),
        })
    }

    /// Fetch one feed and store what changed. Errors are recorded in `fetches` too.
    pub fn poll(&mut self, feed: &str, url: &str) -> Result<PollStats> {
        let now = Utc::now().timestamp();
        let body = match self.fetch(url) {
            Ok(b) => b,
            Err(e) => {
                self.conn.execute(
                    "INSERT INTO fetches (feed, fetched_at, error) VALUES (?1, ?2, ?3)",
                    params![feed, now, format!("{e:#}")],
                )?;
                return Err(e);
            }
        };
        self.ingest(feed, now, &body)
    }

    /// Store what changed in one fetched feed (`body` = GTFS-RT protobuf).
    pub fn ingest(&mut self, feed: &str, now: i64, body: &[u8]) -> Result<PollStats> {
        let msg = FeedMessage::decode(body).context("decoding GTFS-RT")?;
        let header_ts = msg.header.as_ref().and_then(|h| h.timestamp);
        self.maybe_store_raw(feed, now, body)?;
        // Many feeds regenerate less often than we poll: nothing to do then.
        if header_ts.is_some() && self.last_header.get(feed) == header_ts.as_ref() {
            return Ok(PollStats {
                bytes: body.len(),
                entities: msg.entity.len(),
                rows: 0,
                unchanged_feed: true,
            });
        }
        if let Some(ts) = header_ts {
            self.last_header.insert(feed.to_string(), ts);
        }
        self.reset_caches_daily();

        let min = self.opts.min_change_s;
        let today = Local::now().format("%Y%m%d").to_string();
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO fetches (feed, fetched_at, header_ts, bytes, entities) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![feed, now, header_ts.map(|t| t as i64), body.len() as i64, msg.entity.len() as i64],
        )?;
        let fetch = tx.last_insert_rowid();
        let mut rows = 0usize;
        {
            let mut trip_upsert = tx.prepare_cached(
                "INSERT INTO trips (feed, trip_id, start_date, start_time, route_id, first_seen, last_seen)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)
                 ON CONFLICT (feed, trip_id, start_date) DO UPDATE SET last_seen = excluded.last_seen
                 RETURNING id",
            )?;
            let mut stop_upsert = tx.prepare_cached(
                "INSERT INTO stops (feed, stop_id) VALUES (?1, ?2)
                 ON CONFLICT (feed, stop_id) DO UPDATE SET stop_id = excluded.stop_id RETURNING id",
            )?;
            let mut trip_event = tx
                .prepare_cached("INSERT INTO trip_events (trip, fetch, rel) VALUES (?1, ?2, ?3)")?;
            let mut stop_event = tx.prepare_cached(
                "INSERT INTO stop_events (trip, stop, seq, fetch, rel, arr_delay, arr_time, dep_delay, dep_time)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            )?;
            for ent in &msg.entity {
                let Some(tu) = &ent.trip_update else { continue };
                let Some(td) = &tu.trip else { continue };
                let Some(trip_id) = td.trip_id.clone() else {
                    continue;
                };
                let date = td.start_date.clone().unwrap_or_else(|| today.clone());
                // Upsert every time: keeps last_seen current (a trip that vanishes has ended).
                let trip: i64 = trip_upsert.query_row(
                    params![feed, trip_id, date, td.start_time, td.route_id, now],
                    |r| r.get(0),
                )?;
                self.trip_ids.insert((feed.into(), trip_id, date), trip);
                let rel = td.schedule_relationship.unwrap_or(0);
                if self.last_trip.insert(trip, rel) != Some(rel) {
                    trip_event.execute(params![trip, fetch, rel])?;
                    rows += 1;
                }
                for stu in &tu.stop_time_update {
                    let srel = stu.schedule_relationship.unwrap_or(0);
                    if srel == STOP_NO_DATA {
                        continue;
                    }
                    let pred = Pred::new(srel, stu.arrival.as_ref(), stu.departure.as_ref());
                    if srel != STOP_SKIPPED
                        && pred.arr_delay.or(pred.arr_time).is_none()
                        && pred.dep_delay.or(pred.dep_time).is_none()
                    {
                        continue;
                    }
                    let Some(stop_id) = stu.stop_id.clone() else {
                        continue;
                    };
                    let stop = match self.stop_ids.get(&(feed.to_string(), stop_id.clone())) {
                        Some(id) => *id,
                        None => {
                            let id: i64 =
                                stop_upsert.query_row(params![feed, stop_id], |r| r.get(0))?;
                            self.stop_ids.insert((feed.into(), stop_id), id);
                            id
                        }
                    };
                    let key = (trip, stop, stu.stop_sequence.unwrap_or(0));
                    if self
                        .last_stop
                        .get(&key)
                        .is_some_and(|old| !pred.differs(old, min))
                    {
                        continue;
                    }
                    self.last_stop.insert(key, pred);
                    stop_event.execute(params![
                        trip,
                        stop,
                        stu.stop_sequence,
                        fetch,
                        srel,
                        pred.arr_delay,
                        pred.arr_time,
                        pred.dep_delay,
                        pred.dep_time
                    ])?;
                    rows += 1;
                }
            }
        }
        tx.execute(
            "UPDATE fetches SET rows = ?1 WHERE id = ?2",
            params![rows as i64, fetch],
        )?;
        tx.commit()?;
        Ok(PollStats {
            bytes: body.len(),
            entities: msg.entity.len(),
            rows,
            unchanged_feed: false,
        })
    }

    fn fetch(&self, url: &str) -> Result<Vec<u8>> {
        let r = self.http.get(url).send()?.error_for_status()?;
        Ok(r.bytes()?.to_vec())
    }

    /// Caches only decide what's new; dropping them once a day (after 04:00) bounds memory.
    /// The first poll after that writes one full snapshot again, which is harmless.
    fn reset_caches_daily(&mut self) {
        let now = Local::now();
        let day = now.format("%Y%m%d").to_string();
        if day != self.caches_day && now.format("%H").to_string().as_str() >= "04" {
            if !self.caches_day.is_empty() {
                self.trip_ids.clear();
                self.stop_ids.clear();
                self.last_trip.clear();
                self.last_stop.clear();
                self.prune_raw();
            }
            self.caches_day = day;
        }
    }

    fn maybe_store_raw(&mut self, feed: &str, now: i64, body: &[u8]) -> Result<()> {
        let Some(dir) = &self.opts.raw_dir else {
            return Ok(());
        };
        if self.opts.raw_every_min <= 0 {
            return Ok(());
        }
        if self
            .last_raw
            .get(feed)
            .is_some_and(|t| now - t < self.opts.raw_every_min * 60 - 5)
        {
            return Ok(());
        }
        let t = Utc::now();
        let day_dir = dir.join(feed).join(t.format("%Y-%m-%d").to_string());
        std::fs::create_dir_all(&day_dir)?;
        let path = day_dir.join(format!("{}.pb.gz", t.format("%H%M%SZ")));
        let mut gz = flate2::write::GzEncoder::new(
            std::fs::File::create(&path)?,
            flate2::Compression::default(),
        );
        gz.write_all(body)?;
        gz.finish()?;
        self.last_raw.insert(feed.to_string(), now);
        Ok(())
    }

    /// Delete raw day folders older than `raw_keep_days`.
    fn prune_raw(&self) {
        let Some(dir) = &self.opts.raw_dir else {
            return;
        };
        let cutoff = (Utc::now() - chrono::Duration::days(self.opts.raw_keep_days as i64))
            .format("%Y-%m-%d")
            .to_string();
        for feed in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            for day in std::fs::read_dir(feed.path())
                .into_iter()
                .flatten()
                .flatten()
            {
                if day.file_name().to_string_lossy().as_ref() < cutoff.as_str() {
                    let _ = std::fs::remove_dir_all(day.path());
                }
            }
        }
    }
}

pub fn run(opts: Options) -> Result<()> {
    let interval = Duration::from_secs(opts.interval_s.max(15));
    let once = opts.once;
    let mut log = Logger::open(opts)?;
    loop {
        let t0 = Instant::now();
        for (feed, url) in FEEDS {
            let t = Instant::now();
            match log.poll(feed, url) {
                Ok(s) if s.unchanged_feed => eprintln!("{feed}: unchanged"),
                Ok(s) => eprintln!(
                    "{feed}: {} KB, {} trips, {} rows ({} ms)",
                    s.bytes / 1024,
                    s.entities,
                    s.rows,
                    t.elapsed().as_millis()
                ),
                Err(e) => eprintln!("{feed}: ERROR {e:#}"),
            }
        }
        if once {
            return Ok(());
        }
        std::thread::sleep(interval.saturating_sub(t0.elapsed()));
    }
}

/// Short summary of a logger database: coverage, volume, and latest-known delays.
pub fn report(db: &Path) -> Result<()> {
    let conn = Connection::open(db)?;
    let size = std::fs::metadata(db).map(|m| m.len()).unwrap_or(0);
    println!("{}: {:.1} MB", db.display(), size as f64 / 1e6);
    for (feed, _) in FEEDS {
        let (n, errors, first, last): (i64, i64, Option<i64>, Option<i64>) = conn.query_row(
            "SELECT COUNT(*), COUNT(error), MIN(fetched_at), MAX(fetched_at) FROM fetches WHERE feed = ?1",
            [feed],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )?;
        let (trips, events): (i64, i64) = conn.query_row(
            "SELECT (SELECT COUNT(*) FROM trips WHERE feed = ?1),
                    (SELECT COUNT(*) FROM stop_events e JOIN trips t ON t.id = e.trip WHERE t.feed = ?1)",
            [feed],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let cancelled: i64 = conn.query_row(
            "SELECT COUNT(DISTINCT e.trip) FROM trip_events e JOIN trips t ON t.id = e.trip
             WHERE t.feed = ?1 AND e.rel = 3",
            [feed],
            |r| r.get(0),
        )?;
        let span_h = match (first, last) {
            (Some(a), Some(b)) => (b - a) as f64 / 3600.0,
            _ => 0.0,
        };
        println!(
            "\n{feed}: {n} fetches over {span_h:.1} h ({errors} errors), {trips} trips, {events} stop rows \
             ({:.0}/h), {cancelled} cancelled trips",
            if span_h > 0.0 { events as f64 / span_h } else { 0.0 }
        );
        // Latest known departure delay per (trip, stop): for passed stops, the final one.
        let mut stmt = conn.prepare(
            "WITH last AS (SELECT MAX(e.rowid) AS r FROM stop_events e JOIN trips t ON t.id = e.trip
                           WHERE t.feed = ?1 GROUP BY e.trip, e.stop, e.seq)
             SELECT e.dep_delay FROM last JOIN stop_events e ON e.rowid = last.r
             WHERE e.dep_delay IS NOT NULL AND e.dep_time < ?2",
        )?;
        let mut delays: Vec<i64> = stmt
            .query_map(params![feed, Utc::now().timestamp()], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        if delays.is_empty() {
            continue;
        }
        delays.sort_unstable();
        let q = |p: f64| delays[((delays.len() - 1) as f64 * p) as usize] as f64 / 60.0;
        let late = |m: i64| {
            delays.iter().filter(|d| **d >= m * 60).count() as f64 / delays.len() as f64 * 100.0
        };
        println!(
            "  departures already made: {} · delay p50 {:.1}′ p80 {:.1}′ p95 {:.1}′ max {:.0}′ · ≥3′ {:.0} % · ≥5′ {:.0} %",
            delays.len(),
            q(0.5),
            q(0.8),
            q(0.95),
            q(1.0),
            late(3),
            late(5)
        );
    }
    let raw: Option<i64> = conn
        .query_row("SELECT SUM(bytes) FROM fetches", [], |r| r.get(0))
        .optional()?
        .flatten();
    if let Some(b) = raw {
        println!("\nraw feed bytes fetched: {:.0} MB", b as f64 / 1e6);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gtfs_rt::*;

    fn feed(ts: u64, delay: i32, cancelled: bool) -> Vec<u8> {
        let stu = |seq: u32, d: Option<i32>, rel: i32| StopTimeUpdate {
            stop_sequence: Some(seq),
            stop_id: Some(format!("s{seq}")),
            departure: d.map(|d| StopTimeEvent {
                delay: Some(d),
                time: Some(1_800_000_000 + d as i64),
            }),
            arrival: None,
            schedule_relationship: Some(rel),
        };
        FeedMessage {
            header: Some(FeedHeader {
                gtfs_realtime_version: Some("2.0".into()),
                timestamp: Some(ts),
            }),
            entity: vec![FeedEntity {
                id: Some("e1".into()),
                is_deleted: None,
                trip_update: Some(TripUpdate {
                    trip: Some(TripDescriptor {
                        trip_id: Some("t1".into()),
                        start_date: Some("20261105".into()),
                        schedule_relationship: Some(if cancelled { 3 } else { 0 }),
                        ..Default::default()
                    }),
                    stop_time_update: vec![stu(1, Some(delay), 0), stu(2, None, STOP_NO_DATA)],
                    ..Default::default()
                }),
            }],
        }
        .encode_to_vec()
    }

    fn count(l: &Logger, table: &str) -> i64 {
        l.conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn stores_only_changes() {
        let mut l = Logger::open(Options {
            db: ":memory:".into(),
            interval_s: 60,
            raw_dir: None,
            raw_every_min: 0,
            raw_keep_days: 1,
            min_change_s: 60,
            once: true,
        })
        .unwrap();
        // First sight: one trip event, one stop row (NO_DATA is skipped).
        l.ingest("x", 1, &feed(100, 120, false)).unwrap();
        assert_eq!((count(&l, "trip_events"), count(&l, "stop_events")), (1, 1));
        // Same header: feed not regenerated, nothing to do.
        assert!(
            l.ingest("x", 2, &feed(100, 300, false))
                .unwrap()
                .unchanged_feed
        );
        // 30 s drift: below the threshold.
        l.ingest("x", 3, &feed(101, 150, false)).unwrap();
        assert_eq!(count(&l, "stop_events"), 1);
        // A real change and a cancellation.
        l.ingest("x", 4, &feed(102, 240, true)).unwrap();
        assert_eq!((count(&l, "trip_events"), count(&l, "stop_events")), (2, 2));
        let last: i64 = l
            .conn
            .query_row(
                "SELECT dep_delay FROM stop_events ORDER BY rowid DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(last, 240);
    }
}
