# Spec: style — canvas

Milestones: M1 (bed, grid, shapes, selection bounds and handles, selection
boxes, drawing previews, snapping, zoom and pan: all exist today and are
restyled here), M2 (layer colors, Output and Show on the canvas), M3 (node
editing visuals, live previews of tool windows), M4 (the canvas's relation
to the job preview), M6 (head marker, job origin marker, frame path, Move
head tool, the canvas in the machine-connected states), M7 (images on the
canvas), M9 (camera overlay, registration targets); everything else here
(rulers, guide lines, alignment lines, Move and Skew handles, snap cursors,
start arrows, tab markers, Measure, linked copies, per-tool cursors, Show
fills, Smooth curves) is later, with no milestone yet.

This file fixes how the central drawing area looks and answers the pointer:
the surround, the bed, the grid, the rulers, every marker, how shapes and
the selection are drawn, every handle and its states, the selection boxes,
hover and snap feedback, the preview each tool draws while in use, node
editing, and the cursor for each tool. It does not repeat what other style
files own: where the canvas region sits and how it resizes is `layout.md`'s
("Canvas region"); the names used here are `naming.md`'s; every color, type
size and spacing word is a token of `visual-language.md` (the master table,
which this file feeds with the canvas tokens' meanings); the toolbar buttons
that pick each tool are `toolbars.md`'s; the X, Y, size and rotation fields
and the reference-point grid are `property-bar.md`'s; the Show switches and
layer colors are `layers-panel.md`'s; the job bar, the Machine and Jog
panels and the Show head marker switch are `machine-panels.md`'s; the View,
Arrange and canvas context menus are `menus.md`'s; tool windows with live
previews are `dialogs.md`'s; the preview window is `job-preview.md`'s; the
Camera panel is `later-windows.md`'s; icon and cursor-badge artwork is
`icons.md`'s. The tiers (Match, Echo, Own) are meant to be explained in
`README.md`, which is still to be written; until then "How to read this
file" below states them, and `visual-language.md` uses the same three. The
behaviour rules in `docs/spec/canvas.md`, `docs/spec/shapes.md`,
`docs/spec/nodes.md` and `docs/spec/keymap.md` win where they already
decide something; this file proposes changes to them only under "Proposed
changes to other specs".

## How to read this file

- **Reference** paragraphs say what LightBurn's public 2.1 docs show, each
  fact tagged [Rn] (see Sources). Where the docs are silent the fact is
  "unknown" and the LaserCraft part is marked own design.
- **LaserCraft** paragraphs are the spec. Colors are named only by token
  (`bed`, `selection`, …); sizes only relative to `icon-small`, `row` or the
  normal stroke ("hairline", "one step heavier"). There are no values.
- **Tier** (the guide's shared tiers; `README.md` will hold them once it
  is written): Match (same arrangement and
  behaviour), Echo (same idea, our expression), Own (ours alone).
- **Stroke weights** are named on the one relative ladder that
  `visual-language.md` also uses, all constant on screen at every zoom
  (Match [R7]): *hairline* (the thinnest line the display shows crisply;
  grid lines, ruler ticks, handle stems), *thin* (one step heavier; the bed
  edge, chip outlines), *normal* (unselected shapes, boxes, guides, handle
  and marker outlines), *heavy* (about twice normal; selected and hovered
  shapes, hovered segments, the head marker and the origin marker's arms).
  There is no step above *heavy*. All weights follow the display's scale
  factor so they stay crisp at fractional scaling on X11 and Wayland
  (decision 12).
- **Dashes** come in two rhythms only: *dashed* (dashes about as long as a
  normal handle, equal gaps) and *dotted* (short dots, wider gaps). A third
  rhythm, *dash-dot*, is reserved for linked copies.
- **Layering** (back to front) is fixed: `canvas` surround → `bed` sheet →
  camera overlay (M9) → grid → `bed-edge` → shapes in drawing order (images
  first within their place in the order, M7) → guide-layer shapes →
  frame path → selection bounds and handles, nodes, start arrows, tab
  markers → snap and alignment feedback, drawing previews, measure lines →
  job origin marker → machine origin marker → head marker → rulers (always
  on top, outside the view area). Markers that carry safety meaning (head
  marker, frame path) are never hidden by artwork.

---

## Surround

### Reference

- The default canvas background is one flat, very light, almost white
  neutral grey that fills the whole view, inside and outside the bed; only
  the grid inside the bed tells the two apart [R1] [R7] [R14].
- An option switches the whole view to a medium-dark grey with slightly
  different grey grid lines and lighter ruler numbers; the toolbars, docks
  and swatch strip stay light [R1].
- There is no shading, vignette or texture [R1].

### LaserCraft

- The area of the view outside the bed is filled with `canvas`: a neutral
  grey a little darker than `bed`, so the bed reads as a sheet lying on a
  table even when no grid is drawn (Own: the reference uses one tone for
  both). The difference is small; it must not compete with shapes parked
  outside the bed.
- Flat fill, no gradient, shadow or texture.
- In the dark theme `canvas` goes dark and the bed does not (see "Dark
  theme").
- The reference's dark-background switch is not offered as a separate
  canvas setting: the theme choice covers it and the bed stays light
  (`naming.md`, Omitted; decision 13).
- Today the surround is egui's extreme background color of the current
  theme (`crates/lasercraft-ui/src/canvas.rs:229`), near-black on a dark
  desktop; the target uses `canvas` in both themes.

Tier: **Echo** (a plain flat surround), **Own** (a separate surround tone).
Milestone: M1 (restyle).

---

## Bed

### Reference

- The bed is a rectangle the size of the machine's work area, set from the
  device's width and height [R1] [R34].
- It is outlined by a thin, crisp, near-black line that stays dark in the
  dark-background mode [R1] [R15] [R7].
- The grid is drawn only inside it; the outside is plain background [R7]
  [R14].
- Imported pictures and artwork may sit outside the bed [R14].

### LaserCraft

- Placement: the work area (`project.set_work_area`, today default
  400 × 400 mm) as a rectangle with the rear of the bed at the top of the
  screen (`docs/spec/canvas.md`).
- Fill: `bed`, near-white in every theme (`docs/spec/canvas.md`: so black
  layer 0 stays visible).
- Edge: a *thin* solid line in `bed-edge` all round (the weight
  `visual-language.md` gives the token), drawn on top of the
  grid so the edge never disappears under grid lines that coincide with it.
- Fit: "Fit bed" (Ctrl+0, `view.zoom_fit`) shows the whole bed with a
  margin of about two `row`s of `canvas` on the tightest side (today a fixed
  screen margin, `canvas.rs:21`; the relative margin is own design and keeps
  room for the origin letters and the rulers).
- Shapes outside the bed are drawn exactly like shapes inside it; the bed
  never clips artwork. The job check (safety rule 5) is what refuses
  off-bed output, in the job preview and at Start (`job-preview.md`,
  `machine-panels.md`). Whether the canvas should also mark off-bed parts is
  an open question.
- When the bed is very small on screen (zoomed far out), the edge stays
  *thin* and the grid disappears (see Grid), so the bed reads as a
  small light card on `canvas`.

Tier: **Match** (a work-area rectangle, outline, grid only inside),
**Own** (separate fill token, margin).
Milestone: M1 (restyle).

---

## Grid

### Reference

- Thin, light-grey, solid major lines are drawn only inside the bed; a
  finer subdivision of ten by ten appears inside each major square as the
  user zooms in, much fainter, and at some zoom levels it reads as a faint
  dotted texture [R5] [R1] [R12] [R11].
- The spacing that is labelled changes with zoom; observed label steps
  include halvings and doublings, and labels are rounded for display so
  they can look uneven [R12] [R5] [R7] [R11] [R14].
- Settings set a drawn grid spacing, a separate (invisible) snap lattice
  spacing, and a contrast choice for the grid lines [R5].
- In the dark mode the lines are a muted grey close to the background [R1].

### LaserCraft

Levels and density (own design, proposed change to `docs/spec/canvas.md`,
which fixes 10 mm minor and 50 mm major lines today):

- Two levels are drawn at any zoom: **major** lines in `grid-major` and
  **minor** lines in `grid-minor`, both *hairline*, solid, only inside the
  bed.
- The minor spacing is chosen from the sequence 1, 2, 5, 10, 20, 50, 100 …
  millimetres (and 0.1, 0.2, 0.5 below one millimetre): the smallest value
  whose lines would still sit comfortably apart on screen (about one
  `icon-small` apart; the threshold in `docs/spec/canvas.md` stays the
  rule for "too close"). The major spacing is always ten minor steps
  (Echo [R5]: a ten-by-ten subdivision), so it is itself a value of the
  same 1, 2, 5 sequence and major lines always land on round numbers.
- As the user zooms in far enough, the minor step moves to the next
  smaller value of the sequence and the major spacing follows it (ten new
  minor steps); the new, finer minor level fades in. As the user zooms
  out, the minor step moves to the next larger value before the lines
  would crowd and the dropped level fades out. Both levels always come
  from the one sequence; no other spacing is ever drawn. The fade is a short change of strength, not
  an animation of position; with Reduce motion on (Display preferences,
  `naming.md`) the levels switch at once.
- Major lines always pass through the machine origin, so every major line
  is a round bed coordinate (see Rulers for the coordinate frame).
- When even the major spacing would crowd (the bed tiny on screen), no grid
  is drawn.
- Grid strength (Off, Faint, Medium, Strong; Grid and snapping preferences,
  later) scales the strength of `grid-minor` and `grid-major` together.
  Faint is the default. Off draws no grid but keeps the bed fill and edge.
- The grid has no labels of its own; the rulers carry the numbers.
- Snapping to the grid uses its own lattice (Snap spacing (mm), later; today
  the 10 mm lattice of `docs/spec/canvas.md`), independent of the drawn
  spacing (Match [R5]). The lattice is never drawn; while a drag is snapping
  to it, the snap feedback shows the point (see Snapping).
- With the camera overlay on (M9), the grid draws on top of the picture;
  only major lines are drawn, so the picture stays readable.

Tier: **Match** (two levels, finer level appears with zoom, grid only on
the bed, a separate snap lattice), **Own** (the round-number sequence, the
fade, the strength names).
Milestone: M1 (two fixed levels today, `canvas.rs:439` onward); adaptive
levels and Grid strength later.

---

## Rulers

### Reference

- Numbers run along all four edges of the view; the bottom and right rows
  repeat the top and left values [R1] [R7] [R12] [R33].
- There is no separate ruler band: small, regular-weight, medium-grey
  numbers sit in a narrow margin of the canvas background, each lined up
  with a major grid line; ticks are minimal or absent; left and right
  numbers are written horizontally, not rotated [R1] [R3] [R5] [R12].
- Values can be negative before the origin, and the direction of the
  numbers follows the machine's origin corner [R1] [R15] [R12] [R14] [R10].
- At the corners the first labels of the two rulers can overlap [R1] [R10].
- In the dark mode the margin is dark grey with light-grey numbers [R1].
- Hovering the top or left ruler changes the cursor, and dragging from it
  into the view makes a guide line [R6].

### LaserCraft

Placement (`layout.md`, "Canvas region"):

- Two ruler bands inside the canvas region: one along the **top** edge and
  one along the **left** edge. Each is a little taller (top) or wider
  (left) than one line of `type-small`. A small square fills the corner
  where they meet.
- No rulers on the bottom and right edges (Own; the reference repeats them
  there [R1]). Whether to offer them is an open question.

Look:

- Band fill `ruler`; numbers in `ruler-text`, `type-small`; a *hairline*
  `border` line separates each band from the view.
- Ticks (Own; the reference has almost none): a short tick at every minor
  grid step and a longer tick with its number at every major step, both
  `ruler-text` *hairline*. Ticks and numbers sit exactly over the grid lines
  of the bed and continue past the bed over the `canvas` surround, so
  positions outside the bed can still be read.
- Left-ruler numbers are written horizontally (Match [R1]), right-aligned
  against the band's inner edge, centred on their tick.
- Numbers are whole when the major step is whole and show only the
  decimals the step needs (0.5, 0.2, 0.1 …), never rounded so that steps
  look uneven (Own; fixes the uneven labels seen in the reference [R12]).
  Numbers carry no unit; the corner square shows the unit ("mm"; "in" once
  inch display exists, decision 14) in `type-small` `ruler-text`.
- Where two numbers would collide (near the corner square, or when a label
  is wider than the step), the one nearer the corner is dropped; labels
  never overlap (Own; the reference lets them overlap [R1]).

Coordinate frame (proposed change to `docs/spec/canvas.md`, shared with
`property-bar.md`):

- The rulers show **bed coordinates** measured from the machine-origin
  corner (`project.set_origin`, front-left by default), positive into the
  bed along both edges, exactly as the Pointer position status field and
  the X (mm) and Y (mm) fields do (`property-bar.md`, "Coordinate frame").
  With the origin at the front left (bottom left on screen), top-ruler
  numbers grow to the right and left-ruler numbers grow upward; with the
  origin at the rear right they grow leftward and downward; and so on
  (Match [R1] [R12] [R14]: the numbers follow the origin corner).
- Positions beyond the origin corner show negative numbers with a minus
  sign (Match [R1]).

Pointer and selection cues (Own):

- While the pointer is over the view, a *hairline* `selection` mark on
  each ruler shows its position (`selection`, not `accent`: decision 3
  keeps `accent` to its listed uses; open question 1).
- While something is selected, each ruler shows the selection's extent as a
  faint band of `selection` behind the ticks (open question 1 covers this
  canvas-family use).

Interactions:

- Hovering a ruler shows the vertical-resize (top ruler) or
  horizontal-resize (left ruler) cursor once guide lines exist (later); a
  drag into the view makes a guide line (see "Guide lines"). Until then the
  rulers do not react to the pointer apart from the wheel, which zooms as
  over the view.
- Clicking the corner square runs Fit bed (own design; tooltip "Fit the
  bed in the view (Ctrl+0)").
- Rulers never take keyboard focus.

Tier: **Match** (numbers on the edges, aligned to the grid, horizontal
numbers, direction from the origin corner, negatives, drag out guides),
**Echo** (two edges instead of four), **Own** (bands, ticks, cues, label
rules, corner square).
Milestone: later (not specced today; `docs/spec/canvas.md` needs the
section first).

---

## Machine origin marker and axis letters

### Reference

- A small filled square in a strong or dark red sits exactly on the bed
  corner that is the machine origin; two tiny letters, X and Y, in a muted
  or dark red, sit just past that corner along the horizontal and vertical
  bed edges [R1] [R15] [R7] [R11] [R12] [R26] [R25].
- It stays visible in the dark mode [R1].
- In some screenshots a small green square overlaps it; the coordinates
  page explains the green square as the job-origin marker, which coincides
  with the machine origin in the absolute placement mode [R26] [R33].

### LaserCraft (own design)

- Placement: the machine-origin corner of the bed (front-left by default;
  `canvas.rs:235-241` already picks the corner from `work_area.origin`).
- Shape: two short arms in `origin-marker`, *heavy* weight, starting at
  the corner and running a short way along the two bed edges **inside** the
  bed, each ending in a small arrowhead pointing toward positive values; a
  small filled square in `origin-marker` sits exactly on the corner where
  the arms meet. The arms are about one `icon-small` long and do not grow
  with zoom.
- Axis letters: "X" beside the tip of the arm along the horizontal edge,
  "Y" beside the tip of the arm along the vertical edge, both `type-small`
  bold in `origin-marker`, drawn inside the bed so they stay readable on
  `bed` in every theme. When the bed is so small on screen that the arms
  would cover a third of an edge, only the corner square is drawn.
- `origin-marker` (own choice, defined in `visual-language.md`): a strong
  dark slate, clearly darker and heavier than `bed-edge` but not black and
  not any semantic hue. It is never red: red means danger (decision 6),
  and the reference's red marker is its own expression.
- Tooltip when hovering the corner square (Select tool, nothing being
  dragged): "Machine origin (front left): where the controller's X0 Y0 is"
  with the corner in machine words (`naming.md`).
- It never moves except when the work area's origin corner changes.

Tier: **Match** (a marker on the origin corner with X and Y letters),
**Own** (arms, arrowheads, color, letters inside the bed).
Milestone: M1 (today a red-orange dot, `canvas.rs:34`, `canvas.rs:241`);
letters and arms with the restyle.

---

## Shapes

### Reference

- Unselected shapes are hairline strokes in their layer's color, and the
  stroke stays a hairline at every zoom [R10] [R6] [R7] [R5].
- Text is drawn as glyph outlines (or filled in the filled view style); a
  picture is drawn as the picture and may sit outside the bed [R6] [R8]
  [R14].
- A selected shape's own path is redrawn with a moving dash pattern in the
  layer's color instead of a solid line; a group uses a dot-dot-dash
  rhythm; the dash direction is the path's direction, which is also the
  default cut direction; overlapping duplicates make the dashes flicker
  [R3] [R4] [R9] [R10].
- How output-off or locked shapes look is not shown [R35].

### LaserCraft

States, from `docs/spec/canvas.md` and `docs/spec/layers.md`, in our look:

| State | Stroke | Fill (Show fills on) | Reacts to pointer |
|---|---|---|---|
| Normal | *normal* solid, layer color | fill layers: layer color at a lighter strength (`layers-panel.md`, "Layer colors elsewhere") | yes |
| Hovered (Select tool, see "Hover feedback") | *heavy* solid, layer color (lifted one step; no selection bounds, which is what tells it from Selected) | as Normal | yes |
| Selected | *heavy* solid, layer color, not recolored (`docs/spec/canvas.md`) | as Normal | yes |
| Selected, layer Output off | *heavy*, layer color faded | faded | yes |
| Layer Output off | *normal*, layer color faded to about a third of its strength (today's fade, `canvas.rs:253-262`) | faded | yes |
| Layer Show off (hidden) | not drawn | not drawn | no: never hit, never boxed (`docs/spec/shapes.md`) |
| Locked (later, proposed in `property-bar.md`) | as its other state | as its other state | selectable, but drags do nothing; bounds and handles in `text-disabled` |
| Guide layer (later, owner decision) | *normal* **dashed**, guide layer's color, never faded by Output (`layers-panel.md`) | never filled | yes |
| Linked copy (later) | *normal* **dash-dot**, layer color at a lighter strength | lighter still | no (see "Linked copies") |
| Being moved, scaled or rotated | drawn live at the dragged position (today's behaviour) with the same weight; the original position is not drawn | as Normal | — |

- Weights are relative to each other and constant on screen at every zoom
  (Match [R7]).
- Selected is shown by weight, not by a dash pattern on the path (existing
  spec wins; Own). The reference's moving dashes are echoed on the
  selection bounds instead (see "Selection bounds"); direction is shown by
  start arrows (see "Start arrows"). Whether selected paths should also
  march to show direction is an open question.
- Groups: every member of a selected group is drawn Selected; the group is
  also shown by its own bounds (one box around the whole group, see
  "Selection bounds"). No extra dash rhythm (Own; the reference uses one
  [R3]).
- Duplicates: two identical selected paths look like one. Delete
  duplicates (Alt+D, later) is the remedy; the canvas does not try to
  signal them (Own; the reference's flicker is a side effect we do not
  copy).
- Text shapes are paths today (`docs/spec/text.md`) and are drawn as
  outlines like any path; with Show fills on they fill when their layer is
  a fill layer.
- Images (M7): drawn as the picture inside their bounds, in grey or
  processed tones as `later-windows.md` and M7 decide; Output off fades the
  picture like a stroke; selected images show the selection bounds only
  (no heavier outline, since there is no stroke); hidden images are not
  drawn.
- Low-contrast layer colors (for example the bright yellows of the palette
  on the near-white `bed`, `layers-panel.md` open question 2): proposed
  (owner decision) that a layer color whose contrast against `bed` is below
  the WCAG 2.1 non-text minimum of 3:1 is drawn with a *hairline* darker
  edge on both sides of the stroke, in the same hue darkened, so the stroke
  stays visible without changing the palette data (decision 5).

Show fills and Smooth curves (View menu check items, `menus.md`):

- **Show fills** (later; Alt+Shift+W once keymap.md binds it, not bound
  today; keymap.md calls that chord "outline / filled", and this file
  proposes the label Show fills there): on draws closed shapes on fill layers with a fill of the layer
  color at a lighter strength under their stroke, in drawing order, so a
  fill hides what lies under it on screen (Match [R2]: the reference's
  filled style hides objects behind fills). Even-odd nesting is shown as it
  will burn (`docs/spec/layers.md`): rings stay unfilled. Off draws strokes
  only (today's look). Default: off, so every shape stays pickable; the
  reference notes the filled style makes hidden objects harder to pick
  [R2].
- **Smooth curves** (later): on flattens curves for display more finely,
  so curves look smooth at close zoom; off uses today's tolerance
  (`canvas.rs:23`), which is faster on weak machines. Default: on.
- Neither changes the job (Match [R2]); both are view state, not commands,
  and not saved in the project.
- Edge smoothing of strokes (antialiasing) is always on; egui does it at no
  extra cost, so there is no switch (Own; the reference has one [R1]).

Tier: **Match** (layer-color hairlines, constant weight, faded and hidden
layers, fills that hide what is behind them, display-only view switches),
**Echo** (selected by weight; groups by their bounds), **Own** (weights,
fill strength, low-contrast edge, no duplicate signalling).
Milestone: M1/M2 (today), Show fills and Smooth curves later, images M7.

---

## Selection bounds

### Reference

- Handles are laid out on the smallest upright rectangle around the whole
  selection, sitting a little outside the artwork with a small gap [R10]
  [R3] [R4] [R11].
- In some screenshots the box itself is not drawn as a line; only the
  handles mark it [R24] [R25].

### LaserCraft

- The selection bounds (`naming.md`: "Selection bounds") are the combined,
  axis-aligned bounds of the selection (`docs/spec/canvas.md`), drawn as a
  rectangle in `selection`, *normal* weight, **dashed**, offset outward
  from the exact bounds by about half an `icon-small` on every side so the
  line never sits on top of a selected stroke. Handles sit on this offset
  rectangle (Match [R10]: handles a little outside the artwork).
- The bounds use exact geometry bounds (`docs/spec/shapes.md`), including
  rotated ellipses, so a rotated shape gets an upright box around it.
- The dashes move slowly along the rectangle (clockwise on screen, a calm
  drift, not a flicker), echoing the reference's moving selection dashes
  (Echo [R3]). With Reduce motion on, or when egui reports that the
  system asks for reduced motion, the dashes stand still (Own; WCAG 2.1,
  2.3.3). The canvas repaints for this only while something is selected and
  the window has focus; with nothing selected the canvas repaints only on
  input, as today.
- `selection` (own choice, defined in `visual-language.md`): a deeper shade
  of the `accent` family, so it reads on `bed` and on every layer color and
  is never confused with a layer stroke (the palette's blues are tested
  against it in `visual-language.md`). It is the only use of the `accent`
  family on the canvas besides `marquee`, `drawing`, `node-selected` and
  the ruler cues (which use `selection` itself); this is the "canvas selection" exception decision 6
  leaves to this file.
- One selected object: one rectangle. Several: one rectangle around all of
  them (canvas.md: combined bounds). Group members do not get their own
  rectangles.
- During a move, scale or rotate drag the rectangle follows the preview;
  the handles not being dragged are hidden until release (Own; keeps the
  view calm).
- With the Edit nodes tool the selection bounds and handles are hidden
  (`docs/spec/canvas.md`).
- Locked selection (later): the rectangle and handles are drawn in
  `text-disabled`, solid (not moving), and the pointer shows the
  not-allowed cursor over the handles (proposed in `property-bar.md`).
- Reference-point cue (proposed in `property-bar.md`): while the pointer is
  over the Position and size toolbar's reference-point grid, or one of its
  fields has keyboard focus, a small cross in `selection` marks the chosen
  point of the bounds (one of the nine points: corners, edge middles,
  centre). Note that the cross is on the exact bounds, not on the offset
  rectangle, since the numbers refer to the exact bounds.

Tier: **Match** (handles around the upright bounds, slightly outside),
**Echo** (moving dashes, on the box instead of the paths), **Own** (drawn
box, color, offset, hidden handles while dragging).
Milestone: M1 (today an orange dashed box on the exact bounds,
`canvas.rs:364-368`); motion with the restyle.

---

## Handles

All handles share one look so the user learns one thing (Own):

- **Shape and size:** small squares (scale), a circle (rotate and move),
  small diamonds (skew), a little smaller than `icon-small`, the same size
  at every zoom. The Display preference Handle size (later, `naming.md`)
  scales all of them together, and also the nodes.
- **Fill and outline:** fill `handle-fill` (white), *normal* outline in
  `selection`. Never filled grey (Own; the reference uses flat mid-grey
  squares [R10] [R24]).
- **Hit area:** larger than the drawn handle by about half its size on every
  side, so small handles are easy to catch, but never larger than the gap to
  the next handle; where two handles' hit areas would overlap (very small
  selections), the one drawn on top wins in the order rotate, scale corner,
  scale edge, skew, move.
- **States:**

| State | Look | Cursor |
|---|---|---|
| Rest | `handle-fill`, `selection` outline | — |
| Hover | fill stays `handle-fill`, `selection` outline one step heavier (*heavy*); no `accent-soft` or `hover` fill, since `visual-language.md` keeps those to chrome | the handle's cursor (table per handle) |
| Pressed and dragging | fill `selection`, outline `selection`; other handles hidden; a drag read-out near the pointer (see "Drag read-outs") | the handle's cursor stays until release |
| Disabled by a handle switch (later) | not drawn; the gesture does nothing | — |
| Locked selection (later) | `handle-fill` with `text-disabled` outline | not-allowed |

- **Very small selections:** when the bounds on screen are smaller than
  about three handles across, only the four corner scale handles and the
  rotate handle are drawn; the edge, move and skew handles come back when
  the user zooms in (Own; keeps the selection grabbable).
- **Esc during any handle drag** cancels it: the selection returns to where
  it was and no command runs (Match [R10]; proposed change to
  `docs/spec/canvas.md`, which only says "releasing without movement
  changes nothing"). The selection stays selected (Own; the reference also
  clears it [R10]).
- **Handle switches** (Move, Scale, Rotate, Skew in the status bar, later,
  `layout.md`): turning one off hides that kind of handle on every
  selection and blocks its gesture for the whole project (Match [R10]).

### Scale handles

#### Reference

- Eight handles, at the four corners and the four edge middles of the
  bounds; hovering shows a two-headed arrow cursor; a corner scales both
  directions keeping proportions, an edge one direction, with the opposite
  side fixed; Shift on a corner frees the proportions, Ctrl/Cmd scales about
  the centre; Esc cancels [R10] [R4].

#### LaserCraft

- Eight squares on the selection bounds: four corners, four edge middles
  (`docs/spec/canvas.md`, already today, `canvas.rs:369-374`).
- Cursors: diagonal resize cursors on the corners, oriented to the corner
  (egui `CursorIcon::ResizeNwSe` / `ResizeNeSw`); horizontal and vertical
  resize cursors on the edge middles (`ResizeHorizontal` /
  `ResizeVertical`).
- Behaviour unchanged from `docs/spec/canvas.md`: corner keeps the aspect
  ratio unless Shift is held; edge scales one axis; Ctrl scales about the
  centre; dragging past the anchor mirrors.
- While Ctrl is held during a scale drag, a small cross in `selection`
  marks the centre being scaled about; otherwise a small cross marks the
  fixed opposite corner or edge middle (Own; shows the anchor).
- Tooltip after a short rest on a handle (no drag): "Drag to resize. Shift
  frees the proportions; Ctrl resizes from the centre." for corners;
  "Drag to stretch one way. Ctrl stretches from the centre." for edges.

Tier: **Match** (eight handles, gestures). **Own** (look, anchor cross).
Milestone: M1 (restyle).

### Rotate handle

#### Reference

- A curved two-headed arrow hugs the outside of each corner handle, four in
  all; the cursor becomes the same curved arrow; dragging turns the
  selection about its centre; Shift snaps to 15° and Ctrl/Cmd to 5°; Esc
  cancels; the status bar shows the angle change [R10] [R4].

#### LaserCraft

- One rotate handle, a circle above the middle of the top edge, joined to
  the top-middle scale handle by a *hairline* stem in `selection`
  (`docs/spec/canvas.md`: "a rotate handle above the top edge";
  `interaction.rs:19`). The stem is about one `icon-small` long on screen at
  every zoom.
- Cursor: a rotate cursor. egui has no rotate cursor, so the canvas hides
  the system cursor over this handle (`CursorIcon::None`) and paints its own
  rotate glyph (a short arc with an arrowhead at each end, `icons.md`,
  `cursor-rotate`) in `text` with a `handle-fill` halo. During the drag the
  glyph follows the pointer.
- During the drag: a *hairline* dotted line in `selection` from the
  selection centre to the pointer, a small cross at the centre, and the
  angle read-out (see "Drag read-outs").
- Behaviour unchanged from `docs/spec/canvas.md`: about the selection
  centre; 15° steps unless Ctrl is held (free rotation). The reference's
  Shift and Ctrl/Cmd steps are not adopted (keymap.md keeps the existing
  rule).
- Tooltip: "Drag to rotate about the centre. Turns in 15° steps; hold Ctrl
  to turn freely."
- Four corner rotate zones like the reference's are not added; one handle
  keeps the corners for scaling (Own; open question).

Tier: **Echo** (a rotate handle with its own cursor, rotation about the
centre), **Own** (one handle above the top edge, painted cursor, guide
line).
Milestone: M1 (today an orange ring on an orange stem,
`canvas.rs:375-378`); painted cursor with the restyle.

### Move handle

#### Reference

- A handle at the centre of the bounds; hovering shows a four-way-arrow
  cursor; dragging moves the selection; Shift locks the direction to 90° or
  45°, Alt/Option shows alignment guides, Ctrl/Cmd suppresses snapping; Esc
  before release cancels; a drag from any line or snap point of the
  selection also moves it, from that point [R10] [R4].

#### LaserCraft

- A small circle at the centre of the selection bounds with a tiny plus
  inside it, both `selection` on `handle-fill` (later; `naming.md`: Move
  handle).
- Cursor: `CursorIcon::Move` (four-way) over the move handle, and over the
  stroke or inside of any selected closed shape (today a drag on a selected
  shape already moves the selection, `docs/spec/canvas.md`).
- The move handle is the way to move a selection whose shapes are too thin
  or small to grab, and to move without starting on a snap point; a drag
  from a shape's stroke moves from the grabbed point, which then snaps
  (Match [R10]; today's rule snaps the nearest bounds corner or centre,
  `docs/spec/canvas.md`; proposed change there: a drag that starts on a
  snap point of the selection snaps that point).
- Modifiers unchanged from `docs/spec/canvas.md`: Shift keeps the move
  horizontal, vertical or at 45°; Ctrl turns snapping off; Alt shows
  alignment lines (later, see "Alignment lines"; proposed change to
  `docs/spec/canvas.md` and keymap.md's mouse section).
- Shift-constrained moves draw a *hairline* dotted `guide` line through the
  start point along the allowed direction while Shift is held (Own; the
  reference shows temporary guide lines for constrained node drags [R11]).
- Tooltip: "Drag to move. Shift keeps the direction straight; Ctrl turns
  snapping off."

Tier: **Match** (centre handle, cursor, gestures), **Own** (look).
Milestone: later (moving by dragging a selected shape is M1, today).

### Skew handles

#### Reference

- Two smaller handles in a slightly bluish grey sit near one corner, one
  along the horizontal edge and one along the vertical edge; the first
  slants the selection horizontally, the second vertically; Esc cancels;
  which corner they use differs between screenshots and is not explained
  [R10] [R3] [R4] [R11] [R24].

#### LaserCraft (later)

- Two small diamonds (squares turned 45°), slightly smaller than the scale
  squares, same `handle-fill` and `selection` outline: one on the top edge
  and one on the right edge of the bounds, each halfway between the
  top-right corner and that edge's middle handle. Always the top-right
  corner on screen, whatever the machine origin (Own; a fixed place is
  easier to learn than the reference's varying corner).
- Cursors: horizontal resize cursor for the top-edge diamond (it slants
  horizontally), vertical resize cursor for the right-edge diamond.
- Dragging slants the selection about the opposite edge; Shift snaps the
  slant angle to 15° steps; Esc cancels. Needs a skew command (proposed
  change to `docs/spec/shapes.md`: `shape.skew <x°> <y°> [ids]`, undone by
  `shape.set_transform`).
- Tooltip: "Drag to slant the selection sideways" / "Drag to slant the
  selection up or down".

Tier: **Echo** (two small slant handles near a corner), **Own** (diamonds,
fixed corner, Shift steps).
Milestone: later.

### Other handles (later)

- **Corner radius handle** (later; proposed change to
  `docs/spec/shapes.md`, rectangles have no corner radius today): a small
  circle in `handle-fill` with a `selection` outline, on the inside of the
  top-left corner of a single selected rectangle, a short way in along its
  diagonal. Dragging it inward rounds all four corners; the Tool hint shows
  the radius. Shown whenever exactly one rectangle is selected (Own; the
  reference shows its handle only while Ctrl/Cmd is held [R17]). Cursor:
  `CursorIcon::Crosshair`.
- **Arc handle** (later, text.md owner decision): a small circle above the
  middle of a single selected text shape, distinct from the rotate handle
  by sitting on the text's top edge rather than above the bounds. Dragging
  up or down bends the text along an arc; double-click straightens it
  (Match [R24]).
- **Polygon sides / star points** (later): no handle; the counts are
  Properties panel fields (`naming.md`). Own (the reference uses a
  Ctrl/Cmd handle [R17]).
- **Perspective warp and Mesh warp** (later): round handles as above at the
  four corners (warp) or on a four-by-four lattice joined by *hairline*
  `selection` lines (mesh), with the target outline previewed in `drawing`
  (Echo [R23]; own look).

### Drag read-outs

- Reference: while rotating, the status bar shows the angle change; while
  drawing a line it shows dx, dy and length [R10] [R12] [R16].
- LaserCraft (Own): the Tool hint status field always shows the live values
  (`layout.md`). In addition, during a move, scale or rotate drag a small
  chip next to the pointer (below and to the right, flipped to stay inside
  the view) shows the change: "Δx 12.5  Δy −3" for a move, "40 × 25 mm"
  (and "125 %" when proportional) for a scale, "+30°" for a rotate. Chip:
  `field` fill, *hairline* `border` outline, slightly rounded corners,
  `type-mono` at `type-small`, `text`. Values use bed coordinates' signs
  (see Rulers). The chip disappears on release or Esc.

---

## Selection boxes (Inside box and Touching box)

### Reference

- Dragging to the right draws a red box that selects only objects fully
  inside it; dragging to the left draws a green box that selects anything
  inside it or touched by it [R3] [R4].
- Shift adds, Ctrl/Cmd toggles, Ctrl/Cmd+Shift removes; a group counts as
  one object: inside needs the whole group, touching needs any part [R3].
- The boxes' line style and fill are not visible in the docs [R3].

### LaserCraft

- Gesture unchanged (`docs/spec/canvas.md`): dragging from empty space with
  the Select tool draws a selection box; left to right is the **Inside box**,
  right to left the **Touching box** (`naming.md`).
- Look (`marquee` token): both boxes have a *normal* `marquee` outline (the
  `accent` family) and a very faint `marquee` fill. The Inside box's outline
  is **solid**; the Touching box's is **dashed**. The two differ by line
  style only, never by hue (decision 6; Own: the reference uses two hues
  [R3]).
- While dragging, shapes that would be picked on release are lifted to
  *heavy* in their layer color (see "Hover feedback"), so the user sees
  the result before letting go (Own).
- Modifiers show in the Tool hint ("Adding to the selection", "Toggling");
  Shift adds and Ctrl toggles. keymap.md's mouse section and
  `docs/spec/canvas.md` give these modifiers for clicks only; extending
  them to selection boxes is a proposed change to canvas.md and keymap.md
  (mouse section). Ctrl+Shift to remove is not bound (keymap.md lists
  none); open question.
- A box shorter than about one `icon-small` in both directions counts as a
  click (today's click tolerance, `interaction.rs:21`).
- The box does not move the view; dragging to the view edge does not
  auto-scroll (Own; open question for long boxes).

Tier: **Match** (direction rule, modifiers, groups), **Own** (one hue,
solid vs dashed, live pick preview).
Milestone: M1 (today blue solid vs green dashed, `canvas.rs:386-405`).

---

## Hover feedback on shapes

### Reference

- Whether shapes highlight under the pointer with the selection tool is not
  stated; only cursor changes are described [R3] (unknown).
- The measuring tool marks the hovered shape in green and the hovered
  segment in purple [R18]; the trimming tool turns the piece that would be
  removed red [R20].

### LaserCraft (own design)

- **No chrome tints on the canvas.** `visual-language.md` keeps `hover`
  and `accent-soft` to chrome backgrounds and gives `danger` on the canvas
  only to the head marker while the laser may fire (and to error marks);
  hover and target feedback here therefore uses only line weight, line
  style and the shape's own layer color, plus the canvas tokens that
  `visual-language.md` lists (`selection`, `guide`, `text-muted`).
- **Select tool**: the topmost shape under the pointer (the one a click
  would pick, `docs/spec/shapes.md` hit rules) is lifted: its stroke is
  drawn *heavy* in its own layer color, as when selected, but without
  selection bounds or handles (those are what mark Selected). A hovered
  shape that is already selected shows no change. A hovered group lifts
  every member. Nothing is shown while a drag is in progress, over hidden
  layers, or with the pointer over a handle.
- **Measure tool** (later): the hovered shape is lifted the same way, and
  the single segment under the pointer is drawn *heavy* in `guide` (one
  meaning: measuring and snapping feedback are both `guide`, which
  `visual-language.md` lists for measure read-outs).
- **Trim tool** (later), behaviour Match [R20], look own design:

| Situation | Look | Tool hint |
|---|---|---|
| Pointer over a segment of a shape that crosses others | the piece a click would remove (from the hovered point back to the nearest crossing on each side) is drawn *heavy* **dashed** in the shape's own layer color; the rest of the shape keeps its *normal* stroke | "Click: remove this piece · Ctrl+click: remove without joining · Shift+click: move the piece to the current layer" |
| Pointer over a shape that crosses nothing | the whole shape is drawn *heavy* **dashed** in its layer color, since a click deletes it | "Click: delete this shape (it crosses nothing)" |
| Pointer over a group, or over a hidden or locked shape | no preview; cursor not-allowed | "Trim works on single shapes; ungroup first" (groups) |
| After a click | the piece is gone; where possible the pieces left over are joined at the old crossings into one path, which becomes the selection (drawn Selected with its bounds); a built-in shape becomes a path first | — |
| After Ctrl+click | as a click, but the leftover pieces stay separate paths (no join); they are selected together | — |
| After Shift+click | as a click, and the removed piece is kept as its own path on the **current layer** (`naming.md`); if that is the shape's own layer the status bar shows a short warning band "The kept piece is on the same layer" (`layout.md`) | — |

  The reference previews the removal in red [R20]; LaserCraft keeps red for
  danger and the head marker (`visual-language.md`), and a trim is an
  ordinary undoable edit, so the dashed heavy stroke in the layer color
  carries the meaning (Echo). The modifiers, the auto-join and the
  selected result are Match [R20]; they are a proposed change to
  `docs/spec/canvas.md` (Trim is not yet specified there; keymap.md lists
  Ctrl+K as later).
- **Edit nodes tool**: the node or segment under the pointer is the target
  of the node keys (`docs/spec/nodes.md`); see "Node editing".
- **Place tabs tool** (later): over a line-layer shape the pointer's
  position on the path shows a ghost tab marker in `text-muted`; over a
  shape on a fill or image layer, nothing is shown and the cursor is
  not-allowed (Match [R19]: only line-layer shapes take tabs).
- **Highlight the current layer's shapes** (Edit menu, later, `menus.md`):
  every shape of the current layer is lifted to *heavy* with a *hairline*
  `selection` outline just outside its stroke for about a second, then
  returns; with Reduce motion it appears and disappears without fading.
  The layer color is never changed.

Tier: **Match** (trim modifiers, join and selected result), **Echo**
(removal previewed by its own look; measure hover marks), **Own** (the
weight lift and everything else).
Milestone: later (no hover feedback today).

---

## Snapping

### Reference

- Dragged objects and drawing tools are pulled onto nearby points of other
  objects: nodes, midpoints, centres, intersections and points on lines;
  over each kind of point the arrow cursor is replaced by a small black
  glyph naming the kind [R5] [R16] [R11].
- Grid snapping jumps to the corners of a lattice set separately from the
  visible grid [R5].
- Alt/Option widens the snap reach and turns on alignment guides; Ctrl/Cmd
  flips snapping while held; snapping engages within a screen distance set
  in the settings [R5] [R10].

### LaserCraft

Snap targets (`docs/spec/canvas.md`, today): grid lattice points; other
shapes' bounds corners, edge middles and centres; the object point wins
over the grid. Later (proposed change to `docs/spec/canvas.md`): path
nodes, segment middles, intersections between paths, the nearest point on a
path, the centre of a path's bounds; the user's guide lines.

**Snap points** (`naming.md`), look (own design):

- When a drag or a drawing tool is snapping, a small marker in `guide`
  (teal, the only teal use) is drawn exactly on the snapped point, with a
  `handle-fill` halo so it reads on any stroke. Its shape names the kind:

| Snap kind | Marker shape (`guide`, about `icon-small` across) | Today |
|---|---|---|
| Grid lattice point | small plus | yes |
| Bounds corner | small square outline | yes |
| Bounds edge middle | small triangle outline pointing outward | yes |
| Bounds centre, path centre | small circle with a dot | yes (bounds) |
| Node | small filled square | later |
| Segment middle | small triangle outline on the segment | later |
| Intersection | small X | later |
| Point on a path | short bar across the path | later |
| Guide line | short bar across the guide line | later |

- The marker stays as long as the snap holds and disappears when the
  pointer leaves the snap distance (Snap distance (px), later; today the
  fixed radius of `docs/spec/canvas.md`).
- The target shape is not otherwise highlighted (Own).
- Ctrl held: no snapping and no markers (`docs/spec/canvas.md`).
- Alt held (later, with alignment lines): the snap distance is doubled
  (Match [R5]).

**Snap cursors** (`naming.md`; later): while a marker is shown, the
pointer's badge (see "Cursors") gains the same marker shape in `guide` at
its lower right, so the kind is visible at the pointer as well as at the
point (Echo [R5]: the cursor names the snap kind; our glyphs are our own
and are not the reference's circle-based cursors).

Snap to grid and Snap to shapes (View menu check items, `menus.md`; later)
turn each kind off; with both off Ctrl turns snapping **on** while held
(Match [R5]).

Tier: **Match** (snap kinds, separate lattice, Ctrl to suspend, Alt to
widen), **Echo** (a glyph per kind), **Own** (markers at the point, shapes,
`guide` color).
Milestone: M1 snapping exists with no feedback; markers later.

---

## Alignment lines (later)

### Reference

- Holding Alt/Option while dragging a selection or drawing a shape shows
  temporary guide lines where the selection's edge or centre lines up with
  another object's edge or centre, and the selection snaps there [R6] [R5].
- Their color and style are not shown [R6] (unknown).

### LaserCraft

- Shown only while Alt is held during a move drag or a drawing drag
  (Match [R6]); Alt alone, without a drag, shows nothing.
- Each alignment is a *hairline* solid `guide` line running from the
  matching edge or centre of the moving bounds to the same edge or centre
  of the other object, extended a short way past both, with a small tick
  across each end. Centres are marked by a small circle on the line.
- At most one vertical and one horizontal alignment line are drawn at a
  time: the nearest. The selection snaps to that alignment (it counts as an
  object snap).
- Equal-spacing cues (two gaps equal) are not drawn (Own; open question).

Tier: **Match** (Alt while dragging, edges and centres, snap), **Own**
(look).
Milestone: later (proposed change to `docs/spec/canvas.md` and keymap.md's
mouse section).

---

## Guide lines (later)

### Reference

- Dragging from the top ruler makes a horizontal line and from the left
  ruler a vertical one; they can be moved, snapped to and locked later;
  they are ordinary objects on the first tool layer, never output, but they
  count in the job bounds of relative placement modes unless removed or
  excluded [R6].
- Their color and line style are not shown; the tool layer's swatch is an
  orange-red [R6] [R1] (unknown).

### LaserCraft

- Dragged out of the top or left ruler (see "Rulers"); while dragging, the
  line follows the pointer across the whole view with a chip showing its
  coordinate in bed coordinates ("Y 120.0"); released over the ruler it is
  discarded.
- Each guide line is a straight shape on guide layer 1 (owner decision,
  `layers-panel.md`), as long as the bed plus the visible surround, drawn as
  every guide-layer shape: *normal*, **dashed**, in the guide layer's own
  color (not the `guide` token: `guide` is transient feedback, guide layers
  are lasting helper shapes; `visual-language.md` keeps the two hues
  apart).
- Guide-layer shapes are never output, never previewed and never framed,
  and they are **not** counted in the job bounds in any placement mode
  (Own; removes the reference's pitfall [R6]; proposed change to
  `docs/spec/layers.md` and `docs/spec/planning.md`).
- They snap like paths and can be selected, moved, locked and deleted.
- Cursor over a ruler: the vertical-resize cursor (top ruler) or the
  horizontal-resize cursor (left ruler).

Tier: **Match** (drag from rulers, horizontal or vertical, ordinary helper
objects never output), **Own** (look, bounds rule, coordinate chip).
Milestone: later, after guide layers and rulers.

---

## Drawing in progress

### Reference

- Shapes grow from the first corner as the user drags (Ctrl/Cmd from the
  centre, Shift for the regular form); the new shape is selected and its
  handles work at once while the tool stays active [R17].
- While a line is being built, the segment being placed follows the
  pointer as a thin solid line in the currently picked palette color; a
  dragged curve handle is a thin light line ending in a small square; Shift
  keeps new points at 0°, 45° or 90°; the status bar shows the pointer, dx,
  dy, length and angle [R16].
- Text being typed is drawn as thin outlined letters [R24].
- Tool windows such as the offset window draw their result on the canvas
  while open: the original stays dashed (selected) and the result is drawn
  in thin solid dark lines; other tools preview copies while their options
  change [R29] [R28].

### LaserCraft

Common look (`drawing` token, `accent`, dashed):

- Every not-yet-created shape is drawn *normal* **dashed** in `drawing`. It
  takes the layer color only once created (Own; the reference draws it in
  the picked palette color [R16]; ours keeps one clear "not created yet"
  look and avoids invisible previews in pale layer colors). The current
  layer color is shown in the cursor badge instead (see "Cursors").
- A drawing that would create nothing (smaller than the minimum of
  `docs/spec/canvas.md`) is drawn in `text-muted` dashed, so the user sees
  why release does nothing.
- The Tool hint status field shows the live values (`layout.md`).

Per tool:

| Tool | While in progress | Extra marks |
|---|---|---|
| Rectangle | the rectangle from the press point to the pointer | with Ctrl: a small cross in `drawing` at the press point, which is the centre; with Shift: nothing extra (the shape is square); Tool hint "W × H mm" |
| Ellipse | the ellipse in the same box | as Rectangle (centre cross with Ctrl); Tool hint "W × H mm" |
| Shape tools (later: Triangle, Pentagon, Polygon, Octagon, Star, Alternating star) | the shape in the same box | as Rectangle |
| Line | placed segments solid `drawing`; the segment from the last point to the pointer dashed `drawing`; placed points as small `node` squares; the first point as a `start-marker` square | when the pointer is close enough to the first point to close the shape, a ring in `guide` around it; with Shift a *hairline* dotted `guide` ray along the allowed direction; Tool hint "Δx … Δy … length … angle …" |
| Line, starting on the end of an open path (later) | before the first click, when the pointer is within snap distance of the start or end node of an open path on a shown layer, that end node is drawn as a `node` square with a *normal* `guide` ring; clicking there continues that path instead of starting a new one (Match [R16]); the continued path keeps its layer, and its placed segments preview as Line | Tool hint "Click: continue this path" |
| Line, ending on the end of another open path (later) | while placing, when the pointer is within snap distance of an end node of a different open path, that end gets the same `guide` ring; clicking there joins the two paths into one, which is selected when the tool finishes (Match [R16]) | Tool hint "Click: join to this path · Ctrl+click: end here without joining"; with Ctrl held the ring is not drawn and the click only places a point (Match [R16]: Ctrl/Cmd stops the automatic join) |
| Line, finishing | double-click or Enter finishes the open path (`docs/spec/canvas.md`); clicking the first point closes it; right-click also finishes (Match [R16]; proposed change to `docs/spec/canvas.md`, which lists only double-click and Enter); Esc cancels | — |
| Line, corner and smooth placement (later, when Line makes curves) | the S key switches whether the next click places a corner or a smooth point (Match [R16]); the cursor badge changes with it (see "Cursors"); a smooth point previews the bent segment live as dashed `drawing` | Tool hint "S: smooth points" or "S: corner points" (the choice the key would switch to). S is a tool-local key while the Line tool is drawing, like the node keys; it is not in `default.keymap` and is a proposed change to `docs/spec/keymap.md` and `docs/spec/canvas.md` |
| Line, dragging a curve handle (later, when Line makes curves) | the handle as a *hairline* `text-muted` line from the point to a small circle in `handle-fill` with `selection` outline | as Line |
| Text | after the click, a small cross in `drawing` at the click point until the Add text window closes | the window's text appears only when created (no live text preview today; later preview proposed) |
| Tool window previews (Offset…, Grid array…, Circular array…, later Path array…, Combine shapes…, Perspective and Mesh warp) | the result drawn *normal* **dashed** `drawing`; the original keeps its Selected look and its selection bounds | the preview redraws on every field change; OK creates it, closing runs nothing (`docs/spec/ui-shell.md`); proposed change to `docs/spec/ui-shell.md` and `dialogs.md` (today no previews) |
| Add tangent circle (later) | candidate circles *normal* dashed `drawing`; the hovered one *heavy* (Echo [R30]) | click creates it |
| Two-point rotate and scale, Mirror across a line (later) | the pivot point as a small cross in `guide`; a dashed `guide` line from it to the pointer; the selection drawn live | prompts in the Tool hint (Match [R14]) |

- Esc cancels any drawing in progress (`docs/spec/canvas.md`); the preview
  disappears at once.
- After a finished drawing the new shape is selected and drawn Selected
  with its selection bounds while the tool stays current
  (`docs/spec/canvas.md`).

Tier: **Match** (growing from the press point, modifiers, live values,
previews of tool windows), **Own** (one `drawing` look instead of the
palette color, too-small look, closing ring, centre cross).
Milestone: M1 (rectangle, ellipse, line exist today in green,
`canvas.rs:412-434`), M3 (tool windows; previews proposed), later (the
rest).

---

## Node editing

### Reference

- In node mode the path is drawn with the selection's dash pattern; corner
  nodes are small squares in a muted reddish-brown, smooth nodes small dots
  in a muted teal, the start node a slightly larger green square (a green
  circle if smooth) with a dark outline, and curve handles are thin light
  blue lines through a node, each ending in a small blue-violet square; the
  usual selection handles are not shown [R11] [R16].
- Nodes and handles snap with the normal snap cursors; Shift constrains a
  drag to 90° or 45° and shows temporary guide lines [R11].

### LaserCraft

Behaviour is `docs/spec/nodes.md`'s; this is the look (own design, tokens
`node`, `node-selected`, `start-marker`):

| Element | Look | Hover | Selected (later: click to select nodes) |
|---|---|---|---|
| Path being edited | its normal layer-color stroke at *heavy* weight (as Selected) | the segment under the pointer gets a *heavy* `selection` overlay (it is the target of D, I, M, L, C and B) | — |
| Corner node | small square, `node` (white fill, dark outline), a little smaller than `icon-small` | outline `selection`, one step heavier | filled `node-selected` |
| Smooth node (later, with smoothing) | small circle, `node` | as corner | filled `node-selected` |
| Start node of a subpath | same shape, slightly larger, filled `start-marker` (the first node of every subpath, `docs/spec/nodes.md`) | outline `selection` | `node-selected` ring around the `start-marker` fill |
| End node of an open subpath | the corner or smooth shape with a *heavy* outline, so open ends are easy to find (join targets) | as corner | as corner |
| Control handle line | *hairline* solid in `text-muted` from the node to the handle end | — | — |
| Control handle end | small circle, smaller than a node, `handle-fill` with a `selection` outline | `selection` outline one step heavier | fill `selection` |
| Node or handle being dragged | fill `selection`; the old position is not shown; the path redraws live (today, `canvas.rs:265-270`) | — | — |
| Join target (dragging an end node near another end) | the other end gets a ring in `guide` | — | — |

- Cursor: `CursorIcon::Default` in general; `Move` over a node or a handle
  end; `Crosshair` over a segment (where I and M act); the node keys show
  in the Tool hint while the pointer is over a node or segment ("D delete
  node · B break here" or "D delete segment · I add node · M add at middle
  · L straighten · C make curve").
- Shift-constrained drags draw the dotted `guide` ray (as for moves).
- Snapping shows the snap markers (see "Snapping").
- The selection bounds and handles are hidden in this tool
  (`docs/spec/canvas.md`). Selected shapes that are not paths show no
  nodes; they keep their Selected stroke and the Tool hint says "Convert
  the shape to a path first (Ctrl+Shift+C)".
- Node tools buttons that arm an action (later, `toolbars.md`) add that
  action's badge to the cursor (see "Cursors").
- Every node and handle has the same hit-area rule as the handles.

Tier: **Match** (squares and dots, a marked start node, handle lines and
ends, no selection handles in this mode, Shift guides), **Own** (tokens,
open-end emphasis, join ring, hover targets).
Milestone: M3 (today orange squares and rings, `canvas.rs:334-348`);
restyle with this file.

---

## Start arrows and the Start point tool (later)

### Reference

- With the start-point tool on, the selected shape shows its nodes and an
  arrow at its start: grey for the default start, blue for a start the user
  chose; clicking a node moves the start, Shift-click reverses the
  direction, Ctrl/Cmd-click resets both; the default start is the first
  node and the default direction follows the selection dashes [R9].
- The arrow's exact shape is not shown [R9] (unknown).

### LaserCraft

- **Start arrow** (`naming.md`): a small solid triangle arrowhead in
  `start-marker`, sitting on the path just after the start node and
  pointing along the cut direction, with a small ring in `start-marker`
  around the start node itself. About `icon-small` long.
- Default start (the first node, `docs/spec/nodes.md`) vs chosen start:
  the default arrow is drawn as an **outline** triangle, a chosen one
  **filled** (Own: fill tells the two apart, not hue; one hue per meaning,
  decision 6).
- `start-marker` (own choice, defined in `visual-language.md`): a dark
  violet, distinct from `accent`, `selection`, `guide`, the semantic hues
  and the palette's darkest violet (tested in `visual-language.md`). Also
  used for the start node in node editing and in line drawing.
- Shown: for every selected shape while the Start point tool is current;
  nowhere else on the canvas (Own). The job preview shows the real order
  and starts (`job-preview.md`).
- Interactions (Match [R9]): click a node to move the start; Shift-click to
  reverse the direction; Ctrl-click to reset both. The cursor over a node
  is the crosshair with the start-point badge; elsewhere the default
  arrow. Needs commands (proposed change to `docs/spec/planning.md` and
  `docs/spec/nodes.md`: a per-subpath chosen start and direction the
  planner keeps when Cut order allows it).

Tier: **Match** (arrow at the start, default vs chosen, click gestures),
**Own** (shape, fill rule, color, where it shows).
Milestone: later.

---

## Tab markers (later)

### Reference

- The tab tool's cursor is a plus with a small tab glyph; a placed tab is a
  small round pinkish-red dot with a dark outline on the path, which can be
  dragged along the shape or onto another shape and removed with a
  double-click or Shift-click; only line-layer shapes take tabs; the result
  illustration shows faint gaps in the cut [R19].

### LaserCraft

- **Tab markers** (`naming.md`; hand-placed tabs only; `naming.md`'s row
  reads "Marks where hand-placed tabs sit", a short bar across the path): a
  short thick bar
  across the path at the tab's centre, about as long as a node is wide, in
  `text` with a `handle-fill` halo, plus a *hairline* gap in the stroke
  either side showing the tab width at the current zoom (when the width is
  large enough on screen to see).
- Hover: the bar gains a *normal* `selection` outline around its
  `handle-fill` halo and the cursor becomes `Move`;
  dragging slides the tab along the path (and onto another line-layer
  shape, Match [R19]); Shift-click or double-click removes it (Match
  [R19]).
- Automatic tabs (`tabs per shape`, `docs/spec/layers.md`) are **not**
  drawn on the canvas: the planner places them, and the job preview shows
  them as unlit moves (`job-preview.md`). Open question whether to show them
  faintly.
- Tab markers are drawn only when their layer is shown and in line mode;
  on an Output-off layer they fade with the layer.

Tier: **Echo** (a marker on the path, drag and remove gestures), **Own**
(bar shape, color, automatic tabs not drawn).
Milestone: later (Ctrl+Tab once bound; keymap.md: later).

---

## Measure (later)

### Reference

- Hovering shows measurements of the shape and segment under the pointer
  (shape marked green, segment purple) in a small window; dragging between
  two points, snapped or free, measures a temporary line; Shift keeps it at
  0°, 45° or 90° [R18].

### LaserCraft

- Hover: as in "Hover feedback" (the shape lifted to *heavy* in its layer
  color, `guide` on the segment); the Measurements dialog (`dialogs.md`) shows the numbers
  (Match [R18]).
- Drag: a *normal* solid `guide` line from the press point to the pointer,
  with a short tick across each end and a small chip at its middle
  (`field`, *hairline* `border`, `type-mono` at `type-small`) showing
  "length mm · angle°"; snapping applies at both ends; Shift keeps the line
  at 0°, 45° or 90°. The measure line stays on screen until the next press
  or Esc, and is never a shape.
- The Tool hint shows the same values as the chip.

Tier: **Match** (hover and drag measuring, snap, Shift), **Own** (look).
Milestone: later (Alt+M once bound; keymap.md: later).

---

## Head marker (M6)

### Reference

- A button in the laser panel places a crosshair on the canvas where the
  head was last reported; it moves when the user jogs from the app but is
  deliberately not updated during a job, so the user watches the machine
  [R13].
- The canvas crosshair itself is not pictured [R13] (unknown). A small
  green square seen on canvases is the job-origin marker [R26].

### LaserCraft

- **Head marker** (`naming.md`): a crosshair in `head-marker`, *heavy*
  weight, with a small open circle at its centre and four arms with a gap
  at the middle so the exact point stays visible; about twice `icon-small`
  across, the same size at every zoom; drawn on top of everything except
  the rulers.
- Shown while Show head marker is on (`machine-panels.md`; default on).
  While connected it shows the reported position; with no connection, or
  while status replies have stalled, it stays at the last known position
  in `text-disabled` (the rule `machine-panels.md` gives for its switch). It follows GRBL status reports (Own: a live marker; the
  reference's one-shot button [R13] is not copied). The position mapping
  from machine to bed coordinates is the app's (the UI receives the bed
  point through `lasercraft_device::Event` handling in the app, never
  computes it).
- States (token meanings fixed in `visual-language.md`: neutral vs danger):

| Machine state (`naming.md`) | Look |
|---|---|
| Ready, Jogging, Homing, Paused (laser off, M4 job), Door open, Checking, Sleeping | `head-marker` (dark neutral) |
| Running, Framing at low power (later), Paused during a constant-power (M3) job | `danger`, with a faint `danger-soft` disc behind the circle: the laser may be firing (decision 7, safety rule 9). See the note below the table for the M3 pause. |
| Framing (laser off) | `head-marker`, as Ready (framing is laser-off by default, safety rule 2) |
| Alarm | `head-marker` with a small `danger` ring around the circle; the position may be unknown after an alarm, so the ring means "not trusted" |
| No response (stalled), Not connected | the last known position in `text-disabled`, no disc, no ring, so it never reads as current; the Machine panel state line says why (the job bar is hidden while not connected) |

- Note on the M3 pause: the job bar and Machine panel show that state with
  `warning-soft` and "Pause (motion only)" (`machine-panels.md`), because
  the controls say "the laser may stay on". The head marker deliberately
  follows its own token rule instead: `head-marker` is drawn in `danger`
  whenever the laser may be firing (`visual-language.md`), and with
  constant power a held laser can still be lit (GRBL laser mode). So the
  chrome warns and the marker on the bed shows the hazard where it is.
  `visual-language.md` should confirm this (open question).

- The marker never blinks or pulses (Own; the user should watch the
  machine, which is the reference's stated aim [R13]; with a live marker we
  keep it calm instead of hiding it).
- If the reported position is off screen, a small arrow in the same color
  sits at the view edge pointing toward it (Own).
- Tooltip when hovered: "Laser head: X … Y … mm" in bed coordinates.
- It is not a handle: it cannot be dragged. Moving the head is the Move
  head tool or the Jog panel (`machine-panels.md`).

Tier: **Echo** (a crosshair for the head's position), **Own** (live,
states, colors, off-screen arrow).
Milestone: M6.

### Move head tool (M6)

- Reference: clicking the canvas sends the head to that point; the tool
  needs a homed machine and switches itself off after about ten seconds
  [R22].
- LaserCraft (`toolbars.md` owns the one-shot rule): the cursor is the
  crosshair with the `badge-move-head` badge (the head ring, `icons.md`);
  under the pointer a ghost head marker in `text-muted` shows where the
  head would go; outside the work area the ghost is drawn outline-only in
  `text-disabled`, the cursor is not-allowed and the Tool hint says
  "Outside the work area: the head cannot go there" (the move would be
  refused, safety rule 5 in spirit; no `danger`, which on the canvas
  belongs to the head marker while the laser may fire). After the click the tool
  returns to Select and the real head marker moves as reports arrive.

---

## Job origin marker (M6)

### Reference

- A small green square marks the point on the bed where the job's chosen
  reference point will land; in the absolute placement mode it sits on the
  machine origin, and it moves when the reference point, the
  selected-output switch or the selection-origin switch change [R26] [R13]
  [R33].

### LaserCraft

- **Job origin marker** (`naming.md`): a small ring with a cross through
  it, about `icon-small` across, *normal* weight, in `origin-marker` with a
  `handle-fill` halo (Own: the machine origin marker is filled, the job
  origin marker is hollow, so the two read apart by shape in one color;
  green is our success hue and is not used, decision 6). There is no
  separate token: `visual-language.md` defines `origin-marker` as covering
  the job origin marker in its hollow form.
- Where: with Place job at "Current position", on the head's position; with
  "User origin", on the stored user origin (`machine-panels.md`). The
  Job reference point (`naming.md`) decides which point of the job's bounds
  lands there, and a *hairline* dotted `origin-marker` rectangle shows the
  job's bounds at that placed position (Own: the user sees where the job
  will go before framing).
- With "Absolute coordinates" no job origin marker and no placed bounds are
  drawn: the design already sits where it will burn (Own; the reference
  draws it on the machine origin [R26]).
- Selected shapes only and Reference point from selection change which
  bounds are used (`naming.md`); the marker and placed bounds move at once.
- If the placed bounds leave the work area, the placed rectangle stays
  *hairline* dotted `origin-marker`, the part outside the bed is drawn
  *normal* **dashed** `origin-marker`, and a `mark-error` (`icons.md`; an
  error mark, one of `danger`'s listed uses) sits at the bed edge where
  the bounds cross it; Start in the Machine panel and the Machine menu
  is refused, with the Machine panel's Job refused band saying why
  (safety rule 5; `machine-panels.md`). The job bar has no Start.
- Tooltip: "Job origin: the job's ‹reference point› will be placed here".

Tier: **Match** (a marker where the job's reference point lands, moving
with the placement settings), **Own** (shape, color, placed bounds, hidden
in absolute mode).
Milestone: M6.

---

## Frame path (M6)

### Reference

- The docs show the path a frame will trace as a thin bright green outline
  around the design on the canvas: a rectangle for the bounding-box frame,
  a faceted hull for the tight-outline frame [R13] [R27].

### LaserCraft

- **Frame path** (`naming.md`): the outline the head will trace, drawn
  *normal* **dashed** in `frame-preview` (amber, `warning` family: "the
  machine is about to move or is moving").
- When shown:
  - while the pointer is over Frame or Frame outline (Machine panel or
    Machine menu; the job bar has no Frame), so the user sees the path before clicking
    (Own);
  - for the whole time framing runs, together with the head marker;
  - during Repeat framing (`naming.md`, device profile) until it is
    stopped.
- Frame: the job's bounding rectangle at its placed position. Frame
  outline: the tight convex outline around the job (`naming.md`). Both
  use the same planned job the preview and Start use (computed by the app,
  never by the UI).
- Laser-off framing (the default and the only framing in M6, safety rule
  2): dashed. Low-power framing (later, opt-in per use, owner decision):
  the path is drawn **solid** `warning` and the head marker is `danger`
  while it runs, and the Machine panel state line and the Low-power frame
  dialog show the power (safety rule 2, decision 7).
- A frame that would leave the work area is refused before it starts
  (safety rule 5); while the pointer is over the button the frame path is
  still drawn in `frame-preview` dashed, with a `mark-error` at the bed
  edge where it crosses (an error mark), and the button's tooltip says
  why.

Tier: **Echo** (the frame path drawn on the canvas), **Own** (color,
dashes, when it shows, the low-power look).
Milestone: M6.

---

## Linked copies (later)

### Reference

- Clone copies made by the grid-array window's linked option are drawn
  with dashed outlines and paler, washed-out fills; they cannot be
  selected or edited, and edits to the original show in all of them [R28].

### LaserCraft

- **Linked copies** (`naming.md`; later, proposed change to
  `docs/spec/arrange.md`): drawn in the layer color at a lighter strength,
  *normal* weight, with the **dash-dot** rhythm reserved for them; with
  Show fills on, their fills are lighter still (Match [R28]: dashed and
  paler).
- They are not hit by clicks or boxes (Match [R28]); hovering one lifts its
  **original** to *heavy* (as in "Hover feedback") and the Tool hint "Linked copy of
  ‹shape›: edit the original". Selecting the original draws a *hairline*
  dotted `selection` box around each of its copies (Own), so the set is
  visible.

Tier: **Match** (paler, dashed, not selectable), **Own** (dash-dot, links
shown on the original's selection).
Milestone: later.

---

## Camera overlay (M9)

### Reference

- A captured camera picture is painted under the bed grid as a background;
  lowering its opacity fades it so the artwork reads better; its size and
  position can be adjusted [R31] [R1].

### LaserCraft

- **Camera overlay** (`naming.md`): the picture is drawn over the `bed`
  fill and under the grid, clipped to the bed (Match [R31]). Overlay
  opacity, size and offset are Camera panel controls (`later-windows.md`).
- With the overlay on, the grid draws only its major lines, and every
  shape stroke gets a thin `bed`-colored halo underneath so layer colors
  read on any picture (Own).
- A stale picture (taken before the last jog or job) is not marked on the
  canvas; the Camera panel says when it was taken (Own; open question).
- The overlay is never part of the job, never framed and never previewed.

Tier: **Match** (picture under the grid, opacity), **Own** (halos, major
lines only).
Milestone: M9.

### Registration targets (M9)

- **Registration targets** (`naming.md`): each target as a ring with a
  cross, larger than the job origin marker, in `text`, numbered "1" and "2"
  in `type-small`; a target not yet set is drawn dashed in `text-muted`,
  one set solid; the target being picked follows the pointer in `drawing`.
  Own design; the wizard is `later-windows.md`'s.

---

## Cursors (own design)

### Reference

- Cursors are small black monochrome glyphs: arrow, four-way arrow,
  two-way arrow, curved arrow, open hand, and the snap glyphs [R5] [R7]
  [R10]; the line, text and rounding tools use a thin crosshair with a small
  hollow centre and a small colored badge at the lower right that shows
  the mode, with a red no-room mark when rounding is impossible [R16] [R24]
  [R21]; trimming shows scissors and the tab tool a plus with a tab glyph
  [R20] [R19].

### LaserCraft

- System cursors come from egui's `CursorIcon` set; LaserCraft's own badges
  are painted next to the pointer on the canvas (a small rounded `field`
  chip with a *hairline* `border` outline and the badge glyph in `text`, at
  the pointer's lower right, about `icon-small`). Badges are drawn only
  over the view, never over the rulers. Badge artwork is `icons.md`'s (ids
  `badge-…`).
- The badge of every drawing tool also shows a small square of the
  **current layer color** (the color new shapes will get), so the user sees
  the target layer without looking at the palette (Own; replaces the
  reference's palette-colored line preview [R16]).

| Tool or situation | System cursor | Badge |
|---|---|---|
| Select, over empty canvas | `Default` | none |
| Select, over an unselected shape | `PointingHand` | none |
| Select, over a selected shape or the Move handle | `Move` | none |
| Over a scale handle | `ResizeNwSe` / `ResizeNeSw` / `ResizeHorizontal` / `ResizeVertical` | none |
| Over the rotate handle | `None` (painted rotate glyph, see "Rotate handle") | — |
| Over a skew handle (later) | `ResizeHorizontal` / `ResizeVertical` | none |
| Over a locked selection's handles (later) | `NotAllowed` | none |
| Line | `Crosshair` | `badge-line` + layer color; when Line makes curves (later), a corner or smooth variant of the badge shows which kind of point the next click places, switched with S (Match [R16]: the badge shows the mode; the variant artwork is `icons.md`'s, proposed there) |
| Rectangle, Ellipse, Shape tools | `Crosshair` | the shape's badge (`badge-rect`, `badge-ellipse`, …, `icons.md`) + layer color |
| Text | `Crosshair` | `badge-text` (the `text-tool` icon; no letterform) + layer color |
| Edit nodes | see "Node editing" | an armed node action's badge (later, `icons.md` lists them) |
| Trim (later) | `Crosshair`; `NotAllowed` over groups | `badge-trim` (the `trim` icon; not scissors, which mean Cut) |
| Place tabs (later) | `Crosshair` | `badge-tab`; `NotAllowed` over shapes that cannot take tabs |
| Start point (later) | `Crosshair` over nodes, `Default` elsewhere | `badge-start-point` |
| Round corners (later) | `Crosshair` | `badge-round-corner`; `badge-sharp-corner` over an already rounded corner; `badge-no-room` in `warning` when the radius does not fit (Match [R21]: a no-room mark; `warning`, not `danger`, as `icons.md` decides) |
| Measure (later) | `Crosshair` | `badge-measure` (the caliper) |
| Move head (M6) | `Crosshair`; `NotAllowed` outside the work area or when not connected | `badge-move-head` (the head ring; not a pin) |
| Pan tool, or Space held, or middle button | `Grab`; `Grabbing` while dragging | none (Match [R7]: open hand) |
| Over a ruler, guide lines exist (later) | `ResizeVertical` / `ResizeHorizontal` | none |
| Snapping | unchanged | the snap kind's marker in `guide` added to the badge (see "Snapping") |
| Busy (the app is planning a large job, later) | unchanged | none (the job preview shows the busy state) |

- Cursor size follows the system; the Display preference Handle size
  (later) scales the badges with the handles.

Tier: **Match** (crosshair plus a mode badge for drawing tools, resize and
move arrows, open hand for panning), **Own** (badge chip, layer color
square, every glyph).
Milestone: later (today there are no cursor changes per tool,
current-ui.md 1.11); system cursors could come first, badges with the
icons.

---

## Zoom and pan (look)

### Reference

- The wheel zooms about the pointer (direction can be inverted); + and −
  zoom when the canvas has focus; a fit-the-bed command and a
  zoom-to-selection command; pan with a toolbar pan mode, the middle
  button, or Space and drag, with an open-hand cursor; grid density and
  ruler labels change with zoom and line weight stays a hairline [R7]
  [R8].

### LaserCraft

- Behaviour stays `docs/spec/canvas.md`'s (wheel about the pointer; Ctrl+=,
  Ctrl+- and Ctrl+0; middle button or Space and drag; zoom limits).
  Zoom to selection is later (`naming.md`; its chord is an open question in
  `toolbars.md`, keymap.md reserves Ctrl+Shift+A (M6, legacy) for
  framing the selection; it is not in `default.keymap`). The Pan tool is proposed in `toolbars.md`.
- Zoom steps jump; there is no animated zoom (Own; keeps the view exact).
- Keyboard focus: the canvas draws no focus ring or focus line. As
  `visual-language.md` ("Focus and keyboard indication") fixes, its focus
  is implied whenever no other control has it: then the node keys and the
  plain-key chords such as `.` and `,` work. When a field or control takes
  focus it shows its own `accent` ring, and the focused dock group its
  title strip line; clicking the canvas gives focus back and those cues
  disappear (keymap.md: shortcuts stop while a text field has focus).

Tier: **Match** (gestures, constant line weight, grid and rulers following
zoom), **Own** (no zoom animation, implied canvas focus).
Milestone: M1 (today).

---

## Canvas menu (right-click)

- The reference's canvas right-click menu is not described on the pages in
  our notes (unknown).
- LaserCraft: `menus.md` owns the canvas menu's contents and proposes the
  rules (right-click on an unselected object selects it first; no menu
  while drawing). The canvas only adds: while the menu is open, the object
  it acts on keeps its Selected look and hover lifting is suppressed.

---

## Dark theme

### Reference

- The reference's dark option darkens only the canvas: the bed edge stays
  dark, grid lines become a grey close to the background, ruler numbers
  lighter grey, and the origin marker stays visible; everything around the
  canvas stays light [R1]. A system-following theme exists but is not shown
  [R36].

### LaserCraft (own design, decision 13)

| Token | Light theme | Dark theme |
|---|---|---|
| `canvas` | neutral grey a little darker than `bed` | dark neutral grey |
| `bed` | near-white | **near-white (unchanged)** |
| `bed-edge` | dark neutral | dark neutral (unchanged; it sits on the light bed) plus a *hairline* light halo outside it so the edge reads on the dark surround |
| `grid-minor`, `grid-major` | very faint / faint grey | unchanged (they are on the light bed) |
| `ruler`, `ruler-text` | `panel`-like grey / `text-muted` | the dark `panel` / the dark `text-muted` |
| `origin-marker`, `start-marker`, `node`, `handle-fill`, `selection`, `marquee`, `drawing`, `guide` | as defined | unchanged inside the bed; marks that fall on the dark surround get a thin `bed`-colored halo |
| `head-marker`, `frame-preview` | as defined | unchanged, with the same halo rule |
| Drag read-out and badge chips | `field` / `border` / `text` | the dark `field` / `border` / `text` |

- Everything on the bed keeps the light-theme look, so layer colors, which
  were chosen against a light sheet, look the same in both themes and black
  layer 0 stays visible (`docs/spec/canvas.md`).
- Shapes parked on the dark surround keep their layer color and get a thin
  `bed`-colored halo, so black and dark layers stay visible there (Own).
- Reduce motion, Handle size and Grid strength behave the same in both
  themes.

Tier: **Echo** (a dark surround with a light work-area look), **Own** (the
bed stays light, halos).
Milestone: later (themes; today the surround follows the OS theme and the
bed is always light, `canvas.rs:229-232`).

---

## The job preview and the canvas

- Reference: the preview is its own window with a separate look: the bed
  edge in pale green, a small red plus for the simulated head, travel in
  light red [R32].
- LaserCraft: the job preview (`job-preview.md`, `docs/spec/preview.md`)
  is a separate window that draws the planned moves; the canvas never draws
  planned moves, travel or the cut order.
- Shared so the two read as the same bed: the same `bed`, `bed-edge` and
  grid tokens; the same orientation (rear at the top); the same bed
  coordinates on any rulers or read-outs; the same machine origin marker.
- Not shared: the preview's own move tokens (`move-burn`, `move-unlit`,
  `move-travel`) and its simulated head (`preview-head`, an `accent` dot)
  are never drawn on the canvas, and the canvas's `head-marker` (the real
  head) is never drawn in the preview. One look per meaning: the real head
  is a crosshair, the simulated head a dot.
- Show fills on the canvas and the preview's fill scan lines are separate:
  Show fills is a display aid; the preview shows what will burn.

Tier: **Own**.

---

## Deliberately omitted

- The reference's dark-background canvas switch (the theme covers it and
  the bed stays light; `naming.md`, Omitted).
- The separate antialiasing switch (always on in egui).
- Galvo-only canvas items (galvo framing, red-light preview, lens field
  outline; decision 1).
- Shift-click framing with the laser on (decision 7; no modifier fires the
  laser).
- Copying the reference's colors for markers (red origin, green job origin,
  green and red selection boxes, green start node): ours follow decision 6.

---

## Current state → target

Colors in today's code and how the tokens fix the clashes (current-ui.md
section 2; values deliberately not repeated):

| Today | Clash | Target |
|---|---|---|
| Orange selection box, orange handle outlines, orange rotate ring, orange nodes and Bézier rings (`canvas.rs:32`, `canvas.rs:334-378`) | orange means "selected" on the canvas but "laser off" (unlit moves) in the preview (`preview.rs:13`) | selection bounds and handles in `selection` (deep `accent` family); nodes `node` / `node-selected` / `start-marker`; amber is only `warning` and `frame-preview` |
| Soft green for the Touching box and for drawing in progress (`canvas.rs:33`, `canvas.rs:390-434`) and medium green for the preview's bed outline (`preview.rs:12`) | green has three meanings | `marquee` for both boxes (solid vs dashed), `drawing` for previews, `bed-edge` in the preview; green is only `success` (connected, finished) |
| Inside box in sky blue, constant named `SHAPE` but used only there (`canvas.rs:31`, `canvas.rs:386-389`) | box direction told by hue; misleading name | one `marquee` hue, line style tells the direction |
| Red-orange origin dot (`canvas.rs:34`, `canvas.rs:235-241`) | red-orange reads as danger next to a future red Stop and `danger` head marker | `origin-marker` dark slate corner square, arms and X/Y letters |
| Preview travel in strong red (`preview.rs:14`) | red reads as danger or error | `move-travel` mid grey dashed (`job-preview.md`) |
| Preview head in royal blue (`preview.rs:15`) | — | `preview-head`, an `accent` dot (kept, different shape from the canvas `head-marker`) |
| Canvas surround = egui extreme background (`canvas.rs:229`), near-black in a dark OS theme | the surround changes with the OS theme while nothing else is themed | `canvas` token in both themes |
| Grid and bed edge in one dark grey at different strengths (`canvas.rs:30`, `canvas.rs:233-234`) | the edge is the grid color, so they merge where they meet | `grid-minor`, `grid-major`, `bed-edge` (edge drawn last) |
| Handles filled with the theme's extreme background (`canvas.rs:369-374`) | near-black handles on a dark OS theme | `handle-fill` (white) in every theme |

Other differences:

- Fixed 10 mm / 50 mm grid with minor lines hidden when crowded
  (`canvas.rs:17-19`, `interaction.rs:14`) → adaptive round-number levels
  with a fade (proposed change to `docs/spec/canvas.md`).
- No rulers, no on-canvas coordinates (current-ui.md 1.11) → top and left
  rulers in bed coordinates.
- Pointer read-out in document coordinates, rear-left origin, y down
  (`shell.rs:637-639`) → bed coordinates from the machine origin, shared
  with the rulers and fields (`property-bar.md`).
- Selection box drawn on the exact bounds (`canvas.rs:364-368`) → offset
  outward, moving dashes, handles hidden while dragging.
- Rotate handle as a ring on a stem (`canvas.rs:375-378`) → `handle-fill`
  circle with a painted rotate cursor.
- Selected shapes drawn heavier (`canvas.rs:256-260`): kept; weights become
  the relative ladder above.
- Output-off fade (`canvas.rs:253-255`): kept.
- Drawing previews solid green (`canvas.rs:409-434`) → `drawing` dashed;
  the line tool gains placed-point nodes, a closing ring and a rubber-band
  segment style.
- No hover feedback, no snap feedback, no cursor changes, no drag read-outs
  (current-ui.md 1.11) → as specified above.
- No Esc cancel during handle drags → Esc cancels (proposed change to
  `docs/spec/canvas.md`).
- No canvas focus indicator: kept (focus implied, `visual-language.md`);
  today's egui default focus stroke, if any, is not drawn on the canvas.
- Line tool: double-click or Enter finishes (`docs/spec/canvas.md`) →
  also right-click; continuing and joining open paths, with Ctrl to stop
  the join (later).
- No Trim tool → hover preview in the layer color, auto-join with a
  selected result, Ctrl and Shift click variants (later).
- Tool windows (Offset, Grid array, Circular array) draw nothing on the
  canvas while open (`dialogs.rs:97-176`) → live `drawing` previews.

## Proposed changes to other specs

- **Proposed change to canvas.md**:
  - View: the surround uses its own tone; the bed edge is drawn above the
    grid; the grid has adaptive round-number levels (minor step from the
    1, 2, 5 sequence above the crowding threshold, major always ten minor
    steps, so both levels step through the same sequence) with major lines through the machine origin;
    Grid strength (later).
  - A Rulers section: top and left bands in bed coordinates from the
    machine-origin corner, ticks at grid steps, unit in the corner square,
    pointer and selection cues, click on the corner fits the bed.
  - The machine origin marker with arms and X/Y letters inside the bed.
  - Pointer position in bed coordinates (with `property-bar.md`).
  - Selection: bounds drawn offset outward with moving dashes (still under
    Reduce motion); handles hidden while dragging; Esc cancels a handle
    drag and keeps the selection; very small selections show only corner
    and rotate handles; the hit-area rule.
  - The Move handle, Skew handles (with `shape.skew`), the reference-point
    cross, locked selections in `text-disabled`, the handle switches.
  - A drag that starts on a snap point of the selection snaps that point.
  - Hover feedback (the hovered shape lifted to *heavy* in its layer
    color) and live pick preview during a selection box.
  - Snap markers per kind; later snap kinds (nodes, segment middles,
    intersections, points on paths, guide lines); Alt widens the snap
    distance and shows alignment lines; with snapping off in the View menu
    Ctrl turns it on.
  - Guide lines from the rulers onto guide layer 1.
  - Drawing previews in `drawing` dashed; too-small drawings in
    `text-muted`; the line tool's closing ring and rubber band.
  - The Trim, Place tabs, Measure, Start point, Round corners and Move
    head tools' canvas looks (with `toolbars.md`'s tool list). Trim: the
    leftover pieces are joined at the old crossings and selected;
    Ctrl+click removes without joining; Shift+click keeps the removed
    piece on the current layer, with a warning when that is its own
    layer; groups are refused; built-in shapes become paths.
  - Line tool: right-click finishes as well as double-click and Enter;
    clicking an end of an open path continues it; ending on an end of
    another open path joins the two, unless Ctrl is held; later, with
    curves, S switches corner and smooth placement.
  - Per-tool cursors with badges; the current layer color in the badge.
  - Show fills and Smooth curves as display-only view state.
  - The head marker, job origin marker with placed bounds, and frame path
    (M6), the camera overlay (M9).
  - The dark-theme rule for halos on the surround.
  - Sentence case "Edit nodes" (as `naming.md` proposes).
- **Proposed change to nodes.md**: the node look (start node in
  `start-marker`, open ends emphasised, join-target ring, hovered segment
  overlay, handle ends as circles); node selection by click (later).
- **Proposed change to shapes.md**: `shape.skew` for the Skew handles;
  rectangle corner radius for the Corner radius handle (with `naming.md`).
- **Proposed change to layers.md and planning.md**: guide-layer shapes are
  never counted in job bounds in any placement mode.
- **Proposed change to planning.md and nodes.md**: a per-subpath chosen
  start and direction for the Start point tool.
- **Proposed change to ui-shell.md and dialogs.md**: tool windows draw a
  live `drawing` preview of their result while open.
- **Proposed change to keymap.md**: in the mouse and modifiers section, Alt
  while dragging shows alignment lines and widens snapping; Esc during a
  drag cancels it; Shift adds and Ctrl/Cmd toggles for selection boxes as
  well as clicks (with the matching change to canvas.md); the Alt+Shift+W
  row (described today as "outline / filled") is worded "Show fills", the
  View menu label, when it is bound.
- **Proposed change to canvas.md and keymap.md (mouse section)**: the
  Shift and Ctrl/Cmd click modifiers also apply to the Inside box and the
  Touching box (see "Selection boxes").
- **Proposed change to arrange.md**: linked copies from the grid array
  (later): drawn paler in dash-dot, not hit by clicks or boxes, edits to
  the original show in all copies (see "Linked copies").
- **Proposed change to text.md**: the Arc handle on a single selected text
  shape (later, owner decision; see "Handles").
- **Proposed change to visual-language.md**: confirm the ruler cues, the
  reference-point cross, the hovered-segment overlay in node editing and
  the hover outlines of handles and tab markers in `selection` (all
  canvas uses of the listed `selection` token, no chrome tints); confirm
  the `danger` head marker during a paused constant-power job and the
  small `danger` ring on the head marker in Alarm, which its danger list
  does not name yet (see "Head marker"). This file now follows its
  ladder (*hairline*, *thin*, *normal*, *heavy*; `bed-edge` *thin*), its
  "no focus ring on the canvas" rule, and its complete accent and danger
  lists: no `hover` or `accent-soft` on the canvas, no `danger` previews
  (Trim, placed bounds, Move head ghost and an out-of-bed frame path use
  other looks plus `mark-error`).
- **Proposed change to keymap.md**: S as a tool-local key while the Line
  tool is drawing (corner or smooth points, later with curves), not bound
  in `default.keymap`, like the node keys; Ctrl while placing a Line
  point stops the automatic join (it already turns snapping off).
- **Proposed change to icons.md**: corner and smooth variants of
  `badge-line` for the Line tool's two placement modes (later, with
  curves).
- **Proposed change to preview.md** (through `job-preview.md`): unlit moves
  `move-unlit` and travel `move-travel` instead of orange and red; the bed
  outline `bed-edge` instead of green.
- **Proposed change to layers.md** (owner decision): low-contrast layer
  colors get a darker *hairline* edge on the canvas instead of new palette
  data.

## egui implementation notes

These collect the egui details mentioned in the sections above; the
sections stay the spec.

- **One painter, fixed order.** The canvas is one `egui::Painter` over the
  central region, drawn back to front exactly in the Layering order of
  "How to read this file" (hovered shapes are simply stroked at the
  heavier weight in their place), with any painted cursor last.
  Floating chips that must sit above other panels use an `egui::Area` on
  the foreground order; the job bar's own `Area` stays above everything
  (`machine-panels.md`).
- **Constant screen weights.** Stroke widths, handle sizes and marker
  sizes are computed in screen points from the token ladder, not scaled by
  zoom; geometry is transformed to the screen before stroking. Dashes
  (selection, `marquee`, `drawing`, frame path, linked copies) are drawn
  with `egui::Shape::dashed_line` on the screen-space polyline.
- **Halos.** A halo is the same shape stroked first, wider, in
  `handle-fill` (marks and badges) or `bed` (overlay and dark-theme rule),
  then the mark on top; there is no blur.
- **Antialiasing** is egui's feathering, always on; there is no switch.
- **Cursors.** System cursors come from `egui::CursorIcon` (`Default`,
  `Crosshair`, `Move`, `ResizeNwSe`, `ResizeNeSw`, `ResizeHorizontal`,
  `ResizeVertical`, `NotAllowed`, `Grab`/`Grabbing` for panning). The
  rotate cursor and every badge are painted: the canvas sets
  `CursorIcon::None` over that hit area and paints the glyph at the
  pointer in `text` with a `handle-fill` halo, as the last thing in the
  frame. Cursor artwork is `icons.md`'s.
- **Repaint.** The canvas repaints on input as today. The moving
  selection dashes and the grid fades request repaints
  (`Context::request_repaint_after`) only while something is selected or a
  fade is running and the window has focus; with Reduce motion on, or when
  egui reports the system asks for reduced motion, they never request a
  repaint. The live head marker repaints when a device `Event` arrives,
  never on a timer.
- **Hit testing** uses the hit areas of the Handles section in screen
  space, checked handles first, then shapes through the hit rules of
  `docs/spec/shapes.md`; the UI only reads the result and invokes
  commands (no business logic in the UI crate).
- **Focus.** The canvas takes keyboard focus through its `Response`
  (`request_focus` on click); nothing is drawn for it, and egui's default
  focus stroke is suppressed on the canvas widget.
- **Device data.** The head marker position and machine state come only
  from `lasercraft_device::Event` handling in the app; the canvas never
  waits for the device.

## Open questions

1. **Accent on the canvas.** Decision 3 lists the uses of `accent`
   ("nowhere else"), while the shared token list puts `selection`,
   `marquee`, `drawing` and `node-selected` in the `accent` family. This
   file treats them as the canvas-selection exception of decision 6, and
   also uses `selection` for the ruler pointer mark, the ruler selection
   band, the reference-point cross, the node and segment hover marks and
   the hover outlines of handles, control handle ends and tab markers (no
   `accent-soft` or `hover` on the canvas). `visual-language.md` should
   confirm these, or pick a separate canvas selection hue.
2. **Marching paths.** Should selected paths themselves carry slowly moving
   dashes in their layer color (the reference's way of showing the cut
   direction), instead of or besides the moving bounds?
3. **Rotate zones.** Keep one rotate handle above the top edge
   (`docs/spec/canvas.md`), or add the reference's four corner rotate
   zones?
4. **Bottom and right rulers.** Offer them (off by default)?
5. **Off-bed shapes.** Mark the parts of shapes outside the bed (for
   example a faint `warning` tint) while editing, or leave that to the job
   preview and the Start check?
6. **Live head marker during jobs.** The reference deliberately does not
   update its marker during a job so users watch the machine; this file
   makes it live but calm. Owner decision.
7. **Head marker during an M3 pause.** The marker stays `danger` (laser
   may be firing) while the chrome shows `warning-soft`; confirm, or draw
   the marker in `warning` for that state?
8. **Automatic tabs on the canvas.** Show planner-placed tabs faintly, or
   only in the preview?
9. **Remove-from-selection box.** Bind Ctrl+Shift with a selection box to
   remove from the selection (the reference does)? keymap.md lists no such
   gesture today.
10. **Low-contrast colors.** Approve the darker-edge rule, or change the
    palette data (`layers-panel.md` open questions 1 and 2)?
11. **Guide layer hue vs `guide`.** The guide token (teal) and the guide
    layer colors must stay clearly apart; `visual-language.md` picks both.
    Is the word clash ("guide" token, "guide layer") acceptable in specs?
12. **Equal-spacing cues** with alignment lines, and auto-scroll when a
    selection box or drag reaches the view edge: wanted?
13. **Stale camera picture.** Mark an old overlay on the canvas?
14. **Drag read-out chip.** Keep the chip next to the pointer in addition
    to the Tool hint, or the Tool hint only?
15. **Hover by weight alone.** A hovered shape is lifted to the Selected
    weight without bounds. Is that distinct enough, or should
    `visual-language.md` grant a canvas hover tint after all?
16. **Trim preview without red.** The reference previews the removed piece
    in red; LaserCraft uses a heavy dashed stroke in the layer color. Keep,
    or let `visual-language.md` add the trim preview to `danger`'s list?
17. **Ctrl in the Line tool** both turns snapping off (keymap.md, while
    dragging) and stops the automatic join. Is one key for both acceptable,
    given that a join needs a snapped end anyway?

## Sources

All pages read 2026-10-21, through the research notes for this guide.

- [R1] https://docs.lightburnsoftware.com/2.1/Reference/UI/Workspace/ —
  the flat light background, the dark-background option, bed outline, grid
  only on the bed, ruler margin and number style, the red origin square
  with X and Y letters, display switches (antialiasing).
- [R2] https://docs.lightburnsoftware.com/2.1/Reference/ViewStyle/ —
  filled versus outline display, fills hiding what is behind them, smooth
  versus coarse curves, none affecting output.
- [R3] https://docs.lightburnsoftware.com/2.1/Reference/Selection/ —
  click and box selection, the two box directions and colors, modifiers,
  groups in boxes, moving dashes on selected paths, the group dash rhythm,
  duplicates flickering, handles slightly outside the artwork.
- [R4] https://docs.lightburnsoftware.com/2.1/GetStarted/SelectingMovingSizing/
  — box selection, the selected look, scale, rotate and move gestures.
- [R5] https://docs.lightburnsoftware.com/2.1/Reference/Snapping/ — snap
  kinds and their cursor glyphs, grid levels with zoom, the separate snap
  lattice, grid settings, Alt and Ctrl/Cmd modifiers, hairline strokes.
- [R6] https://docs.lightburnsoftware.com/2.1/Reference/AutomaticGuidelines/
  — guide lines dragged from the rulers, their tool-layer nature and the
  job-bounds pitfall, Alt alignment guides, hairline strokes.
- [R7] https://docs.lightburnsoftware.com/2.1/Reference/ZoomPan/ — wheel,
  keys, fit and zoom-to-selection, panning and its open-hand cursor,
  constant line weight, rulers on all edges, the bed in a frame-selection
  example.
- [R8] https://docs.lightburnsoftware.com/2.1/GetStarted/MovingAroundTheWorkspace/
  — pan and zoom gestures; text drawn as outlines.
- [R9] https://docs.lightburnsoftware.com/2.1/Reference/SetStartPoint/ —
  start arrow grey versus blue, click, Shift-click and Ctrl/Cmd-click,
  default start and direction.
- [R10] https://docs.lightburnsoftware.com/2.1/Reference/TransformControls/
  — the handle layout on the upright bounds, move, scale, rotate and shear
  handles, their cursors and modifiers, Esc cancelling, the status-bar
  angle, the handle switches, layer-colored selected dashes.
- [R11] https://docs.lightburnsoftware.com/2.1/Reference/EditNodes/ — node,
  start node and handle looks, no selection handles in node mode, Shift
  guide lines, node snapping; origin letters and minor grid in examples.
- [R12] https://docs.lightburnsoftware.com/2.1/Reference/UI/StatusBar/ —
  pointer coordinates and tool read-outs next to the canvas; rulers on all
  edges, uneven rounded labels, origin direction in examples.
- [R13] https://docs.lightburnsoftware.com/2.1/Reference/LaserWindow/ — the
  last-position crosshair and why it is not live, the frame path shown on
  the canvas, the job-origin placement controls.
- [R14] https://docs.lightburnsoftware.com/2.1/Reference/TwoPointRotateScale/
  — the two-point flow and prompts; pictures outside the bed with no grid;
  ruler direction following the origin.
- [R15] https://docs.lightburnsoftware.com/2.1/Reference/CenterFinder/ —
  the bed outline and origin corner in examples; the guide circle on a tool
  layer.
- [R16] https://docs.lightburnsoftware.com/2.1/Reference/DrawLines/ — the
  line tool's crosshair and badge, the corner and smooth badge marks and
  the S switch, the segment following the pointer in the picked color,
  curve handles, Shift, live read-outs, finishing by right-click, continuing
  an open path from an end, joining two open paths and Ctrl/Cmd stopping it.
- [R17] https://docs.lightburnsoftware.com/2.1/Reference/PrimaryShapes/ —
  drawing from a corner or the centre, Shift for regular shapes, the shape
  staying selected with working handles, the Ctrl/Cmd corner-radius and
  side-count handles.
- [R18] https://docs.lightburnsoftware.com/2.1/Reference/Measure/ — hover
  marking of the shape and segment, drag measuring, Shift.
- [R19] https://docs.lightburnsoftware.com/2.1/Reference/AddTabs/ — the
  tab cursor, the tab marker, dragging and removing tabs, line-layer shapes
  only.
- [R20] https://docs.lightburnsoftware.com/2.1/Reference/TrimShapes/ — the
  removal preview in red and the scissors cursor, the join of the leftover
  pieces and its selection, Ctrl/Cmd+click without join, Shift+click
  keeping the piece on the current layer with a same-layer warning, no
  groups, built-in shapes converted.
- [R21] https://docs.lightburnsoftware.com/2.1/Reference/RadiusFillet/ —
  the rounding cursor badges, including the no-room mark.
- [R22] https://docs.lightburnsoftware.com/2.1/Reference/PositionLaser/ —
  click to send the head, homed machine, the timed switch-off.
- [R23] https://docs.lightburnsoftware.com/2.1/Reference/WarpDeform/ —
  warp and mesh handles and the dashed target outline.
- [R24] https://docs.lightburnsoftware.com/2.1/Reference/Text/ — the text
  cursor, outlined letters while typing, the bend handle, handles without a
  drawn box.
- [R25] https://docs.lightburnsoftware.com/2.1/Reference/NumericEditsToolbar/
  — positions measured from the machine origin, the origin square and axis
  letters, handles without a drawn box.
- [R26] https://docs.lightburnsoftware.com/2.1/Reference/CoordinatesOrigin/
  — the job-origin square and when it moves, the machine-origin square with
  faint X and Y letters, the two coinciding in absolute mode.
- [R27] https://docs.lightburnsoftware.com/2.1/GetStarted/FramingBeginner/
  — the frame path drawn as an outline on the canvas.
- [R28] https://docs.lightburnsoftware.com/2.1/Reference/GridArray/ —
  linked clones drawn dashed and paler, not selectable, following the
  original.
- [R29] https://docs.lightburnsoftware.com/2.1/Reference/OffsetShapes/ —
  the live result preview while the window is open, the original kept
  dashed.
- [R30] https://docs.lightburnsoftware.com/2.1/Reference/TangentCircleGenerator/
  — candidate circles drawn as outlines, one emphasised on hover.
- [R31] https://docs.lightburnsoftware.com/2.1/Reference/Cameras/CamerasWindow/
  — the overlay painted under the grid, opacity, size and offset controls.
- [R32] https://docs.lightburnsoftware.com/2.1/Reference/Preview/ — the
  preview as a separate window with its own bed edge, head mark and travel
  look.
- [R33] https://docs.lightburnsoftware.com/2.1/Reference/UI/ — the default
  layout's canvas: numbers on all four edges, the origin corner marks.
- [R34] https://docs.lightburnsoftware.com/2.1/GetStarted/UITour/ — the
  grid standing for the machine's work area.
- [R35] https://docs.lightburnsoftware.com/2.1/Reference/LockShapes/ —
  locked shapes refuse edits; their canvas look is not shown.
- [R36] https://docs.lightburnsoftware.com/2.1/NewFeatures/NewIn2.1/ — an
  application theme setting with a system choice (dark look not shown).

Existing specs used: `docs/spec/canvas.md`, `docs/spec/shapes.md`,
`docs/spec/nodes.md`, `docs/spec/keymap.md` and
`crates/lasercraft-ui/keymap/default.keymap`, `docs/spec/layers.md`,
`docs/spec/planning.md`, `docs/spec/preview.md`, `docs/spec/safety.md`,
`docs/spec/ui-shell.md`, `docs/spec/text.md`, `docs/spec/arrange.md`,
`docs/clean-room.md`; style files `naming.md`, `layout.md`, `toolbars.md`,
`property-bar.md`, `layers-panel.md`, `machine-panels.md`,
`visual-language.md`, `menus.md`.

Open standards: WCAG 2.1, https://www.w3.org/TR/WCAG21/ — success criterion
1.4.11 (non-text contrast, 3:1) for the low-contrast edge rule, and 2.3.3
(animation from interactions) for stopping the moving dashes and fades under
Reduce motion. GRBL v1.1 laser mode (`M3` constant power keeps the laser on
while stopped), https://github.com/gnea/grbl/wiki/Grbl-v1.1-Laser-Mode, for
the head marker's danger state during a paused constant-power job.

Own design: the surround tone; the stroke-weight ladder and dash rhythms;
the layering order; the adaptive round-number grid and its fade; ruler
bands, ticks, label rules, corner square, pointer and selection cues; the
machine origin marker's arms, letters and color; selection by weight, the
offset dashed and moving selection bounds; the shared handle look, hit
areas, states and small-selection rule; one rotate handle with a painted
cursor; the move circle and skew diamonds and their fixed corner; drag
read-out chips; one-hue selection boxes told apart by line style and the
live pick preview; hover by weight lift; snap markers per kind and the badge
marker; alignment-line look; guide lines excluded from job bounds; the
`drawing` look and per-tool marks; node, handle and open-end looks; start
arrow shape and the outline-versus-filled rule; tab bars; the measure line
and chip; the live head marker with its states and off-screen arrow; the
hollow job origin marker with placed bounds; when and how the frame path
shows; the dash-dot linked copies; overlay halos; every cursor badge and
the layer-color square; implied canvas focus; the trim preview look and
Tool hints; the Line continue and join rings; the dark-theme table; and every
label and tooltip.
