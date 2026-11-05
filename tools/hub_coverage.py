"""Which shared-mobility offers at Flemish Hoppinpunten does our GBFS config cover?

The Hoppinpunt dataset (DMOW) names, for every hub, the operators and vehicle types
present ("Deelsystemen_operator_aanbod"). For each bike/cargo-bike/step offer this
tool asks: does a feed we configure for that operator have a station (or a parked
free-floating vehicle) within --radius metres of the hub? The answer per operator and
vervoerregio is the coverage measure from TODO-sharing.md (A0, ADR-012).

Feeds come from `deploy/motis/config.yml` (gbfs.feeds) plus any `--feed name=url`
candidates, which are reported separately so a candidate's gain is visible before it
is added. Local-directory feeds (generated, ADR-011) resolve against the config's dir.

Usage:
  python3 tools/hub_coverage.py                       # live hubs + live feeds
  python3 tools/hub_coverage.py --geojson hubs.geojson --missing Waasland
  python3 tools/hub_coverage.py --feed donkey-kortrijk=https://.../gbfs
"""
import argparse
import collections
import json
import math
import re
import sys
import urllib.parse
import urllib.request
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from hoppy_gbfs import FINDER_URL, fetch_features  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
VEHICLE_TYPES = ("Deelfietsen", "Deelbakfietsen", "Deelsteps")

# Hoppinpunt operator name → prefix of our feed names (config.yml keys).
OPERATOR_FEEDS = {
    "Blue-bike": "blue-bike",
    "Donkey Republic": "donkey",
    "Hoppy": "hoppy",
    "Velo": "velo",
    "Nextbike": "nextbike",
    "Mobit": "mobit",
    "Lime": "lime",
    "Bird": "bird",
    "Villo!": "villo",
    "Dott": "dott",
    "Bolt": "bolt",
    "Mobilize Share": "mobilize",
}


def offers(props: dict) -> list[tuple[str, str]]:
    """'Op - url - logo - Deelfietsen - VOM - Elektrisch, …' → [(op, 'Deelfietsen VOM Elektrisch')]."""
    out = []
    for part in re.split(r",\s*(?=[^,]+ - (?:https?://|www\.))", props.get("Deelsystemen_operator_aanbod") or ""):
        fields = [x.strip() for x in part.split(" - ")]
        if len(fields) >= 4 and fields[3] in VEHICLE_TYPES:
            out.append((fields[0], " ".join(fields[3:])))
    return out


def read_feeds_from_config(path: Path) -> dict[str, str]:
    """Minimal reader for the `gbfs: feeds: <name>: url: <url>` block (no PyYAML needed)."""
    feeds, in_gbfs, name = {}, False, None
    for line in path.read_text().splitlines():
        if re.match(r"^\S", line):
            in_gbfs = line.startswith("gbfs:")
            continue
        if not in_gbfs or line.strip().startswith("#"):
            continue
        if m := re.match(r"^    ([\w.-]+):\s*(#.*)?$", line):
            name = m.group(1)
        elif (m := re.match(r"^      url:\s*(\S+)", line)) and name:
            feeds[name] = m.group(1)
    return feeds


def load(url: str, base: Path):
    if re.match(r"https?://", url):
        req = urllib.request.Request(url, headers={"User-Agent": "overstap/0.1"})
        return json.load(urllib.request.urlopen(req, timeout=30))
    return json.loads((base / url).read_text())


def feed_points(url: str, base: Path) -> tuple[list[tuple[float, float]], str]:
    """Station and free-floating vehicle coordinates of one GBFS system."""
    if not re.match(r"https?://", url):
        base, url = base / url, "gbfs.json"
    disc = load(url, base)
    data = disc.get("data", {})
    fl = data.get("feeds") or next((v["feeds"] for v in data.values() if isinstance(v, dict) and "feeds" in v), [])
    urls = {f["name"]: f["url"] for f in fl}

    def sub(name):
        u = urls[name]
        if not re.match(r"https?://", u) and re.match(r"https?://", url):
            u = urllib.parse.urljoin(url, u)
        return load(u, base)["data"]

    pts, info = [], []
    if "station_information" in urls:
        st = sub("station_information")["stations"]
        pts += [(s["lat"], s["lon"]) for s in st]
        info.append(f"{len(st)} stations")
    for name in ("vehicle_status", "free_bike_status"):
        if name in urls:
            d = sub(name)
            v = [x for x in d.get("vehicles") or d.get("bikes") or [] if "lat" in x]
            pts += [(x["lat"], x["lon"]) for x in v]
            info.append(f"{len(v)} free-floating")
            break
    return pts, f"v{disc.get('version', '?')}, " + ", ".join(info)


