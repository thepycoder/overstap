//! `overstap serve`: the web UI (static files from `web/`) and a small JSON API over MOTIS.
//!
//! | Endpoint | What |
//! |---|---|
//! | `GET /api/config` | map centre, bounds, time zone |
//! | `GET /api/search?q=&near=lat,lng` | place search (Citymapper, else MOTIS; `places.rs`) |
//! | `GET /api/resolve?id=` | coordinates of a Citymapper search hit |
//! | `GET /api/reverse?lat=&lng=` | name for a dropped pin |
//! | `GET /api/journeys?start=&end=&mode=&time=&arrive=&window=&kmh=&fast=&maxbike=&bike=` | journeys, MOTIS-shaped; `bike=here` (your own bike is with you) or `bike=lat,lng` (parked there) |
//! | `POST /api/live {"trips":[tripId…]}` | current state of those trips (live times, tracks — NMBS's from iRail —, cancellations) |
//! | `POST /api/ride {mode,to,changes,rentals}` | ride mode: plan B per change (get off there, or stay on and change later), bikes at rental pickups |
//! | `GET /healthz` | MOTIS reachable? |
//!
//! Blocking and threaded (tiny_http + reqwest::blocking): one rider, a handful of requests.

use crate::motis::{Client, Leg};
use crate::places::Places;
use crate::planner::{self, Carry, OwnBike};
use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tiny_http::{Header, Method, Request, Response, Server};

const THREADS: usize = 8;

pub fn run(c: Client, places: Places, listen: &str, web: &Path) -> Result<()> {
    let web = web
        .canonicalize()
        .with_context(|| format!("web directory {} not found", web.display()))?;
    let server = Arc::new(Server::http(listen).map_err(|e| anyhow!("listen on {listen}: {e}"))?);
    let c = Arc::new(c);
    let places = Arc::new(places);
    eprintln!(
        "overstap: http://{listen} (MOTIS {}, web {})",
        c.base_url(),
        web.display()
    );
    let workers: Vec<_> = (0..THREADS)
        .map(|_| {
            let (server, c, places, web) = (server.clone(), c.clone(), places.clone(), web.clone());
            std::thread::spawn(move || {
                for req in server.incoming_requests() {
                    handle(&c, &places, &web, req);
                }
            })
        })
        .collect();
    for w in workers {
        let _ = w.join();
    }
    Ok(())
}

struct Reply {
    status: u16,
    body: Vec<u8>,
    content_type: &'static str,
}

fn json_reply(status: u16, v: &Value) -> Reply {
    Reply {
        status,
        body: serde_json::to_vec(v).unwrap_or_default(),
        content_type: "application/json; charset=utf-8",
    }
}

fn handle(c: &Client, places: &Places, web: &Path, mut req: Request) {
    let t0 = Instant::now();
    let url = reqwest::Url::parse(&format!("http://local{}", req.url()))
        .unwrap_or_else(|_| reqwest::Url::parse("http://local/").unwrap());
    let q: HashMap<String, String> = url.query_pairs().into_owned().collect();
    let path = url.path().to_string();
    let reply = if path.starts_with("/api/") || path == "/healthz" {
        let mut body = String::new();
        if *req.method() == Method::Post {
            let _ = req.as_reader().take(1 << 20).read_to_string(&mut body);
        }
        match api(c, places, req.method(), &path, &q, &body) {
            Ok(v) => json_reply(200, &v),
            Err(e) => {
                let status = if e.to_string().starts_with("bad request") {
                    400
                } else {
                    502
                };
                json_reply(status, &json!({ "error": format!("{e:#}") }))
            }
        }
    } else {
        static_file(web, &path)
    };
    // Docker's health check every 30 s would drown out the requests worth reading.
    if !(path == "/healthz" && reply.status == 200) {
        eprintln!(
            "{} {} {} → {} ({} ms)",
            client(&req),
            req.method(),
            req.url(),
            reply.status,
            t0.elapsed().as_millis()
        );
    }
    let mut resp = Response::from_data(reply.body).with_status_code(reply.status);
    resp.add_header(Header::from_bytes("Content-Type", reply.content_type).unwrap());
    resp.add_header(Header::from_bytes("Cache-Control", "no-cache").unwrap());
    let _ = req.respond(resp);
}

