# Spec: toolpath planning

Milestone: M4. Crate: `lasercraft-plan`. G-code export (M5) and streaming
(M6) consume the plan; the preview (`docs/spec/preview.md`) draws it.

## Contract

- `lasercraft_plan::plan(&Project, &Machine) -> Plan` is pure and
  deterministic: the same project and machine always give the same plan, bit
  for bit. No I/O, no clocks, no randomness, no hash-map iteration order.
- A `Plan` is the job as an ordered list of straight moves plus statistics
  and warnings. Every move has a kind, start and end point (document
  millimetres, `docs/spec/geometry.md`), feed speed (mm/s), power (%), the
  layer color, the pass number (1-based), the shape id it belongs to (none
  for travel) and its estimated duration.
- Move kinds:
  - `burn`: laser on, at the layer's speed and power.
  - `unlit`: laser off while moving along the toolpath at the layer's speed:
    overscan run-ons, gaps between fill segments on one scan line, and tab
    gaps. Keeping the feed speed keeps the burn even on either side.
  - `travel`: laser off, moving between cuts at the machine's top speed.
  M5 decides how each kind becomes G-code (`travel` as `G0`).
- `unlit` and `travel` moves always carry power 0.
- The head starts at the work area's origin corner (`work_area.origin`, in
  document coordinates). The plan does not add a final return move.
- Curves are flattened within 0.01 mm (own choice).

## Machine parameters

`Machine` describes what the planner needs from the device until device
profiles arrive (M8). Defaults are our own conservative guesses for a hobby
GRBL laser, not read from any device:

| Field | Meaning | Default | Valid |
| --- | --- | --- | --- |
| `max_speed` | Top speed of any move, mm/s; travel runs at it (GRBL `$110`/`$111`, which also set the `G0` rate) | 100 | 1–10 000 |
| `acceleration` | mm/s² (GRBL `$120`/`$121`) | 1000 | 1–100 000 |
| `max_power` | Device power ceiling, % of full scale | 100 | 0–100 |

Layer speeds above `max_speed` are kept in the plan (the controller caps
them itself) but timed at `max_speed`, with a warning per layer.

## Layers, passes and shapes

- Layers run in output order (`docs/spec/layers.md`); only layers with
  `output` on and at least one shape take part. `show` does not matter.
- Each layer runs `passes` times; pass *n* repeats pass 1's moves exactly
  (same order and start points), joined by a travel move.
- A shape contributes each of its outlines (subpaths), flattened, in
  document coordinates. Shapes are visited in drawing order before sorting.
- Power: every `burn` move carries the layer's `max_power`, clamped to
  0–100 and to the machine's `max_power`. Nothing in the plan can exceed
  either limit (`docs/spec/safety.md` rule 6). `min_power` only matters for
  grayscale images (M7) and is not used here.

## Line mode

- Every outline is traced: closed outlines as loops, open ones end to end.
- **Kerf offset** (`kerf`, mm): applied to closed outlines only. A positive
  value moves the path outward from the part so the finished part keeps its
  drawn size; a negative value moves it inward. An outline nested inside an
  odd number of other closed outlines on the same layer is a hole, and its
  offset goes the other way (a positive kerf makes the hole's path smaller,
  so the hole keeps its drawn size). Offsets use round corners. An outline
  that an inward offset removes completely is not cut and gets a warning.
  Open outlines are cut unchanged, with one warning per layer.
- **Tabs** (`tabs` per shape, `tab_width` mm): on closed outlines that are
  not inside another closed outline on the same layer (holes fall out as
  scrap anyway), `tabs` gaps of `tab_width` are left uncut, centered at
  arc lengths (i + ½) · P / `tabs` from the outline's first point, where P is
  the perimeter after kerf. The head keeps moving through a gap with the
  laser off (`unlit`). If the gaps would take half the perimeter or more,
  the outline is cut without tabs and a warning says so. Tabs apply to every
  pass. `tabs` 0 turns them off.
