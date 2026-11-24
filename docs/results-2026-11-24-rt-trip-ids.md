# NMBS real time dropped: GTFS-RT trip IDs don't match the static feed — 2026-11-24

Trigger: Antwerpen-Berchem → Mol, ~07:33. S33 train 2957 (Antwerpen-C 07:24 → Mol 08:25)
sat on track 5 well after its 07:29 departure; overstap showed it on time (gone).

## What overstap had

Live instance, `/api/journeys` + `/api/live`, 06:34 UTC: train 2957 `realTime: false`,
planned times only. Same for IC 4308 (08:09). IC 4307 (07:09) did have RT (+1′ Herentals).

| Train | Static trip_id (in MOTIS) | RT trip_id (NMBS feed) | RT in overstap |
|---|---|---|---|
| IC 4307 | `88____:007::8821006:8832664:11:842:20261211` | same | yes |
| S33 2957 | `88____:A72::8821006:8832409:13:825:20270930` | `…:825:20271210` | **no** |
| IC 4308 | `88____:007::8821006:8832664:11:942:20270930` | `…:942:20271210` | **no** |

The RT feed (header 06:37 UTC) had 2957 at **+4′ from Antwerpen-C, +5′ at Berchem**
(stop 8821121), matching what was seen on the platform. The data was right; MOTIS dropped
it because it matches GTFS-RT on exact `trip_id` + `start_date`.

## It's NMBS's two feeds, not our import

- NMBS static GTFS fetched 06:38 UTC (Last-Modified 06:14 UTC, `feed_version` 2026-11-24,
  `feed_end_date` **20270930**) still has `…:825:20270930`: the NAS import is current.
- The last `:`-segment of NMBS trip IDs is the trip's last service date. The static export
  clips it to `feed_end_date` (20270930); the HAFAS RT export uses the full horizon
  (end of the 2027 timetable year: 20271209–20271211).
- 615 trip updates in the feed; **244 (40 %) match no static trip**. By RT suffix:
  20271210 ×207, 20271211 ×29, 20261125 ×6, 20271209 ×1, 20261127 ×1.
  So it hits every train that runs on unchanged past the 13 Dec timetable change.

## Fixable on our side

Matching on the trip_id without its last segment, restricted to static trips whose service
runs on the RT `start_date`: **239 of 244** resolve to exactly one static trip; 5 have two
candidates (variants with different end dates both active today), which stop times would
settle. MOTIS can't be told to match fuzzily, so the rewrite has to sit in front of it:
an RT proxy (overstap serves the rewritten feed, `config.yml` points the sncb `rt:` url at
it). Rewrite trip_id bytes in place rather than re-encode: `gtfs_rt.rs` is a subset of
the schema and would drop fields.

Not checked: since when. The rt-log history on the NAS would show when the
20271210 suffixes appeared (likely when the static feed was extended past 13 Dec).

## Fix: `overstap rt-fix` (ADR-027), tested the same morning

| Feed MOTIS reads | MOTIS `GTFS-RT update stats for tag=sncb` | 2957 at Berchem |
|---|---|---|
| NMBS direct (frozen 06:52 snapshot) | 369 / 613 resolved, `trip_resolve_error=244` (39.8 %) | 06:29, no real time |
| same snapshot through rt-fix | 610 / 613, 3 errors (0.5 %) | **06:29 → 06:34**, real time |
| live feed through rt-fix (`motis-run` → `run.sh`) | 608 / 612 (99.3 %) | — |

Berchem departures 06:20–08:00 with real time: 15 of 40 before, 29 of 40 after. The 3 left:
one has yesterday's `start_date` (outside `first_day: TODAY`), two are cancellations.
rt-fix output vs input (Google's `gtfs-realtime-bindings`): same 613 entities and header; 241
trip IDs changed, nothing else; every update now names a static trip, no (trip, date) twice.
Matching checked against stop times: 238 of 244 rewritten trips have every RT stop at the
static time; the other 6 are the only candidate with one stop off (a foreign stop or the
last stop's arrival). The static index (30,421 trips, 1.24 M calendar dates) loads in ~140 ms.
