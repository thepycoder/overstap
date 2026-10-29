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
deploy/motis/         config.yml + fetch-data.sh to self-host MOTIS
tools/                data preprocessing (OSM cleaner)
src/                  Rust orchestration layer: MOTIS client, last-mile rules, overshoot/delay
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

cargo test   # unit tests + end-to-end tests (skipped when MOTIS is down)
```

## Status

MOTIS runs with real Belgian data; all three use cases work end to end
(`docs/results-2026-10-29.md`). Next steps are in `TODO.md`.
