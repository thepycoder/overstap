# Data sources

Status as researched on 2026-10-29. "Verify" = not yet confirmed hands-on.

## Public transport

| Source | Format | Access | Notes |
|---|---|---|---|
| NMBS/SNCB timetable | GTFS static (daily) | Free, requires signing a licence contract | Also NeTEx. Real-time updated every 30 s |
| NMBS/SNCB real time | GTFS-RT | Same contract | Delays, cancellations, platforms |
| iRail | REST (liveboards, connections) | Open, unofficial | Good for prototyping; not our primary source |
| De Lijn timetable | GTFS static | API key via data.delijn.be (portal.delijn.be) | Daily updates |
| De Lijn real time | GTFS-RT | API key, separate subscription | Every minute. Limits: 864k calls/product/day, 6000/min |
| De Lijn Open Data API | REST | API key | Search, stops, detours, its own routing — handy for debugging |
| TEC (Wallonia) | GTFS | Verify | Lower priority |
| STIB/MIVB (Brussels) | GTFS + API | API key (opendata.stib-mivb.be) | Lower priority |
| Transitous feed list | JSON feed definitions | Open (git) | Reuse the Belgian entries for our MOTIS config. Verify exact file (likely `feeds/be.json`) and licences |

Note: Transitous's public API (api.transitous.org) has a usage policy; we
self-host MOTIS anyway, so we only reuse their *feed definitions*.

## Shared bikes (GBFS)

GBFS = General Bikeshare Feed Specification: JSON files for station info,
station status (bikes/docks available), free-floating vehicles, and
geofencing zones. MOTIS ingests GBFS directly.

| Operator | Type | GBFS? |
|---|---|---|
| Blue-bike | station-based, mostly return-to-same-station | Verify |
| Velo Antwerpen | docked | Verify |
| Donkey Republic | docks / virtual parking zones | Verify |
| Dott | free-floating | Verify |
| Others (Bolt, Mobit, …) | | Verify |

Check each against the MobilityData GBFS systems catalog.

Return rules (e.g. Blue-bike zones that allow returning at another station)
are probably *not* in GBFS and will need a small hand-maintained config.

## Streets

| Source | Use |
|---|---|
| OpenStreetMap, Geofabrik Belgium extract | MOTIS street graph (osr): walk/bike footpaths, first/last mile |
| BRouter (optional) | More tunable bike profiles if osr's bike routing is too crude |
| Citymapper web API (unauthenticated) | **Benchmark only**, never a dependency — can vanish any day. Reuse the old Citymapper project's client for comparisons |
| OsmAnd | Reference for bike-routing quality |

## Our own data

| Dataset | Source | Why |
|---|---|---|
| GTFS-RT history | Logger polling NMBS + De Lijn RT | Delay distributions → reliability scoring. Start ASAP: value grows with time |
| Station transfer times | Hand-curated + OSM indoor data | Replace NMBS's blanket 5-min rule with real platform-to-platform times |
| Folding-bike penalties | Config | Fold/unfold time, stairs, crowding |
