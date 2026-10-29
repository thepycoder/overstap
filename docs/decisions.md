# Decision log

Lightweight ADRs. Newest at the bottom. Each entry: context → decision → consequences.

---

## ADR-001 — Build a personal planner rather than use an existing one
*2026-10-29 · accepted*

**Context.** Google Maps ignores bikes around transit; Citymapper's mixed mode
ignores buses and doesn't model folding bikes vs. bike-share constraints; NMBS
hides tight transfers. None re-plans well when delayed or stranded.

**Decision.** Build a single-user planner optimised for one rider's modes and
risk tolerance. Latency is not critical; search can be exhaustive.

---

## ADR-002 — Use one unified data layer, not live API stitching per mode
*2026-10-29 · accepted*

**Context.** Every source exists as a standard feed (GTFS, GTFS-RT, GBFS, OSM).

**Decision.** Ingest all feeds into one routing backend instead of calling each
operator's API at query time.

---

## ADR-003 — MOTIS as backend, run unmodified
*2026-10-29 · accepted*

**Options.** OTP2 (Java, mature, JVM-heavy); fork MOTIS (C++); custom RAPTOR
from scratch; MOTIS unmodified + orchestration layer.

**Decision.** Self-host MOTIS unmodified. Forking means working in heavily
templated, performance-tuned C++ (nigiri), slow compile/iterate cycles, and
rebasing on an active upstream.

**Consequences.** Anything MOTIS can't express is done by calling it repeatedly
and stitching results. Features with general value (cycling transfers, custom
min transfer time) can be proposed upstream instead of forked.

---

## ADR-004 — Rust orchestration layer
*2026-10-29 · accepted*

**Decision.** Write the orchestration layer in Rust (originally Python was
suggested; user chose Rust). Talk to MOTIS via its OpenAPI REST spec
(generate or hand-write a typed client).

**Escalation path.** If stitching hits a wall, implement McRAPTOR in Rust over
the same GTFS, still using MOTIS for street legs.

---

## ADR-005 — Reuse Transitous feed definitions
*2026-10-29 · accepted*

**Decision.** Take the Belgian feed list from the Transitous repo as the
starting MOTIS config; add our own feeds (extra GBFS, etc.) on top.
Transitous itself is mostly config + a MOTIS deployment; its public API is not
used.

---

## ADR-006 — Log GTFS-RT from day one
*2026-10-29 · accepted*

**Decision.** Run a GTFS-RT logger before the planner works. Delay
distributions need months of history; this is the long pole for reliability
scoring (use case 3).

---

## ADR-007 — Citymapper only as a benchmark
*2026-10-29 · accepted*

**Decision.** The unauthenticated Citymapper web API may be used to compare
results, never as a runtime dependency.
