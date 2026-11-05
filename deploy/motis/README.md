# Self-hosting MOTIS

## Sizing

- MOTIS 2 is a single binary (or the upstream `Dockerfile`). Workflow:
  `motis config <osm.pbf> <gtfs...>` → `motis import` → `motis server` (port 8080).
- Reference point from upstream: a Benelux + Germany demo (≈400 MB GTFS, ≈2 GB
  OSM) runs on 16 GB RAM / 4 vCPU using ≈22 GB disk. Belgium alone is a fraction
  of that, so a small homelab box or a modest VPS is enough.
- Import is the RAM-hungry step. Option: import on a beefy machine, ship the
  `data/` folder (e.g. `.tar.zst`) to a smaller server that only serves.
- Real-time updates are applied by the server process; keep an eye on update
  latency once GTFS-RT feeds are added.

## Inputs

- OSM: Geofabrik `belgium-latest.osm.pbf` (and possibly a small buffer into NL/FR/LU later)
- GTFS: NMBS, De Lijn (+ TEC, STIB later) — see `docs/data-sources.md`
- GTFS-RT: NMBS, De Lijn
- GBFS: per operator

## Quick start (tested 2026-10-29, MOTIS v2.11.4)

Use `./fetch-data.sh` (binary + inputs + first import), then `./run.sh` (server + nightly update). `update.sh` refreshes and re-imports with a health-checked swap; see its header.
Measured: 2 min 51 s import on 2 vCPU, 1.4 GB data dir, ~1.2 GB RAM serving.

Manual equivalent:

```sh
TARGET=linux-amd64   # or linux-arm64, macos-arm64
wget https://github.com/motis-project/motis/releases/latest/download/motis-$TARGET.tar.bz2
tar xf motis-$TARGET.tar.bz2
wget https://download.geofabrik.de/europe/belgium-latest.osm.pbf
# put GTFS zips next to it, then:
./motis config belgium-latest.osm.pbf nmbs.zip delijn.zip
# edit config.yml: add GTFS-RT + GBFS feeds, API keys
./motis import
./motis server
```

Ensure GTFS timetables are current; an outdated feed has no trips for upcoming dates.

## Adding a feed after import

`motis server` reads `data/config.yml`, the copy `motis import` makes, not the
`config.yml` next to it. A GBFS feed (they load at runtime) can be added without
re-importing by adding it to both files and restarting the server. Timetables
(GTFS) need a re-import. For Hoppy: `python3 ../../tools/hoppy_gbfs.py --region Kempen --out gbfs/hoppy-kempen`
(the cloud workspace used an absolute path in `data/config.yml`).
