# Spec: style — icons and cursors

Milestones: M1–M5 (every icon whose action exists today: files, history,
clipboard, view, arrange, drawing tools, shape operations, job preview),
M6 (machine, job bar, jog and connection icons, Move head cursor), M7
(image icons), M8 (device profile, test and material icons), M9 (camera,
rotary and print registration icons), M10 (app icon for the release
packages); the icon set itself, the cursor badges and the icon pipeline are
built by the milestone that builds the toolbar band (later, no milestone
yet).

This file owns LaserCraft's pictograms: the drawing rules every icon
follows, the complete list of icon names (ids) with the generic concept each
one draws, the cursors and cursor badges of the canvas, the direction for
the app icon, how icons get into egui, where their source files live and the
checklist reviewers use to reject an icon that looks copied. Everything here
is **own design** (decision 17): the reference is described only so that we
know what to stay away from. Which buttons exist, in which order and with
which tooltip is `toolbars.md`'s; which menu rows carry an icon is
`menus.md`'s; the job buttons and jog pad are `machine-panels.md`'s; message
boxes and dialog pictograms are `dialogs.md`'s; canvas markers, handles and
which cursor shows when are `canvas.md`'s; the Layers panel's buttons are
`layers-panel.md`'s; the camera, material, test and registration windows are
`later-windows.md`'s; the job preview is `job-preview.md`'s; title strips,
the More button and the job bar are `layout.md`'s; the lock and stepper
pictograms of the field row are used by `property-bar.md`. Labels come from
`naming.md`; colors, type and sizes are only the tokens of
`visual-language.md`; `README.md` explains the tiers (Match, Echo, Own).
Where another style file names an icon, it uses the id from this file; where
the two disagree today, the difference is listed under "Proposed changes to
other specs".

## How to read this file

- **Icon name (id)**: lower-case kebab case, unique, never a size token name
  (`icon`, `icon-small`, `icon-big`) and never a color token name. The id is
  the file name of the icon's source (`<id>.svg`, see "Where icon source
  files live") and the name the toolbar and menu data use. Ids already used
  by `toolbars.md`, `layers-panel.md` and `canvas.md` are kept as they are.
- **Used by** lists every place the icon appears, by LaserCraft label from
  `naming.md`: toolbar buttons, menu rows, flyout rows, panel buttons, job
  and jog buttons, message boxes, status bands, switch states, cursor
  badges. One icon has one drawing everywhere (decision 17: same icon in
  the toolbar and in its menu row).
- **Concept** is a generic metaphor in our own words. It is a starting
  point for the person drawing the icon, not a drawing. It never describes
  the reference's artwork.
- **Accent detail** names the one part drawn in `accent`, or "none".
- **Notes** give the milestone (*Now*, *M6*–*M10*, *Later*, as in
  `naming.md`), aliases, and anything a developer must know.
- An icon is drawn only once its action exists (the rule of `toolbars.md`
  and `menus.md`): the set below is the full target, not a list to build in
  one go.

## The reference, in general terms

### Reference

- Toolbar buttons carry a pictogram only, with no caption; the name and the
  shortcut come from a tooltip [R1][R11][R13].
- Most menu rows that repeat a toolbar action show the same pictogram as
  that button in a narrow column at the left of the row; rows without a
  toolbar twin mostly have none; since 2.1, blank placeholders keep the
  labels of icon-less rows aligned [R11][R4][R6][R28].
- Toggle rows in the window menu show a check box in that same column
  instead of a pictogram [R9]. The menu bar's own titles carry no icons
  [R3].
- Docked panels' title strips carry two small pale glyph buttons, one to
  float the panel and one to close it [R33][R26].
- The pictograms are dark and monochrome, drawn heavy, mostly as filled
  silhouettes and thick strokes on a transparent ground, all one size within
  a toolbar [R1][R12][R13].
- Color is rare and small: a colored corner marker on buttons that hold a
  family of tools [R13][R16][R18]; a red-centred target on the action that
  moves things to the head's position [R2][R31]; small colored marks on the
  node-editing strip, and a recolored node tool while it is active [R17];
  some help rows carry the program's brand mark [R10].
- A button or row that cannot act is drawn with a washed-out pale grey
  pictogram; a family button's corner marker greys with it [R1][R5][R14]
  [R18].
- The current canvas tool sits on a soft rounded background [R13].
- The job-control panel's large buttons put a glyph left of their word; two
  framing buttons share one word and differ by glyph only; the jog pad's
  buttons are glyph-only [R26][R27].
- Message boxes show a warning triangle or a question circle at the left of
  their text; 2.1 renewed the message-box icon set and added a warning icon
  that reads better when small [R32][R28].
- The settings window's tabs and some dialogs' title strips carry small
  pictograms; device lists show vendor logos [R29][R26][R30].
- A display setting scales toolbar icons, another scales cursors and node
  marks; 2.1 picks a suitable icon size for the screen by itself [R29][R28].
- Cursors: a thin crosshair with a hollow centre and a small mode badge at
  its lower right for drawing and rounding, with a red mark when rounding
  has no room; scissors for trimming; a plus with a small tab glyph for
  tabs; one glyph per snap kind; arrows for moving, resizing and rotating;
  an open hand while panning; a line glyph over the rulers [R19][R20][R21]
  [R22][R23][R25][R24][R34].

### LaserCraft (what we do differently, on purpose)

- **Outline, not silhouette**: medium-weight strokes with round joins on an
  open ground, where the reference draws heavy filled glyphs. This is the
  single biggest visual difference between the two sets and it is fixed
  (decision 17).
- **One accent, one meaning**: at most one detail per icon in `accent`, and
  it always marks *the part the action creates or changes* (the plus of
  New, the axis of a mirror, the reference line of an align). The
  reference's mix of teal, green, red and cyan marks is not used; our
  flyout marker is neutral (`toolbars.md`).
