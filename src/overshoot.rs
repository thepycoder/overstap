//! Use case 2 — "I stayed on the train too long".
//!
//! MOTIS has no "origin = aboard trip X" query, so we emulate it: fetch the
//! trip's remaining stops, then for each of the next N stops ask MOTIS for the
//! best continuation departing at the (real-time) arrival there, and rank.
//! This is the brute-force orchestration pattern from ADR-003.

use crate::motis::{Client, Geometry, Itinerary, Leg, Place, PlanQuery};
use crate::render::{hm, legs, summary};
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TripResponse {
    legs: Vec<TripLeg>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TripLeg {
    to: Place,
    #[serde(default)]
    intermediate_stops: Vec<Place>,
    #[serde(default)]
    display_name: Option<String>,
}

/// One alighting option and the best continuation from it.
pub struct Option_ {
    pub alight: Place,
    pub alight_at: DateTime<Utc>,
    pub best: Option<Itinerary>,
}

impl Option_ {
    pub fn arrival(&self) -> Option<DateTime<Utc>> {
        self.best.as_ref().map(|it| it.end_time)
    }
}

fn fetch_trip(c: &Client, trip_id: &str) -> Result<TripLeg> {
    let url = format!("{}/api/v6/trip", c.base_url());
    let r: TripResponse = reqwest::blocking::Client::new()
        .get(url)
        .query(&[
            ("tripId", trip_id),
            ("detailedLegs", "false"),
            ("language", "nl"),
        ])
        .send()?
        .error_for_status()
        .context("MOTIS /trip")?
        .json()?;
    r.legs.into_iter().next().context("trip has no legs")
}

/// Remaining stops strictly after `now` (by real-time arrival), in order.
fn downstream(trip: &TripLeg, now: DateTime<Utc>) -> Vec<(Place, DateTime<Utc>)> {
    trip.intermediate_stops
        .iter()
        .chain(std::iter::once(&trip.to))
        .filter_map(|p| p.arrival.map(|a| (p.clone(), a)))
        .filter(|(_, a)| *a > now)
        .collect()
}

pub fn evaluate(
    c: &Client,
    trip_id: &str,
    dest: &str,
    next: usize,
    now: DateTime<Utc>,
    folding: bool,
    delay_min: i64,
) -> Result<(String, Vec<Option_>)> {
    let trip = fetch_trip(c, trip_id)?;
    let name = trip
        .display_name
        .clone()
        .unwrap_or_else(|| trip_id.to_string());
    let stops = downstream(&trip, now);
    if stops.is_empty() {
        bail!("no downstream stops after {} on {name}", hm(now));
    }
    let mut out = vec![];
    for (stop, arr) in stops.into_iter().take(next) {
        // Simulated extra delay on top of whatever real time MOTIS already has.
        let arr = arr + Duration::minutes(delay_min);
        let from = stop
            .stop_id
            .clone()
            .unwrap_or(format!("{},{}", stop.lat, stop.lon));
        let mut q = PlanQuery::new(from, dest.to_string());
        // Earliest we can be on the platform again; MOTIS adds the transfer time
        // for the first boarding itself when starting at a stop.
        q.time = Some(arr);
        q.search_window_s = 3 * 3600;
        q.direct_modes = vec!["WALK".into()];
        if folding {
            q.direct_modes.push("BIKE".into());
            q.post_transit_modes = vec!["BIKE".into()];
            q.require_bike_transport = true;
            q.max_direct_s = Some(5400);
        }
        let r = c.plan(&q)?;
        let best = r
            .itineraries
            .into_iter()
            .chain(r.direct)
            .min_by_key(|i| i.end_time);
        out.push(Option_ {
            alight: stop,
            alight_at: arr,
            best,
        });
    }
    Ok((name, out))
}

pub fn run(
    c: &Client,
    trip_id: &str,
    dest: &str,
    next: usize,
    now: DateTime<Utc>,
    folding: bool,
    delay_min: i64,
) -> Result<()> {
    let (name, opts) = evaluate(c, trip_id, dest, next, now, folding, delay_min)?;
    let d = if delay_min != 0 {
        format!(", assuming +{delay_min} min delay")
    } else {
        String::new()
    };
    println!("On {name}, now {}{d}. Options by final arrival:", hm(now));
    let mut ranked: Vec<&Option_> = opts.iter().collect();
    ranked.sort_by_key(|o| o.arrival().unwrap_or(now + Duration::days(2)));
    for (i, o) in ranked.iter().enumerate() {
        match &o.best {
            Some(it) => {
                println!(
                    "\n{}. Get off at {} ({}) → arrive {}",
                    i + 1,
                    o.alight.name,
                    hm(o.alight_at),
                    hm(it.end_time)
                );
                println!("   {}", summary(it));
                print!("{}", legs(it));
            }
            None => println!(
                "\n{}. Get off at {} ({}) → no connection found",
                i + 1,
                o.alight.name,
                hm(o.alight_at)
            ),
        }
    }
    Ok(())
}

// ---------- staying on: a stretch of a trip as a leg of its own (ride mode's plan B) ----------


/// A trip's stops in order: origin, intermediate stops, terminus.
pub fn trip_places(trip: &Leg) -> Vec<&Place> {
    std::iter::once(&trip.from)
        .chain(trip.intermediate_stops.iter())
        .chain(std::iter::once(&trip.to))
        .collect()
}

fn sq_dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = (a.1 - b.1) * a.0.to_radians().cos();
    let dy = a.0 - b.0;
    dx * dx + dy * dy
}

