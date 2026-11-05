# TODO — shared mobility coverage (Flanders)

Goal: every shared bike, e-bike, cargo bike, step (e-kick-scooter) and moped Victor could
meet in Flanders, plus Brussels as a transfer hub, ingested as well as its data allows.
Work through one entry at a time. Scope and priorities: ADR-012.

Compiled 2026-11-05 from MobilityData `systems.csv` (34 BE systems), Hoppin, Flemish
government press releases and the press. **No feed was probed for this list:** this cloud
session could reach only GitHub, and the operator hosts and hoppin.be were blocked.
"Registry" = listed in `systems.csv` but not fetched; "✓ live" = fetched and imported on
2026-10-29. Counts and dates come from the cited releases and age fast.

**Level** of ingestion: `L0` absent · `L1` static (generated from Hoppinpunt hubs,
availability assumed, ADR-011) · `L2` live GBFS in MOTIS · `L3` L2 plus return
model, pricing, speed and an e2e query checked.

**Done when** (per entry): (1) confirm the service exists and its area (app or city page);
(2) fetch `gbfs.json` from home and note its version, feeds, vehicle count, form factors,
`return_constraint`, geofencing and `last_updated`; (3) add it to `deploy/motis/config.yml`
(and to `fetch-data.sh` if static); (4) confirm it imports and shows in `/rentals`;
(5) check the return model, pricing and speed against `lastmile.rs` and the provider
rules; (6) run one real query plus an e2e test, then record the result in
`docs/results-*.md` and `docs/data-sources.md`.

Return models: **dock** (fixed stations) · **hub** (back-to-many: any hub in the network, no dock) ·
**round** (back to the start) · **free** (free-floating inside a geofence).

---

## A. Regional Hoppin networks: the backbone

All 15 vervoerregio's tender shared bikes at Hoppinpunten (mostly stations and bus hubs),
which is exactly where overstap switches mode. These come first.

- [ ] **A0 Coverage checker**: `tools/hub_coverage.py` diffs `Deelsystemen_operatoren` per
  Hoppinpunt (DMOW dataset) against the configured systems and prints the hubs that are
  not covered, by vervoerregio and operator. It runs again after each entry and is the
  definition of "complete". It also settles the list of 15 vervoerregio's.
- [x] **A1 Donkey Republic, VR Antwerpen** (Lantis): ~1,500 e-bikes, ~250 hubs, 32
  municipalities (2022–23 figures). hub. `donkey_antwerp` ✓ live, hub-only fix in
  `lastmile.rs`. **L3**
- [ ] **A2 Donkey Republic, VR Waasland**, *Lokeren's own region*: 169 bikes at
  53 locations (Mar 2024), via Lantis and Interwaas. hub. Probably inside
  `donkey_antwerp` (same Lantis contract). Verify that the Lokeren, Zele and
  Sint-Niklaas hubs are in `station_information`. **L2?**
- [ ] **A3 Donkey Republic, VR Mechelen** (IGEMO): 360 bikes at 101 locations in 12
  municipalities (Jun 2024). hub. No `donkey_mechelen` in the registry, and the bikes
  reportedly show under `donkey_antwerp`. Confirm that by bounding box. **L2?**
- [ ] **A4 Blue-bike, VR Gent**: 1,250 bikes at 65 locations (Oct 2024).
- [ ] **A5 Blue-bike, VR Leuven**: 700 bikes at 62 locations in 31 municipalities (Sep 2025).
- [ ] **A6 Blue-bike, VR Limburg**: 248 bikes at 17 locations (Jun 2024).
  - A4–A6 all sit in the `blue-bike` feed (✓ live). **Check the return model:** the
    Leuven release says only the *non-electric* bikes are round-trip, so the regional
    e-bikes may be A→B. If so, the round-trip pricing in `lastmile.rs` is wrong for them.
    Split by vehicle type or station.
- [x] **A7 Blue-bike, NMBS station network**: classic bikes, round. ✓ live, round-trip
  pricing. **L3**
- [~] **A8 Hoppy, VR Kempen**: 485 e-bikes at 95 zones (Jul 2025). hub. Static
  `tools/hoppy_gbfs.py`. **L1**
  - [ ] Ask Hoppy or IOK for a public GBFS feed with live availability
  - [ ] Add the 17 zones missing from the Hoppinpunt data (hand-curated list?)