def metres(a: tuple[float, float], b: tuple[float, float]) -> float:
    dy = (a[0] - b[0]) * 111_320
    dx = (a[1] - b[1]) * 111_320 * math.cos(math.radians(a[0]))
    return math.hypot(dx, dy)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--geojson", type=Path, help="Hoppinpunt features from a file instead of hoppin.be")
    ap.add_argument("--config", type=Path, default=ROOT / "deploy/motis/config.yml")
    ap.add_argument("--feed", action="append", default=[], metavar="NAME=URL", help="candidate feed (repeatable)")
    ap.add_argument("--radius", type=float, default=250, help="hub ↔ station match radius in metres")
    ap.add_argument("--status", action="append",
                    help="only hubs with this Status (repeatable). Default: all; 'Mobiliteitsknooppunt' "
                         "hubs (≈70 %%) carry real offers too, e.g. most Hoppy Kempen zones")
    ap.add_argument("--missing", nargs="*", metavar="VVR", help="list uncovered hubs (optionally only these VVRs)")
    a = ap.parse_args()

    feats = json.loads(a.geojson.read_text())["features"] if a.geojson else fetch_features(FINDER_URL)
    if a.status:
        feats = [f for f in feats if f["properties"].get("Status") in a.status]

    configured = read_feeds_from_config(a.config)
    candidates = dict(x.split("=", 1) for x in a.feed)
    points: dict[str, list] = {}
    print("## Feeds")
    for name, url in {**configured, **candidates}.items():
        tag = "candidate" if name in candidates else "configured"
        try:
            points[name], info = feed_points(url, a.config.parent)
            print(f"- {name} ({tag}): {info}")
        except Exception as e:  # report and carry on: one dead feed must not hide the rest
            print(f"- {name} ({tag}): **ERROR** {type(e).__name__}: {str(e)[:100]}")

    def covered_by(op: str, hub: tuple[float, float], names) -> bool:
        prefix = OPERATOR_FEEDS.get(op)
        return bool(prefix) and any(
            n.startswith(prefix) and any(metres(hub, p) <= a.radius for p in points.get(n, ()))
            for n in names
        )

    # (operator, vvr) → [hubs, covered by config, covered incl. candidates]
    table = collections.defaultdict(lambda: [0, 0, 0])
    missing = collections.defaultdict(list)
    for f in feats:
        p = f["properties"]
        hub = tuple(float(x) for x in p["Centroide_coordinaten"].split())
        for op in sorted({op for op, _ in offers(p)}):
            row = table[(op, p.get("VVR") or "?")]
            row[0] += 1
            if covered_by(op, hub, configured):
                row[1] += 1
            elif covered_by(op, hub, points):
                row[2] += 1
            else:
                missing[(op, p.get("VVR") or "?")].append(f"{p['Naam']} ({p.get('Gemeente')})")

    print(f"\n## Coverage of {len(feats)} hubs (radius {a.radius:.0f} m)\n")
    print("| Operator | Vervoerregio | Hubs | Covered | +candidates | Missing |")
    print("|---|---|---:|---:|---:|---:|")
    tot = [0, 0, 0]
    for (op, vvr), (n, c, cc) in sorted(table.items()):
        print(f"| {op} | {vvr} | {n} | {c} | {cc} | {n - c - cc} |")
        tot = [tot[0] + n, tot[1] + c, tot[2] + cc]
    print(f"| **all** | | **{tot[0]}** | **{tot[1]}** | **{tot[2]}** | **{tot[0] - tot[1] - tot[2]}** |")

    if a.missing is not None:
        print("\n## Uncovered hubs")
        for (op, vvr), hubs in sorted(missing.items()):
            if not a.missing or vvr in a.missing:
                print(f"- {op} / {vvr} ({len(hubs)}): " + "; ".join(sorted(hubs)))


if __name__ == "__main__":
    main()
