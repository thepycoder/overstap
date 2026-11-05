# overstap on the NAS

Three containers from one image (`Dockerfile`, built by GitHub Actions on every push to
`main` → `ghcr.io/thepycoder/overstap:latest`):

| Service | What | Port |
|---|---|---|
| `motis` | MOTIS server; imports Belgium on first start; refreshes every night at `UPDATE_AT` (timetables daily, OSM weekly) and swaps the new data in with a health check and rollback (`deploy/motis/update.sh`) | 8080 (LAN, optional) |
| `overstap` | web UI + API (`overstap serve`), on the `proxy` network for cloudflared | 8738 on the LAN |
| `rtlog` | GTFS-RT history logger (`overstap rt-log`) → `data/rt/rt.sqlite` | — |

## Install

```sh
git clone https://github.com/thepycoder/overstap && cd overstap/deploy/nas
docker compose pull          # or: docker compose build (builds from this checkout)
docker compose up -d
docker compose logs -f motis # first import ≈ 10 min, needs ~4 GB RAM at its peak
```

On the UGREEN DXP2800 it runs as the UGOS Container Manager stack `overstap`
(`/volume1/docker/overstap`, created 2026-11-05 with the `ugos-nas-docker` skill). That copy
of `compose.yaml` has no `build: ../..` line: the NAS policy refuses build contexts outside
the stack folder, and the image comes from GHCR anyway. Update it with `ugos.py apply overstap
<file> --pull`. First import there: downloads 1.5 min, import 150 s.

Then in Cloudflare Zero Trust → Tunnels → your tunnel → Public hostnames: add e.g.
`overstap.projectwhy.be` → `http://overstap:8737` (same as bike-transit). The service
worker (ride-mode alerts) needs HTTPS, which the tunnel provides. The app has no login:
put it behind Cloudflare Access if it shouldn't be public.

Live since 2026-11-05: https://overstap.projectwhy.be (tunnel `ugreen-nas`, proxied CNAME),
public without Access by choice. Who visits: Cloudflare dashboard → projectwhy.be →
Security → Analytics, filter *Host equals overstap.projectwhy.be*; the sampled logs show
source IP, country, user agent and path (31 days on the Free plan). The `overstap`
container log has a line per request, starting with the visitor's IP and country
(`203.0.113.7 NL GET / → 200`; LAN requests show the peer address and `--`); successful
health checks aren't logged.

The GHCR package is private after the first build: either make it public (GitHub →
Packages → overstap → settings) or `docker login ghcr.io` on the NAS with a token that
can read packages.

## Resources

| | |
|---|---|
| RAM | MOTIS ≈ 1.2 GB serving, ≈ 4 GB during an import (nightly ≈ 1–2 min, weekly OSM ≈ 4 min); overstap and rtlog < 50 MB |
| Disk | `data/motis` ≈ 3 GB (inputs + data) + one copy during the nightly update; `data/rt` ≈ 50–70 MB/day estimated (SQLite) + ≈ 30 MB/day raw snapshots kept 30 days |
| Network | ≈ 250 MB/night (timetables), 800 MB/week (OSM), RT ≈ 3 MB/min while logging |

## Operations

- Update now: `docker compose exec motis /app/motis/update.sh`
- Logger summary: `docker compose exec rtlog overstap rt-report --db /data/rt/rt.sqlite`
- MOTIS config: `data/motis/config.yml` (copied from the image on first start; edit there).
  `update.sh` re-imports with it, so changes take effect at the next update.
- Logs of the nightly update: `data/motis/update.log`, `import.log`.
