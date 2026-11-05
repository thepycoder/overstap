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
//!
//! Both produce real itineraries (legs with geometry), so the web UI can draw them
//! like anything MOTIS planned.

use crate::motis::{Client, Itinerary, Leg, PlanQuery, Rental};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::HashMap;

/// Providers whose bikes must be returned at one of their stations/hubs,
/// although their GBFS feed doesn't say so.
const HUB_ONLY_PREFIXES: &[&str] = &["donkey"];
/// Round-trip providers worth pricing as a tour.
const TOUR_PROVIDERS: &[&str] = &["blue-bike"];
/// Time to unlock/park a shared bike.
const RENTAL_OVERHEAD_S: i64 = 60;
/// Hubs considered for a hub-only drop-off (nearest to the destination first).
const MAX_HUBS: usize = 5;
/// Tour stations considered per alighting stop.
const MAX_TOUR_STATIONS: usize = 3;

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

/// A last-mile option built by overstap, to compare with MOTIS's own itineraries.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub label: String,
    pub notes: Vec<String>,
    pub it: Itinerary,
}

impl Candidate {
    pub fn arrive(&self) -> DateTime<Utc> {
        self.it.end_time
    }
}

/// What `fix_hub_only` made of an itinerary.
pub enum Fix {
    /// No hub-only rental at the end: keep MOTIS's itinerary.
    NotApplicable,
    Fixed(Candidate),
    /// Hub-only rental with no hub near the destination: MOTIS's plan can't be done.
    Unusable(String),
}

/// MOTIS lookups for one request, cached: many itineraries share stations and stops.
pub struct Street<'a> {
    pub c: &'a Client,
    routes: RefCell<HashMap<String, Option<Itinerary>>>,
    stations: RefCell<HashMap<String, Vec<RentalStation>>>,
}

impl<'a> Street<'a> {
    pub fn new(c: &'a Client) -> Self {
        Self {
            c,
            routes: RefCell::default(),
            stations: RefCell::default(),
        }
    }

    pub fn stations_near(&self, lat: f64, lon: f64, radius_m: u32) -> Result<Vec<RentalStation>> {
        let key = format!("{lat:.5},{lon:.5},{radius_m}");
        if let Some(v) = self.stations.borrow().get(&key) {
            return Ok(v.clone());
        }
        let r: RentalsResponse = reqwest::blocking::Client::new()
            .get(format!("{}/api/v1/rentals", self.c.base_url()))
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
        let stations: Vec<RentalStation> = r
            .stations
            .into_iter()
            .map(|mut s| {
                s.name = s.name.trim().to_string();
                s
            })
            .collect();
        self.stations.borrow_mut().insert(key, stations.clone());
        Ok(stations)
    }

    /// Street-only route (`WALK` or `BIKE`), fastest MOTIS direct itinerary. Street routing
    /// doesn't depend on the time, so the legs are cached and shifted to start at `at`.
    pub fn route(
        &self,
        from: (f64, f64),
        to: (f64, f64),
        mode: &str,
        at: DateTime<Utc>,
    ) -> Result<Option<Itinerary>> {
        let key = format!("{mode}:{:.5},{:.5}>{:.5},{:.5}", from.0, from.1, to.0, to.1);
        let cached = self.routes.borrow().get(&key).cloned();
        let it = match cached {
            Some(it) => it,
            None => {
                let mut q = PlanQuery::new(
                    format!("{},{}", from.0, from.1),
                    format!("{},{}", to.0, to.1),
                );
                q.time = Some(at);
                q.direct_modes = vec![mode.into()];
                q.max_direct_s = Some(3 * 3600);
                q.max_transfers = Some(0);
                q.detailed_legs = true;
                let it = self
                    .c
                    .plan(&q)?
                    .direct
                    .into_iter()
                    .min_by_key(|i| i.duration);
                self.routes.borrow_mut().insert(key, it.clone());
                it
            }
        };
        Ok(it.map(|mut it| {
            let d = at - it.start_time;
            it.legs.iter_mut().for_each(|l| l.shift(d));
            Itinerary::from_legs(it.legs)
        }))
    }
}

