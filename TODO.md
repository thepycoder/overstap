# TODO

Legend: `[ ]` open · `[~]` in progress · `[x]` done. Keep items small; link to docs where useful.

## Phase 0 — Foundations

- [x] Write down design, data sources and decisions (`docs/`)
- [x] Scaffold Rust crate
- [ ] Push repo to GitHub (`gh repo create overstap --private --source=. --push`)
- [x] Find Transitous Belgian feed definitions → `docs/data-sources.md`
- [x] Verify GBFS per operator (Blue-bike, Velo, Donkey, Dott all public)
- [ ] Register a `bmc-partner-key` on the Belgian Mobility portal (De Lijn/TEC/STIB) in case anonymous access closes
- [x] Find Donkey Republic Mechelen feed: covered by `donkey_antwerp` (86 Mechelen-area + 126 Waasland hubs incl. Lokeren Markt, 2026-11-05)
- [x] **GTFS-RT logger** (ADR-006, ADR-015): `overstap rt-log`, changes only in SQLite + raw snapshots
- [~] Run the logger on the NAS (deploy/nas): running since 2026-11-05 13:55; check real volume after a day; add De Lijn/NMBS alerts feeds

## Phase 1 — MOTIS running

- [x] Run MOTIS v2.11.4 with Belgium OSM + NMBS + De Lijn + RT + 5 GBFS feeds (cloud workspace)
- [x] `deploy/motis/config.yml` + `fetch-data.sh`
- [x] Check the OpenAPI spec (see design §4: more is built in than assumed)
- [x] Run the three use cases → `docs/results-2026-10-29.md`
- [x] Deployment bundle for the NAS: one image, compose with motis/overstap/rtlog, GHCR workflow (deploy/nas, ADR-014)
- [ ] Deploy on the NAS + Cloudflare hostname (needs the user: tunnel config, GHCR package visibility)
- [ ] Enable tiles for the web UI
- [x] Nightly re-import with health-checked swap (`deploy/motis/update.sh`, `run.sh`): RT matching 74 → 99 % (De Lijn), 92 → 98 % (NMBS)
- [ ] Add TEC, STIB, more GBFS (Bolt, Lime, Voi, Donkey Kortrijk/Westhoek)
- [x] Hoppy Kempen as a generated static GBFS feed (`tools/hoppy_gbfs.py`, ADR-011)
- [ ] Hoppy: ask Hoppsharing BV / IOK / SOLVA for a public GBFS feed with live availability (Hoppsharing is registered on transportdata.be with 0 datasets; no public web map exists, see data-sources "Unofficial live feeds")
- [ ] Hoppy: merge extra zone sources into `tools/hoppy_gbfs.py`: OSM `bicycle_rental` Hoppy points (4 Kempen zones missing from Hoppinpunt) and the SOLVA locations CSV (11 missing in Dender/Vlaamse Ardennen)
- [ ] Hoppy: other regions (Vlaamse Ardennen, Dender, Midwest, Brugge, …) once their rules are checked
- [ ] E-bike speed: MOTIS uses ~15 km/h for every rental; re-time `ELECTRIC_ASSIST` legs (≈20 km/h) in overstap

## Phase 2 — Orchestration MVP (Rust)