- [ ] **A9 Hoppy, VR Dender (Aalst) + Vlaamse Ardennen**: ~366 e-bikes in 22
  municipalities (Aug 2024). Operated as **one** area: you can rent in
  Sint-Lievens-Houtem and return in Erpe-Mere. The Hoppinpunt data tags 41 + 46 hubs.
  Build a single static system so cross-region returns work. **L0→L1**
- [ ] **A10 Hoppy, VR Brugge + Midwest + Westhoek**: one joint tender, 575 bikes at
  65–66 hubs in 34 municipalities (Sep 2025). Brugge alone has 195 at 20 hubs, Westhoek
  120 at 13. The Hoppinpunt data tags 22 / 28 / 13 hubs. Check whether one system or
  three. **L0→L1**
- [ ] **A11 Mobit, VR Kortrijk / Leiedal**: 178 bikes at 20 locations (Jan 2024), on
  Leiedal's framework contract (13 municipalities). No feed known. **Conflict:** the
  registry lists `donkey_kortrijk_leiedal`. Find out which one runs today. That may be
  both, or Donkey may be the older system. **L0**
- [ ] **A12 nextbike, VR Vlaamse Rand 2.0**: 68 locations around Brussels (Jun 2026).
  The 2025 launch had 185 classic bikes and 80 e-bikes. dock? Registry `nextbike_hf`
  (GBFS 2.3). Also covers the edge of Brussels. **L0→L2**
- [ ] **A13 Hoppy, smaller pockets**: Genk city has 50 e-bikes at 11+ locations (2024).
  The Hoppinpunt data also tags Hoppy at 10 hubs in Limburg and 6 in Mechelen. Static. **L0→L1**

## B. City permits: free-floating and docked

### Antwerpen
- [x] **B1 Velo Antwerpen**: dock. ✓ live (GBFS 2.0). Pricing (day or year pass) not yet modelled. **L2**
- [ ] **B2 Bolt Antwerpen**: 1,000 steps (the e-bike fleet is unconfirmed). Bolt
  publishes one feed per city (`mds.bolt.eu/gbfs/3/<id>/gbfs`), but only Brussels (`336`)
  is in the registry. Find the Antwerp id in the Transitland atlas or Bolt's open-data
  page. free.
- [ ] **B3 Voi Antwerpen**: steps under a permit for 1,000 that runs 9 Oct 2024 to 2029.
  The URL is in the registry, but transportdata.be says access needs a **licence by
  email**. Request one. free.
- [ ] **B4 Lime Antwerpen**: steps and/or e-bikes. Registry (GBFS 2.2). Status in 2026
  unconfirmed. free.
- [ ] **B5 Bird Antwerpen**: registry entry, but the permit lapsed on 8 Oct 2024.
  Probably dead. Verify, then drop.
- [ ] **B6 felyx (Cooltra) Antwerpen**: e-mopeds. Registry (GBFS 3.0, also on
  transportdata.be). Also holds a permit for up to 300 bikes (Jul 2024). free.
- [ ] **B7 GO Sharing Antwerpen**: e-mopeds and e-bikes, with a permit for up to 300
  bikes (Jul 2024). No feed known. free.
- [ ] **B8 Poppy, Antwerpen + Mechelen**: steps and mopeds (cars out of scope). No feed
  known. free.
- [ ] **B9 Cargo bikes, Antwerpen**: Cargoroo went bankrupt in Dec 2024 (drop it).
  Unimobility holds a cargo permit. Find out who operates now and whether they publish GBFS.

### Gent (the city allows no shared steps; Hoppy withdrew in May 2024)
- [x] **B10 Donkey Republic Gent**: 500 bikes. ✓ live. **L2**
- [x] **B11 Dott Gent**: 600 e-bikes and 45 cargo e-bikes. ✓ live. Still to check:
  form factors (exclude cargo by default?) and geofencing. **L2**
- [ ] **B12 Bolt Gent**: 400 e-bikes. Feed id unknown (see B2). free.
- [ ] **B13 Baqme Gent**: cargo e-bikes, paused after the Babboe recall. Check whether it is back.
- [ ] **B14 Mobit Gent**: 200 bikes announced around 2022. Probably gone. Verify.

