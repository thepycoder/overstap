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