- [x] Typed MOTIS client (`/plan`, `/trip`, `/geocode`, `/rentals`)
- [x] CLI: `overstap plan <from> <to> [--at] [--carry foot|folding|shared]`
- [x] Use case 2: `overstap overshot <dest> --trip <id> [--folding]`
- [x] Use case 1: hub-only correction (Donkey) + round-trip tour pricing (Blue-bike)
- [x] End-to-end tests (`tests/use_cases.rs`, skip when MOTIS is down)
- [ ] Look up the current trip by "I'm on the 08:09 to Mol" instead of a raw tripId
- [~] Geocode venues to entrances (Nekkerhal centroid snapped wrong): web UI uses Citymapper/Google place search (ADR-017) → Plattebeekstraat 1; CLI free text still MOTIS; venues now route to their street address (ADR-021)
- [ ] Real entrances for large sites: OSM `entrance=*` / gates on the site boundary, picked by direction of arrival
- [ ] Blue-bike legs with `returnConstraint: NONE` from MOTIS (Geel Muziekacademie): check the De Lijn GBFS feed; price them as tours if they're really round-trip
- [~] Better geocoding ("'t Veld, Retie" not found): Citymapper finds "'t Veldt" (ADR-017); CLI free text still MOTIS
- [ ] Measure MOTIS RAM on the NAS with `geocoding`/`reverse_geocoding` off; drop the MOTIS geocoder only if it's worth losing the fallback (ADR-017)
- [ ] Present walk-vs-wait trade-offs (long walk vs late bus) instead of a hard walk cap
- [ ] Preprocess NMBS `transfers.txt` for personal transfer slack (MOTIS can only raise transfer times)
- [ ] Bike-between-transit-legs stitching (folding bike mid-journey)
- [x] Own (non-folding) bike with a place: with you (first leg) or parked (last leg from there), next to shared bikes (ADR-018)
- [ ] Own bike mid-trip: pick it up at a station and ride to another station (bike between transit legs)
- [ ] Bike-parking time at the station for the own-bike first leg (lock + walk to the platform)
- [ ] Provider rules as a config file instead of constants in `lastmile.rs`
- [ ] Blue-bike legs with `returnConstraint: NONE` from MOTIS (e.g. Geel Muziekacademie, results-2026-11-10): check the De Lijn GBFS feed; if they really are round-trip, price them as tours like the rest
- [ ] Shared tab: build Blue-bike tours from the rental plans too (Donkey to the station + Blue-bike tour at Mol is never offered)
- [ ] Rental dock availability at drop-off (docks free) and bike availability at pickup time
- [ ] Benchmark harness vs Citymapper / NMBS planner on my real trips

## Phase 3 — Real time & reliability

- [x] Delay *simulation* (`--delay N`) for use case 3
- [x] NMBS platforms from iRail in `/api/live`; ride mode shows the next track big (ADR-022)
- [ ] Ride mode: flag platform changes iRail reports (`platforminfo.normal = 0`)
- [ ] De Lijn platforms: check whether its GTFS has `platform_code` (MOTIS `track`) at stations like Gent-Sint-Pieters
- [ ] Delay distributions from logger history
- [ ] P(transfer holds) and expected-arrival scoring
- [ ] Contingency policies (decision trees) as output, via `numLegAlternatives`
- [~] Watch mode: ride mode v0 re-checks every 30 s and alerts from the page (ADR-016)
- [ ] Server-side ride watcher + Web Push (VAPID), so alerts arrive with the app closed
- [ ] Auto-switch to plan B when a connection is certainly lost (now: one tap)

## UI (see docs/ui-concepts.md)

- [x] Web UI v0: bike-transit lane timetable on overstap data (`overstap serve`, `web/`, ADR-012)
- [x] PWA manifest + icon + service worker
- [ ] Boarding buffer for the folding bike (fold + platform): bike legs end N min before departure
- [ ] Shared tab: search e-bike rentals at 20 km/h too (ADR-013); parallelise the street routes
- [ ] Show rental availability (bikes at pickup, docks at drop-off) on rental blocks
- [ ] "Open in app" links for rentals (MOTIS gives `rentalUriAndroid/IOS`)
- [ ] Lane detail via one corridor vs another (canvas "Two corridors" D/E): group or tint lanes by corridor
- [x] Longest bike ride as a setting ("Bike ≤ N min", per tab), also capping Blue-bike tours (ADR-020)
- [x] Variants view of a route (canvas round 4, braided lane) from the search's own results (ADR-019)
- [ ] `/api/variants`: per-leg alternatives for the braided lane (ride/bus to each stop of the chosen train, adjacent departures per leg, dominance pruning; maybe MOTIS `numLegAlternatives`)
- [ ] Pick 2–3 concepts; likely: When-to-leave staircase (before), Next move + buffers (during), Plan B tree (when things break)
- [ ] API endpoints for the views: leave-time sweep, per-change slack + fallbacks
- [x] Ride mode v0: next move + countdown, strip to arrival, spare bars per change, plan B per change, bikes at pickup, alerts on changed facts (ADR-016)
- [ ] Ride mode: GPS position instead of the clock (detect getting off early / overshooting)
- [ ] Ride mode: plan B for the last mile (no bikes → other operator/walk), 'Something's wrong' flows from the canvas
- [ ] Log GBFS station stock (predict empty stations)
- [ ] Post-trip review: user-reported delays for vehicles without live data

## Later / ideas

- [ ] Custom McRAPTOR if stitching gets too slow (ADR-004 escalation)
- [ ] Upstream to MOTIS: GBFS hub-only providers, lowering transfer times
