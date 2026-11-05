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

Full candidate list for Flanders + Brussels (bikes, steps, mopeds), with status per
operator and region: `TODO-sharing.md`. The table below holds what is verified.

| Operator | GBFS URL | Notes |
|---|---|---|
| Blue-bike (all stations) | `https://api.delijn.be/gbfs/gbfs.json` | GBFS 2.3, 331 stations, `geofencing_zones`. One vehicle type (`human`, `return_constraint: free_floating`): it doesn't separate e-bikes or encode the round-trip rule |
| Velo Antwerpen | `https://gbfs.smartbike.com/antwerp/1.0/gbfs.json` | GBFS 2.0, 323 stations |
| Donkey Republic Antwerp | `https://stables.donkey.bike/api/public/gbfs/3.0/donkey_antwerp/gbfs.json` | Also covers **VR Waasland (Lokeren) and VR Mechelen**: 866 hubs, ~2,000 bikes |
| Donkey Republic Ghent | `https://stables.donkey.bike/api/public/gbfs/3.0/donkey_gh/gbfs.json` | |
| Donkey Westhoek | `…/donkey_westhoek/gbfs.json` | Added 2026-11-05. 55 hubs on the coast |
| Donkey Kortrijk & Leiedal | `…/donkey_kortrijk_leiedal/gbfs` | **Empty** (2026-11-05); Mobit runs Kortrijk |
| Dott Ghent / Brussels | `https://gbfs.api.ridedott.com/public/v2/{ghent,brussels}/gbfs.json` | Ghent configured. No other Flemish Dott city exists (404) |
| nextbike Vlaamse Rand | `https://gbfs.nextbike.net/maps/gbfs/v2/nextbike_hf/gbfs.json` | Added 2026-11-05. 68 stations; hub-only fix |
| Lime Kortrijk | `https://data.lime.bike/api/partners/v2/gbfs/kortrijk/gbfs.json` | Added 2026-11-05. ~205 e-bikes. Lime Antwerp is 404 |
| Voi Antwerp + Brussels | `https://api.voiapp.io/gbfs/be/7a4cb689-a2ee-409d-a5fe-49d2e8d50717/v2/gbfs.json` | Added 2026-11-05. One feed, e-bikes + steps; worked without a licence |
| Villo, Bolt Brussels, Cooltra Brussels | see `docs/results-2026-11-05.md` | Live, not configured yet (TODO-sharing C) |
| Hoppy (Kempen, Dender + Vlaamse Ardennen, Brugge, Midwest, Westhoek, Genk) | — (generated, `deploy/motis/gbfs/hoppy-*`) | **No public GBFS.** Built by `tools/hoppy_gbfs.py` from the Hoppinpunt dataset, 238 hubs in all; availability assumed (ADR-011). Mechelen Hoppy is steps only |
| Mobit (Kortrijk, Hasselt, Mechelen) | — | No feed known; 44 Hoppinpunten |

Probe of every candidate, and coverage per operator and region:
`docs/results-2026-11-05.md`. Coverage check: `tools/hub_coverage.py`.

### Hoppinpunt dataset (DMOW)

Every Flemish mobility hub as a polygon, with `Deelsystemen_operatoren` and
`Deelsystemen_operator_aanbod` (operator + vehicle type). Two ways in:

- `https://hoppin.be/nl/vind-hoppinpunten/` embeds all 961 hubs as GeoJSON
  Features in the HTML (what `tools/hoppy_gbfs.py` and `tools/hub_coverage.py` scrape).
  Reachable from the cloud workspace on 2026-10-29 but blocked on 2026-11-05;
  reachable from home. Fields include `VVR` (15 regions), `Status` (`Actief` or
  `Mobiliteitsknooppunt`; both carry offers) and `Deelsystemen_operator_aanbod`
  (operator, vehicle type, VOM/niet-VOM, propulsion). Some offers are stale (Bird, Lime Antwerp).
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