/// Who asked, for the request log: the visitor's IP and country as Cloudflare passes them on
/// (public hostname), else the peer address and "--" (LAN). Anyone on the LAN can forge
/// the headers; it's only for reading logs.
fn client(req: &Request) -> String {
    let header = |name: &'static str| {
        req.headers()
            .iter()
            .find(|h| h.field.equiv(name))
            .map(|h| h.value.as_str().to_string())
    };
    match header("CF-Connecting-IP") {
        Some(ip) => format!(
            "{ip} {}",
            header("CF-IPCountry").unwrap_or_else(|| "??".into())
        ),
        None => match req.remote_addr() {
            Some(a) => format!("{} --", a.ip()),
            None => "- --".into(),
        },
    }
}

fn static_file(web: &Path, path: &str) -> Reply {
    let rel = path.trim_start_matches('/');
    let rel = if rel.is_empty() { "index.html" } else { rel };
    let not_found = || Reply {
        status: 404,
        body: b"not found".to_vec(),
        content_type: "text/plain",
    };
    if rel.split('/').any(|p| p == ".." || p.starts_with('.')) {
        return not_found();
    }
    let file: PathBuf = web.join(rel);
    let Ok(mut body) = std::fs::read(&file) else {
        return not_found();
    };
    if rel == "index.html" {
        body = versioned_assets(web, &body);
    }
    let content_type = match file.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("json") => "application/json",
        Some("webmanifest") => "application/manifest+json",
        _ => "application/octet-stream",
    };
    Reply {
        status: 200,
        body,
        content_type,
    }
}

/// Point index.html at `app.js?v=<content hash>` (and app.css): Cloudflare turns our
/// `no-cache` into a 4 h browser cache for .js/.css, so without a new URL a phone keeps
/// running the old script against the new page after a deploy.
fn versioned_assets(web: &Path, html: &[u8]) -> Vec<u8> {
    use std::hash::{Hash, Hasher};
    let mut html = String::from_utf8_lossy(html).into_owned();
    for (attr, name) in [("href", "app.css"), ("src", "app.js")] {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        std::fs::read(web.join(name))
            .unwrap_or_default()
            .hash(&mut h);
        let v = format!("{:x}", h.finish() & 0xffff_ffff);
        html = html.replace(
            &format!("{attr}=\"{name}\""),
            &format!("{attr}=\"{name}?v={v}\""),
        );
    }
    html.into_bytes()
}

fn bad(msg: &str) -> anyhow::Error {
    anyhow!("bad request: {msg}")
}

fn coord(q: &HashMap<String, String>, key: &str) -> Result<(f64, f64)> {
    q.get(key)
        .and_then(|s| planner::parse_coord(s))
        .ok_or_else(|| bad(&format!("{key} must be lat,lng")))
}

fn num<T: std::str::FromStr>(q: &HashMap<String, String>, key: &str) -> Result<Option<T>> {
    match q.get(key).filter(|s| !s.is_empty()) {
        None => Ok(None),
        Some(s) => s
            .parse()
            .map(Some)
            .map_err(|_| bad(&format!("{key} is not a number"))),
    }
}

