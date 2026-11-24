#!/usr/bin/env bash
# Keep MOTIS serving and up to date: the container entrypoint on the NAS, usable anywhere.
#
#   * First start without data/: fetch everything and import (~5 min).
#   * `motis server` runs in a loop; update.sh restarts it by killing the PID in motis.pid.
#   * Every day at UPDATE_AT (local time, default 03:30) update.sh refreshes the data.
#   * When config.yml takes NMBS real time from `overstap rt-fix` (localhost:8739, ADR-027),
#     that runs alongside, also in a loop: it matches NMBS's trip IDs to sncb.zip's.
#
# Environment: as update.sh, plus UPDATE_AT (HH:MM), TZ and OVERSTAP_BIN (the overstap
# binary for rt-fix; default: `overstap` on PATH, else this checkout's target/release).
set -uo pipefail

MOTIS_DIR="${MOTIS_DIR:-$(cd "$(dirname "$0")" && pwd)}"
MOTIS_BIN="${MOTIS_BIN:-$MOTIS_DIR/motis}"
UPDATE="$(cd "$(dirname "$0")" && pwd)/update.sh"
UPDATE_AT="${UPDATE_AT:-03:30}"
export MOTIS_DIR MOTIS_BIN
cd "$MOTIS_DIR"

log() { echo "$(date -u +%FT%TZ) run: $*"; }

if [ ! -d data ]; then
  log "no data yet: first import"
  "$UPDATE" --no-restart || { log "first import failed"; exit 1; }
fi

# NMBS GTFS-RT with fixed trip IDs, in the background.
rtfix=
if grep -q 'localhost:8739' config.yml data/config.yml 2>/dev/null; then
  OVERSTAP_BIN="${OVERSTAP_BIN:-$(command -v overstap || echo "$(dirname "$UPDATE")/../../target/release/overstap")}"
  if [ -x "$OVERSTAP_BIN" ]; then
    (
      trap 'kill $child 2>/dev/null; exit 0' TERM
      while true; do
        "$OVERSTAP_BIN" rt-fix --gtfs sncb.zip --listen 127.0.0.1:8739 & child=$!
        wait "$child"
        log "rt-fix exited ($?), restarting"
        sleep 5
      done
    ) &
    rtfix=$!
  else
    log "WARNING: config.yml takes NMBS real time from rt-fix, but there's no overstap binary ($OVERSTAP_BIN): no NMBS real time"
  fi
fi

# Nightly update, in the background.
(
  while true; do
    now=$(date +%s)
    next=$(date -d "today $UPDATE_AT" +%s)
    [ "$next" -le "$now" ] && next=$(date -d "tomorrow $UPDATE_AT" +%s)
    log "next update at $(date -d "@$next" '+%F %H:%M %Z')"
    sleep $(( next - now ))
    "$UPDATE" 2>&1 | tee -a update.log
  done
) &
updater=$!

server=
stop() {
  kill "$updater" 2>/dev/null
  [ -n "$rtfix" ] && kill "$rtfix" 2>/dev/null
  [ -n "$server" ] && kill "$server" 2>/dev/null
  exit 0
}
trap stop TERM INT

while true; do
  "$MOTIS_BIN" server -d data &
  server=$!
  echo "$server" > motis.pid
  wait "$server"
  log "motis server exited ($?), restarting"
  sleep 2
done
