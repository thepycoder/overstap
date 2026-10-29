//! Last-mile corrections MOTIS can't express from GBFS alone (use case 1).
//!
//! * Hub-only providers (Donkey Republic): the GBFS feed has no geofencing
//!   zones and no return constraint, so MOTIS treats the bikes as free-floating
//!   and drops them at your destination's door. In reality they must end at a
//!   hub. We move the drop-off to the best hub and add the walk.
//! * Round-trip providers (Blue-bike): GBFS says `ROUNDTRIP_STATION`, so MOTIS
//!   never proposes them for a one-way last mile. But a round trip is often
//!   still the best option (go, come back later). We price it as a *tour*:
//!   ride there now, ride back to the station later.

use crate::motis::{Client, Itinerary, PlanQuery};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;

/// Providers whose bikes must be returned at one of their stations/hubs,
/// although their GBFS feed doesn't say so.
const HUB_ONLY_PREFIXES: &[&str] = &["donkey"];
/// Round-trip providers worth pricing as a tour.
const TOUR_PROVIDERS: &[&str] = &["blue-bike"];
/// Time to unlock/park a shared bike.
const RENTAL_OVERHEAD_S: i64 = 60;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RentalStation {
    pub provider_id: String,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    #[serde(default)]
    pub num_vehicles_available: i64,
}

#[derive(Deserialize)]
struct RentalsResponse {
    stations: Vec<RentalStation>,
}

/// A last-mile option to compare against MOTIS's own itineraries.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub label: String,
    pub depart: DateTime<Utc>,
    pub arrive: DateTime<Utc>,
    pub notes: Vec<String>,
    pub base: Itinerary,
}

pub fn stations_near(c: &Client, lat: f64, lon: f64, radius_m: u32) -> Result<Vec<RentalStation>> {
    let r: RentalsResponse = reqwest::blocking::Client::new()
        .get(format!("{}/api/v1/rentals", c.base_url()))
        .query(&[
            ("point", format!("{lat},{lon}")),
            ("radius", radius_m.to_string()),
            ("withStations", "true".into()),
            ("withVehicles", "false".into()),
            ("withZones", "false".into()),
        ])
        .send()?
        .error_for_status()?
        .json()?;
    Ok(r.stations)
}

/// Street-only travel time in seconds via MOTIS (`WALK` or `BIKE`).
pub fn street_secs(
    c: &Client,
    from: (f64, f64),
    to: (f64, f64),
    mode: &str,
    at: DateTime<Utc>,
) -> Result<Option<i64>> {
    let mut q = PlanQuery::new(
        format!("{},{}", from.0, from.1),
        format!("{},{}", to.0, to.1),
    );
    q.time = Some(at);
    q.direct_modes = vec![mode.into()];
    q.max_direct_s = Some(3 * 3600);
    q.max_transfers = Some(0);
    Ok(c.plan(&q)?.direct.iter().map(|i| i.duration).min())
}

fn is_hub_only(provider: &str) -> bool {
    HUB_ONLY_PREFIXES.iter().any(|p| provider.starts_with(p))
}

