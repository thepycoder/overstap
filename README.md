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
deploy/motis/         how we self-host MOTIS
src/                  the Rust orchestration layer (skeleton)
```

## Status

Design phase. Nothing routes yet. See `TODO.md` for the next steps.
