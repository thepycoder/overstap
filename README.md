# overstap

A personal multimodal journey planner for Belgium, built for one rider:
someone who mixes trains (NMBS/SNCB), buses and trams (De Lijn, later TEC/STIB),
shared bikes (Blue-bike, Velo Antwerpen, Donkey Republic, Dott…), a folding bike,
and walking — and who wants the planner to *re-plan live* when a train is late or
they've overshot their stop.

> *Overstap* is Dutch for "transfer". Rename freely.

## Architecture in one paragraph

[MOTIS](https://github.com/motis-project/motis) runs **unmodified** as the data and
routing backend: it ingests GTFS/GTFS-RT timetables, GBFS bike-share feeds and an
OpenStreetMap extract, and answers transit and street-routing queries over a REST
API. On top sits **overstap**, a Rust orchestration layer that calls MOTIS many
times and stitches the results into journeys MOTIS can't express on its own:
cycling between two transit legs, rental bikes that must be returned, starting
from "I'm on train IC 1234", delay-risk scoring, and contingency plans
("if X is >4 min late, get off at Lier and take bus Y"). Feed configuration is
borrowed from [Transitous](https://transitous.org)'s Belgian feed list.

## Layout

```
README.md             this file
TODO.md               living task list
docs/design.md        problem, use cases, architecture, algorithm background
docs/data-sources.md  every feed/API we plan to use, with status and caveats
docs/decisions.md     decision log (ADR-style) — why things are the way they are
deploy/motis/         config.yml, fetch-data.sh (setup), update.sh (nightly refresh), run.sh (supervisor)
deploy/nas/           compose.yaml + README for the home NAS (Dockerfile at the root)
tools/                data preprocessing (OSM cleaner, Hoppy GBFS generator)
src/                  Rust orchestration layer: MOTIS client, planner, last-mile rules,
                      overshoot/delay, web server (server.rs)
web/                  web UI: lane timetable + route detail + map (vanilla JS, no build step)
tests/                end-to-end tests against a running MOTIS
```

## Quick start

```sh
cd deploy/motis && ./fetch-data.sh && ./motis import && ./motis server   # ~3 min import
cargo build
export MOTIS_URL=http://localhost:8080

# Use case 1: compare last-mile options (walk / folding bike / shared bikes)
overstap plan 51.1069,3.9859 51.02566,4.50349 --at "2026-10-30 08:00" --carry shared
overstap plan 51.1069,3.9859 51.02566,4.50349 --at "2026-10-30 08:00" --carry folding

# Use cases 2/3: on a train, missed the stop or delayed → where to get off?
overstap overshot <dest> --trip <tripId> --now "2026-10-30 08:42" [--folding] [--delay 15]

# Web UI + JSON API on http://127.0.0.1:8737 (serves ./web)
overstap serve [--listen 0.0.0.0:8737] [--web path/to/web]

# GTFS-RT history (delays, cancellations) for reliability scoring
overstap rt-log --db rt.sqlite --raw-dir raw     # runs until stopped
overstap rt-report --db rt.sqlite

cargo test   # unit tests + end-to-end tests (skipped when MOTIS is down)
```

## Web UI

`overstap serve` serves the lane timetable from the earlier bike-transit app, now fed by
overstap instead of Citymapper. One column (phone) or row (desktop) per option, drawn to
scale in time, packed into lanes like a calendar; tap one for the stop-by-stop detail and
the map. Tabs: **Folding bike** (own bike on both ends and on board, timed at 18 km/h;
trains you'd only catch at 25 km/h are marked with that speed), **Shared bikes**
(Blue-bike round trips, Donkey via hub, Hoppy, Velo… with their parking rules), **On foot**.
Live delays, cancellations and tracks refresh every minute. Saved routes and recent
places stay in the browser.

**Ride mode** (the Ride button on a route): a big countdown to your next move, the rest of
the trip to scale, spare time per change (grey = the walk between platforms), plan B per
change ("If missed: IC 21:58 → arrive 23:54"), bikes at a shared-bike pickup, and an alert
when the next move, a change's spare time or the arrival changes, or 2 min before you get off.

## Running it for real

`deploy/nas/`: one image, three containers (MOTIS with a nightly self-update, the web UI,
the RT logger). See `deploy/nas/README.md`.

API (all JSON; legs are MOTIS itineraries passed through, plus `label`/`notes` where
overstap changed them): `GET /api/journeys?start=lat,lng&end=lat,lng&mode=folding|shared|foot&time=RFC3339&arrive=1`,
`POST /api/live {"trips":[tripId…]}`, `GET /api/search?q=&near=`, `GET /api/resolve?id=`, `GET /api/reverse?lat=&lng=`,
`GET /api/config`, `GET /healthz`. See `src/server.rs`. Place search asks Citymapper first and
falls back to the MOTIS geocoder (ADR-017); `--no-citymapper` uses MOTIS only.

## Status

MOTIS runs with real Belgian data; all three use cases work end to end
(`docs/results-2026-10-29.md`). A first web UI (lane timetable) runs on top
(`docs/results-2026-11-05.md`). Next steps are in `TODO.md`.
