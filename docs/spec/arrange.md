# Spec: groups, booleans, offsets, alignment and arrays

Milestone: M3.

## Groups

- A group is a set of shapes (and smaller groups) that is selected, moved,
  scaled and aligned as one object. Groups can be nested.
- Model: each shape stores `groups`, the ids of the groups it belongs to,
  outermost first (empty: not grouped). Group ids come from the same id
  allocator as shapes, so they are never reused while the project is open.
  The outermost group id is the shape's *top-level object*; ungrouped shapes
  are their own object.
- Selecting any member of a group selects every shape of its top-level
  group: every selection change (clicks, Shift+click, Ctrl+click toggles,
  boxes, `select.*`) is widened to whole top-level groups. Toggling a member
  off toggles the whole group off.
- Commands given explicit shape ids act on exactly those shapes (so undo
  inverses stay exact); commands acting on the selection act on whole
  groups because the selection always holds whole groups.
- Copies made by duplicate, paste and arrays get fresh group ids (shapes
  that shared a group share the new one), so a copy never joins the
  original's group.
- Node editing works on grouped shapes too (own choice; LightBurn asks to
  ungroup first).

| Command | Undo | Effect |
| --- | --- | --- |
| `group.create [ids]` | set_groups | Group the objects (top-level groups or shapes) the ids belong to into a new outermost group. Needs at least two objects. |
| `group.ungroup [ids]` | set_groups | Remove the outermost group of every grouped object among the ids. Fails if none is grouped. |
| `group.set <id> [group …]` | set_groups | Set a shape's group chain exactly. Used by undo. |

## Anchor object

Alignment, subtraction and moving together refer to one *anchor* object in
the selection: the object added to the selection last. When one selection
change adds several objects at once (a box, select all), the anchor is the
first of them in drawing order. If the anchor is deselected, the first
selected object in drawing order becomes the anchor. Commands given
explicit ids use the last id given as the anchor (ids are listed in the
order they would be selected).

Subtract instead removes the object selected *second*: the last object
added. When the latest selection change added both objects at once (a box,
select all, a layer), drawing order stands in for selection order, so the
object drawn on top is removed from the one below: draw a plate, draw a hole
on it, box-select both, subtract.

## Booleans, weld and offset

Shared rules:

- Inputs must be closed: an open shape fails with "booleans need closed
  shapes" (offset: "offset needs closed shapes").
- Curves are flattened within 0.01 mm; the result is one new `path` shape
  made of straight segments, in document coordinates, with identity
  transform, added on top of the drawing order and selected. The inputs are
  removed. Undo deletes the result and restores every input exactly.
- An empty result fails ("the result is empty") and changes nothing.

| Command | Effect | Result layer |
| --- | --- | --- |
| `path.union [ids]` | Area in either of exactly two objects. | bottom input |
| `path.subtract [ids]` | The object selected second (see above; with ids, the last id) is removed from the other one. | the other one |
| `path.intersect [ids]` | Area inside both of exactly two objects. | bottom input |
| `path.weld [ids]` | Merge any number of shapes into one outline. | bottom input |
| `path.offset <distance> [round\|miter\|bevel] [ids]` | New outline `distance` mm outside (positive) or inside (negative) the shapes; the inputs are kept. Default corners: round. | bottom input |

- Objects for union, subtract and intersect are top-level objects: a group
  counts as one object whose inside is decided by the even-odd rule across
  all its shapes (a group of two nested circles is a ring). Exactly two
  objects are needed ("select exactly two objects").
- Weld treats every shape on its own (even-odd inside each shape) and
  merges them all; a shape fully inside another disappears into it. Group
  the inner and outer parts and use union instead to keep holes made by
  separate shapes.
- Offset first welds the selected shapes as above, then grows or shrinks
  the outline. Shrinking past the middle leaves nothing (an error).
  Distance 0 rebuilds the outline, which can repair overlapping or
  self-crossing art. Offset keeps the inputs and selects the new outline.
- "Bottom input" is the input shape lowest in drawing order.

## Align, distribute, move together, move on the bed

