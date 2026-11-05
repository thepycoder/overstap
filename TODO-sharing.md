# TODO — shared mobility coverage (Flanders)

Goal: every shared bike, e-bike, cargo bike, step (e-kick-scooter) and moped Victor could
meet in Flanders, plus Brussels as a transfer hub, ingested as well as its data allows.
Work through one entry at a time. Scope and priorities: ADR-012.

Compiled 2026-11-05 from MobilityData `systems.csv` (34 BE systems), Hoppin, Flemish
government press releases and the press. Feeds were probed the same day from home:
see `docs/results-2026-11-05.md`. "Registry" = listed in `systems.csv`; "probed" = fetched
2026-11-05; "✓ live" = fetched and imported into MOTIS. Counts from press releases age fast.

**Coverage** (`tools/hub_coverage.py`, rerun after each entry): **654 of 746** bike, cargo
and step offers at Hoppinpunten (88 %), up from 430. The 92 left are Mobit 44, dead
Bird/Lime step feeds 22, Hoppy steps 6, Donkey 7, Blue-bike 7, nextbike 4, other 2.
**The six feeds added on 2026-11-05 have not been imported into MOTIS yet.**

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

- [x] **A0 Coverage checker**: `tools/hub_coverage.py` compares each offer in the
  Hoppinpunt data with the configured feeds, within 250 m of the hub. The 15
  vervoerregio's are Antwerpen, Brugge, Dender, Gent, Kempen, Kortrijk, Leuven, Limburg,
  Mechelen, Midwest, Oostende, Vlaamse Ardennen, Vlaamse Rand, Waasland and Westhoek.
  Don't filter on `Status`: 685 of the 961 hubs are `Mobiliteitsknooppunt` and still
  carry real offers.
- [x] **A1 Donkey Republic, VR Antwerpen** (Lantis): ~1,500 e-bikes, ~250 hubs, 32
  municipalities (2022–23 figures). hub. `donkey_antwerp` ✓ live, hub-only fix in
  `lastmile.rs`. **L3**
- [x] **A2 Donkey Republic, VR Waasland**, *Lokeren's own region*: 169 bikes at
  53 locations (Mar 2024), via Lantis and Interwaas. hub. Confirmed inside
  `donkey_antwerp`: 25 of 29 hubs, and Lokeren Station has 11 stations within 500 m.
  Lokeren Melkerijstraat has no Donkey station within 1.7 km (hub data wrong or not
  rolled out yet). **L3**
- [x] **A3 Donkey Republic, VR Mechelen** (IGEMO): 360 bikes at 101 locations in 12
  municipalities (Jun 2024). hub. Confirmed inside `donkey_antwerp` (51 of 52 hubs);
  there is no separate feed. **L3**
- [ ] **A4 Blue-bike, VR Gent**: 1,250 bikes at 65 locations (Oct 2024).
- [ ] **A5 Blue-bike, VR Leuven**: 700 bikes at 62 locations in 31 municipalities (Sep 2025).
- [ ] **A6 Blue-bike, VR Limburg**: 248 bikes at 17 locations (Jun 2024).
  - A4–A6 all sit in the `blue-bike` feed (✓ live; 209 of 216 hub offers covered).
    **Check the return model:** the Leuven release says only the *non-electric* bikes
    are round-trip, so the regional e-bikes may be A→B. If so, the round-trip pricing in
    `lastmile.rs` is wrong for them. The feed can't tell us: it has one vehicle type
    (`bicycle`/`human`, `return_constraint: free_floating`) and no e-bikes, even though
    `lastmile.rs` says MOTIS reports `ROUNDTRIP_STATION`. Check what MOTIS makes of it.
    The Hoppinpunt data does separate them: Elektrisch at 91 hubs, Conventioneel at 205.
    Check the rules on blue-bike.be, then split by station using the Hoppinpunt
    propulsion.
- [x] **A7 Blue-bike, NMBS station network**: classic bikes, round. ✓ live, round-trip
  pricing. **L3**
- [~] **A8 Hoppy, VR Kempen**: 485 e-bikes at 95 zones (Jul 2025). hub. Static
  `tools/hoppy_gbfs.py`. **L1**
  - [ ] Ask Hoppy or IOK for a public GBFS feed with live availability
  - [ ] Add the 17 zones missing from the Hoppinpunt data (hand-curated list?)
- [~] **A9 Hoppy, VR Dender (Aalst) + Vlaamse Ardennen**: ~366 e-bikes in 22
  municipalities (Aug 2024). Operated as **one** area: you can rent in
  Sint-Lievens-Houtem and return in Erpe-Mere. Built as a single static system,
  `hoppy-dender-vlaamse-ardennen` (87 hubs). **L1**
- [~] **A10 Hoppy, VR Brugge + Midwest + Westhoek**: one joint tender, 575 bikes at
  65–66 hubs in 34 municipalities (Sep 2025). Built as three static systems: Brugge 22,
  Midwest 28 (classic bikes too), Westhoek 13. **L1**
  - [ ] Check whether you can return across the three regions. If so, merge them into one system.
