//! One planning request → the journeys worth showing, for the CLI and the web API.
//!
//! Each way of travelling (`Carry`) is one or more MOTIS queries plus overstap's own
//! corrections (see `lastmile`). Journeys come back as MOTIS itineraries, so the web UI
//! reads them in MOTIS's own shape.

use crate::lastmile::{self, Fix, Street};
use crate::motis::{Client, Itinerary, PlanQuery};
use anyhow::{bail, Result};
use chrono::{DateTime, Duration, Utc};
use clap::ValueEnum;
use serde::Serialize;

/// Minutes a round-trip bike must save over walking to be offered.
const TOUR_MIN_GAIN_MIN: i64 = 3;

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Carry {
    /// Walk only around transit.
    Foot,
    /// Folding bike: ride before/after transit, bike must be allowed on board.
    Folding,
    /// Use shared bikes (GBFS) for first/last mile.
    Shared,
}

impl Carry {
    pub fn parse(s: &str) -> Option<Self> {
        <Self as ValueEnum>::from_str(s, true).ok()
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    /// "lat,lon" or a MOTIS stop id.
    pub from: String,
    pub to: String,
    pub time: Option<DateTime<Utc>>,
    pub arrive_by: bool,
    pub window_min: u32,
    pub carry: Carry,
    /// Longest bike/rental ride before and after transit, in minutes.
    pub max_bike_min: u32,
    pub min_transfer_min: Option<u32>,
    /// Your usual speed on your own bike.
    pub ride_kmh: f64,
    /// Also search as if riding this fast, to find trains you'd only catch riding hard.
    pub fast_kmh: Option<f64>,
}

impl Request {
    pub fn new(from: String, to: String, carry: Carry) -> Self {
        Self {
            from,
            to,
            time: None,
            arrive_by: false,
            window_min: 60,
            carry,
            max_bike_min: 45,
            min_transfer_min: None,
            ride_kmh: 18.0,
            fast_kmh: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Journey {
    /// What overstap changed, e.g. "Donkey Republic via hub". None for MOTIS's own plans.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    #[serde(flatten)]
    pub it: Itinerary,
}

impl Journey {
    fn motis(it: Itinerary) -> Self {
        Self {
            label: None,
            notes: vec![],
            it,
        }
    }
}

pub fn parse_coord(s: &str) -> Option<(f64, f64)> {
    let mut p = s.split(',').map(|x| x.trim().parse::<f64>());
    match (p.next(), p.next(), p.next()) {
        (Some(Ok(a)), Some(Ok(b)), None) => Some((a, b)),
        _ => None,
    }
}

fn signature(it: &Itinerary) -> String {
    let mut parts: Vec<String> = it
        .legs
        .iter()
        .map(|l| match (&l.trip_id, &l.rental) {
            (Some(t), _) if l.is_transit() => t.clone(),
            (_, Some(r)) => format!("{}:{}", l.mode, r.provider_id),
            _ => l.mode.clone(),
        })
        .collect();
    parts.dedup();
    if !it.legs.iter().any(|l| l.is_transit()) {
        parts.push(it.start_time.timestamp().to_string());
    }
    parts.join(">")
}

fn base_query(r: &Request) -> PlanQuery {
    let mut q = PlanQuery::new(r.from.clone(), r.to.clone());
    q.time = r.time;
    q.arrive_by = r.arrive_by;
    q.search_window_s = r.window_min * 60;
    q.min_transfer_time_min = r.min_transfer_min;
    // Leg shapes for the map (MOTIS leaves them empty otherwise).
    q.detailed_legs = true;
    q
}

/// Plans `r`, sorted by departure. May contain near-duplicates (same trains reached
/// differently); the caller decides what to hide.
pub fn plan(c: &Client, r: &Request) -> Result<Vec<Journey>> {
    let mut out = match r.carry {
        Carry::Foot => {
            let mut q = base_query(r);
            // MOTIS defaults to 15 min; a 2 km walk from the station is normal.
            q.max_pre_transit_s = Some(3600);
            q.max_post_transit_s = Some(3600);
            q.direct_modes = vec!["WALK".into()];
            q.max_direct_s = Some(3600);
            let p = c.plan(&q)?;
            p.itineraries
                .into_iter()
                .chain(p.direct)
                .map(Journey::motis)
                .collect()
        }
        Carry::Folding => folding(c, r)?,
        Carry::Shared => shared(c, r)?,
    };
    out.sort_by_key(|j| (j.it.start_time, j.it.end_time));
    // The same vehicles reached the same way are one option (e.g. found by both the
    // normal and the fast search, or the same tour built from two walking plans).
    let mut seen = std::collections::HashSet::new();
    out.retain(|j| seen.insert(signature(&j.it)));
    Ok(out)
}

fn folding(c: &Client, r: &Request) -> Result<Vec<Journey>> {
    let query = |kmh: f64, lead: Duration| {
        let mut q = base_query(r);
        q.pre_transit_modes = vec!["BIKE".into()];
        q.post_transit_modes = vec!["BIKE".into()];
        q.max_pre_transit_s = Some(r.max_bike_min * 60);
        q.max_post_transit_s = Some(r.max_bike_min * 60);
        q.direct_modes = vec!["BIKE".into()];
        q.max_direct_s = Some(3 * 3600);
        q.require_bike_transport = true;
        q.cycling_speed = Some(kmh / 3.6);
        // Start the window earlier (later, arriving) by what riding faster saves.
        q.time = r
            .time
            .map(|t| if r.arrive_by { t + lead } else { t - lead });
        q.search_window_s += lead.num_seconds() as u32;
        q
    };
    let p = c.plan(&query(r.ride_kmh, Duration::zero()))?;
    let mut out: Vec<Journey> = p
        .itineraries
        .into_iter()
        .chain(p.direct)
        .map(Journey::motis)
        .collect();
    if let Some(fast) = r.fast_kmh.filter(|f| *f > r.ride_kmh) {
        let lead =
            Duration::seconds((r.max_bike_min as f64 * 60.0 * (1.0 - r.ride_kmh / fast)) as i64);
        let p = c.plan(&query(fast, lead))?;
        out.extend(p.itineraries.into_iter().map(Journey::motis));
    }
    Ok(out)
}

/// Shared bikes: MOTIS's rental plans (hub-only providers corrected), walking plans,
/// and round-trip station bikes priced as a tour from every alighting stop.
fn shared(c: &Client, r: &Request) -> Result<Vec<Journey>> {
    let Some(dest) = parse_coord(&r.to) else {
        bail!("shared bikes need the destination as lat,lon")
    };
    let mut q = base_query(r);
    q.pre_transit_modes = vec!["WALK".into(), "RENTAL".into()];
    q.post_transit_modes = vec!["WALK".into(), "RENTAL".into()];
    q.max_pre_transit_s = Some(r.max_bike_min * 60);
    q.max_post_transit_s = Some(r.max_bike_min * 60);
    q.direct_modes = vec!["WALK".into(), "RENTAL".into()];
    q.max_direct_s = Some(3600);
    let rental = c.plan(&q)?;

    let mut walk_q = base_query(r);
    walk_q.max_pre_transit_s = Some(3600);
    walk_q.max_post_transit_s = Some(3600);
    let walk = c.plan(&walk_q)?.itineraries;

    let s = Street::new(c);
    let mut out = vec![];
    for it in rental.itineraries.into_iter().chain(rental.direct) {
        match lastmile::fix_hub_only(&s, &it, dest)? {
            Fix::NotApplicable => out.push(Journey::motis(it)),
            Fix::Fixed(cand) => out.push(Journey {
                label: Some(cand.label),
                notes: cand.notes,
                it: cand.it,
            }),
            Fix::Unusable(why) => eprintln!("  dropped: {why}"),
        }
    }
    for it in walk {
        // A round trip is a hassle (you have to come back for it): only worth it when it
        // gets you there clearly earlier than walking.
        let worth = |c: &lastmile::Candidate| {
            c.arrive() + Duration::minutes(TOUR_MIN_GAIN_MIN) <= it.end_time
        };
        if let Some(cand) = lastmile::tour_option(&s, &it, dest)?.filter(worth) {
            out.push(Journey {
                label: Some(cand.label),
                notes: cand.notes,
                it: cand.it,
            });
        }
        out.push(Journey::motis(it));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::motis::{Leg, Rental};

    fn leg(mode: &str, trip: Option<&str>) -> Leg {
        Leg {
            mode: mode.into(),
            trip_id: trip.map(Into::into),
            ..Default::default()
        }
    }

    #[test]
    fn coords_parse_strictly() {
        assert_eq!(parse_coord("51.1,4.0"), Some((51.1, 4.0)));
        assert_eq!(parse_coord("sncb_8821006"), None);
        assert_eq!(parse_coord("1,2,3"), None);
    }

    #[test]
    fn signature_ignores_times_and_repeated_walks() {
        let a = Itinerary::from_legs(vec![
            leg("BIKE", None),
            leg("REGIONAL_RAIL", Some("t1")),
            leg("WALK", None),
            leg("WALK", None),
            leg("REGIONAL_RAIL", Some("t2")),
        ]);
        let mut b = a.clone();
        b.legs.remove(3);
        b.legs[0].shift(Duration::minutes(4));
        assert_eq!(signature(&a), signature(&b));
        let mut c = a.clone();
        c.legs[0] = Leg {
            rental: Some(Rental {
                provider_id: "blue-bike".into(),
                ..Default::default()
            }),
            ..leg("RENTAL", None)
        };
        assert_ne!(signature(&a), signature(&c));
    }
}
