# overstap — context for Claude

Personal multimodal journey planner for Belgium, for one rider (Victor, lives in Lokeren).
Mixes NMBS trains, De Lijn buses/trams, shared bikes (Blue-bike, Donkey, Velo, Dott, Hoppy),
a folding bike and walking; re-plans live when delayed or stranded.

## Architecture
- **MOTIS 2 (v2.11.4) runs unmodified** as backend: GTFS + GTFS-RT (NMBS, De Lijn), GBFS, Belgium OSM.
  Config and data fetch: `deploy/motis/` (`fetch-data.sh`, `config.yml`).
- **Rust orchestration layer** (`src/`) calls MOTIS many times and stitches what it can't express:
  overshot/aboard-trip origin (`overshoot.rs`), hub-only and round-trip bike-share fixes (`lastmile.rs`),
  delay simulation. `planner.rs` = one request → journeys (shared by CLI and web).
  CLI: `overstap plan|overshot|geocode|health|serve`, `MOTIS_URL` env var.
- **Web UI** (`web/`, vanilla JS + Leaflet, no build step) served by `overstap serve` (`server.rs`,
  tiny_http, port 8737). Ported from the user's earlier Citymapper-based app (laptop
  `~/Projects/bike-transit`). The API passes MOTIS itineraries through; `web/app.js` `normLeg()`
  is the only adapter (ADR-012). Ride mode (`renderRide`, `/api/ride` for plan B) in the same files,
  and the route detail's Variants view (braided lane, `braidOf`/`braidHTML`, ADR-019).
- **RT logger**: `overstap rt-log` → SQLite (`rtlog.rs`, hand-written prost types in `gtfs_rt.rs`), `rt-report`.
- **Deploy**: `deploy/motis/update.sh` (nightly refresh + swap + rollback), `run.sh` (supervisor);
  `Dockerfile` + `deploy/nas/compose.yaml` (motis, overstap, rtlog), image built by `.github/workflows/image.yml`.
- Read first: `docs/design.md`, `docs/decisions.md` (ADRs — add one for every real decision),
  `docs/data-sources.md`, `TODO.md`, `docs/ui-concepts.md`.

## Working rules
- Run `cargo test` with MOTIS up (`MOTIS_URL=http://localhost:8080`); e2e tests skip when it's down.
- Record findings from real queries in `docs/results-*.md`; keep `TODO.md` current.
- Citymapper's web API: place search only (`places.rs`, MOTIS geocoder as fallback, ADR-017);
  never for planning or live data (benchmark only, ADR-007).
- Cloud workspaces: Geofabrik is blocked → use openstreetmap.fr + `tools/fix_osm_refs.py` (ADR-009);
  background processes die after ~2 h, so restart `motis server` when needed.
- `motis server` reads `data/config.yml` (copied at import), not `config.yml`: a GBFS feed added
  after import must go into `data/config.yml` too (or re-import).
- Docker works in cloud workspaces after starting `dockerd` (background); builds need the
  proxy CA as a build secret and HTTPS apt sources — keep such tweaks in a scratch copy of the
  Dockerfile, not the repo one. Containers' own HTTPS fails there (TLS interception); fine at home.
- Ride mode can be tested with Playwright's `page.clock.setFixedTime` to jump into a trip.
- UI testing: Playwright, 390×844 touch viewport (`hasTouch`, `isMobile`), scroll with CDP
  `Input.dispatchTouchEvent`, press with `tap()`; flick the timetable to both ends; also 1280×800.
- Don't `pkill -f "overstap serve"`/`"motis server"` from a shell whose own command line contains
  that text (it kills itself); use `kill $(pidof overstap)` / `kill $(pidof motis)`.

## UI direction (user decisions)
- The user finds printed times hard to judge: always show time proportionally.
- **Build on the lane timeline first** (user, 2026-11-05): the bike-transit lane timetable
  (canvas board "A · Today: lane timeline", pages "Two corridors" D/E) is the base of the web UI;
  space-time charts are parked, not dropped.
- Space-time chart (x = best-case minutes to home, y = time), one shared chart for
  multiple corridors with highlighting; select a route → overlay alternative legs.
- **Ride mode** = next move + spare-time bars + fallback per change.
- Mockups live on the Design canvas "Overstap UI concepts" (claude.ai artifact).
