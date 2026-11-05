# overstap — context for Claude

Personal multimodal journey planner for Belgium, for one rider (Victor, lives in Lokeren).
Mixes NMBS trains, De Lijn buses/trams, shared bikes (Blue-bike, Donkey, Velo, Dott, Hoppy),
a folding bike and walking; re-plans live when delayed or stranded.

## Architecture
- **MOTIS 2 (v2.11.4) runs unmodified** as backend: GTFS + GTFS-RT (NMBS, De Lijn), GBFS, Belgium OSM.
  Config and data fetch: `deploy/motis/` (`fetch-data.sh`, `config.yml`).
- **Rust orchestration layer** (`src/`) calls MOTIS many times and stitches what it can't express:
  overshot/aboard-trip origin (`overshoot.rs`), hub-only and round-trip bike-share fixes (`lastmile.rs`),
  delay simulation. CLI: `overstap plan|overshot|geocode|health`, `MOTIS_URL` env var.
- Read first: `docs/design.md`, `docs/decisions.md` (ADRs — add one for every real decision),
  `docs/data-sources.md`, `TODO.md`, `TODO-sharing.md` (shared-mobility coverage), `docs/ui-concepts.md`.

## Working rules
- Run `cargo test` with MOTIS up (`MOTIS_URL=http://localhost:8080`); e2e tests skip when it's down.
- Record findings from real queries in `docs/results-*.md`; keep `TODO.md` current.
- Never use the Citymapper API at runtime (benchmark only, ADR-007).
- Cloud workspaces: Geofabrik is blocked → use openstreetmap.fr + `tools/fix_osm_refs.py` (ADR-009);
  background processes die after ~2 h, so restart `motis server` when needed.

## UI direction (user decisions)
- The user finds printed times hard to judge: always show time proportionally.
- Favourite: **space-time chart** (x = best-case minutes to home, y = time), one shared chart for
  multiple corridors with highlighting; select a route → overlay alternative legs.
- **Ride mode** = next move + spare-time bars + fallback per change.
- Mockups live on the Design canvas "Overstap UI concepts" (claude.ai artifact).
