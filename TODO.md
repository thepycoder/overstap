# TODO

Legend: `[ ]` open · `[~]` in progress · `[x]` done. Keep items small; link to docs where useful.

## Phase 0 — Foundations

- [x] Write down design, data sources and decisions (`docs/`)
- [x] Scaffold Rust crate
- [ ] Push repo to GitHub (`gh repo create overstap --private --source=. --push`)
- [x] Find Transitous Belgian feed definitions → `docs/data-sources.md`
- [x] Verify GBFS per operator (Blue-bike, Velo, Donkey, Dott all public)
- [ ] Register a `bmc-partner-key` on the Belgian Mobility portal (De Lijn/TEC/STIB) in case anonymous access closes
- [ ] Find Donkey Republic Mechelen feed (bikes appear under `donkey_antwerp`; confirm coverage)
- [ ] **GTFS-RT logger** (ADR-006): poll NMBS + De Lijn RT every 30–60 s, store raw protobuf + parsed delays. Run on an always-on box

## Phase 1 — MOTIS running

- [x] Run MOTIS v2.11.4 with Belgium OSM + NMBS + De Lijn + RT + 5 GBFS feeds (cloud workspace)
- [x] `deploy/motis/config.yml` + `fetch-data.sh`
- [x] Check the OpenAPI spec (see design §4: more is built in than assumed)
- [x] Run the three use cases → `docs/results-2026-10-29.md`
- [ ] Pick a permanent host (homelab/VPS; 2 GB RAM serving is enough, ~4 GB import)
- [ ] Enable tiles for the web UI
- [ ] Nightly re-import (GTFS changes daily; De Lijn RT matching dropped to 75 % with a week-old import)
- [ ] Add TEC, STIB, more GBFS (Bolt, Lime, Voi, Donkey Kortrijk/Westhoek)
- [x] Hoppy Kempen as a generated static GBFS feed (`tools/hoppy_gbfs.py`, ADR-011)
- [ ] Hoppy: ask Hoppy / IOK (Vervoerregio Kempen) for a public GBFS feed with live availability
- [ ] Hoppy: the 17 Kempen zones not in the Hoppinpunt data (hand-curated list?)
- [ ] Hoppy: other regions (Vlaamse Ardennen, Dender, Midwest, Brugge, …) once their rules are checked
- [ ] E-bike speed: MOTIS uses ~15 km/h for every rental; re-time `ELECTRIC_ASSIST` legs (≈20 km/h) in overstap

## Phase 2 — Orchestration MVP (Rust)

- [x] Typed MOTIS client (`/plan`, `/trip`, `/geocode`, `/rentals`)
- [x] CLI: `overstap plan <from> <to> [--at] [--carry foot|folding|shared]`
- [x] Use case 2: `overstap overshot <dest> --trip <id> [--folding]`
- [x] Use case 1: hub-only correction (Donkey) + round-trip tour pricing (Blue-bike)
- [x] End-to-end tests (`tests/use_cases.rs`, skip when MOTIS is down)
- [ ] Look up the current trip by "I'm on the 08:09 to Mol" instead of a raw tripId
- [ ] Geocode venues to entrances (Nekkerhal centroid snapped wrong)
- [ ] Better geocoding fallback ("'t Veld, Retie" not found): local names / hamlets list, fuzzy street match
- [ ] Present walk-vs-wait trade-offs (long walk vs late bus) instead of a hard walk cap
- [ ] Preprocess NMBS `transfers.txt` for personal transfer slack (MOTIS can only raise transfer times)
- [ ] Bike-between-transit-legs stitching (folding bike mid-journey)
- [ ] Provider rules as a config file instead of constants in `lastmile.rs`
- [ ] Rental dock availability at drop-off (docks free) and bike availability at pickup time
- [ ] Benchmark harness vs Citymapper / NMBS planner on my real trips

## Phase 3 — Real time & reliability

- [x] Delay *simulation* (`--delay N`) for use case 3
- [ ] Delay distributions from logger history
- [ ] P(transfer holds) and expected-arrival scoring
- [ ] Contingency policies (decision trees) as output, via `numLegAlternatives`
- [ ] Watch mode: re-run on every RT update, push notification when the best option changes

## UI (see docs/ui-concepts.md)

- [x] Web UI v0: bike-transit lane timetable on overstap data (`overstap serve`, `web/`, ADR-012)
- [ ] Deploy next to MOTIS on the NAS (Docker image with motis + overstap, cloudflared like bike-transit)
- [ ] PWA manifest + icon, so it opens full-screen from the home screen
- [ ] Boarding buffer for the folding bike (fold + platform): bike legs end N min before departure
- [ ] Shared tab: search e-bike rentals at 20 km/h too (ADR-013); parallelise the street routes
- [ ] Show rental availability (bikes at pickup, docks at drop-off) on rental blocks
- [ ] "Open in app" links for rentals (MOTIS gives `rentalUriAndroid/IOS`)
- [ ] Lane detail via one corridor vs another (canvas "Two corridors" D/E): group or tint lanes by corridor
- [ ] Alternatives for a selected route (canvas round 4): per-leg variants, using MOTIS `numLegAlternatives`
- [ ] Pick 2–3 concepts; likely: When-to-leave staircase (before), Next move + buffers (during), Plan B tree (when things break)
- [ ] API endpoints for the views: leave-time sweep, per-change slack + fallbacks
- [ ] Ride mode: live position on the space-time line, precomputed fallback per change, buzz only when action/spare/arrival changes
- [ ] Log GBFS station stock (predict empty stations)
- [ ] Post-trip review: user-reported delays for vehicles without live data

## Later / ideas

- [ ] Custom McRAPTOR if stitching gets too slow (ADR-004 escalation)
- [ ] Upstream to MOTIS: GBFS hub-only providers, lowering transfer times
