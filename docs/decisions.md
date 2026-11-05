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
*2026-10-29 · accepted · place search amended by ADR-017*

**Decision.** The unauthenticated Citymapper web API may be used to compare
results, never as a runtime dependency.

---

## ADR-008 — Use the official MOTIS static binary; no Docker needed at runtime
*2026-10-29 · accepted*

**Context.** GitHub release downloads were blocked in the cloud workspace; the
`ghcr.io/motis-project/motis` image was reachable.

**Decision.** Extract the statically linked `/motis` binary (plus `/ui`) from the
image and run it natively. `deploy/motis/fetch-data.sh` does this when Docker is
available and falls back to the GitHub release otherwise.

---

## ADR-009 — Clean OSM extracts before import
*2026-10-29 · accepted*

**Context.** The openstreetmap.fr Belgium extract contains 3,041 ways that reference
nodes missing from the file. osr throws `osmium::invalid_location` in
`annotate_ways` and the import exits with code 1 **without an error message**
(found via gdb `catch throw`).

**Decision.** Prefer Geofabrik (reference-complete). When using another source,
run `tools/fix_osm_refs.py`, which drops those ways and the relations that use them.

---

## ADR-010 — Provider quirks are corrected in overstap, not in feeds
*2026-10-29 · accepted*

**Context.** Donkey Republic's GBFS has no geofencing zones or return constraint
→ MOTIS lets you drop the bike anywhere. Blue-bike is `ROUNDTRIP_STATION` →
MOTIS never offers it one-way, although a round trip is often the best plan.

**Decision.** Keep provider rules in overstap (`lastmile.rs`): hub-only providers
are re-routed to their best hub; round-trip providers are priced as tours.
Consider upstreaming a MOTIS config option for hub-only providers later.

---

## ADR-011 — Synthesise static GBFS for operators without a feed
*2026-10-29 · accepted*

**Context.** Hoppy (Vervoerregio Kempen: 485 e-bikes, 95 zones, return at any
zone) publishes no GBFS. Its zones are, however, recorded in DMOW's Hoppinpunt
dataset, which names the operators at every hub. Missing it cost a real trip:
Retie Markt had bikes while bus 47 was 36 min late.

**Decision.** `tools/hoppy_gbfs.py` turns the Hoppy-tagged hubs of one
vervoerregio into a static GBFS 2.3 feed: virtual stations (centroid + hub
polygon as `station_area`), one `electric_assist` vehicle type with
`return_constraint: any_station`, and a constant `--assume-bikes` per hub. MOTIS
reads it from a local directory (a non-HTTP `url` in `config.yml`).

**Consequences.**
- MOTIS models the return rule natively, so no overstap correction is needed.
- No live availability: a plan using Hoppy means "if a bike is there". Treat it
  as an option to check in the app, not a guarantee.
- MOTIS routes all rentals with osr's bike-sharing profile (~15 km/h), so e-bike
  legs come out ~25 % slower than reality — conservative.
- Hubs missing from the Hoppinpunt data (17 of 95 in Kempen) are invisible.
- Replace with a real feed as soon as Hoppy or IOK publishes one.

---

## ADR-012 — First web UI: port the bike-transit lane timetable; API passes MOTIS itineraries through
*2026-11-05 · accepted*

**Context.** The user's earlier app (bike-transit, Citymapper-based) has a lane
timetable they like: every option drawn to scale in time, packed into lanes like a
calendar, horizontal on desktop and vertical on a phone, with a stop-by-stop detail
and map. They asked to build overstap's first UI on those lane-timeline screens
rather than the space-time charts (ui-concepts rounds 1–4).

**Decision.**
- Copy its static frontend (`web/`, vanilla JS + Leaflet, no build step) and replace
  only the data layer: Citymapper normalisation → `normLeg()` over MOTIS legs; three
  tabs (folding / shared / foot) instead of Mixed / Classic; live refresh via MOTIS
  trips instead of Citymapper journey times and station boards; no sweep of several
  requests, since one MOTIS search covers a whole window.
