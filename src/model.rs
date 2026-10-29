//! Domain model. Deliberately minimal; grows in Phase 2.
#![allow(dead_code)]

/// What the traveller is carrying / riding right now.
/// Determines which edges are legal next (docs/design.md §2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VehicleState {
    OnFoot,
    /// Can ride anywhere and board any vehicle when folded.
    FoldingBike,
    /// Cannot board transit; must end according to `return_rule`.
    Rental {
        operator: String,
        picked_up_at: String,
        return_rule: ReturnRule,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReturnRule {
    /// e.g. Blue-bike: back to the station it came from (makes the trip a tour).
    SameStation,
    /// Any station within the same operator zone.
    AnyStationInZone(String),
    /// Docked or virtual parking zones (Donkey Republic, Velo).
    AnyDockOrZone,
    /// Free-floating within the operator geofence (Dott).
    Geofence,
}

/// Where a query starts. `AboardTrip` covers use case 2 ("I overshot my stop").
#[derive(Debug, Clone)]
pub enum Origin {
    Place { lat: f64, lon: f64 },
    Stop { stop_id: String },
    AboardTrip { trip_id: String },
}

#[derive(Debug, Clone)]
pub enum LegMode {
    Walk,
    Bike,
    Transit { route: String, trip_id: String },
}

#[derive(Debug, Clone)]
pub struct Leg {
    pub mode: LegMode,
    pub from: String,
    pub to: String,
    /// Unix seconds.
    pub depart: i64,
    pub arrive: i64,
}

#[derive(Debug, Clone, Default)]
pub struct Journey {
    pub legs: Vec<Leg>,
}

impl Journey {
    pub fn arrival(&self) -> Option<i64> {
        self.legs.last().map(|l| l.arrive)
    }

    pub fn transfers(&self) -> usize {
        self.legs
            .iter()
            .filter(|l| matches!(l.mode, LegMode::Transit { .. }))
            .count()
            .saturating_sub(1)
    }
}
