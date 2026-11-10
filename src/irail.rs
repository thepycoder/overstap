//! NMBS platforms from iRail (api.irail.be, ADR-022).
//!
//! The NMBS GTFS has no tracks, and MOTIS knows one only once GTFS-RT assigns it, minutes
//! before departure. iRail has each train's platform at every stop well ahead (NMBS's own
//! data, changes included), so `/api/live` fills the tracks MOTIS lacks from it. One request
//! per train, paced and cached; any failure just leaves the tracks empty.

use crate::motis::{Leg, Place};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use chrono_tz::Europe::Brussels;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

const UPSTREAM: &str = "https://api.irail.be/v1";
const USER_AGENT: &str = "overstap (personal journey planner)";
/// iRail allows 3 requests a second.
const MIN_INTERVAL: Duration = Duration::from_millis(350);
const TTL: Duration = Duration::from_secs(5 * 60);
/// Platforms are only worth asking for trains you could still be catching soon.
const AHEAD: chrono::TimeDelta = chrono::TimeDelta::hours(2);

static IRAIL: LazyLock<Irail> = LazyLock::new(Irail::new);

struct Irail {
    http: reqwest::blocking::Client,
    last: Mutex<Option<Instant>>,
    /// "number date" → (fetched at, platform per stop); `None` when iRail didn't know the train.
    cache: Mutex<HashMap<String, (Instant, Stops)>>,
}

type Stops = Option<Vec<Stop>>;

#[derive(Clone)]
struct Stop {
    uic: u64,
    /// Scheduled time at the stop (departure, arrival at the last one).
    time: i64,
    platform: String,
}

/// Fill the tracks MOTIS doesn't have on an NMBS trip (a whole trip from `/api/v6/trip`).
pub fn fill_tracks(leg: &mut Leg, now: DateTime<Utc>) {
    let Some(number) = leg
        .trip_short_name
        .clone()
        .filter(|n| n.bytes().all(|b| b.is_ascii_digit()))
    else {
        return;
    };
    if uic(&leg.from).is_none()
        || leg.scheduled_end_time < now
        || leg.scheduled_start_time > now + AHEAD
    {
        return;
    }
    let places = || {
        std::iter::once(&leg.from)
            .chain(&leg.intermediate_stops)
            .chain(std::iter::once(&leg.to))
    };
    if places().all(|p| p.track.is_some()) {
        return;
    }
    let date = leg
        .scheduled_start_time
        .with_timezone(&Brussels)
        .format("%d%m%y")
        .to_string();
    let stops = match IRAIL.vehicle(&number, &date) {
        Ok(Some(s)) => s,
        Ok(None) => return,
        Err(e) => return eprintln!("  live: iRail train {number}: {e:#}"),
    };
    let fill = |p: &mut Place| {
        let (Some(id), Some(t)) = (uic(p), p.scheduled_departure.or(p.scheduled_arrival)) else {
            return;
        };
        if p.track.is_some() {
            return;
        }
        // Nearest in time: a train can call at the same station twice.
        let hit = stops
            .iter()
            .filter(|s| s.uic == id && (s.time - t.timestamp()).abs() <= 30 * 60)
            .min_by_key(|s| (s.time - t.timestamp()).abs());
        if let Some(s) = hit {
            p.track = Some(s.platform.clone());
        }
    };
    fill(&mut leg.from);
    leg.intermediate_stops.iter_mut().for_each(fill);
    fill(&mut leg.to);
}

/// The station's UIC code from an NMBS stop id (`sncb_8894201`, or a platform's `sncb_8894201_3`).
fn uic(p: &Place) -> Option<u64> {
    let rest = p.stop_id.as_deref()?.strip_prefix("sncb_")?;
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

impl Irail {
    fn new() -> Self {
        Self {
            http: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(8))
                .user_agent(USER_AGENT)
                .build()
                .expect("http client"),
            last: Mutex::new(None),
            cache: Mutex::new(HashMap::new()),
        }
    }

    fn vehicle(&self, number: &str, date: &str) -> Result<Stops> {
        let key = format!("{number} {date}");
        if let Some((at, v)) = self.cache.lock().unwrap().get(&key) {
            if at.elapsed() < TTL {
                return Ok(v.clone());
            }
        }
        {
            let mut last = self.last.lock().unwrap();
            if let Some(t) = *last {
                std::thread::sleep(MIN_INTERVAL.saturating_sub(t.elapsed()));
            }
            *last = Some(Instant::now());
        }
        let r = self
            .http
            .get(format!("{UPSTREAM}/vehicle/"))
            .query(&[
                ("id", number),
                ("date", date),
                ("format", "json"),
                ("lang", "en"),
            ])
            .send()
            .context("iRail vehicle")?;
        // iRail answers 404 for a train it doesn't run that day; remember that too.
        let stops = if r.status() == reqwest::StatusCode::NOT_FOUND {
            None
        } else {
            Some(parse(&r.error_for_status()?.json()?))
        };
        self.cache
            .lock()
            .unwrap()
            .insert(key, (Instant::now(), stops.clone()));
        Ok(stops)
    }
}

/// iRail's stops: `{"stops": {"stop": [{"stationinfo": {"id": "BE.NMBS.008894201"},
/// "time": "1794291900", "platform": "1"}, …]}}`, all values strings; "?" = not known yet.
fn parse(v: &Value) -> Vec<Stop> {
    let stops = v["stops"]["stop"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default();
    stops
        .iter()
        .filter_map(|s| {
            let platform = s["platform"]
                .as_str()
                .filter(|p| !p.is_empty() && *p != "?")?;
            Some(Stop {
                uic: s["stationinfo"]["id"]
                    .as_str()?
                    .strip_prefix("BE.NMBS.")?
                    .parse()
                    .ok()?,
                time: s["time"].as_str()?.parse().ok()?,
                platform: platform.to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_platforms_and_skips_unknown() {
        let v = json!({ "stops": { "stop": [
            { "stationinfo": { "id": "BE.NMBS.008894201" }, "time": "1794291900", "platform": "1" },
            { "stationinfo": { "id": "BE.NMBS.008893120" }, "time": "1794292860", "platform": "?" },
        ]}});
        let s = parse(&v);
        assert_eq!(s.len(), 1);
        assert_eq!(
            (s[0].uic, s[0].time, s[0].platform.as_str()),
            (8894201, 1794291900, "1")
        );
    }

    #[test]
    fn uic_from_stop_ids() {
        let p = |id: &str| Place {
            stop_id: Some(id.into()),
            ..Default::default()
        };
        assert_eq!(uic(&p("sncb_8894201")), Some(8894201));
        assert_eq!(uic(&p("sncb_8894201_3")), Some(8894201));
        assert_eq!(uic(&p("delijn_123")), None);
    }
}
