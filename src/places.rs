//! Place search for the web UI: citymapper.com first, the MOTIS geocoder as fallback (ADR-017).
//!
//! Citymapper's search is Google's (venues, shops, hamlets, house numbers); the MOTIS
//! geocoder only knows what OSM tags. Citymapper's endpoints are the website's own:
//! undocumented, no key, and they may change or block us any day. So this is their only
//! runtime use (planning never touches them), requests are paced and cached like
//! bike-transit did, and every failure falls back to MOTIS.

use crate::motis::{self, GeocodeMatch};
use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const UPSTREAM: &str = "https://citymapper.com";
/// Citymapper's Brussels region covers all of Belgium.
const REGION: &str = "be-brussels";
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0 Safari/537.36";
const MIN_INTERVAL: Duration = Duration::from_millis(350);
const MAX_RESULTS: usize = 8;
/// Furthest a venue's street address may be from its pin to be used instead (ADR-021).
const MAX_STREET_SNAP_M: f64 = 500.0;

/// One place, in the shape the UI's autocomplete expects (bike-transit's).
#[derive(Debug, Clone, Serialize)]
pub struct Place {
    pub id: Option<String>,
    pub name: String,
    pub address: Option<String>,
    /// [lat, lon]. Citymapper's Google hits come without; the UI fetches them from
    /// `/api/resolve` when one is picked.
    pub coords: Option<[f64; 2]>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    /// For a venue: the street address its coordinates were moved to (ADR-021).
    #[serde(rename = "routedTo", skip_serializing_if = "Option::is_none")]
    pub routed_to: Option<String>,
}

impl From<&GeocodeMatch> for Place {
    fn from(m: &GeocodeMatch) -> Self {
        Place {
            id: m.id.clone(),
            name: m.label(),
            address: m.town().map(str::to_string),
            coords: Some([m.lat, m.lon]),
            kind: Some(m.kind.clone()),
            routed_to: None,
        }
    }
}

pub struct Places {
    citymapper: Option<Citymapper>,
}

impl Places {
    /// `use_citymapper: false` is the kill switch: MOTIS only.
    pub fn new(use_citymapper: bool) -> Self {
        Self {
            citymapper: use_citymapper.then(Citymapper::new),
        }
    }

    pub fn search(
        &self,
        c: &motis::Client,
        text: &str,
        near: Option<(f64, f64)>,
    ) -> Result<Vec<Place>> {
        if let Some(cm) = &self.citymapper {
            match cm.search(text, near) {
                // A venue's pin can sit deep inside its grounds: leave its coordinates out so
                // the UI resolves it, which moves it to the street address (ADR-021).
                Ok(mut hits) if !hits.is_empty() => {
                    for h in &mut hits {
                        if is_venue(h) && h.address.as_deref().and_then(street_of).is_some() {
                            h.coords = None;
                        }
                    }
                    return Ok(hits);
                }
                Ok(_) => eprintln!("  search: Citymapper has nothing for '{text}', trying MOTIS"),
                Err(e) => eprintln!("  search: Citymapper failed, using MOTIS: {e:#}"),
            }
        }
        Ok(c.geocode_near(text, near)?
            .iter()
            .take(MAX_RESULTS)
            .map(Place::from)
            .collect())
    }

    /// Coordinates for a Citymapper search hit (MOTIS hits always carry their own). A venue
    /// goes to its street address when that's close: Google's venue pin can be inside private
    /// grounds that MOTIS can't reach after public transport, the address is on the street
    /// (ADR-021). Addresses and dropped pins stay where they are.
    pub fn resolve(&self, id: &str) -> Result<Place> {
        let Some(cm) = &self.citymapper else {
            anyhow::bail!("bad request: place ids need Citymapper, which is switched off")
        };
        let mut p = cm.resolve(id)?;
        if is_venue(&p) {
            match cm.street_point(&p) {
                Ok(Some((street, at))) => {
                    p.coords = Some(at);
                    p.routed_to = Some(street);
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!("  resolve: no street address for {id}, keeping its pin: {e:#}")
                }
            }
        }
        Ok(p)
    }