/// Where `trip` stops at `at` (a stop id, or "lat,lng"): an index into [`trip_places`].
/// Stop ids match exactly; otherwise the nearest stop within ~300 m.
pub fn stop_index(trip: &Leg, at: &str) -> Option<usize> {
    let places = trip_places(trip);
    if let Some(i) = places
        .iter()
        .position(|p| p.stop_id.as_deref() == Some(at))
    {
        return Some(i);
    }
    let here = crate::planner::parse_coord(at)?;
    let (i, d) = places
        .iter()
        .enumerate()
        .map(|(i, p)| (i, sq_dist(here, (p.lat, p.lon))))
        .min_by(|a, b| a.1.total_cmp(&b.1))?;
    // ~300 m in degrees of latitude, squared.
    (d < (300.0_f64 / 111_000.0).powi(2)).then_some(i)
}

/// The ride on `trip` from its `a`-th to its `b`-th stop (indices into [`trip_places`]).
pub fn slice(trip: &Leg, a: usize, b: usize) -> Option<Leg> {
    let places = trip_places(trip);
    if a >= b || b >= places.len() {
        return None;
    }
    let (from, to) = (places[a].clone(), places[b].clone());
    let start = from.departure.or(from.scheduled_departure)?;
    let end = to.arrival.or(to.scheduled_arrival)?;
    let mut leg = trip.clone();
    leg.scheduled_start_time = from.scheduled_departure.unwrap_or(start);
    leg.scheduled_end_time = to.scheduled_arrival.unwrap_or(end);
    leg.start_time = start;
    leg.end_time = end;
    leg.duration = (end - start).num_seconds();
    leg.distance = None;
    leg.intermediate_stops = places[a + 1..b].iter().map(|p| (*p).clone()).collect();
    leg.leg_geometry = trip
        .leg_geometry
        .as_ref()
        .and_then(|g| slice_geometry(g, (from.lat, from.lon), (to.lat, to.lon)));
    leg.from = from;
    leg.to = to;
    Some(leg)
}

/// The part of a shape between the points nearest to `a` and (after that) to `b`.
fn slice_geometry(g: &Geometry, a: (f64, f64), b: (f64, f64)) -> Option<Geometry> {
    let pts = decode_polyline(&g.points, g.precision);
    let nearest = |from: usize, to: (f64, f64)| {
        (from..pts.len()).min_by(|&i, &j| sq_dist(pts[i], to).total_cmp(&sq_dist(pts[j], to)))
    };
    let i = nearest(0, a)?;
    let j = nearest(i, b)?;
    if j <= i {
        return None;
    }
    Some(Geometry {
        points: encode_polyline(&pts[i..=j], g.precision),
        precision: g.precision,
        length: (j - i + 1) as u32,
    })
}

fn decode_polyline(s: &str, precision: u32) -> Vec<(f64, f64)> {
    let scale = 10f64.powi(precision as i32);
    let bytes = s.as_bytes();
    let (mut i, mut lat, mut lng) = (0, 0i64, 0i64);
    let mut out = vec![];
    'outer: while i < bytes.len() {
        let mut v = [0i64; 2];
        for x in v.iter_mut() {
            let (mut shift, mut result) = (0, 0i64);
            loop {
                let Some(&c) = bytes.get(i) else { break 'outer };
                i += 1;
                let b = i64::from(c) - 63;
                result |= (b & 0x1f) << shift;
                shift += 5;
                if b < 0x20 {
                    break;
                }
            }
            *x = if result & 1 == 1 { !(result >> 1) } else { result >> 1 };
        }
        lat += v[0];
        lng += v[1];
        out.push((lat as f64 / scale, lng as f64 / scale));
    }
    out
}

