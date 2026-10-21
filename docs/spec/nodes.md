# Spec: paths and node editing

Milestone: M3.

## Converting to paths

- Rectangles, ellipses and polylines are kept as parametric shapes until
  converted. `shape.to_path` turns them into `path` geometry with the same
  outline and the same transform: a rectangle into four lines, a polyline
  into lines, an ellipse into four cubic quarter arcs. Paths are left alone.
- Text is created as a path already (`docs/spec/text.md`).
- Node commands need path geometry; on another kind they fail with "convert
  the shape to a path first".

## Nodes and segments

A path is one or more subpaths (`docs/spec/geometry.md`). Node commands
address parts of a shape by numbers, all counted from 0:

- `<sub>`: the subpath.
- `<node>`: node 0 is the subpath's start; node k is the end of segment
  k − 1. In a closed subpath whose last segment ends exactly on the start,
  that end is node 0 again, not a separate node.
- `<seg>`: segment k runs from node k to node k + 1. A closed subpath has
  one segment per node: the last one runs back to node 0 (a straight line
  when the file has no explicit closing segment).
- A cubic segment has two handles: handle 1 belongs to its first node,
  handle 2 to its second.
- Positions in node commands are document millimetres; the command maps
  them into the shape's own coordinates through its transform.
- After an edit, a closed subpath's closing segment is written explicitly
  only when it is a curve; a straight closing segment stays implicit.

## Commands

All node commands are undoable; the inverse is `shape.set_geometry` with
the shape's previous geometry, so undo is exact.

| Command | Effect |
| --- | --- |
| `shape.to_path [ids]` | Convert shapes to paths (above). Shapes that are already paths are skipped; with nothing to convert it fails. |
| `shape.set_geometry <id> <json>` | Replace a shape's geometry exactly. Used by undo. |
| `node.move <id> <sub> <node> <x> <y>` | Move a node to (x, y). Its handles move with it. |
| `node.set_handle <id> <sub> <seg> <1\|2> <x> <y>` | Move one handle of a cubic segment to (x, y). |
| `node.delete <id> <sub> <node>` | Remove a node. Its two segments become one, a curve if either was a curve (keeping the outer handles), else a line. Removing an end node of an open subpath drops its segment. A subpath left without segments is removed; removing the shape's last segment fails ("delete the shape instead"). |
| `node.insert <id> <sub> <seg> [t]` | Split a segment at parameter t (0 < t < 1, default 0.5, the midpoint for lines). Curves are split exactly (de Casteljau), so the outline does not change. |
| `node.to_line <id> <sub> <seg>` | Make a segment straight. |
| `node.to_curve <id> <sub> <seg>` | Make a straight segment a cubic with handles at one and two thirds along it (the outline does not move until a handle is dragged). |
| `node.break <id> <sub> <node>` | Open the path at a node. A closed subpath becomes open, starting and ending at that node; an open subpath splits in two there. End nodes of open subpaths cannot be broken. |
| `node.delete_segment <id> <sub> <seg>` | Remove one segment: a closed subpath opens there, an open one splits in two (or loses an end segment). |
| `path.close [ids]` | Close open subpaths whose ends are within 0.5 mm of each other with a straight line (or by merging the ends when they coincide exactly). Fails when nothing could be closed. |
| `path.join [ids]` | Join the selected shapes' open subpaths whose ends lie within 0.05 mm of each other, and close chains whose ends meet. The result is one new path shape (on the layer of the bottom shape) replacing the selected shapes. |
| `path.break_apart [ids]` | Replace each shape by one open path shape per segment (a rectangle becomes four lines). Shapes converted first if needed. Each piece keeps its shape's layer. |

Validation: subpath, node and segment numbers must exist; t must be inside
(0, 1); handles exist only on cubic segments ("segment 2 is not a curve").

## Node tool (canvas)

- Ctrl+\` (or Tools → Edit Nodes) switches the canvas to the node tool.
- Clicking a shape selects it, as in the select tool. Every selected path
  shows its nodes as small squares (the start of each subpath filled) and,
  for curves, its handles as small circles joined to their nodes.
- Dragging a node or handle previews the edit; releasing runs one
  `node.move` or `node.set_handle` (one undo step). Snapping follows the
  canvas rules; Shift keeps the drag horizontal, vertical or at 45°.
- With the pointer over a node or segment of a selected path, these keys act
  on it (tool-local keys, active only in the node tool and not rebindable
  in the keymap file): `D` deletes the node (or, over a segment, the
  segment), `I` inserts a node at the pointer, `M` inserts one at the
  segment's middle, `L` straightens the segment, `C` makes it a curve, `B`
  breaks the path at the node.
- Selected shapes that are not paths show no nodes; convert them first
  (Ctrl+Shift+C).
- Esc returns to the select tool.

## Sources

- Converting built-in shapes and text to editable paths, Ctrl+Shift+C:
  LightBurn docs, "Convert to Path",
  https://docs.lightburnsoftware.com/2.1/Reference/ConvertToPath/
- Node editing tool, Ctrl+\`, dragging nodes and handles, the per-node
  actions (delete, insert, insert at midpoint, break, delete segment,
  straighten, make curve) chosen by hovering and pressing a key: LightBurn
  docs, "Edit Nodes", https://docs.lightburnsoftware.com/2.1/Reference/EditNodes/
- Close path for ends within 0.5 mm: "Close Path",
  https://docs.lightburnsoftware.com/2.1/Reference/ClosePath/
- Joining ends within 0.05 mm, Alt+J: "Auto-Join Selected Shapes",
  https://docs.lightburnsoftware.com/2.1/Reference/AutoJoinSelectedShapes/
- Splitting into individual lines and curves, Alt+B: "Break Apart",
  https://docs.lightburnsoftware.com/2.1/Reference/BreakApart/
- All read 2026-10-21. Node and segment numbering, document-space
  arguments, how delete merges curves, the handle placement of a new curve,
  `C` for "make curve" (the docs use `S` for both smoothing and line→curve;
  smoothing nodes is not implemented in M3), and the inverse through
  `shape.set_geometry`: own design.
- Curve splitting: de Casteljau's algorithm (standard Bézier math).
