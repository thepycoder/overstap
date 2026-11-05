#!/usr/bin/env bash
# Refresh MOTIS's inputs and data, safely. Meant to run nightly (run.sh does that).
#
#   1. Download fresh timetables (every run) and OSM (when older than OSM_MAX_AGE_DAYS);
#      a failed or broken download keeps the previous file.
#   2. Regenerate the Hoppy GBFS feed (non-fatal).
#   3. Import into a copy of the current data (data.next/). MOTIS re-runs only the steps
#      whose inputs changed: daily that is the timetable (~1-2 min), weekly also the street
#      graph and geocoder (~3 min).
#   4. Swap data.next/ in, restart the server, check it answers a real query, and swap
#      back if it doesn't.
#
# Usage: update.sh [--no-restart]   (--no-restart: import and swap only, e.g. first install)
#
# Environment:
#   MOTIS_DIR         working directory with config.yml and the inputs (default: this script's dir)
#   MOTIS_BIN         motis binary (default: $MOTIS_DIR/motis)
#   OSM_MAX_AGE_DAYS  re-download OSM when older than this (default 7)
#   RESTART_CMD       command that restarts the server (default: kill the PID in motis.pid,
#                     which run.sh restarts)
#   MOTIS_URL         where the restarted server answers (default http://localhost:8080)
set -euo pipefail

TOOLS="$(cd "$(dirname "$0")/../../tools" 2>/dev/null && pwd || echo /app/tools)"
MOTIS_DIR="${MOTIS_DIR:-$(cd "$(dirname "$0")" && pwd)}"
MOTIS_BIN="${MOTIS_BIN:-$MOTIS_DIR/motis}"
OSM_MAX_AGE_DAYS="${OSM_MAX_AGE_DAYS:-7}"
MOTIS_URL="${MOTIS_URL:-http://localhost:8080}"
RESTART=1
[ "${1:-}" = "--no-restart" ] && RESTART=0
cd "$MOTIS_DIR"

log() { echo "$(date -u +%FT%TZ) update: $*"; }

exec 9>update.lock
if ! flock -n 9; then log "another update is running"; exit 0; fi

# --- 1. downloads -----------------------------------------------------------------------

# fetch URL FILE CHECK: download to FILE.tmp, keep it only if CHECK FILE.tmp passes.
fetch() {
  local url=$1 file=$2 check=$3
  if curl -fsSL --retry 3 --retry-delay 20 --max-time 1800 -o "$file.tmp" "$url" && $check "$file.tmp"; then
    mv "$file.tmp" "$file"
    log "downloaded $file ($(du -h "$file" | cut -f1))"
  else
    rm -f "$file.tmp"
    if [ -e "$file" ]; then log "WARNING: $file download failed, keeping the old one"; else log "ERROR: no $file"; return 1; fi
  fi
}

gtfs_ok() {  # a readable zip with stop_times.txt (catches truncated downloads and error pages)
  python3 -c "import sys, zipfile; sys.exit('stop_times.txt' not in zipfile.ZipFile(sys.argv[1]).namelist())" "$1" 2>/dev/null
}
pbf_ok() { [ "$(stat -c %s "$1")" -gt 300000000 ]; }  # Belgium is ~800 MB

fetch https://sncb-opendata.hafas.de/gtfs/static/c21ac6758dd25af84cca5b707f3cb3de sncb.zip gtfs_ok
fetch https://api-management-opendata-production.azure-api.net/api/gtfs/feed/delijn/static delijn.zip gtfs_ok

if [ ! -e belgium.osm.pbf ] || [ -n "$(find belgium.osm.pbf -mtime +"$OSM_MAX_AGE_DAYS")" ]; then
  # Geofabrik is reference-complete; the openstreetmap.fr extract must be cleaned (ADR-009).
  if ! fetch https://download.geofabrik.de/europe/belgium-latest.osm.pbf belgium.osm.pbf pbf_ok; then :; fi
  if [ -z "$(find belgium.osm.pbf -mmin -60 2>/dev/null)" ]; then
    log "Geofabrik unavailable, trying openstreetmap.fr"
    if curl -fsSL --max-time 1800 -o belgium-raw.osm.pbf https://download.openstreetmap.fr/extracts/europe/belgium-latest.osm.pbf \
      && python3 "$TOOLS/fix_osm_refs.py" belgium-raw.osm.pbf belgium.osm.pbf.tmp && pbf_ok belgium.osm.pbf.tmp; then
      mv belgium.osm.pbf.tmp belgium.osm.pbf
      log "downloaded and cleaned belgium.osm.pbf (openstreetmap.fr)"
    else
      log "WARNING: OSM refresh failed, keeping the old extract"
    fi
    rm -f belgium-raw.osm.pbf belgium.osm.pbf.tmp
  fi
fi

# --- 2. generated feeds -------------------------------------------------------------------

python3 "$TOOLS/hoppy_gbfs.py" --region Kempen --out gbfs/hoppy-kempen >/dev/null 2>&1 \
  && log "regenerated Hoppy GBFS" || log "WARNING: Hoppy GBFS not refreshed (keeping the old one)"

# --- 3. import into a copy ------------------------------------------------------------------

rm -rf data.next
if [ -d data ]; then cp -a data data.next; fi
t0=$(date +%s)
if ! "$MOTIS_BIN" import -c config.yml -d data.next > import.log 2>&1; then
  log "ERROR: import failed (see import.log); still serving the old data"
  rm -rf data.next
  exit 1
fi
log "import done in $(( $(date +%s) - t0 )) s"

# --- 4. swap, restart, check ----------------------------------------------------------------

restart() {
  if [ -n "${RESTART_CMD:-}" ]; then eval "$RESTART_CMD"
  elif [ -s motis.pid ]; then kill "$(cat motis.pid)" 2>/dev/null || true
  else log "WARNING: don't know how to restart the server (no RESTART_CMD, no motis.pid)"
  fi
}

healthy() {  # up, and a real query (Brussel-Centraal → Antwerpen-Centraal tomorrow 09:00) finds trains
  local when; when="$(date -u -d 'tomorrow 08:00' +%FT%TZ)"
  for _ in $(seq 1 60); do
    if curl -fs -m 5 "$MOTIS_URL/api/v1/health" >/dev/null 2>&1; then
      curl -fs -m 60 "$MOTIS_URL/api/v6/plan?fromPlace=50.8454,4.3571&toPlace=51.2172,4.4211&time=$when" \
        | python3 -c "import sys, json; sys.exit(not json.load(sys.stdin)['itineraries'])" 2>/dev/null
      return $?
    fi
    sleep 5
  done
  return 1
}

rm -rf data.prev
[ -d data ] && mv data data.prev
mv data.next data
if [ "$RESTART" = 0 ]; then rm -rf data.prev; log "swapped in new data (no restart)"; exit 0; fi

restart
sleep 5
if healthy; then
  rm -rf data.prev
  log "serving new data"
else
  log "ERROR: new data fails the health check, rolling back"
  rm -rf data.failed && mv data data.failed
  [ -d data.prev ] && mv data.prev data
  restart
  sleep 5
  healthy && log "rolled back, serving the old data" || log "ERROR: old data unhealthy too"
  exit 1
fi