- **Red only for stop, unrecoverable deletes and errors**: `danger`
  appears only on Stop, on deletes that Undo cannot bring back
  (`delete-stored`) and on the error state marks (`mark-error`,
  `message-error`, `badge-failed`); see "Color" below. No icon that stands
  for the head is red; the head is a neutral crosshair ring (decision 17;
  the canvas head marker and its colors are `canvas.md`'s, not an icon).
- **Own metaphors where a choice exists**: where the reference's concept
  for an action is known from the notes and another generic metaphor works
  equally well, ours differs (see "Metaphors we avoid" in the checklist).
  Universal metaphors that no one owns (scissors, magnifier, padlock, house,
  waste bin, folder, clipboard, the play, pause and stop symbols) are kept,
  drawn in our own style.
- **No brand marks, no vendor logos, no logo in title strips** (decision 1,
  `dialogs.md`, `machine-panels.md`).
- **Disabled stays legible**: disabled icons use `text-disabled` strokes,
  not an embossed or washed-out effect, and keep their shape.
- **Icons do not change color with state**: the active tool, hover and
  pressed states change the background only (`toolbars.md`), unlike the
  reference's recolored node tool [R17].

Tier: **Echo** (icon-only toolbar buttons, the same icon in the menu row,
disabled greying, a mode badge on cursors), **Own** (every drawing, the
style, the accent rule, every concept listed below).

## Icon style rules (LaserCraft, own design)

### Drawing grid and optical size

- Every icon is drawn on one square grid, the same for all ids. The grid
  is fine enough that every stroke and every key point sits on a whole
  grid unit at the `icon` size; the icons module fixes the unit count once
  and every source file uses it (Open questions).
- A safe margin of about one stroke width is kept free on every side, so
  icons never touch the edge of their button or menu gutter and never get
  clipped by rounding.
- **Consistent optical size**: shapes are sized so they *look* equally
  big, not so they measure equally. A circle or a diagonal shape reaches a
  little further toward the margin than a square; a tall narrow object (pen,
  caliper, flag) uses the full height; a wide flat object (bars, barcode)
  the full width. Two neighbouring toolbar icons must look the same weight
  and the same size at a glance.
- Primary forms are centred on the grid. Small secondary marks (plus,
  minus, pencil, arrowhead, state mark) sit in the lower-right quarter
  unless the concept says otherwise, so the eye finds them in one place.
- Perspective, shading, gradients, drop shadows and 3D effects are never
  used. Everything is flat and seen straight on.

### Strokes, joins and fills

- **Stroke weight**: one medium weight for the whole set, clearly heavier
  than the canvas *hairline* and lighter than a bold glyph; at the `icon`
  size it looks about as heavy as the stems of `type-body` letters. Small
  marks may use the same weight; nothing uses a second, heavier weight.
- **Joins and caps**: round joins and round caps everywhere. No sharp
  miter tips.
- **Corners** of rectangles and squares are slightly rounded (a radius of
  about one stroke width), so the set reads friendly and stays apart from
  the reference's hard-edged glyphs.
- **Dash language** (own design, used the same way in every icon):
  - solid stroke: a shape, an object, a tool;
  - dashed stroke (short dashes, round caps): a selection box, a marquee,
    a laser-off path, the bed area being targeted;
  - dotted stroke (round dots): something removed, a ghost of the state
    before the action, a copy that will be made.
- **Fills**: icons are outlines. Fills are allowed only for: small dots
  (nodes, centre points, the head ring's centre), arrowheads, the flyout
  marker, the menu choice dot, and the three media symbols (start, pause,
  stop), which are filled with rounded corners so the job controls read at
  a glance on the big safety buttons. Every allowed fill is a convex shape
  (which also keeps painted icons simple, see "Implementation options").
- **Hatching** (a few short parallel strokes) stands in for "shaded" or
  "filled area" (invert selection, fill mode, interval test), never a solid
  area fill.

### Color

- **Base strokes** in `text` (near-black in the light theme; its dark-theme
  value comes from `visual-language.md`). In places where the whole
  control is quiet (title strip buttons, the More button at rest, list
  row pictograms such as the mode marks) the base strokes use `text-muted`
  as the owning file says.
- **Accent detail**: at most one part in `accent`. It marks what the action
  creates or changes. Many icons have none. An icon is never mostly accent.
- **Danger**: `danger` strokes or fill only on `stop`, `delete-stored`
  (deletes that Undo cannot bring back) and the three error state marks
  `mark-error`, `message-error` and `badge-failed`. This is the one rule;
  decision 17 names Stop and delete-everything, and the error marks are the
  state-mark use of `danger` that decision 6 allows. Nothing else is red,
  including the no-room cursor badge (`warning`). An undoable delete (Delete, Delete
  shapes, Remove linked copies) uses normal strokes, as `layers-panel.md`
  already decided.
- **State marks**: a small mark that reports a state may use the matching
  semantic token (decision 6): `warning` on `pause-motion-only` and
  `frame-low-power`; `success`, `warning` and `danger` on the camera status
  badges; the four status marks and message icons below. A state mark
  replaces the accent detail; an icon never has both.
- **Never**: teal (reserved for `guide`), green outside `success` marks,
  orange, any layer palette color inside an icon (the cursor badge's layer
  square is a separate chip element, see "Cursors"), gradients, or
  translucency used as a color.
- **Contrast**: base strokes against their background meet the WCAG 2.x
  non-text contrast target (3:1) in both themes, and so does the accent
  detail; disabled icons are exempt as WCAG allows, but must still show
  their shape. Text-sized marks inside big buttons (Stop) meet 4.5:1
  (`machine-panels.md`).

### No text inside icons

- No letters, words or typeset characters inside any icon. The only
  exceptions: the two-digit numbers drawn on palette swatches and the digit
  after the guide pictogram (`layers-panel.md`), which are labels beside or
  on swatches, not part of an icon drawing.
- The exclamation, question and information marks of the status icons are
  drawn as strokes and dots (a stem and a dot; a hook and a dot), not set
  in a font; they count as symbols.
- Consequences: the Text tool shows a text cursor and text lines, not a
  letter; the print registration icons tell target 1 and target 2 apart by
  which corner mark carries the accent, not by a digit (proposed change to
  `menus.md` and `later-windows.md`).

### Sizes and where each is used

| Token | Used for |
|---|---|
| `icon` | toolbar buttons (Standard, Arrange, Align and distribute, Drawing tools, Node tools, Shape operations, Slide and nest), the More button, the jog pad, the preset icon buttons, Refresh ports, the Keep proportions lock, the Camera list row pictogram |
| `icon-small` | the Layers panel's button column (`layers-panel.md`), menu gutter icons, flyout and More popup rows, compact job bar buttons, ordinary buttons with an icon left of the label (Home, Connect, Copy), title strip buttons (float, close), band close buttons, status band and notes band marks, Preferences tab pictograms, list row pictograms (mode marks, linked-entry chain, image row), camera header buttons and badges, cursor badges, check ticks and choice dots, the submenu arrow, stepper chevrons |
| `icon-big` | Job setups buttons, the Machine panel's big job and framing buttons, message box icons, Combine shapes result buttons, wizard option cards and step pictograms, the Find center diagrams |

- One drawing serves all three sizes: the source is scaled, never redrawn
  per size. At `icon-small` the stroke weight scales down with the icon;
  details that would merge (inner hatching, a third dash) are allowed to
  drop out only if the icon module draws a documented small variant
  (Open questions); otherwise the concept must read at `icon-small` as
  drawn.
- The later Icon size preference (`naming.md`, Display preferences) scales
  the three tokens together; cursor badges follow the Handle size
  preference instead (`canvas.md`).

### States

| State | Pictogram | Background (owned by the control's file) |
|---|---|---|
| Normal | base strokes `text`, accent `accent` | none or the control's face |
| Hover | unchanged | `hover` |
| Pressed | unchanged | `pressed` |
| Active tool, checked toggle | unchanged (never recolored) | `accent-soft`, plus an `accent` outline for the active tool (`toolbars.md`) |
| Disabled | every stroke, the accent and any state mark in `text-disabled`; shape unchanged | none |
| On a `danger` fill (Stop's big and compact buttons) | strokes and fill in `text-on-strong` (`visual-language.md`, `machine-panels.md`) | `danger` |
| On an `accent` fill (primary dialog button, if it ever carries an icon) | strokes in `text-on-strong` | `accent` |
| Quiet chrome (title strip buttons, More at rest, list row marks) | `text-muted`; `text` on hover | as the owning file says |
| Keyboard focus | unchanged | focus ring in `accent` around the control |
| Dark theme | the same tokens, resolved to their dark values; nothing is redrawn | as above |

- No icon animates. The only motion near icons is egui's normal hover
  feedback; Reduce motion (`naming.md`) therefore needs nothing here.

### Icons beside labels

- **Order**: icon first, then the label, on one line (menus, flyouts, the
  More popup, ordinary and big buttons, Preferences tabs, camera header).
  Never a label above an icon, except in the result and option buttons
  that `dialogs.md` and `later-windows.md` lay out as pictogram above a
  caption.
- **Spacing**: `gap-tight` between icon and label.
- **Vertical position**: the icon's optical centre lines up with the
  middle of the label's capital letters, not with its baseline.
- **Big buttons** (`row-big`): an `icon-big` pictogram left of the label,
  the pair centred together in the button (`machine-panels.md`).
- **Menus**: the icon sits in the gutter that every menu reserves
  (`menus.md`); a row without an icon leaves the gutter empty; a check item
  shows the tick there instead.
- **Icon-only controls** always have a tooltip whose first line names the
  action (`toolbars.md`, `naming.md`); accessibility tools read the
  button's label from the toolbar data.

### How we choose a metaphor

1. Use the metaphor a laser or drawing-software user would guess first,
   if it is a universal one (scissors for cut, magnifier for zoom).
2. Otherwise build it from LaserCraft's own motifs, which recur across the
   set so related icons look related:
   - **sheet**: a page with one folded corner (files);
   - **tray**: an open tray (saving into storage);
   - **bed**: a slightly rounded rectangle with a small dot in one corner
     (the work area and its origin corner);
   - **head**: a small ring with four short ticks (the laser head; never
     red);
   - **selection box**: a dashed rectangle;
   - **gantry**: a rectangle frame with one cross rail carrying the head
     (a machine, a device profile);
   - **picture**: a rectangle holding a mountain line and a small circle
     (images);
   - **flag**: a small flag on a short pole (the user origin).
3. If the reference's concept for the same action is known (checklist,
   "Metaphors we avoid") and it is not universal, pick another.
4. Keep it to two or three forms. If an idea needs four, it belongs in the
   tooltip.

Tier: **Own** (all of the above).

## Icon table

All concepts are own design; every row follows the style rules above.
"Universal" in Notes marks a metaphor kept because no one owns it.

### Files and projects

| Icon name (id) | Used by | Concept (generic metaphor, own design) | Accent detail | Notes |
|---|---|---|---|---|
| `new-project` | Standard: New; File → New; Materials panel Library ▸ New library | sheet with a plus at its lower right | the plus | Now. Universal (page with plus). |
| `open-project` | Standard: Open…; File → Open…; Edit → Preferences backup ▸ Open preferences folder; Library ▸ Open library…; Add camera "folder" option card | folder with its front flap tipped open | none | Now. Universal. |
| `save-project` | Standard: Save; File → Save; Library ▸ Save library; Image adjustments and Speed and power test preset rows Save preset… | tray with an arrow coming down into it | the arrow | Now. Deliberately not a disk. |
| `save-as` | File → Save as…; Library ▸ Save library as… | the `save-project` tray and arrow with a small pencil at the upper right | the pencil | Now. |
| `import-art` | Standard: Import…; File → Import…; Camera menu Import camera…; preset rows Import presets… | sheet with a straight arrow entering through its left edge | the arrow | Now. |
| `export-art` | File → Export vector art…; Camera menu Export…; preset rows Export presets… | sheet with a straight arrow leaving through its right edge | the arrow | Later (Alt+X, keymap.md); presets M7; camera M9. |
| `save-gcode` | File → Save G-code…; Machine panel Save G-code… | sheet carrying a short zigzag toolpath | the zigzag | Now (menu); M6 (button). |
| `run-gcode` | Machine → Run G-code file…; Machine panel Run G-code file… | the `save-gcode` sheet with a small filled start triangle at its lower right | none | Later. The triangle is the `start` form, small, in `text`. |
| `save-image` | Job preview Save image…; File → Export image as engraved… | picture with an arrow coming down into a small tray under it | the arrow | Later / M7. |
| `project-notes` | File → Project notes… | notepad: a sheet with a bound top edge and three short text lines | none | Now. |
| `new-window` | File → New window | window frame (rectangle with a title-bar line) with a plus | the plus | Later, optional. |
| `quit` | File → Quit | door standing ajar with an arrow leaving it | the arrow | Now. |
| `merge-library` | Library ▸ Merge library… | two arrows joining into one | the joined arrow | M8. |
| `linked-entry` | Materials panel entries that a layer follows | two chain links | none | M8. `icon-small`, `text-muted`. |

### History, selection and clipboard

| Icon name (id) | Used by | Concept (generic metaphor, own design) | Accent detail | Notes |
|---|---|---|---|---|
| `undo` | Standard: Undo; Edit → Undo ‹action›; History rows of done actions | an open arc ending in an arrowhead that points back to the left | none | Now. An outline arc on an open ground; no disc. |
| `redo` | Standard: Redo; Edit → Redo; History rows of undone actions (`text-muted`) | the `undo` arc mirrored | none | Now. |
| `history` | Edit → History… | clock face with an arc arrow running back along part of its rim | the arc arrow | Later. |
| `select-all` | Edit → Select all; canvas menu Select all; Layer menu Select shapes | selection box (dashed) with small square handles at its four corners | none | Now. |
| `invert-selection` | Edit → Invert selection | the `select-all` box split by a diagonal, one half hatched | the hatching | Now. |
| `cut` | Standard: Cut; Edit → Cut; canvas menu Cut | open scissors | none | Now. Universal. |
| `copy` | Standard: Copy; Edit → Copy; canvas menu Copy; Console and Serial port access "Copy" buttons | two overlapping sheets, the front one offset down and right | none | Now. Universal. |
| `duplicate` | Edit → Duplicate; canvas menu Duplicate | two overlapping small shapes (square in front of circle) with a plus | the plus | Now. Differs from `copy` (shapes, not sheets). |
| `paste` | Standard: Paste; Edit → Paste; canvas menu Paste | clipboard board with its clip and a sheet on it | none | Now. Universal. |
| `paste-in-place` | Edit → Paste in place; canvas menu Paste in place | the `paste` clipboard with a small head-style ring on the sheet | the ring | Now. |
| `delete` | Standard: Delete; Edit → Delete; canvas menu Delete; Layers panel Delete shapes; Saved artwork menu Delete; Saved head positions Delete; preset rows Delete preset | waste bin with a lid and two upright slats | none | Now. Universal. Undoable deletes only. |
| `delete-stored` | Camera menu Remove; Device profiles Remove; Materials Delete; Edit macros Delete; Delete preset confirmation | the `delete` waste bin drawn entirely in `danger` | n/a (`danger`) | M8/M9/later. The only action icon in `danger` besides `stop` (the error state marks are the other `danger` uses); for deletes Undo cannot bring back. |

### View and preview

| Icon name (id) | Used by | Concept (generic metaphor, own design) | Accent detail | Notes |
|---|---|---|---|---|
| `pan` | Standard: Pan | open hand, palm toward the viewer, fingers slightly apart | none | Later. Matches the panning cursor. |
| `fit-bed` | Standard: Fit bed; View → Fit bed | the bed motif with a small inner grid cross, between two inward-pointing arrowheads at its left and right | the arrowheads | Now. No magnifier, no corner brackets. |
| `zoom-in` | Standard: Zoom in; View → Zoom in | magnifier with a plus in its lens | the plus | Now. Universal. |
| `zoom-out` | Standard: Zoom out; View → Zoom out | magnifier with a minus in its lens | the minus | Now. Universal. |
| `zoom-selection` | Standard: Zoom to selection; View → Zoom to selection | magnifier whose lens holds a small dashed selection box | the dashed box | Later. |
| `job-preview` | Standard: Job preview…; View → Job preview…; canvas menu Job preview… | an eye above a path that is dashed for its first half and solid for its second | the solid half of the path | Now. Not a screen or monitor. |
| `play-preview` | Job preview Play | small filled triangle inside a circle outline | none | Later. Deliberately unlike `start`, so no preview control looks like the laser's Start. |
| `fill-canvas` | Job preview title strip button fill the canvas region | four short arrows pointing outward from the centre toward the corners | none | Later. `icon-small`, `text-muted` like other title strip buttons. |
| `visible` | Camera header Use in overlay (on); Text data Show values | open eye | none | M9 / later. |
| `hidden` | Camera header Use in overlay (off) | the `visible` eye crossed by one diagonal stroke | none | M9. Drawn in `text-muted` (`later-windows.md`). |

### Settings, devices and help

| Icon name (id) | Used by | Concept (generic metaphor, own design) | Accent detail | Notes |
|---|---|---|---|---|
| `preferences` | Standard: Preferences…; Edit → Preferences… | two level slider tracks with their knobs at different positions | the upper knob | Later. Deliberately not gears. |
| `keyboard-shortcuts` | Edit → Keyboard shortcuts… | two key caps joined by a plus | the plus | Later. |
| `device-profiles` | Machine → Device profiles… | gantry motif: rectangle frame, one cross rail, the head ring on the rail | none | M8. Deliberately no tools (wrench, screwdriver) and no gear. |
| `device-profile` | Standard: Edit device profile…; Machine → Edit device profile… | the gantry motif with a small pencil at the lower right | the pencil | M8. |
| `grbl-settings` | Machine → GRBL settings… | chip outline with short pins on two sides and three short rows inside | none | Later. |
| `prefs-general` | Preferences tab General | one pill switch drawn as an outline, knob at the right | the knob | Later. |
| `prefs-grid` | Preferences tab Grid and snapping | a grid of four cells with a dot on the centre crossing | the dot | Later. |
| `prefs-display` | Preferences tab Display | window frame whose inside is split by a diagonal, one half hatched (light and dark) | none | Later. |
| `prefs-import-export` | Preferences tab Import and export | sheet with an arrow entering on the left and one leaving on the right | none | Later. |
| `camera` | Machine → Camera list…; Preferences tab Camera; Camera panel | camera body with a round lens and a small viewfinder bump | none | M9. Universal. |
| `about` | Help → About LaserCraft… | circle holding the information mark (dot over a short stem) | none | Later. |
| `license` | Help → License… | sheet with a round seal at its lower right | the seal | Later. |
| `report-problem` | Help → Report a problem | speech bubble holding an exclamation mark | none | Later. |
| `quick-reference` | Help → Quick reference… | keyboard: a wide rounded rectangle with two rows of small key squares and a space bar | none | M10. |
| `user-guide` | Help → User guide | open book seen from above | none | M10. |
| `copy-system-info` | Help → Copy system info | the `paste` clipboard with the information mark on the sheet | the mark | Later. |
| `edit` | Camera menu Edit…; Materials Edit settings… and Edit description…; Edit macros Edit | a pencil, slanted | none | M8/M9. Generic edit, not a wrench. |
| `add` | Saved artwork menu Add selection; Saved head positions Add; Edit macros Add | a plus | none | Later. |

### Arrange

| Icon name (id) | Used by | Concept (generic metaphor, own design) | Accent detail | Notes |
|---|---|---|---|---|
| `group` | Arrange: Group; Arrange → Group; canvas menu Group | a small square and a small circle held inside one solid rounded box | the box | Now. Solid, because dashed means selection. |
| `ungroup` | Arrange: Ungroup; Arrange → Ungroup; canvas menu Ungroup | the same box broken open at two opposite sides, the two shapes nudged apart | none | Now. |
| `mirror-horizontal` | Arrange: Mirror horizontally; Arrange → Mirror horizontally | an asymmetric L-shaped outline (solid) and its mirror image (dotted) on either side of an upright axis | the axis | Now. No triangles or pennants. |
| `mirror-vertical` | Arrange: Mirror vertically; Arrange → Mirror vertically | the same pair across a level axis | the axis | Now. |
| `mirror-line` | Arrange: Mirror across a line; Arrange → Mirror across a line | the same pair across a slanted axis, with a small dot at each end of the axis (the two points) | the axis | Later. |
| `rotate-cw` | Arrange → Rotate 90° clockwise | a square with a quarter arc hugging one corner, ending in a clockwise arrowhead | none | Now. |
| `rotate-ccw` | Arrange → Rotate 90° counter-clockwise | the `rotate-cw` drawing mirrored | none | Now. |
| `rotate-two-points` | Arrange → Rotate and scale by two points | two dots joined by a straight line, an arc arrow swinging around one dot | the pivot dot | Later. |
| `align-left` | Arrange: flyout Left edges; Align and distribute Align left edges; Arrange → Align ▸ Left edges | two level bars of different lengths, flush against an upright line on their left | the line | Now. |
| `align-center-x` | flyout Vertical centers; Align vertical centers; Align ▸ Vertical centers | two level bars of different lengths centred on an upright line | the line | Now. |
| `align-right` | flyout Right edges; Align right edges; Align ▸ Right edges | two level bars flush against an upright line on their right | the line | Now. |
| `align-top` | flyout Top edges; Align top edges; Align ▸ Top edges | two upright bars of different heights hanging from a level line | the line | Now. |
| `align-center-y` | flyout Horizontal centers; Align horizontal centers; Align ▸ Horizontal centers | two upright bars centred on a level line | the line | Now. |
| `align-bottom` | flyout Bottom edges; Align bottom edges; Align ▸ Bottom edges | two upright bars standing on a level line | the line | Now. |
| `align-centers` | Arrange: Align centers; Align and distribute Align centers; Align ▸ Centers | two boxes of different sizes sharing one centre cross | the cross | Now. No target ring. |
| `distribute-h-spaced` | flyout Horizontal spacing; Distribute horizontal spacing; Distribute ▸ Horizontal spacing | three upright bars of different widths side by side, two equal gap brackets between them | the gap brackets | Now. |
| `distribute-h-centered` | flyout Horizontal centers; Distribute horizontal centers; Distribute ▸ Horizontal centers | three upright bars of different widths, a short tick through each centre, equal spacing marks between the ticks | the ticks | Now. |
| `distribute-v-spaced` | flyout Vertical spacing; Distribute vertical spacing; Distribute ▸ Vertical spacing | the `distribute-h-spaced` drawing turned a quarter (stacked level bars) | the gap brackets | Now. |
| `distribute-v-centered` | flyout Vertical centers; Distribute vertical centers; Distribute ▸ Vertical centers | the `distribute-h-centered` drawing turned a quarter | the ticks | Now. |
| `pack-horizontal` | Arrange → Distribute ▸ Pack horizontally | two upright bars touching side by side, a short arrow pressing toward the touching edge from each side | the touching edge | Now. |
| `pack-vertical` | Arrange → Distribute ▸ Pack vertically | the same stacked | the touching edge | Now. |
| `match-width` | Arrange: Match width; Align and distribute; Arrange → Match width | two boxes of different heights and equal widths, one double-headed level arrow spanning their width above | the arrow | Later. |
| `match-height` | Arrange: Match height; Align and distribute; Arrange → Match height | two boxes of equal heights, one double-headed upright arrow beside them | the arrow | Later. |
| `nest` | Slide and nest Nest; Arrange → Nest | several small irregular parts packed into one corner of a sheet outline | none | Later. |
| `slide-left` | Slide and nest Slide left; Arrange → Slide ▸ Left | a box, a short motion arrow behind it, stopping at an upright wall bar on its left | the wall bar | Later. |
| `slide-right` | Slide right; Slide ▸ Right | the same, wall on the right | the wall bar | Later. |
| `slide-up` | Slide up; Slide ▸ Up | the same, wall above | the wall bar | Later. |
| `slide-down` | Slide down; Slide ▸ Down | the same, wall below | the wall bar | Later. |
| `bed-center` | Move on bed flyout Center; Align and distribute Move to bed center; Arrange → Move on bed ▸ Center | the bed motif (solid) with a small square (the selection) in its middle | the small square | Now. Default face of the Move on bed flyout. |
| `bed-top-left`, `bed-top-right`, `bed-bottom-left`, `bed-bottom-right` | the matching Move on bed rows and Align and distribute buttons | the bed motif with the small square in that corner | the small square | Now. |
| `bed-left`, `bed-right`, `bed-top`, `bed-bottom` | the matching Move on bed rows | the bed motif with the small square against the middle of that edge | the small square | Later (edges). |
| `bed-head` | Move on bed To head position; Align and distribute Move to head position | the bed motif with the head ring and the small square centred on it | the small square | M6. The head stays neutral; never red. |
| `goto-center` | Move on bed flyout Go to selection block Center; Machine → Go to selection ▸ Center | a dashed selection box with the head ring at its centre | the head ring | M6. |
| `goto-top-left`, `goto-top-right`, `goto-bottom-left`, `goto-bottom-right`, `goto-top`, `goto-bottom`, `goto-left`, `goto-right` | the matching Go to selection rows | the dashed selection box with the head ring on that corner or edge middle | the head ring | M6. One shared drawing with the ring moved. |
| `grid-array` | Shape operations Grid array…; Arrange → Grid array… | one solid small square at the upper left followed by five dotted copies, two rows of three | the solid original | Now. Not a full grid of identical squares. |
| `circular-array` | Shape operations Circular array…; Arrange → Circular array… | a centre dot, a dashed circle through four small diamonds, one diamond solid and the rest dotted | the solid diamond | Now. |
| `path-array` | Arrange → Path array… | a curved solid path with small squares spaced along it | none | Later. |
| `outline-selection` | Arrange → Outline around selection | two small shapes wrapped by one dashed convex hull | the hull | Later. |
| `break-apart` | Arrange → Break apart | a bent path drawn as three pieces with small gaps, the piece ends nudged apart | none | Now. |
| `bring-forward` | Arrange → Bring forward | two overlapping squares, the back one dotted, a short arrow pointing up by one step | the arrow | Now. |
| `send-backward` | Arrange → Send backward | the same, arrow pointing down by one step | the arrow | Now. |
| `bring-to-front` | Arrange → Bring to front | the same, arrow pointing up to a short bar | the arrow | Now. |
| `send-to-back` | Arrange → Send to back | the same, arrow pointing down to a short bar | the arrow | Now. |
| `lock` | Arrange → Lock; canvas menu Lock; Position and size Keep proportions (on) | closed padlock | none | Later. Universal. |
| `unlock` | Arrange → Unlock; canvas menu Unlock; Keep proportions (off) | open padlock, shackle lifted to one side | none | Later. Universal. |

### Drawing tools, shape tools and tools

| Icon name (id) | Used by | Concept (generic metaphor, own design) | Accent detail | Notes |
|---|---|---|---|---|
| `select` | Drawing tools Select; Tools → Select | arrow pointer, tip at the upper left, drawn as an outline | none | Now. Universal. |
| `line` | Drawing tools Line; Tools → Line | slanted pen nib drawing a short stroke from its tip | the stroke | Now. Not a pencil. |
| `shape-tools` | Tools → Shape tools ▸ (row icon) | a square overlapping a circle | none | Now. The toolbar flyout shows its member instead. |
| `rect` | Shape tools flyout Rectangle; Tools → Shape tools ▸ Rectangle | rectangle outline, wider than tall | none | Now. Default face of Shape tools. |
| `ellipse` | Shape tools Ellipse; Shape tools ▸ Ellipse | ellipse outline, wider than tall | none | Now. |
| `triangle` | Shape tools Triangle | triangle outline, point up | none | Later. |
| `pentagon` | Shape tools Pentagon | regular five-sided outline | none | Later. |
| `polygon` | Shape tools Polygon | regular six-sided outline | none | Later. |
| `octagon` | Shape tools Octagon | regular eight-sided outline | none | Later. |
| `star` | Shape tools Star | five-pointed star outline | none | Later. |
| `alternating-star` | Shape tools Alternating star | star outline whose points alternate long and short | none | Later. |
| `edit-nodes` | Drawing tools Edit nodes; Tools → Edit nodes | a smooth S-curve with three small square nodes and one handle line | the middle node (filled) | Now. A curve, not a polygon with dots. |
| `trim` | Drawing tools Trim; Tools → Trim | two crossing lines; the end of one beyond the crossing is dotted (removed) | a short tick at the crossing | Later (Ctrl+K once bound). Not scissors: scissors are Cut. |
| `place-tabs` | Drawing tools Place tabs; Tools → Place tabs | a closed rounded outline with two short gaps, each spanned by a tiny bridge | one bridge | Later (Ctrl+Tab once bound). |
| `text-tool` | Drawing tools Text; Tools → Text | a text cursor (I-beam) beside three level lines of different lengths | the cursor | Now. No letterform (no text inside icons). |
| `move-head` | Drawing tools Move head; Tools → Move head | the head ring with a short arrow pointing at it from the lower left | the arrow | M6 (Alt+L once bound). Not a map pin. |
| `measure` | Drawing tools Measure; Tools → Measure | caliper: a bar with two jaws | none | Later (Alt+M once bound). Not a ruler. |
| `barcode` | Tools → Barcode…; canvas menu Edit barcode… | upright stripes of mixed widths | none | Later. |
| `offset` | Shape operations Offset…; Tools → Offset… | a rounded square with a parallel dashed outline around it | the outer outline | Now. |
| `weld` | Shape operations Weld; Tools → Weld | three small overlapping shapes drawn as one merged outline, their inner crossings dotted | none | Now. Three shapes: weld takes many. |
| `union` | Booleans flyout Union; Tools → Union; Combine shapes Union | a square and a circle overlapping; their merged outer contour solid, the inner arcs dotted | the merged contour | Now. Default face of Booleans. |
| `subtract` | Booleans Subtract; Tools → Subtract; Combine shapes First minus second | the same pair; the square's remaining outline solid, the bitten-out part of the circle dotted | the remaining outline | Now. |
| `subtract-reverse` | Combine shapes Second minus first | the `subtract` drawing with the roles swapped (circle remains) | the remaining outline | Later. |
| `intersect` | Booleans Intersect; Tools → Intersect; Combine shapes Intersect | the same pair; only the lens-shaped overlap solid, the rest dotted | the overlap | Now. |
| `combine-shapes` | Booleans Combine shapes…; Tools → Combine shapes… | the `union` pair with three short list lines beside it | none | Later (Ctrl+B once bound). |
| `reset` | Combine shapes Reset; Speed override and Power override "100%" | one arc turning back to a small dot (return to start) | none | Later. |
| `start-point` | Shape operations Start point | a circle outline with a filled node dot and an arrowhead on the path just after it | the arrowhead | Later. |
| `round-corners` | Shape operations Round corners | an L-shaped corner whose bend is a round arc, the old sharp corner shown dotted | the arc | Later. |
| `image-adjust` | Tools → Image adjustments…; Image menu Image adjustments… | the picture motif with a half-hatched circle (contrast) at its lower right | the circle | M7. Not sliders: sliders are Preferences. |
| `trace-image` | Tools → Trace to vectors…; Image menu Trace to vectors… | the picture motif with a pen nib tracing its mountain line | the traced line | M7. |
| `trace-camera` | Camera panel Trace camera image… | the `camera` with a pen nib at its lower right | the pen nib | M9. |

### Node tools

| Icon name (id) | Used by | Concept (generic metaphor, own design) | Accent detail | Notes |
|---|---|---|---|---|
| `node-pointer` | Node tools Node pointer (button 1; drags nodes and handles, nothing armed) | an arrow pointer with a small square at its tip | none | Later. Shows the active-tool look while no node action is armed (`toolbars.md`). |
| `node-add` | Node tools Add node here; armed badge | a short curve with a square node and a plus | the plus | Later (key I now). |
| `node-add-middle` | Node tools Add node at middle; armed badge | a short curve with end ticks and a node at its middle, plus a plus | the plus | Later (key M now). |
| `node-delete` | Node tools Delete node; armed badge | a short curve with a square node and a minus | the minus | Later (key D now). |
| `node-straight` | Node tools Make straight; armed badge | a dotted curve becoming a straight segment | the straight segment | Later (key L now). |
| `node-curve` | Node tools Make curve; armed badge | a dotted straight segment becoming a curve | the curve | Later (key C now). |
| `node-smooth` | Node tools Make node smooth; armed badge | a node with two handles in one straight line | the handles | Later. |
| `node-sharp` | Node tools Make node sharp; armed badge | a node with two handles at an angle | the handles | Later. |
| `node-break` | Node tools Break at node; armed badge | a path with a gap at one node, two end squares facing each other | none | Later (key B now). |
| `segment-trim` | Node tools Trim segment; armed badge | the `trim` drawing at node-tool scale | the tick | Later. |
| `segment-extend` | Node tools Extend to crossing; armed badge | an open end with a dashed extension reaching a crossing line | the extension | Later. |
| `segment-level` | Node tools Level by segment; armed badge | a slanted segment, an arc arrow turning it, a level line under it | the level line | Later. |

- No green, red or cyan marks on node icons (decision 6; `toolbars.md`).

### Machine, job and jog

| Icon name (id) | Used by | Concept (generic metaphor, own design) | Accent detail | Notes |
|---|---|---|---|---|
| `connect` | Machine panel Connect; Machine → Connect | a plug pushed into a socket | none | M6. |
| `disconnect` | Machine panel Disconnect; Machine → Disconnect | the plug pulled back with a gap before the socket | none | M6. |
| `refresh-ports` | Machine panel Refresh ports | two arcs with arrowheads chasing each other round a circle | none | M6. Differs from `reset` (one arc). |
| `start` | Machine panel Start; Machine → Start | filled right-pointing triangle, rounded corners | none | M6. Media symbol, universal, filled (allowed fill). |
| `start-rotary` | Machine panel Start while Use rotary is on | the `start` triangle with a small `rotary` mark at its lower right | none | M9. |
| `resume` | Pause / Resume while held; job bar; Machine → Resume | the `start` drawing | none | M6. Alias of `start`; the label says "Resume". |
| `pause` | Machine panel Pause; job bar Pause; Machine → Pause | two filled upright bars, rounded ends | none | M6. Universal. |
| `pause-motion-only` | Pause (motion only) during a constant-power job, panel, job bar and menu | the `pause` bars with a small warning triangle at the lower right | the warning mark in `warning` (state mark) | M6, safety rule 9. |
| `stop` | Machine panel Stop; job bar Stop; Machine → Stop | filled square, rounded corners | n/a: `danger` in menus; `text-on-strong` on the `danger` button fill | M6. Universal. The strongest icon in the set. |
| `frame` | Machine panel Frame; Machine → Frame | a dashed rectangle with an arrowhead running along its top edge | the arrowhead | M6. Laser-off; nothing about it suggests a beam. |
| `frame-outline` | Machine panel Frame outline; Machine → Frame outline | a dashed convex hull (irregular five-sided outline) with the same arrowhead | the arrowhead | M6. |
| `frame-low-power` | Machine → Frame at low power… | the `frame` rectangle with a short downward beam line ending in a dot | the beam mark in `warning` (state mark: the laser is on) | Later, optional, owner decision (safety rule 2). |
| `home` | Machine panel Home; jog pad centre; Machine → Home | house outline | none | M6. Universal. |
| `go-user-origin` | Machine panel Go to user origin; Machine → Go to user origin | the flag motif planted on a small cross | the flag | M6. Not a pin. |
| `set-user-origin` | Machine → Set user origin | the flag motif with a plus | the plus | M6. |
| `clear-user-origin` | Machine → Clear user origin | the flag motif with a minus | none | M6. Not destructive red: it can be set again. |
| `set-end-position` | (reserved; no row uses an icon yet) | a dashed path ending at a small square stop mark | none | Later. |
| `jog-x-minus`, `jog-x-plus`, `jog-y-plus`, `jog-y-minus` | Jog pad X−, X+, Y+, Y− | one chevron pointing left, right, up, down | none | M6. |
| `jog-z-down` | Jog pad Z down | an arrow pointing down at a short level bar | none | M6, motorized Z only. |
| `jog-z-up` | Jog pad Z up | an arrow pointing up at a short level bar above it | none | M6, motorized Z only. |
| `jog-rotary-ccw`, `jog-rotary-cw` | Jog pad rotary turn counter-clockwise / clockwise | the two small rollers of `rotary` seen end-on, each with a short straight motion stroke on its upper edge pointing that way | none | M9. No curved arrow around a dot. |
| `jog` | Print registration wizard "Go to expected target 2" pictogram | the head ring with four short arrows pointing outward | none | M9. |
| `override-up`, `override-down` | Speed override and Power override +1%, −1% | one chevron up, one chevron down | none | Later. |
| `override-up-big`, `override-down-big` | Speed override and Power override +10%, −10% | two stacked chevrons up, two down | none | Later. Own refinement (proposed change to `machine-panels.md`). |
| `rotary` | Job setups Rotary attachment…; Machine → Rotary attachment… | a cylinder lying on two small rollers | none | M9. Not looping arrows. |
| `registration` | Job setups Print registration…; Machine → Print registration ▸ | a sheet with two corner-tick registration marks (a short L of two strokes) at opposite corners | none | M9. Not a printer; no ring-and-cross targets. |
| `set-target-1` | Print registration ▸ Set target 1; wizard step 1 pictogram | the `registration` sheet with the upper-left corner mark in accent | that mark | M9. No digit. |
| `set-target-2` | Print registration ▸ Set target 2; wizard step 2 pictogram | the same with the lower-right corner mark in accent | that mark | M9. No digit. |
| `fit-to-targets` | Print registration wizard Fit job to targets | a frame with two diagonal scale arrows and a corner-tick mark at each end of the diagonal | none | M9. |
| `place-on-targets` | Print registration wizard Place job on targets (no scaling) | a corner-tick mark with a short straight arrow sliding it into place | none | M9. |
| `camera-capture` | Standard: Capture camera image; Machine → Capture camera image; Camera panel and header Capture buttons | the `camera` with a small circular arrow at its lower right | the arrow | M9 (Alt+C once bound). |
| `camera-add` | Camera menu Add camera…; Camera list add | the `camera` with a plus | the plus | M9. |
| `source-usb` | Camera header Source badge; Camera list row; Add camera option card | a cable plug end: a rounded rectangle with two small squares in it, a short cable behind | none | M9. |
| `source-network` | Camera list row; Add camera option card | a network socket: a square with a stepped notch | none | M9. |
| `lens` | Camera header Lens badge; Add camera option card | a lens: two arcs facing outward joined at their ends | none | M9. |
| `alignment-bed` | Camera header Alignment badge | the bed motif inside four corner ticks of a camera view | none | M9. |
| `mount-above-bed` | Add camera option card | the `camera` above the bed motif | none | M9. |
| `mount-on-head` | Add camera option card | the `camera` fixed to the head ring | none | M9. |
| `printed-pattern`, `capture-frames`, `lens-correct` | Camera lens calibration step cards | a sheet with a dot grid; the `camera` with two stacked frames behind it; a straight frame beside a bowed one | none | M9. |
| `taper-correction` | Machine → Taper correction… | a trapezoid (a cup seen from the side) with a dashed rectangle laid over it | the dashed rectangle | M9. |
| `focus-test` | Machine → Focus height test… | a staircase of short level lines stepping down | none | M8, needs a motorized Z. |
| `interval-test` | Machine → Line interval test… | three small squares hatched at three different line spacings | none | M8. |
| `power-test` | Machine → Speed and power test… | a grid outline four cells wide and two cells high, with an arrow along its top edge and one down its left edge | none | M8. Not filled squares and not a three-by-three block. |
| `find-center` | Machine → Find center…; Find center diagrams | a circle with three short ticks on its edge and two chords from those ticks meeting at its centre | the meeting point of the chords | Later. No crosshair inside a circle. |

### Layers panel

| Icon name (id) | Used by | Concept (generic metaphor, own design) | Accent detail | Notes |
|---|---|---|---|---|
| `layer-up` | Layers panel Move up | one chevron pointing up | none | Now (today a text button). Same drawing as `jog-y-plus`. |
| `layer-down` | Layers panel Move down | one chevron pointing down | none | Now. Same drawing as `jog-y-minus`. |
| `settings-copy` | Layers panel Copy settings | the `copy` sheets with a short slider line on the front sheet | the slider knob | Later. |
| `settings-paste` | Layers panel Paste settings | the `paste` clipboard with a short slider line on the sheet | the slider knob | Later. |
| `mode-line` | Operations tab marks; Materials entry marks | one open stroke path | none | Later / M8. `icon-small`, `text-muted`. |
| `mode-fill` | Operations tab marks; Materials entry marks | a rounded square filled with level hatching | none | Later / M8. |
| `mode-image` | Operations tab marks; Materials entry marks | the picture motif | none | M7 / M8. |
| `image-row` | Layers panel image rows, after the name | the `mode-image` drawing | none | M7. Alias of `mode-image`. |
| `operation-add` | Operations Add operation | a small tab shape with a plus | the plus | Later, owner decision. |
| `operation-duplicate` | Operations Duplicate operation | two small tab shapes overlapping | none | Later, owner decision. |
| `operation-delete` | Operations Delete operation | a small tab shape with a minus | none | Later, owner decision. Undoable. |
| `guide-layer` | Guide swatches 1 and 2 (the digit is drawn beside it, not inside) | a dashed line meeting a set-square corner | none | Later, owner decision. |

### Chrome glyphs

| Icon name (id) | Used by | Concept (generic metaphor, own design) | Accent detail | Notes |
|---|---|---|---|---|
| `flyout-marker` | every flyout button | small filled right-angled triangle filling the lower-right corner of the button | none | `text-muted`; disabled `text-disabled` (`toolbars.md`). Never teal, never accent. |
| `more` | the More button at the end of a folded toolbar | two chevrons pointing right, side by side | none | Drawn, not typed as a font character. |
| `menu-check` | check items in menus and flyouts | a tick | none | `text`. No box. |
| `menu-choice` | the active one of a set (current tool in Tools) | a small filled dot | none | `text`. |
| `submenu-arrow` | submenu rows | small filled triangle pointing right | none | `text-muted` (`menus.md`). |
| `panel-float` | title strip Float button | a small square lifting out of a larger frame, a short arrow up and right | none | Docking stage b only (`layout.md`). |
| `panel-close` | title strip Close; notes band and job refused band close | a cross of two short strokes | none | `text-muted`; `text` on hover. |
| `stepper-up`, `stepper-down` | numeric field steppers (`property-bar.md`) | small chevron up / down | none | `text-muted`. |
| `dropdown-chevron` | every dropdown | small chevron down | none | `text-muted`. |
| `chevron-left`, `chevron-right` | Text data Previous, Next | one chevron left / right | none | Later. |
| `to-first` | Text data Back to first | an arrow pointing left at an upright bar | none | Later. |
| `freeze` | Text data Freeze values | a six-spoked snowflake | none | Later. |
| `disclosure-closed`, `disclosure-open` | collapsible sections, camera header disclosure | small filled triangle pointing right / down | none | egui's collapsing header may draw its own; if kept, it follows these rules. |

### Status marks and message icons

| Icon name (id) | Used by | Concept (generic metaphor, own design) | Accent detail | Notes |
|---|---|---|---|---|
| `mark-error` | Alarm banner, Job refused band, bounds check refused, inline field errors (`dialogs.md` "error") | circle outline holding an exclamation mark | n/a: whole mark in `danger` | `icon-small`. |
| `mark-warning` | Constant-power note, warnings in bands and the preview | triangle outline holding an exclamation mark | n/a: `warning` | `icon-small`. Also the small mark on `pause-motion-only`. |
| `mark-success` | bounds check "Fits the work area.", success bands | a tick | n/a: `success` | `icon-small`. |
| `mark-info` | info messages in bands and the `message-info` box mark; never on hint bands, which carry no mark (`visual-language.md`) | circle outline holding the information mark | n/a: `text-muted` | `icon-small`. |
| `message-error` | message boxes of kind Error | `mark-error` at `icon-big` | n/a: `danger` | Outline circle, not a filled disc (proposed change to `dialogs.md`). |
| `message-warning` | message boxes of kind Warning | `mark-warning` at `icon-big` | n/a: `warning` | |
| `message-question` | message boxes of kind Question | circle outline holding a question mark | the whole mark in `accent` | `dialogs.md` gives Question `accent`. |
| `message-info` | message boxes of kind Info; About tab (if any pictogram) | `mark-info` at `icon-big` | the whole mark in `accent` | `dialogs.md` gives Info `accent`. |
| `badge-ok`, `badge-missing`, `badge-failed` | Camera header status badges (corner of `source-usb`, `lens`, `alignment-bed`) | a tick; a small exclamation triangle; a small cross | n/a: `success`, `warning`, `danger` | M9. State marks, drawn on a `field` disc so they read over the base icon. |

Tier of the whole table: **Own**.

## Cursors

### Reference

- Drawing, text and rounding use a thin crosshair with a hollow centre and
  a small badge at the lower right; the badge shows the next node kind for
  lines, and for rounding shows round, sharp, or a red no-room mark
  [R19][R20][R35].
- Trimming shows scissors; placing tabs shows a plus with a small tab glyph
  [R21][R22].
- Snapping replaces the arrow with a small glyph per snap kind (node,
  midpoint, intersection, line); over the centre of a selection the move
  arrows show [R23][R25]; while picking points for two-point rotate and
  scale the cursor over a node is a crosshair [R36].
- Handles show four-way, two-way and curved-arrow cursors [R25].
- Panning shows an open hand [R24]; hovering a ruler shows a line glyph
  before a guide is dragged out [R34]; a warp handle shows a crosshair
  [R37].
- A display setting scales cursors together with node marks [R29].
- The notes record no cursor for the head-positioning tool [R38].

### LaserCraft

Which cursor shows in which situation is `canvas.md`'s ("Cursors"); this
section fixes how each looks and where its hotspot is. Two kinds exist:

- **System cursors** come from egui's `CursorIcon` set and look like the
  operating system's cursors; their hotspot is the system's own (arrow tip,
  crosshair centre, centre of the hand, centre of the resize arrows). We do
  not draw them.
- **Painted parts** are drawn by LaserCraft on the canvas only: the rotate
  glyph (which replaces the system cursor) and the badges (which sit next
  to a system cursor). A badge never moves the hotspot: the click lands
  where the system cursor points.

| Situation | Cursor (concept) | Hotspot | Badge (id, concept) | Notes |
|---|---|---|---|---|
| Select over empty canvas | system arrow (`Default`) | arrow tip | none | Now. |
| Select over an unselected shape | system pointing hand (`PointingHand`) | fingertip | none | Later (`canvas.md`). |
| Select over a selected shape, the Move handle, or a node while dragging nodes | system four-way arrows (`Move`) | centre of the cross | none | Later. |
| Over a corner scale handle | system diagonal resize (`ResizeNwSe` / `ResizeNeSw`, matching the corner) | centre of the arrow | none | Later. |
| Over an edge scale handle | system horizontal or vertical resize | centre of the arrow | none | Later. |
| Over the rotate handle, and while rotating | painted `cursor-rotate`: a short arc with an arrowhead at each end, in `text` with a `handle-fill` halo | centre of the arc's chord, placed on the pointer | — | Later. The system cursor is hidden (`CursorIcon::None`). |
| Over a skew handle | system horizontal or vertical resize | centre of the arrow | none | Later. |
| Over a locked selection's handles | system not-allowed (`NotAllowed`) | centre | none | Later. |
| Line tool | system crosshair (`Crosshair`) | crosshair centre | `badge-line` (the `line` icon) plus the layer square | Now (cursor), later (badge). |
| Rectangle, Ellipse and every Shape tools member | system crosshair | crosshair centre | `badge-rect`, `badge-ellipse`, `badge-triangle`, `badge-pentagon`, `badge-polygon`, `badge-octagon`, `badge-star`, `badge-alternating-star` (the shape's icon) plus the layer square | Later (badges). |
| Text tool | system crosshair | crosshair centre | `badge-text` (the `text-tool` icon) plus the layer square | Later. While typing in the Add text window egui's text cursor applies there, not on the canvas. |
| Edit nodes, nothing armed | system arrow; four-way arrows over a node or handle | as the system cursor | none | Now (arrow), later (four-way). |
| Edit nodes, a node action armed | system crosshair | crosshair centre | the armed action's node icon (`badge-node-add`, `badge-node-add-middle`, `badge-node-delete`, `badge-node-straight`, `badge-node-curve`, `badge-node-smooth`, `badge-node-sharp`, `badge-node-break`, `badge-segment-trim`, `badge-segment-extend`, `badge-segment-level`) | Later. |
| Trim tool | system crosshair | crosshair centre | `badge-trim` (the `trim` icon) | Later. Not scissors. |
| Place tabs tool | system crosshair; not-allowed over shapes that cannot take tabs | crosshair centre | `badge-tab` (the `place-tabs` icon) | Later. |
| Start point tool | system crosshair over nodes; arrow elsewhere | as the system cursor | `badge-start-point` (the `start-point` icon) | Later. |
| Round corners tool | system crosshair | crosshair centre | `badge-round-corner` (the `round-corners` icon); `badge-sharp-corner` (an L corner with a sharp bend, its old arc dotted) over a rounded corner; `badge-no-room` (two short outward arrows with a small cross, in `warning`) when the radius does not fit | Later. `badge-no-room` uses `warning`, not `danger`: it reports that the action cannot happen here, not an error, and it keeps clear of the reference's red no-room mark. |
| Measure tool | system crosshair | crosshair centre | `badge-measure` (the `measure` caliper) | Later. |
| Move head tool | system crosshair; not-allowed outside the work area or when not connected | crosshair centre | `badge-move-head` (the head ring) | M6. Not a pin (proposed change to `canvas.md`). |
| Rotate and scale by two points | system crosshair | crosshair centre | `badge-pivot` (a dot with a short arc arrow) until the pivot is set, then none | Later. |
| Perspective warp, Mesh warp handles | system crosshair | crosshair centre | none | Later. |
| Arc handle of a text shape | system vertical resize | centre | none | Later (proposed change to text.md, owner decision). |
| Pan tool, Space held, or middle button | system open hand (`Grab`); closed hand (`Grabbing`) while dragging | centre of the hand | none | Now (gestures), later (cursor). |
| Over a ruler (once guide lines exist) | system vertical resize (top ruler) or horizontal resize (left ruler) | centre | none | Later. |
| Snapping (any tool) | unchanged | unchanged | the snap kind's marker from `canvas.md` added to the badge, in `guide`: `snap-grid` (small plus), `snap-corner` (small square outline), `snap-edge-middle` (small triangle outline), `snap-center` (circle with a dot), `snap-node` (small filled square), `snap-segment-middle` (triangle on a short line), `snap-intersection` (small X), `snap-on-path` (short bar across a line), `snap-guide` (short bar across a dashed line) | Later. `guide` (teal) is allowed here only, as `canvas.md` decides. |
| Busy (the app is planning a large job) | canvas: unchanged; inside the job preview's drawing area: system progress cursor (`Progress`) | system | none | Later (proposed change to `job-preview.md`). |
| Text fields anywhere | system text cursor (`Text`) | the I-beam | none | egui default. |
| Dock splitters and resizable panel edges | system column or row resize | centre | none | egui default. |

Badge look (from `canvas.md`, repeated for the artist): a small rounded
`field` chip with a *hairline* `border` outline at the pointer's lower right,
about `icon-small`, holding the badge glyph in `text` at a little less than
`icon-small`; the drawing tools add a small square of the current layer
color at the chip's right end, outlined in `border` so black and pale
layers both show. Badges are never drawn over the rulers. A badge reuses
its tool's icon drawing; there is no separate artwork to keep in step.

Tier: **Match** (crosshair plus a mode badge, resize and move arrows, open
hand), **Echo** (the snap kind shown at the pointer), **Own** (chip, layer
square, every painted glyph, the rotate glyph's halo).

## App icon (own design)

### Reference

- The reference's own logo appears in its window title bar, dialog title
  strips and some help rows [R12][R10]. It is a trademark and is not
  described or used here in any way.

### LaserCraft

Direction (the owner approves the final mark; Open questions):

- **Our own mark.** It must not resemble the reference's logo, any laser
  vendor's or controller vendor's logo, or any other product's mark. It
  uses no flame, no lightning bolt and no red-orange color family (the
  reference's chrome colors, decision 2), so it cannot be read as related.
- **Built from LaserCraft's motifs**: a light bed sheet (the bed motif),
  a path on it that is dashed for travel and solid where it burns, and the
  head ring where the solid part ends. One accent-blue element, the rest
  dark neutral on a light ground, matching decision 3. Flat, no gradient,
  no shadow, no perspective.
- **Shape**: a rounded square tile, so it sits well beside other apps on
  Linux desktops, in the Windows taskbar and in the macOS dock. The design
  must still read when the operating system draws it very small: at the
  smallest size only the tile, the bed edge and the head ring need to
  survive.
- **No text** in the mark. A wordmark ("LaserCraft" set in egui's bundled
  sans-serif) may appear next to it in the About tab and on release pages,
  never inside it.
- **Where it shows**: the operating system's window and taskbar icon (set
  through eframe's `ViewportBuilder::with_icon` with `IconData`; today no
  window icon is set, current-ui.md 1.1), the Flatpak, AppImage, .msi and
  .dmg packages (M10), and the About tab of Quick reference (`dialogs.md`
  says no logo until this file defines one). Never in dialog title strips
  and never in menus (`dialogs.md`).
- **Files**: one master vector drawing, from which M10's packaging step
  produces the raster sizes each package format asks for; the Linux
  packages install it under the app id `io.github.bhimio1.LaserCraft`, as
  the freedesktop icon theme rules expect.
- **Reduced variant**: a single-color version (dark strokes only) for places
  that tint icons, such as the macOS menu bar if ever used.

Tier: **Own**. Milestone: M10 (window icon may land earlier, with the
toolbar milestone).

## Implementation options

Today there are no icons at all (current-ui.md 3). The options, in the order
current-ui.md lists them:

| Option | How | New dependency | Strengths | Limits |
|---|---|---|---|---|
| 1. Painted egui shapes | an `icons` module in `lasercraft-ui` with one function per id that draws lines, circles, rounded rectangles, `PathShape` and `CubicBezierShape` with `Painter` | none | crisp at every scale and theme; strokes take token colors directly, so the accent and state marks need nothing special; unit-testable; no assets | every icon is hand-written code, which is slow to draw and hard for a non-programmer to review; epaint fills only convex polygons (our style needs only convex fills); epaint has no round line joins, so joins and caps need a small filled circle at each vertex and end |
| 2. Our own SVG files through our importer | one `<id>.svg` per icon, embedded with `include_bytes!`, parsed once with `lasercraft_import::svg::parse` into `lasercraft_geom::Path`, flattened and stroked with `Painter` in token colors | none (the UI already depends on `lasercraft-import`); ARCHITECTURE.md's crate-graph line, which says the UI uses import only for font families, must change | icons are drawn in any vector editor (Inkscape runs on the owner's Linux desktop) and reviewed as pictures; same crisp, tintable output as option 1; one source of truth per icon | the importer returns geometry, not styling: base and accent parts must be told apart (by element id if the importer passes ids through, or by a second file `<id>.accent.svg`); same convex-fill and round-join limits as option 1; only a small SVG subset is honoured (below) |
| 3. `egui_extras` with its `svg` feature | `install_image_loaders`, then `egui::Image::new(include_image!(…))` with `.tint(…)` | `egui_extras` and resvg / tiny-skia (usvg is already in the tree; versions must match); needs a line in ARCHITECTURE.md's Dependencies table | true SVG rendering with antialiasing, round joins and any fill | renders to textures, so every size and theme is a re-render; `tint` colors the whole image, so an accent detail needs a second image layered on top; a larger dependency tree |
| 4. Glyph font | an icon font registered with `ctx.set_fonts(FontDefinitions)` | none in code; a font asset | icons sit in text like characters | we would have to build our own font (a third-party icon set conflicts with "draw every icon ourselves" in clean-room.md); one color per glyph, so accents need overlays; egui's bundled symbol glyphs are uneven in coverage and style |

### Recommendation (a decision for the milestone that builds the toolbar)

Own design; the milestone that builds the toolbar band makes the final call
and records it in ARCHITECTURE.md:

- **Use option 2** for every icon in the table: SVG source files in a
  restricted subset, rendered as painted strokes in token colors. It keeps
  the drawings reviewable as pictures, adds no crate, and gives the same
  crisp, theme-following result as option 1.
- **Use option 1** for the chrome glyphs that are pure geometry and depend
  on state (`flyout-marker`, `menu-check`, `menu-choice`, `submenu-arrow`,
  steppers, `dropdown-chevron`, `more`, `panel-close`, `cursor-rotate`, the
  badge chip and layer square): they are a few lines of code each.
- **Not options 3 and 4** for now: option 3 adds a dependency and makes the
  accent rule awkward; option 4 needs a font we would have to make.
- The SVG subset (own design): a square `viewBox`; only `path`, `line`,
  `polyline`, `polygon`, `rect` (with `rx`), `circle` and `ellipse`; stroke
  only, except the allowed convex fills; no `text`, no `image`, no
  gradients, no filters, no masks, no transforms other than plain
  translate and scale; the accent part marked by the id `accent` (or the
  separate accent file, whichever the importer supports); `stroke-dasharray`
  for the dash language. A test rejects files outside the subset.
- Tests (CLAUDE.md: a test for every feature): every id used by toolbar and
  menu data has a source file and renders without error; no file uses an
  element outside the subset; no file has more than one accent part; only
  `stop`, `delete-stored`, `mark-error`, `message-error` and
  `badge-failed` may use `danger`; the
  rendered icon fits inside the safe margin; a golden test renders a few
  icons at `icon` size into a headless egui frame and compares the shapes
  emitted.

### egui notes

- The icons module exposes one call, "paint icon ‹id› in this rectangle
  with these colors", used by the toolbar button widget (`toolbars.md`),
  menu rows (`menus.md`), buttons and badges. Colors are passed in from
  `Visuals` and the tokens; the module never picks a color itself.
- Parsed icons are cached once per id at start; drawing is scaling and
  stroking only, cheap enough for every frame.
- Menu rows use egui's menu buttons with the icon painted into the gutter
  (`menus.md`); the reserved gutter means rows without an icon still line
  up.
- The window icon is generated from the master app icon at build time or
  decoded with the image decoder M7 brings; until then a small RGBA buffer
  drawn in code can serve (current-ui.md 3).

Tier: **Own**.

## Where icon source files live

Own design; the paths are created by the milestone that builds the toolbar
(and M10 for the app icon):

- `crates/lasercraft-ui/icons/<id>.svg`: one file per id in the table
  (and `<id>.accent.svg` if the separate-file accent route is chosen).
  Ids that are aliases (`resume`, `image-row`) have no file of their own;
  the alias is listed in the toolbar data.
- `crates/lasercraft-ui/icons/PROVENANCE.md`: one line per file: id, who
  drew it, date, and the statement "own design, drawn without reference
  artwork open". Reviewers check this list (checklist below).
- `crates/lasercraft-ui/src/icons.rs`: the icons module (embedding,
  caching, painting, the option 1 chrome glyphs).
- `crates/lasercraft-app/icons/lasercraft.svg`: the master app icon; M10's
  packaging files refer to it. Generated raster files are build outputs and
  are not committed by hand.
- Icon sources are part of LaserCraft and carry its license
  (GPL-3.0-or-later); no third-party icon set is vendored (clean-room.md).

## Reviewer checklist: reject icons that look like copies

A reviewer (human or agent) rejects an icon, or a batch of icons, if any
answer below is "no". Provenance comes first (clean-room.md, Process 4).

Provenance

1. Is the icon listed in `PROVENANCE.md` with an author, a date and the
   own-design statement?
2. Was it drawn from this file's concept text only, without a reference
   screenshot, icon file or video open while drawing (clean-room.md:
   never trace or copy artwork)?
3. Is the source file our own vector drawing (not an imported, traced or
   auto-vectorized image, not a third-party icon pack)?

Style (a copy usually fails these first)

4. Is it an outline drawing at the set's one medium stroke weight, with
   round joins and caps, and fills only where this file allows them?
5. Does it use at most one `accent` part, marking what the action creates
   or changes, and no other color except an allowed `danger` or state
   mark?
6. Is it free of letters and words (digits only on swatches and guide
   swatches)?
7. Does it sit on the shared grid inside the safe margin, at the same
   optical size and weight as its neighbours?
8. Does it follow the dash language (solid object, dashed selection or
   laser-off path, dotted removed or ghost)?

Differentiation

9. Laid side by side with the reference's icon for the same action (as
   described in words in the research notes, not as a picture), would a
   user say they are different drawings? A universal metaphor drawn in our
   style passes; the same composition, proportions and fill pattern as the
   reference fails.
10. Does it avoid every concept in "Metaphors we avoid" for its action?
11. Is the set as a whole free of the reference's signature traits: heavy
    filled silhouettes, a colored corner triangle on family buttons, round
    filled discs behind arrows, red head targets, brand marks?
12. For the app icon: does it avoid any resemblance to the reference's logo
    or any vendor's mark, any flame or bolt, and the red-orange family?

Fit

13. Does it read at `icon-small` in a menu and at `icon-big` on a button,
    in the light and the dark theme, and disabled?
14. Does every place listed under "Used by" use this same drawing?

### Metaphors we avoid (for the actions named)

These are the generic concepts the research notes record for the
reference's icons [R1][R2][R7][R8][R13][R14][R15][R26][R31][R39][R40]
[R42][R43][R44]. They are listed so reviewers can spot a copy; an
icon for the same action must use a different concept (ours is in the
table):

| Action | Concept to avoid |
|---|---|
| Save | a floppy disk |
| Undo, Redo, history rows | an arrow inside a solid dark disc |
| Group, Ungroup | person silhouettes |
| Mirror | two mirrored triangles across a dashed line, one outlined and one filled |
| Align centers | a target ring with a crosshair |
| Move on bed targets | dashed corner brackets or a dashed plus |
| To head position, any head target | a crosshair with a red centre |
| Preferences | gears |
| Edit device profile | crossed wrench and screwdriver |
| Job preview | a computer monitor |
| Fit bed | a magnifier with a page in its lens |
| Pan button | a four-headed arrow (the system Move cursor may still use one) |
| Text tool | a bold capital letter |
| Move head | a map pin |
| Measure | a ruler laid on a diagonal |
| Trim | scissors (ours are Cut) |
| Place tabs | an open bracket with a separate bar |
| Offset | a letter drawn with a double outline |
| Booleans | overlapping squares with dotted parts and a colored corner mark |
| Grid array, Speed and power test | a three-by-three block of filled squares |
| Circular array | a ring of small hollow squares with one filled |
| Start point | a polygon with a curved arrow at its top corner |
| Round corners | a corner with one solid curved side and dashed others |
| Frame, Frame outline | a square of corner brackets; a dashed circle |
| Rotary | two curved arrows forming a loop |
| Print registration | a printer with scissors |
| Line interval test | a plain stack of level lines |
| Focus height test | a dashed frame with a plus |
| Find center | a crosshair inside a circle |
| Quit | a dark filled square with a cross |
| Keyboard shortcuts | a single key cap |
| Experimental items | a laboratory flask |
| Any flyout button | a colored corner triangle |
| Help rows, title strips, device lists | the program's or a vendor's brand mark |

Tier: **Own**.

## Current state → target

From `current-ui.md` (commit 151eb26); paths relative to the repo root.

- No icons, images, textures, image loaders or icon fonts anywhere
  (current-ui.md 3) → the icon set of this file, painted by the icons module
  (`crates/lasercraft-ui/src/icons.rs`, new).
- Tool column: six text `selectable_label`s (Select, Line, Rectangle,
  Ellipse, Text, Edit Nodes) at `crates/lasercraft-ui/src/shell.rs:537-554`
  → icon-only toolbar buttons (`select`, `line`, the Shape tools flyout with
  `rect` and `ellipse`, `edit-nodes`, `text-tool`); until the set is drawn,
  `toolbars.md`'s transition rule (label in `type-small`) applies.
- Menus: plain text rows, chords via `Button::shortcut_text`, no gutter
  (`shell.rs:419-427`, `429-501`) → a reserved `icon-small` gutter with the
  icons listed under "Used by", ticks and choice dots (`menus.md`).
- Layers panel buttons "Move up", "Move down", "Delete shapes" as small
  text buttons (`crates/lasercraft-ui/src/layers_panel.rs:97-109`) → the
  icon column of `layers-panel.md` with `layer-up`, `layer-down`,
  `delete`, and later `settings-copy`, `settings-paste`.
- No window icon (`crates/lasercraft-app/src/main.rs:246-251` builds the
  viewport without one) → the app icon through `ViewportBuilder::with_icon`.
- No cursor changes per tool (current-ui.md 1.11; `canvas.rs`) → the cursor
  table above; system cursors first, badges with the icon set.
- The rotate handle is an orange ring on a line
  (`crates/lasercraft-ui/src/canvas.rs:375-378`) with the default arrow
  cursor → `canvas.md`'s handle with the painted `cursor-rotate`.
- Job preview "Cannot run:" is plain red text
  (`crates/lasercraft-ui/src/preview.rs:141`) → a `danger-soft` band with
  `mark-error` (`job-preview.md`); "Fits the work area." gains
  `mark-success`.
- egui's bundled default fonts are used as they are; no symbol glyphs are
  used as icons today, and none should be once the set exists (one style).

## Proposed changes to other specs

Each is a proposal (decision 15); the owning file is updated first, then
code. Where an owning style file and this file disagree, this file's
concept is proposed because it keeps one drawing per id and differs more
from the reference.

- **proposed change to toolbars.md**: `save-project` concept "tray with an
  arrow coming down into it" (not a storage card); `job-preview` "an eye
  above a half-dashed path" (not a screen); `device-profile` "gantry with a
  pencil" (not a gear); `measure` "caliper" (not a ruler); `trim` "crossing
  lines with the overhang dotted"; `text-tool` (renamed from `text`, which
  is also a color token) "text cursor beside text lines"
  (no letter T); `fit-bed` "bed between inward arrowheads" (no corner
  ticks); `group` and `ungroup` use a solid box (dashed means selection);
  the Move on bed rows (`bed-…`) show a solid bed with an accent square, not
  a dashed bed; open question 9 there (icon names) is answered by adopting
  the ids here, which keep every id toolbars.md already uses.
- **proposed change to menus.md**: Preferences… uses `preferences`
  (sliders), not a gear; Text uses `text-tool` (no letter); Fit bed uses
  `fit-bed` (no magnifier); Rotary attachment… uses `rotary` (rollers);
  Set target 1 and 2 use accent position, not digits; Slide ▸ rows use the
  motion-arrow-and-wall concept; Keyboard shortcuts… uses two key caps
  with a plus; camera Edit… uses `edit` (pencil), not a wrench; camera
  Export… uses `export-art` (not the save tray); Go to user origin, Set and
  Clear user origin use the flag motif, not a pin; Image adjustments… uses
  `image-adjust` (not sliders, which are Preferences); Job preview… uses
  `job-preview` (eye) as it already says; Remove linked copies uses normal
  strokes, because it is undoable (only deletes Undo cannot bring back are
  `danger`, matching `layers-panel.md`); the "Icon" column adopts this
  file's ids.
- **proposed change to machine-panels.md**: Go to user origin uses the
  flag motif (`go-user-origin`), not a pin; ±10% override buttons use the
  double chevrons `override-up-big` / `override-down-big`; Resume uses the
  `start` drawing (alias `resume`) as it says today; Refresh ports uses
  `refresh-ports` (two chasing arcs), distinct from the override "100%"
  `reset`.
- **proposed change to canvas.md**: the Move head badge is the head ring
  (`badge-move-head`), not a pin; the Measure badge is the caliper, not a
  ruler; the Trim badge is the `trim` icon (two crossing lines with the
  overhang dotted), not a scissors glyph; the Round corners no-room glyph
  (`badge-no-room`) is drawn in `warning`, not `danger`, so that `danger`
  stays with Stop, unrecoverable deletes and errors; badge ids are
  `badge-<tool id>` as listed here; snap marker ids `snap-…` as listed
  here.
- **proposed change to dialogs.md**: the Error message icon is an outline
  circle (`message-error`), not a filled disc; the Preferences General tab
  uses `prefs-general` (an outlined switch) so it does not repeat
  `preferences`; the Display tab uses `prefs-display` (a split window), not
  an eye, because the eye means job preview and overlay visibility.
- **proposed change to later-windows.md**: registration pictograms carry no
  digits (`set-target-1` and `set-target-2` tell the targets apart by the
  accent position); the Text data "freeze" concept is a snowflake
  (`freeze`); camera status badges are `badge-ok`, `badge-missing`,
  `badge-failed`; preset Delete uses `delete` and its confirmation button
  `delete-stored`; preset Export uses `export-art`.
- **proposed change to job-preview.md**: Play uses `play-preview`, which
  differs from the laser's `start`; inside the preview's drawing area the
  system progress cursor shows while a large job is being planned.
- **proposed change to layers-panel.md**: none in substance; it already
  uses `layer-up`, `layer-down`, `delete`, `settings-copy`,
  `settings-paste`, `mode-…`, `operation-…`, `image-row` (alias of
  `mode-image`) and `guide-layer`.
- **proposed change to ARCHITECTURE.md**: if option 2 is chosen, the crate
  graph line "lasercraft-ui ──► lasercraft-import (the UI only lists font
  families)" also names icon parsing; if option 3 is chosen instead,
  `egui_extras` (svg) gets a one-line justification in Dependencies. The
  app icon's raster generation (M10) is described under the packaging
  milestone.
- **proposed change to ui-shell.md**: the tool palette (Drawing tools,
  `naming.md`) shows icons with tooltips instead of text labels once the set
  exists; the main window sets an app icon.

## Open questions

1. **Implementation route** (decision for the toolbar milestone): option 2
   (own SVG files through our importer) as recommended, option 1 (all
   painted in code), or option 3 (a new SVG-rendering crate)? Options 2 and
   3 need an ARCHITECTURE.md change.
2. **Accent marking in source files**: does `lasercraft_import::svg::parse`
   pass element ids through (then the accent part is the element with id
   `accent`), or do we use a second `<id>.accent.svg` file?
3. **Grid unit count** for the drawing grid, and whether a documented small
   variant per icon is allowed at `icon-small` or every icon must read as
   drawn.
4. **Who draws the set**: the owner, the coding agent from this file's text,
   or a contributor; in every case the provenance list records it.
5. **Universal metaphors the reference also uses** (scissors, magnifier,
   padlock, house, waste bin, folder, clipboard, play, pause, stop): keep
   them, as this file proposes, or differentiate even these?
6. **App icon**: approve the direction (bed sheet, dashed-then-solid path,
   head ring, one accent blue, rounded tile) and the final drawing; is a
   wordmark wanted beside it, and in which font?
7. **Danger rule**: is "deletes that Undo cannot bring back" the right
   reading of decision 17's "delete-everything", which makes Remove linked
   copies normal and Remove (camera, device profile) `danger`?
8. **Digits**: confirm that print registration icons drop their digits;
   allow digits anywhere else (for example the jog distance presets, if any
   get icons)?
9. **Keyboard focus on icon buttons**: shared with `toolbars.md`; if
   toolbars become keyboard-navigable, the `accent` focus ring around icon
   buttons needs checking against the active-tool outline.
10. **High-contrast mode**: should a later accessibility theme thicken the
    stroke weight, or only change the tokens?
11. **egui's collapsing-header triangle**: keep egui's own drawing or
    replace it with `disclosure-closed` / `disclosure-open`?
12. **Cursor badges and Handle size**: should badges follow the Icon size
    preference instead of Handle size (`canvas.md` chose Handle size)?
13. **Pause key and icons**: none; but should `pause-motion-only`'s warning
    mark also appear on the job bar's compact Pause at `icon-small`, where
    it may be too small to read? (`machine-panels.md` says it does.)

## Sources

LightBurn 2.1 public documentation, read 2026-10-21, through the research
notes (`top-toolbars.md`, `left-toolbars.md`, `layout.md`, `menus.md`,
`machine-windows.md`, `canvas.md`). Only general facts about icon style,
placement, states and cursors are taken; no artwork is described beyond a
generic concept.

1. [R1] https://docs.lightburnsoftware.com/2.1/Reference/UI/MainToolbar/ — icon-only file and view toolbar, dark monochrome heavy glyphs, washed-out disabled look, concepts recorded for the "avoid" list
2. [R2] https://docs.lightburnsoftware.com/2.1/Reference/UI/ArrangeToolbar/ — icon-only arrange toolbar, the red-centred head target as the one colored icon, concepts for the "avoid" list
3. [R3] https://docs.lightburnsoftware.com/2.1/Reference/UI/MenuToolbar/ — the menu bar itself carries no icons
4. [R4] https://docs.lightburnsoftware.com/2.1/Reference/UI/FileMenu/ — icon gutter in menu rows, rows with and without icons
5. [R5] https://docs.lightburnsoftware.com/2.1/Reference/UI/EditMenu/ — disabled rows grey their icon with the label
6. [R6] https://docs.lightburnsoftware.com/2.1/Reference/UI/ToolsMenu/ — tool and boolean rows carry icons, long-tail rows none, an experimental marker icon
7. [R7] https://docs.lightburnsoftware.com/2.1/Reference/UI/ArrangeMenu/ — arrange menu icons (concepts for the "avoid" list), greyed rows with nothing selected
8. [R8] https://docs.lightburnsoftware.com/2.1/Reference/UI/LaserToolsMenu/ — icons on test, setup and device rows (concepts for the "avoid" list)
9. [R9] https://docs.lightburnsoftware.com/2.1/Reference/UI/WindowMenu/ — check boxes in the icon column, zoom and preview rows share toolbar icons
10. [R10] https://docs.lightburnsoftware.com/2.1/Reference/UI/HelpMenu/ — some help rows carry the program's brand mark
11. [R11] https://docs.lightburnsoftware.com/2.1/GetStarted/UITour/ — menus repeat toolbar icons and show shortcuts; tooltips name toolbar buttons
12. [R12] https://docs.lightburnsoftware.com/2.1/Reference/UI/ — default window map, overall icon style, logo in the title bar
13. [R13] https://docs.lightburnsoftware.com/2.1/Reference/UI/CreationToolbar/ — icon-only tool column, active-tool rounded background, colored family marker
14. [R14] https://docs.lightburnsoftware.com/2.1/Reference/UI/ModifiersToolbar/ — disabled modifier buttons and their marker turn grey; concepts for the "avoid" list
15. [R15] https://docs.lightburnsoftware.com/2.1/Reference/UI/ModesToolbar/ — larger icon-only setup buttons; concepts for the "avoid" list
16. [R16] https://docs.lightburnsoftware.com/2.1/Reference/PrimaryShapes/ — family button with a colored corner marker whose face follows the last choice
17. [R17] https://docs.lightburnsoftware.com/2.1/Reference/EditNodes/ — node strip glyphs with small colored marks; the node tool recolored while active
18. [R18] https://docs.lightburnsoftware.com/2.1/Reference/BooleanTools/ — flyout rows with small icon and label; marker greys when disabled
19. [R19] https://docs.lightburnsoftware.com/2.1/Reference/DrawLines/ — crosshair cursor with hollow centre and a small mode badge
20. [R20] https://docs.lightburnsoftware.com/2.1/Reference/RadiusFillet/ — rounding cursor badges including the red no-room mark
21. [R21] https://docs.lightburnsoftware.com/2.1/Reference/TrimShapes/ — scissors cursor while trimming
22. [R22] https://docs.lightburnsoftware.com/2.1/Reference/AddTabs/ — plus-with-tab cursor
23. [R23] https://docs.lightburnsoftware.com/2.1/Reference/Snapping/ — one cursor glyph per snap kind
24. [R24] https://docs.lightburnsoftware.com/2.1/Reference/ZoomPan/ — open-hand cursor while panning
25. [R25] https://docs.lightburnsoftware.com/2.1/Reference/TransformControls/ — four-way, two-way and curved-arrow cursors over handles
26. [R26] https://docs.lightburnsoftware.com/2.1/Reference/LaserWindow/ — big job buttons with a glyph left of the word, framing buttons told apart by glyph, title strip float and close glyphs, device-type logo badges
27. [R27] https://docs.lightburnsoftware.com/2.1/Reference/MoveWindow/ — glyph-only jog pad buttons, chevron and circular-arrow icons on override buttons
28. [R28] https://docs.lightburnsoftware.com/2.1/NewFeatures/NewIn2.1/ — blank menu icons for alignment, renewed message-box and warning icons, automatic toolbar icon size, rotary icon on Start
29. [R29] https://docs.lightburnsoftware.com/2.1/Reference/UI/Workspace/ — toolbar icon size and cursor size settings; settings tabs with pictograms
30. [R30] https://docs.lightburnsoftware.com/2.1/Reference/Devices/ — device list entries carry vendor logos
31. [R31] https://docs.lightburnsoftware.com/2.1/Reference/MoveSelectedObjects/ — move-target icon family and the red-centred head target
32. [R32] https://docs.lightburnsoftware.com/2.1/Reference/FileManagement/ — prompts with a warning triangle and a question circle
33. [R33] https://docs.lightburnsoftware.com/2.1/Guides/CustomizingTheLightBurnWindow/ — dock title strips with small float and close glyphs
34. [R34] https://docs.lightburnsoftware.com/2.1/Reference/AutomaticGuidelines/ — line cursor over the rulers before dragging a guide
35. [R35] https://docs.lightburnsoftware.com/2.1/Reference/Text/ — crosshair cursor of the text tool
36. [R36] https://docs.lightburnsoftware.com/2.1/Reference/TwoPointRotateScale/ — crosshair over a node while picking points
37. [R37] https://docs.lightburnsoftware.com/2.1/Reference/WarpDeform/ — crosshair cursor over warp handles
38. [R38] https://docs.lightburnsoftware.com/2.1/Reference/PositionLaser/ — the head-positioning tool (the notes record no cursor for it)
39. [R39] https://docs.lightburnsoftware.com/2.1/Reference/ManagingPreferences/ — preference submenu icons (concepts for the "avoid" list)
40. [R40] https://docs.lightburnsoftware.com/2.1/Reference/UndoRedo/ — history list rows with round arrow icons
42. [R42] https://docs.lightburnsoftware.com/2.1/Reference/Align/ — align toolbar icons (concepts for the "avoid" list)
43. [R43] https://docs.lightburnsoftware.com/2.1/Reference/FlipMirror/ — mirror icons (concepts for the "avoid" list)
44. [R44] https://docs.lightburnsoftware.com/2.1/Reference/Grouping/ — group and ungroup icons (concepts for the "avoid" list)

Existing specs and decisions used: `docs/clean-room.md` (draw every icon
ourselves, no copied artwork, provenance and review), `docs/spec/safety.md`
(rules 2, 7 and 9: laser-off framing, Stop and Pause always reachable,
motion-only pause), `docs/spec/ui-shell.md`, `docs/spec/keymap.md` and
`crates/lasercraft-ui/keymap/default.keymap` (chords named in Notes),
`ARCHITECTURE.md` (crate graph, dependency rule), the style-guide decisions
(1, 2, 3, 6, 7, 12, 13, 15, 16, 17), the shared tokens, and the other files
in `docs/spec/style/` named above.

Open standards: WCAG 2.x (non-text contrast target for icons, text contrast
on the Stop button); SVG (W3C) for the icon source subset; the freedesktop
icon theme conventions for installing the app icon on Linux.

Own design: every icon concept and drawing, the style rules (grid, stroke
weight, round joins, dash language, fills, the one-accent rule, danger and
state marks, no text), the motifs (sheet, tray, bed, head, selection box,
gantry, picture, flag), every id, the cursor badge glyphs and hotspot
rules, the app icon direction, the implementation recommendation, the file
locations and the reviewer checklist.