### Other centrumsteden
- [ ] **B15 Mechelen, Hoppy steps**: the city's step operator. No feed known. free.
- [ ] **B16 Kortrijk, Lime**: registry (GBFS 2.2). free.
- [ ] **B17 Kortrijk, Donkey Kortrijk & Leiedal**: registry, not in config yet. See A11.
- [ ] **B18 Westhoek coast, Donkey Westhoek** (De Panne and around): registry, not in config yet.
- [ ] **B19 Ieper, Bolt steps**: 150 steps per old news. Verify.
- [ ] **B20 Hasselt, Mobit**: bikes at ~30 fixed spots. The city phased out steps in
  2023. No feed known. Static via the Hoppinpunt data if Mobit hubs are tagged there.
- [ ] **B21 Brugge, Hoppy steps**: mentioned on a tourist page only. Verify.
- [ ] **B22 Sweep the remaining centrumsteden** for free-floating permits: Leuven,
  Oostende, Aalst, Sint-Niklaas, Roeselare, Turnhout, Genk. None were found in this
  pass. Check the city mobility pages.

## C. Brussels (transfer hub; lower priority)

On 29 Jul 2026 the Raad van State annulled every free-floating permit, and vehicles had
to be off the street by 1 Sep 2026. Shared bikes then got a transition regime until
May 2028 (two permits of up to 4,800 bikes each). **Steps are banned from 1 Jan 2027.**

- [ ] **C1 Villo!** (JCDecaux, runs until Sep 2028): dock. Registry `bruxelles` (GBFS 3.0).
- [ ] **C2 Free-floating bikes under the transition regime**: Lime, Dott (`dott-brussels`
  in the registry), Bolt (`336`). Verify which ones came back after 1 Sep 2026. The
  current config has none of them.
- [ ] **C3 Mopeds**: felyx/Cooltra Brussels (registry) and Pony Brussels (registry, key in
  URL). Check whether the annulment affected them.
- **Not ingesting:** Brussels steps. They vanish within weeks.

## D. Out of scope (recorded so it isn't researched twice)

- Wallonia: Dott Charleroi/Liège/Mons, Pony Charleroi/Liège, Li Bia Velo (Namur).
- Car sharing: Getaround (9 BE cities in the registry), Cambio, Poppy cars, Dégage,
  Battmobility. Revisit if cars become a mode.
- Cargoroo (bankrupt, Dec 2024).

## E. Cross-cutting work (do once, used by many entries)

- [ ] **E1** Generalise `tools/hoppy_gbfs.py` into a `hub_gbfs.py --operator <regex> --region`
  that writes static GBFS for any feed-less operator: Hoppy elsewhere, Mobit, Hoppy Genk.
  This unblocks A9–A13, A11 and B20.
- [ ] **E2** The coverage checker (A0).
- [ ] **E3** Return model per system and per vehicle type (dock / hub / round / free),
  in the provider-rules config file (TODO.md, Phase 2) rather than constants in
  `lastmile.rs`. The Blue-bike split (A4–A6) is the first real case.
- [ ] **E4** Rider preferences per form factor: steps on or off, mopeds (licence),
  cargo off by default. Map them to MOTIS's rental filters (form factor, propulsion and
  provider per pre-, post- and direct leg; check the exact names in the OpenAPI spec).
- [ ] **E5** Speed per form factor. MOTIS routes every rental at bike-sharing speed
  (~15 km/h). Re-time `ELECTRIC_ASSIST` legs (≈20 km/h), steps (≈18), mopeds (25 or 45)
  and cargo bikes (slower) in overstap.
- [ ] **E6** Feed access and health. Track licences (Voi), keys (Pony) and User-Agent
  needs. `overstap health` should report vehicles per system and flag 0 vehicles or a
  stale `last_updated`.
- [ ] **E7** GBFS version support in MOTIS v2.11.4: confirm that 1.0, 2.0 (Velo), 2.2
  (Lime) and 3.0 (Donkey, Cooltra, nextbike) all import. Some operators serve several
  versions, so prefer 2.3 or 3.0.
- [ ] **E8** Pricing per operator (unlock fee plus per-minute rate, passes) for the
  cost comparison.