All translate whole top-level objects (shapes keep their shape; groups keep
their members' relative positions) and are one undo step whose inverse sets
each moved shape's transform back exactly. An object's edges are its bounds
(the extreme points of its outlines).

| Command | Effect |
| --- | --- |
| `arrange.align <left\|right\|top\|bottom\|center-x\|center-y\|center> [ids]` | Line every object up with the anchor: same left/right/top/bottom edge; `center-x` the same vertical center line (same x center), `center-y` the same horizontal center line (same y center), `center` both. Needs two objects. |
| `arrange.distribute <h-spaced\|h-centered\|v-spaced\|v-centered> [ids]` | Even spacing, horizontally or vertically. The outermost objects (by center) stay; `-centered` spaces the centers evenly, `-spaced` makes the gaps between neighboring edges equal (gaps may be negative when the objects overlap more than they are apart). Needs three objects. |
| `arrange.move_together <horizontal\|vertical> [ids]` | Pack the objects edge to edge in a row (or column), in their current order by center, keeping the anchor where it is. Each object keeps its other coordinate. Needs two objects. |
| `arrange.move_to_bed <position> [ids]` | Move the selection as a block so its bounds touch the work area's edge or center: `center`, `top-left`, `top`, `top-right`, `left`, `right`, `bottom-left`, `bottom`, `bottom-right` (top is the rear of the bed, as on screen). |

## Arrays

Arrays copy the selection as a block (all selected shapes keep their
relative positions); the originals stay where they are. The copies are
placed above the originals in drawing order, get fresh ids and fresh group
ids, and the originals plus copies end up selected. Undo deletes the
copies. One array makes at most 10 000 copied shapes.

| Command | Effect |
| --- | --- |
| `arrange.grid <columns> <rows> <x_gap> <y_gap> [ids]` | A grid of `columns × rows` blocks (the original is the top-left one); neighboring blocks are `x_gap` / `y_gap` mm apart edge to edge (negative overlaps). Columns go right (+x), rows down (+y). Needs at least two cells. |
| `arrange.circular <count> <cx> <cy> <sweep_deg> <rotate\|upright> [ids]` | `count` blocks in total (the original is the first) around (cx, cy). Block k is turned by `k · step` degrees clockwise, where `step = sweep / count` for a full turn (\|sweep\| = 360) and `sweep / (count − 1)` otherwise, so a partial arc ends exactly at the sweep. `rotate` turns each copy with its position; `upright` only moves each copy so its center follows that turn. |

## Keyboard

Ctrl+G group, Ctrl+U ungroup, Ctrl+W weld, Alt++ union, Alt+- subtract,
Alt+O offset (opens the offset window), Alt+←/→/↑/↓ align
left/right/top/bottom, Alt+PgUp align vertical centers (`center-x`),
Alt+PgDn horizontal centers (`center-y`), Alt+Home centers,
Alt+Shift+H / Alt+Shift+V move together. Alt+* (intersect) is not bound:
egui has no key for `*` (it is Shift+8 on many layouts, which differs
per layout); intersect is in the Tools menu. The legacy single-letter
canvas keys (L R T B C E P) are not bound: they clash with the node tool's
keys and with typing.

## Sources

- Groups (Ctrl+G/Ctrl+U, nested groups, groups act as one unit for
  selection, moving, alignment and booleans; imported shapes grouped):
  LightBurn docs, "Grouping", https://docs.lightburnsoftware.com/2.1/Reference/Grouping/
- Booleans need two closed objects, subtract removes the second-selected
  object, intersect keeps the overlap, weld merges any number, weld versus
  grouped union for nested shapes, shortcuts: "Boolean Tools",
  https://docs.lightburnsoftware.com/2.1/Reference/BooleanTools/
- Offset outward/inward, corner styles, offset 0 repairs art, Alt+O:
  "Offset Shapes", https://docs.lightburnsoftware.com/2.1/Reference/OffsetShapes/
- Align to the last-selected object, draw-order fallback for objects
  selected together, groups as single objects, edges from extreme points,
  shortcuts: "Align", https://docs.lightburnsoftware.com/2.1/Reference/Align/
- Distribute by centers or edge spacing using the average spacing, move
  together keeping the last-selected object fixed, shortcuts: "Distribute",
  https://docs.lightburnsoftware.com/2.1/Reference/Distribute/
- Moving the selection to the work area's center, corners and edge
  midpoints as one block: "Move Selected Objects",
  https://docs.lightburnsoftware.com/2.1/Reference/MoveSelectedObjects/
- Grid array of columns and rows with center or edge spacing; circular
  array with a copy count, center, start/end range and optionally rotated
  copies: "Grid Array", https://docs.lightburnsoftware.com/2.1/Reference/GridArray/,
  "Circular Array", https://docs.lightburnsoftware.com/2.1/Reference/CircularArray/
- Arrange and Tools menu contents:
  https://docs.lightburnsoftware.com/2.1/Reference/UI/ArrangeMenu/,
  https://docs.lightburnsoftware.com/2.1/Reference/UI/ToolsMenu/
- All read 2026-10-21. Own design: the group-chain model, widening every
  selection change, the anchor rule for explicit ids, flattening tolerance,
  result placement and layers, offset welding its inputs first, edge-gap
  grid spacing (the only grid mode), the circular step rule, the
  10 000-copy limit, and leaving out mirrored/shifted/random grid options,
  docking and virtual arrays.
