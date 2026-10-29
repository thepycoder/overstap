use crate::motis::{Itinerary, Leg};
use chrono::{DateTime, Utc};
use chrono_tz::Europe::Brussels;

pub fn hm(t: DateTime<Utc>) -> String {
    t.with_timezone(&Brussels).format("%H:%M").to_string()
}

fn delay_min(leg: &Leg) -> i64 {
    (leg.start_time - leg.scheduled_start_time).num_minutes()
}

pub fn summary(it: &Itinerary) -> String {
    let modes: Vec<String> = it.legs.iter().map(short_mode).collect();
    format!(
        "{} → {}  ({} min, {} transfer{})  {}",
        hm(it.start_time),
        hm(it.end_time),
        it.duration / 60,
        it.transfers,
        if it.transfers == 1 { "" } else { "s" },
        modes.join(" › ")
    )
}

fn short_mode(l: &Leg) -> String {
    match l.mode.as_str() {
        "WALK" => format!("walk {}′", l.duration / 60),
        "BIKE" => format!("bike {}′", l.duration / 60),
        "RENTAL" => format!(
            "{} {}′",
            l.rental
                .as_ref()
                .and_then(|r| r.system_name.clone())
                .unwrap_or("rental".into()),
            l.duration / 60
        ),
        _ => l
            .display_name
            .clone()
            .or_else(|| l.route_short_name.clone())
            .unwrap_or_else(|| l.mode.clone()),
    }
}

pub fn legs(it: &Itinerary) -> String {
    let mut s = String::new();
    for l in &it.legs {
        let d = delay_min(l);
        let rt = if l.real_time && d != 0 {
            format!(" (+{d}′ RT)")
        } else {
            String::new()
        };
        let track = l
            .from
            .track
            .as_ref()
            .map(|t| format!(" [spoor {t}]"))
            .unwrap_or_default();
        s.push_str(&format!(
            "   {} {:<34} {} {:<30} {}{}{}\n",
            hm(l.start_time),
            truncate(&l.from.name, 34),
            hm(l.end_time),
            truncate(&l.to.name, 30),
            short_mode(l),
            track,
            rt
        ));
    }
    s
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n - 1).collect::<String>() + "…"
    }
}
