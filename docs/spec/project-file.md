# Spec: project file (`.lcproj`)

Milestones: M0 (version 1), M1 (version 2 adds shapes). Later milestones add
fields and bump `version` when old readers must refuse a file.

## Format

A UTF-8 JSON object, pretty-printed with two-space indent and a trailing
newline, so files diff cleanly and golden tests compare exact text.

```json
{
  "format": "lasercraft-project",
  "version": 2,
  "name": "Untitled",
  "notes": "",
  "work_area": {
    "width_mm": 400.0,
    "height_mm": 400.0,
    "origin": "front_left"
  },
  "next_id": 2,
  "shapes": [
    {
      "id": 1,
      "geometry": { "kind": "rect", "width": 20.0, "height": 10.0 },
      "transform": [1.0, 0.0, 0.0, 1.0, 5.0, 5.0]
    }
  ]
}
```

(Shown compactly; files are pretty-printed with one value per line.)

| Field | Meaning |
| --- | --- |
| `format` | Always `lasercraft-project`. |
| `version` | Format version, currently 2. Version 1 files (no shapes) load and are saved as version 2. |
| `name` | Display name, 1–200 characters after trimming. |
| `notes` | Free text, at most 20 000 characters. Optional when reading. |
| `work_area.width_mm`, `height_mm` | Reachable bed size, 1–10 000 mm, finite. |
| `work_area.origin` | Bed corner where machine coordinates are zero: `front_left`, `front_right`, `rear_left`, `rear_right`. "Front" is the side nearest the operator. |
| `next_id` | Next object id to hand out; greater than every shape id. |
| `shapes` | Shapes in drawing order (`docs/spec/shapes.md`): `id` (unique, ≥ 1), `geometry` (`rect` with `width`/`height`, `ellipse` with `rx`/`ry`, or `polyline` with `points` as `[x, y]` pairs and `closed`), and `transform` as the six numbers `[a, b, c, d, e, f]` of SVG's `matrix()`. |

Document coordinates are millimetres with x to the right and y downwards
(SVG convention, `docs/spec/geometry.md`). The origin corner says how they map
to machine coordinates; that mapping is used from M5 on.

## Reading

- Not JSON, or `format` missing or different: refused as "not a valid
  LaserCraft project".
- `version` missing or 0: refused. Greater than the supported version: refused
  with a message asking the user to update LaserCraft.
- Unknown fields are refused, so a damaged or newer file is never half-read and
  then overwritten.
- Every value is validated with the same rules as the commands that set it:
  positive sizes, at least two finite points, invertible finite transforms,
  coordinates within ±1 000 000 mm, unique ids.
- Numbers are written in their shortest exact form and parsed exactly, so a
  save and load never changes a value by even one bit.
- Selection and clipboard are never saved.

## Writing

- The text goes to `<path>.tmp` beside the target, which is then renamed over
  the target, so an interrupted save never leaves a half-written project.
- The default work area is 400 × 400 mm with a front-left origin (own choice).

## Sources

- JSON: RFC 8259.
- Own design. LightBurn's `.lbrn`/`.lbrn2` formats are not read, written or
  imitated (`docs/clean-room.md`).
