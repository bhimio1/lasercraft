# Spec: style — job preview window

Milestones: M4 (the window exists today: drawing area, time slider, job
summary, bounds check, Show travel; everything here restyles it), M6 (the
preview follows Selected shapes only and Place job at, "Show in job
preview" from the job refused band, the constant-power note, the window
never covers the job bar), M7 (images and grey levels in the drawing), M8
(time estimates from the device profile); Play, Playback speed, Lighter
for low power, Invert colors, Show legend, Save image… and Run from here
are later, with no milestone yet (Run from here needs M6 first).

This file fixes how the job preview window looks and behaves as a piece of
UI: where it opens and how big it is, the order of its parts, how every
kind of move is drawn, the playback controls and switches, the legend, the
wording and look of the summary and the bounds-check line, its buttons, the
keys and mouse gestures inside it, and its dark theme. What the preview
*computes* (the plan, the move kinds, the estimate, the bounds check) stays
in `docs/spec/preview.md` and `docs/spec/planning.md`, which win where they
already decide something; this file proposes changes to them only under
"Proposed changes to other specs". Labels come from `naming.md`; colors,
type and sizes are tokens of `visual-language.md` (the master table, which
this file feeds with the meaning of `move-burn`, `move-unlit`,
`move-travel` and `preview-head`); the menu item and toolbar button that
open the window are in `menus.md` and `toolbars.md`; the bed, grid, origin
marker, stroke ladder and gestures it shares with the main drawing area are
`canvas.md`'s; window chrome, button placement and the "no modal over the
job bar" rule are `dialogs.md`'s and `layout.md`'s; the job bar, the job
refused band and the constant-power wording are `machine-panels.md`'s;
icon artwork is `icons.md`'s. `README.md` explains the tiers (Match, Echo,
Own).

## How to read this file

- **Reference** paragraphs say what LightBurn's public 2.1 docs show about
  the reference's preview window, each fact tagged [Rn] (see Sources).
  Where the docs are silent the fact is "unknown" and the LaserCraft part
  is marked own design. Reference controls are described in our own words,
  never by their on-screen captions; the mapping from our labels to the
  doc terms is only in `naming.md`.
- **LaserCraft** paragraphs are the spec. Colors are named only by token;
  sizes only relative to `row`, `icon-small`, `gap` or the stroke ladder.
- **Stroke weights and dashes** use `canvas.md`'s ladder ("How to read this
  file"): *hairline*, *normal*, *heavy*, *marker*; dash rhythms *dashed*
  and *dotted*. All are constant on screen at every zoom and follow the
  display's scale factor.
- **Tier** per element: Match (same arrangement and behaviour), Echo (same
  idea, our own expression), Own (ours alone).
- **Milestone** per element: *Now* means it exists in some form today and
  is restyled; M6–M8 as in CLAUDE.md; *later* means no milestone yet.

---

## The window

### Reference

- The preview is a separate top-level window with the operating system's
  own title bar, opened from the reference's window menu, a button in its
  main toolbar, Alt+P (Option+P on macOS), or the drawing area's
  right-click menu [R1] [R2].
- It is resizable (a size grip shows in the lower right corner) and wider
  than tall; its default size and position, whether it is modal and
  whether it remembers its size are not stated (unknown) [R1] [R2] [R3].
- The drawing takes most of the window, roughly the upper four-fifths; the
  controls form a compact band of a few short rows along the bottom [R1]
  [R2] [R3].
- The docs advise previewing every job before sending it, and say the
  preview's settings change only the preview, never the output [R1] [R2].

### LaserCraft

**Title and opening**

- Title: "Job preview" (`naming.md`; tests look for it). The title never
  carries an ellipsis and does not change with the job.
- Opened by View → Job preview… (Alt+P, `ui.preview`; `menus.md`), the
  Standard toolbar's Job preview… button (`toolbars.md`), the canvas menu's
  Job preview… item (later, `menus.md`) and, from M6, the "Show in job
  preview" button of the job refused band (`machine-panels.md`). It works
  with no device connected (`docs/spec/preview.md`).
- Alt+P, or any other way of opening it, while the window is already open
  brings it to the front and gives it focus; it never opens a second
  preview (Own).
- Opening plans the job (the app does this, not the UI;
  `docs/spec/preview.md`). The window appears at once; if planning is still
  running (worker thread, later) it shows the busy state below.

**Kind of window**

- An `egui::Window` inside LaserCraft's main window (as today), titled,
  movable, resizable from its edges and lower right corner, **not
  collapsible** (double-clicking the title does nothing; Own), with the
  standard close button (×) at the right end of the title strip
  (`dialogs.md` chrome).
- Non-modal: the canvas, panels and menus stay usable while it is open, and
  editing the project re-plans the preview (`docs/spec/preview.md`). This
  is what makes "edit, glance at the preview, edit again" quick.
- Never covers the toolbar band: the window is kept inside the area below
  toolbar row 2 and above the status bar, so the job bar at the right end
  of toolbar row 1 stays visible and clickable while connected (decision 7,
  `layout.md`). Dragging its title above that line stops at the line.
- A separate OS window (as the reference has) is later and optional: it
  needs eframe multi-viewport, and Wayland may ignore its position
  (`layout.md`, floating panels). Open question 2.

**Size and position**

- First opening in a session: centered over the canvas region, about
  two-thirds of the canvas region's width and three-quarters of its height,
  so the canvas stays partly visible around it (Own). Landscape, like the
  reference [R1].
- Minimum size: wide enough for the playback row at `type-body` without
  wrapping in its "Now" form (Time row and Show travel), and tall enough
  for the drawing area to stay at least about six `row`s tall below the
  title and above the controls. Below that minimum the window cannot be
  made smaller (Own).
- Later rows (Play, Playback speed, the extra switches) wrap onto a second
  line rather than forcing a wider minimum.
- Size and position are remembered while LaserCraft runs; closing and
  reopening restores them. Keeping them across runs needs eframe's
  `persistence` feature or our own settings file (`layout.md`; later).
- If the main window shrinks so the remembered rectangle no longer fits,
  the preview is moved and shrunk to fit inside the area described above.
- A larger "fill the canvas region" size (the full-screen preview that
  `layout.md` mentions) is later: a button in the title strip, beside ×,
  with an icon of four outward corners (`icons.md`), tooltip "Fill the
  canvas area". Even then the toolbar band and the job bar stay visible.

**Focus**

- When the window has focus its title strip shows the focused-panel
  `accent` line along its lower edge (decision 2, `layout.md`), and the
  view keys below act on the preview instead of the canvas.
- Clicking in the drawing area gives the window focus without changing
  the slider.

Tier: **Echo** (a separate preview window opened the same ways, wide, with
the drawing on top and a short control band below), **Own** (in-app window,
non-modal, never over the toolbar band, size rules).
Milestone: Now (window), later (separate OS window, fill-canvas size,
persistence).

---

## Layout inside the window

### Reference

- Top to bottom: the drawing area; a thin time slider across the full
  width; one centered line of job statistics; a row of playback settings
  (a labelled speed slider with its value, three switches, and the time at
  the slider position at the right end); a bottom row with a legend switch
  at the left and four equal-width text buttons at the right [R1] [R2]
  [R3].