    /// A name for a dropped pin or the current location.
    pub fn reverse(&self, c: &motis::Client, lat: f64, lon: f64) -> Result<Option<Place>> {
        if let Some(cm) = &self.citymapper {
            match cm.reverse(lat, lon) {
                Ok(Some(p)) => return Ok(Some(p)),
                Ok(None) => {
                    eprintln!("  reverse: Citymapper has no address at {lat},{lon}, trying MOTIS")
                }
                Err(e) => eprintln!("  reverse: Citymapper failed, using MOTIS: {e:#}"),
            }
        }
        Ok(c.reverse_geocode(lat, lon)?.first().map(Place::from))
    }
}

/// Paced, cached access to citymapper.com.
struct Citymapper {
    http: reqwest::blocking::Client,
    /// When the last request went out; held while pacing so requests queue up.
    last: Mutex<Option<Instant>>,
    cache: Mutex<HashMap<String, (Instant, Value)>>,
}

impl Citymapper {
    fn new() -> Self {
        Self {
            http: reqwest::blocking::Client::builder()
                // Short: a hanging Citymapper should fall back to MOTIS while you type.
                .timeout(Duration::from_secs(8))
                .user_agent(USER_AGENT)
                .build()
                .expect("http client"),
            last: Mutex::new(None),
            cache: Mutex::new(HashMap::new()),
        }
    }