fn is_hub_only(provider: &str) -> bool {
    HUB_ONLY_PREFIXES.iter().any(|p| provider.starts_with(p))
}

fn dist2(a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = (a.1 - b.1) * a.0.to_radians().cos();
    let dy = a.0 - b.0;
    dx * dx + dy * dy
}

/// The legs of a street route, with its bike legs relabelled as `mode` (MOTIS's bike route
/// becomes the rental ride).
fn as_mode(it: Itinerary, mode: &str, rental: Option<&Rental>) -> Vec<Leg> {
    it.legs
        .into_iter()
        .map(|mut l| {
            if l.mode == "BIKE" {
                l.mode = mode.into();
                l.rental = rental.cloned();
            }
            l
        })
        .collect()
}

/// If the itinerary ends with a hub-only rental dropped outside a hub,
/// re-route the drop-off to the hub that minimises arrival time.
pub fn fix_hub_only(s: &Street, it: &Itinerary, dest: (f64, f64)) -> Result<Fix> {
    let Some(idx) = it.legs.iter().rposition(|l| l.mode == "RENTAL") else {
        return Ok(Fix::NotApplicable);
    };
    let leg = &it.legs[idx];
    let Some(rental) = &leg.rental else {
        return Ok(Fix::NotApplicable);
    };
    if !is_hub_only(&rental.provider_id)
        || rental
            .to_station_name
            .as_deref()
            .is_some_and(|s| !s.is_empty())
    {
        return Ok(Fix::NotApplicable);
    }
    let start = (leg.from.lat, leg.from.lon);
    let mut hubs: Vec<_> = s
        .stations_near(dest.0, dest.1, 1500)?
        .into_iter()
        .filter(|h| h.provider_id == rental.provider_id)
        .collect();
    hubs.sort_by(|a, b| dist2((a.lat, a.lon), dest).total_cmp(&dist2((b.lat, b.lon), dest)));
    let mut best: Option<(DateTime<Utc>, Vec<Leg>, RentalStation, i64, i64)> = None;
    for h in hubs.into_iter().take(MAX_HUBS) {
        let Some(ride) = s.route(start, (h.lat, h.lon), "BIKE", leg.start_time)? else {
            continue;
        };
        let parked = ride.end_time + Duration::seconds(RENTAL_OVERHEAD_S);
        let Some(walk) = s.route((h.lat, h.lon), dest, "WALK", parked)? else {
            continue;
        };
        if best.as_ref().is_some_and(|b| walk.end_time >= b.0) {
            continue;
        }
        let mut r = rental.clone();
        r.to_station_name = Some(h.name.clone());
        let (ride_s, walk_s) = (ride.duration, walk.duration);
        let mut legs = as_mode(ride, "RENTAL", Some(&r));
        if let Some(last) = legs.last_mut() {
            // Parking and locking the bike happens on the bike leg.
            last.to.name = h.name.clone();
            last.to.vertex_type = Some("BIKESHARE".into());
            last.end_time += Duration::seconds(RENTAL_OVERHEAD_S);
            last.scheduled_end_time = last.end_time;
            last.duration += RENTAL_OVERHEAD_S;
        }
        legs.extend(walk.legs);
        best = Some((walk.end_time, legs, h, ride_s, walk_s));
    }
    let name = rental.system_name.clone().unwrap_or("rental".into());
    let Some((_, tail, hub, ride, walk)) = best else {
        return Ok(Fix::Unusable(format!(
            "{name}: no hub within 1.5 km of the destination"
        )));
    };
    let mut legs: Vec<Leg> = it.legs[..idx].to_vec();
    legs.extend(tail);
    let fixed = Itinerary::from_legs(legs);
    Ok(Fix::Fixed(Candidate {
        label: format!("{name} via hub"),
        notes: vec![format!(
            "{name} bikes must be parked at a hub: '{}' ({} min ride, {} min walk). MOTIS had it at the door at {}.",
            hub.name,
            ride / 60,
            walk / 60,
            crate::render::hm(it.end_time)
        )],
        it: fixed,
    }))
}