- **Cut order** within a layer:
  1. Containment: outline A is inside closed outline B when A's bounding
     box lies within B's and is strictly smaller in area, and up to 16
     evenly spaced vertices of A are all inside B (even-odd). The strict
     area rule makes the relation acyclic, so duplicates cannot deadlock.
  2. An outline is ready when every outline inside it has been cut ("inner
     shapes first", so a part does not drop before its holes are cut).
  3. Greedy nearest neighbour: from the current head position, cut the
     ready outline whose entry point is nearest. A closed outline may be
     entered at any of its vertices (it then runs once round from there in
     its own direction); an open one at either end (reversed if entered at
     its last point). Ties go to the earlier outline in drawing order, then
     the earlier vertex.
  This is O(n²) in the number of outlines, which is fine for the thousands
  of outlines hobby jobs have.

## Fill mode

- All closed outlines on the layer are filled together with the even-odd
  rule: nested outlines leave rings, overlapping outlines on one layer leave
  their overlap unfilled (as described in the fill-mode doc). Open outlines
  are skipped with one warning per layer. Kerf and tabs do not apply.
- **Scan geometry**: with scan angle θ (degrees, positive clockwise on
  screen), lines run along u = (cos θ, sin θ) and successive lines step
  along s = (sin θ, −cos θ). So 0° scans horizontal lines working from the
  bottom of the design up, 90° scans vertical lines working left to right,
  and 180° works top down.
- Lines sit at s = s_min + interval · (k + ½) for k = 0, 1, … while below
  s_max, where s_min/s_max bound the outlines along s. Each line is cut
  against every edge (an edge counts when its end points lie on opposite
  sides, half-open, so vertices on a line are counted once); crossings are
  sorted along u and paired even-odd into burn segments.
- **Direction**: bidirectional fills alternate direction on each line that
  has segments; one-way fills burn every line along +u and travel back unlit
  to the next line's start.
- Within one line, the gap between two segments is an `unlit` move.
- **Overscan**: when on, each line gets an `unlit` lead-in before its first
  segment and lead-out after its last, of length d = v² / (2a), where v is
  the layer speed (capped at `max_speed`) and a the machine acceleration:
  exactly the distance needed to reach v from rest, so the laser only fires
  at full speed. (LightBurn's docs describe the distance as a percentage of
  speed; we derive it from acceleration instead, which needs no extra
  setting.)
- **Crosshatch**: after the lines at θ, the same fill runs again at θ + 90°
  within the same pass.

## Statistics

- Lengths: `burn`, `unlit` and `travel` distance, each in mm.
- Bounds: the box around every move end point except the starting head
  position, so overscan and kerf are included. Empty for a job with no
  moves.
- **Time estimate** (own design, informed by GRBL's acceleration and max
  rate settings): moves run back to back with a trapezoidal speed profile at
  the machine's acceleration. Each move's target speed is its feed speed
  (travel: `max_speed`), capped at `max_speed`. The speed through the
  junction between two moves is at most min(v₁, v₂) · max(0, cos φ), where φ
  is the turn angle (full speed straight on, a stop at 90° or more). A
  backward then a forward pass limit entry and exit speeds to what
  acceleration allows over each move's length; the job starts and ends at
  rest. A move of length L from v₀ to v₁ with cruise speed v takes
  (v − v₀)/a + (v − v₁)/a + (L − d₀ − d₁)/v when the ramps d = (v² − vᵢ²)/2a
  fit, and otherwise peaks at v_p = √((2aL + v₀² + v₁²)/2) and takes
  (v_p − v₀)/a + (v_p − v₁)/a. The estimate ignores controller buffering
  and serial throughput.

## Safety checks

- `Plan::check_bounds(&WorkArea)` checks every move end point (overscan and
  kerf included) against the bed rectangle (0, 0)–(width, height) in
  document coordinates. The point furthest out wins (on a tie, a burn or
  unlit move over the travel move leading to it, then the earliest), and
  the error names the edge (left, right, top or bottom, i.e. axis x or y), the
  distance outside in mm, the shape id and its layer, and says whether the
  point is an overscan or tab run-on. M5 and M6 must call it and refuse the
  whole job on error (`docs/spec/safety.md` rule 5). The job-origin offset
  of M6 is applied before checking, there.
- Travel and unlit moves always have power 0; burn power is clamped (above).

## Warnings

Warnings carry the layer and, where it applies, the shape: open shapes on a
fill layer, open shapes with a kerf, outlines removed by kerf, tabs that do
not fit, a layer speed above `max_speed`, a fill layer with nothing to fill.

## Text dump

`Plan::dump()` writes the plan as text for golden tests and
`lasercraft-cli preview --moves`: a header, one line per layer, one line per
move (kind, from, to, layer, pass, shape, speed, power), then the
statistics. Numbers have at most three decimals.

## Sources

- Cut inner shapes first, reduce travel by proximity, choosing the best
  start point; layer order as the first ordering key:
  https://docs.lightburnsoftware.com/2.1/Reference/OptimizationSettings/
- Kerf offset on closed shapes, positive outward and negative inward, the
  direction inverted for shapes inside other shapes on the same layer:
  https://docs.lightburnsoftware.com/2.1/Reference/CutSettingsEditor/LineMode/,
  https://docs.lightburnsoftware.com/2.1/Guides/Test-KerfOffset/
- Tabs as skipped sections of a cut, a count per shape spaced evenly with a
  tab size, skipping inner shapes, closed shapes only:
  https://docs.lightburnsoftware.com/2.1/Reference/AddTabs/
- Fill: even-odd interaction of shapes on one layer, line interval,
  bidirectional vs one-way with an unlit return, crosshatch at 90°, scan
  angle direction (0° horizontal working bottom to top, 90° left to right,
  180° top down), overscan as laser-off run-on to reach speed:
  https://docs.lightburnsoftware.com/2.1/Reference/CutSettingsEditor/FillMode/,
  https://docs.lightburnsoftware.com/2.1/Explainers/Overscanning/
- Preview time estimate depends on the machine's speed and acceleration
  settings:
  https://docs.lightburnsoftware.com/2.1/Troubleshooting/PreviewWindow/InaccurateTimeEstimates/
- GRBL max rate (`$110`, also the `G0` rate) and acceleration (`$120`):
  https://github.com/gnea/grbl/wiki/Grbl-v1.1-Configuration
- All read 2026-10-21. Own design: the move kinds, the greedy algorithm and
  containment test, tab placement formula, the overscan formula, the
  junction and trapezoid time model, machine defaults, the dump format.
