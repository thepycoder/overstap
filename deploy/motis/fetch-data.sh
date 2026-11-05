#!/usr/bin/env bash
# Set up a Belgian MOTIS instance in ./ (run from deploy/motis): binary, inputs, first import.
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

# 2. Inputs (timetables, OSM, Hoppy) and the first import: the same script as the nightly update.
./update.sh --no-restart

echo "Now: ./run.sh   (serves MOTIS, re-imports nightly at UPDATE_AT, default 03:30)"
