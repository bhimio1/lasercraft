# Spec: test patterns

Milestone: M8. Test patterns are temporary jobs that burn a grid of
samples, so the user can pick settings for a material. A pattern is never
added to the project; it is built from its settings, planned, previewed,
exported or streamed like a job, and obeys `docs/spec/safety.md` exactly
like Start.

## Speed and power test (`speed_power`)

A grid of boxes. Down the rows one setting varies, across the columns
another; every other setting comes from the box settings.

Settings (JSON object; missing keys take the defaults):

| Key | Meaning | Default | Valid |
| --- | --- | --- | --- |
| `rows.count`, `columns.count` | Number of rows / columns | 10, 10 | 2–20 |
| `rows.setting`, `columns.setting` | `speed`, `power`, `interval` or `passes` | `speed`, `power` | different from each other |
| `rows.from`, `rows.to`, `columns.from`, `columns.to` | First and last value; the values between are spaced evenly | 10–100, 10–100 | the setting's range in `docs/spec/layers.md`; passes are rounded to whole numbers |
| `rows.size`, `columns.size` | Box height / width, mm | 5, 5 | 1–100 |
| `spacing` | Gap between boxes, mm | 1.5 | 0–20 |
| `center` | `[x, y]`, the pattern's center in document mm, or `null` for the bed center | `null` | the pattern must fit the work area |
| `box` | Layer settings for the boxes (layer object, keys as `docs/spec/layers.md`; missing keys from the defaults) | fill, 100 mm/s, 30%, interval 0.1 mm, 1 pass | as layers |
| `labels` | Layer settings for the value labels and titles | line, 50 mm/s, 20% | as layers |
| `border` | Layer settings for the border | line, 50 mm/s, 20% | as layers |
| `draw_labels`, `draw_border` | Burn the labels / the border | true, true | |

The pattern (document mm, y down):

- Rows run top to bottom from `rows.from` to `rows.to`, columns left to
  right from `columns.from` to `columns.to`. Each box is a rectangle of
  the box settings with the row's and the column's value applied.
- Labels in LaserCraft's own single-stroke font (below): each row's value
  left of its row, each column's value under its column, the row setting's
  name and unit written upward at the far left ("SPEED MM/S"), the column
  setting's centered under the column values ("POWER %"), and one line
  above the grid with the box settings that do not vary (mode, speed,
  power, interval for fill, passes). Text height is 2.5 mm, smaller when
  a value would not fit its box (never below 1 mm).
- The border is a rectangle 2 mm outside everything else.
- Run order: labels and border first (low power), then the boxes from the
  least energy to the most, where energy is power × passes ÷ (speed ×
  interval) (interval counts only in fill mode). Ties keep row, then
  column order.
- Every power is clamped to the device's max power (safety rule 6); the
  window warns when a value is clamped.

## Line interval test (`interval`)

A row of square patches, each filled at its own line interval.

| Key | Meaning | Default | Valid |
| --- | --- | --- | --- |
| `speed` | mm/s | 100 | as layers |
| `power` | Max power, % | 30 | 0–100 |
| `patches` | Number of patches | 8 | 2–20 |
| `smallest`, `largest` | Interval range, mm, spaced evenly | 0.05, 0.3 | 0.001–10, smallest < largest |
| `size` | Patch side, mm | 10 | 2–50 |
| `spacing` | Gap between patches, mm | 3 | 0–20 |
| `fill` | `solid` (a fill scan) or `gradient` (a left-to-right white-to-black ramp engraved as an image with `dither`) | `solid` | |
| `dither` | Image method for `gradient` | `floyd_steinberg` | as `docs/spec/layers.md` |
| `center` | as above | `null` | |
| `draw_labels` | Each interval printed under its patch, and a line above with speed and power | true | |

- Run order: labels first, then patches from the largest interval (least
  energy) to the smallest.

## Single-stroke label font

- Own design: uppercase letters A–Z, digits, space and `. , - + % ( ) / :
  × =`, each drawn as open polylines on a 4 × 6 unit cell with one unit
  between characters. Text is upper-cased before drawing; characters
  without a glyph draw nothing and take one cell.