fn api(
    c: &Client,
    places: &Places,
    method: &Method,
    path: &str,
    q: &HashMap<String, String>,
    body: &str,
) -> Result<Value> {
    match (method, path) {
        (Method::Get, "/healthz") => Ok(json!({ "ok": c.health().unwrap_or(false) })),
        (Method::Get, "/api/config") => Ok(json!({
            "center": [51.05, 4.25],
            "timezone": "Europe/Brussels",
            "bounds": { "southwest": [49.5, 2.55], "northeast": [51.5, 6.4] },
        })),
        (Method::Get, "/api/search") => {
            let text = q.get("q").map(|s| s.trim()).unwrap_or_default();
            if text.chars().count() < 2 {
                return Ok(json!([]));
            }
            let near = q.get("near").and_then(|s| planner::parse_coord(s));
            Ok(json!(places.search(c, text, near)?))
        }
        (Method::Get, "/api/resolve") => {
            let id = q
                .get("id")
                .filter(|s| !s.is_empty())
                .ok_or_else(|| bad("id"))?;
            Ok(json!(places.resolve(id)?))
        }
        (Method::Get, "/api/reverse") => {
            let lat = num::<f64>(q, "lat")?.ok_or_else(|| bad("lat"))?;
            let lng = num::<f64>(q, "lng")?.ok_or_else(|| bad("lng"))?;
            Ok(match places.reverse(c, lat, lng)? {
                Some(p) => json!({ "name": p.name, "address": p.address }),
                None => json!({ "name": null }),
            })
        }
        (Method::Get, "/api/journeys") => journeys(c, q),
        (Method::Post, "/api/live") => live(c, body),
        (Method::Post, "/api/ride") => ride(c, body),
        _ => bail!("bad request: no such endpoint {path}"),
    }
}

fn journeys(c: &Client, q: &HashMap<String, String>) -> Result<Value> {
    let (a, b) = (coord(q, "start")?, coord(q, "end")?);
    let carry = match q.get("mode") {
        None => Carry::Folding,
        Some(m) => Carry::parse(m).ok_or_else(|| bad("mode must be foot, folding or shared"))?,
    };
    let mut r = planner::Request::new(
        format!("{},{}", a.0, a.1),
        format!("{},{}", b.0, b.1),
        carry,
    );
    r.time = match q.get("time").filter(|s| !s.is_empty()) {
        None => None,
        Some(t) => Some(
            DateTime::parse_from_rfc3339(t)
                .map_err(|_| bad("time must be RFC 3339"))?
                .with_timezone(&Utc),
        ),
    };
    r.arrive_by = q.get("arrive").is_some_and(|v| v == "1" || v == "true");
    r.window_min = num(q, "window")?.unwrap_or(60).clamp(10, 240);
    r.max_bike_min = num(q, "maxbike")?.unwrap_or(45).clamp(5, 120);
    r.ride_kmh = num(q, "kmh")?.unwrap_or(18.0_f64).clamp(8.0, 40.0);
    r.fast_kmh = num::<f64>(q, "fast")?.map(|v| v.clamp(8.0, 45.0));
    r.bike = own_bike(q.get("bike"))?;
    let p = planner::plan_full(c, &r)?;
    Ok(json!({ "journeys": p.journeys, "warnings": p.warnings }))
}

fn own_bike(v: Option<&String>) -> Result<OwnBike> {
    v.map_or(Some(OwnBike::None), |s| OwnBike::parse(s))
        .ok_or_else(|| bad("bike must be here or lat,lng (where it's parked)"))
}

#[derive(Deserialize)]
struct LiveRequest {
    trips: Vec<String>,
}