- `overstap serve` (`src/server.rs`): tiny_http + blocking reqwest, a few threads. The
  API returns MOTIS-shaped itineraries (the `motis.rs` structs, serialised camelCase)
  plus `label`/`notes` where overstap changed them, so there is one adapter, in the
  browser. Synthetic legs (hub drop-off, Blue-bike tour) are built as MOTIS legs with
  geometry from MOTIS street routes.
- Everything the UI shows is computed per request; nothing is stored server-side.

**Consequences.**
- Fields the UI reads must exist in `motis.rs` (unknown MOTIS fields are dropped).
- Payloads are 50–150 KB per search (leg shapes); fine for one rider.
- The shared-bike tab costs 2 plans + up to ~30 street routes (cached per request):
  ~5 s. Parallelise if it annoys.
- The space-time chart can later be another view over the same `/api/journeys` data.

---

## ADR-013 — Own-bike speed is a MOTIS query parameter; rentals are re-timed in the UI
*2026-11-05 · accepted*

**Context.** MOTIS rides bikes at ~15 km/h; the rider does ~18 on the folding bike,
faster when it matters. bike-transit handled this by re-timing rides in the browser
and asking Citymapper a second time from earlier.

**Decision.** `/api/v6/plan` accepts `cyclingSpeed` (m/s). The folding tab asks
MOTIS at 18 km/h and again at 25 km/h with the window moved by what the faster ride
saves; the UI times your own rides at 18 and marks options that need more (up to 25)
with that speed. Rental e-bikes (Hoppy) are re-timed at 20 km/h in the UI only;
human-powered rentals keep MOTIS's time.

**Consequences.** E-bike options that only work at 20 km/h (e.g. Hoppy Retie → Mol for
IC 20:58) are still missed, because MOTIS searches rentals at 15 km/h. Fix: a second
shared search with `cyclingSpeed` for e-bike providers, if MOTIS applies it to rentals.

---

## ADR-014 — Nightly re-import with a health-checked swap; one image for the NAS
*2026-11-05 · accepted*

**Context.** GTFS-RT updates name trips by the static timetable's ids. With a week-old
import De Lijn matching fell to 74 % (NMBS 92 %); after a fresh import: 99 % and 98 %.
`first_day: TODAY` also means the 14-day window must move. MOTIS keeps hashes of every
input and redoes only the import steps whose inputs changed.

**Decision.**
- `deploy/motis/update.sh`: download timetables (daily) and OSM (weekly), keeping the old
  file if a download fails or is broken; regenerate Hoppy; import into a *copy* of `data/`
  (so unchanged steps are reused: ≈ 1 min daily); swap; restart; check a real query
  (Brussel-C → Antwerpen-C tomorrow); roll back on failure.
- `run.sh` supervises: first import, server loop (restart = kill the PID), nightly update
  at `UPDATE_AT`. It is the container entrypoint (`motis-run`), so no host cron is needed.
- One Debian-slim image with MOTIS (copied from the official 2.11.4 image), overstap, web
  and tools; compose runs it three times (motis, overstap, rtlog). Built by GitHub Actions.

**Consequences.** ≈ 10–20 s of MOTIS downtime per night, at 03:30. Disk needs room for
two copies of `data/` (≈ 1.4 GB each). Tested in a cloud workspace with Docker: build,
compose up, update with downloads, update with failed downloads (kept old data).

---

## ADR-015 — GTFS-RT history: changes only, in SQLite, plus sparse raw snapshots
*2026-11-05 · accepted*

**Context.** Reliability scoring (Phase 3) needs delay histories. Per minute NMBS sends
≈ 4k stop predictions (all with delay and time); De Lijn ≈ 80k stop entries of which
≈ 90 % are `NO_DATA`, with GPS-based delays that drift by seconds.

