#!/usr/bin/env bash
# Download all inputs for a Belgian MOTIS instance into ./ (run from deploy/motis).
# Tested 2026-10-29 with MOTIS v2.11.4.
set -euo pipefail
cd "$(dirname "$0")"

# 1. MOTIS binary: GitHub release, or extract it from the official image.
if [ ! -x motis ]; then
  if docker info >/dev/null 2>&1; then  # CLI alone isn't enough: needs a running daemon
    cid=$(docker create ghcr.io/motis-project/motis:latest)
    docker cp "$cid":/motis ./motis; docker cp "$cid":/ui ./ui; docker rm "$cid" >/dev/null
  else
    curl -fL https://github.com/motis-project/motis/releases/latest/download/motis-linux-amd64.tar.bz2 | tar xj
  fi
fi

# 2. Timetables (both public, no key needed as of 2026-10).
curl -fL -o sncb.zip   https://sncb-opendata.hafas.de/gtfs/static/c21ac6758dd25af84cca5b707f3cb3de
curl -fL -o delijn.zip https://api-management-opendata-production.azure-api.net/api/gtfs/feed/delijn/static

# 3. Street network. Geofabrik is preferred (reference-complete). The
#    openstreetmap.fr extract has ways with missing nodes that crash osr
#    (osmium::invalid_location), so it must be cleaned first.
if curl -fL -o belgium.osm.pbf https://download.geofabrik.de/europe/belgium-latest.osm.pbf; then
  :
else
  curl -fL -o belgium-raw.osm.pbf https://download.openstreetmap.fr/extracts/europe/belgium-latest.osm.pbf
  python3 -m pip install --quiet osmium
  python3 ../../tools/fix_osm_refs.py belgium-raw.osm.pbf belgium.osm.pbf
  rm belgium-raw.osm.pbf
fi

# 4. Operators without a public GBFS feed: build a static one (ADR-011).
python3 ../../tools/hoppy_gbfs.py --region Kempen --out gbfs/hoppy-kempen

echo "Now: ./motis import && ./motis server"
