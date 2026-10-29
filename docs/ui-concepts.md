# UI concepts (brainstorm, 2026-10-29)

Problem: printed times are hard to judge. Lists of "20:24 → 23:21" hide what matters:
how long you wait, where the slack is, and what happens if a change fails.

Baseline: the previous app (transit.projectwhy.be) already draws journeys as a
calendar-style timeline (time = space, proportional blocks, "now" line, lanes).

Mockups (Design canvas "Overstap UI concepts"), all on the real Retie → Lokeren evening:

1. **Progress over time** (Marey / space-time chart): x = distance to destination,
   y = time. Slope is speed, vertical stretches are waiting, the right edge is arrival.
   Shows *where* time is lost (e.g. 16′ standing at Berchem).
2. **Plan B tree**: while travelling, the plan as a tree of decision points (changes) with
   slack per change and the fallback outcome of each failure (IC 21:58 → home 00:21).
   Last-mile choice as leaves (Donkey 22:58 / walk 23:21 / first bus 05:42).
3. **When to leave** (staircase): x = leave time, y = arrival. Plateaus = leaving later
   costs nothing; cliffs = a missed bus/train. "Any time before 20:24 is the same."
4. **Next move + buffers**: in-the-moment screen. Countdown to the next action, a
   proportional strip of the rest of the trip, slack bars per change (walk vs spare).
5. **Clock face**: options as rings on an analogue dial; arcs per leg. Uses the
   analogue sense of "how much of the evening" a trip takes.

Data the views need beyond MOTIS itineraries:
- per-change slack (arrival + walk vs departure) → from legs, done
- fallback per change (re-plan from the change stop at "missed" time) → overshoot.rs pattern
- leave-time sweep (many /plan calls over a window) → cheap, latency doesn't matter
- delay distributions → GTFS-RT logger (Phase 3)

## Round 2 (same canvas, three new pages)

**Plan & ride**
- A · Today's lane timeline (transit.projectwhy.be, portrait) vs B · space-time chart with a
  bottom sheet; tap an arrival to highlight that route. Verdict so far: space-time wins (user).
- Ride styles: (1) Next move + plan B per change, (2) you as a dot on the space-time line with
  grey ghost routes per change, (3) dark one-column "thread" to scale, (4) lock screen + watch:
  next action, spare time, arrival; buzz only when one of those changes.

**When plans break** (alternatives are real MOTIS results unless marked "example")
- Delay eats a buffer (11′ → 2′ at Berchem): go for it vs relax on IC 22:42 (23:22).
- Connection cancelled: cancelled line crossed out, replacement lines fan out in space-time.
- Missed my stop: space-time where the train carries you *away*; Olen + folding bike 09:12.
- Last train lost: night mode; "ride it out" is a 5 h 26 hatched block, bail-out + bike 01:19.
- Bike-share empty: stock sparkline (needs GBFS logging), Donkey hub vs walk.

**Around the trip**: home with "leave by" + 10-minute-slot sparkline per saved trip; gear and
tolerances (bike pass, speeds, own minimum change time, play-it-safe at night); post-trip
planned-vs-actual with one-tap delay reports for vehicles without live data.

New backend needs: GBFS stock logging; user-reported delays as training data.

## Multiple corridors on one space-time chart (round 3)

Problem: the x axis is one line of stations; via Mol and via Turnhout are different geographies.
Real example tonight: via Mol home 22:54, via Turnhout (bus 47 north, X41, IC) home 23:22.

- **A · x = straight-line distance to home** (recommended default). Works for any number of
  corridors; columns merge where corridors physically meet (Berchem ≈ Antwerpen-C., both
  35 km); corridor = colour halo; station names written on the line at waits.
  Side effect worth keeping: detours become visible (Mol is as far from home as Retie: the
  bus there buys speed, not distance).
- **B · small multiples**: one narrow panel per corridor, shared time axis, compare arrivals
  by height. Good for 2–3 corridors, breaks for routes that switch corridor mid-way.
- **C · x = % of route done + column highlight on tap**: every route 0→100 %, selected
  corridor's stations light up. Clean, but equal x no longer means the same place.

Edge cases for A: routes that move *away* from home (Mol) draw leftward segments; stations
at the same distance in different directions share a column (label them on the line, not in
the header). Alternative x if crow-fly misleads: network distance or "best-case time to home".