- Label strokes are burned in line mode with the label settings.

## Commands

| Command | Effect |
| --- | --- |
| `test.export <kind> <path> [json]` | Plan the pattern with the project's device profile and write verified G-code (as `project.export_gcode`; refusals the same). Does not change the project. |
| `device.test <kind> [json] [placement] [s_max=] [air=]` | Stream the pattern (`Stream { Job }`), with the placement arguments of `device.start`. |
| `device.test_frame <kind> [json] [outline] [placement] [speed=]` | Trace the pattern's box (or outline) with the laser off. |
| `test.preset_save <name> <kind> <json>` | Store a user preset (replaces one of the same name and kind). |
| `test.preset_delete <name> <kind>` | Remove a user preset; built-in presets cannot be removed. |
| `test.preset_list` | Built-in and user presets. |
| `test.remember <kind> <json>` | Check the settings and keep them as the last used for `kind` (the test windows run it as values settle). Runs nothing. |
| `test.preset_import <path>` / `test.preset_export <path>` | Read or write the user presets as a `.lctests` file. |

- `kind` is `speed_power` or `interval`; `json` is the settings object
  (missing keys take the defaults).
- Built-in presets (own data): "Diode engraving" (speed 20–200 mm/s down
  the rows, power 10–60% across, boxes in fill mode at 0.1 mm) and "Diode
  cutting" (speed 1–10 mm/s, power 60–100%, boxes in line mode, 2 passes).
- User presets and the last settings of each kind are kept in
  `test-presets.json` in the configuration folder (`docs/spec/devices.md`):
  `{"format": "lasercraft-test-presets", "version": 1, "presets": [{"name",
  "kind", "settings"}], "last": {kind: settings}}`, written at once.

## Windows (app)

As `docs/spec/style/later-windows.md`, "Test patterns": "Speed and power
test" and "Line interval test", opened from Machine → "Speed and power
test…" and "Line interval test…". Movable, non-modal, never covering the
job bar.

- Speed and power test: Preset row ("Preset" dropdown with "(none)",
  "Built-in", "Yours"; Save preset…, Delete preset, Import presets…,
  Export presets…), the "Rows" and "Columns" groups (Count, Setting, From
  and To with the setting's unit, Box height (mm) / Box width (mm), Center
  Y (mm) / Center X (mm)), the "Run" group (Job preview…, Frame, Start,
  Pause, Stop, Save G-code… and "Pattern size ‹w› × ‹h› mm"), the parts
  row (Box settings…, Label settings…, Border settings… with swatch chips
  00, 01, 02; switches Draw labels, Draw border) and Close.
- Line interval test: Speed (mm/s), Max power (%), Patches; Smallest
  interval (mm), Largest interval (mm), Patch size (mm) with "‹n› lines
  per inch" hints; Solid / Gradient choice dots; the Run group; Close.
- The Center fields are enabled only when Place job at is Absolute
  coordinates; otherwise Place job at and the job reference point place
  the pattern, exactly as for Start.
- Job preview… shows the pattern in the job preview window. Frame, Start,
  Pause and Stop behave as in the Machine panel; Pause and Stop stay in
  the job bar too.
- Values are remembered as they change (no OK/Cancel): once no field is
  being dragged, the app runs `test.remember`. The app checks and plans
  the window's settings (the UI never plans); invalid settings or a
  pattern outside the bed show "Cannot run: …" and disable the job
  buttons.
- The Focus height test needs Z moves, which LaserCraft does not write
  yet; its menu row is not shown until then.

## Sources

- Material test grid with rows and columns varying speed, power, interval
  or passes, presets, box, label and border settings, least risky box
  first: https://docs.lightburnsoftware.com/2.1/Reference/MaterialTest/
  (read 2026-10-22).
- Interval test as a row of squares at different line intervals, solid or
  dithered: https://docs.lightburnsoftware.com/2.1/Reference/IntervalTest/
  (read 2026-10-22).
- Own design: settings format and defaults, layout, energy order, the
  stroke font, commands, presets file and built-in presets.
