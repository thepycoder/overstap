//! One planning request → the journeys worth showing, for the CLI and the web API.
//!
//! Each way of travelling (`Carry`) is one or more MOTIS queries plus overstap's own
//! corrections (see `lastmile`). Journeys come back as MOTIS itineraries, so the web UI
//! reads them in MOTIS's own shape.

use crate::lastmile::{self, Fix, Street};
use crate::motis::{Client, Itinerary, Leg, PlanQuery};
use anyhow::{bail, Result};
use chrono::{DateTime, Duration, Utc};
use clap::ValueEnum;
use serde::Serialize;

/// Minutes a round-trip bike must save over walking to be offered.
const TOUR_MIN_GAIN_MIN: i64 = 3;
/// Starting this close to your parked bike means it's with you.
const NEAR_BIKE_M: f64 = 300.0;
/// Longest walk from the last stop to your parked bike.
const PARKED_WALK_MAX_S: u32 = 10 * 60;
/// Unlocking your own bike.
const UNLOCK_S: i64 = 60;
/// A base search still "reaches" the destination when something leaves (or arrives) within
/// the search window plus this. Later than that, MOTIS has searched on into the afternoon.
const REACH_SLACK_MIN: i64 = 120;
/// A route the timetable allows but live times break by at most this much is still shown,
/// as one you might catch if the train makes up time (ADR-026).
const CATCH_UP_MAX_MIN: i64 = 5;
/// Live times only exist around now: no catch-up search for times further away.
const CATCH_UP_HORIZON_MIN: i64 = 180;
/// Shown when public transport runs but the destination point itself can't be reached
/// after it (e.g. a map pin inside private grounds, ADR-021).
pub const UNREACHABLE_WARNING: &str = "Hardly any route reaches this exact point after public transport, \
though the stops around it are served now. It may be inside private grounds: try its street address.";

/// Where your own (non-folding) bike is. It never goes on board: the planner uses it for
/// whichever street leg it can reach, and the rest of the journey is `carry`.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum OwnBike {
    #[default]
    None,
    /// With you at the start: ride it to the first stop and leave it there.
    Here,
    /// Parked here (lat, lon) from an earlier trip: travel there, then ride it on.
    Parked((f64, f64)),
}

impl OwnBike {
    /// "here", or where it's parked as "lat,lon".
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "" | "none" => Some(Self::None),
            "here" => Some(Self::Here),
            other => parse_coord(other).map(Self::Parked),
        }
    }
}

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
    /// Your own (non-folding) bike. No effect for `Folding`.
    pub bike: OwnBike,
    /// Also look for routes that only work if a late ride makes up time (one more search).
    pub catch_up: bool,
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
            catch_up: true,
            fast_kmh: None,
            bike: OwnBike::None,
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
    /// Set when a change only works if a late ride makes up time (live times say it's short).
    #[serde(rename = "catchUp", skip_serializing_if = "Option::is_none")]
    pub catch_up: Option<CatchUp>,
    #[serde(flatten)]
    pub it: Itinerary,
}

/// The tightest change of a catch-up route.
#[derive(Debug, Clone, Serialize)]
pub struct CatchUp {
    /// Where you change.
    pub at: String,
    /// How much later than needed you'd get there on live times, in seconds.
    #[serde(rename = "shortS")]
    pub short_s: i64,
}

