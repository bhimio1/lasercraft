# Spec: style — position and size bar, text style bar, properties panel

Milestones: M1 (the selection, transforms and canvas handles that these
controls edit exist since M1), M3 (the Add text window, whose font and
capital height become the Text style toolbar's next-text defaults), M7 (image
rows of the Properties panel); the Position and size toolbar, the Text style
toolbar, the Properties panel, the handle switches, editable text and inch
units are later, with no milestone yet.

This file fixes the typed-value controls that act on the current selection:
the Position and size toolbar and the Text style toolbar in toolbar row 2, the
Properties panel in the right dock, and the four handle switches at the left
end of the status bar. For each it gives the order of items, labels,
tooltips, units and precision, how values are typed, stepped, dragged and
committed, which command each change runs, and how every item looks in each
state. Where these toolbars and the panel sit and how rows fold is
`layout.md`'s; the buttons of every other toolbar are `toolbars.md`'s; the
menus that repeat Lock and the rotate items are `menus.md`'s; labels come from
`naming.md` (its "Numbers in fields and read-outs" table sets precision);
colors, type and sizes are only the tokens of `visual-language.md`; pictograms
(lock, chevrons) are `icons.md`'s; the canvas handles, the selection bounds
and the reference-point marker are drawn as `canvas.md` says; the Font browser
dialog is `dialogs.md`'s; the Layers panel is `layers-panel.md`'s; the Text
data panel and the Properties panel's dock tab are listed in
`later-windows.md` and `layout.md`, but the Properties panel's rows are
specified here. `README.md` explains the tiers (Match, Echo, Own).

## How to read this file

- Every element has a **Reference** part (what the public docs show, each
  fact tagged with a citation key such as [R1], listed under Sources), a
  **LaserCraft** part (the binding spec) and a **Tier** line.
- Labels in double quotes or in tables are exact UI labels from `naming.md`,
  except the messages and the placeholder this file adds ("Not a number:
  ‹text›", "‹n› shapes are locked", the hints of the handle switches and the
  placeholder "varies"), which are proposed additions to `naming.md` (see
  "Proposed changes to other specs").
  Tooltips are our own words; the chord, where one is shown, is read from the
  keymap at run time and drawn with key symbols (`naming.md`, Tooltips).
- "Field" means a single-line numeric or text input. "Commit" means the
  moment a typed or dragged value becomes a command.
- Tokens are written in backticks (`field`, `accent`, `row`, `gap-tight`).
  No colors or sizes are given in any other form.
- Nothing in this file exists in the code today except the Add text window
  (current-ui.md 1.3) and the scale and rotate handles on the canvas
  (current-ui.md 1.11). See "Current state → target".

## Shared field behaviour

All three places (Position and size, Text style, Properties) use the same
field widget and the same commit rules. Each element below only adds what is
specific to it.

### Reference

- Numeric fields in these areas are white boxes with a short label directly
  to their left, a unit written after some of them, and small up/down
  arrows for stepping [R1][R9].
- Each field keeps a fixed number of decimals: three for positions and sizes,
  two for the rotation and text sizes, none for counters [R1][R3][R4].
- A typed value takes effect when Enter is pressed [R1]; whether leaving the
  field also applies it is not stated in the notes.
- Position and size fields accept small arithmetic expressions and a unit
  written after the number, which is converted to the current unit [R1].
- Fields that cannot be used are drawn greyed, staying in place [R1][R4][R9].
- In a dock form, a field that has keyboard focus shows its text selected
  with a blue highlight [R3].

### LaserCraft

Anatomy, left to right:

| Part | Look | Notes |
|---|---|---|
| Label | `type-body` in `text`, right-aligned, `gap-tight` before the field; no colon; the unit in parentheses as part of the label ("Width (mm)") | `naming.md` Units rule: the field itself shows no unit suffix |
| Value box | `field` background, thin `border` outline, slightly rounded corners, one `row` tall | all numeric fields of one toolbar share one width (below) |
| Value text | `type-body` in `text`, right-aligned inside the box so decimal points line up between the two lines of a toolbar | own design; the reference aligns left |
| Stepper | two small chevrons (up over down) stacked inside the right end of the box, each half the box tall, drawn in `text-muted` | numeric fields only; not on Font, Text source or align dropdowns |

States:

| State | Box | Text and label | Stepper | When |
|---|---|---|---|---|
| Normal | `field`, `border` outline | value `text`, label `text` | `text-muted` chevrons | enabled, not focused |
| Hover | outline turns `text-muted` | unchanged | the chevron under the pointer gets a `hover` background | pointer over an enabled field |
| Stepper pressed | unchanged | unchanged | the pressed chevron gets `pressed` | primary button held on a chevron |
| Focused (editing) | `accent` focus ring just outside the outline | the whole value is selected on focus, selection drawn in `accent-soft` behind the text; caret in `text` | shown | the field has keyboard focus |
| Invalid | `danger` outline, `danger-soft` background | value `text` | shown | the typed text cannot be read as a number, until it is fixed or reverted |
| Varies | `field` | empty value with the placeholder "varies" in `text-muted` | shown | several selected objects have different values (Text style and Properties only; Position and size always shows one block) |
| Read-only | `field-alt`, `border` outline | value `text-muted`, label `text` | hidden | the selection holds a locked shape (Properties' Locked, later) |
| Disabled | `field-alt`, `border` outline | value empty or as the element says, label `text-disabled` | hidden | the field cannot apply to the current selection |

Typing and committing (own design, extending ui-shell.md's rule that a
typed value is committed when entered and a dragged one when the drag ends):

- **Enter** commits and keeps focus in the field with its new value
  selected, so another value can be typed straight away.
- **Tab** and **Shift+Tab** commit and move focus to the next or previous
  field (see "Keyboard focus and Tab order").
- **Focus loss** (clicking the canvas, another control or another panel)
  commits too. A value is never silently dropped.
- **Esc** reverts the field to the current value without running anything
  and gives keyboard focus back to the canvas.
- Each commit runs **one command** (one undo step; the Edit menu's Undo
  names it, `naming.md`). A commit whose parsed value equals the current
  value runs nothing.
- Text that cannot be read as a number shows the Invalid state, runs
  nothing and puts the message "Not a number: ‹text›" on a `danger-soft`
  band in the status bar's Messages field (`layout.md`). Esc or a valid
  value clears it.
- Out-of-range values are clamped to the field's range and the clamped
  value is what the command receives (`naming.md`, Numbers). A field that
  only takes positive values refuses a minus sign as it is typed.
- The canvas is not updated while the user types; it updates on commit.

Stepping (own design; the reference's step sizes are not known):

| Quantity | Click on a chevron | Shift+click | Ctrl+click |
|---|---|---|---|
| Lengths (X, Y, Width, Height, Capital height, Max width, Corner radius) | 1 mm | 10 mm | 0.1 mm |
| Percentages (Width (%), Height (%), spacing, Power (% of layer)) | 1 | 10 | 0.1 |
| Angle (Rotation (°)) | 1° | 15° | 0.1° |
| Whole numbers (Run order, Counter offset) | 1 | 10 | 1 |

- The length steps are the same as the nudge chords (keymap.md: arrows
  1 mm, Shift 10 mm, Ctrl 0.1 mm), so one rule covers both.
- With the field focused, the Up and Down arrow keys step like the
  chevrons (with the same modifiers). Shortcuts are ignored while a text
  field has keyboard focus (keymap.md, Behavior), so these keys never nudge
  the selection at the same time.
- Each click on a chevron commits one command at once. Holding a chevron
  repeats the step after a short delay; the repeats are one gesture
  (commands.md, Gestures) and become one undo step when the button is
  released.
- The mouse wheel never changes a field that is not focused (so scrolling
  a panel cannot edit values). Over a focused field the wheel steps like the
  chevrons, each wheel step being one command.

Dragging a value (own design, ui-shell.md's drag rule):

- Pressing on the value text of an enabled numeric field that is not focused
  and dragging sideways changes the value: right increases, left decreases;
  Shift drags ten times faster, Ctrl ten times slower. The canvas previews
  the change live. Releasing commits one command; Esc during the drag
  restores the value and runs nothing.
- A click without movement focuses the field for typing instead.

Precision and display:

- Shown decimals follow `naming.md` "Numbers in fields and read-outs":
  positions and sizes like other lengths, percentages and angles as that
  table says, whole numbers with none. LaserCraft follows that table, not
  the reference's three decimals [R1].
- A field keeps more decimals than it shows; only the drawing is rounded.
- The decimal separator is a full stop (open question in `naming.md`).

Expressions and unit suffixes (later, own design after [R1]):

- A later version lets length fields take a small expression (numbers,
  + − × ÷ written as `+ - * /`, parentheses, `pi`) and a unit after a number
  (`mm`, `cm`, `in` or `"`), converted to the current unit. The result is
  computed on commit and shown in the field.
- The evaluator is a pure function outside the UI crate (CLAUDE.md: no
  business logic in the UI), tested on its own. Until it exists, a field
  accepts a plain number only. Open question: whether the command line
  should accept the same expressions.

Field width:

- Every numeric field in one toolbar has the same width: enough for the
  longest value the bed allows, with a sign and the shown decimals, in
  `type-body`, plus the stepper. Properties fields fill the panel's field
  column up to the same width and no wider.

Tier: **Echo** (labelled fields with steppers, greyed in place, Enter to
apply), **Own** (right-aligned values, every state's tokens, focus-loss
commit, Esc, step sizes, drag on the value, the Invalid and Varies states).

## Position and size toolbar

### Reference

- The reference has a toolbar for typing the position, size and rotation of
  the selection. Everything selected is handled as one block: one bounding
  box, with the relative positions and proportions of the objects kept
  [R1].
- By default it sits at the left of the second toolbar row, under the main
  icon row and above the canvas, with the reference's text toolbar to its right;
  it can be hidden and shown again from the Window menu [R1][R8][R12].
- It is about twice as tall as one field row; its items stand in columns,
  some columns holding two fields one above the other and some items centred
  across both lines [R1][R9].
- Items, left to right: horizontal position over vertical position, each
  with a unit after it; a padlock button covering both lines that keeps the
  width-to-height ratio when on; width over height with units; two
  percentage fields (width over height) that show one hundred; a grid of
  nine small round choices; a rotation field with no unit; and a units
  control [R1][R9].
- The positions are those of the chosen point of the selection's bounding
  box, measured from the machine's origin corner, which is marked on the
  canvas; positive values lie on the bed and negative ones outside it.
  Typing a position moves the whole selection so that point lands there
  [R1].
- Width and height are the bounding box size; changing them rescales toward
  or away from the chosen grid point [R1].
- The padlock: closed and highlighted when proportions are kept, open and
  plain when width and height change independently; a click switches it
  [R1].
- The nine-point grid picks the point of the bounding box used for the
  positions, the fixed point when resizing and the pivot when rotating:
  four corners, four edge midpoints and the centre; exactly one is chosen
  [R1].
- The rotation field turns the selection by the typed number of degrees,
  clockwise for positive values, counter-clockwise for negative ones; it
  reads zero in every screenshot [R1][R2]; whether it is a turn-by entry or
  shows an accumulated angle is not stated.
- The units control flips the design units between millimetres and inches;
  in 2.1 it changed from a single push button to two stacked radio choices,
  and design and machine units can be linked or kept apart [R1][R10][R8].
- With nothing selected, the values, the padlock and the grid are drawn pale
  grey but keep their last values [R1][R9]. With text placed on a path,
  the position and size fields are greyed while the grid stays [R6].
- The toolbar is the precise counterpart of the canvas handles [R1][R2].
- In a narrow window, a double chevron at the end of the clipped toolbar
  hides the items that do not fit [R1][R15].

### LaserCraft

Purpose: type the position, size, scale and rotation of the selection, and
choose which point of its bounds those values refer to. Default place: toolbar
row 2, first (`layout.md`, Toolbar band). Window → Toolbars entry: "Position
and size" (`menus.md`). Milestone: later.

#### Two-line arrangement

The toolbar is two `row`s tall. Items stand in columns, left to right. A
column with two fields puts one on each line; an item marked "both" is
vertically centred across the two lines.

| Column | Top line | Bottom line | Group | Notes |
|---|---|---|---|---|
| 1 | X (mm) | Y (mm) | Position | labels right-aligned to one edge, fields left-aligned to one edge |
| — | separator | | | thin `border` line, `gap` each side |
| 2 | Keep proportions (both) | | Size | lock toggle button |
| 3 | Width (mm) | Height (mm) | Size | |
| 4 | Width (%) | Height (%) | Size | |
| — | separator | | | |
| 5 | Reference point (both) | | Reference point | 3×3 point grid, about as tall as both lines |
| — | separator | | | |
| 6 | Rotation (°) (both) | | Rotation | one field, centred between the lines |
| — | separator | | | |
| 7 | mm (top choice) | in (bottom choice) | Units | later; hidden until inch units exist |

- Spacing: `gap-tight` between a label and its field, `gap` between columns
  inside a group, `gap-wide` around separators.
- Both lines share the toolbar's `panel` background; there is no frame
  around the toolbar. In stage a of docking there is no grip; a thin
  `border` separator divides this toolbar from Text style (`layout.md`,
  Toolbar model).

#### Items

| # | Label (exact) | Kind | Shows | Range | Commit runs | Tooltip (first line; second line in `text-muted`) |
|---|---|---|---|---|---|---|
| 1 | X (mm) | numeric field | horizontal position of the reference point of the selection bounds | any; values off the bed are allowed (the job check refuses them later, safety rule 5) | `shape.move_to` (proposed, below) | "Horizontal position of the reference point" / "Measured from the machine origin." |
| 2 | Y (mm) | numeric field | vertical position of the same point | any | `shape.move_to` | "Vertical position of the reference point" / "Measured from the machine origin." |
| 3 | Keep proportions | lock toggle button | on or off | — | no command (front-end setting) | on: "Keep proportions: width and height change together"; off: "Width and height change separately" |
| 4 | Width (mm) | numeric field | width of the selection bounds | greater than zero | `shape.resize` (proposed) | "Width of the selection" / "Resizes toward the reference point." |
| 5 | Height (mm) | numeric field | height of the selection bounds | greater than zero | `shape.resize` | "Height of the selection" / "Resizes toward the reference point." |
| 6 | Width (%) | numeric field | 100 at rest | greater than zero | `shape.resize` | "Scale the width by a percentage" / "Goes back to 100 after each change." |
| 7 | Height (%) | numeric field | 100 at rest | greater than zero | `shape.resize` | "Scale the height by a percentage" / "Goes back to 100 after each change." |
| 8 | Reference point | 3×3 point grid | the chosen point | one of nine | no command (front-end setting) | per dot: "Reference point: ‹place›" with ‹place› one of top left, top, top right, left, center, right, bottom left, bottom, bottom right / "Used for X, Y, resizing and rotation." |
| 9 | Rotation (°) | numeric field | 0 at rest | any; values are reduced to the range of one turn | `shape.rotate_at` (proposed) | "Turn the selection by this angle" / "Positive turns clockwise, about the reference point." |
| 10 | mm / in | two stacked choices | the design unit | mm or in | `ui.units <mm\|in>` (new front-end action) | "Units for lengths on the canvas and in fields" |

#### X (mm) and Y (mm)

- **What they show**: the position of the chosen reference point of the
  selection's combined bounds (the same box the Selection size status field
  measures and the canvas draws, `canvas.md`).
- **Coordinate frame** (Match [R1], proposed change to canvas.md): bed
  coordinates, measured from the machine-origin corner of the work area
  (`project.set_origin`, today front-left by default) with positive values
  running into the bed along both edges. A shape lying on the bed always
  has positive X and Y, whatever corner the origin is in. The Pointer
  position status field uses the same frame, so the two always agree.
- The model keeps document coordinates (geometry.md: y down, origin at the
  rear-left of the bed as drawn). The mapping between document and bed
  coordinates is one pure function in `lasercraft-project`, next to the work
  area, used by the status bar, these fields and later the Jog panel; the
  UI calls it and holds no geometry of its own.
- **Commit**: moves the whole selection so the reference point lands on the
  typed position; the other coordinate is passed unchanged. One command,
  one undo step ("Undo move").
- Typing a position off the bed is allowed (artwork may wait outside the
  bed); the job's work-area check refuses it at Start or Save G-code…
  (safety rule 5).

#### Width (mm) and Height (mm)

- **What they show**: the width and height of the selection's combined,
  axis-aligned bounds, in the current unit.
- **Commit**: resizes the selection so its bounds get the typed size,
  keeping the reference point fixed (Match [R1]). With Keep proportions on,
  the other dimension changes by the same factor in the same command; with
  it off, only the typed dimension changes.
- A rotated selection is resized as a block: its axis-aligned bounds are
  scaled, exactly as the canvas scale handles do (`canvas.md`).
- **Zero size**: a selection with zero height (a horizontal line) shows
  Height (mm) as zero and disables Height (mm) and Height (%), whose
  tooltip's second line reads "A flat selection cannot be given a height.";
  the same for zero width. Width changes keep a zero height zero, even with
  Keep proportions on. Zero and negative sizes are refused (shapes.md: the
  transform must stay invertible); mirroring is done with the Mirror
  commands, not with a negative size.

#### Width (%) and Height (%)

- Show 100 at rest (Match [R1]).
- **Commit**: scales the typed dimension by that percentage of its current
  size, about the reference point, then both fields go back to 100 and the
  absolute fields show the new size. Own design; the reference's behaviour
  after a change is not known.
- With Keep proportions on, typing either percentage scales both
  dimensions, and while the user types, the other percentage field shows the
  same number in `text-muted` as a preview of what will happen.
- The same command as the absolute fields runs (`shape.resize` with the
  resulting size), so undo reads "Undo resize" either way.

#### Keep proportions

Look (Echo after [R1]; the pictograms are `icons.md`'s lock pictograms):

| State | Background | Pictogram | Extra |
|---|---|---|---|
| On (default) | `accent-soft`, slightly rounded | closed lock, strokes in `text` | none (the toolbar "Checked" state in `toolbars.md`) |
| Off | none | open lock, strokes in `text` | none |
| Hover | on: stays `accent-soft`; off: `hover` | unchanged | tooltip |
| Pressed | `pressed` | unchanged | |

- Size: a button one `icon` square, vertically centred across both lines,
  just left of the Width and Height column. Own design: smaller than the
  reference's button that fills both lines [R1], so the column lines up
  with the other toolbar buttons.
- Click toggles it. Keyboard focus goes back to the canvas after the click
  (`toolbars.md`: toolbar buttons never keep focus).
- It is a front-end setting, not a project change: no command, no undo,
  not saved in the project. It is remembered for the session; saving it with
  the preferences is later.
- It applies only to the typed fields of this toolbar. The canvas scale
  handles keep their own rule (corner keeps proportions unless Shift is
  held, `canvas.md`).
- Default: on (own design; it matches the canvas corner handles).
- It stays enabled with nothing selected (it is a setting, not a value).

#### Reference point

Look (Echo after [R1]; own drawing):

- The shared 3×3 point grid of `visual-language.md` ("3×3 point grid"),
  about as tall as both lines; this section only adds what is particular to
  this toolbar.
- Nine small circles, each a choice dot, on a square `field` backing with a
  *thin* `border`. The dots are joined by *hairline* `border` lines that
  outline the box they stand for, so the grid reads as the points of the
  selection bounds rather than nine loose options. No recess, no shading.
- Not chosen: `field` fill with a *thin* `border` ring.
- Chosen: a filled `accent` dot, a little larger than the others.
- Hover: the dot under the pointer gets a `hover` disc behind it.
- Disabled: not used here. The grid is a setting, so it stays enabled in
  every row of the state table below; the shared disabled look in
  `visual-language.md` applies only where another file greys a grid.

Behaviour:

- A click on a dot chooses it. Exactly one dot is chosen at a time.
- The nine points are named in screen words (`naming.md`: top is the rear
  of the bed on screen): top left, top, top right, left, center, right,
  bottom left, bottom, bottom right. The grid is always drawn as the bounds
  look on screen, whichever corner the machine origin is in.
- The chosen point is used by X (mm) and Y (mm) (which point is measured),
  by every size field (which point stays fixed) and by Rotation (°) (the
  pivot). It does not affect the canvas handles, the nudge keys or the
  rotate chords, which keep the rules of `canvas.md` and keymap.md
  (rotation about the centre).
- While the pointer is over the grid, or any field of this toolbar has
  keyboard focus, the canvas marks the chosen point on the selection bounds
  with a small cross in `selection` (proposed change to canvas.md), so the
  user sees which point the numbers refer to.
- Default: center (own design; the reference's default is not stated and
  its screenshots differ). It is a front-end setting like Keep proportions:
  no command, no undo, remembered for the session, saved with the
  preferences later.
- It stays enabled with nothing selected (own design; the reference greys
  it [R1]).
- Clicking it does not keep keyboard focus. Keyboard choice of a dot is an
  open question shared with `toolbars.md` (keyboard navigation of toolbars).

#### Rotation (°)

- Own design where the reference is silent: a "turn by" field (Match the
  reference's observable behaviour, which reads zero at rest [R1]). It shows
  0 at rest. Committing a value turns the selection by that many degrees
  about the reference point, positive clockwise on screen (geometry.md,
  [R1]), then the field shows 0 again.
- It never shows an accumulated angle: a selection of several shapes has no
  single angle, and the shapes' transforms are not split into rotation and
  scale for display.
- Steppers and Up/Down keys turn the selection directly in 1° steps (15°
  with Shift, 0.1° with Ctrl), each step one command; the field stays at 0.
- Values beyond one turn are reduced to the range of one turn before the
  command runs; zero runs nothing.
- One command, one undo step ("Undo rotate").

#### mm / in (later)

- Two stacked choices filling the two lines: "mm" on top, "in" below
  (Match the 2.1 arrangement [R10][R8]). Each choice is a small radio dot
  (the same circle as a Reference point dot: `field` with `border` outline,
  chosen one filled `accent`) followed by its text in `type-small`.
- Hidden until inch display exists (decision 14 and `naming.md`: inches are
  a later preference). Until then all lengths are in millimetres and the
  labels say "(mm)".
- When it exists, choosing "in" changes every length label in the app from
  "(mm)" to "(in)", converts every shown value, and shows lengths with the
  inch precision from `naming.md`. The project file keeps millimetres; only
  display and typed input change.
- It changes design units only (lengths). Speeds stay in mm/s; whether a
  separate speed unit follows, and whether the two are linked as in the
  reference [R10], is an open question.
- Runs the front-end action `ui.units <mm|in>` (no project change, no undo).

#### Commit rules and commands

- Every commit follows "Shared field behaviour": Enter, Tab, focus loss;
  Esc reverts; one command per commit; unchanged values run nothing.
- The toolbar never computes geometry. It sends the typed target and the
  chosen reference point to a command (proposed change to shapes.md):

| Command (proposed) | Undo | Effect |
|---|---|---|
| `shape.move_to <x> <y> <point> [ids]` | set_transform | Move the shapes so the given point of their combined bounds lands on (x, y), in document coordinates |
| `shape.resize <w> <h> <point> [ids]` | set_transform | Scale the shapes so their combined bounds become w × h, keeping the given point fixed; w and h greater than zero |
| `shape.rotate_at <deg> <point> [ids]` | set_transform | Rotate the shapes about the given point of their combined bounds |

  `<point>` is one of `top-left`, `top`, `top-right`, `left`, `center`,
  `right`, `bottom-left`, `bottom`, `bottom-right`. The UI converts the
  typed bed coordinates to document coordinates with the project's mapping
  function before building `shape.move_to`. The command line accepts the
  same commands, so everything the toolbar does can be typed or scripted.
- Typing in Width with Keep proportions on runs one `shape.resize` carrying
  both new sizes, not two commands.

#### Enabled states

| Situation | X, Y | Width, Height | Width (%), Height (%) | Keep proportions | Reference point | Rotation (°) | mm / in |
|---|---|---|---|---|---|---|---|
| Nothing selected | disabled, empty | disabled, empty | disabled, empty | enabled | enabled | disabled, empty | enabled |
| One or more objects selected | enabled | enabled (a zero dimension disabled, above) | enabled (same) | enabled | enabled | enabled | enabled |
| Selection holds a locked shape (later) | read-only | read-only | disabled | enabled | enabled | disabled | enabled |
| Edit nodes tool active | disabled, values shown | disabled, values shown | disabled | enabled | enabled | disabled | enabled |
| A drag on the canvas in progress | values follow the drag live, not editable | same | same | enabled | enabled | same | enabled |
| Text placed on a path is selected (later) | disabled, values shown | disabled, values shown | disabled | enabled | enabled | disabled | enabled |

- Nothing selected: the fields are empty rather than showing the last
  selection's values greyed as the reference does [R1][R9]. Own design:
  stale numbers are easily read as the current state.
- A disabled field's tooltip second line says why: "Select something to
  edit its position and size.", "Leave Edit nodes to type a position or
  size.", "The selection includes locked shapes.", "Text on a path follows
  its path; move the path instead."
- The Edit nodes rule is own design: node editing changes geometry, not the
  transform, and typed whole-shape changes in the middle of it would be
  confusing. Typed node coordinates are an open question.
- The handle switches (below) do not disable these fields: a typed value
  is never accidental (own design; open question).

#### Multi-selection and groups

- Everything selected is one block (Match [R1]): one bounding box, one set
  of values; typing moves, resizes or turns all of it together, keeping
  every object's place in the block and its proportions inside the block.
  There is no Varies state in this toolbar.
- A group counts as one object (arrange.md); its members move with it.
- The values are those of the combined bounds of every selected shape,
  exactly what the Selection size status field shows (count and size), and
  the canvas selection bounds.

#### Live updates

- While the user drags a canvas handle or moves the selection, the fields
  show the bounds of the dragged state, updated every frame; on release they
  show the committed values. Esc during the drag restores them. Own design.
- After undo, redo, a command typed in the command line or any change from
  a menu, the fields show the new values at once (they are read from the
  project every frame, never cached).
- A field being edited is not overwritten by these updates; its edit wins
  until it is committed or reverted.

#### Narrow windows

- Folding follows `layout.md` (Toolbar model, Narrow and short windows):
  Text style folds first because it is to the right; then this toolbar
  folds from its end by whole groups: units, then rotation, then reference
  point, then the size group. The position group never folds.
- The More "»" button opens a small popup panel (not a menu) that shows
  the folded groups in the same two-line arrangement and with the same
  behaviour; it closes when focus leaves it.
- Before any group folds, the percentage labels shorten from "Width (%)"
  and "Height (%)" to "%", with the full label kept as the field's tooltip
  (`naming.md`, Tooltips: fields whose label is shortened).

Tier: **Match** (place, two-line columns and their order, one-block
behaviour, reference point meaning, rotation sign, units as two stacked
choices), **Echo** (lock toggle, point grid, steppers), **Own** (labels,
look and tokens, empty fields with no selection, enabled settings with no
selection, percent reset rule, commands, coordinate mapping in the project
crate, the canvas reference-point cross, fold order).

## Text style toolbar

### Reference

- A toolbar right of the numeric bar in the same row, for the font, size,
  spacing, anchoring and style flags of selected text, plus a mode for
  changing data [R4][R8].
- Its left block has a font dropdown that can be typed into, then a height
  field; under them a small grid of switches: bold and italic in the first
  column, capitals and letter distortion in the second, welding and (only
  when enabled in the settings) right-to-left in the third [R4][R9].
- Its right block has two lines: letter spacing, horizontal anchoring and a
  text mode dropdown on top; line spacing, vertical anchoring and a counter
  offset below [R4][R7].
- Spacing values are percentages, negative values pulling characters or
  lines closer; the height is roughly the height of a capital X [R4].
- The horizontal anchor offers left, middle or right; the vertical one
  bottom, top or middle; on a path they place the text along and across the
  path [R4][R6].
- The text mode offers plain text and several data sources (date and
  time, merged CSV, a serial number, the layer's settings); the offset adds
  to the counter for that object only [R7].
- The font list comes from installed fonts plus single-line fonts from a
  chosen folder, with favourites and recent fonts first; right-clicking the
  font box adds an entry that opens a font browsing window [R4][R10].
- With no text selected the whole toolbar is greyed; switches lose their
  colour [R4][R9]. By default the click point of the text tool is the
  middle of the new text [R4].

### LaserCraft

Purpose: choose the font and capital height for the next text now; later,
also edit every style setting of selected text shapes. Default place: toolbar
row 2, after Position and size (`layout.md`). Window → Toolbars entry: "Text
style". Milestone: later.

#### What exists now and what comes later

- **Now** (M3): text is converted to outlines when it is created
  (text.md). The only text settings are the Add text window's Font (system
  families) and Capital height (mm), which start every time at text.md's
  defaults (dialogs.rs:79-87).
- **Toolbar, first stage (later, no text.md change needed)**: the toolbar
  shows Font and Capital height (mm) as the **next-text defaults**. They are
  always enabled. The Add text window opens with them, and pressing OK in
  that window writes the values it used back to the toolbar, so the two
  never disagree. They are front-end settings: no command, no undo, not
  saved in the project; remembered for the session, saved with the
  preferences later ("Default font", "Default capital height (mm)",
  `naming.md` Text preferences).
- **Toolbar, second stage (later, owner decision)**: every other item needs
  editable text objects, which text.md today rules out (proposed change to
  text.md). Until then those items are not drawn at all: the first stage
  shows only Font and Capital height (mm), so the toolbar is short and
  shows nothing that cannot work. Own design; the reference shows all items
  greyed [R4].

#### Arrangement (second stage)

Two `row`s tall, columns left to right:

| Column | Top line | Bottom line | Group |
|---|---|---|---|
| 1 | Font (wide editable dropdown) | Bold, Italic, All caps, Weld letters, Bend letters along arc, Right to left (switches in one line) | Font and style |
| 2 | Capital height (mm) | (switches continue under it if they need the width) | Font and style |
| — | separator | | |
| 3 | Letter spacing (%) | Line spacing (%) | Spacing |
| — | separator | | |
| 4 | Horizontal align | Vertical align | Anchor |
| — | separator | | |
| 5 | Text source | Counter offset | Variable text |

- The reference stacks its style switches in three thin lines under the
  font [R4]; LaserCraft keeps the two lines of the row and puts the
  switches side by side in the bottom line, labels in `type-small` (Echo).
- The Font dropdown is about as wide as three numeric fields (Match the
  proportion [R4]); the other fields share the numeric field width of
  "Shared field behaviour"; the align and source dropdowns are as wide as
  their longest choice.
- First stage: only column 1's Font and column 2's Capital height (mm), on
  the top line; the bottom line is empty.

#### Items

| # | Label (exact) | Kind | Choices or range | Default | Applies to | Tooltip (first line / second line) | Stage |
|---|---|---|---|---|---|---|---|
| 1 | Font | editable dropdown | installed font families; typing filters the list | text.md's default family | next text; selected text (stage 2) | "Font family" / "Right-click for the font browser." | 1 |
| 2 | Capital height (mm) | numeric field | text.md's range | text.md's default (10 mm) | next text; selected text | "Height of capital letters" / "Some fonts draw slightly larger or smaller." | 1 |
| 3 | Bold | switch | on, off | off | selected text | "Use the font's bold face" / "Only when the font has one." | 2 |
| 4 | Italic | switch | on, off | off | selected text | "Use the font's italic face" / "Only when the font has one." | 2 |
| 5 | All caps | switch | on, off | off | selected text | "Show every letter as a capital" | 2 |
| 6 | Weld letters | switch | on, off | off | selected text | "Merge touching letters into one outline" / "Gives one clean outline where letters overlap." | 2 |
| 7 | Bend letters along arc | switch | on, off | off | selected bent text | "Curve each letter with the arc" / "Off keeps letters straight." | 2 |
| 8 | Right to left | switch | on, off | off | selected text | "Lay the text out from right to left" | 2, only shown with the Right-to-left text preference |
| 9 | Letter spacing (%) | numeric field | negative allowed | 0 | selected text | "Extra space between letters" / "A share of the capital height; negative pulls letters together." | 2 |
| 10 | Line spacing (%) | numeric field | negative allowed | 0 | selected text | "Extra space between lines" / "0 keeps the font's own spacing." | 2 |
| 11 | Horizontal align | dropdown | Left, Center, Right | Left | selected text | "Which side of the text sits on its anchor point" | 2 |
| 12 | Vertical align | dropdown | Top, Center, Baseline | Baseline | selected text | "Which height of the text sits on its anchor point" / "Baseline is the first line's baseline." | 2 |
| 13 | Text source | dropdown | Plain, Date and time, From CSV, Counter, Layer speed and power | Plain | selected text | "Where the text comes from" / "See the Text data panel for counters and CSV files." | 2 |
| 14 | Counter offset | whole-number field | negative allowed | 0 | selected text | "Added to the counter for this shape only" / "Lets copies print consecutive values." | 2 |

- Defaults for Horizontal align and Vertical align keep today's placement
  (text.md: the click point is the start of the first baseline). The
  reference centres new text on the click point [R4]; whether to follow it
  is an open question.
- Choice values are our own words. "Center" is used on both dropdowns
  rather than the reference's "middle" wording.
- Weld letters defaults to off so new text keeps today's output (separate
  letter outlines, text.md); the reference shows it on [R4]. Open question.
- Counter offset is enabled only when Text source is Counter or From CSV
  (for CSV it skips rows). Bend letters along arc is enabled only for text
  bent with the Arc handle (`canvas.md`, later).

#### Font field

- An editable dropdown: a `field` box showing the family name in
  `type-body`, with a small down chevron at the right end in `text-muted`.
  Typing filters the open list to families containing the typed text; Enter
  picks the first match; Esc closes the list and reverts.
- The list opens below the field as a `field` popup with one row per
  family, the current one on an `accent-soft` row, the hovered row on
  `accent-soft` too (`menus.md` row look). Each family name is drawn in the
  UI font, not in its own face (own design; previews in the family's own
  face belong to the Font browser).
- Right-click inside the field opens a small menu: Cut, Copy, Paste,
  Select all, a separator, then "Font browser…" (the dialog in `dialogs.md`,
  later). The reference offers a similar edit menu with an entry for a font
  browsing window [R4][R10].
- An unknown family typed and committed shows the Invalid state and the
  message "Font ‹name› is not installed" (text.md's wording, sentence case).
- Single-line (SHX) fonts and a mark for them in the list are later
  (`naming.md`, Text preferences).

#### Selected text (second stage)

- The toolbar edits **text shapes** only. With a selection that holds at
  least one text shape, every item shows the text shapes' values (Varies
  state where they differ) and a change applies to every selected text
  shape, as one command (one undo step); other shapes in the selection are
  left alone. Font and Capital height (mm) then show the selection, not the
  next-text defaults, and changing them does not change the defaults.
- With no text shape selected, Font and Capital height (mm) show the
  next-text defaults (enabled) and items 3–14 are disabled: labels and
  values `text-disabled`, switches in their disabled look (faded,
  decision 4), dropdowns `field-alt`. Tooltip second line: "Select a text
  shape to change this."
- Text on a path (later, "Put text on path"): every item stays editable
  (Match [R6]).
- Each change runs a text command proposed for text.md (for example
  `text.set_style <property> <value> [ids]`); the toolbar holds no text
  layout logic.

#### Switch look

- Every switch is the pill switch of decision 4 and `visual-language.md`:
  `accent` track with the knob at the right when on, the `text-muted`-family
  off track with the knob at the left when off, faded when disabled (the
  Switches table in `visual-language.md`). The reference's green and
  dark red are not used.
- Label after the switch, `type-small` in `text`, `gap-tight` between.
  Clicking the label toggles too.
- Switches do not keep keyboard focus after a click (like toolbar
  buttons).

Tier: **Match** (place, the font-then-size start, spacing and anchor
columns, two lines), **Echo** (switches in the bottom line, editable font
dropdown with a browser entry), **Own** (labels, choices, defaults, the
two-stage rollout, hidden rather than greyed items in stage 1, switch look).

## Properties panel (later)

### Reference

- A dock panel that lists editable properties of the selection; its rows
  change with the kind of object. It is off by default and, when turned on
  from the Window menu, joins the upper right dock group as another tab,
  with tabs along the bottom edge [R3][R12].
- Every object kind starts with the same three rows, in this order: a
  whole-number ordering value (lower runs earlier; used only when the
  optimiser orders by it), a percentage that scales the shape's power within
  its layer's minimum and maximum, and a lock switch [R3][R5].
- Then rows by kind: for an image, tone and sharpening controls (a midtone
  curve value, contrast, brightness, a sharpening radius and amount, and
  noise reduction); for an ellipse, width and height; for a polygon, width,
  height and the number of sides; for a rectangle, width, height and a
  corner radius (negative values notch the corners); for text, a maximum
  width with a horizontal-only shrink switch and a switch that skips empty
  data fields [R3].
- For built-in stars the reference lists a point count, a value that curves
  each point from tip to base, and a ratio between the inner and outer
  radius; its two-radius star builds two sets of points, takes the count
  per set (so the total is double) and has a second ratio for the second
  set; the curve and ratio values run from 0 to 1 [R16].
- With several shapes selected, a change applies to all of them [R3].
- The form is a narrow two-column block (right-aligned labels without
  colons, then fields) pushed to the right of the panel and centred
  vertically when there are few rows; booleans are small switches placed
  where a field would start; size fields show no unit [R3].
- A locked shape refuses every kind of edit; locking is also offered in the
  arrange menu and the right-click menu; how a locked shape looks on the
  canvas is not shown [R5].
- The notes do not cover plain paths beyond the three common rows, nor
  groups [R3].

### LaserCraft

Purpose: show and edit settings that belong to individual shapes rather
than to their layer. Dock place: the right dock's upper group, tab 5, after
the other tabs (`layout.md`, Right dock; Window → Panels entry
"Properties"). Shown by default, as tab 5 (`layout.md`; the reference
keeps its panel off by default [R3][R12], see Open questions).
Milestone: later; image rows M7.

#### Layout

- Panel chrome (title strip, tab, scrolling) is `layout.md`'s.
- Top line: the kind of what is shown, in `text-muted` `type-body`:
  "Rectangle", "Ellipse", "Line", "Polyline", "Path", "Text", "Image",
  "Group of ‹n› shapes", or "‹n› shapes" for a selection of different kinds. Own
  design.
- Then a one-column form: labels right-aligned in a left column, fields in a
  right column (the density rule of `visual-language.md`). Rows are
  top-aligned, with `gap-tight` between rows; the form does not move when
  the kind changes and leaves the panel's free space below it (own design;
  the reference centres a short form vertically [R3], which makes it jump).
- A thin `border` separator with `gap` around it divides the common rows
  from the kind rows.
- Switches stand at the start of the field column with no text after them;
  their label is in the label column (Echo [R3]).
- Numeric fields follow "Shared field behaviour" (steppers, commit rules,
  one command per commit). Sliders (image rows) have a numeric field after
  them; dragging the slider previews live and commits one command on
  release.
- Empty state (nothing selected): the line "Select one shape to see its
  settings." in `text-muted` (`naming.md`, Empty states), top-aligned.

#### Common rows (every kind, always first, in this order)

| # | Label (exact) | Kind | Range | Default | Meaning | Tooltip (first line / second line) |
|---|---|---|---|---|---|---|
| 1 | Run order | whole-number field | 0 and up | 0 | lower numbers run earlier when the Cut order dialog uses run order | "Order in which this shape runs" / "Lower runs first; used when cut order follows run order." |
| 2 | Power (% of layer) | numeric field | 0–100 | 100 | this shape's share of its layer's power | "Burn this shape at a share of its layer's power" / "Never above the layer's or the device's Max power (%)." |
| 3 | Locked | switch | on, off | off | the shape refuses every edit | "Protect this shape from changes" / "Unlock it to move, resize or delete it." |

- **Run order** (proposed change to planning.md and project-file.md): a new
  per-shape number, used only when the Cut order dialog chooses to order by
  it; otherwise ignored.
- **Power (% of layer)** (proposed change to layers.md, planning.md,
  gcode.md and project-file.md): scales the layer's power for this shape.
  The range stops at 100, so a shape can only burn at or below its layer;
  the result is then clamped to the layer's and the device's maximum like
  every power value (safety rule 6). Whether it scales Max power only or
  both Min and Max power is an open question; the reference scales between
  the layer's minimum and maximum [R3].
- **Locked** (proposed change to shapes.md): a locked shape cannot be
  moved, resized, rotated, node-edited, deleted, moved to another layer or
  combined; commands that would change it fail with "‹n› shapes are
  locked" and change nothing. It can still be selected, copied and
  unlocked. The same lock is in the Arrange menu ("Lock", "Unlock",
  `menus.md`) and the canvas right-click menu (Match [R5]). Its canvas look
  is `canvas.md`'s (proposed there: the selection bounds and handles in
  `text-disabled`).
- While Locked is on, every other row of this panel is Read-only, and the
  Position and size toolbar is read-only for that selection (above).

#### Rows by kind (after the common rows)

| Kind | Rows, in order (exact labels) | Meaning | Milestone |
|---|---|---|---|
| Rectangle | Width (mm), Height (mm), Corner radius (mm) | the rectangle's own width and height before its transform; corner rounding (Corner radius: proposed change to shapes.md) | later |
| Ellipse | Width (mm), Height (mm) | the ellipse's own diameters before its transform | later |
| Line, Polyline | (none) | only the common rows | later |
| Path (imported art, outline text, boolean results) | (none) | only the common rows | later |
| Text (editable text, second stage) | Max width (mm), Shrink width only, Skip empty fields | width limit (0 means no limit); when the limit is hit, narrow the letters instead of shrinking the whole text; drop line breaks before empty CSV fields | later; proposed change to text.md |
| Image | Gamma, Contrast, Brightness, Sharpen radius, Sharpen amount, Denoise | tone and sharpening of this image only; a button "Image adjustments…" after the rows opens the full window | M7 |
| Polygon (when it exists) | Width (mm), Height (mm), Sides | the polygon's own size; the number of sides, a whole number of at least 3 | later; proposed change to shapes.md |
| Star (when it exists) | Width (mm), Height (mm), Points, Inner radius (%) | the star's own size; the number of points, a whole number of at least 3; the inner corners' distance from the centre as a percentage of the tips' distance (0 to 100) | later; proposed change to shapes.md |
| Alternating star (when it exists) | Width (mm), Height (mm), Points, Inner radius (%), Second inner radius (%) | as Star, except that the points come in two alternating sets: Points is the count of each set (the star has twice as many points, Echo [R16]), Inner radius (%) shapes the first set and Second inner radius (%) the second, so neighbouring points differ in length | later; proposed change to shapes.md |
| Group | (none) | the common rows act on every shape in the group | later |

- **Star point curve is left out** (own design): the reference's value that
  bends each star point into an arc [R16] has no field in LaserCraft. Stars
  have straight sides; a curved star is drawn as a path. If the owner wants
  it, the field would be "Point curve (%)" (0 to 100) after Inner radius
  (%), listed in Open questions.
- **Ratios as percentages** (own design): the reference's 0 to 1 ratios
  [R16] are shown as percentages, like Power scale, so all relative values
  in this panel share one unit.
- **Width (mm) and Height (mm) here are not the toolbar's**: they are the
  shape's own size in its local geometry (shapes.md: a rect's width and
  height, an ellipse's two radii doubled), unchanged by rotation. The
  toolbar's Width and Height are the axis-aligned bounds of the whole
  selection. For an unrotated, unscaled shape both agree. Changing them runs
  `shape.set_geometry` (nodes.md) with the new size, keeping the shape's
  centre fixed (own design).
- **Image rows** follow the reference's order (Match [R3]); labels are
  `naming.md`'s. Ranges, neutral values and units are the M7 image spec's;
  each row is a slider with a numeric field.
- The reference does not cover paths or groups [R3]; LaserCraft shows only
  the common rows for them. Read-only facts such as node count or length
  are the Measurements dialog's (later), not this panel's (open question).

#### Several objects selected

- One shape or one group: all rows for its kind.
- Several shapes or groups: the common rows, and the kind rows only when
  every selected shape has the same kind. Values that differ show the Varies
  state. A change applies to every selected shape as one command (Match
  [R3]).
- The kind line says "‹n› shapes" for a selection of different kinds.
- `naming.md` describes the panel as showing one selected shape; this file
  extends it to several (open question, so the owner confirms).

#### States

| State | Look |
|---|---|
| Nothing selected | empty-state line only |
| Selection shown | rows as above |
| Locked on | Locked enabled; all other rows Read-only |
| Edit nodes active | rows stay editable (they do not change node positions) |
| A canvas drag in progress | values follow the drag live where they depend on it (none of the common rows do) |
| Panel hidden or in a background tab | nothing is computed for it |

Tier: **Match** (dock place as a tab of the upper group, common rows first
in that order, rows by kind, image row order, apply to all selected),
**Echo** (switches in the field column), **Own** (labels, top-aligned form,
kind line, local-size meaning, rows for paths and groups, locked behaviour
details, power range).

## Handle switches (status bar)

### Reference

- Four small switches with word labels sit at the left end of the status
  bar, under the colour swatches; they are on by default [R2][R11][R8].
- Turning one off removes that kind of handle (moving, sizing, rotating,
  skewing) from every selection in the project and prevents that kind of
  edit [R2].
- They guard against accidental edits; the lock on single shapes is a
  separate feature [R5][R11].
- The reduced interface mode hides them [R2][R13].
- While rotating, the status bar shows the angle change [R2].

### LaserCraft

Place: the first field of the status bar, at its left end (`layout.md`,
Status bar, field 1). Labels: "Move", "Scale", "Rotate", "Skew" (prose:
handle switches). Milestone: later.

Look:

- Four pill switches (decision 4) at `type-small` scale, each followed by
  its word in `type-small` `text`, `gap-tight` between switch and word and
  `gap` between pairs. A thin vertical `border` line separates the group
  from Pointer position, like every status field (`layout.md`).
- On: `accent` fill, knob right. Off: the `text-muted`-family off
  track (`visual-language.md`, Switches), knob left. Disabled:
  faded. No green or red.
- Hover: the pair (switch and word) gets a `hover` background with slightly
  rounded corners and shows its tooltip.

Behaviour:

| Switch | When off, on the canvas | Still works when off | Tooltip (first line / second line) |
|---|---|---|---|
| Move | the Move handle is not drawn (later); pressing on a selected shape and dragging does not move it (a drag from a selected shape starts a selection box instead) | arrow-key nudges, X (mm) and Y (mm), Move on bed, align, paste | "Allow moving the selection by dragging" / "Off protects positions from stray drags." |
| Scale | the scale handles are not drawn | Width and Height fields, Mirror commands | "Allow resizing with the corner and edge handles" |
| Rotate | the rotate handle is not drawn | Rotation (°), the rotate chords and menu items | "Allow turning with the rotate handle" |
| Skew | the skew handles are not drawn (later) | nothing else skews | "Allow slanting with the skew handles" |

- Own design where the reference is silent: the switches only stop
  **pointer gestures** on the canvas. Typed values, chords and menu
  commands are deliberate and keep working (open question).
- When a gesture is stopped, the status bar's Messages field shows a hint
  on `info-soft`: "Dragging to move is off. Turn on Move in the status bar
  to drag shapes." (and the same pattern for the other switches). The hint
  clears when the pointer is released.
- With all four off, the selection bounds are still drawn, without handles.
- In the Edit nodes tool they have no effect (that tool hides the
  selection handles anyway, `canvas.md`).
- They act for the whole current project: every selection in it (Match
  [R2], `naming.md`, `canvas.md`). Each open project keeps its own four
  states; they are not saved in the project file (whether to save them, per
  project or in the preferences, is open question 2). Changing one runs the front-end action
  `ui.handles <move|scale|rotate|skew> <on|off>` (new; no undo). They start on
  in every newly opened or created project.
- The Skew switch is shown only once skew handles exist; until then the
  group has three switches.
- Simple layout (later) hides the group (Match [R13]); handles then all
  work.
- Clicking a switch does not take keyboard focus from the canvas.
- The rotation angle while dragging the rotate handle appears in the Tool
  hint field (`layout.md`), Match [R2].

Tier: **Match** (place, four kinds, on by default, hidden in the reduced
layout), **Echo** (switch with word), **Own** (switch look, names, gestures
only, the hint, session scope).

## Keyboard focus and Tab order

### Reference

- The notes say only that a typed value is applied with Enter [R1]; the
  order in which Tab moves through these toolbars is not known.
- On the canvas, Tab and Shift+Tab step through objects [R14], which is also
  LaserCraft's binding (keymap.md).

### LaserCraft

Own design.

- **Getting in**: click a field. There is no chord that moves focus into the
  toolbar band (not bound; keymap.md has none). Tab on the canvas keeps its
  meaning (select the next object) and never enters a toolbar.
- **Tab order inside the band** (fields only, disabled fields skipped):

| # | Field | Toolbar |
|---|---|---|
| 1 | X (mm) | Position and size |
| 2 | Y (mm) | Position and size |
| 3 | Width (mm) | Position and size |
| 4 | Height (mm) | Position and size |
| 5 | Width (%) | Position and size |
| 6 | Height (%) | Position and size |
| 7 | Rotation (°) | Position and size |
| 8 | Font | Text style |
| 9 | Capital height (mm) | Text style |
| 10 | Letter spacing (%) | Text style (stage 2) |
| 11 | Line spacing (%) | Text style (stage 2) |
| 12 | Horizontal align | Text style (stage 2) |
| 13 | Vertical align | Text style (stage 2) |
| 14 | Text source | Text style (stage 2) |
| 15 | Counter offset | Text style (stage 2) |

  The order is column by column, top line before bottom line, left to
  right, which is the reading order of the two-line layout. After the last
  enabled field Tab wraps to the first; Shift+Tab goes backwards and wraps
  too. Fields folded into More are skipped while the popup is closed.
- **Not in the Tab order**: Keep proportions, the Reference point grid,
  mm / in, the style switches and the handle switches. They behave like
  toolbar buttons (`toolbars.md`): clicking them leaves focus on the canvas.
  Whether keyboard users get another way to reach them is the open question
  `toolbars.md` already lists.
- **Properties panel**: Tab moves through its rows top to bottom; here the
  switches (Locked, text switches) are in the Tab order and Space toggles
  the focused one, because a dock form is a form, not a toolbar. Tab from
  the last row wraps to the first row of the panel.
- **Getting out**: Esc reverts the focused field and returns focus to the
  canvas; clicking the canvas commits and returns focus there. Enter keeps
  focus in the field (Shared field behaviour).
- **Shortcuts while a field has focus**: ignored (keymap.md, Behavior),
  except the Pause and Ctrl+Break safety chords, which `layout.md` proposes
  to keep working in text fields (proposed change to keymap.md there).
- **Focus look**: the `accent` focus ring of "Shared field behaviour"; the
  toolbar shows no other focus mark.

Tier: **Own**.

## egui implementation notes

- Numeric fields: one shared widget built from `egui::DragValue` (for the
  drag and the typed edit) plus two small painted chevrons for the stepper,
  with `custom_formatter` and `custom_parser` for decimals and the later
  expressions. A typed commit is detected as the change that happens while
  not dragged (as layers_panel.rs:379 does today) and also on `lost_focus`;
  a drag commit on `drag_stopped`. Each commit emits one `Action` holding an
  invocation; the UI crate never applies changes itself.
- The two-line toolbars are an `egui::Grid` per group (two rows), laid out
  inside one horizontal strip; groups that span both lines are vertically
  centred cells.
- Reference point grid and lock: painted with `Painter` circles and the
  `icons.md` pictograms; their state lives in the shell's front-end state,
  next to the current tool.
- Font field: `egui::ComboBox` cannot be typed into, so the field is a
  `TextEdit::singleline` with a popup list below it (`egui::Popup` or an
  `Area`), filtered as the user types; the family list is the one the Add
  text window loads today.
- Switches: one shared painted switch widget (`visual-language.md`), used
  here, in the Properties panel and in the status bar.
- Tab order: egui moves focus in widget creation order, so the toolbars are
  created in the order of the Tab table; disabled widgets are not focusable.
- Tests (in `crates/lasercraft-ui/tests/`, headless): typing a value and
  pressing Enter emits exactly one invocation; Esc emits none; an unchanged
  value emits none; Keep proportions on gives one `shape.resize` with both
  sizes; the percent fields read 100 after a commit; Rotation (°) reads 0
  after a commit; fields are disabled with an empty selection; a Move
  switch that is off makes a drag on a selected shape start a selection box
  instead of a move.

## Deliberately omitted

- Nothing in this area is a controller-specific feature, so nothing is
  omitted for scope reasons (decision 1).
- The reference's arithmetic in fields is kept as a later item, not omitted.
- Single-line font folders and the font kind marks in the list are later,
  not omitted.

## Current state → target

| Today (current-ui.md) | Target (this file) |
|---|---|
| No toolbar band and no numeric position, size or rotation fields; the only size read-out is the status bar's "‹n› selected, ‹w› × ‹h› mm" (shell.rs:641-649, current-ui.md 1.10, 1.11) | Position and size toolbar in row 2 with typed X, Y, size, percent, rotation and the reference point grid; the status read-out stays |
| Pointer position shows document coordinates, rear-left origin with y down (shell.rs:637-639) | Pointer position and X/Y fields both in bed coordinates from the machine-origin corner (proposed change to canvas.md) |
| No `shape.move_to`, `shape.resize`, `shape.rotate_at` commands; the UI would need `shape.move` / `shape.scale_about` / `shape.rotate_about` and do the arithmetic itself (shapes.md) | The three proposed commands take the target and the reference point (proposed change to shapes.md) |
| The Add text window starts at text.md's default font and 10 mm every time (dialogs.rs:79-87); its height field is a `DragValue` with a " mm" suffix (dialogs.rs:131-136, 190) | Font and Capital height (mm) are the next-text defaults on the Text style toolbar and the Add text window reads and writes them; units move into the label |
| Booleans in panels are checkboxes (layers_panel.rs:390-402) | Pill switches everywhere in these areas (decision 4) |
| Layer editor numbers commit on drag end or typed entry (layers_panel.rs:358-388) | Same rule, extended with Tab, focus loss and Esc (proposed change to ui-shell.md) and shared by all fields here |
| Canvas has scale handles and one rotate handle (current-ui.md 1.11); no move or skew handles, no handle switches | Handle switches Move, Scale, Rotate (and Skew once skew handles exist) at the status bar's left end |
| No per-shape lock, run order or power share in the model (shapes.md) | Properties panel with Run order, Power (% of layer), Locked (proposed changes to shapes.md, layers.md, planning.md, gcode.md, project-file.md) |
| Text is outlines only (text.md) | Stage 2 of the Text style toolbar and the Text rows of Properties wait for editable text (proposed change to text.md, owner decision) |
| No theme set; egui defaults everywhere (current-ui.md 2) | Fields, switches and grids drawn with `visual-language.md` tokens |

## Proposed changes to other specs

Each needs the owner's agreement and an update of that spec before code
(decision 15).

- **Proposed change to ui-shell.md**: add the toolbar band's row 2 with the
  Position and size and Text style toolbars (pointing to this file); extend
  the commit rule ("typed values when entered") to: Enter, Tab and focus
  loss commit, Esc reverts, an unchanged value runs nothing; add the handle
  switches to the status bar (with `layout.md`'s new field order).
- **Proposed change to shapes.md**: add `shape.move_to`, `shape.resize` and
  `shape.rotate_at` with a reference-point argument (table above); add a
  per-shape `locked` flag with `shape.lock [ids]` and `shape.unlock [ids]`
  (undo: the opposite command), and the rule that commands changing a
  locked shape fail with "‹n› shapes are locked"; add rectangle corner
  radius (with `naming.md`'s other shape-kind proposals); for the later
  polygon, star and alternating-star kinds, store the side or point count,
  the inner radius as a share of the outer radius and, for the alternating
  star, a second inner radius, all editable through Properties (no point
  curve unless the owner adopts it, Open questions).
- **Proposed change to canvas.md**: the Pointer position read-out uses bed
  coordinates from the machine origin; a small `selection` cross marks the
  chosen reference point on the selection bounds while the Position and
  size toolbar is in use; the handle switches stop the matching pointer
  gestures and hide the matching handles; locked shapes draw their
  selection bounds and handles in `text-disabled` and do not react to
  drags; field values follow canvas drags live.
- **Proposed change to text.md**: editable text objects (owner decision),
  with the Text style toolbar's second-stage items (style switches,
  spacing, alignment with Left/Center/Right and Top/Center/Baseline, Text
  source, Counter offset), text commands such as `text.set_style`, the
  Properties text rows (Max width (mm), Shrink width only, Skip empty
  fields) and the rule that the text tool's Font and Capital height come from
  the next-text defaults on the toolbar.
- **Proposed change to layers.md, planning.md and gcode.md**: a per-shape
  power share (Power (% of layer), 0–100, default 100) applied before the
  existing clamps to the layer's and device's maximum (safety rule 6
  unchanged); a per-shape Run order used by an optional cut-order rule.
- **Proposed change to project-file.md**: shape fields `locked`,
  `run_order` and `power_share` (names to be settled there), each optional
  with the defaults above, so older files load unchanged.
- **Proposed change to keymap.md**: list the new front-end actions
  `ui.units <mm|in>` and `ui.handles <move|scale|rotate|skew> <on|off>`
  (not bound to chords); no chord is added for focusing the toolbar band.
- **Proposed change to naming.md**: add to Messages and errors "Not a
  number: ‹text›", "‹n› shapes are locked" and the handle switches' hints
  ("Dragging to move is off. Turn on Move in the status bar to drag
  shapes." and the same pattern for Scale, Rotate and Skew); add to Empty
  states the placeholder "varies" for a field whose selected objects differ
  (chosen so it does not reuse the layer-mode name Mixed); add to the shape
  properties row "Second inner radius (%)" (Alternating star, Own) next to
  "Corner radius (mm), Sides, Points, Inner radius (%)", and record "Point
  curve (%)" as the reserved label should the star point curve be adopted.

## Open questions

1. **Coordinate frame**: switch X/Y and the Pointer position read-out to
   bed coordinates from the machine origin (this file, Match with [R1]), or
   keep today's document frame (rear-left, y down)?
2. **Reference point default**: center (this file) or a corner? Should the
   choice, Keep proportions and the handle switches be saved with the
   preferences, or per project?
3. **Empty fields with nothing selected** (this file) versus the reference's
   greyed last values.
4. **Handle switches' reach**: should they also block arrow-key nudges,
   rotate chords and typed values, or only pointer gestures (this file)?
5. **Expressions and unit suffixes** in fields: wanted, and in which crate
   should the evaluator live? Should the command line accept them too?
6. **Units**: when inches arrive, does mm / in change lengths only, or also
   speeds, and should design and machine units be linkable as in the
   reference [R10]?
7. **Editable text objects** (text.md today: outlines only): approve stage 2
   of the Text style toolbar, the Text rows of Properties and Text data?
8. **New text placement**: keep Left/Baseline (today's behaviour) or centre
   new text on the click point like the reference [R4]?
9. **Weld letters default**: off (today's output) or on (the reference's
   screenshots)?
10. **Properties with several shapes**: confirm that several shapes show
    the common rows and apply changes to all (this file, Match [R3]),
    although `naming.md` describes the panel for one shape.
11. **Power (% of layer)**: scale Max power only, or Min and Max power (the
    reference scales between them [R3])? Should values above 100 ever be
    allowed (this file: no)?
12. **Run order** needs a cut-order rule in planning.md that uses it; which
    dialog switches it on?
13. **Keyboard access** to Keep proportions, the Reference point grid, the
    units choice and the switches (shared with `toolbars.md`'s open
    question on toolbar keyboard navigation).
14. **Typed node coordinates** while Edit nodes is active (this file
    disables the toolbar there).
15. **Read-only facts** (node count, path length, area) in Properties for
    paths, or only in the Measurements dialog (this file)?
16. **Mouse wheel over a focused field** steps the value (this file); some
    users expect the wheel to scroll the panel instead.
17. **Properties default visibility**: this file follows `layout.md`
    (shown, tab 5); the reference keeps its panel off by default [R3][R12].
    Should LaserCraft hide it by default instead (a proposed change to
    `layout.md`)?
18. **Star point curve**: the reference lets star points bow into arcs
    [R16]. This file leaves it out (straight-sided stars only). Should a
    "Point curve (%)" row be added after Inner radius (%) for Star and
    Alternating star (a proposed change to shapes.md)?

## Sources

All LightBurn pages are part of the public 2.1 documentation, read
2026-10-21 through the style guide's research notes (property bars, canvas,
layout). They support the Reference parts only; no wording is taken from
them.

1. [R1] https://docs.lightburnsoftware.com/2.1/Reference/NumericEditsToolbar/ — numeric position and size toolbar: place, items and order, one-block behaviour, grid point meaning, lock looks, rotation sign, units control, expressions and unit suffixes, Enter to apply, greyed with no selection, overflow chevron
2. [R2] https://docs.lightburnsoftware.com/2.1/Reference/TransformControls/ — canvas handles and their status bar switches: place, on by default, project-wide effect, hidden in the reduced mode, rotation angle in the status bar, rotation field takes negative angles
3. [R3] https://docs.lightburnsoftware.com/2.1/Reference/ShapeProperties/ — properties panel: dock place, common rows and their order, rows per object kind, apply to all selected, form layout, switches in the field column
4. [R4] https://docs.lightburnsoftware.com/2.1/Reference/Text/ — text options toolbar: items, switch grid, spacing and anchor meanings, right-to-left only when enabled, greyed without text, font list sources and right-click entry, default click placement
5. [R5] https://docs.lightburnsoftware.com/2.1/Reference/LockShapes/ — per-shape lock: refuses every edit, three places to lock and unlock, no canvas picture
6. [R6] https://docs.lightburnsoftware.com/2.1/Reference/ApplyPathToText/ — text on a path: position and size fields greyed, text options still editable, anchoring along a path
7. [R7] https://docs.lightburnsoftware.com/2.1/Reference/VariableText/ — text mode choices and the per-object counter offset
8. [R8] https://docs.lightburnsoftware.com/2.1/Reference/UI/ — default window map: the numeric and text toolbars in row 2, the status bar switches, the 2.1 units pair
9. [R9] https://docs.lightburnsoftware.com/2.1/GetStarted/UITour/ — numeric and text toolbar contents in a region tour, greyed states with no selection
10. [R10] https://docs.lightburnsoftware.com/2.1/NewFeatures/NewIn2.1/ — units toggle replaced by two radio choices, design and machine units split or linked, font manager from the font list
11. [R11] https://docs.lightburnsoftware.com/2.1/Reference/UI/StatusBar/ — status bar layout with the switches at its left end, followed by the pointer coordinates
12. [R12] https://docs.lightburnsoftware.com/2.1/Reference/UI/WindowMenu/ — Window menu entries that show the numeric toolbar, the text toolbar and the properties panel
13. [R13] https://docs.lightburnsoftware.com/2.1/Reference/BeginnerMode/ — the reduced mode hides the handle switches
14. [R14] https://docs.lightburnsoftware.com/2.1/Reference/Selection/ — Tab steps through objects; a selection fills the position fields and enables the text options
15. [R15] https://docs.lightburnsoftware.com/2.1/Guides/CustomizingTheLightBurnWindow/ — toolbars shown, hidden and clipped with an overflow chevron
16. [R16] https://docs.lightburnsoftware.com/2.1/Reference/PrimaryShapes/ — built-in star properties: point count (per set for the two-radius star), point curve and inner ratio (0 to 1), and the second ratio of the two-radius star

Existing specs and documents used: `docs/spec/ui-shell.md` (commit rule for
dragged and typed values, status bar, Actions only), `docs/spec/canvas.md`
(selection bounds, handles, drag rules, Edit nodes hides handles, pointer
read-out), `docs/spec/shapes.md` (commands, invertible transforms, kinds),
`docs/spec/geometry.md` (document coordinates, clockwise rotation),
`docs/spec/arrange.md` (groups as one object), `docs/spec/text.md` (outline
text, defaults, capital height range, font errors), `docs/spec/nodes.md`
(`shape.set_geometry`), `docs/spec/layers.md` and `docs/spec/planning.md`
(power, cut order), `docs/spec/commands.md` (one undo step, gestures),
`docs/spec/keymap.md` and `crates/lasercraft-ui/keymap/default.keymap`
(nudge steps, rotate chords, Tab on the canvas, shortcuts ignored in text
fields; no chord is added here), `docs/spec/safety.md` (rules 5 and 6),
`docs/clean-room.md`, and the style files `layout.md`, `naming.md`,
`toolbars.md` and `menus.md`. Open standards: WCAG 2.2 (focus visible and
non-text contrast, for the focus ring and the grid dots),
https://www.w3.org/TR/WCAG22/.

Own design: every label, tooltip and message; the field anatomy and its
state tokens; right-aligned values; commit on Tab and focus loss, Esc to
revert, nothing run for unchanged values; step sizes, chevron repeat as one
gesture, wheel only on focused fields, drag on the value; empty fields with
no selection; settings (lock, reference point, units) enabled without a
selection; the center default; the "turn by" rotation that resets to zero;
the percent fields' reset rule and preview; zero-size handling; bed
coordinates through a project-crate mapping; the proposed `shape.move_to`,
`shape.resize`, `shape.rotate_at`, `ui.units` and `ui.handles`; the
reference-point cross on the canvas; the star rows as percentages, the
Second inner radius label and leaving out the point curve; fold order and the More popup panel;
the Text style toolbar's two stages, choices, defaults and switch line; the
font field's filtering and menu; the Properties panel's top-aligned form,
kind line, local-size meaning, rows for paths and groups, locked behaviour
and power range; the handle switches' gesture-only scope, hint and session
scope; the Tab order; the egui notes and tests.