- In an older 2.0 screenshot the bottom-left spot holds a fixed hint
  explaining the two line colors instead of the legend switch [R2].

### LaserCraft

The order (top to bottom) follows the reference (Match); every part is our
own expression. Parts marked *later* are simply absent until built; the
rows close up around them.

| # | Part | Contents, left to right | Height | Milestone |
|---|---|---|---|---|
| 1 | Drawing area | the bed, the job's moves, the simulated head, the legend (top-left corner), a busy or empty-state chip (centered) | everything left over; grows and shrinks with the window | Now |
| 2 | Separator | one *hairline* in `border`, full width | — | Now |
| 3 | Time row | label "Time (s)", the time slider (stretches), its value field, button "Show whole job" | one `row` | Now |
| 4 | Job summary | up to three lines of text (see "Job summary") | one line of `type-body` each | Now |
| 5 | Check and notes | the bounds-check line, then the constant-power note (M6), the Selected shapes only note (M6), then warnings | one band each; collapsible after three warnings | Now (check, warnings), M6 (notes) |
| 6 | Playback row | label "Playback speed" and its slider and value (later), switches "Show travel", "Lighter for low power" (later), "Invert colors" (later), then (pushed right) the position read-out | one `row` (wraps to two when narrow) | Now (Show travel, read-out), later (rest) |
| 7 | Button row | switch "Show legend" (later) at the left; at the right, buttons "Run from here" (later, needs M6), "Save image…" (later), "Play" (later), "Close" | one `row` plus `gap` above | Now (Close), later (rest) |

- Margins: the drawing area runs edge to edge inside the window frame (no
  inner margin), so the bed gets every bit of space; rows 3–7 sit inside the
  normal window margin with `gap` between rows and `gap-wide` above row 7.
- All controls rows are left-aligned except the read-out at the end of
  row 6 and the buttons at the right of row 7, which are right-aligned.
- Rows 4 and 5 can grow (many warnings); when the window is short they
  scroll inside a box whose height is capped at about four `row`s, so the
  drawing area never shrinks below its minimum.
- The window's body is `panel`; the drawing area is drawn on `canvas`
  (see "Drawing area").

Tier: **Match** (drawing above, slider, statistics, settings row, button
row with the legend switch at the left and the buttons at the right),
**Own** (the check and notes bands, read-out content, margins).
Milestone: Now (order and rows that exist), later as marked.

---

## Drawing area

### Reference

- The drawing background is a very light, slightly cool grey (white in an
  older screenshot) [R1] [R2] [R3].
- The edge of the machine's work area is a thin pale green line; zooming
  out shows it as a full rectangle, at normal zoom often only one or two
  of its sides are in view [R1] [R3].
- When first opened the preview shows the finished result as a still
  picture [R2].
- No grid, rulers or origin marker are mentioned or visible (unknown) [R1]
  [R3].

### LaserCraft

Back to front (Own):

| Layer | What | Token and stroke | Notes |
|---|---|---|---|
| 1 | Surround (all of the drawing area outside the bed) | filled `canvas` | as on the main canvas |
| 2 | Bed sheet | filled `bed` (or the inverted sheet, see Invert colors) | the work area from the project (M8: the device profile) |
| 3 | Grid | `grid-minor`, `grid-major`, *hairline*, same spacing and hiding rules as `canvas.md` "Grid" | only on the bed; turned off by Invert colors |
| 4 | Bed edge | *normal* solid `bed-edge` | the same look as the canvas, so the two read as the same bed (`canvas.md`, "The job preview and the canvas") |
| 5 | Machine origin marker | as `canvas.md` "Machine origin marker" | shared look |
| 6 | Travel moves | `move-travel` | when Show travel is on |
| 7 | Unlit moves | `move-unlit` | always |
| 8 | Burn moves | `move-burn` | always |
| 9 | Out-of-bounds marker | `danger` ring | only when the bounds check fails |
| 10 | Simulated head | `preview-head` | see "Simulated head" |
| 11 | Legend chip, busy chip, empty-state chip | `field` chips with `border` | on top of everything |

- Orientation and coordinates are the canvas's: rear of the bed at the
  top, the same machine origin corner (`canvas.md`).
- From M6 the job is drawn where it will actually land: Place job at and
  the job reference point move the drawing on the bed exactly as they move
  the real job (`machine-panels.md`); with Selected shapes only on, only
  the selection's moves are drawn. The preview always shows the same job
  that Start, Frame and Save G-code… would use.
- Nothing is drawn outside the drawing area: the painter is clipped to it.
- Antialiasing is always on (egui); no switch.

**Initial view and fitting**

- The first time the window opens (and whenever "fit" is asked for) the
  view fits the union of the bed and the job's bounds, with a small margin
  of `canvas` all round, so off-bed moves are always in view (today's
  behaviour, `docs/spec/preview.md`).
- Resizing the window keeps the zoom and keeps the point at the center of
  the drawing area in place (Own). Today any resize refits the view
  (`preview.rs` canvas()); that is a difference to fix.
- Re-planning after an edit keeps the zoom and pan (today's behaviour).
- Opened through "Show in job preview" (M6), the view instead centers on
  the out-of-bounds point at a zoom where the offending shape fills about
  half the drawing area.

**Busy state** (later, with planning on a worker thread)

- While the app plans, the last drawing stays, faded towards `canvas`, and
  a centered chip says "Planning…" in `type-body` with `text-muted`. The
  slider, Play, Run from here and Save image… are disabled; zoom, pan and
  Close keep working.

**Empty state**

- A job with no output (no output layer has shapes) draws the bed only and
  shows a centered `field` chip with `text-muted` text "Nothing to cut: no
  output layer has shapes." (today's text, `naming.md` empty states). The
  slider is disabled at 0; Show whole job, Play, Run from here and Save
  image… are disabled.

Tier: **Match** (the work area edge drawn around the job, fit to the bed
and job on open), **Own** (layer order, the shared bed look, grid and
origin marker, busy and empty chips, resize rule).
Milestone: Now (restyle), M6 (placement, selection, Show in job preview),
later (busy state).

---

## How moves are drawn

### Reference

- Laser-on moves are near-black, whatever the layer: line work as thin
  dark strokes, filled areas as solid dark regions that resolve into tight
  parallel scan lines when the line interval is coarse or when zoomed in,
  and images as dark raster areas; areas filled twice look darker [R1] [R2]
  [R3].
- Laser-off moves (travel between cuts, and the overscan turnarounds of
  fills and images) are thin light red or pink lines; beside dense fills
  they merge into solid pinkish bands [R1] [R2] [R3].
- The two kinds are not told apart beyond these two colors [R1] [R3].

### LaserCraft

LaserCraft keeps `docs/spec/preview.md`'s three move kinds, which come
from the planner (`docs/spec/planning.md`), and draws each one in its own
way. Color is never the only difference: line style differs too, so the
kinds stay apart for color-blind users and in the inverted view (WCAG 2.2,
1.4.1 Use of Color).

| Kind (prose name) | What it is (planning.md) | Token | Stroke | Rhythm |
|---|---|---|---|---|
| Burn moves | laser on, at the layer's speed and power | `move-burn`: the move's layer color (`docs/spec/preview.md`) | *normal* | solid |
| Unlit moves | laser off along the toolpath at layer speed: overscan run-ons, gaps between fill segments, tab gaps | `move-unlit`: light grey | *hairline* | *dotted* |
| Travel moves | laser off between cuts at top speed | `move-travel`: mid grey | *hairline* | *dashed* |

Rules:

- **Burn moves keep layer colors.** `docs/spec/preview.md` already decides
  this, and it tells the user at a glance which layer does what. This
  differs from the reference's single dark color (open question 1 asks
  whether to add a one-color option). With Invert colors on, burn moves
  turn light instead (see "Invert colors").
