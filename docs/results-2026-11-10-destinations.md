# Same address, different point: VITO Mol — 2026-11-10

Live instance, Lokeren Hillarestraat → VITO, depart 05:32 Tue, own bike with me.

| Destination | Folding | Shared | Foot |
|---|---|---|---|
| "Boeretang 200" address point 51.21902, 5.09354 | 5 routes via Mol (06:32 IC → 08:25) | Mol + Blue-bike tour 06:19 → 08:32 shown, Geel hidden | bus 47 + walk |
| "vito, mol" venue pin 51.21867, 5.09082 | **0 routes** | Geel only in the morning (08:17, 08:53); Mol tours only from 13:09 | afternoon only, 44' final walk |
| 100 m east of the pin, 51.2187, 5.0920 | 5 routes via Mol | — | — |

- Direct BIKE Mol station → pin works (22'), so MOTIS's street graph reaches it; the
  post-transit search doesn't.
- Shared tab: MOTIS's own one-way Blue-bike from Geel (`returnConstraint: NONE`) ends with a
  4' walk to the pin, so it still "works"; the tours (built on the walking plans) don't.
- Warning check replayed on live data (foot mode, control to the last stop): pin at 05:32 →
  warn (control to Dessel Sportcentrum reaches); address → no warning; pin and address at
  01:00 → no warning (control doesn't reach either: night).
- Blue-bike Mol Station: 32 bikes at 06:37. An evening search could find 0 and drop the tour
  for the next morning; fixed (ADR-021).