impl Journey {
    fn motis(it: Itinerary) -> Self {
        Self {
            label: None,
            notes: vec![],
            catch_up: None,
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

fn meters(a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = (a.1 - b.1).to_radians() * a.0.to_radians().cos();
    let dy = (a.0 - b.0).to_radians();
    6_371_000.0 * (dx * dx + dy * dy).sqrt()
}

/// Your own bike is with you at the start (told so, or you start next to where it's parked).
fn bike_here(r: &Request) -> bool {
    match r.bike {
        OwnBike::None => false,
        OwnBike::Here => true,
        OwnBike::Parked(at) => parse_coord(&r.from).is_some_and(|f| meters(f, at) < NEAR_BIKE_M),
    }
}

/// Your own bike at the start: ride or walk to the first stop, the bike stays there (no bike
/// transport needed), or ride all the way.
fn bike_first(q: &mut PlanQuery, r: &Request) {
    q.pre_transit_modes = vec!["WALK".into(), "BIKE".into()];
    q.max_pre_transit_s = Some(r.max_bike_min * 60);
    if !q.direct_modes.iter().any(|m| m == "BIKE") {
        q.direct_modes.push("BIKE".into());
    }
    q.max_direct_s = Some(q.max_direct_s.unwrap_or(0).max(r.max_bike_min * 60));
}

/// What `plan_full` found: the journeys, and warnings about the request itself.
#[derive(Debug, Default)]
pub struct Planned {
    pub journeys: Vec<Journey>,
    pub warnings: Vec<String>,
}

/// Plans `r`, sorted by departure. May contain near-duplicates (same trains reached
/// differently); the caller decides what to hide.
pub fn plan(c: &Client, r: &Request) -> Result<Vec<Journey>> {
    Ok(plan_full(c, r)?.journeys)
}

/// `plan`, plus a warning when the destination point can't be reached after transit.
pub fn plan_full(c: &Client, r: &Request) -> Result<Planned> {
    // `base`: the tab's plain search, whose reach says whether the destination works.
    let (mut out, base) = match r.carry {
        Carry::Foot => {
            let p = c.plan(&foot_query(r))?;
            let base = p.itineraries.clone();
            let out = p
                .itineraries
                .into_iter()
                .chain(p.direct)
                .map(Journey::motis)
                .collect();
            (out, base)
        }
        Carry::Folding => folding(c, r)?,
        Carry::Shared => shared(c, r)?,
    };
    if r.catch_up {
        out.extend(catch_up(c, r));
    }
    let mut warnings = vec![];
    if unreachable(c, r, &base)? {
        warnings.push(UNREACHABLE_WARNING.to_string());
    }
    // Your parked bike is one more way to do the last stretch, next to the tab's own.
    if let OwnBike::Parked(at) = r.bike {
        if r.carry != Carry::Folding && !bike_here(r) {
            out.extend(parked(c, r, at)?);
        }
    }
    out.sort_by_key(|j| (j.it.start_time, j.it.end_time));
    // The same vehicles reached the same way are one option (e.g. found by both the
    // normal and the fast search, or the same tour built from two walking plans).
    let mut seen = std::collections::HashSet::new();
    out.retain(|j| seen.insert(signature(&j.it)));
    Ok(Planned {
        journeys: out,
        warnings,
    })
}

/// Something in `its` uses transit and leaves (arrives, for arrive-by) close to the asked time.
fn reaches(its: &[Itinerary], r: &Request) -> bool {
    let t = r.time.unwrap_or_else(Utc::now);
    let span = Duration::minutes(i64::from(r.window_min) + REACH_SLACK_MIN);
    its.iter()
        .filter(|it| it.legs.iter().any(|l| l.is_transit()))
        .any(|it| {
            if r.arrive_by {
                it.end_time >= t - span
            } else {
                it.start_time <= t + span
            }
        })
}

/// The base search doesn't reach the destination around the asked time, but a control search
/// to the stop those routes end at (or that walking would use) does: transit runs, and it's
/// the last stretch to this exact point that fails. Without the control a night search, with
/// nothing running for hours, would look the same.
fn unreachable(c: &Client, r: &Request, base: &[Itinerary]) -> Result<bool> {
    if reaches(base, r) {
        return Ok(false);
    }
    let last_stop = |its: &[Itinerary]| {
        its.iter()
            .find_map(|it| it.legs.iter().rev().find(|l| l.is_transit()))
            .map(|l| (l.to.lat, l.to.lon))
    };
    let mut q = base_query(r);
    q.max_pre_transit_s = Some(3600);
    q.max_post_transit_s = Some(3600);
    let stop = match last_stop(base) {
        Some(s) => s,
        // Nothing at all (e.g. no bike route to the point): ask where walking would get off.
        None => match last_stop(&c.plan(&q)?.itineraries) {
            Some(s) => s,
            None => return Ok(false),
        },
    };
    q.to = format!("{},{}", stop.0, stop.1);
    Ok(reaches(&c.plan(&q)?.itineraries, r))
}

/// The foot tab's search: walking (or your own bike, when it's with you) to and from transit.
fn foot_query(r: &Request) -> PlanQuery {
    let mut q = base_query(r);
    // MOTIS defaults to 15 min; a 2 km walk from the station is normal.
    q.max_pre_transit_s = Some(3600);
    q.max_post_transit_s = Some(3600);
    q.direct_modes = vec!["WALK".into()];
    q.max_direct_s = Some(3600);
    if bike_here(r) {
        bike_first(&mut q, r);
        q.cycling_speed = Some(r.ride_kmh / 3.6);
    }
    q
}

/// The folding tab's search at `kmh`, its window moved by `lead` (what riding faster saves).
fn folding_query(r: &Request, kmh: f64, lead: Duration) -> PlanQuery {
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
}

/// The tightest change of `it` on live times, if it's short: arriving (plus the walk or
/// transfer time MOTIS puts in between) after the next ride leaves.
fn short_change(it: &Itinerary) -> Option<CatchUp> {
    let rides: Vec<usize> = (0..it.legs.len())
        .filter(|&k| it.legs[k].is_transit())
        .collect();
    rides
        .windows(2)
        .filter_map(|w| {
            let (a, b) = (&it.legs[w[0]], &it.legs[w[1]]);
            let walk: i64 = it.legs[w[0] + 1..w[1]].iter().map(|l| l.duration).sum();
            let short = (a.end_time - b.start_time).num_seconds() + walk;
            (short > 0).then(|| CatchUp {
                at: a.to.name.clone(),
                short_s: short,
            })
        })
        .max_by_key(|c| c.short_s)
}

/// Routes the timetable allows but today's live times break by a few minutes: MOTIS's live
/// search drops them, yet a late train often makes up time. Searched on the timetable
/// (`REALTIME_ANNOTATION_ONLY`: live times still reported) and kept when the worst change is at
/// most CATCH_UP_MAX_MIN short.
fn catch_up(c: &Client, r: &Request) -> Vec<Journey> {
    let t = r.time.unwrap_or_else(Utc::now);
    if (t - Utc::now()).num_minutes().abs() > CATCH_UP_HORIZON_MIN {
        return vec![];
    }
    let mut q = match r.carry {
        Carry::Folding => folding_query(r, r.ride_kmh, Duration::zero()),
        Carry::Foot | Carry::Shared => foot_query(r),
    };
    q.realtime_mode = Some("REALTIME_ANNOTATION_ONLY");
    let p = match c.plan(&q) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("  catch-up search: {e:#}");
            return vec![];
        }
    };
    let mut trips = std::collections::HashMap::new();
    p.itineraries
        .into_iter()
        .filter(|it| !it.legs.iter().any(|l| l.cancelled == Some(true)))
        .filter_map(|it| {
            let it = rechange(c, it, &mut trips);
            let cu = short_change(&it)?;
            (cu.short_s <= CATCH_UP_MAX_MIN * 60).then(|| Journey {
                label: Some("If the train catches up".into()),
                notes: vec![format!(
                    "On live times you'd reach {} {} min too late to change. Late trains often make up a few minutes: decide there.",
                    cu.at,
                    (cu.short_s + 59) / 60
                )],
                catch_up: Some(cu),
                it,
            })
        })
        .collect()
}

/// On the timetable, changing at Herentals or one stop later at Lier (same two trains) is a tie
/// and MOTIS returns one of them; on live times the later stop can be the one that works. For
/// each short change between two trains that share more stops, move it to the shared stop
/// with the most slack. `trips` caches MOTIS trips by id.
fn rechange(
    c: &Client,
    mut it: Itinerary,
    trips: &mut std::collections::HashMap<String, Option<Leg>>,
) -> Itinerary {
    let rides: Vec<usize> = (0..it.legs.len())
        .filter(|&k| it.legs[k].is_transit())
        .collect();
    for w in rides.windows(2) {
        let (ia, ib) = (w[0], w[1]);
        // Only a change within one station: nothing between the rides but one transfer walk.
        let between = &it.legs[ia + 1..ib];
        if between.len() > 1 || between.iter().any(|l| l.mode != "WALK") {
            continue;
        }
        let walk: i64 = between.iter().map(|l| l.duration).sum();
        let (a, b) = (&it.legs[ia], &it.legs[ib]);
        let slack = (b.start_time - a.end_time).num_seconds() - walk;
        if slack >= 0 {
            continue;
        }
        let mut trip = |id: &Option<String>| -> Option<Leg> {
            let id = id.clone()?;
            trips
                .entry(id.clone())
                .or_insert_with(|| c.trip(&id).ok().and_then(|t| t.legs.into_iter().next()))
                .clone()
        };
        let (Some(ta), Some(tb)) = (trip(&a.trip_id), trip(&b.trip_id)) else {
            continue;
        };
        let stop = |t: &Leg, p: &crate::motis::Place| {
            crate::overshoot::stop_index(t, p.stop_id.as_deref()?)
        };
        let (Some(board), Some(alight)) = (stop(&ta, &a.from), stop(&tb, &b.to)) else {
            continue;
        };
        let Some((i, j, best)) = crate::overshoot::best_change(&ta, board, &tb, alight, walk)
        else {
            continue;
        };
        if best <= slack {
            continue;
        }
        let (Some(na), Some(nb)) = (
            crate::overshoot::slice(&ta, board, i),
            crate::overshoot::slice(&tb, j, alight),
        ) else {
            continue;
        };
        if let Some(t) = it.legs.get_mut(ia + 1).filter(|_| ib == ia + 2) {
            t.from = na.to.clone();
            t.to = na.to.clone();
            t.start_time = na.end_time;
            t.end_time = na.end_time + Duration::seconds(walk);
            t.scheduled_start_time = t.start_time;
            t.scheduled_end_time = t.end_time;
            t.leg_geometry = None;
            t.distance = None;
        }
        it.legs[ia] = na;
        it.legs[ib] = nb;
    }
    Itinerary::from_legs(it.legs)
}

fn folding(c: &Client, r: &Request) -> Result<(Vec<Journey>, Vec<Itinerary>)> {
    let query = |kmh: f64, lead: Duration| folding_query(r, kmh, lead);
    let p = c.plan(&query(r.ride_kmh, Duration::zero()))?;
    let base = p.itineraries.clone();
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
    Ok((out, base))
}

/// Your bike parked at `at` from an earlier trip: transit to there (getting to the first stop
/// as the tab does), then ride it to the destination. Only when that ride is short enough
/// to count as a last mile; otherwise the bike is no use for this trip.
fn parked(c: &Client, r: &Request, at: (f64, f64)) -> Result<Vec<Journey>> {
    let Some(dest) = parse_coord(&r.to) else {
        return Ok(vec![]);
    };
    let s = Street::new(c);
    let Some(ride) = s.route(at, dest, "BIKE", r.time.unwrap_or_else(Utc::now))? else {
        return Ok(vec![]);
    };
    if ride.duration > i64::from(r.max_bike_min) * 60 {
        return Ok(vec![]);
    }
    let mut q = base_query(r);
    q.to = format!("{},{}", at.0, at.1);
    if r.carry == Carry::Shared {
        q.pre_transit_modes = vec!["WALK".into(), "RENTAL".into()];
        q.max_pre_transit_s = Some(r.max_bike_min * 60);
    } else {
        q.max_pre_transit_s = Some(3600);
    }
    q.max_post_transit_s = Some(PARKED_WALK_MAX_S);
    if r.arrive_by {
        q.time = r
            .time
            .map(|t| t - Duration::seconds(ride.duration + UNLOCK_S));
    }
    let mut out = vec![];
    for it in c.plan(&q)?.itineraries {
        let Some(stop) = it.legs.iter().rev().find(|l| l.is_transit()) else {
            continue;
        };
        let stop = stop.to.name.clone();
        let unlocked = it.end_time + Duration::seconds(UNLOCK_S);
        let Some(ride) = s.route(at, dest, "BIKE", unlocked)? else {
            continue;
        };
        let mut legs = it.legs;
        let mut bike = ride.legs;
        if let Some(first) = bike.first_mut() {
            // Unlocking happens on the bike leg.
            first.start_time -= Duration::seconds(UNLOCK_S);
            first.scheduled_start_time = first.start_time;
            first.duration += UNLOCK_S;
        }
        legs.extend(bike);
        out.push(Journey {
            label: Some("Your bike".into()),
            notes: vec![format!(
                "Your bike is parked near {stop}: ride it the rest of the way ({} min).",
                ride.duration / 60
            )],
            catch_up: None,
            it: Itinerary::from_legs(legs),
        });
    }
    Ok(out)
}

/// Seconds spent on shared bikes, after the last transit leg (the leg MOTIS's own
/// `max_post_transit_s` caps for its rental plans).
fn rental_s(it: &Itinerary) -> i64 {
    let after = it
        .legs
        .iter()
        .rposition(|l| l.is_transit())
        .map_or(0, |i| i + 1);
    it.legs[after..]
        .iter()
        .filter(|l| l.mode == "RENTAL")
        .map(|l| l.duration)
        .sum()
}

/// Shared bikes: MOTIS's rental plans (hub-only providers corrected), walking plans,
/// and round-trip station bikes priced as a tour from every alighting stop.
fn shared(c: &Client, r: &Request) -> Result<(Vec<Journey>, Vec<Itinerary>)> {
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
    // MOTIS's cycling speed would also time the rentals, so own-bike legs keep its default
    // here and the UI re-times them (ADR-013).
    if bike_here(r) {
        bike_first(&mut q, r);
    }
    let rental = c.plan(&q)?;

    let mut walk_q = base_query(r);
    walk_q.max_pre_transit_s = Some(3600);
    walk_q.max_post_transit_s = Some(3600);
    if bike_here(r) {
        bike_first(&mut walk_q, r);
    }
    let walk = c.plan(&walk_q)?.itineraries;
    let base = walk.clone();

    let s = Street::new(c);
    let mut out = vec![];
    for it in rental.itineraries.into_iter().chain(rental.direct) {
        match lastmile::fix_hub_only(&s, &it, dest)? {
            Fix::NotApplicable => out.push(Journey::motis(it)),
            Fix::Fixed(cand) => out.push(Journey {
                label: Some(cand.label),
                notes: cand.notes,
                catch_up: None,
                it: cand.it,
            }),
            Fix::Unusable(why) => eprintln!("  dropped: {why}"),
        }
    }
    for it in walk {
        // A round trip is a hassle (you have to come back for it): only worth it when it
        // gets you there clearly earlier than walking, and the ride is within your limit.
        let worth = |c: &lastmile::Candidate| {
            c.arrive() + Duration::minutes(TOUR_MIN_GAIN_MIN) <= it.end_time
                && rental_s(&c.it) <= i64::from(r.max_bike_min) * 60
        };
        if let Some(cand) = lastmile::tour_option(&s, &it, dest)?.filter(worth) {
            out.push(Journey {
                label: Some(cand.label),
                notes: cand.notes,
                catch_up: None,
                it: cand.it,
            });
        }
        out.push(Journey::motis(it));
    }
    Ok((out, base))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timed(mode: &str, to: &str, start: &str, end: &str) -> crate::motis::Leg {
        let t = |s: &str| format!("2026-11-10T{s}:00Z").parse().unwrap();
        crate::motis::Leg {
            mode: mode.into(),
            to: crate::motis::Place {
                name: to.into(),
                ..Default::default()
            },
            start_time: t(start),
            end_time: t(end),
            duration: (t(end) - t(start)).num_seconds(),
            ..Default::default()
        }
    }

    #[test]
    fn short_change_measures_the_tightest_change() {
        // L into Lier 16:39 (live), 4 min transfer, IC leaves 16:40: 3 min short.
        let it = Itinerary::from_legs(vec![
            timed("WALK", "Mol", "16:00", "16:04"),
            timed("REGIONAL_RAIL", "Lier", "16:05", "16:39"),
            timed("WALK", "Lier", "16:39", "16:43"),
            timed("REGIONAL_RAIL", "Brussel-Noord", "16:40", "17:16"),
            timed("WALK", "Home", "17:16", "17:20"),
        ]);
        let cu = short_change(&it).unwrap();
        assert_eq!((cu.at.as_str(), cu.short_s), ("Lier", 180));
        // Arriving 16:35 leaves a minute to spare: nothing to catch up.
        let mut ok = it.clone();
        ok.legs[1] = timed("REGIONAL_RAIL", "Lier", "16:05", "16:35");
        assert!(short_change(&ok).is_none());
    }
    use crate::motis::{Leg, Rental};

    fn leg(mode: &str, trip: Option<&str>) -> Leg {
        Leg {
            mode: mode.into(),
            trip_id: trip.map(Into::into),
            ..Default::default()
        }
    }

    #[test]
    fn own_bike_only_before_transit() {
        let mut r = Request::new("51.1,4.0".into(), "51.2,4.4".into(), Carry::Shared);
        r.max_bike_min = 20;
        let mut q = base_query(&r);
        q.post_transit_modes = vec!["WALK".into(), "RENTAL".into()];
        q.direct_modes = vec!["WALK".into()];
        q.max_direct_s = Some(3600);
        bike_first(&mut q, &r);
        assert_eq!(q.pre_transit_modes, ["WALK", "BIKE"]);
        assert_eq!(q.post_transit_modes, ["WALK", "RENTAL"]);
        assert_eq!(q.direct_modes, ["WALK", "BIKE"]);
        assert_eq!(q.max_pre_transit_s, Some(1200));
        assert_eq!(q.max_direct_s, Some(3600));
        assert!(!q.require_bike_transport);
    }

    #[test]
    fn own_bike_parses_and_is_here_when_you_start_next_to_it() {
        assert_eq!(OwnBike::parse("here"), Some(OwnBike::Here));
        assert_eq!(OwnBike::parse(""), Some(OwnBike::None));
        assert_eq!(
            OwnBike::parse("51.1036,3.9917"),
            Some(OwnBike::Parked((51.1036, 3.9917)))
        );
        assert_eq!(OwnBike::parse("station"), None);
        let mut r = Request::new("51.1038,3.9920".into(), "51.2,4.4".into(), Carry::Foot);
        r.bike = OwnBike::Parked((51.1036, 3.9917));
        assert!(bike_here(&r));
        r.from = "50.8453,4.3571".into();
        assert!(!bike_here(&r));
    }

    #[test]
    fn reach_is_judged_around_the_asked_time() {
        let at = |h: u32, m: u32| {
            chrono::NaiveDate::from_ymd_opt(2026, 11, 10)
                .unwrap()
                .and_hms_opt(h, m, 0)
                .unwrap()
                .and_utc()
        };
        let trip = |start: DateTime<Utc>, end: DateTime<Utc>| {
            let mut it = Itinerary::from_legs(vec![leg("REGIONAL_RAIL", Some("t"))]);
            it.start_time = start;
            it.end_time = end;
            it
        };
        let mut r = Request::new("51.1,4.0".into(), "51.2,5.0".into(), Carry::Shared);
        r.time = Some(at(4, 32));
        // Window 60 min + 2 h slack: 07:32 still counts, the afternoon doesn't.
        assert!(reaches(&[trip(at(7, 30), at(9, 0))], &r));
        assert!(!reaches(&[trip(at(12, 9), at(15, 3))], &r));
        assert!(!reaches(&[], &r));
        // A walk-only plan never counts.
        assert!(!reaches(
            &[Itinerary::from_legs(vec![leg("WALK", None)])],
            &r
        ));
        r.arrive_by = true;
        r.time = Some(at(8, 0));
        assert!(reaches(&[trip(at(5, 0), at(7, 0))], &r));
        assert!(!reaches(&[trip(at(1, 0), at(4, 50))], &r));
    }

    #[test]
    fn rental_time_counts_only_the_last_mile() {
        let ride = |mins: i64| Leg {
            duration: mins * 60,
            ..leg("RENTAL", None)
        };
        let it = Itinerary::from_legs(vec![
            ride(10),
            leg("REGIONAL_RAIL", Some("t1")),
            leg("WALK", None),
            ride(29),
        ]);
        assert_eq!(rental_s(&it), 29 * 60);
        assert_eq!(rental_s(&Itinerary::from_legs(vec![ride(12)])), 12 * 60);
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
