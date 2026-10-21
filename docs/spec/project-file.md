# Spec: project file (`.lcproj`)

Milestone: M0. Later milestones add fields and bump `version` when old readers
must refuse a file.

## Format

A UTF-8 JSON object, pretty-printed with two-space indent and a trailing
newline, so files diff cleanly and golden tests compare exact text.

```json
{
  "format": "lasercraft-project",
  "version": 1,
  "name": "Untitled",
  "notes": "",
  "work_area": {
    "width_mm": 400.0,
    "height_mm": 400.0,
    "origin": "front_left"
  },
  "next_id": 1
}
```

| Field | Meaning |
| --- | --- |
| `format` | Always `lasercraft-project`. |
| `version` | Format version, currently 1. |
| `name` | Display name, 1–200 characters after trimming. |
| `notes` | Free text, at most 20 000 characters. Optional when reading. |
| `work_area.width_mm`, `height_mm` | Reachable bed size, 1–10 000 mm, finite. |
| `work_area.origin` | Bed corner where machine coordinates are zero: `front_left`, `front_right`, `rear_left`, `rear_right`. "Front" is the side nearest the operator. |
| `next_id` | Next object id to hand out, at least 1. |

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
- Every value is validated with the same rules as the commands that set it.

## Writing

- The text goes to `<path>.tmp` beside the target, which is then renamed over
  the target, so an interrupted save never leaves a half-written project.
- The default work area is 400 × 400 mm with a front-left origin (own choice).

## Sources

- JSON: RFC 8259.
- Own design. LightBurn's `.lbrn`/`.lbrn2` formats are not read, written or
  imitated (`docs/clean-room.md`).
