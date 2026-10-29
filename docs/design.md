# Design

Status: draft, 2026-10-29. Captures the initial brainstorm.

## 1. Problem

Existing planners each miss part of the picture:

- **Google Maps** only considers walking around public transport.
- **Citymapper**'s mixed profile assumes a bike is available after a train ride,
  but ignores buses in that scenario, and doesn't know whether you carry a folding
  bike or depend on a bike-share station existing at the far end.
- **NMBS's own planner** hides connections with less than ~5 min transfer time,
  which is often too conservative (and sometimes too optimistic).

None of them can take "its sweet time" to optimise a long journey for one person,
or react intelligently when you're stranded or delayed.

## 2. Modes and their constraints

| Mode | Where it can be used | Can board transit? | End constraint |
|---|---|---|---|
| Walk | anywhere | yes | none |
| Folding bike | anywhere | yes (free on NMBS/De Lijn when folded) | none |
| Station bike-share (Blue-bike) | from a station | no | usually return to the *same* station (some zones allow another) |
| Docked/zone bike-share (Donkey, Velo) | from dock/zone | no | must end at a dock / inside a parking zone |
| Free-floating (Dott, …) | from vehicle location | no | inside operator geofence, outside no-parking zones |
| Train / bus / tram | stops | — | — |

The key modelling idea: a journey carries a **vehicle state**
(`OnFoot | FoldingBike | Rental{operator, picked_up_at, return_rule}`), and that
state determines which edges are legal next.

## 3. Use cases (acceptance tests)

