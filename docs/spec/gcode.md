# Spec: GRBL G-code export

Milestone: M5. The planner's moves (`docs/spec/planning.md`) become G-code
for GRBL 1.1 in laser mode. The device connection (M6) streams this text.

## Before anything is written

1. The job is planned with the machine parameters (`docs/spec/planning.md`).
2. The plan is checked against the work area (`Plan::check_bounds`, overscan
   and kerf included). A job that leaves the bed is refused with the planner's
   error naming the edge, distance, move and shape. Nothing is written.
3. A job with no burn moves (no output layers, or empty ones) is refused:
   "nothing to export".
4. The generated text is checked again by an independent verifier (below)
   before it is returned or written. A violation refuses the export; it means
   a bug in the generator, so the message says so.

## Output

- Plain ASCII, one block per line, `\n` line ends, every line at most 70
  characters.
- A comment header in parentheses names LaserCraft, the job name (characters
  other than letters, digits, spaces and `-_.,:` are replaced by `_`, and the
  name is shortened to fit), the bounds and the time estimate.
- `G90 G21` comes before any motion: absolute coordinates, millimetres. Both
  are GRBL's power-on defaults; we state them anyway.
- `M5 S0`: laser off before the first move.
- Travel moves are `G0 X… Y…`. GRBL never fires the laser on `G0`.
- Burn moves are `G1 X… Y…` with `S` and `F` written only when they change.
- Unlit moves (overscan, fill gaps, tab gaps) are `G1 X… Y… S0`: the head
  keeps the layer's feed so overscan does its job, with the beam off.
- Before a layer's first move, the laser mode is set when it changes: `M4`
  (dynamic power, the default) or `M3` for layers with `constant_power` on,
  always together with `S0`. `M3`/`M4` changes make GRBL stop, so they are
  only written between layers.
- Air assist: `M8` (or `M7`, an export option) before the first move of a
  layer with `air` on, `M9` before the first move of a layer with it off.
- The end: `M5 S0`, then `M9` if air assist was used, then, if the finish
  option is on, `G0 X0 Y0` (back to the origin). Nothing follows.
- Numbers: coordinates to 3 decimals, `F` to 1 decimal, `S` to 2 decimals,
  trailing zeros and points dropped, never `-0`.

## Units and values

- `F` is the move's speed in mm/min (layer speed × 60).
- `S` is `power / 100 × s_max`, where power is the planner's burn power,
  already clamped to the layer's and the machine's maximum. `s_max` must equal
  GRBL's `$30` (default 1000; older GRBL used 255); a mismatch scales the
  power wrongly, so the export option says so.
- Unlit and travel moves always have `S0` or no `S`.

## Coordinates

- Machine coordinates are distances from the work area's origin corner and
  are never negative: with the origin front-left, X grows to the right and Y
  grows towards the rear; with other corners, X and Y grow away from that
  corner. Only absolute coordinates are written (the "current position" and
  "user origin" job modes are M6).

## Options

| Option | Default | Valid |
| --- | --- | --- |
| `s_max` | 1000 | 1–100000 |
| `air` | `m8` | `m7`, `m8` |
| `finish` | `off` | `on` (end with `G0 X0 Y0`), `off` |
| `max_speed`, `accel`, `max_power` | the planner's machine defaults | as in planning |

## Verifier

`lasercraft_gcode::verify` reads G-code and replays GRBL laser-mode rules,
independently of the generator. It accepts only the words this exporter
writes (`G0 G1 G21 G90 M3 M4 M5 M7 M8 M9 X Y S F` and parenthesised comments)
and fails on:

- any other word, a malformed number, a line over 70 characters, or a
  comment that is not closed;
- motion before both `G21` and `G90`, or a `G1` before an `F`;
- an `S` above the allowed ceiling (`s_max × max_power / 100`) or below 0;
- a position outside the work area (machine coordinates, 0.001 mm slack);
- a file that does not end with the laser off (`M5`).

It reports the burn length (G1 with `M3`/`M4` active and `S` > 0), the unlit
and travel lengths, the highest `S`, the line count and the bounds. Tests
compare these with the plan. M6's simulated GRBL device builds on it.

## Commands and front ends

- `project.export_gcode <path> [s_max=<n>] [air=m7|m8] [finish=on|off]
  [max_speed=<mm/s>] [accel=<mm/s²>] [max_power=<%>]`: writes the file (not
  undoable; it does not change the project).
- `lasercraft-cli export <project.lcproj> <out.gcode> [--s-max n] [--air
  m7|m8] [--finish] [--max-speed v] [--accel a] [--max-power p]`; prints the
  verifier's summary.
- In the app, Alt+Shift+L (or File → Save G-code…) asks for a path.

## Sources

- GRBL v1.1 laser mode (`$32`, `M3`/`M4`, `G0` never fires, `S0`, stops on
  `M3`/`M4`/`M5` changes): https://github.com/gnea/grbl/wiki/Grbl-v1.1-Laser-Mode
- GRBL v1.1 commands (power-on modal defaults `G0 G54 G17 G90 G94 G21 M5
  M9`, coolant group `M7`/`M8`/`M9`): https://github.com/gnea/grbl/wiki/Grbl-v1.1-Commands
- GRBL v1.1 configuration (`$30`, `$31`): https://github.com/gnea/grbl/wiki/Grbl-v1.1-Configuration
- S-value max must match `$30`, defaults 1000/255, wrong values scale power:
  https://docs.lightburnsoftware.com/2.1/Reference/DeviceSettings/BasicSettings/,
  https://docs.lightburnsoftware.com/2.1/Troubleshooting/JobQuality/GRBLPowerOutput/
- Per-layer constant power with `M3`, `M4` by default; air assist on `M7` or
  `M8`; `S` only when it changes; overscan as `G1`:
  https://docs.lightburnsoftware.com/2.1/Reference/DeviceSettings/GCode/,
  https://docs.lightburnsoftware.com/2.1/Guides/GRBLConfiguration/
- Positive coordinates from the origin corner, front-left example, absolute
  coordinates, return to a finish position:
  https://docs.lightburnsoftware.com/2.1/Reference/CoordinatesOrigin/,
  https://docs.lightburnsoftware.com/2.1/Guides/GRBLConfiguration/
- Parenthesised comments: NIST RS274/NGC interpreter, version 3 (2000).
- All read 2026-10-21. Own design: the comment header, number precision, the
  70-character limit, `S0` with every mode change, the end sequence, the
  verifier, the mapping for origin corners other than front-left, and the
  finish option defaulting to off.