**Decision.** `overstap rt-log` polls both feeds every 60 s and writes a row only when a
stop first appears, changes status, or a delay moves ≥ 60 s; trip status changes
(cancellations) separately; strings interned in `trips`/`stops` tables. Raw feeds are kept
gzipped every 30 min for 30 days, so the parser can be improved and re-run. The GTFS-RT
types are hand-written prost structs (`src/gtfs_rt.rs`): no `protoc` at build time.

**Consequences.** Measured at morning rush: ≈ 1,550 rows/min, ≈ 110 bytes/row →
estimated 50–70 MB/day. Rows are predictions as seen at fetch time: "final" delays are the
last row before the predicted time passed (`rt-report` uses that). Joining to the static
schedule (route, scheduled times for De Lijn's NO_DATA stops) is left to analysis, which
needs the matching GTFS: keep dated copies of the zips if that becomes a problem.

---

## ADR-016 — Ride mode alerts from the page first, Web Push later
*2026-11-05 · accepted*

**Context.** Ride mode should buzz only when the next move, the spare time at the next
change or the arrival changes (canvas "Ride_Glance"), and plan B must be ready.

**Decision.** v0 runs in the browser: the ride page refreshes live times every 30 s and
plan B (`POST /api/ride`, one planner run per change, in parallel) every 3 min, compares
the three facts and shows a notification through the service worker (`web/sw.js`), plus
a "get off soon" alert 2 min before each stop. Switching to plan B is one tap (no
automatic switch yet).

**Consequences.** Alerts arrive while the page is open or recently backgrounded; a phone
that freezes the tab misses them. Real push needs a server-side watcher (rides stored in
overstap, evaluated every minute, Web Push with VAPID keys) — the next step.

---

## ADR-017 — Place search from Citymapper, MOTIS geocoder as fallback
*2026-11-05 · accepted*

**Context.** The MOTIS geocoder (adr, OSM only) misses venues and hamlets ("'t Veld,
Retie") and snaps venues to centroids (Nekkerhal). Citymapper's web search is Google's
and finds them; bike-transit used it for a year. It is the website's own API: no key,
undocumented, could change or block us without notice.

**Decision.** The web UI's place search, place resolve and reverse geocoding call
citymapper.com first (`src/places.rs`: paced at ≥ 350 ms between requests, cached
5 min / 1 day, 8 s timeout). On an error or no result, search and reverse fall back to
the MOTIS geocoder. `--no-citymapper` / `OVERSTAP_NO_CITYMAPPER=true` switches it off.
Planning, live times and the CLI's free-text places stay on MOTIS: ADR-007 still holds
for everything except place search.

**Consequences.** Google hits come without coordinates: picking one costs one more
request (`/api/resolve`), and only works while Citymapper does. Queries and the map
position go to Citymapper. MOTIS keeps `geocoding: true` for now, so there's no RAM
saving yet; turning it off would remove the fallback, decided after measuring on the NAS.

---

## ADR-018 — Your own bike has a place; the planner picks the leg it's good for
*2026-11-05 · accepted*

**Context.** The tabs were either/or: the folding bike for the whole trip, or shared bikes /
walking for every street leg. But your ordinary (non-folding) bike is often part of a trip
too, and which leg depends on where it is: with you at home (ride to the station, leave it
there), or still at Lokeren station from this morning (coming back from Brussels, the last
leg is your bike). You don't know beforehand which leg that is, so you shouldn't have to
assign it; you only know where the bike is.

**Decision.** You tell overstap *where your bike is*: `none`, `here` (with you) or parked at a
place (`bike=here` / `bike=lat,lng` in the API, `--bike here|lat,lon` in the CLI,
`planner::OwnBike`). It never goes on board, and the tab (Shared bikes, On foot) covers
every other leg:
- **Here** (or you start within 300 m of where it's parked): `BIKE` joins MOTIS's
  pre-transit and direct modes without `requireBikeTransport`, so the bike stays at the
  boarding stop. Post-transit is untouched, so the Shared tab's last-mile fixes still apply.
- **Parked at P**: besides the tab's own journeys, overstap plans to P (getting to the
  first stop as the tab does, ≤ 10 min walk from the last stop to the bike) and adds the ride
  from P to the destination (+1 min to unlock), only when that ride fits the max bike time.
  Arriving by T plans to P by T minus the ride. Both kinds come back in one list, so the
  timetable shows your bike next to the shared bikes.

The web UI keeps the bike's place in the browser (a select next to the time, and a place
field when it's parked) because it's a fact about the world, not about one search. Starting
Ride mode on a route moves the bike along: to the station it's left at, or to the
destination when you ride it there (so the next trip from there sees it "here"). Ride-mode
plan B keeps a parked bike (it's still waiting) but drops a bike that was with you at the
start (it's parked behind you). Own-bike legs are re-timed at your speed in the UI (any
`BIKE` leg without a rental is yours); On foot also sends the speed to MOTIS, Shared doesn't,
because MOTIS's cycling speed would also time the rentals.

**Consequences.** One extra MOTIS query (plus a cached street route) when the bike is
parked. The bike is used for one leg per trip: not picked up mid-trip and ridden between
two stations, and not taken back from the destination. No locking time at the station
for the "here" case, and the 25 km/h fast-rider search only runs for the folding bike.
Moving the bike happens when you start a ride, not when you actually travel: change it by
hand if you didn't go.

## ADR-019 — Variants of a route: braided from the search's own results first
*2026-11-05 · accepted*

**Context.** The brainstorm's "Variations of one route" (canvas board 1, braided lane) asks
what else you could do around a chosen route: reach the same train another way, take the
next one, end on a Donkey instead of a Blue-bike. The full version needs new backend work
(per-leg alternatives: ride or bus to each stop of the chosen train, adjacent departures,
dominance pruning). But a search already returns many routes that overlap the chosen one,
and the Shared tab's last-mile variants are separate routes too.

**Decision.** The route detail gets a second view, **Variants** (`view=braid` in the URL),
next to the stop list. It is built in the page from `state.journeys` (shown and beaten),
with no new API:
- **Strands** come from routes sharing at least one vehicle (`tripId`) with the open route.
  Wherever they differ, before the first shared ride, between two, or after the last,
  that stretch is drawn in a column beside the lane. It leaves the lane where that route
  gets off a shared ride and rejoins it where it gets back on, at an intermediate stop if
  need be (a tick on the lane). Getting-there strands go left, the rest right.
- **Ghosts** come from routes sharing no vehicle: their first ride from a station the open
  route also leaves from, within 90 min, as a dashed block with its home time.
- Each strand or ghost belongs to one real route. Tapping it first shows a card (user,
  2026-11-05: details before slotting it in): how it changes your leave and home times, its
  rides, where it gets off or back on yours, and both routes to scale on one axis. **Use this
  route** then opens that route (still as Variants). A route that differs in one place only
  is preferred as that strand's target.
- Faded = beaten: leaves no later and gets home no sooner, with ±2 min counting as the same.
- On a phone the map hides in this view (the lane needs the height); Stops brings it back.
- Changes show the time they give you; orange when that is all "walking", because MOTIS
  pads a change to the station's minimum transfer time with a 0 m walk.

**Consequences.** What you see depends on what MOTIS returned. Real Retie → Lokeren
searches give other trains at Mol and Berchem (ghosts), the Tielen/Lier route joining
the same S34, and on the Shared tab the last-mile choice (Blue-bike / Donkey / walk).
They do not give the ride-to-Geel/Herentals ladder or the bus-vs-bike-to-Mol choice from
the mockup: that needs the backend step (`/api/variants`, per-leg alternatives), which can
feed the same view later. Loading earlier or later routes in the timetable adds variants.
