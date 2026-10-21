# Spec: shapes, selection and transforms

Milestone: M1.

## Model

- A shape has an `id` (unique in the project, never reused while the project
  is open), a local `geometry`, and an affine `transform` from local to
  document coordinates (millimetres, y down; `docs/spec/geometry.md`).
- Geometry kinds:
  - `rect`: `width`, `height` > 0, local corners (0,0) and (width,height).
  - `ellipse`: `rx`, `ry` > 0, centered on the local origin.
  - `polyline`: at least 2 finite `points`, open or `closed`. A line is a
    polyline with 2 points.
- Shapes are kept in drawing order: later shapes are drawn on top and win hit
  tests.
- Move, scale, rotate and mirror change only the transform. The transform must
  stay invertible (no zero scale).
- Bounds are exact for every kind, including rotated ellipses.

## Selection and clipboard

- The selection is a set of shape ids; the clipboard holds copies of shapes.
  Both are view state: they are not saved, not undone, and not part of
  document equality. They still change only through commands.
- Adding, duplicating or pasting shapes selects the new shapes. Deleting a
  shape removes it from the selection.
- Commands that act on shapes take explicit ids, or use the selection when no
  ids are given. With no ids and an empty selection they fail with "nothing
  selected". Inverses always name ids explicitly.

## Hit testing

- Shapes on hidden layers are never hit and never picked by a box.
- A point hits a shape when it is within the tolerance of its outline, or
  inside a closed shape (even-odd rule). The topmost hit wins.
- Box selection: "contained" picks shapes whose bounds lie inside the box;
  "touching" picks shapes whose outline crosses or lies inside the box, or
  that contain the box.

## Commands

Lengths in mm, angles in degrees (positive is clockwise on screen). `[ids]`
means optional trailing shape ids, defaulting to the selection. `[@color]`
puts a new shape on a layer (`docs/spec/layers.md`); the default is 0.

| Command | Undo | Effect |
| --- | --- | --- |
| `shape.add_rect <x> <y> <w> <h> [@color]` | delete | Rectangle with top-left corner (x, y). |
| `shape.add_ellipse <cx> <cy> <rx> <ry> [@color]` | delete | Ellipse. |
| `shape.add_line <x1> <y1> <x2> <y2> [@color]` | delete | Two-point polyline. |
| `shape.add_polyline [closed] <x1> <y1> <x2> <y2> … [@color]` | delete | Polyline through the points. |
| `shape.insert <index> <json>` | delete | Insert a shape (with its id) at a drawing-order position. Used by undo. |
| `shape.delete [ids]` | insert | Delete shapes. |
| `shape.duplicate [ids]` | delete | Copies placed exactly on top of the originals, above them in drawing order. |
| `shape.move <dx> <dy> [ids]` | set_transform | Translate. |
| `shape.scale <sx> <sy> [ids]` | set_transform | Scale about the center of the shapes' combined bounds. Negative factors mirror. |
| `shape.scale_about <sx> <sy> <cx> <cy> [ids]` | set_transform | Scale about a point. |
| `shape.rotate <deg> [ids]` | set_transform | Rotate about the center of the combined bounds. |
| `shape.rotate_about <deg> <cx> <cy> [ids]` | set_transform | Rotate about a point. |
| `shape.flip <horizontal\|vertical> [ids]` | set_transform | Mirror about the center of the combined bounds. |
| `shape.set_transform <id> <a> <b> <c> <d> <e> <f>` | set_transform | Set a shape's transform exactly. |
| `shape.raise [ids]`, `shape.lower [ids]` | set_order | One step up / down in drawing order, keeping the selected shapes' relative order. |
| `shape.to_front [ids]`, `shape.to_back [ids]` | set_order | To the top / bottom. |
| `shape.set_order <id> …` | set_order | Reorder; the ids must be every shape exactly once. |
| `select.set <id> …`, `select.add`, `select.remove`, `select.toggle` | no | Change the selection. Unknown ids fail. |
| `select.clear`, `select.all`, `select.invert` | no | |
| `select.at <x> <y> <tolerance> [replace\|add\|toggle]` | no | Hit-test; a miss with `replace` clears the selection. |
| `select.in_rect <x0> <y0> <x1> <y1> <contained\|touching> [replace\|add\|toggle]` | no | Box selection. |
| `select.next`, `select.previous` | no | Select the next / previous shape in drawing order, wrapping round. |
| `edit.copy [ids]` | no | Copy shapes to the clipboard. |
| `edit.cut [ids]` | insert | Copy, then delete. |
| `edit.paste [<x> <y>]` | delete | Paste copies with new ids, centered on (x, y) if given, else in place. |
| `edit.paste_in_place` | delete | Paste copies at their original position. |

Several shapes' inverses are combined into one `edit.batch`, so every command
is one undo step.

## Sources

- Selection gestures, duplicate, paste in place, drawing order, rotation steps:
  LightBurn 2.0 hotkey cheat sheet,
  https://docs.lightburnsoftware.com/2.0/download/LightBurnWindowsHotkeysV2.0.pdf
- Transforms: SVG 1.1 coordinate systems, https://www.w3.org/TR/SVG11/coords.html
- Model, ids, commands, hit-test rules and paste placement: own design.