- [ ] **A11 Mobit, VR Kortrijk / Leiedal**: 178 bikes at 20 locations (Jan 2024), on
  Leiedal's framework contract (13 municipalities). Mobit is the operator: the
  `donkey_kortrijk_leiedal` feed is empty (0 stations), and the Hoppinpunt data tags
  Mobit at 23 Kortrijk hubs. No feed known. Mobit is the biggest remaining gap (44 hubs,
  with B20 and Mechelen). Ask Mobit for GBFS, or go static via E1 once the return
  rules are known. **L0**
- [~] **A12 nextbike, VR Vlaamse Rand 2.0**: 68 stations, 237 bikes (64 e-bikes), GBFS
  2.3 with 33 geofence zones and no `return_constraint`. Added to the config as
  `nextbike-vlaamse-rand`, with the hub-only fix in `lastmile.rs`. Covers 62 of 66 hubs.
  The only pricing plan in the feed (`Rate0679`) reads "€0 under 24 h, then €20 per
  24 h". It looks like a subscriber plan; the 2025 launch price was €1/h. Check. **L2, MOTIS import untested**
- [x] **A13 Hoppy, Genk**: built as static `hoppy-limburg` (10 hubs). **L1**
  - The 6 Hoppy offers in Mechelen are **steps**, not bikes. See B15.

## B. City permits: free-floating and docked

### Antwerpen
- [x] **B1 Velo Antwerpen**: dock. ✓ live (GBFS 2.0). Pricing (day or year pass) not yet modelled. **L2**
- [ ] **B2 Bolt Antwerpen**: 1,000 steps (the e-bike fleet is unconfirmed). Neither the
  registry nor the Transitland atlas has a Belgian Bolt feed other than Brussels; ask Bolt. Bolt
  publishes one feed per city (`mds.bolt.eu/gbfs/3/<id>/gbfs`), but only Brussels (`336`)
  is in the registry. Find the Antwerp id in the Transitland atlas or Bolt's open-data
  page. free.
- [~] **B3 Voi, Antwerpen + Brussels**: the registry feed works without a licence. It is
  one feed for both cities: 3,205 vehicles (2,280 e-bikes, 948 steps), 5,756 virtual
  stations and geofencing. Added as `voi-antwerp-brussels`. Still to check: whether the
  virtual stations are mandatory drop zones; if so, add the hub-only fix. Also check
  the licence terms on transportdata.be. **L2, MOTIS import untested**
- [x] **B4 Lime Antwerpen**: the feed returns **404**. Gone from Antwerp, or moved;
  the Hoppinpunt data still lists 7 hubs. Dropped.
- [x] **B5 Bird Antwerpen**: the feed is live but has **0 vehicles**. Dead. Dropped.
- [x] **B6 felyx (Cooltra) Antwerpen**: the feed returns **410 Gone**. Dropped.
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
- [ ] **B12 Bolt Gent**: 400 e-bikes. No public feed (see B2). free.
- [ ] **B13 Baqme Gent**: cargo e-bikes, paused after the Babboe recall. Check whether it is back.
- [ ] **B14 Mobit Gent**: 200 bikes announced around 2022. Probably gone. Verify.

### Other centrumsteden
- [ ] **B15 Mechelen, Hoppy steps**: the city's step operator, at 6 Hoppinpunten. No
  feed known. free.
- [~] **B16 Kortrijk, Lime**: ~205 e-bikes, GBFS 2.2, free-floating. Added as
  `lime-kortrijk`. **L2, MOTIS import untested**
- [x] **B17 Kortrijk, Donkey Kortrijk & Leiedal**: the feed is empty (0 stations, 0
  vehicles). Dropped. See A11.
- [~] **B18 Westhoek coast, Donkey Westhoek** (De Panne to Nieuwpoort): 55 hubs, 110
  bikes. Added as `donkey-westhoek`; the hub-only fix applies through the `donkey` prefix.
  The pricing plans are tiered by duration. **L2, MOTIS import untested**
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

- [ ] **C1 Villo!** (JCDecaux, runs until Sep 2028): 360 stations, GBFS 3.0,
  `return_constraint: any_station`. Ready to add.
- [ ] **C2 Free-floating bikes under the transition regime**: as of 5 Nov, Dott has 4,569
  vehicles (2,784 of them steps) and Bolt `336` has 5,558 vehicles on 3,192 virtual
  stations (`return_constraint: station`, i.e. drop zones). Lime has only 8 vehicles.
  Voi is already in through B3. Check how MOTIS handles drop zones before adding these.
- [ ] **C3 Mopeds**: felyx/Cooltra Brussels is live (261 mopeds, GBFS 3.0, no stations).
  Pony returns **400**, because the key in the registry URL is dead. Blocked on E4
  (licence preference).
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
- [x] **E2** The coverage checker (A0): `tools/hub_coverage.py`.
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
- [ ] **E7** GBFS version support in MOTIS v2.11.4. 2.0 (Velo), 2.3 and 3.0 (Donkey) were
  imported on 2026-10-29. Still to confirm: 2.2 (Lime) and the larger 2.3 feeds (Voi has
  6,191 geofence zones). This also covers the import test the 2026-11-05 additions still need.
- [ ] **E8** Pricing per operator (unlock fee plus per-minute rate, passes) for the
  cost comparison.