/// Each trip as MOTIS knows it now: one leg from its first to its last stop.
fn live(c: &Client, body: &str) -> Result<Value> {
    let r: LiveRequest =
        serde_json::from_str(body).map_err(|_| bad("expected {\"trips\": [tripId…]}"))?;
    let mut out: HashMap<String, Leg> = HashMap::new();
    for id in r.trips.into_iter().take(40) {
        if out.contains_key(&id) {
            continue;
        }
        match c.trip(&id) {
            Ok(it) => {
                if let Some(mut leg) = it.legs.into_iter().next() {
                    crate::irail::fill_tracks(&mut leg, Utc::now());
                    out.insert(id, leg);
                }
            }
            Err(e) => eprintln!("  live: trip {id}: {e}"),
        }
    }
    Ok(json!({ "trips": out, "at": Utc::now() }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RideRequest {
    mode: Option<String>,
    /// Destination, "lat,lng".
    to: String,
    /// Where your own bike is parked ("lat,lng"), if the route ends on it. A bike you had
    /// at the start is parked behind you by now, so it's never sent.
    bike: Option<String>,
    #[serde(default)]
    changes: Vec<ChangeRequest>,
    #[serde(default)]
    rentals: Vec<RentalRequest>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChangeRequest {
    /// Where you'd be stranded: the stop you got off at (stop id) or "lat,lng".
    from: String,
    /// When the connection you'd miss leaves (live); plan B leaves after it.
    after: DateTime<Utc>,
    /// The trip you'd have missed, so it's never offered as its own plan B.
    avoid_trip: Option<String>,
    /// The trip that brings you to the change, and the stop you boarded it at (stop id or
    /// "lat,lng"): staying on it to a later stop is a plan B too.
    on_trip: Option<String>,
    board: Option<String>,
}

/// How many stops past a change ride mode tries for "stay on and change there".
const STAY_ON_STOPS: usize = 5;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RentalRequest {
    lat: f64,
    lng: f64,
    provider: String,
}

/// Earliest home first, then fewer rides (ties go to the first candidate).
fn rank_plan_b(j: &planner::Journey) -> (DateTime<Utc>, usize) {
    let rides = j.it.legs.iter().filter(|l| l.is_transit()).count();
    (j.it.end_time, rides)
}

/// Plan B by getting off at the change: anything leaving that stop after the missed ride.
fn plan_b_here(
    c: &Client,
    ch: &ChangeRequest,
    to: &str,
    carry: Carry,
    bike: OwnBike,
) -> Option<planner::Journey> {
    let mut q = planner::Request::new(ch.from.clone(), to.to_string(), carry);
    q.time = Some(ch.after + chrono::Duration::seconds(60));
    q.bike = bike;
    q.window_min = 90;
    let js = planner::plan(c, &q)
        .map_err(|e| eprintln!("  ride: plan B from {}: {e:#}", ch.from))
        .ok()?;
    js.into_iter()
        .filter(|j| {
            let first = j.it.legs.iter().find(|l| l.is_transit());
            first.is_some_and(|l| {
                l.trip_id.is_some() && l.trip_id != ch.avoid_trip && l.trip_id != ch.on_trip
            }) || (first.is_none() && j.it.legs.iter().any(|l| l.mode != "WALK"))
        })
        .min_by_key(|j| (j.it.end_time, -j.it.start_time.timestamp()))
}

/// Plan B by staying on the ride that brings you to the change and changing at one of the
/// next stops (MOTIS can't start a search aboard a trip; ADR-025). The journey starts with
/// that ride from where you boarded it, so it replaces your current leg.
fn plan_b_stay_on(
    c: &Client,
    ch: &ChangeRequest,
    to: &str,
    carry: Carry,
    bike: OwnBike,
) -> Option<planner::Journey> {
    let (trip_id, board) = (ch.on_trip.as_deref()?, ch.board.as_deref()?);
    let trip = c
        .trip(trip_id)
        .map_err(|e| eprintln!("  ride: stay on {trip_id}: {e:#}"))
        .ok()?
        .legs
        .into_iter()
        .next()?;
    let a = crate::overshoot::stop_index(&trip, board)?;
    let k = crate::overshoot::stop_index(&trip, &ch.from)?;
    if k <= a {
        return None;
    }
    let places = crate::overshoot::trip_places(&trip);
    let stops: Vec<(usize, String, DateTime<Utc>)> = (k + 1..places.len())
        .take(STAY_ON_STOPS)
        .filter_map(|b| {
            let p = places[b];
            let id = p.stop_id.clone().unwrap_or(format!("{},{}", p.lat, p.lon));
            Some((b, id, p.arrival.or(p.scheduled_arrival)?))
        })
        .collect();
    std::thread::scope(|s| {
        let handles: Vec<_> = stops
            .iter()
            .map(|(b, id, arr)| {
                let (trip, places) = (&trip, &places);
                s.spawn(move || -> Option<planner::Journey> {
                    let mut q = planner::Request::new(id.clone(), to.to_string(), carry);
                    q.time = Some(*arr);
                    q.bike = bike;
                    q.window_min = 60;
                    let js = planner::plan(c, &q)
                        .map_err(|e| eprintln!("  ride: plan B from {id}: {e:#}"))
                        .ok()?;
                    // Going on with this same trip is a later stop's candidate, not this one's.
                    let on = js
                        .into_iter()
                        .filter(|j| {
                            let first = j.it.legs.iter().find(|l| l.is_transit());
                            first.map_or(!j.it.legs.is_empty(), |l| {
                                l.trip_id.as_deref() != Some(trip_id)
                            })
                        })
                        .min_by_key(|j| j.it.end_time)?;
                    let ride = crate::overshoot::slice(trip, a, *b)?;
                    let mut legs = vec![ride];
                    legs.extend(on.it.legs);
                    Some(planner::Journey {
                        label: Some(format!("Stay on to {}", places[*b].name)),
                        notes: on.notes,
                        it: crate::motis::Itinerary::from_legs(legs),
                    })
                })
            })
            .collect();
        handles
            .into_iter()
            .filter_map(|h| h.join().ok().flatten())
            .min_by_key(rank_plan_b)
    })
}

/// Ride mode extras. For every change still ahead: the best way on if you miss it (earliest
/// arrival, leaving that stop after the missed departure). For every shared-bike pickup:
/// how many bikes the nearest station of that operator has now.
fn ride(c: &Client, body: &str) -> Result<Value> {
    let r: RideRequest =
        serde_json::from_str(body).map_err(|e| bad(&format!("ride request: {e}")))?;
    let carry = match r.mode.as_deref() {
        None => Carry::Folding,
        Some(m) => Carry::parse(m).ok_or_else(|| bad("mode"))?,
    };
    if planner::parse_coord(&r.to).is_none() {
        return Err(bad("to must be lat,lng"));
    }
    let bike = match own_bike(r.bike.as_ref())? {
        OwnBike::Here => OwnBike::None,
        b => b,
    };
    let changes: Vec<ChangeRequest> = r.changes.into_iter().take(4).collect();
    // Per change, in parallel: get off and go on from that stop, or stay on and change later.
    let fallbacks: Vec<Option<planner::Journey>> = std::thread::scope(|scope| {
        let handles: Vec<_> = changes
            .iter()
            .map(|ch| {
                let to = r.to.as_str();
                scope.spawn(move || {
                    std::thread::scope(|s| {
                        let on = s.spawn(|| plan_b_stay_on(c, ch, to, carry, bike));
                        let here = s.spawn(|| plan_b_here(c, ch, to, carry, bike));
                        // Staying on first: on a tie it's the calmer plan B.
                        [on, here]
                            .into_iter()
                            .filter_map(|h| h.join().ok().flatten())
                            .min_by_key(rank_plan_b)
                    })
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().ok().flatten())
            .collect()
    });
    let street = crate::lastmile::Street::new(c);
    let rentals: Vec<Value> = r
        .rentals
        .iter()
        .take(4)
        .map(|q| {
            let near = street.stations_near(q.lat, q.lng, 150).unwrap_or_default();
            let best = near
                .into_iter()
                .filter(|s| s.provider_id == q.provider)
                .min_by(|a, b| {
                    let d = |s: &crate::lastmile::RentalStation| {
                        (s.lat - q.lat).powi(2) + (s.lon - q.lng).powi(2)
                    };
                    d(a).total_cmp(&d(b))
                });
            match best {
                Some(s) => json!({ "name": s.name, "bikes": s.num_vehicles_available }),
                None => Value::Null,
            }
        })
        .collect();
    Ok(json!({ "fallbacks": fallbacks, "rentals": rentals, "at": Utc::now() }))
}