- **Fill scan lines** are burn moves like any other: each line is drawn
  *normal*, so a fill seen from afar merges into a solid area of its layer
  color and resolves into lines when zoomed in, like the reference [R2]
  [R3] (Match). Where two fills overlap, the overlap simply draws twice;
  no extra darkening is computed (Own).
- **Overscan beside fills** becomes many dotted `move-unlit` lines that
  merge into a pale grey band beside the fill; this is wanted, because it
  shows how far the head runs past the art (the reference shows the same
  idea in pink [R3]).
- **Tab gaps** in line work show as short dotted stretches in an otherwise
  solid outline, so tabs are visible without any extra marker.
- **Travel is never red.** Red means danger and errors only (decision 6);
  travel is ordinary and safe. It is also never `warning` amber, which is
  the frame path's and "laser may stay on" color.
- **Images** (M7): image layers are drawn as their scan lines, each burn
  segment in the layer color (or, with Lighter for low power, in its grey
  level); from afar the image reads as a picture in the layer's color.
- **Low-contrast layer colors** (the bright yellows on the light bed) get
  the same darker *hairline* edge treatment as on the canvas, if the owner
  approves it there (`canvas.md` open question 10); the preview follows
  whatever the canvas decides.
- **Order of drawing** is the order of the job within each kind's layer of
  the stack (back to front: travel, unlit, burn), so burns are never hidden
  by laser-off lines.
- **Partly drawn move.** While the slider is before the end, only moves
  that have started by that time are drawn; the move under way is drawn up
  to the head (`docs/spec/preview.md`).
- **Line weight does not change with zoom** (stroke ladder), so zooming in
  separates scan lines rather than thickening them.

Tier: **Match** (laser-on and laser-off moves drawn differently, travel
optional, fills resolving into scan lines, overscan visible), **Own**
(three kinds, layer colors, grey dotted and dashed laser-off styles, no red).
Milestone: Now (restyle), M7 (images, grey levels).

---

## Simulated head

### Reference

- A small thin red plus marks the simulated head; at the start of the
  slider it sits at the start position, at the end at the job's last point
  [R1] [R3].

### LaserCraft

- A filled dot in `preview-head` (an `accent` dot) about the size of a
  canvas scale handle, with a thin `field` ring around it so it stays
  visible over dark layer colors and over the inverted sheet (Own).
- A dot, not a crosshair: the canvas's `head-marker` crosshair means the
  *real* head and is never drawn in the preview, and the simulated head is
  never drawn on the canvas (`canvas.md`, "The job preview and the
  canvas"). Not red, because nothing here is firing.
- Shown at all times once a job exists: at the slider's time while
  scrubbing or playing, and at the end of the last move when the slider is
  at the end (Match [R1] [R3]). Today it appears only after the slider has
  been moved (`preview.rs` canvas()); that is a difference to fix.
- At time 0 it sits at the job's start position (the start of the first
  move).
- Hidden in the empty state.

Tier: **Echo** (a small marker that follows the slider), **Own** (accent
dot with a ring, never a crosshair).
Milestone: Now.

---

## Out-of-bounds marker

### Reference

- The docs show the work-area edge so the user can zoom out and check the
  art's placement [R1]. No specific marker for a point outside the bed is
  described (unknown).

### LaserCraft (own design)

- When the bounds check fails (`Plan::check_bounds`), the furthest-out
  point it names is marked with a `danger` ring, *heavy*, about twice the
  head dot's size, with a thin `field` ring outside it for contrast.
- The part of the bed edge that the job crosses is drawn *heavy* in
  `danger` along the offending side, so the problem is visible even when
  the ring is off screen.
- The marker never blinks.
- Needs the point and the edge to be handed to the UI with the error
  (today only the message string is), a proposed change to preview.md.

Tier: **Own**.
Milestone: Now (with the restyle; needs the data change).

---

## Time row: Time (s) and Show whole job

### Reference

- A thin full-width slider sits directly under the drawing. It opens at the
  far right, showing the whole job; dragging it left shows the job only up
  to that moment and the head marker follows [R1] [R3].
- In the current screenshots the groove left of the handle is filled in a
  saturated blue and the handle is a small light rounded square [R1]. An
  older screenshot differs: a solid blue upright handle on a plain grey
  groove with no fill [R2].
- There is no separate "whole job" button (unknown whether keys move the
  slider).

### LaserCraft