fn encode_polyline(pts: &[(f64, f64)], precision: u32) -> String {
    let scale = 10f64.powi(precision as i32);
    let mut out = String::new();
    let (mut plat, mut plng) = (0i64, 0i64);
    for &(lat, lng) in pts {
        let (lat, lng) = ((lat * scale).round() as i64, (lng * scale).round() as i64);
        for d in [lat - plat, lng - plng] {
            let mut v = if d < 0 { !(d << 1) } else { d << 1 };
            while v >= 0x20 {
                out.push((((v & 0x1f) | 0x20) + 63) as u8 as char);
                v >>= 5;
            }
            out.push((v + 63) as u8 as char);
        }
        (plat, plng) = (lat, lng);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stop(name: &str, id: &str, lat: f64, arr: Option<&str>, dep: Option<&str>) -> Place {
        let t = |s: Option<&str>| s.map(|s| s.parse::<DateTime<Utc>>().unwrap());
        Place {
            name: name.into(),
            lat,
            lon: 4.5,
            stop_id: Some(id.into()),
            arrival: t(arr),
            departure: t(dep),
            scheduled_arrival: t(arr),
            scheduled_departure: t(dep),
            ..Default::default()
        }
    }

    /// Mol → Herentals → Lier → Berchem → Antwerpen-Centraal, straight north.
    fn trip() -> Leg {
        let pts: Vec<(f64, f64)> = (0..=8).map(|i| (51.0 + 0.1 * f64::from(i), 4.5)).collect();
        Leg {
            mode: "REGIONAL_RAIL".into(),
            trip_id: Some("L".into()),
            from: stop("Mol", "mol", 51.0, None, Some("2026-11-10T16:05:00Z")),
            intermediate_stops: vec![
                stop("Herentals", "her", 51.2, Some("2026-11-10T16:23:00Z"), Some("2026-11-10T16:24:00Z")),
                stop("Lier", "lie", 51.4, Some("2026-11-10T16:35:00Z"), Some("2026-11-10T16:36:00Z")),
                stop("Berchem", "ber", 51.6, Some("2026-11-10T16:45:00Z"), Some("2026-11-10T16:47:00Z")),
            ],
            to: stop("Centraal", "cen", 51.8, Some("2026-11-10T16:51:00Z"), None),
            leg_geometry: Some(Geometry {
                points: encode_polyline(&pts, 6),
                precision: 6,
                length: pts.len() as u32,
            }),
            ..Default::default()
        }
    }

    #[test]
    fn polyline_round_trips() {
        let pts = vec![(50.869884, 4.342844), (51.190649, 5.11535), (51.0, -0.5)];
        let back = decode_polyline(&encode_polyline(&pts, 6), 6);
        for (a, b) in pts.iter().zip(&back) {
            assert!((a.0 - b.0).abs() < 1e-6 && (a.1 - b.1).abs() < 1e-6);
        }
        // Google's reference example (precision 5).
        assert_eq!(
            encode_polyline(&[(38.5, -120.2), (40.7, -120.95), (43.252, -126.453)], 5),
            "_p~iF~ps|U_ulLnnqC_mqNvxq`@"
        );
    }

    #[test]
    fn finds_stops_by_id_or_place() {
        let t = trip();
        assert_eq!(stop_index(&t, "her"), Some(1));
        assert_eq!(stop_index(&t, "cen"), Some(4));
        assert_eq!(stop_index(&t, "51.4001,4.5002"), Some(2));
        assert_eq!(stop_index(&t, "52.5,4.5"), None);
        assert_eq!(stop_index(&t, "nowhere"), None);
    }

    #[test]
    fn slices_a_stretch_of_the_trip() {
        let t = trip();
        let l = slice(&t, 0, 3).unwrap();
        assert_eq!((l.from.name.as_str(), l.to.name.as_str()), ("Mol", "Berchem"));
        assert_eq!(l.start_time.to_rfc3339(), "2026-11-10T16:05:00+00:00");
        assert_eq!(l.end_time.to_rfc3339(), "2026-11-10T16:45:00+00:00");
        assert_eq!(l.duration, 40 * 60);
        let names: Vec<_> = l.intermediate_stops.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["Herentals", "Lier"]);
        let g = decode_polyline(&l.leg_geometry.unwrap().points, 6);
        assert!((g[0].0 - 51.0).abs() < 1e-6 && (g.last().unwrap().0 - 51.6).abs() < 1e-6);
        assert!(slice(&t, 2, 2).is_none() && slice(&t, 3, 9).is_none());
    }
}
