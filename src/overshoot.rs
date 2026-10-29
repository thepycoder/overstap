//! Use case 2 — "I stayed on the train too long".
//!
//! MOTIS has no "origin = aboard trip X" query, so we emulate it: fetch the
//! trip's remaining stops, then for each of the next N stops ask MOTIS for the
//! best continuation departing at the (real-time) arrival there, and rank.
//! This is the brute-force orchestration pattern from ADR-003.

use crate::motis::{Client, Itinerary, Place, PlanQuery};
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
        match &self.best {
            Some(it) => Some(it.end_time),
            None => None,
        }
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
