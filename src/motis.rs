//! Typed client for the subset of the MOTIS 2 REST API that overstap uses.
//! Source of truth: `openapi.yaml` in motis-project/motis (tested against v2.11.4).

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;

pub struct Client {
    base_url: String,
    http: reqwest::blocking::Client,
}

/// Query for `/api/v6/plan`. Only the knobs we actually use are modelled;
/// `extra` lets callers pass anything else from the OpenAPI spec.
#[derive(Debug, Clone)]
pub struct PlanQuery {
    /// `lat,lon` or a stop id (e.g. `sncb_8821006`).
    pub from: String,
    pub to: String,
    pub time: Option<DateTime<Utc>>,
    pub arrive_by: bool,
    pub search_window_s: u32,
    pub max_transfers: Option<u32>,
    /// MOTIS default is 0 → use the feed's own transfer times. This is our
    /// lever against NMBS's conservative change times.
    pub min_transfer_time_min: Option<u32>,
    pub additional_transfer_time_min: Option<u32>,
    pub transit_modes: Vec<String>,
    pub direct_modes: Vec<String>,
    pub pre_transit_modes: Vec<String>,
    pub post_transit_modes: Vec<String>,
    pub max_pre_transit_s: Option<u32>,
    pub max_post_transit_s: Option<u32>,
    pub max_direct_s: Option<u32>,
    /// Folding bike on board: require trips that allow bikes.
    pub require_bike_transport: bool,
    pub use_routed_transfers: bool,
    pub detailed_legs: bool,
    pub extra: Vec<(String, String)>,
}

