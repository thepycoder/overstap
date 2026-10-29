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
mod render;

use anyhow::{bail, Result};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Europe::Brussels;
use clap::{Parser, Subcommand, ValueEnum};

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

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Carry {
    /// Walk only around transit.
    Foot,
    /// Folding bike: ride before/after transit, bike must be allowed on board.
    Folding,
    /// Use shared bikes (GBFS) for first/last mile.
    Shared,
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

fn parse_coord(s: &str) -> Option<(f64, f64)> {
    let mut p = s.split(',').map(|x| x.trim().parse::<f64>());
    match (p.next(), p.next()) {
        (Some(Ok(a)), Some(Ok(b))) => Some((a, b)),
        _ => None,
    }
}

/// Use case 1: compare walking, MOTIS's rental suggestions (corrected for
/// hub-only providers) and round-trip station bikes priced as a tour.
fn shared_last_mile(
    c: &motis::Client,
    q: &motis::PlanQuery,
    rental_its: Vec<motis::Itinerary>,
    dest: Option<(f64, f64)>,
    show: usize,
) -> Result<()> {
    let Some(dest) = dest else {
        bail!("--carry shared needs the destination as lat,lon")
    };
    let mut walk_q = q.clone();
    walk_q.pre_transit_modes = vec!["WALK".into()];
    walk_q.post_transit_modes = vec!["WALK".into()];
    let walk_its = c.plan(&walk_q)?.itineraries;

    let mut cands: Vec<lastmile::Candidate> = vec![];
    for it in walk_its.iter() {
        cands.push(lastmile::Candidate {
            label: "walk".into(),
            depart: it.start_time,
            arrive: it.end_time,
            notes: vec![],
            base: it.clone(),
        });
        if let Some(t) = lastmile::tour_option(c, it, dest)? {
            cands.push(t);
        }
    }
    for it in rental_its.iter() {
        match lastmile::fix_hub_only(c, it, dest)? {
            Some(fixed) => cands.push(fixed),
            None if it.legs.iter().any(|l| l.mode == "RENTAL") => cands.push(lastmile::Candidate {
                label: "rental (as MOTIS planned)".into(),
                depart: it.start_time,
                arrive: it.end_time,
                notes: vec![],
                base: it.clone(),
            }),
            None => {}
        }
    }
    cands.sort_by_key(|c| (c.arrive, -c.depart.timestamp()));
    cands.dedup_by_key(|c| (c.label.clone(), c.arrive, c.depart));
    for (n, cand) in cands.iter().take(show).enumerate() {
        println!(
            "\n#{} {} → {}  ({} min)  last mile: {}",
            n + 1,
            render::hm(cand.depart),
            render::hm(cand.arrive),
            (cand.arrive - cand.depart).num_minutes(),
            cand.label
        );
        let transit: Vec<String> = cand
            .base
            .legs
            .iter()
            .filter(|l| !matches!(l.mode.as_str(), "WALK" | "BIKE" | "RENTAL"))
            .map(|l| {
                format!(
                    "{} {}→{} {}",
                    l.display_name.clone().unwrap_or(l.mode.clone()),
                    render::hm(l.start_time),
                    render::hm(l.end_time),
                    l.to.name
                )
            })
            .collect();
        println!("   transit: {}", transit.join(" | "));
        for note in &cand.notes {
            println!("   note: {note}");
        }
    }
    Ok(())
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
        } => {
            let mut q = motis::PlanQuery::new(resolve_place(&c, &from)?, resolve_place(&c, &to)?);
            q.time = at.as_deref().map(parse_local).transpose()?;
            q.search_window_s = window * 60;
            q.min_transfer_time_min = min_transfer;
            q.direct_modes = vec!["WALK".into(), "BIKE".into()];
            match carry {
                Carry::Foot => {
                    // MOTIS defaults to 15 min; a 2 km walk from the station is normal.
                    q.max_pre_transit_s = Some(3600);
                    q.max_post_transit_s = Some(3600);
                }
                Carry::Folding => {
                    q.pre_transit_modes = vec!["BIKE".into()];
                    q.post_transit_modes = vec!["BIKE".into()];
                    q.max_pre_transit_s = Some(max_bike * 60);
                    q.max_post_transit_s = Some(max_bike * 60);
                    q.max_direct_s = Some(max_bike.max(60) * 60 * 3);
                    q.require_bike_transport = true;
                }
                Carry::Shared => {
                    q.pre_transit_modes = vec!["WALK".into(), "RENTAL".into()];
                    q.post_transit_modes = vec!["WALK".into(), "RENTAL".into()];
                    q.max_pre_transit_s = Some(max_bike * 60);
                    q.max_post_transit_s = Some(max_bike * 60);
                }
            }
            let r = c.plan(&q)?;
            let mut its = r.itineraries;
            its.sort_by_key(|i| (i.end_time, -i.start_time.timestamp()));
            if matches!(carry, Carry::Shared) {
                let dest = parse_coord(&q.to);
                return shared_last_mile(&c, &q, its, dest, show);
            }
            for (n, it) in its.iter().take(show).enumerate() {
                println!("\n#{} {}", n + 1, render::summary(it));
                print!("{}", render::legs(it));
            }
            for it in r.direct.iter().take(2) {
                println!("\n[direct] {}", render::summary(it));
            }
        }
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