/// Price a round-trip station bike (Blue-bike), trying every transit alighting
/// point of the itinerary (e.g. skip the last bus and bike from the station).
pub fn tour_option(s: &Street, it: &Itinerary, dest: (f64, f64)) -> Result<Option<Candidate>> {
    let mut best: Option<Candidate> = None;
    for (i, _) in it.legs.iter().enumerate().filter(|(_, l)| l.is_transit()) {
        if let Some(cand) = tour_from(s, it, i, dest)? {
            if best.as_ref().is_none_or(|b| cand.arrive() < b.arrive()) {
                best = Some(cand);
            }
        }
    }
    Ok(best)
}

fn tour_from(
    s: &Street,
    it: &Itinerary,
    idx: usize,
    dest: (f64, f64),
) -> Result<Option<Candidate>> {
    let last_transit = &it.legs[idx];
    let alight = (last_transit.to.lat, last_transit.to.lon);
    let t0 = last_transit.end_time;
    let mut stations: Vec<_> = s
        .stations_near(alight.0, alight.1, 500)?
        .into_iter()
        .filter(|st| {
            TOUR_PROVIDERS.contains(&st.provider_id.as_str()) && st.num_vehicles_available > 0
        })
        .collect();
    stations
        .sort_by(|a, b| dist2((a.lat, a.lon), alight).total_cmp(&dist2((b.lat, b.lon), alight)));
    let mut best: Option<Candidate> = None;
    for st in stations.into_iter().take(MAX_TOUR_STATIONS) {
        let Some(walk) = s.route(alight, (st.lat, st.lon), "WALK", t0)? else {
            continue;
        };
        let Some(ride) = s.route(
            (st.lat, st.lon),
            dest,
            "BIKE",
            walk.end_time + Duration::seconds(RENTAL_OVERHEAD_S),
        )?
        else {
            continue;
        };
        if best.as_ref().is_some_and(|b| ride.end_time >= b.arrive()) {
            continue;
        }
        let rental = Rental {
            provider_id: st.provider_id.clone(),
            system_name: Some(pretty_provider(&st.provider_id)),
            from_station_name: Some(st.name.clone()),
            return_constraint: Some("ROUNDTRIP_STATION".into()),
            form_factor: Some("BICYCLE".into()),
            propulsion_type: Some("HUMAN".into()),
            ..Default::default()
        };
        let (walk_s, ride_s) = (walk.duration, ride.duration);
        let mut legs: Vec<Leg> = it.legs[..=idx].to_vec();
        legs.extend(walk.legs);
        let mut ride_legs = as_mode(ride, "RENTAL", Some(&rental));
        if let Some(first) = ride_legs.first_mut() {
            first.from.name = st.name.clone();
            first.from.vertex_type = Some("BIKESHARE".into());
            // Unlocking the bike happens on the bike leg.
            first.start_time -= Duration::seconds(RENTAL_OVERHEAD_S);
            first.scheduled_start_time = first.start_time;
            first.duration += RENTAL_OVERHEAD_S;
        }
        legs.extend(ride_legs);
        let name = pretty_provider(&st.provider_id);
        best = Some(Candidate {
            label: format!("{name} round trip"),
            notes: vec![format!(
                "Round trip: take a {name} at '{}' ({} bikes now, {} min walk from {}), ride {} min, and ride it back to the station later (≈{} min).",
                st.name,
                st.num_vehicles_available,
                walk_s / 60,
                last_transit.to.name,
                ride_s / 60,
                ride_s / 60
            )],
            it: Itinerary::from_legs(legs),
        });
    }
    Ok(best)
}

fn pretty_provider(id: &str) -> String {
    match id {
        "blue-bike" => "Blue-bike".into(),
        other => other.to_string(),
    }
}