    fn fetch(&self, req: reqwest::blocking::RequestBuilder, ttl: Duration) -> Result<Value> {
        let req = req
            .header("Accept", "application/json")
            .header("Referer", format!("{UPSTREAM}/directions"))
            .build()?;
        let body = req.body().and_then(|b| b.as_bytes()).unwrap_or_default();
        let key = format!(
            "{} {} {}",
            req.method(),
            req.url(),
            String::from_utf8_lossy(body)
        );
        if let Some((until, v)) = self.cache.lock().unwrap().get(&key) {
            if *until > Instant::now() {
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
        let path = req.url().path().to_string();
        let v: Value = self
            .http
            .execute(req)
            .with_context(|| format!("Citymapper {path}"))?
            .error_for_status()?
            .json()
            .with_context(|| format!("decoding Citymapper {path}"))?;
        let mut cache = self.cache.lock().unwrap();
        if cache.len() > 1000 {
            let now = Instant::now();
            cache.retain(|_, (until, _)| *until > now);
        }
        cache.insert(key, (Instant::now() + ttl, v.clone()));
        Ok(v)
    }

    fn search(&self, text: &str, near: Option<(f64, f64)>) -> Result<Vec<Place>> {
        let mut body = json!({
            "query": text,
            "region_id": REGION,
            "is_refinement": true,
            "refinement_data": { "is_chain": true },
            "powersearch": false,
        });
        if let Some((lat, lon)) = near {
            body["location"] = json!(format!("{lat},{lon}"));
        }
        // JSON under a form content type: that's what citymapper.com's own page sends.
        let req = self
            .http
            .post(format!("{UPSTREAM}/api/4/searchpost"))
            .header(
                "Content-Type",
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .header("Citymapper-Client", "Web Search")
            .header("Citymapper-Region", REGION)
            .body(body.to_string());
        Ok(parse_search(&self.fetch(req, Duration::from_secs(300))?))
    }

    fn resolve(&self, id: &str) -> Result<Place> {
        let req = self
            .http
            .get(format!("{UPSTREAM}/api/1/placeresolve"))
            .query(&[("id", id), ("region_id", REGION)]);
        parse_resolve(&self.fetch(req, Duration::from_secs(86400))?)
            .with_context(|| format!("Citymapper has no coordinates for {id}"))
    }

    /// Where a venue's street address is, if it has one with a house number near its pin.
    fn street_point(&self, venue: &Place) -> Result<Option<(String, [f64; 2])>> {
        let (Some(addr), Some(pin)) = (venue.address.as_deref(), venue.coords) else {
            return Ok(None);
        };
        let Some(street) = street_of(addr) else {
            return Ok(None);
        };
        let hits = self.search(addr, Some((pin[0], pin[1])))?;
        let Some(hit) = match_street(&hits, street) else {
            return Ok(None);
        };
        let at = match (hit.coords, hit.id.as_deref()) {
            (Some(c), _) => c,
            (None, Some(id)) => match self.resolve(id)?.coords {
                Some(c) => c,
                None => return Ok(None),
            },
            (None, None) => return Ok(None),
        };
        Ok((meters(pin, at) <= MAX_STREET_SNAP_M).then(|| (street.to_string(), at)))
    }

    fn reverse(&self, lat: f64, lon: f64) -> Result<Option<Place>> {
        let req = self
            .http
            .get(format!("{UPSTREAM}/geocode"))
            .query(&[("lat", lat.to_string()), ("lng", lon.to_string())]);
        Ok(parse_reverse(
            &self.fetch(req, Duration::from_secs(86400))?,
            lat,
            lon,
        ))
    }
}

fn is_venue(p: &Place) -> bool {
    p.kind.as_deref() == Some("place")
}

/// "Boeretang 200, 2400 Mol, Belgium" → "Boeretang 200": the street and house number, if
/// the address starts with one ("2400 Mol, Belgium" and "Plattebeekstraat, …" don't).
fn street_of(address: &str) -> Option<&str> {
    let first = address.split(',').next()?.trim();
    let mut words = first.split_whitespace();
    let number = words.next_back()?;
    (words.next().is_some() && number.starts_with(|c: char| c.is_ascii_digit())).then_some(first)
}

/// The address hit for exactly this street and number.
fn match_street<'a>(hits: &'a [Place], street: &str) -> Option<&'a Place> {
    let norm = |s: &str| {
        s.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    };
    hits.iter()
        .find(|h| h.kind.as_deref() == Some("address") && norm(&h.name) == norm(street))
}

fn meters(a: [f64; 2], b: [f64; 2]) -> f64 {
    let dx = (a[1] - b[1]).to_radians() * a[0].to_radians().cos();
    let dy = (a[0] - b[0]).to_radians();
    6_371_000.0 * (dx * dx + dy * dy).sqrt()
}

fn str_of(v: &Value, key: &str) -> Option<String> {
    v.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn coords_of(v: &Value) -> Option<[f64; 2]> {
    let c = v.get("coords")?.as_array()?;
    Some([c.first()?.as_f64()?, c.get(1)?.as_f64()?])
}

/// `searchpost` mixes section headings and attributions in with the hits.
fn parse_search(v: &Value) -> Vec<Place> {
    let results = v.get("results").and_then(Value::as_array);
    results
        .into_iter()
        .flatten()
        .filter(|r| {
            !matches!(
                r.get("result_type").and_then(Value::as_str),
                Some("heading" | "attribution")
            )
        })
        .filter_map(|r| {
            Some(Place {
                id: str_of(r, "id"),
                name: str_of(r, "name")?,
                address: str_of(r, "address"),
                coords: coords_of(r),
                kind: str_of(r, "type"),
                routed_to: None,
            })
        })
        .take(MAX_RESULTS)
        .collect()
}

fn parse_resolve(v: &Value) -> Option<Place> {
    let p = v.get("place_result")?;
    Some(Place {
        id: str_of(p, "id"),
        name: str_of(p, "name")?,
        address: str_of(p, "address"),
        coords: Some(coords_of(p)?),
        kind: str_of(p, "type"),
        routed_to: None,
    })
}

/// `/geocode` is Google's reverse geocoder: "Street 12" and the town, like MOTIS's.
fn parse_reverse(v: &Value, lat: f64, lon: f64) -> Option<Place> {
    let first = v.get("results")?.as_array()?.first()?;
    let mut parts: HashMap<&str, &str> = HashMap::new();
    for c in first.get("address_components")?.as_array()? {
        let Some(name) = c.get("long_name").and_then(Value::as_str) else {
            continue;
        };
        for t in c
            .get("types")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(t) = t.as_str() {
                parts.insert(t, name);
            }
        }
    }
    let street = [parts.get("route"), parts.get("street_number")]
        .into_iter()
        .flatten()
        .copied()
        .collect::<Vec<_>>()
        .join(" ");
    let name = if street.is_empty() {
        str_of(first, "formatted_address")?
    } else {
        street
    };
    Some(Place {
        id: None,
        name,
        address: parts.get("locality").map(|s| s.to_string()),
        coords: Some([lat, lon]),
        kind: None,
        routed_to: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_skips_headings_and_keeps_hits_without_coords() {
        let v: Value = serde_json::from_str(
            r#"{"results":[
              {"id":"stations","name":"Stations","result_type":"heading"},
              {"id":"citymapper:BrusselsStation_Gandsaintpierre","name":"Gent-Sint-Pieters",
               "coords":[51.0359,3.710673],"type":"station"},
              {"id":"google:ChIJva4ZuTbkw0cRrAlOkIgmszA","name":"Nekkerhal - Mechelen",
               "address":"Plattebeekstraat, Mechelen, Belgium","type":"place","result_type":"autocomplete"},
              {"id":"results","name":"Results","result_type":"attribution"}]}"#,
        )
        .unwrap();
        let hits = parse_search(&v);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].coords, Some([51.0359, 3.710673]));
        assert_eq!(hits[1].name, "Nekkerhal - Mechelen");
        assert_eq!(hits[1].coords, None);
    }

    #[test]
    fn resolve_needs_coordinates() {
        let v: Value = serde_json::from_str(
            r#"{"place_result":{"address":"Plattebeekstraat 1, 2800 Mechelen, Belgium",
              "coords":[51.027481,4.498251],"id":"google:ChIJva4ZuTbkw0cRrAlOkIgmszA",
              "name":"Plattebeekstraat 1, 2800 Mechelen, Belgium","type":"place"}}"#,
        )
        .unwrap();
        assert_eq!(
            parse_resolve(&v).unwrap().coords,
            Some([51.027481, 4.498251])
        );
        assert!(parse_resolve(&json!({"place_result": {"name": "x"}})).is_none());
    }

    #[test]
    fn venues_use_their_street_address() {
        assert_eq!(
            street_of("Boeretang 200, 2400 Mol, Belgium"),
            Some("Boeretang 200")
        );
        assert_eq!(
            street_of("Rue de la Loi 16A, 1000 Bruxelles"),
            Some("Rue de la Loi 16A")
        );
        assert_eq!(street_of("2400 Mol, Belgium"), None);
        assert_eq!(street_of("Plattebeekstraat, Mechelen, Belgium"), None);
        assert_eq!(street_of("200"), None);
        // What Citymapper returns for the address of VITO's pin (2026-11-10).
        let v: Value = serde_json::from_str(
            r#"{"results":[
              {"id":"google:ChIJD47WNkM1wUcRjNc15e4Q5B0","name":"Boeretang 200",
               "address":"2400 Mol, Belgium","type":"address"},
              {"id":"google:x","name":"Boeretang 200","address":"Mol","type":"place"}]}"#,
        )
        .unwrap();
        let hits = parse_search(&v);
        let hit = match_street(&hits, "boeretang  200").unwrap();
        assert_eq!(
            hit.id.as_deref(),
            Some("google:ChIJD47WNkM1wUcRjNc15e4Q5B0")
        );
        assert!(match_street(&hits, "Boeretang 2").is_none());
        // The pin and the address point: ~200 m apart.
        let d = meters([51.218673, 5.090817], [51.219023, 5.093538]);
        assert!((150.0..250.0).contains(&d), "{d}");
    }

    #[test]
    fn reverse_names_street_and_town() {
        let v: Value = serde_json::from_str(
            r#"{"status":"OK","results":[{"formatted_address":"Markt 72, 9160 Lokeren, Belgium",
              "address_components":[
                {"long_name":"72","types":["street_number"]},
                {"long_name":"Markt","types":["route"]},
                {"long_name":"Lokeren","types":["locality","political"]}]}]}"#,
        )
        .unwrap();
        let p = parse_reverse(&v, 51.1, 3.99).unwrap();
        assert_eq!(
            (p.name.as_str(), p.address.as_deref()),
            ("Markt 72", Some("Lokeren"))
        );
        assert!(
            parse_reverse(&json!({"status": "ZERO_RESULTS", "results": []}), 0.0, 0.0).is_none()
        );
    }
}
