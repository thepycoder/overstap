# Data sources

Status checked hands-on on 2026-10-29 (download + MOTIS import) unless marked "verify".

## Public transport

| Source | Format | URL | Access | Notes |
|---|---|---|---|---|
| NMBS/SNCB timetable | GTFS static | `https://sncb-opendata.hafas.de/gtfs/static/c21ac6758dd25af84cca5b707f3cb3de` | Public (key in URL, from Transitous) | 39 MB zip. `transfers.txt` sets 240 s at most stations, 300 s at some |
| NMBS/SNCB real time | GTFS-RT | `https://sncb-opendata.hafas.de/gtfs/realtime/c21ac6758dd25af84cca5b707f3cb3de` | Public | ~130 KB per fetch |
| De Lijn timetable | GTFS static | `https://api-management-opendata-production.azure-api.net/api/gtfs/feed/delijn/static` | Worked **without** a key (Transitous sends a `bmc-partner-key` header) | 208 MB zip, 1.7 GB `stop_times.txt` |
| De Lijn real time | GTFS-RT | `…/api/gtfs/feed/delijn/rt/trip-update?format=protobuf` (+ `/rt/alert`) | Worked without a key | ~1.8 MB per fetch |
| NMBS (alt. portal) | GTFS + RT | `…/api/gtfs/feed/nmbssncb/static`, `…/nmbssncb/rt/trip-update` | Same Belgian Mobility portal | From Transitland atlas; not tested |
| TEC | GTFS + RT | `…/api/gtfs/feed/tec/static`, `…/tec/rt/trip-update` | Same portal | Lower priority |
| STIB/MIVB | GTFS | `…/api/gtfs/feed/stibmivb/static` | Same portal | Lower priority |
| iRail | REST | api.irail.be | Open, unofficial | Prototyping only |

The "Belgian Mobility" portal (`api-management-opendata-production.azure-api.net`) is
the national access point; registering for a `bmc-partner-key` is still wise in
case anonymous access gets closed.

Feed definitions come from Transitous (`feeds/be.json` in
public-transport/transitous) and the Transitland atlas
(`feeds/belgianmobility.io.dmfr.json`, `feeds/nabsa.github.com.dmfr.json`).

## Shared bikes (GBFS) — all reachable, no key

| Operator | GBFS URL | Notes |
|---|---|---|
| Blue-bike (all stations) | `https://api.delijn.be/gbfs/gbfs.json` | GBFS 2.3, has `geofencing_zones` |
| Velo Antwerpen | `https://gbfs.smartbike.com/antwerp/1.0/gbfs.json` | |
| Donkey Republic Antwerp | `https://stables.donkey.bike/api/public/gbfs/3.0/donkey_antwerp/gbfs.json` | |
| Donkey Republic Ghent | `https://stables.donkey.bike/api/public/gbfs/3.0/donkey_gh/gbfs.json` | |
| Donkey Kortrijk / Westhoek | `…/donkey_kortrijk_leiedal/gbfs`, `…/donkey_westhoek/gbfs.json` | Not in config yet |
| Dott Ghent / Brussels | `https://gbfs.api.ridedott.com/public/v2/{ghent,brussels}/gbfs.json` | |
| Bolt Brussels, Lime/Voi/Bird/Felyx Antwerp | see Transitland atlas | Not in config yet |
| Donkey Mechelen | — | **Not found** in either feed list (verify) |
| Hoppy Kempen (485 e-bikes, 95 zones, back-to-many) | — (generated, `deploy/motis/gbfs/hoppy-kempen`) | **No public GBFS** (not in MobilityData `systems.csv`, Transitous, transportdata.be). Built by `tools/hoppy_gbfs.py` from the Hoppinpunt dataset → 78 of 95 zones; availability assumed (ADR-011) |
| Hoppy elsewhere | same tool, `--region <VVR>` | Hoppinpunt data also tags Hoppy bikes in Vlaamse Ardennen (46), Dender (41), Midwest (28), Brugge (22), Westhoek (13), Limburg (10), Mechelen (6). Pricing/rules per region unchecked |

### Hoppinpunt dataset (DMOW)

Every Flemish mobility hub as a polygon, with `Deelsystemen_operatoren` and
`Deelsystemen_operator_aanbod` (operator + vehicle type). Two ways in:

- `https://hoppin.be/nl/vind-hoppinpunten/` embeds all 961 hubs as GeoJSON
  Features in the HTML (what `tools/hoppy_gbfs.py` scrapes; works from the cloud workspace).
- Official WMS/WFS `https://geoserver.gis.cloud.mow.vlaanderen.be/geoserver/hoppin/…`
  (listed on transportdata.be). **403** from the cloud workspace; untested from home.

## Streets

| Source | Use | Status |
|---|---|---|
| Geofabrik Belgium extract | Preferred OSM source | **Blocked** from the cloud workspace; fine from home |
| openstreetmap.fr Belgium extract | Fallback | Works, but 3,041 ways reference missing nodes → crashes osr with `osmium::invalid_location`. Cleaned with `tools/fix_osm_refs.py` |
| BRouter (optional) | Tunable bike profiles | Not set up |
| Citymapper web API | Benchmark only (ADR-007) | — |

## Our own data

| Dataset | Source | Why |
|---|---|---|
| GTFS-RT history | Logger polling NMBS + De Lijn RT | Delay distributions → reliability scoring. Start ASAP |
| Station transfer times | NMBS `transfers.txt` + hand-curated | Allow tighter transfers than the feed's 4–5 min where realistic |
| Folding-bike penalties | Config | Fold/unfold time, stairs, crowding |
