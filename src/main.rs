//! overstap — orchestration layer on top of MOTIS.
//!
//! Pipeline (see docs/design.md §4):
//! 1. candidate generation: many MOTIS calls (transit, bike, walk)
//! 2. stitching: join legs under vehicle-state constraints
//! 3. scoring: time, transfers, cost, delay risk
//! 4. policy building: contingency branches on delay events

mod model;
mod motis;

fn main() {
    let motis = motis::Client::new("http://localhost:8080");
    println!("overstap skeleton — MOTIS backend at {}", motis.base_url());
    println!("Nothing routes yet; see TODO.md (Phase 2).");
}
