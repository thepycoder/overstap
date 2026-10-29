//! End-to-end tests against a running MOTIS loaded with the Belgian feeds
//! (deploy/motis). Skipped automatically when MOTIS isn't reachable.
//! Run: MOTIS_URL=http://localhost:8080 cargo test --test use_cases -- --nocapture
//!
//! Trip ids and times are tied to the timetable that was imported, so these
//! use "tomorrow" relative to the import and search for trips dynamically.

use std::process::Command;

fn motis_url() -> String {
    std::env::var("MOTIS_URL").unwrap_or_else(|_| "http://localhost:8080".into())
}

fn motis_up() -> bool {
    Command::new("curl")
        .args([
            "-sf",
            "-m",
            "3",
            "-o",
            "/dev/null",
            &format!("{}/api/v1/health", motis_url()),
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn overstap(args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_overstap"))
        .args(args)
        .env("MOTIS_URL", motis_url())
        .output()
        .expect("run overstap");
    assert!(
        out.status.success(),
        "overstap {:?} failed: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn tomorrow_at(hm: &str) -> String {
    let d = chrono::Utc::now()
        .with_timezone(&chrono_tz::Europe::Brussels)
        .date_naive()
        + chrono::Duration::days(1);
    format!("{} {hm}", d.format("%Y-%m-%d"))
}

macro_rules! require_motis {
    () => {
        if !motis_up() {
            eprintln!("MOTIS not reachable at {}, skipping", motis_url());
            return;
        }
    };
}

const LOKEREN: &str = "51.1069,3.9859";
const NEKKERHAL: &str = "51.02566,4.50349"; // Nekkerspoel-Borcht 19

#[test]
fn uc1_folding_bike_beats_walking() {
    require_motis!();
    let at = tomorrow_at("08:00");
    let foot = overstap(&["plan", LOKEREN, NEKKERHAL, "--at", &at, "--show", "1"]);
    let fold = overstap(&[
        "plan", LOKEREN, NEKKERHAL, "--at", &at, "--carry", "folding", "--show", "1",
    ]);
    let mins = |s: &str| -> i64 {
        let i = s.find(" min,").expect("duration");
        s[..i].rsplit('(').next().unwrap().trim().parse().unwrap()
    };
    println!("{foot}\n{fold}");
    assert!(fold.contains("bike"), "folding plan should use the bike");
    assert!(
        mins(&fold) < mins(&foot),
        "folding bike should be faster than walking"
    );
}

#[test]
fn uc1_shared_bikes_respect_provider_rules() {
    require_motis!();
    let out = overstap(&[
        "plan",
        LOKEREN,
        NEKKERHAL,
        "--at",
        &tomorrow_at("08:00"),
        "--carry",
        "shared",
        "--show",
        "6",
    ]);
    println!("{out}");
    // Donkey must be parked at a hub, never at the door.
    if out.contains("Donkey") {
        assert!(
            out.contains("via hub"),
            "Donkey options must be corrected to end at a hub"
        );
    }
    // Blue-bike (round-trip only) must be offered as a tour.
    assert!(
        out.contains("blue-bike") && out.contains("round trip"),
        "Blue-bike tour option missing"
    );
}

#[test]
fn uc2_overshot_offers_earlier_return_than_terminus() {
    require_motis!();
    // Find tomorrow's 08:09-ish IC Antwerpen-Centraal → Mol via the plan API.
    let at = tomorrow_at("08:00");
    let json = Command::new("curl").args(["-s", &format!(
        "{}/api/v6/plan?fromPlace=sncb_S8821006&toPlace=sncb_S8832409&maxTransfers=0&detailedLegs=false&time={}",
        motis_url(), chrono::NaiveDateTime::parse_from_str(&at, "%Y-%m-%d %H:%M").unwrap()
            .and_local_timezone(chrono_tz::Europe::Brussels).unwrap()
            .with_timezone(&chrono::Utc).format("%Y-%m-%dT%H:%M:%SZ"))]).output().unwrap().stdout;
    let v: serde_json::Value = serde_json::from_slice(&json).unwrap();
    let trip = v["itineraries"][0]["legs"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|l| l["tripId"].as_str())
        .expect("a trip to Mol")
        .to_string();
    // Pretend we wanted Herentals but are now past it.
    let out = overstap(&[
        "overshot",
        "sncb_S8821717",
        "--trip",
        &trip,
        "--now",
        &tomorrow_at("08:42"),
        "--next",
        "3",
    ]);
    println!("{out}");
    assert!(
        out.contains("1. Get off at"),
        "should rank alighting options"
    );
    assert!(
        !out.contains("1. Get off at Mol"),
        "terminus should not be the best option"
    );
}
