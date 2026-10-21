# Spec: canvas

Milestones: M1, M2 (layer colors).

## View

- The canvas shows the work area as a light sheet in every theme (so the
  black layer stays visible) with a grid: minor lines every
  10 mm and major lines every 50 mm; minor lines are hidden when they would be
  closer than 6 px. The rear of the bed is at the top of the screen. The
  machine-origin corner is marked.
- Zoom: mouse wheel about the pointer; zoom in / out by a factor of 1.25 about
  the view center; "fit" shows the whole bed with a margin. Zoom is limited to
  0.05–200 px per mm.
- Pan: middle-button drag, or hold Space and drag with the left button.
- Shapes are drawn in their layer's color, selected ones with a heavier line.
  Layers with output off are drawn faded; hidden layers are not drawn.
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