| Order | Control | Label | Tooltip | Enabled | Milestone |
|---|---|---|---|---|---|
| 1 | label | "Time (s)" | — | always | Now (today "time" after the slider) |
| 2 | slider (stretches to fill the row) | — | "Drag to see the job up to this moment" | a job exists | Now |
| 3 | value field (egui's slider value) | — | "Time into the job in seconds; type a value to jump there" | a job exists | Now |
| 4 | button | "Show whole job" | "Move the slider to the end of the job" | a job exists and the slider is not at the end | Now (today "Whole job") |

- Look: label `type-body` `text`, right-aligned to the slider like every
  label in LaserCraft (`visual-language.md` density). Slider groove a thin
  `border` line; the part left of the handle filled `text-muted` (egui's
  trailing fill, kept neutral because decision 3 does not list slider
  fills among the uses of `accent`; open question 15 proposes adding it);
  handle a small round knob in `field` with a `border` ring, `accent`
  focus ring when focused (focus rings are an allowed `accent` use; Echo
  of [R1]). Value field `field`
  with `type-mono` digits, one decimal, no unit suffix (the unit is in the
  label, `naming.md`).
- Range 0 to the total estimated time. Opens at the end (whole job),
  matching the reference [R1] (Match). Re-planning keeps the slider time,
  clamped to the new total (today's behaviour).
- Dragging redraws continuously. Clicking the groove jumps there.
- With the slider focused: Left and Right step one move back or forward
  (Own; the read-out's move number changes by one); Shift+Left/Right step
  ten moves; Home and End jump to the start and the end. Left, Right,
  Shift+Left and Shift+Right are bound in `default.keymap` to nudging the
  selection (`shape.move`, keymap.md "Arrows: Nudge selection"), and a
  focused slider is not a text field, so keymap.md's text-field rule does
  not cover it. This file therefore makes the slider keys a focus-scoped
  override: while the preview's slider has keyboard focus these keys step
  moves and do **not** nudge the selection; with the slider unfocused they
  nudge as usual. Home and End alone are not bound in `default.keymap`.
  This override is a proposed change to keymap.md (see "Proposed changes
  to other specs").
- Later: clicking a burn move in the drawing area moves the slider to the
  end of that move (Own; open question 12).

Tier: **Match** (full-width slider under the drawing, opens at the end,
head follows), **Echo** (filled groove, in a neutral token), **Own**
(label, typed value, Show whole job, step keys).
Milestone: Now.

---

## Playback row

### Reference

- A labelled short slider sets the playback speed, from a tenth of real
  time to forty times real time, with its value shown beside it [R1].
- Three switches follow: show or hide travel moves, shade output by power
  (only meaningful for greyscale images, power scaling or ramped power),
  and invert to light on dark for dark materials [R1].
- The time at the slider position is shown at the right end of the row; it
  reads zero at the start and the total at the end [R1] [R2].
- Defaults seen: travel shown, shading off (on in some shots), invert off
  [R1] [R2] [R3].

### LaserCraft

| Order | Control | Label | Tooltip | Default | Enabled | Milestone |
|---|---|---|---|---|---|---|
| 1 | label + slider + value | "Playback speed" | "How fast Play runs: from a tenth of real time to forty times" | 1× | a job exists | later |
| 2 | switch | "Show travel" | "Draw laser-off travel moves." | on | always | Now (a checkbox today) |
| 3 | switch | "Lighter for low power" | "Draw lower-power burns lighter. Only changes the picture." | off | always (has an effect only when powers differ) | later |
| 4 | switch | "Invert colors" | "Draw burns light on a dark sheet, as on dark materials that engrave lighter. Only changes the picture." | off | always | later |
| 5 | read-out (pushed right) | — | "Time at the slider and total estimated time; move number at the slider" | — | — | Now (new) |

- Switches are LaserCraft's pill switch (decision 4, `visual-language.md`):
  `accent` fill with the knob right when on, neutral grey with the knob
  left when off; the label sits right of the switch. Separated by `gap`;
  `gap-wide` between the speed control and the first switch.
- None of these settings change the job or the G-code; they are view
  settings of the preview, not saved in the project, remembered while
  LaserCraft runs (later: across runs with persistence). They are not
  commands and are not undoable (like zoom).
- **Playback speed**: a short slider, about as wide as three `row`s, on a
  logarithmic scale from 0.1× to 40× (Match [R1] range), with the value
  shown after it as "0.1×", "1×", "2.5×", "40×" (`type-mono`). It snaps
  to 0.1, 0.25, 0.5, 1, 2, 5, 10, 20 and 40 while dragging unless Shift is
  held (Own).
- **Read-out**, `type-small` in `text-muted`, right-aligned at the end of
  the row: "‹time at slider› of ‹total› · move ‹n› of ‹N›", times as
  h:mm:ss (`naming.md` numbers table), for example "0:01:12 of 0:03:40 ·
  move 120 of 1234". Hours keep counting past 24 (as the reference's
  screenshots show for long jobs [R3]). In the empty state it reads "No
  moves".
- When the window is too narrow, the read-out moves to its own line first,
  then the switches wrap.

### Lighter for low power (later)

- Each burn move is drawn with its layer color blended towards `bed` in
  proportion to how far its power is below the device's maximum: full
  power draws the pure layer color, low power a pale version of it (Own;
  the reference shades in grey [R1] [R3]). With Invert colors on, the blend
  goes towards the inverted sheet instead.
- Needs the power of each move in what the app hands the UI (today
  `PreviewKind::Burn` carries only the color); proposed change to
  preview.md.
- Meaningful once moves of one layer differ in power (M7 grey-level images,
  later power ramps); with one power per layer the picture barely changes,
  and the tooltip's second line says so: "Most useful for images and
  ramped power."

### Invert colors (later)

- The bed sheet is filled with `text` (near-black) instead of `bed`, the
  grid is hidden, and every burn move is drawn in `bed` (near-white), or in
  grey levels with Lighter for low power. Unlit and travel moves keep their
  grey tokens but are drawn over the dark sheet, where both still read
  (Own; the reference draws light output on black [R1]).
- Layer colors are dropped in this view on purpose: its job is to show
  roughly how the burn will look on a dark material, not which layer does
  what. The legend says so (see "Legend").
- Independent of the app theme: the dark theme does not invert the sheet,
  and Invert colors works in both themes.
- Open question 8 asks `visual-language.md` for dedicated tokens for the
  inverted sheet and burns.

Tier: **Match** (playback speed range, the three switches and their order,
time at the right end), **Own** (labels, pill switches, snapping speeds,
read-out with the move number, colored shading, inverted rendering).
Milestone: Now (Show travel, read-out), later (rest).

---

## Legend (later)

### Reference

- When its switch is on, a small key floats at the drawing's upper left,
  with no panel behind it: two rows, each a filled square swatch
  with a thin dark outline and a word, one for laser-on moves and one for
  laser-off moves [R1].

### LaserCraft

- Switch "Show legend" at the left end of the button row (Match [R1]);
  tooltip "Show a key to the line styles in the corner of the drawing."
  Default **on** (Own: our line styles are new to switchers; open question
  5).
- The legend sits at the upper left of the drawing area, a `gap` in
  from both edges, on a small `field` chip with a *hairline* `border` and
  slightly rounded corners, so it stays readable over moves (Echo: the
  reference draws it straight onto the drawing).
- Rows, top to bottom (labels from `naming.md`), each a short sample line
  about two `icon-small`s long drawn exactly like the moves, then the label
  in `type-small` `text`:

| Order | Label | Sample | Shown when |
|---|---|---|---|
| 1 | "Burn" | a solid *normal* line made of short segments in the colors of the job's output layers, in output order (at most six; more layers are not added) | always; with Invert colors the sample is `bed` on a small `text` square |
| 2 | "Unlit" | a *hairline* *dotted* line in `move-unlit` | always |
| 3 | "Travel" | a *hairline* *dashed* line in `move-travel` | Show travel is on |

- Samples are lines, not filled squares, because the kinds differ by line
  style as well as color (Own).
- The legend is not part of the job's picture: it never moves with zoom or
  pan, and Save image… leaves it out unless the legend is on (then it is
  included in the same corner).
- Pointer over the legend: it does not block wheel zoom or panning.

Tier: **Match** (a switchable key in the top-left corner), **Echo** (chip
behind it), **Own** (line samples, three rows, layer-color sample).
Milestone: later.

---

## Job summary

### Reference

- One centered line of plain dark text under the slider: the distance
  travelled with the laser on and its approximate time, the distance with
  the laser off and its time, and the total estimated time [R1] [R2].
- Times read as minutes and seconds or hours, minutes and seconds [R3].
- The estimates come from simulation values in the device settings
  (speeds, accelerations, scale factors), which only tune the simulation
  and time estimate and do not change how the job runs [R1] [R4].

### LaserCraft

The app builds these lines and hands them to the UI (`docs/spec/preview.md`;
the UI computes nothing). Content stays the existing spec's: burn, unlit and
travel distances with their times, the total, the move count, and the job
bounds with overscan.

| Line | Text (‹…› filled by the app) | Look |
|---|---|---|
| 1 | "Burn ‹d› mm (‹t›), unlit ‹d› mm (‹t›), travel ‹d› mm (‹t›)" | `type-body`, `text` |
| 2 | "Estimated time ‹t› for ‹n› moves" | `type-body`, `text`; the time in bold weight of the same family |
| 3 | "Job bounds (‹x›, ‹y›) – (‹x›, ‹y›) mm, ‹w› × ‹h› mm with overscan" | `type-small`, `text-muted` |
| 3 (empty job) | "Nothing to cut: no output layer has shapes." | `type-small`, `text-muted` (the same text is in the empty-state chip) |

- Wording changes from today: "laser-off run-on" becomes "unlit"
  (`naming.md`); nothing else.
- Numbers follow `naming.md` "Numbers in fields and read-outs": distances
  with two decimals, times as h:mm:ss ("0:03:40"; hours keep counting past
  24). Today's times use the planner's `duration` format ("4 min 05 s");
  that is a difference to fix (the app formats UI times; the CLI keeps its
  own format).
- Left-aligned, not centered (Own: left-aligned lines are easier to scan
  and do not jump when numbers change).
- Line 2's tooltip says where the estimate comes from: "Estimated from the
  default machine's speed and acceleration." until M8, then "Estimated
  from ‹device profile›'s speed and acceleration."; a second line adds
  "Real jobs can take longer." (Echo of [R1] [R4]).
- Later, owner decision: a "Per layer" disclosure under line 3 opening a
  small striped table (swatch, layer name, mode, burn distance, time) like
  the CLI's per-layer lines (`docs/spec/cli.md`). Open question 13.

Tier: **Echo** (statistics under the slider: distances with times and the
total), **Own** (three lines, unlit as its own figure, move count, bounds,
left alignment, wording).
Milestone: Now (restyle and wording), M8 (device profile in the tooltip).

---

## Check and notes

### Reference

- The preview window itself shows no bounds message in the screenshots;
  the docs only suggest zooming out to compare the art with the work-area
  edge [R1]. Unknown whether anything else is shown there.

### LaserCraft (own design)

Each item is a band the width of the controls area, one line tall unless
the text wraps, with a small mark at its left (`icon-small`, `icons.md`)
and text in `type-body` `text`. Order, top to bottom:

| Order | Item | Text | Look | When | Milestone |
|---|---|---|---|---|---|
| 1 | Bounds check: fits | "Fits the work area." (keep; tests) | no band; a `success` check mark and `text` | the job fits | Now |
| 1 | Bounds check: refused | "Cannot run: ‹reason›" (keep; tests look for "Cannot run:") | `danger-soft` band, `danger` mark, `text` | `Plan::check_bounds` failed | Now (today plain red text) |
| 2 | Constant-power note | "Constant power (M3) on ‹layer names›. The laser may stay on while paused; Stop turns it off." (`machine-panels.md` wording) | `warning-soft` band, `warning` mark | an output layer uses constant power | M6 |
| 3 | Selected shapes only note | "Selected shapes only is on: previewing ‹n› selected objects." | `info-soft` band | the Selected shapes only switch is on (`machine-panels.md`) | M6 |
| 4… | Warnings | "Warning: ‹text›" (keep; tests) | `warning-soft` band, `warning` mark | one per planner warning | Now (today weak text) |

- ‹reason› is the planner's message (axis, distance outside and shape;
  `docs/spec/safety.md` rule 5), unchanged.
- The bounds check runs on every plan, connected or not, so problems show
  before anyone reaches Start (safety rule 5 is enforced again at Start,
  `machine-panels.md`).
- After three warnings the rest fold into a line "‹n› more warnings" with
  a disclosure arrow; clicking it shows them all (inside the scrolling box
  of the layout section).
- Clicking a refused band or a warning that names a shape selects that
  shape on the canvas (later; Own) and, for the refused band, pans the
  preview to the out-of-bounds marker.
- "Fits the work area." uses no band on purpose: the common, good case
  stays calm; only problems get colored bands.
- A `success` mark alone is never the only signal: the words say it
  (WCAG 2.2, 1.4.1).

Tier: **Own**.
Milestone: Now (check and warnings restyled), M6 (notes).

---

## Button row

### Reference

- At the right of the bottom row, four equal-width text buttons: start the
  real job from the slider's moment (greyed while the slider is at the
  end, meant for recovering an interrupted job), save the preview as an
  image file, play the simulation from start to end at the playback speed,
  and close the window [R1] [R2].
- Whether starting partway asks for confirmation, what Play shows while
  playing, and the image formats are not stated (unknown) [R1].

### LaserCraft

| Order | Button | Tooltip | Enabled | Milestone |
|---|---|---|---|---|
| left | switch "Show legend" | see "Legend" | always | later |
| 1 (right group) | "Run from here" | "Start the real job from the slider position. Asks before anything is sent." | see below | later (needs M6) |
| 2 | "Save image…" | "Save the drawing as a PNG image" | a job exists | later |
| 3 | "Play" / "Stop playback" | "Animate the job at the playback speed" | a job exists | later |
| 4 | "Close" | "Close the job preview" | always | Now (new; today only ×) |

- Buttons are text-only push buttons of equal width (the widest label sets
  it), `row` tall, `gap` apart, right-aligned (Match [R1]). None is styled
  as the primary (`accent`) button and none is the default for Enter: the
  preview has no "confirm" action, and Run from here must never be one key
  press away (Own).
- Close is last, at the far right (Match the reference's closing button
  position; label "Close" per `naming.md`, because the window edits
  nothing).
- Esc while the preview window has focus closes it, like Close (Own;
  proposed change to keymap.md, open question 4). Esc with focus elsewhere
  keeps its keymap meaning.

### Play (later)

- Play runs the slider forward at Playback speed times real time,
  redrawing the moves and the head as it goes (Match [R1] [R2]).
- If the slider is at the end, Play starts from 0; otherwise from the
  slider's position (Own).
- While playing the button reads "Stop playback" (Own; deliberately not
  "Pause", which in LaserCraft means the machine's Pause, `naming.md`).
  Clicking it, dragging the slider, or pressing Show whole job stops
  playback where it is. Reaching the end stops it and leaves the slider at
  the end.
- Changing Playback speed while playing takes effect at once.
- Edits to the project during playback re-plan and stop playback.
- Playback asks egui to repaint every frame only while playing; when idle
  the window does not repaint continuously.
- The Reduce motion preference (`canvas.md`) does not affect Play: the
  user started it.

### Save image… (later)

- Opens a small window titled "Save image" (`dialogs.md` path window, until
  native file dialogs exist) asking for a `.png` path. The image is the
  drawing area's content at the current slider time and with the current
  switches, framed to fit the bed and the job (not the current zoom), on
  the `bed` sheet without the surround, at a size chosen in the window
  (Own). The legend is included only if it is shown.
- Writing PNG needs an encoder (the `image` crate is planned for M7);
  adding it needs an ARCHITECTURE.md line. Open question 6.
- A failure is reported in the window as "Cannot save image: ‹reason›"
  (`naming.md` error form).

### Run from here (later; needs M6)

This button starts the real laser, so `docs/spec/safety.md` applies in full.
Own design throughout, built on the reference's idea of restarting an
interrupted job from a point in the timeline [R1].

- **Shown** only while connected (`machine-panels.md` machine states);
  absent, not just disabled, when disconnected, so the preview stays a
  pure simulation for users without a machine.
- **Enabled** only when all of these hold: the machine is Ready (Idle);
  the bounds check passed ("Fits the work area."); the slider is after the
  start and before the end of the job (at the start, Start does the same
  job; at the end there is nothing left; matches the reference's greyed
  state at the end [R1]); no job or framing is running. Otherwise the
  tooltip's second line names what is missing ("Needs a connected,
  ready machine.", "Move the slider into the job.", "The job does not fit
  the work area.").
- **Start point**: the job restarts at the beginning of the move under the
  slider, rounded back to the start of that move (never mid-move); the
  head goes there with a travel move, laser off (safety rule 1, 4). The
  drawing marks this restart point with the `preview-head` dot inside a
  second ring while the button is hovered (Own).
- **Always confirmed**: clicking opens a confirmation (`dialogs.md`
  confirmation layout) titled "Run from here" with the text "Run the job
  from ‹time› (move ‹n› of ‹N›)? The head moves to that point with the
  laser off, then the rest of the job runs.", then the constant-power note
  if it applies, and buttons "Run from here" and "Cancel". Cancel is the
  default and Esc cancels. This confirmation must not be an `egui::Modal`
  over the job bar (`layout.md`).
- **Never** bound to a key chord and never reachable from the command line
  without its own explicit command (`job.start_from`, new, M6, not
  undoable).
- The remaining job is checked against the work area again before anything
  is sent, including the travel to the restart point (safety rule 5); the
  same power clamping and `M4` default apply (rules 6, 9).
- Once running, the job bar shows the job as for Start, with Pause and Stop
  (decision 7); the preview stays open and non-modal, and its controls that
  would change the running job (Run from here, the slider jump while
  following, later) are disabled until the job ends.
- Whether partial restarts are wanted at all, and how they treat fill lines
  and passes, is open question 7.

Tier: **Match** (the four buttons and their order, legend switch at the
left), **Echo** (Run from here, Save image…), **Own** (labels, no default
button, Stop playback, confirmation and every safety rule).
Milestone: Now (Close), later (rest; Run from here after M6).

---

## Keyboard and mouse in the preview

### Reference

- The wheel zooms, a trackpad two-finger drag zooms, and the + and − keys
  zoom; the middle button drags the view, and holding Space while dragging
  pans [R1]. On the main canvas the same wheel, middle-button and Space
  gestures apply [R5].

### LaserCraft

The preview uses the canvas's gestures (`canvas.md` "Zoom and pan";
`docs/spec/keymap.md` "Mouse and modifiers") so nothing new has to be
learned:

| Input (pointer over the drawing area, or window focused) | Effect | Today |
|---|---|---|
| Mouse wheel | zoom about the pointer | yes |
| Trackpad pinch | zoom about the pointer | via egui zoom delta; check |
| Middle-button drag | pan | no (to add) |
| Space held + left drag | pan (open-hand cursor, grabbing while dragging, as `canvas.md` "Cursors") | no (to add) |
| Ctrl+= / Ctrl+- (window focused) | zoom in / out about the drawing area's center | no: today these chords zoom the main canvas even with the preview focused |
| Ctrl+0 (window focused) | fit bed and job (the initial view) | no |
| Left click on the drawing | focus the window (later: jump to the clicked burn move) | no |
| Left drag on the drawing (no Space) | nothing (Own: avoids accidental pans while reaching for the slider; open question 11) | — |
| Esc (window focused) | close the window (proposed) | no |
| Alt+P | bring the window to the front | opens only |
| Left / Right, Shift+Left / Shift+Right, Home / End (slider focused) | step one move, ten moves, jump to start / end | no |

- Plain + and − are **not bound** (keymap.md binds the zoom chords with
  Ctrl only; the reference's plain keys are not adopted, consistent with
  the canvas).
- Zoom limits are the canvas's (`docs/spec/canvas.md`).
- Zooming and panning never change the slider, and the slider never
  changes the view (Own); "Show whole job" changes the slider only.
- No right-click menu in the preview (Own; open question 11 asks whether
  one with "Fit", "Show whole job" and "Save image…" is wanted).
- Shortcuts are ignored while the slider's value field has text focus
  (keymap.md).
- While the preview window has focus, chords that act on the project
  (Ctrl+Z, Delete, nudges) still act on the project as usual and re-plan
  the preview (Own; the preview has no selection of its own). The one
  exception is the slider: while it has keyboard focus, Left, Right,
  Shift+Left and Shift+Right step moves instead of nudging (see "Time
  row").
- Safety chords for Pause and Stop (M6, `machine-panels.md`) work whatever
  has focus, including the preview.

Tier: **Match** (wheel, middle-button and Space gestures), **Own** (focus
rules for the zoom chords, slider keys, no plain +/−).
Milestone: Now (wheel; the rest with the restyle), later (click to jump).

---

## Dark theme

### Reference

- The docs show no dark preview theme and say nothing about one
  (unknown) [R1].

### LaserCraft (own design, decision 13)

| Element | Light theme | Dark theme |
|---|---|---|
| Window body, rows, bands, buttons, switches | the light chrome tokens | the dark versions of the same tokens (`visual-language.md`) |
| Drawing surround | `canvas` (neutral grey) | `canvas` (dark neutral grey) |
| Bed sheet | `bed` near-white | **`bed` near-white (unchanged)** |
| Grid, bed edge, origin marker | as on the canvas | as on the canvas in the dark theme (`canvas.md` "Dark theme"), including the light halo outside `bed-edge` |
| Burn, unlit and travel moves on the bed | as defined | unchanged (they lie on the light sheet) |
| Moves off the bed (on the dark surround) | as defined | the same tokens with a thin `bed`-colored halo, so dark layer colors and grey travel stay visible |
| Simulated head, out-of-bounds marker | `preview-head`, `danger` with `field` rings | the same marks with `bed`-colored rings (as for off-bed halos), since the dark `field` would vanish on the dark surround |
| Legend, busy and empty chips | `field` chip, `border`, `text` | the dark `field`, `border`, `text`; the layer-color sample keeps its colors |
| Invert colors | dark sheet, light burns | identical: Invert colors is a property of the picture, not of the theme |

- Because the bed stays light, layer colors (chosen against a light sheet)
  and black layer 0 look the same in both themes (`docs/spec/canvas.md`).
- Contrast targets: text in rows and bands meets WCAG 2.2 1.4.3 (at least
  4.5:1) in both themes; the move styles and marks on the bed meet 1.4.11
  (at least 3:1 against `bed`) except where a layer color itself is low
  contrast (see "How moves are drawn").

Tier: **Own**.
Milestone: later (themes; today the window follows the OS theme while the
preview bed is always light).

---

## egui implementation notes

- Window: `egui::Window::new("Job preview")` with `.collapsible(false)`,
  `.resizable(true)`, a default size relative to the canvas region,
  `.min_size(...)` from the layout minimum, and `.constrain_to(...)` the
  area below the toolbar band and above the status bar. The id stays the
  title so egui remembers its rectangle in memory for the session.
- Drawing area: `ui.allocate_painter(available, Sense::click_and_drag())`
  so middle-drag and Space+drag can pan, and a painter clipped to the
  response rect. The controls rows are laid out *below* it by reserving
  their height first (a bottom-up layout or `TopBottomPanel`s inside the
  window), so the drawing area takes whatever remains.
- Strokes, dashes and dots come from the stroke ladder, scaled by
  `ctx.pixels_per_point()` so they stay crisp at fractional scaling
  (decision 12). Dotted unlit lines can use `Shape::dotted_line`; dashed
  travel `Shape::dashed_line`.
- Many moves: build the shapes once per plan and per view change, and draw
  only up to the slider's move index; very large jobs may need
  level-of-detail later (drawing scan lines as filled bands when they are
  closer together than a hairline). Planning stays outside the UI
  (`ARCHITECTURE.md`).
- The view type is the canvas's `View` (fit, zoom about a point, pan), so
  behaviour matches the canvas exactly.
- Switch widget: the shared pill switch from `visual-language.md`, not
  `ui.checkbox`.
- No `unsafe`, no new dependency except the PNG encoder for Save image…
  (open question 6).

---

## Deliberately omitted

- The reference's built-in help links, product branding in the window
  title and version text (decision 1 and `naming.md`).
- Galvo-specific preview behaviour and any DSP "send" path from the preview
  (decision 1).
- A hold-a-key-to-fire or test-fire control in the preview (decision 7):
  the preview never fires the laser except through Run from here, which
  always asks first.
- The reference's fixed hint line about line colors (older layout) [R2]:
  the legend replaces it.

---

## Current state → target

From `current-ui.md` 1.3, 1.8 and the color table, and the code in
`crates/lasercraft-ui/src/preview.rs` and `crates/lasercraft-app/src/preview.rs`.

| Today | Problem | Target |
|---|---|---|
| Text lines, then the bounds line, then warnings, then the slider row, then the drawing (`preview.rs:134-163`) | the drawing, the main thing, is pushed to the bottom; the order is the reverse of the reference's | drawing area on top, then Time row, summary, check and notes, playback row, button row ("Layout inside the window") |
| `egui::Window` with a fixed default size in pixels and egui defaults (`preview.rs:124-131`): collapsible, can be dragged anywhere | can collapse to a bare title; can cover the toolbar band and later the job bar | not collapsible; size relative to the canvas region; constrained below the toolbar band ("The window") |
| Bed filled `BED` light grey, outline `BED_EDGE` medium green, a heavier line than the moves (`preview.rs:11-12`, `preview.rs:188-194`) | green also means "touching box" and "drawing" on the canvas and "connected" in decision 6; the canvas bed edge is grey | `bed` sheet, `bed-edge` outline (*normal*), the canvas grid and machine origin marker, `canvas` surround |
| Unlit moves bright orange, thin dashed (`preview.rs:13`, `preview.rs:210-216`) | orange means "selected" on the canvas | `move-unlit`, *hairline*, *dotted* |
| Travel moves strong red, thin dashed (`preview.rs:14`, `preview.rs:218-224`) | red means danger; travel is safe | `move-travel`, *hairline*, *dashed* |
| Burn moves in layer color, a fixed thin weight (`preview.rs:207-209`) | weight not on the ladder | `move-burn`, *normal* |
| Head: royal blue dot shown only after the slider moved (`preview.rs:15`, `preview.rs:229-233`) | invisible at first; no ring for contrast | `preview-head` dot with a `field` ring, always shown ("Simulated head") |
| "Cannot run: …" in deep red text (`preview.rs:16`, `preview.rs:139-142`) | color-only signal | `danger-soft` band with a `danger` mark; out-of-bounds point and edge marked on the drawing (needs the point in `Preview`) |
| "Fits the work area." plain label (`preview.rs:143-145`) | — | `success` check mark plus the same text |
| Warnings as weak text (`preview.rs:147-149`) | easy to miss | `warning-soft` bands, folding after three |
| Slider labelled "time" with suffix " s" (`preview.rs:152-156`) | label after the slider; unit typed in the field | label "Time (s)" before it; no suffix (`naming.md`) |
| Button "Whole job" (`preview.rs:159-161`) | a noun on a button | "Show whole job", disabled at the end |
| Checkbox "Show travel" (`preview.rs:162`) | checkbox instead of LaserCraft's switch | pill switch in the playback row |
| Summary "Burn …, laser-off run-on …, travel …" with `duration` times "4 min 05 s" (`app/preview.rs:41-69`, `lasercraft-plan` `dump.rs:28-41`) | "laser-off run-on" is not our word; time format differs from `naming.md` | "unlit"; times h:mm:ss formatted by the app |
| Wheel zoom only; no pan; any resize refits the view (`preview.rs:172-185`) | cannot move around a zoomed job; resizing loses the zoom | middle-drag and Space+drag pan; Ctrl+=/Ctrl+-/Ctrl+0 when focused; resize keeps the zoom |
| No Close button, no read-out, no legend, no Play | — | Close now; read-out now; the rest later as marked |
| Drawing area with a fixed pixel minimum (`preview.rs:167`) | numbers in code are fine, but the minimum must follow the layout rule | minimum from "Size and position" |
| Menu item "Job Preview…" (`shell.rs:492-499`) | Title Case | "Job preview…" (`naming.md`, `menus.md`) |

Labels the tests look for stay unchanged: "Job preview", "Cannot run:" and
"Warning:" (`crates/lasercraft-ui/tests/shell.rs:248-295`).

---

## Proposed changes to other specs

- **Proposed change to preview.md** (Drawing): unlit moves drawn in
  `move-unlit` as thin dotted grey lines and travel in `move-travel` as thin
  dashed grey lines instead of orange and red; the work area outline in
  `bed-edge` with the canvas's grid and origin marker; the head shown at
  all times; the drawing area above the controls.
- **Proposed change to preview.md** (Text): times as h:mm:ss; "unlit"
  instead of "laser-off run-on"; the bounds error shown as a band and its
  furthest-out point and edge handed to the UI and marked on the drawing;
  warnings as bands.
- **Proposed change to preview.md** (controls): "Time (s)", "Show whole
  job", "Close"; pan with the middle button or Space and drag; zoom chords
  act on the preview while it has focus; the view keeps its zoom on resize;
  later Play, Playback speed, Lighter for low power (needs each burn move's
  power in `Preview`), Invert colors, Show legend, Save image….
- **Proposed change to preview.md** (M6): the preview shows the same job as
  Start (Selected shapes only, Place job at); the constant-power note; Run
  from here with the rules in "Button row".
- **Proposed change to safety.md**: add a rule for partial restarts (Run
  from here): only by an explicit button click followed by a confirmation;
  never from a chord; restart at the beginning of a move; the travel to it
  is laser-off; the remaining job and that travel pass the bounds check
  before anything is sent; Pause and Stop as for any job.
- **Proposed change to keymap.md**: while the preview window has focus,
  Ctrl+=, Ctrl+- and Ctrl+0 act on the preview's view, Esc closes the
  preview, and the slider keys (Left, Right, Shift+Left, Shift+Right, Home,
  End) step through moves. This is a focus-scoped override: while the
  preview's slider has keyboard focus, Left, Right, Shift+Left and
  Shift+Right step moves and do not run their bound `shape.move` nudges;
  without that focus they nudge as today. Middle-button and Space+drag pan
  the preview as on the canvas.
- **Proposed change to ui-shell.md**: "View → Job Preview…" becomes "Job
  preview…" (sentence case, `naming.md`); the window opens non-modal and
  constrained below the toolbar band.
- **Proposed change to cli.md**: `preview --svg` draws unlit moves dotted
  light grey and travel dashed mid grey (not orange and red), so the CLI's
  picture and the window mean the same thing.
- **Proposed change to device.md** (M6): a `job.start_from` command for Run
  from here, not undoable, refused unless connected and Ready.

---

## Open questions

1. **One color for burns.** `docs/spec/preview.md` draws burns in layer
   colors; the reference uses one dark color. Keep layer colors only, or
   add a "Same color for all burns" switch (later)?
2. **Separate OS window.** Should the preview move to its own OS window
   (multi-viewport) later, given Wayland may ignore its position, or stay
   inside the main window?
3. **Play button wording.** "Play" / "Stop playback" avoids the machine's
   "Pause"; is a pause-and-continue behaviour wanted instead of stop?
4. **Esc closes the preview.** Esc is bound to "leave the current tool"
   (`default.keymap`); is a window-local Esc acceptable?
5. **Legend default.** On (helps switchers learn the new line styles) or
   off (the reference's apparent default)?
6. **Save image format and dependency.** PNG only? Which size choices?
   Approve the encoder dependency (or wait for M7's `image` crate)?
7. **Run from here.** Wanted at all? Restart at the start of the move
   under the slider, of the current fill line, or of the current shape?
   How are passes counted? Must the machine be re-homed first after an
   alarm?
8. **Inverted tokens.** Add `preview-sheet-inverted` and
   `move-burn-inverted` to `visual-language.md` instead of reusing `text`
   and `bed`?
9. **Head at the end.** Show the simulated head at the last point when the
   slider is at the end (this file, as the reference), or hide it there
   (today)?
10. **Grid in the preview.** `canvas.md` shares the grid with the preview;
    the reference shows none. Keep it, or make it faint-only or off?
11. **Left drag and right-click.** Should a plain left drag pan the preview
    (there is nothing to select), and is a right-click menu wanted?
12. **Click to jump.** Clicking a burn move jumps the slider to it (later):
    wanted, and should hovering show the move's layer and time?
13. **Per-layer breakdown.** Add the per-layer table under the summary?
14. **Time format.** `naming.md` asks for h:mm:ss in summaries; the CLI
    keeps "4 min 05 s". Align both, or keep them different?
15. **Slider fill.** Decision 3 limits `accent` to a fixed list, so the
    time slider's filled groove uses `text-muted`. Add "slider fill" to
    decision 3 and `visual-language.md` as an `accent` use (closer to the
    reference's saturated fill), or keep it neutral?

---

## Sources

All pages read 2026-10-21, through the research notes for this guide.

1. [R1] https://docs.lightburnsoftware.com/2.1/Reference/Preview/ — the
   ways to open the preview (window menu, toolbar button, Alt/Option+P,
   workspace right-click), the time slider and how it reveals the job, zoom
   by wheel, trackpad or + and − and pan by middle button or Space, the
   work-area edge drawn in pale green, the statistics line (on and off
   distances with times, total) and that simulation values in device
   settings drive it, the four preview switches and their meanings, the
   playback speed range, the legend in the top-left corner with two entries,
   the four bottom buttons (start partway for interrupted jobs, save image,
   play, close), the laser-on/laser-off colors, the window layout and
   slider look in its screenshots, the head marker.
2. [R2] https://docs.lightburnsoftware.com/2.1/GetStarted/PreviewBeginner/
   — opening routes, the still picture of the finished result on first
   opening, black line work and solid or scan-line fills, red travel lines
   and their switch, play at roughly real time with a speed slider, the
   total time at the lower right, the older layout with a fixed hint line
   instead of the legend switch, focus outline on a button.
3. [R3] https://docs.lightburnsoftware.com/2.1/Troubleshooting/PreviewWindow/
   — screenshots of preview windows: the head marker at the start and end,
   overscan merging into pinkish bands, double fills darker, scan lines
   resolving when zoomed, grey for low power when shading is on, times past
   a hundred hours, the work-area edge partly in view, switch defaults.
4. [R4] https://docs.lightburnsoftware.com/2.1/Reference/DeviceSettings/AdditionalSettings/
   — simulation values (speeds, accelerations, scale factors) only tune
   the preview's simulation and time estimate and do not change how the
   job runs.
5. [R5] https://docs.lightburnsoftware.com/2.1/Reference/ZoomPan/ — the
   main canvas's zoom and pan gestures (wheel about the pointer, middle
   button, Space and drag), which the preview shares.

Existing specs and code used: `docs/spec/preview.md` (wins: three move
kinds, layer colors for burns, the slider and the text content),
`docs/spec/planning.md` (move kinds, bounds, estimate, warnings),
`docs/spec/safety.md` (rules 1, 4–7, 9 for Run from here and the job bar),
`docs/spec/keymap.md` and `crates/lasercraft-ui/keymap/default.keymap`
(Alt+P, Ctrl+=, Ctrl+-, Ctrl+0, Esc, gestures), `docs/spec/canvas.md`,
`docs/spec/ui-shell.md`, `docs/spec/cli.md`, `docs/spec/device.md`, and the
other style files named at the top; the current code as audited in
`current-ui.md` (`crates/lasercraft-ui/src/preview.rs`,
`crates/lasercraft-app/src/preview.rs`, `crates/lasercraft-ui/tests/shell.rs`).

Open standards: WCAG 2.2 (https://www.w3.org/TR/WCAG22/) success criteria
1.4.1 Use of Color, 1.4.3 Contrast (Minimum) and 1.4.11 Non-text Contrast,
for the contrast targets and the rule that color is never the only signal.

Own design: every label, tooltip and message; the in-app non-modal window,
its size rules and the constraint below the toolbar band; the exact row
contents and alignment; the three move styles (layer-colored solid burns,
light grey dotted unlit moves, mid grey dashed travel, never red); the
`accent` dot for the simulated head with its ring; the out-of-bounds
marker; the check and notes bands; the read-out with the move number; the
playback speed snapping; colored shading by power and the inverted
rendering; the legend's chip and line samples; Stop playback; the Save
image… rules; every safety rule of Run from here; the slider keys and the
focus rules for zoom chords; the dark theme table.
