"""Build a static GBFS 2.3 feed for Hoppy e-bikes from the official Hoppinpunt data.

Hoppy publishes no GBFS. The Flemish government's Hoppinpunt dataset (DMOW) lists,
per mobility hub, which shared-mobility operators are present and the hub's
polygon. Every hub tagged "Hoppy ... Deelfietsen" in the given vervoerregio('s)
becomes a virtual station; bikes can be returned at any of them (`any_station`).
Pass several `--region`s for one network that allows returns across regions
(Dender + Vlaamse Ardennen). Hubs offering classic bikes get a second vehicle type.

Limits (see docs/decisions.md ADR-011):
* No live availability: every hub reports `--assume-bikes` bikes, always open.
* Coverage: only hubs in the Hoppinpunt dataset (78 of Hoppy's 95 Kempen zones
  as of 2026-10-29). Hubs outside it are missing.

Usage:
  python3 tools/hoppy_gbfs.py --region Kempen --out deploy/motis/gbfs/hoppy-kempen
  python3 tools/hoppy_gbfs.py --region Dender --region "Vlaamse Ardennen" --out ...
  python3 tools/hoppy_gbfs.py --geojson hoppinpunten.geojson ...   # offline input
"""
import argparse
import json
import re
import sys
import time
import urllib.request
from pathlib import Path

FINDER_URL = "https://hoppin.be/nl/vind-hoppinpunten/"
HOPPY_BIKES = re.compile(r"Hoppy - [^,]*Deelfietsen - [^,]*?(Elektrisch|Conventioneel)")
VEHICLE_TYPES = {  # Hoppinpunt propulsion → GBFS vehicle type
    "Elektrisch": {"vehicle_type_id": "hoppy-ebike", "propulsion_type": "electric_assist",
                   "max_range_meters": 50000, "name": "Hoppy e-bike"},  # range unknown; swappable batteries
    "Conventioneel": {"vehicle_type_id": "hoppy-bike", "propulsion_type": "human", "name": "Hoppy bike"},
}


def fetch_features(url: str) -> list[dict]:
    """The finder page embeds every hub as a GeoJSON Feature in its HTML."""
    req = urllib.request.Request(url, headers={"User-Agent": "overstap/0.1"})
    html = urllib.request.urlopen(req, timeout=60).read().decode("utf-8")
    dec = json.JSONDecoder()
    feats = {}
    for m in re.finditer(r'\{"type":"Feature"', html):
        try:
            f, _ = dec.raw_decode(html, m.start())
        except json.JSONDecodeError:
            continue
        feats[f.get("id")] = f
    if not feats:
        sys.exit(f"no Hoppinpunt features found in {url} (page layout changed?)")
    return list(feats.values())


def hoppy_bike_kinds(f: dict) -> set[str]:
    """{'Elektrisch', 'Conventioneel'} subset offered by Hoppy at this hub."""
    return set(HOPPY_BIKES.findall(f["properties"].get("Deelsystemen_operator_aanbod") or ""))


def multipolygon(geom: dict) -> dict | None:
    if geom["type"] == "MultiPolygon":
        return geom
    if geom["type"] == "Polygon":
        return {"type": "MultiPolygon", "coordinates": [geom["coordinates"]]}
    return None


def build(features: list[dict], regions: list[str], assume_bikes: int) -> dict[str, dict]:
    now = int(time.time())
    label = " + ".join(regions)
    slug = re.sub(r"[^a-z0-9]+", "-", "-".join(regions).lower()).strip("-")
    system_id = f"hoppy-{slug}"
    hubs = sorted(
        (f for f in features if f["properties"].get("VVR") in regions and hoppy_bike_kinds(f)),
        key=lambda f: f["properties"]["Naam"],
    )
    if not hubs:
        sys.exit(f"no Hoppy bike hubs found for vervoerregio('s) {label}")
    kinds_used = set().union(*(hoppy_bike_kinds(f) for f in hubs))

    stations, statuses = [], []
    for f in hubs:
        p = f["properties"]
        lat, lon = (float(x) for x in p["Centroide_coordinaten"].split())
        sid = f"hoppinpunt-{p['ID']}"
        st = {
            "station_id": sid,
            "name": p["Naam"],
            "lat": round(lat, 6),
            "lon": round(lon, 6),
            "is_virtual_station": True,
        }
        if p.get("Adres"):
            st["address"] = p["Adres"]
        if area := multipolygon(f["geometry"]):
            st["station_area"] = area
        stations.append(st)
        kinds = sorted(hoppy_bike_kinds(f))
        statuses.append({
            "station_id": sid,
            "num_bikes_available": assume_bikes * len(kinds),
            "vehicle_types_available": [
                {"vehicle_type_id": VEHICLE_TYPES[k]["vehicle_type_id"], "count": assume_bikes} for k in kinds
            ],
            "is_installed": True,
            "is_renting": True,
            "is_returning": True,
            "last_reported": now,
        })

    def wrap(data: dict) -> dict:
        return {"last_updated": now, "ttl": 86400, "version": "2.3", "data": data}

    names = ["system_information", "vehicle_types", "station_information", "station_status"]
    return {
        "gbfs": wrap({"nl": {"feeds": [{"name": n, "url": f"{n}.json"} for n in names]}}),
        "system_information": wrap({
            "system_id": system_id,
            "language": "nl",
            "name": f"Hoppy {label}",
            "operator": "Hoppy",
            "url": "https://behoppy.eu/",
            "timezone": "Europe/Brussels",
        }),
        "vehicle_types": wrap({"vehicle_types": [
            {**VEHICLE_TYPES[k], "form_factor": "bicycle", "return_constraint": "any_station"}
            for k in sorted(kinds_used)
        ]}),
        "station_information": wrap({"stations": stations}),
        "station_status": wrap({"stations": statuses}),
    }


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--region", action="append",
                    help="vervoerregio (VVR), e.g. Kempen; repeat for one cross-region network")
    ap.add_argument("--out", type=Path, required=True, help="output directory")
    ap.add_argument("--geojson", type=Path, help="read Hoppinpunt features from a file instead")
    ap.add_argument("--assume-bikes", type=int, default=1,
                    help="bikes reported per hub (no live data); 0 disables renting")
    a = ap.parse_args()

    if a.geojson:
        features = json.loads(a.geojson.read_text())["features"]
    else:
        features = fetch_features(FINDER_URL)
    regions = a.region or ["Kempen"]
    files = build(features, regions, a.assume_bikes)
    a.out.mkdir(parents=True, exist_ok=True)
    for name, doc in files.items():
        (a.out / f"{name}.json").write_text(json.dumps(doc, ensure_ascii=False, indent=1))
    n = len(files["station_information"]["data"]["stations"])
    print(f"{n} Hoppy hubs in {' + '.join(regions)} → {a.out}")


if __name__ == "__main__":
    main()
