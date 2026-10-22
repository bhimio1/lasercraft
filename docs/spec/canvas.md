# Spec: canvas

Milestones: M1, M2 (layer colors); restyled to the style guide's tokens
(`docs/spec/style/canvas.md`).

## View

- The canvas shows the work area as a near-white sheet (`bed`) on a slightly
  darker surround (`canvas`) in every theme, so the black layer stays visible,
  with a grid: minor lines every 10 mm and major lines every 50 mm
  (`grid-minor`, `grid-major`, hairline); minor lines are hidden when they
  would be closer than 6 px. The work area's edge (`bed-edge`, thin) is drawn
  above the grid. The rear of the bed is at the top of the screen.
- The machine-origin corner is marked in `origin-marker` (dark slate, never
  red): a small filled square on the corner and two heavy arms along the bed
  edges pointing into the bed, with the letters X and Y beside their tips.
  Very small beds show the square only.
- Colors are theme tokens (`docs/spec/style/visual-language.md`); the canvas
  tokens are the same in every theme.
- Zoom: mouse wheel about the pointer; zoom in / out by a factor of 1.25 about
  the view center; "fit" shows the whole bed with a margin. Zoom is limited to
  0.05–200 px per mm.
- Pan: middle-button drag, or hold Space and drag with the left button.
- Shapes are drawn in their layer's color (normal line), selected ones with a
  heavy line, never recolored. Layers with output off are drawn faded;
  hidden layers are not drawn.
- With the Select tool, the topmost shape under the pointer is drawn with a
  heavy line in its own color (hover); nothing changes if it is already
  selected or while dragging.
- The selection's bounding box is a dashed `selection` line (a deep shade of
  the accent blue) offset slightly outside the shapes; its handles are white
  (`handle-fill`) with `selection` outlines, the rotate handle a circle on a
  short stem. Selection boxes being dragged are `marquee`: a solid outline
  when picking shapes inside the box (left to right), dashed when picking
  shapes it touches (right to left), both with a very faint fill. A shape
  being drawn is a dashed `drawing` line until it is created.
- Node editing: nodes are small white squares with a dark outline
  (`node`), the start of each subpath filled `start-marker` (dark violet);
  Bézier handle lines are hairlines in `text-muted` with small white circles
  outlined in `selection`; a dragged node or handle is filled `selection`.
- The status bar shows the pointer position in mm and the selection's size.
- Zoom and pan are view state, not commands, and are not saved.

## Tools

- **Select** (default; Esc returns to it).
  - Click selects the topmost shape under the pointer (5 px tolerance);
    Shift+click adds, Ctrl+click toggles; clicking empty space clears.
  - Dragging from empty space draws a selection box: left to right selects
    contained shapes, right to left selects touched shapes.
  - Dragging a selected shape moves the selection.
  - The selection shows its combined bounds with eight scale handles and a
    rotate handle above the top edge.
- **Rectangle**, **Ellipse**: drag from corner to corner (Ctrl: from the
  center; Shift: equal width and height). A drag shorter than 0.5 mm in either
  direction draws nothing.
- **Line**: click points to build a polyline (points snap like drawn
  corners); double-click or Enter finishes;
  clicking the first point again closes the shape; Esc cancels the unfinished
  line. A two-point result is a line.
- A finished drawing selects the new shape and stays in the tool.
- **Text** (M3): a click opens the text window at that point
  (`docs/spec/text.md`).
- **Edit Nodes** (M3): nodes and handles of selected paths, dragged and
  edited as in `docs/spec/nodes.md`. The selection box and its handles are
  hidden in this tool.

## Dragging

- Moves, resizes and rotations preview during the drag and become one
  command when the button is released (one undo step). Releasing without
  movement changes nothing.
- Snapping (on by default; Ctrl held turns it off):
  - Points snap to the grid (10 mm) and to other shapes' bounds corners,
    edge midpoints and centers when within 8 px; the object point wins over
    the grid when both are in range.
  - Moves snap the corner or center of the moving bounds that is nearest to a
    target.
- Shift while moving: the move is constrained to horizontal, vertical or 45°.
- Resizing: a corner handle keeps the aspect ratio unless Shift is held; an
  edge handle scales one axis. Ctrl scales about the center instead of the
  opposite handle. Dragging past the anchor mirrors.
- Rotating: about the selection center; the angle snaps to 15° steps unless
  Ctrl is held.

## Sources

- Selection, zoom, pan and modifier gestures: LightBurn 2.0 hotkey cheat sheet,
  https://docs.lightburnsoftware.com/2.0/download/LightBurnWindowsHotkeysV2.0.pdf,
  and "Hotkeys and Gestures", https://docs.lightburnsoftware.com/legacy/Hotkeys
- Grid spacing, snap radius, zoom limits, handle layout and the rotate handle:
  own design.
