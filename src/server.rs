//! `overstap serve`: the web UI (static files from `web/`) and a small JSON API over MOTIS.
//!
//! | Endpoint | What |
//! |---|---|
//! | `GET /api/config` | map centre, bounds, time zone |
//! | `GET /api/search?q=&near=lat,lng` | place search (MOTIS geocoder) |
//! | `GET /api/reverse?lat=&lng=` | name for a dropped pin |
//! | `GET /api/journeys?start=&end=&mode=&time=&arrive=&window=&kmh=&fast=&maxbike=` | journeys, MOTIS-shaped |
//! | `POST /api/live {"trips":[tripId…]}` | current state of those trips (live times, tracks, cancellations) |
//! | `POST /api/ride {mode,to,changes,rentals}` | ride mode: plan B per change, bikes at rental pickups |
//! | `GET /healthz` | MOTIS reachable? |
//!
//! Blocking and threaded (tiny_http + reqwest::blocking): one rider, a handful of requests.

use crate::motis::{Client, Leg};
use crate::planner::{self, Carry};
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

pub fn run(c: Client, listen: &str, web: &Path) -> Result<()> {
    let web = web
        .canonicalize()
        .with_context(|| format!("web directory {} not found", web.display()))?;
    let server = Arc::new(Server::http(listen).map_err(|e| anyhow!("listen on {listen}: {e}"))?);
    let c = Arc::new(c);
    eprintln!(
        "overstap: http://{listen} (MOTIS {}, web {})",
        c.base_url(),
        web.display()
    );
    let workers: Vec<_> = (0..THREADS)
        .map(|_| {
            let (server, c, web) = (server.clone(), c.clone(), web.clone());
            std::thread::spawn(move || {
                for req in server.incoming_requests() {
                    handle(&c, &web, req);
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

fn handle(c: &Client, web: &Path, mut req: Request) {
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
        match api(c, req.method(), &path, &q, &body) {
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
    eprintln!(
        "{} {} → {} ({} ms)",
        req.method(),
        req.url(),
        reply.status,
        t0.elapsed().as_millis()
    );
    let mut resp = Response::from_data(reply.body).with_status_code(reply.status);
    resp.add_header(Header::from_bytes("Content-Type", reply.content_type).unwrap());
    resp.add_header(Header::from_bytes("Cache-Control", "no-cache").unwrap());
    let _ = req.respond(resp);
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
    let Ok(body) = std::fs::read(&file) else {
        return not_found();
    };
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
        (Method::Get, "/api/search") => search(c, q),
        (Method::Get, "/api/reverse") => {
            let lat = num::<f64>(q, "lat")?.ok_or_else(|| bad("lat"))?;
            let lng = num::<f64>(q, "lng")?.ok_or_else(|| bad("lng"))?;
            let hits = c.reverse_geocode(lat, lng)?;
            Ok(match hits.first() {
                Some(m) => json!({ "name": m.label(), "address": m.town() }),
                None => json!({ "name": null }),
            })
        }
        (Method::Get, "/api/journeys") => journeys(c, q),
        (Method::Post, "/api/live") => live(c, body),
        (Method::Post, "/api/ride") => ride(c, body),
        _ => bail!("bad request: no such endpoint {path}"),
    }
}

fn search(c: &Client, q: &HashMap<String, String>) -> Result<Value> {
    let text = q.get("q").map(|s| s.trim()).unwrap_or_default();
    if text.len() < 2 {
        return Ok(json!([]));
    }
    let near = q.get("near").and_then(|s| planner::parse_coord(s));
    let hits = c.geocode_near(text, near)?;
    Ok(Value::Array(
        hits.iter()
            .take(8)
            .map(|m| {
                json!({
                    "id": m.id,
                    "name": m.label(),
                    "address": m.town(),
                    "coords": [m.lat, m.lon],
                    "type": m.kind,
                })
            })
            .collect(),
    ))
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
    let js = planner::plan(c, &r)?;
    Ok(json!({ "journeys": js }))
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
                if let Some(leg) = it.legs.into_iter().next() {
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
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RentalRequest {
    lat: f64,
    lng: f64,
    provider: String,
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
    let changes: Vec<ChangeRequest> = r.changes.into_iter().take(4).collect();
    // One planner run per change, in parallel: each is a few MOTIS calls.
    let fallbacks: Vec<Option<planner::Journey>> = std::thread::scope(|scope| {
        let handles: Vec<_> = changes
            .iter()
            .map(|ch| {
                let to = r.to.clone();
                scope.spawn(move || -> Option<planner::Journey> {
                    let mut q = planner::Request::new(ch.from.clone(), to, carry);
                    q.time = Some(ch.after + chrono::Duration::seconds(60));
                    q.window_min = 90;
                    let js = planner::plan(c, &q)
                        .map_err(|e| eprintln!("  ride: plan B from {}: {e:#}", ch.from))
                        .ok()?;
                    js.into_iter()
                        .filter(|j| {
                            let first = j.it.legs.iter().find(|l| l.is_transit());
                            first.is_some_and(|l| l.trip_id.is_some() && l.trip_id != ch.avoid_trip)
                                || (first.is_none() && j.it.legs.iter().any(|l| l.mode != "WALK"))
                        })
                        .min_by_key(|j| (j.it.end_time, -j.it.start_time.timestamp()))
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