1. **Last-mile choice with return cost.** Lokeren → Mechelen Nekkershal.
   Options: Mechelen-Nekkerspoel + walk, Mechelen (main) + bike, bus.
   Donkey Republic looked good but has no parking near Nekkershal; Blue-bike was
   better *even counting the ride back to the station*. The planner must price
   the return leg of a round-trip rental (it's a **tour**, not a trip) and check
   parking/dock availability at the destination.
2. **Overshot my stop.** On the train towards Mol, missed the stop. Options:
   get off at the next stop and wait for a train back; take a bus; ride on to a
   larger station with more frequent return trains. The origin is a *trip*, not
   a stop — every downstream stop is a candidate alighting point.
3. **Delay breaks a late-night connection.** Train is delayed; the later
   transfer will be missed and evening frequency is low. Ride it out and take the
   next (much later) connection, or bail out now for a bus or a different line?
   Needs real-time data and, ideally, delay *distributions* (how likely is the
   transfer to hold?).

## 4. Architecture

```
            ┌───────────────────────── overstap (Rust) ─────────────────────────┐
 user  ───► │ query: origin (place | stop | "aboard trip X"), dest, time,       │
 (CLI,      │        vehicle state, risk tolerance                              │
  later     │                                                                   │
  app)      │ 1. candidate generation  → many MOTIS calls (transit, bike, walk) │
            │ 2. stitching             → legs joined under state constraints    │
            │ 3. scoring               → time, transfers, cost, delay risk      │
            │ 4. policy building       → contingency branches on delay events   │
            └───────────────┬───────────────────────────────┬───────────────────┘
                            │ REST (OpenAPI)                │ GBFS (direct, optional)
                    ┌───────▼────────┐               ┌──────▼──────────┐
                    │ MOTIS          │               │ GTFS-RT logger  │──► delay history DB
                    │  nigiri  (PT)  │               │ (own process)   │
                    │  osr  (streets)│               └─────────────────┘
                    │  GBFS, GTFS-RT │
                    └────────────────┘
                     inputs: Belgian GTFS (NMBS, De Lijn, TEC, STIB),
                     GTFS-RT, GBFS feeds, Belgium OSM extract
```

### What MOTIS gives us (unmodified)

- **nigiri**: the transit router (RAPTOR family) over all GTFS feeds, with
  GTFS-RT applied (delays, cancellations, track changes).
- **osr**: street routing on OSM for walk / bike / car, used for footpaths
  between stops, first/last mile, and direct trips.
- **GBFS**: bike-share stations, vehicles, geofencing zones.
- Geocoding, reverse geocoding, map tiles, a web UI for debugging.

So "no nigiri" is not a concern: nigiri *is* MOTIS's transit core. We just don't
modify it.

### What MOTIS v2.11 already does (verified against `openapi.yaml`, 2026-10-29)

Better than first assumed:

- `preTransitModes` / `postTransitModes` = `BIKE` → folding bike first/last mile (up to
  `street_routing_max_prepost_transit_seconds`, 1 h).
- `requireBikeTransport=true` → only trips that allow bikes (NMBS GTFS has no
  `bikes_allowed`; we set `default_bikes_allowed: true` for the SNCB dataset).
- `RENTAL` modes + GBFS, **including return constraints**
  (`returnConstraint: NONE | ANY_STATION | ROUNDTRIP_STATION`). Blue-bike is
  `ROUNDTRIP_STATION`, so MOTIS won't propose dropping it at Nekkerhal.
- `minTransferTime`, `additionalTransferTime`, `transferTimeFactor` (≥ 1).
- `numLegAlternatives`: alternatives per transit leg (useful for contingency plans).
- `/api/v6/trip`, `/api/v6/stoptimes`, `/api/v1/one-to-many`, `/api/v1/rentals`.

### What the orchestration layer adds

| Gap in MOTIS | How overstap covers it |
|---|---|
| Bike as a *transfer* between two transit legs (pre/post-transit only) | Query transit candidates, bike between stations via osr (or BRouter), query again for the onward leg |
| Round-trip rental priced as a tour (the ride back to the station) | Add the return leg cost when comparing a `ROUNDTRIP_STATION` rental with other options |
| Origin "aboard trip X" | **Implemented** (`overstap overshot`): query from each downstream stop at its real-time arrival; rank |
| Transfers *shorter* than the feed's (`minTransferTime` can only raise them) | Preprocess NMBS `transfers.txt` before import (NMBS GTFS uses 240 s at most stations, 300 s at some) |
| Delay risk | Score candidate journeys with delay distributions from our own GTFS-RT history |
| Contingency plans | Build a decision tree over key delay events, re-evaluated on each RT update |

Escalation path: if brute-force stitching becomes too slow or combinatorial,
write a custom McRAPTOR in Rust over the same GTFS data (MOTIS still provides
street legs). Only as a last resort, patch MOTIS itself — preferably upstream.

## 5. Algorithm background (primer)

- **RAPTOR** (Round-bAsed Public Transit Optimized Router; Delling, Pajor,
  Werneck, 2012). Round *k* = best arrival at every stop using ≤ *k* vehicles.
  Each round scans only *routes* (trips with identical stop sequences) touching
  stops improved in the previous round, then relaxes footpaths. No priority
  queue, cache-friendly, yields the Pareto set (arrival time × transfers).
  Transfers = footpath edges + minimum change time.
- **rRAPTOR**: range variant — all optimal journeys over a departure window.
- **McRAPTOR**: multi-criteria; a "bag" of non-dominated labels per stop.
  This is where extra criteria like vehicle state, cost or risk would go.
- **ULTRA** (Baum et al., KIT, 2019): precomputed shortcuts enabling
  *unrestricted* walking/cycling transfers between any stops — the formal answer
  to the folding-bike problem. Our poor-man's version: precompute bike times
  between stops within ~8 km.
- **CSA** (Connection Scan Algorithm; Dibbelt et al.): one pass over a
  time-sorted array of elementary connections. Simple; basis of
  **delay-robust journeys / minimum expected arrival time** (Dibbelt, Strasser,
  Wagner, 2014) — the reference for use case 3.
- **OTP2** (OpenTripPlanner, Java, Entur-led): Range RAPTOR + A* street
  routing. Mature, but JVM-heavy. Considered and not chosen (see decisions).
- **MOTIS** (C++, TU Darmstadt / Deutsche Bahn research): nigiri + osr, lean
  and fast; powers Transitous.

## 6. Reliability model (later)

Log every GTFS-RT update (NMBS, De Lijn) from day one. From the history, fit per
(trip pattern, stop, time-of-day, weekday) delay distributions. Then:

- P(transfer holds) = P(arrival delay of feeder − departure delay of connector
  < slack − walk time).
- Score journeys by expected arrival time *including* the fallback when a
  transfer fails (this is where last-train risk dominates at night).
- Similar in spirit to what ChuuChuu does for reliability of connections.