impl PlanQuery {
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            time: None,
            arrive_by: false,
            search_window_s: 900,
            max_transfers: None,
            min_transfer_time_min: None,
            additional_transfer_time_min: None,
            transit_modes: vec![],
            direct_modes: vec![],
            pre_transit_modes: vec![],
            post_transit_modes: vec![],
            max_pre_transit_s: None,
            max_post_transit_s: None,
            max_direct_s: None,
            require_bike_transport: false,
            use_routed_transfers: false,
            detailed_legs: false,
            extra: vec![],
        }
    }

    fn params(&self) -> Vec<(String, String)> {
        let mut p: Vec<(String, String)> = vec![
            ("fromPlace".into(), self.from.clone()),
            ("toPlace".into(), self.to.clone()),
            ("arriveBy".into(), self.arrive_by.to_string()),
            ("searchWindow".into(), self.search_window_s.to_string()),
            ("detailedLegs".into(), self.detailed_legs.to_string()),
            ("language".into(), "nl".into()),
        ];
        if let Some(t) = self.time {
            p.push(("time".into(), t.to_rfc3339()));
        }
        let opt = |p: &mut Vec<(String, String)>, k: &str, v: Option<u32>| {
            if let Some(v) = v {
                p.push((k.into(), v.to_string()));
            }
        };
        opt(&mut p, "maxTransfers", self.max_transfers);
        opt(&mut p, "minTransferTime", self.min_transfer_time_min);
        opt(
            &mut p,
            "additionalTransferTime",
            self.additional_transfer_time_min,
        );
        opt(&mut p, "maxPreTransitTime", self.max_pre_transit_s);
        opt(&mut p, "maxPostTransitTime", self.max_post_transit_s);
        opt(&mut p, "maxDirectTime", self.max_direct_s);
        let list = |p: &mut Vec<(String, String)>, k: &str, v: &[String]| {
            if !v.is_empty() {
                p.push((k.into(), v.join(",")));
            }
        };
        list(&mut p, "transitModes", &self.transit_modes);
        list(&mut p, "directModes", &self.direct_modes);
        list(&mut p, "preTransitModes", &self.pre_transit_modes);
        list(&mut p, "postTransitModes", &self.post_transit_modes);
        if self.require_bike_transport {
            p.push(("requireBikeTransport".into(), "true".into()));
        }
        if self.use_routed_transfers {
            p.push(("useRoutedTransfers".into(), "true".into()));
        }
        p.extend(self.extra.iter().cloned());
        p
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanResponse {
    pub itineraries: Vec<Itinerary>,
    #[serde(default)]
    pub direct: Vec<Itinerary>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Itinerary {
    pub duration: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub transfers: i64,
    pub legs: Vec<Leg>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Leg {
    pub mode: String,
    pub from: Place,
    pub to: Place,
    pub duration: i64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub scheduled_start_time: DateTime<Utc>,
    pub scheduled_end_time: DateTime<Utc>,
    pub real_time: bool,
    #[serde(default)]
    pub distance: Option<f64>,
    #[serde(default)]
    pub headsign: Option<String>,
    #[serde(default)]
    pub route_short_name: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub trip_id: Option<String>,
    #[serde(default)]
    pub agency_name: Option<String>,
    #[serde(default)]
    pub intermediate_stops: Vec<Place>,
    #[serde(default)]
    pub rental: Option<Rental>,
    #[serde(default)]
    pub bikes_allowed: Option<bool>,
    #[serde(default)]
    pub cancelled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Place {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    #[serde(default)]
    pub stop_id: Option<String>,
    #[serde(default)]
    pub track: Option<String>,
    #[serde(default)]
    pub arrival: Option<DateTime<Utc>>,
    #[serde(default)]
    pub departure: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rental {
    pub provider_id: String,
    #[serde(default)]
    pub system_name: Option<String>,
    #[serde(default)]
    pub from_station_name: Option<String>,
    #[serde(default)]
    pub to_station_name: Option<String>,
    #[serde(default)]
    pub return_constraint: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeocodeMatch {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub id: Option<String>,
}

impl Client {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            http: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("http client"),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn health(&self) -> Result<bool> {
        let r = self
            .http
            .get(format!("{}/api/v1/health", self.base_url))
            .send()?;
        Ok(r.status().is_success())
    }

    pub fn plan(&self, q: &PlanQuery) -> Result<PlanResponse> {
        let r = self
            .http
            .get(format!("{}/api/v6/plan", self.base_url))
            .query(&q.params())
            .send()
            .context("MOTIS /plan request failed")?;
        let status = r.status();
        let body = r.text()?;
        if !status.is_success() {
            anyhow::bail!(
                "MOTIS /plan returned {status}: {}",
                &body[..body.len().min(400)]
            );
        }
        serde_json::from_str(&body).context("decoding MOTIS /plan response")
    }

    pub fn geocode(&self, text: &str) -> Result<Vec<GeocodeMatch>> {
        let r = self
            .http
            .get(format!("{}/api/v1/geocode", self.base_url))
            .query(&[("text", text), ("language", "nl")])
            .send()?
            .error_for_status()?;
        Ok(r.json()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn params_encode_lists_and_optionals() {
        let mut q = PlanQuery::new("51.1,4.0", "sncb_8822004");
        q.min_transfer_time_min = Some(1);
        q.post_transit_modes = vec!["WALK".into(), "RENTAL".into()];
        q.require_bike_transport = true;
        let p = q.params();
        assert!(p.contains(&("minTransferTime".into(), "1".into())));
        assert!(p.contains(&("postTransitModes".into(), "WALK,RENTAL".into())));
        assert!(p.contains(&("requireBikeTransport".into(), "true".into())));
        assert!(!p.iter().any(|(k, _)| k == "maxTransfers"));
    }

    #[test]
    fn decodes_minimal_plan_response() {
        let json = r#"{"itineraries":[{"duration":600,"startTime":"2026-10-29T08:00:00Z",
          "endTime":"2026-10-29T08:10:00Z","transfers":0,"id":"x","legs":[{"mode":"RAIL",
          "from":{"name":"A","lat":51.0,"lon":4.0},"to":{"name":"B","lat":51.1,"lon":4.1},
          "duration":600,"startTime":"2026-10-29T08:00:00Z","endTime":"2026-10-29T08:10:00Z",
          "scheduledStartTime":"2026-10-29T08:00:00Z","scheduledEndTime":"2026-10-29T08:10:00Z",
          "realTime":false,"scheduled":true,"legGeometry":{"points":"","precision":6,"length":0}}]}],
          "direct":[],"from":{},"to":{},"requestParameters":{},"debugOutput":{},
          "previousPageCursor":"","nextPageCursor":""}"#;
        let r: PlanResponse = serde_json::from_str(json).unwrap();
        assert_eq!(r.itineraries[0].legs[0].mode, "RAIL");
    }
}
