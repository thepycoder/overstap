//! overstap — orchestration layer on top of MOTIS.
//!
//! Pipeline (see docs/design.md §4):
//! 1. candidate generation: many MOTIS calls (transit, bike, walk)
//! 2. stitching: join legs under vehicle-state constraints
//! 3. scoring: time, transfers, cost, delay risk
//! 4. policy building: contingency branches on delay events

mod lastmile;
mod model;
mod motis;
mod overshoot;
mod planner;
mod render;
mod server;

use anyhow::{bail, Result};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Europe::Brussels;
use clap::{Parser, Subcommand};
use planner::Carry;

#[derive(Parser)]
#[command(
    name = "overstap",
    about = "Personal multimodal planner for Belgium (MOTIS backend)"
)]
struct Cli {
    /// MOTIS base URL
    #[arg(
        long,
        env = "MOTIS_URL",
        default_value = "http://localhost:8080",
        global = true
    )]
    motis: String,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Check that MOTIS is up.
    Health,
    /// Resolve a name to coordinates / stop ids.
    Geocode { text: String },
    /// Plan a journey. Places: "lat,lon", a MOTIS stop id, or free text (geocoded).
    Plan {
        from: String,
        to: String,
        /// Local time (Europe/Brussels), e.g. "2026-10-30 08:00". Default: now.
        #[arg(long)]
        at: Option<String>,
        #[arg(long, value_enum, default_value_t = Carry::Foot)]
        carry: Carry,
        /// Minimum transfer time in minutes (MOTIS default 0 = use the feed's own).
        #[arg(long)]
        min_transfer: Option<u32>,
        /// Search window in minutes.
        #[arg(long, default_value_t = 60)]
        window: u32,
        #[arg(long, default_value_t = 5)]
        show: usize,
        /// Max minutes on the bike / rental before and after transit.
        #[arg(long, default_value_t = 30)]
        max_bike: u32,
        /// Your cycling speed on your own (folding) bike, km/h.
        #[arg(long, default_value_t = 18.0)]
        kmh: f64,
    },
    /// Serve the web UI and its JSON API.
    Serve {
        #[arg(long, default_value = "127.0.0.1:8737", env = "OVERSTAP_LISTEN")]
        listen: String,
        /// Directory with the web UI's static files.
        #[arg(long, default_value = "web", env = "OVERSTAP_WEB")]
        web: std::path::PathBuf,
    },
    /// You're on a train (use cases 2 and 3): evaluate getting off at each of
    /// the next N stops and continuing from there. `--delay` simulates a delay.
    #[command(alias = "aboard")]
    Overshot {
        /// Destination ("lat,lon", stop id or text).
        to: String,
        /// The trip you're on: a MOTIS tripId (from `plan --json`), or
        /// "FROM->TO@TIME" to look it up by searching (see README).
        #[arg(long)]
        trip: String,
        /// How many downstream stops to consider.
        #[arg(long, default_value_t = 4)]
        next: usize,
        #[arg(long)]
        now: Option<String>,
        /// You carry a folding bike: cycling back is an option.
        #[arg(long)]
        folding: bool,
        /// Simulate this many minutes of extra delay on the current train (use case 3).
        #[arg(long, default_value_t = 0)]
        delay: i64,
    },
}

pub fn parse_local(s: &str) -> Result<DateTime<Utc>> {
    let naive = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M")
        .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M"))?;
    match Brussels.from_local_datetime(&naive).single() {
        Some(t) => Ok(t.with_timezone(&Utc)),
        None => bail!("ambiguous or invalid local time: {s}"),
    }
}

/// Accept "lat,lon", a stop id containing '_' and no spaces, or geocode free text.
pub fn resolve_place(c: &motis::Client, s: &str) -> Result<String> {
    let looks_coord =
        s.split(',').count() == 2 && s.split(',').all(|p| p.trim().parse::<f64>().is_ok());
    if looks_coord || (s.contains('_') && !s.contains(' ')) {
        return Ok(s.to_string());
    }
    let hits = c.geocode(s)?;
    let Some(best) = hits.first() else {
        bail!("no geocode match for '{s}'")
    };
    eprintln!(
        "  '{s}' → {} ({}, {:.5},{:.5})",
        best.name, best.kind, best.lat, best.lon
    );
    Ok(match (&best.kind[..], &best.id) {
        ("STOP", Some(id)) => id.clone(),
        _ => format!("{},{}", best.lat, best.lon),
    })
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let c = motis::Client::new(&cli.motis);
    match cli.cmd {
        Cmd::Health => {
            println!(
                "{}: {}",
                c.base_url(),
                if c.health()? { "ok" } else { "unhealthy" }
            );
        }
        Cmd::Geocode { text } => {
            for m in c.geocode(&text)?.iter().take(8) {
                println!(
                    "{:<6} {:<45} {:.5},{:.5}  {}",
                    m.kind,
                    m.name,
                    m.lat,
                    m.lon,
                    m.id.clone().unwrap_or_default()
                );
            }
        }
        Cmd::Plan {
            from,
            to,
            at,
            carry,
            min_transfer,
            window,
            show,
            max_bike,
            kmh,
        } => {
            let mut r =
                planner::Request::new(resolve_place(&c, &from)?, resolve_place(&c, &to)?, carry);
            r.time = at.as_deref().map(parse_local).transpose()?;
            r.window_min = window;
            r.min_transfer_min = min_transfer;
            r.max_bike_min = max_bike;
            r.ride_kmh = kmh;
            let mut js = planner::plan(&c, &r)?;
            // Best first: earliest arrival, then latest departure.
            js.sort_by_key(|j| (j.it.end_time, -j.it.start_time.timestamp()));
            let (direct, its): (Vec<_>, Vec<_>) = js
                .into_iter()
                .partition(|j| j.it.transfers == 0 && !j.it.legs.iter().any(|l| l.is_transit()));
            for (n, j) in its.iter().take(show).enumerate() {
                let label = j
                    .label
                    .as_deref()
                    .map(|l| format!("  [{l}]"))
                    .unwrap_or_default();
                println!("\n#{} {}{label}", n + 1, render::summary(&j.it));
                print!("{}", render::legs(&j.it));
                for note in &j.notes {
                    println!("   note: {note}");
                }
            }
            for j in direct.iter().take(2) {
                println!("\n[direct] {}", render::summary(&j.it));
            }
        }
        Cmd::Serve { listen, web } => server::run(c, &listen, &web)?,
        Cmd::Overshot {
            to,
            trip,
            next,
            now,
            folding,
            delay,
        } => {
            let now = now
                .as_deref()
                .map(parse_local)
                .transpose()?
                .unwrap_or_else(Utc::now);
            let dest = resolve_place(&c, &to)?;
            overshoot::run(&c, &trip, &dest, next, now, folding, delay)?;
        }
    }
    Ok(())
}
