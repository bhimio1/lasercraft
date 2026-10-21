# Spec: layers

Milestone: M2. Planning (M4), G-code (M5) and images (M7) consume these
settings later.

## Colors are layers

- The palette has 30 output colors, numbered 0–29. Each color is a layer: a
  process and its settings, applied to every shape of that color. The color
  only tells processes apart on screen; it says nothing about the result.
- Every shape belongs to exactly one layer. New shapes get the current color
  (UI state, shown in the palette); commands that create shapes take the
  color as an argument, defaulting to 0, so redoing them never depends on UI
  state.
- Clicking a palette color with shapes selected moves them to that layer;
  with nothing selected it sets the current color.
- The project always stores settings for all 30 layers, so assigning a color
  never creates or destroys a layer and undo stays exact. The layer list
  shows only layers that have shapes.
- Layers are kept in output order: the job runs them top to bottom. New
  projects start in color order. Reordering is undoable.
- Deleting a layer deletes its shapes (undoable) and keeps its settings.

## Settings

| Setting | Meaning | Default | Valid |
| --- | --- | --- | --- |
| `name` | Label in the list | `Layer NN` | 1–60 characters |
| `mode` | `line` (trace outlines) or `fill` (scan inside closed shapes) | `line` | |
| `speed` | Top speed, mm/s; the machine may not reach it on short moves | 10 | > 0, ≤ 10 000 |
| `max_power` | Peak power, % | 20 | 0–100 |
| `min_power` | Power at low speed / lightest image shade, % | 0 | 0–100, ≤ `max_power` |
| `passes` | Times the layer is run | 1 | 1–1000 |
| `interval` | Fill line spacing, mm | 0.1 | 0.001–10 |
| `angle` | Fill scan angle, degrees; 0 is horizontal lines | 0 | −360–360 |
| `bidirectional` | Fill burns in both directions; off returns unlit | on | |
| `overscan` | Fill runs on past line ends with the laser off | off | |
| `crosshatch` | Fill adds a second pass at 90° | off | |
| `output` | Included in preview, export and jobs | on | |
| `show` | Drawn on the canvas | on | |
| `air` | Air assist for this layer | off | |
| `image.dither` | Image method: `grayscale`, `threshold`, `floyd_steinberg`, `jarvis`, `stucki`, `atkinson`, `ordered`, `halftone` (`docs/spec/images.md`) | `floyd_steinberg` | |
| `image.interval` | Image line spacing and dot size, mm | 0.1 | 0.001–10 |
| `image.negative` | Invert the image | off | |
| `image.halftone_cells` | Halftone screen, cells per inch (M7) | 50 | 1–1000 |
| `image.halftone_angle` | Halftone screen angle, degrees (M7) | 45 | −360–360 |
| `image.dot_correction` | Shorten each burned run by this much, half at each end, mm (M7) | 0 | 0–`image.interval` |
| `kerf` | Line mode: offset closed shapes by this much, mm; positive moves the cut outward from the part, holes the other way (M4) | 0 | −5–5 |
| `tabs` | Line mode: uncut tabs per closed outer shape; 0 is off (M4) | 0 | 0–100 |
| `tab_width` | Line mode: length of each tab, mm (M4) | 1 | 0.01–50 |
| `constant_power` | G-code: constant power (`M3`) instead of dynamic power (`M4`); with `M3` the laser stays on while the head stops, so Pause is motion only (M5, `docs/spec/gcode.md`) | off | |

- `interval` and `image.interval` also accept `<n>lpi` or `<n>dpi`, meaning
  25.4 / n mm, rounded to the nearest nanometre (so `254lpi` is exactly
  0.1 mm).
- `min_power` matters on GRBL only for grayscale images (lightest shade gets
  `min_power`, darkest `max_power`); line and fill use `max_power` with GRBL's
  dynamic power mode (`docs/spec/safety.md`).
- Setting `max_power` below `min_power` fails; lower `min_power` first.
- Power is clamped again to the device maximum at export (M5).
- Hidden layers' shapes are not drawn and cannot be clicked. Layers with
  output off are drawn faded.
- Fill applies to closed shapes. On a fill layer, nested shapes fill by the
  even-odd rule (the ring between them is filled). Open shapes on a fill layer
  are skipped with a warning (M4).
- Image settings apply to images of that color (M7). `bidirectional`,
  `overscan`, speed, power and passes apply to images too. The layer list
  says `image` for a color with only images and `line + image` (or
  `fill + image`) when it has both; vectors are planned first.
- How the planner uses every setting (order, kerf, tabs, scan geometry,
  overscan distance) is in `docs/spec/planning.md`. The editor shows kerf and
  tabs for line layers only; tab width only when tabs are on.

## Commands

| Command | Undo | Effect |
| --- | --- | --- |
| `layer.set <color> <setting> <value>` | yes | Change one setting (table above; booleans `on`/`off`). |
| `layer.set_all <color> <json>` | yes | Replace all of a layer's settings. Used by undo and paste. |
| `layer.copy_settings <from> <to>` | yes | Copy every setting except the name. |
| `layer.move <color> <up\|down\|top\|bottom>` | yes | Move a layer in output order, among the layers that have shapes. |
| `layer.set_order <color> …` | yes | Set the full output order (all 30 colors). |
| `layer.delete <color>` | yes | Delete every shape on the layer. |
| `layer.select <color> [add]` | no | Select the layer's shapes. |
| `shape.set_layer <color> [ids]` | yes | Move shapes to a layer. |
| `shape.add_*` … `[@<color>]` | yes | Create shapes on a layer (`docs/spec/shapes.md`). |

## Sources

- Colors as layers, palette clicks, list showing used colors, order as output
  order, deleting a layer deletes its objects, output/show/air settings,
  vector vs image split: LightBurn docs,
  https://docs.lightburnsoftware.com/2.1/GetStarted/ColorsAndLayers/,
  https://docs.lightburnsoftware.com/2.1/Reference/UI/ColorPalette/,
  https://docs.lightburnsoftware.com/2.1/Reference/CutsLayersWindow/
- Speed, min/max power, passes, GRBL dynamic power:
  https://docs.lightburnsoftware.com/2.1/Reference/CutSettingsEditor/SharedSettings/
- Line and fill behavior, interval, scan angle, bidirectional, overscan,
  crosshatch, even-odd nesting:
  https://docs.lightburnsoftware.com/2.1/Reference/CutSettingsEditor/LineMode/,
  https://docs.lightburnsoftware.com/2.1/Reference/CutSettingsEditor/FillMode/,
  https://docs.lightburnsoftware.com/2.1/Explainers/LayerModes/
- Kerf offset (closed shapes, positive outward) and tabs (count per shape,
  tab size): https://docs.lightburnsoftware.com/2.1/Reference/CutSettingsEditor/LineMode/,
  https://docs.lightburnsoftware.com/2.1/Reference/AddTabs/ (read 2026-10-21;
  ranges and defaults are our own).
- Image settings and DPI = 25.4 / interval:
  https://docs.lightburnsoftware.com/2.1/Reference/CutSettingsEditor/ImageMode/
- All read 2026-10-21. Own design: palette color values, settings stored for
  all 30 layers, default values and ranges, mm/s units, explicit color on
  create commands, open shapes on fill layers, `lpi`/`dpi` suffixes.
