# TODO

Legend: `[ ]` open · `[~]` in progress · `[x]` done. Keep items small; link to docs where useful.

## Phase 0 — Foundations (start now)

- [x] Write down design, data sources and decisions (`docs/`)
- [x] Scaffold Rust crate
- [ ] Push repo to GitHub (create empty repo, then connect it to Claude sessions)
- [ ] Request API keys: De Lijn (GTFS static, GTFS-RT, Open Data) via data.delijn.be
- [ ] Request NMBS/SNCB open-data licence (GTFS + GTFS-RT)
- [ ] Find the Transitous Belgian feed definitions (likely `feeds/be.json`), list feeds + licences in `docs/data-sources.md`
- [ ] Verify GBFS availability per operator: Blue-bike, Velo Antwerpen, Donkey Republic, Dott (MobilityData catalog)
- [ ] **GTFS-RT logger** (ADR-006): poll NMBS + De Lijn RT, store raw protobuf + parsed delays (SQLite/Parquet). Run on an always-on box

## Phase 1 — MOTIS running locally

- [ ] Pick host (homelab / VPS). Target: 16 GB RAM for import, less for serving
- [ ] `deploy/motis`: download release, Belgium OSM (Geofabrik), GTFS feeds → `motis config` / `import` / `server`
- [ ] Add GTFS-RT and GBFS feeds to `config.yml`
- [ ] Check the OpenAPI spec for: transfer-time params, bike carriage, rental modes, one-to-many/matrix endpoint, trip-based origin
- [ ] Manually run the three use cases in the MOTIS web UI; note what's missing → update `docs/design.md` gap table

## Phase 2 — Orchestration MVP (Rust)

- [ ] Typed MOTIS client (generate from `openapi.yaml` or hand-write the few endpoints we use)
- [ ] Domain model: `VehicleState`, `Leg`, `Journey`, scoring criteria
- [ ] CLI: `overstap plan <from> <to> [--at] [--state folding-bike]`
- [ ] Use case 2: origin "aboard trip X" → query from each downstream stop, Pareto-merge
- [ ] Use case 1: folding bike / rental last-mile comparison incl. return-leg (tour) cost and dock availability
- [ ] Bike-between-transit-legs stitching
- [ ] Custom transfer slack per station (config file)
- [ ] Benchmark harness vs. Citymapper / NMBS planner on a fixed set of my real trips

## Phase 3 — Real time & reliability

- [ ] Delay distributions from logger history
- [ ] P(transfer holds) and expected-arrival scoring
- [ ] Use case 3: delay → replan, compare "ride it out" vs "bail out"
- [ ] Contingency policies (decision trees) as output
- [ ] Push notifications when the best policy changes

## Later / ideas

- [ ] Mobile front-end (PWA?)
- [ ] TEC + STIB feeds
- [ ] Custom McRAPTOR if stitching gets too slow (ADR-004 escalation)
- [ ] Upstream proposals to MOTIS (cycling transfers, transfer slack)
- [ ] Per-station hand-curated transfer times