/// If the itinerary ends with a hub-only rental dropped outside a hub,
/// re-route the drop-off to the hub that minimises arrival time.
pub fn fix_hub_only(c: &Client, it: &Itinerary, dest: (f64, f64)) -> Result<Option<Candidate>> {
    let Some(leg) = it.legs.iter().rev().find(|l| l.mode == "RENTAL") else {
        return Ok(None);
    };
    let Some(rental) = &leg.rental else {
        return Ok(None);
    };
    if !is_hub_only(&rental.provider_id)
        || rental
            .to_station_name
            .as_deref()
            .is_some_and(|s| !s.is_empty())
    {
        return Ok(None);
    }
    let start = (leg.from.lat, leg.from.lon);
    let hubs: Vec<_> = stations_near(c, dest.0, dest.1, 1500)?
        .into_iter()
        .filter(|s| s.provider_id == rental.provider_id)
        .collect();
    let mut best: Option<(i64, RentalStation, i64, i64)> = None;
    for h in hubs {
        let (Some(ride), Some(walk)) = (
            street_secs(c, start, (h.lat, h.lon), "BIKE", leg.start_time)?,
            street_secs(c, (h.lat, h.lon), dest, "WALK", leg.start_time)?,
        ) else {
            continue;
        };
        let total = ride + RENTAL_OVERHEAD_S + walk;
        if best.as_ref().map_or(true, |b| total < b.0) {
            best = Some((total, h, ride, walk));
        }
    }
    let Some((total, hub, ride, walk)) = best else {
        return Ok(Some(Candidate {
            label: format!(
                "{} (no hub within 1.5 km — not usable)",
                rental.system_name.clone().unwrap_or_default()
            ),
            depart: it.start_time,
            arrive: it.end_time + Duration::days(1),
            notes: vec![],
            base: it.clone(),
        }));
    };
    let arrive = leg.start_time + Duration::seconds(total);
    let motis_said = it.end_time;
    Ok(Some(Candidate {
        label: format!("{} via hub", rental.system_name.clone().unwrap_or("rental".into())),
        depart: it.start_time,
        arrive,
        notes: vec![
            format!(
                "MOTIS dropped the bike at the door (arrive {}); hub-only provider → park at '{}' ({} min ride, {} min walk)",
                crate::render::hm(motis_said), hub.name, ride / 60, walk / 60
            ),
        ],
        base: it.clone(),
    }))
}

/// Price a round-trip station bike (Blue-bike), trying every transit alighting
/// point of the itinerary (e.g. skip the last bus and bike from the station).
pub fn tour_option(c: &Client, it: &Itinerary, dest: (f64, f64)) -> Result<Option<Candidate>> {
    let mut best: Option<Candidate> = None;
    for last_transit in it
        .legs
        .iter()
        .filter(|l| !matches!(l.mode.as_str(), "WALK" | "BIKE" | "RENTAL"))
    {
        if let Some(cand) = tour_from(c, it, last_transit, dest)? {
            if best.as_ref().map_or(true, |b| cand.arrive < b.arrive) {
                best = Some(cand);
            }
        }
    }
    Ok(best)
}

fn tour_from(
    c: &Client,
    it: &Itinerary,
    last_transit: &crate::motis::Leg,
    dest: (f64, f64),
) -> Result<Option<Candidate>> {
    let alight = (last_transit.to.lat, last_transit.to.lon);
    let t0 = last_transit.end_time;
    let stations: Vec<_> = stations_near(c, alight.0, alight.1, 500)?
        .into_iter()
        .filter(|s| {
            TOUR_PROVIDERS.contains(&s.provider_id.as_str()) && s.num_vehicles_available > 0
        })
        .collect();
    let mut best: Option<Candidate> = None;
    for s in stations {
        let (Some(walk), Some(ride)) = (
            street_secs(c, alight, (s.lat, s.lon), "WALK", t0)?,
            street_secs(c, (s.lat, s.lon), dest, "BIKE", t0)?,
        ) else {
            continue;
        };
        let arrive = t0 + Duration::seconds(walk + RENTAL_OVERHEAD_S + ride);
        if best.as_ref().map_or(true, |b| arrive < b.arrive) {
            best = Some(Candidate {
                label: format!("{} ({}) round trip", s.provider_id, s.name),
                depart: it.start_time,
                arrive,
                notes: vec![format!(
                    "alight {} at {}, walk {} min to '{}' ({} bikes), ride {} min; return ride ≈{} min later",
                    last_transit.to.name, crate::render::hm(t0), walk / 60, s.name,
                    s.num_vehicles_available, ride / 60, ride / 60
                )],
                base: truncate_after(it, last_transit),
            });
        }
    }
    Ok(best)
}

/// The itinerary up to and including `leg` (the rest is replaced by the bike).
fn truncate_after(it: &Itinerary, leg: &crate::motis::Leg) -> Itinerary {
    let mut out = it.clone();
    if let Some(i) = it.legs.iter().position(|l| std::ptr::eq(l, leg)) {
        out.legs.truncate(i + 1);
    }
    out
}
