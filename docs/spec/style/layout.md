# Spec: style — main window layout

Milestones: M0–M5 (the regions that exist today: menu bar, tool column,
canvas, right dock with Layers, palette strip, command line, status bar),
M6 (Machine, Jog and Console panels, the job bar, the lower dock group),
M8 (Materials panel), M9 (Job setups toolbar, Camera panel, Active setups
field), M10 (packaging and platform checks); toolbars, docking chrome,
Window menu and Simple layout are later.

This file fixes where every region of LaserCraft's main window sits, how
big each one is relative to the others, what is shown by default, and how
panels and toolbars are shown, hidden, docked, resized and reset. It is the
map the other style files hang from. Names come from `naming.md`; colors,
type and sizes are only the tokens defined in `visual-language.md`; icons
are in `icons.md`. The contents of each region live in their own files:
`menus.md` (menu bar and the full Window menu), `toolbars.md` (every
toolbar's buttons), `property-bar.md` (Position and size, Text style),
`layers-panel.md` (Layers panel and Palette strip contents),
`machine-panels.md` (Machine, Jog, Console, Macros, the job bar's buttons in
detail), `canvas.md` (rulers, bed, grid, markers), `dialogs.md` (dialogs and
their relation to the job bar), `job-preview.md` (the preview window) and
`later-windows.md` (Materials, Camera, Properties, Text data, Saved artwork).
Each element carries a tier: **Match** (the reference's arrangement and
behaviour, copied in function only), **Echo** (the same idea in our own
look) or **Own** (wholly LaserCraft's design); the style guide's index,
`README.md`, restates these once it exists.

## Overview and anatomy

### Reference

- The reference's full-mode main window, top to bottom: the system title
  bar, a text menu bar, two full-width toolbar rows (the lower one about
  twice as tall because its fields sit in two lines), then a middle band
  with a thin vertical tool strip on the left, the drawing area in the
  centre and a tall docked column on the right, then a single swatch row
  across the whole width, then a thin status row at the very bottom
  [R1][R2].
- The right column is roughly a quarter of the window wide. It holds two
  tabbed groups stacked vertically; the upper one takes a little more than
  half the height. Both are resizable [R1][R2].
- The left side carries only the two vertical toolbars; no window is docked
  there by default [R3].
- The drawing area is by far the largest region, about three quarters of
  the width [R1].
- The arrangement changes with the device type and with the reference's
  reduced mode [R1][R2][R6].

### LaserCraft

The default arrangement, with LaserCraft names (binding; it is the same
arrangement the other style files assume):

```
+----------------------------------------------------------------------------+
| • Coaster sign — Shop diode — LaserCraft                (OS title bar)     |
+----------------------------------------------------------------------------+
| File  Edit  Tools  Arrange  Machine  View  Window  Help (menu bar)         |
+----------------------------------------------------------------------------+
| [Standard ...................][Arrange ............]      [Job bar (M6)]   | row 1
+----------------------------------------------------------------------------+
| [Position and size ][Text style        ][Job setups]      [Slide and nest] | row 2
| [  (two lines)     ][  (two lines)     ][  (M9)    ]      [ (later, off) ] |
+------+------------------------------------------------+--------------------+
| Draw-|  ruler                                         | Layers        [x]  |
| ing  |                                                |  layer list        |
| tools|                                                |  layer editor      |
|      |                                                |                    |
|      |              Canvas                            |Layers|Jog|Console..|
|------|        (bed sheet, grid, shapes)               |=====  ....  =======| splitter
| Shape|                                                | Machine       [x]  |
| opera|                                                |  machine state     |
| tions|                                                |  job buttons       |
|      |                                                |Machine|Materials   |
+------+------------------------------------------------+--------------------+
| 00 01 02 03 ... 28 29  [g]1 [g]2                          (Palette)        |
| > command line .................................................. [log]    |
| Move Scale Rotate Skew | x y mm | selection / hint | Messages ... | setups | Work area | Project |
+----------------------------------------------------------------------------+
```

- Menu titles and their order are `menus.md`'s; the diagram only shows that
  they sit in one text line.
- "Node tools" (not drawn) appears as a second narrow column between
  Drawing tools and the canvas while Edit nodes is active.
- Until M6 there is no job bar, no lower dock group and no Machine group:
  the Layers panel fills the whole right dock.
- Things marked later or with a milestone are not drawn until they exist;
  the region they would take is simply absent (no empty placeholders).

Tier: **Match** (regions and their default places), with the look Own.

## Bands from top to bottom

Every band spans the full window width except the middle band, which is
split into columns. Heights are relative to the `row` token.

| # | Band | Height | Contents | Can hide? | Milestone |
|---|---|---|---|---|---|
| 1 | OS title bar | the OS's | window title (see "Window title") | no | now |
| 2 | Menu bar | a little less than one `row` | menu titles, `type-body` | no | now |
| 3 | Toolbar row 1 | one `row` plus a thin margin | Standard, Arrange, (Align and distribute), job bar at the right end | the toolbars yes; the row stays while connected | toolbars later; job bar M6 |
| 4 | Toolbar row 2 | two `row`s plus a thin margin | Position and size, Text style, Job setups, Slide and nest | yes, the whole row collapses when all its toolbars are hidden | later; Job setups M9 |
| 5 | Middle band | everything left over | tool column, (Node tools), canvas, right dock | side parts with F12 | now |
| 6 | Palette strip | one `row` | 30 swatches, later two guide swatches | yes (Window → Toolbars) | now |
| 7 | Command line | one `row` collapsed; the log adds height upward when opened | input line, log | yes (Window → Panels) | now |
| 8 | Status bar | a little less than one `row`, `type-small` | read-outs and messages | no | now |

Look of the bands (all Own, tokens from `visual-language.md`):

- Bands 2, 3, 4, 6 and 8 sit on `window`. A thin `border` line separates the
  menu bar from row 1, row 2 from the middle band, and the middle band from
  the palette strip. Row 1 and row 2 are separated by a fainter line, so the
  two rows read as one toolbar band.
- The command line's input field is `field`; its strip is `panel`.
- The status bar has no frame of its own; only its fields are separated by
  thin `border` lines.

Tier: **Match** (band order and relative heights), **Own** (look).

## Menu bar

### Reference

- A single thin line of text menu titles directly under the title bar, no
  icons in the bar itself [R10]. On macOS the reference moves its menus to
  the system menu bar [R10][R9].

### LaserCraft

- Always inside the window, on every platform, including macOS
  (decision 9). It is the first top panel.
- Contents: `menus.md`. Height: one text line with a small margin, a little
  less than `row`. Titles in `type-body`, `text`; the open title gets
  `accent-soft` behind it.
- The menu bar never hides, never overflows (titles are short; when the
  window is too narrow for all titles the window's minimum width prevents
  it, see "Narrow and short windows").

Tier: **Match** (place), **Own** (look; the in-window menu on macOS is a
deliberate difference).

## Toolbar band (rows 1 and 2)

### Reference

- Row 1 holds the file/clipboard/view toolbar at the far left with the
  arrange toolbar directly to its right; together they fill roughly the
  left three quarters of the row, and the rest of the row is empty [R19][R20][R32].
- Row 2 holds the numeric position-and-size fields at the left, then the
  text options, then a short strip of large special-mode buttons, then the
  object-docking toolbar at the far right [R1][R23][R27].
- Row 2 is about twice as tall as row 1 because its fields sit in two (or
  three) lines [R1][R2].
- When a row is too short for its toolbars, the clipped toolbar ends in a
  double chevron that opens the hidden buttons [R1][R3][R9].

### LaserCraft

Row 1, left to right:

| # | Item | Default | Milestone |
|---|---|---|---|
| 1 | Standard toolbar | shown | later |
| 2 | Arrange toolbar | shown | later |
| 3 | Align and distribute toolbar | hidden; when shown it sits here, after Arrange | later |
| — | flexible empty space | | |
| 4 | Job bar, right-aligned | shown while connected | M6 |

Row 2, left to right:

| # | Item | Default | Milestone |
|---|---|---|---|
| 1 | Position and size toolbar | shown | later |
| 2 | Text style toolbar | shown | later |
| 3 | Job setups toolbar | shown once it exists | M9 |
| — | flexible empty space | | |
| 4 | Slide and nest toolbar, right-aligned | hidden | later |

- Row 1 is one `row` tall; its buttons are `icon` squares (`toolbars.md`).
- Row 2 is two `row`s tall: Position and size and Text style lay their
  fields out in two lines (`property-bar.md`); Job setups uses `icon-big`
  buttons that fill both lines.
- Toolbars in a row are packed from the left with `gap-wide` between them.
  A hidden toolbar leaves no gap; the ones after it move left.
- Right-aligned items (job bar, Slide and nest) keep the right end. When
  the row is too short, toolbars on the left overflow first (see "Toolbar
  model"); the job bar never does.
- If every toolbar in row 2 is hidden, row 2 disappears and the middle band
  grows. Row 1 disappears only when all its toolbars are hidden and the
  machine is not connected; while connected it stays to hold the job bar.
- Until the toolbars exist (today) there is no toolbar band at all; the
  menu bar sits directly above the middle band. From M6 row 1 exists at
  least for the job bar.

Tier: **Match** (two rows, their order and relative heights), **Own**
(names, look, the job bar).

## Tool column

### Reference

- A narrow vertical strip along the left edge, about one and a half icons
  wide, from under the toolbar rows to just above the swatch row. Its upper
  part holds the creation tools, its lower part the modifier tools, each
  starting with a dotted grip [R1][R2][R21][R22].
- While node editing is on, a second narrow strip of node actions appears
  directly to the right of the creation tools, between the strip and the
  ruler, with the same background and width [R24].
- Both toolbars can also be laid out horizontally when moved out of the
  column [R21][R22][R3].

### LaserCraft

- Place: left edge of the middle band, from row 2's lower edge to the
  palette strip's upper edge. It is the only thing docked on the left by
  default.
- Width: one `icon` plus `gap-tight` on each side; the column is never
  resizable (it is as wide as its buttons).
- Contents top to bottom: Drawing tools, a thin `border` separator (a grip
  in stage b), Shape operations. Their buttons are in `toolbars.md`. If the
  column is too short, Shape operations overflows into a More button at its
  bottom first; Drawing tools overflows only after Shape operations is
  fully folded away.
- Node tools: while the Edit nodes tool is active, a second column of the
  same width appears immediately right of the tool column (between it and
  the canvas's left ruler). It disappears when another tool is picked. The
  canvas narrows by that width; what happens to its view then is
  `canvas.md`'s to decide (see "Canvas region").
- Background `panel`; a thin `border` line separates the column from the
  canvas ruler.
- Today the column holds the six text tool buttons of the `tools` panel
  (current-ui.md 1.5).

Tier: **Match** (place, order, the contextual node column), **Own** (look).

## Canvas region

### Reference

- The drawing area fills all space between the tool strip, the docked
  column, the toolbar rows and the swatch row; rulers with numbers run
  along its edges [R1][R2].

### LaserCraft

- Place: the `CentralPanel`, everything the other panels leave.
- It is the only region that grows and shrinks freely; every other region
  has a fixed or user-set size.
- Its contents (rulers, bed, grid, markers, overlays) are `canvas.md`'s. The
  region has no frame or title strip; the ruler bands sit inside it along
  its top and left edges (`canvas.md`).
- When panels appear, hide or change size, the region simply gets the new
  space; whether the view keeps the bed point under its centre, keeps its
  top-left corner or re-fits is `canvas.md`'s decision, not this file's
  (Open questions). Today's F12 test only checks that re-fitting after F12
  gives a larger scale (`crates/lasercraft-ui/tests/shell.rs`,
  `f12_hides_the_side_panels_and_the_bed_fits_larger`).
- Floating panels and toolbars (stage b) float above the canvas and never
  change its size.

Tier: **Match**.

## Right dock

### Reference

- A tall column on the right holding two tabbed groups. The upper group's
  tabs, in a strip along its bottom edge, are the layer list, the jog
  window, the console, the macros and the shape properties, with the layer
  list in front [R1][R2][R15][R16][R17]. Console and macros appear only for
  G-code devices [R1][R16][R17].
- The lower group holds the job-control window and the material library,
  with job control in front; its tabs are also at the bottom
  [R1][R2][R14][R31]. For G-code devices that window's top row has three
  big job buttons [R30].
- The upper group takes a bit more than half of the column height [R1].
- In a narrow column the tab captions are cut off with an ellipsis [R3].

### LaserCraft

- Place: right edge of the middle band, from row 2's lower edge to the
  palette strip's upper edge.
- Width: about a quarter of the window width on first start and after
  Reset layout; the user resizes it by dragging its left edge. Its minimum
  width is the widest minimum of every panel it holds, hidden tabs included
  (see "Minimum sizes").
- Two groups, stacked:

Upper group (tabs left to right, Layers in front):

| # | Tab | Default | Milestone |
|---|---|---|---|
| 1 | Layers | shown, in front | now |
| 2 | Jog | shown | M6 |
| 3 | Console | shown | M6 |
| 4 | Macros | shown | later |
| 5 | Properties | shown | later |
| 6 | Text data | hidden | later |
| 7 | Camera | hidden until a camera is set up | M9 |

Lower group (tabs left to right, Machine in front):

| # | Tab | Default | Milestone |
|---|---|---|---|
| 1 | Machine | shown, in front | M6 |
| 2 | Materials | shown | M8 |

- Height split: the upper group takes a bit more than half of the dock's
  height on first start and after Reset layout; the splitter between the
  groups moves it.
- Before M6 only the upper group exists and holds only Layers, so Layers
  fills the whole dock. From M6 the lower group appears below it with
  Machine; Materials joins at M8.
- A group with a single visible panel shows its title strip but no tab
  strip. A group whose panels are all hidden disappears and the other group
  takes the full height. When both are empty, the right dock disappears and
  the canvas takes its width.
- Unlike the reference, Console and Macros are always offered, because every
  LaserCraft device is GRBL (decision 1).
- Saved artwork (later) is not in any group by default; when first shown it
  joins the upper group as the last tab (own design; the reference shows its
  equivalent docked on the left in its examples [R3][R4], which we do not
  copy, so that the left stays tools only).
- Each panel's contents: Layers in `layers-panel.md`; Machine, Jog,
  Console, Macros in `machine-panels.md`; the rest in `later-windows.md`.

Tier: **Match** (two groups, tab order, front tabs, bottom tab strips,
proportions), **Own** (names, look, single-panel groups without tabs).

## Palette strip

### Reference

- One full-width toolbar row just above the status row, about one button
  tall, with the numbered swatches packed to the left and empty space to
  their right; it starts with a grip and shows a chevron at its right end
  when the window is narrow [R2][R11].

### LaserCraft

- Place: directly below the middle band, full width, above the command
  line.
- Height: one `row`. Swatches packed from the left with `gap-tight`, in one
  line only (they never wrap to a second line). The look of the swatches
  and their numbers is `layers-panel.md`'s.
- When the strip is too narrow for all swatches, the last ones fold into a
  More button at its right end (see "Toolbar model").
- Shown or hidden from Window → Toolbars → Palette. F12 does not hide it.

Tier: **Match** (place, one row, packed left), **Echo** (swatch strip look,
in `layers-panel.md`).

## Command line

### Reference

- There is no reference equivalent; the reference's console is a dock panel
  that talks to the controller [R16].

### LaserCraft

- Place: between the palette strip and the status bar, full width.
- Default state: collapsed to its one-line input field. The field fills the
  width except for a small toggle button at its right end that opens the
  log. Field hint: today's hint (current-ui.md 1.9), in `text-muted`.
- Open state: the log appears **above** the input line (it grows upward,
  taking height from the middle band), `type-mono`, newest line at the
  bottom, sticking to the bottom. The boundary between the log and the
  palette strip is a splitter with a grip; dragging it sets the log height.
  The toggle closes it again. The log keeps its last height for the session.
- A failed command shows its error in the status bar's Messages field
  (on `danger-soft`) even while the log is closed.
- Shown or hidden from Window → Panels → Command line. No chord is bound
  to showing it or to opening its log.
- It is not the GRBL Console panel (M6), which is a dock tab in the upper
  group.

Tier: **Own**. Collapsed-by-default is a proposed change to ui-shell.md
(below).

## Status bar

### Reference

- The lowest strip, full width, about one text line tall, on the window's
  light chrome without a frame of its own [R11].
- Left to right: four small switches that turn kinds of handle editing on
  or off for the whole project, then the pointer coordinates, then a tool
  status field (selection corners and object count, or line deltas and
  length while drawing, or the angle while rotating), then a notification
  area, then an indicator of special job modes that are on [R11][R12].
- The pointer coordinates stop updating when the pointer leaves the drawing
  area [R11].
- The notification area is a tinted band: hints on a pale cyan fill, device
  messages and the mode indicator on pale green [R11].
- The reduced mode hides the four switches [R6][R2].

### LaserCraft

Fields, left to right (binding order):

| # | Field | Shows | When visible | Look | Milestone |
|---|---|---|---|---|---|
| 1 | Handle switches: Move, Scale, Rotate, Skew | four small pill switches, each followed by its word; turning one off disables that kind of canvas handle for the project | always (hidden in Simple layout) | switches per `visual-language.md`, labels `type-small` | later |
| 2 | Pointer position | pointer X and Y in mm in bed coordinates from the machine-origin corner (`canvas.md`, `property-bar.md`); today it shows document coordinates (current-ui.md 1.10) | always once the pointer has been over the canvas; when the pointer leaves the canvas the last value stays, drawn in `text-muted` | `type-mono` at `type-small` | now |
| 3 | Selection size or Tool hint | with the Select tool: count and size of the selection (today's text); while a tool is mid-action: what it is measuring (segment deltas and length while drawing a line, the angle while rotating, the size while dragging a shape); with an idle drawing tool: what the next click does | whenever there is something to say; empty otherwise | `text` | now (selection); later (hints) |
| 4 | Messages | the last notice: file saved, device found, a refused job, a failed command, the Linux serial-permission summary (the full help is in the Machine panel) | stretches to fill the free width; empty when there is nothing to report | one line on a status band (below) | later |
| 5 | Active setups | the job setups that are switched on (rotary, print registration) | only while one is on | on `info-soft` | M9 |
| 6 | Work area | size and origin corner in today's form | always | `text-muted` | now |
| 7 | Project | name, unsaved dot, path or "not saved", at the right end | always | `text` | now |

Status bands (Messages field):

| Kind | Band token | Text | Clears |
|---|---|---|---|
| Hint (what a key or tool does) | `info-soft` | `text` | when the tool or hint changes |
| Success (saved, connected, device found) | `success-soft` | `text` | at the next action, or after a short while |
| Warning (job warnings, stalled replies) | `warning-soft` | `text` | when the cause goes away |
| Error (refused job, failed command, alarm summary) | `danger-soft` | `text` | at the next successful action of the same kind |

- A band hugs its text with `gap-tight` padding; the rest of the Messages
  field stays plain `window`. Long text is cut with "…" and the full text is
  the field's tooltip.
- The status bar is read-only except the handle switches. Clicking a
  Messages band that carries an error opens the command line's log
  (own design).
- An alarm is never only a status message: it also shows the Alarm banner
  in the Machine panel and the job bar's state (safety rule 10;
  `machine-panels.md`).
- Field separators: thin vertical `border` lines.
- Shrinking: see "Narrow and short windows".

Tier: **Match** (strip, field order idea, pointer freeze), **Echo** (status
bands), **Own** (field contents, Work area and Project fields, colors).
The new order is a proposed change to ui-shell.md (today: project, work
area, pointer, selection).

## Job bar

### Reference

- In the docs read, Pause, Stop and Start appear in the job-control
  window, a dock panel at the lower right [R14][R18]; like every window
  there it can be hidden behind another tab or closed [R3]. Its state line
  reads disconnected, ready or busy, with a progress bar while busy [R14].
- Whether the reference shows these buttons anywhere else is not known
  from the notes (Open questions).

### LaserCraft

The job bar exists for safety rule 7 (Stop and Pause always visible and
enabled while connected) and decision 7. It is our own design.

- Place: the right end of toolbar row 1, right-aligned, one `row` tall.
  It belongs to no toolbar: it has no grip, no More button, no entry in
  Window → Toolbars, and no close button.
- When shown: from the moment Connect is pressed ("Connecting…") until the
  connection is closed or lost. Hidden while not connected (hardware test
  H2 in `docs/hardware-testing.md`).
- Contents, left to right:

| # | Item | Look | Behaviour |
|---|---|---|---|
| 1 | Machine state word (Not connected never shows here; Connecting…, Ready, Running, Framing, Paused, Jogging, Homing, Alarm, Door open, Checking, Sleeping, No response) | `type-body`; a small round state dot before it: `success` for Ready, `accent` for Running, Framing, Jogging and Homing, `warning` for Paused, Door open and No response, `danger` for Alarm, `text-muted` otherwise | read-only; tooltip gives the GRBL state and, for Alarm, its meaning |
| 2 | Sent percentage (only while a job streams) | `type-small`, `text-muted`, after the state word | read-only; says "sent", not "done" (`naming.md`, Job progress) |
| 3 | Pause / Resume | compact button, `icon-small` plus label, one `row` tall | feed hold; the same button resumes; Pause key |
| 4 | Stop | compact button, `danger` fill with light text, slightly wider than Pause | stop streaming and soft reset; Ctrl+Break |

- Pause and Stop are enabled for the whole time the bar is shown, whatever
  the machine state (safety rule 7). Keyboard focus never rests on them by
  default.
- Constant-power job (`M3`, safety rule 9): Pause reads "Pause (motion
  only)"; a note "The laser may stay on", in `type-small` `text` with a
  `warning` mark (`mark-warning`, `icons.md`) before it, sits immediately
  left of the Pause button so the two read together as rule 9's wording; the bar's background becomes `warning-soft`, and Stop gets
  the *heavy* `accent` ring of `visual-language.md` outside its `danger`
  fill so it is the highlighted action. `warning` is never used as text
  color here (`visual-language.md`).
- The constant-power note and the full Pause label are never shortened,
  cut to a tooltip or dropped, at any window width: while a constant-power
  job runs, nothing visible may leave out that the laser may still be on.
  What gives way instead, in order: the other toolbars in row 1 fold into
  More, the sent percentage goes, then the state word shortens to its dot
  (the word moves to the dot's tooltip). The window's minimum width
  (see "Narrow and short windows") is set so the bar with the note, the
  full Pause label and Stop always fits.
- Alarm: the bar's background becomes `danger-soft`; the buttons stay.
- The job bar repeats, in compact form, the big Pause/Resume and Stop of
  the Machine panel (`machine-panels.md`); both send the same commands.
- Never hidden or covered by:
  - F12 Side panels (the bar is in the toolbar band, not a side panel);
  - the Machine panel being hidden, closed or behind another tab;
  - any toolbar being hidden, row 1 being otherwise empty, or a narrow
    window (other toolbars fold into More first; the bar is the last thing
    in row 1 to shrink, and it only drops the percentage and the state
    word; the constant-power note always stays);
  - any dialog or window: no dialog may be modal over it while connected
    (see `dialogs.md` and "egui implementation notes");
  - Simple layout, full-screen job preview, or any later window.
- Start and Frame are deliberately not in the bar: starting needs the
  Machine panel or Alt+S, so the always-visible strip only offers actions
  that make the machine safer.

Tier: **Own**.

## Window title

### Reference

- The title shows the project file name (an untitled placeholder before the
  first save), a star when there are unsaved changes, the selected device
  profile, a reduced-mode marker when that mode is on, and the product name
  and version, separated by dashes [R1][R2][R6][R34].

### LaserCraft

- Form (decision 10, `naming.md`): `• ‹project› — ‹device profile› —
  LaserCraft`, with an em dash between parts.
  - The leading "• " appears only while there are unsaved changes (today's
    rule, current-ui.md 1.1).
  - ‹project› is the file name without extension, or "Untitled" before the
    first save.
  - "— ‹device profile›" is added once devices exist (M6 with the single
    default profile, M8 with named profiles). Not connected or connected
    makes no difference to the title.
  - With Simple layout on (later), " (simple layout)" ends the title.
  - No version number in the title (the About window shows it).
- Example: `• Coaster sign — Shop diode — LaserCraft`.
- Before any project is open the title is just "LaserCraft" (today).

Tier: **Own** (form), **Match** (it carries project, unsaved marker and
device).

## Panels and toolbars inventory

Every panel and toolbar, with its default place and visibility. "Shown"
means checked in the Window menu on first start and after Reset layout.

| Name | Kind | Default place | Default | Hidden by F12 | Milestone |
|---|---|---|---|---|---|
| Standard | toolbar | row 1, first | shown | no | later |
| Arrange | toolbar | row 1, after Standard | shown | no | later |
| Align and distribute | toolbar | row 1, after Arrange | hidden | no | later |
| Position and size | toolbar | row 2, first | shown | no | later |
| Text style | toolbar | row 2, after Position and size | shown | no | later |
| Job setups | toolbar | row 2, after Text style | shown | no | M9 |
| Slide and nest | toolbar | row 2, right end | hidden | no | later |
| Drawing tools | toolbar | tool column, top | shown | yes | now (text buttons) |
| Node tools | toolbar (contextual) | beside Drawing tools | only while Edit nodes is active | yes | later |
| Shape operations | toolbar | tool column, below Drawing tools | shown | yes | later |
| Palette | toolbar (strip) | bottom band, above the command line | shown | no | now |
| Layers | panel | right dock, upper group, tab 1, in front | shown | yes | now |
| Jog | panel | upper group, tab 2 | shown | yes | M6 |
| Console | panel | upper group, tab 3 | shown | yes | M6 |
| Macros | panel | upper group, tab 4 | shown | yes | later |
| Properties | panel | upper group, tab 5 | shown | yes | later |
| Text data | panel | upper group, tab 6 | hidden | yes | later |
| Camera | panel | upper group, tab 7 | hidden until a camera is set up | yes | M9 |
| Saved artwork | panel | upper group, last tab when first shown | hidden | yes | later |
| Machine | panel | right dock, lower group, tab 1, in front | shown | yes | M6 |
| Materials | panel | lower group, tab 2 | shown | yes | M8 |
| Command line | panel | bottom band, above the status bar, collapsed | shown | no | now |
| Status bar | region | bottom edge | always | no | now |
| Job bar | region | row 1, right end | while connected | never | M6 |
| Menu bar | region | top | always | no | now |

Notes:

- The reference's object-docking toolbar is part of its default layout map
  but its own page says to enable it from the Window menu if missing [R1][R27];
  ours (Slide and nest) starts hidden because the commands are later and
  "slide" is a new idea for our users.
- The reference's Window menu screenshot shows its wide arrange variant
  unchecked and the short one checked [R5], but those check states are
  one user's setup, not necessarily the defaults. Align and distribute
  starts hidden for our own reason (own design): Arrange already offers
  the common align actions, and a second, wider set of the same buttons
  would crowd row 1 on a first start.

Tier: **Match** (which things are on by default and where), **Own**
(names).

## Docking model

The target behaviour is the reference's (decision 8): any panel can be
shown, hidden, put in a tab group, resized, and later dragged to another
place. It is built in two stages.

### Reference

- Every window and toolbar can be shown or hidden, undocked, moved and
  docked again [R3].
- A window is undocked by dragging its top strip; it can float anywhere or
  be dropped on a dock area left, right, above or below the drawing area.
  Windows can be stacked vertically or dropped on another window to share
  its space as tabs [R3].
- While a window is dragged over a valid place, that area is highlighted
  (blue or light grey); releasing docks it [R3].
- How many windows fit in a column depends on the other docks and the
  screen and window sizes; a refused drop may work after maximizing [R3].
- A dock area is resized by dragging its side or bottom edge. Every window
  has a minimum size, and all windows in the area, hidden tabs included,
  must fit the new size [R3].
- Each docked window has a strongly colored gradient strip on top with its
  name in small light text and two small buttons (float, close); on macOS
  the buttons are at the left [R2][R3][R14]. A floating window gets a small
  title bar of its own [R3].
- Tabs are text-only, at the bottom of the group; the selected tab is
  lighter and joins the panel above [R2][R14][R15]. On older macOS the tabs
  look like a centred segmented control [R35].
- The splitter between stacked groups carries a short row of dots [R3].
- Right-clicking any toolbar, or the top strip of any window, opens a
  check list of all windows and toolbars, the same as in the Window menu
  [R3].
- The side-panel toggle (F12) hides every window docked at the left or
  right and brings the same set back when pressed again [R3]. The Window
  menu page words it as hiding all shown windows [R5]; the two pages differ.
- Resetting the layout re-opens and re-docks the default windows and
  toolbars in their original order and closes the others; preferences,
  devices and libraries are kept. Holding Shift while starting the program
  also resets it [R3][R4].
- The 2.1 release notes list fixes for docks resetting at every start, docks
  taking over the whole window after a display-scale change, and the window
  opening partly off screen [R9].

### LaserCraft: stages

| Stage | What exists | Needs | When |
|---|---|---|---|
| a | Fixed dock positions as in the default arrangement; title strips with a close button; hand-drawn tab strips; show and hide from the Window menu, the panels menu and the close button; F12; splitters between groups and along the dock edge; Reset layout; minimum sizes | plain egui only | with the first milestone that adds a second panel (M6) |
| b | Drag a panel by its title strip or tab to float it or dock it elsewhere (any of the canvas's four edges; into another group as a tab; above or below a group); drop zones; float button; dragging toolbars by their grip to other rows or the left/right edges; floating toolbars | a docking crate such as `egui_dock` or `egui_tiles`: owner decision and an `ARCHITECTURE.md` line | later |
| p | The layout, dock width, group split, front tabs and visibility are remembered between runs | eframe's `persistence` feature or our own settings file: owner decision | later |

Until stage p, every start uses the default arrangement (today panel sizes
are not saved either, current-ui.md 1.2).

### Title strip

- Every docked panel group has one title strip across its top; it names the
  panel in front.
- Look (decision 2): flat `panel-title` fill, one step darker than the
  panel body; the name left-aligned in `type-body`, `text`, regular weight;
  no gradient, no red or orange. The strip is a little shorter than `row`.
- The focused group (the one the user last clicked into) shows a thin
  `accent` line along the strip's lower edge. Only one group is focused at
  a time; focus on the canvas removes the line everywhere.
- Buttons at the right end, on every platform (decision 12; the reference's
  macOS placement on the left is not copied):
  1. Float (stage b only): `icon-small` "lift out" pictogram (`icons.md`);
     tooltip "Float this panel".
  2. Close: `icon-small` cross; tooltip "Hide this panel (Window → Panels
     brings it back)". Hides only the panel in front, not the whole group.
- Buttons are drawn in `text-muted`, `text` on hover with a `hover` square
  behind them.
- Right-click on the strip: the panels menu (below). Double-click: nothing
  in stage a; floats the panel in stage b (own design).
- The status bar, palette strip, command line, tool column and toolbars
  have no title strips.

Tier: **Echo** → **Own** (decision 2).

### Tab strip

- Along the **bottom** edge of a group with two or more visible panels.
- Tabs are text only, in `type-body`, left-aligned from the group's left
  edge, sized to their text with `gap` padding; no icons, no close buttons
  on tabs (the title strip's close button hides the front panel).
- The front tab: `panel` fill, joined to the panel body above it (no line
  between them), name in `text`, with a short `accent` marker line along
  its lower edge. Other tabs: `window` fill, names in `text-muted`, thin
  `border` between tabs. Hover: `hover` fill.
- When the tabs do not fit, each caption is shortened with "…" (all tabs
  shrink evenly, never below a few letters); below that, the tabs that do
  not fit fold into a "»" More button at the strip's right end listing them.
- Click: brings that panel to the front. Right-click: panels menu.
  Keyboard: Ctrl+PgUp/PgDn are taken by drawing order (keymap.md), so tab
  switching has no chord (own design).
- Some panels bring themselves to the front: Machine on connect (if it is
  hidden behind Materials), Console when the user picks "Show in console"
  from an alarm (`machine-panels.md`). Nothing else steals the front.
- A group with one visible panel has no tab strip.

Tier: **Match** (bottom text tabs), **Own** (look).

### Splitters and grips

- Between the canvas and the right dock: the dock's left edge is a vertical
  splitter. Between the upper and lower groups: a horizontal splitter. Under
  the open command-line log: a horizontal splitter.
- Each splitter is a thin `border` line with a short centred grip of a few
  small dots in `text-muted`; on hover the pointer becomes a resize cursor
  and the line turns `accent`.
- Dragging stops at the minimum sizes. Double-clicking a splitter returns
  it to its default position (own design).

Tier: **Match** (dotted grips on splitters), **Own** (look).

### Minimum sizes

- Each panel declares, in code, the smallest width and height at which its
  contents are still usable (for example, Machine's big buttons still show
  their labels). No values are written here.
- A group's minimum is the largest minimum of all its panels, hidden tabs
  included, so switching tabs never forces a resize.
- The right dock's minimum width is the larger of its two groups' minimum
  widths. The canvas has a minimum too (enough to see the bed and use the
  rulers).
- When the user drags past a minimum the splitter stops; nothing is
  hidden automatically.

Tier: **Match**.

### Drop zones (stage b)

- While a panel or toolbar is dragged, each place it may land is shown as a
  drop zone; releasing over one docks it there. Releasing elsewhere leaves
  it floating.
- Panel zones: the four edges of the canvas, the centre of any group (join
  as a tab), and above or below any group (stack). Toolbar zones: the end
  of either toolbar row, a new row below row 2, the left edge (beside the
  tool column) and the right edge (beside the dock).
- Look: `visual-language.md` (not the accent, which decision 3 reserves).
- If a drop would push some region below its minimum, that zone is not
  offered; the user sees no zone there instead of a refused drop.

Tier: **Match** (behaviour), **Own** (look).

### Floating panels and toolbars (stage b)

- A floating panel is an `egui::Window` inside the main window with the
  same title strip (no OS title bar), resizable from its edges, over the
  canvas. A floating toolbar is its button strip with a grip at its start,
  a `panel` background, a thin `border` and a soft shadow.
- Floating things are not side panels, so F12 does not hide them.
- Floating outside the main window (separate OS windows) is later still
  and needs multi-viewport; Wayland may ignore their positions.

Tier: **Match** (behaviour), **Own** (look).

### Panels menu

- Opened by right-clicking a title strip, a tab, the empty part of a
  toolbar row, a toolbar's grip or separator, the palette strip's empty end,
  or the tool column's empty part.
- Contents, top to bottom: the Toolbars check list, a separator, the Panels
  check list, a separator, "Side panels" (check item, F12), "Reset layout".
  The check lists are exactly the Window menu's (below), so there is one
  list to learn.

Tier: **Match** (right-click check list), **Own** (the last two items).

### F12: Side panels

- View → Side panels (check item) or F12 (`ui.toggle_panels`, keymap.md).
- Hides: the tool column (Drawing tools, Node tools, Shape operations) and
  the whole right dock (both groups, every tab). This is today's behaviour
  and the existing spec's (ui-shell.md: tool palette and layers panel);
  decision 15 keeps it, although the reference only hides windows, not its
  tool strips [R3].
- Keeps: menu bar, toolbar band, job bar, palette strip, command line,
  status bar, floating panels and toolbars, open dialogs and windows.
- Pressing again shows the same set that was visible before (which panels,
  which tab in front, the dock width).
- Showing a side panel from the Window or panels menu while side panels are
  hidden ends the hidden state: the previous set comes back together with
  that panel.
- While the right dock is hidden and the machine is connected, the job bar
  still offers Pause and Stop; the Machine panel's other controls come back
  with F12.

Tier: **Match** (key, idea), **Own** (exact set).

### Reset layout

- Window → Reset layout (and the last item of the panels menu). No chord.
- Restores the default arrangement exactly: which toolbars and panels are
  shown, their places, tab order, front tabs (Layers, Machine), dock width
  and group split, command line collapsed. Panels that are hidden by
  default (Text data, Saved artwork, Camera without a camera) are hidden;
  floating ones are re-docked.
- Leaves everything else alone: the project, undo history, zoom, the
  current tool, preferences, keymap, device profiles, the connection.
- Is a front-end action (`ui.reset_layout`), not an undoable command (it
  does not change the project).
- Starting with a layout reset: `lasercraft --reset-layout` (own design; the
  reference uses a held Shift at launch [R3][R4], which egui cannot see
  before the window exists). Only meaningful once layouts persist (stage p).

Tier: **Match** (what it restores and what it keeps), **Own** (flag).

### Window menu check lists

`menus.md` owns the whole Window menu; this file fixes its two check lists.
Checked means shown.

Window → Toolbars, in order (spatial: top row, second row, left column,
bottom; own design — the reference's list is in a different, non-spatial
order [R5]):

1. Standard
2. Arrange
3. Align and distribute
4. Position and size
5. Text style
6. Job setups (M9)
7. Slide and nest
8. Drawing tools
9. Shape operations
10. Palette

Window → Panels, alphabetical (own design; the reference's window list
is mostly, but not strictly, alphabetical [R5]);
each entry's tooltip says what the panel does in plain words (`naming.md`):

1. Camera (M9)
2. Command line
3. Console (M6)
4. Jog (M6)
5. Layers
6. Machine (M6)
7. Macros
8. Materials (M8)
9. Properties
10. Saved artwork
11. Text data

- Node tools is not listed (it follows the Edit nodes tool). The status
  bar, menu bar and job bar are not listed (they cannot be hidden).
- Items that do not exist yet are not listed at all.
- Each item is a `ui.*` front-end action that toggles visibility (proposed
  names in "Proposed changes").

Tier: **Match** (check lists), **Own** (names, order of the toolbar list).

## Toolbar model

### Reference

- Each toolbar starts with a dotted grip used to drag it; groups inside a
  toolbar are split by thin lines [R3][R19][R21][R22].
- When the space is too small, the toolbar collapses and shows an arrow;
  clicking it temporarily shows the hidden buttons, and dragging the grip
  gives the toolbar more room [R3].
- Some buttons hold several related tools and carry a small coloured corner
  marker; a quick click runs the last choice, pressing and holding or
  clicking the marker opens the list [R2][R21][R26][R25]. Vertical toolbars
  open that list to the right of the button [R25]; the arrange toolbar's
  lists open below the button [R20].
- A toolbar can be torn off and float over the drawing area as a strip with
  rounded ends and a soft shadow [R29].
- Toolbars are shown and hidden from the Window menu; Reset restores them
  [R5][R21][R22].
- Resting the pointer on a button shows a short description, sometimes
  with its shortcut; the tooltip's look is not shown in the docs [R28][R2].

### LaserCraft

- **Grip** (stage b): a short line of small dots in `text-muted` at the
  toolbar's start (a vertical line of dots on horizontal toolbars, a
  horizontal one on vertical toolbars). Drag starts there. In stage a
  toolbars cannot move, so no grip is drawn; a thin `border` separator sits
  between neighbouring toolbars instead.
- **Separators**: a thin `border` line between button groups (the groups are
  in `toolbars.md`), with `gap` on each side.
- **More**: when a toolbar does not fit its space, the buttons that do not
  fit are removed from its end and a "»" button (one `icon` wide, glyph in
  `text`, tooltip "More") takes their place. Clicking it opens a popup
  listing the hidden buttons in their toolbar order, each as its icon plus
  its name, with its chord in `text-muted` at the right, like a menu
  (`menus.md` look). Hidden flyout buttons appear as submenus. The popup
  closes after a choice.
- Which toolbar folds first in a row: the rightmost left-packed toolbar
  folds button by button; when it is down to its More button alone, the one
  to its left starts folding. Right-aligned items (job bar, Slide and nest)
  never fold before the left-packed ones; Slide and nest then folds; the job
  bar never folds (see "Job bar").
- **Flyout buttons**: a toolbar button with a small corner marker (glyph in
  `icons.md`) holds related tools. A plain click runs the tool last chosen
  from it; pressing and holding, or clicking the marker area, opens the
  flyout. Flyouts of vertical toolbars open to the right of the button, those
  of horizontal toolbars open below it. The flyout is a popup with one row
  per tool: icon, name, chord (`toolbars.md` lists contents).
- **Tooltips**: every toolbar button's tooltip reads "‹what it does›
  (‹chord›)", the chord taken from the keymap and shown with key symbols
  (`naming.md`); the tooltip look is `visual-language.md`'s.
- **Show and hide**: Window → Toolbars, or the panels menu. Toolbars have no
  close button of their own.
- **Orientation**: Drawing tools and Shape operations are vertical in the
  tool column; when docked in a toolbar row (stage b) they become
  horizontal, and fields inside them (the corner radius field of Shape
  operations) move beside their button instead of below it.

Tier: **Match** (grip, separators, overflow, flyouts), **Echo** (flyout
marker, More), **Own** (look and fold order).

## Narrow and short windows

Own design. Rule of thumb: the canvas gives way first, then toolbars fold,
then text shortens; safety controls and the menu bar never go away.

- **Minimum window size**: the main window has a minimum inner size, set in
  code, at which the menu bar's titles, the job bar, the tool column, the
  right dock at its minimum width and the canvas at its minimum still fit.
  The OS cannot make the window smaller.
- **Width shrinking, in order**:
  1. The canvas narrows down to its minimum.
  2. Toolbar rows fold buttons into More, as described in "Toolbar model".
  3. The palette strip folds its last swatches into More.
  4. The right dock narrows down to its minimum (the user's chosen width is
     remembered and restored when the window grows again).
  5. Tab captions shorten with "…", then fold into the tab strip's More.
  6. Status bar fields give way, in this order: the Messages field shrinks
     to its band and then to "…" with a tooltip; Active setups shortens to a
     count; Work area drops the origin words; Project shortens its path, then
     its name, with "…" (the full text is always the tooltip). Handle
     switches, Pointer position and Selection size keep their width.
  7. The job bar drops the sent percentage, then shortens the state word
     to its dot (word in the tooltip). The constant-power note, the Pause
     label and Stop never shrink (safety rule 9; "Job bar").
- **Height shrinking, in order**:
  1. The command-line log (if open) shrinks to its minimum, then closes to
     the input line.
  2. The canvas shortens down to its minimum.
  3. The right dock's groups shorten proportionally down to their minimums;
     panels inside scroll (every dock panel body is a vertical scroll area).
  4. The tool column folds Shape operations, then Drawing tools, into More
     at its bottom.
- **Never hidden by size**: menu bar, job bar (Pause and Stop), status bar,
  the tool column itself, the command line's input line, Stop on the
  Machine panel when that panel is visible (its big job buttons stay at the
  top of the panel and do not scroll away; `machine-panels.md`).
- **Large windows**: extra width goes to the canvas only; toolbars never
  stretch; the right dock keeps its width.

Tier: **Own**.

## Simple layout (later)

### Reference

- The reference has a reduced mode for beginners, switched on in its
  editor settings and marked in the window title [R6][R13].
- It hides the four handle switches in the status row; in the job-control
  window it hides saving and running machine files, the optimization
  button, the selection-only switch and the rotary and cylinder switches
  (hidden ones do not come back when the mode is left, unless their own
  setting is on); the remaining job buttons grow [R6][R2].
- The arrange toolbar is forced to its short form; move targets collapse
  into one dropdown; distribute and mirror-across-a-line disappear; the
  align buttons become one button that opens an align helper with a live
  preview [R6][R20].
- The boolean buttons become one button that opens a boolean helper with a
  live preview; the tabs tool and the start-point tool are removed [R6][R25].
- Advanced layer settings are hidden or defaulted, and the quick layer
  fields lose the minimum power field [R6][R2].
- Some missing features depend on the device type rather than the mode
  [R6].

### LaserCraft

- Name: Simple layout (decision 11, `naming.md`). A switch in Preferences…
  (later); its tooltip says it hides advanced controls. Window title gains
  " (simple layout)".
- When on, LaserCraft hides (own wording of the same idea):
  - status bar: the handle switches;
  - Machine panel: Save G-code…, Run G-code file…, Cut order…, Selected
    shapes only, Use rotary (`machine-panels.md`); also Reference point
    from selection, which is our own addition (the notes do not say the
    reference's reduced mode hides its selection-origin switch; it only
    acts with Selected shapes only, which is hidden); the remaining big
    buttons keep their size (they already are `row-big`);
  - Align and distribute toolbar (forced off; its Window → Toolbars item is
    disabled); in the Arrange toolbar, the align buttons become one button
    that opens the Align shapes dialog, Move on bed becomes one flyout, and
    the distribute and Mirror across a line buttons disappear;
  - Shape operations: Union, Subtract and Intersect become one Combine
    shapes… button; the start-point tool is removed;
  - Drawing tools: Place tabs is removed;
  - Layers panel: Min power and the advanced part of the layer editor
    (`layers-panel.md` names the fields).
- Unlike the reference, turning Simple layout off brings every hidden
  control back (own design; nothing is hidden for good).
- Menus keep all their items (so a user can always find a command); only
  toolbars and panels are reduced.
- Simple layout never hides the job bar, Stop, Pause, the bounds-check
  message or any safety text.
- Layout and tab order are otherwise the same as the default arrangement.

Tier: **Own** (name and exact set), **Match** (idea, decision 11).

## Multiple windows (later)

### Reference

- A menu command opens another full main window, each with its own project
  and its own device, so one can edit while another runs a job [R7][R8].
- All windows share the program settings; the last window closed writes
  them and overwrites changes made in the others [R8].
- A camera or a laser cannot be used by two windows at once [R8].

### LaserCraft

- File → New window (`naming.md`; later, optional) opens a second main window
  with the same default arrangement and its own project, undo history,
  connection and job bar.
- Each window's title names its own project and device profile, so they
  can be told apart in the OS's window list.
- A serial port in use by one window is shown as busy in the other's Port
  list and cannot be opened twice (the device thread owns the port,
  `device.md`).
- Preferences are written when changed, not when a window closes, so one
  window never overwrites another's changes (own design; differs from the
  reference on purpose).
- Built on eframe multi-viewport; each viewport runs the same shell.

Tier: **Match** (idea), **Own** (settings behaviour).

## Platform notes

- **One look everywhere** (decision 12): egui draws every widget, so Linux,
  Windows and macOS look the same; LaserCraft does not imitate any OS's
  native controls. Title-strip buttons are on the right on every platform.
- **Linux first-class**: test the layout on X11 and on Wayland, on GNOME
  and KDE, with light and dark desktop themes (LaserCraft's own theme
  decides the look, decision 13).
- **Fractional scaling**: egui follows the OS scale factor
  (`pixels_per_point`). Test at whole and fractional factors: thin lines
  (separators, `bed-edge`, splitters) must stay visible and not blur into
  two lines, icons must stay crisp (painted shapes scale cleanly,
  `icons.md`), and the default arrangement must still fit a common laptop
  screen at a large factor. The reference's 2.1 notes record display-scale
  bugs (docks taking over the window, oversized UI) [R9]; those are the
  cases to test.
- **Wayland**: the compositor places windows; floating OS windows (later)
  cannot be positioned by us. Window decorations may be drawn by the
  compositor or by eframe; the title text is the same either way.
- **macOS**: menus stay inside the window (decision 9), not in the system
  menu bar. Ctrl in our keymap means Cmd (keymap.md); Ctrl+H (Home, M6) is
  also the system "hide" chord, as keymap.md notes.
- **Windows**: nothing special beyond the shared look.
- **Serial permission help** (Linux) is shown in the Machine panel with a
  one-line summary in the status bar's Messages field
  (`docs/linux-serial.md`, `device.md`); the layout never moves it into a
  dialog only.
- **Recovery**: if a text or icon size preference (later) makes the window
  unusable, Reset layout does not fix it; a preferences reset does. The
  reference documents such a recovery path [R33]; ours is in
  `later-windows.md`/Preferences (open question).

Tier: **Own**.

## egui implementation notes

No numbers; the code picks sizes from `Style` spacing and the size tokens.

- **Panel types** (egui 0.37, current-ui.md): `egui::Panel::top`,
  `egui::Panel::bottom`, `egui::Panel::left`, `egui::Panel::right` and one
  `egui::CentralPanel`. Inside the right dock, the two groups and their tab
  strips are panels shown inside the dock's `Ui` (`show_inside`).
- **Order of creation** (egui gives the outermost space to the panel created
  first, so this order produces the arrangement above):
  1. `Panel::top("menu")` — menu bar.
  2. `Panel::top("toolbar_row1")` — toolbars left-to-right, the job bar in a
     right-to-left layout at its end (M6). Skipped when it has nothing to
     show.
  3. `Panel::top("toolbar_row2")` — skipped when empty.
  4. `Panel::bottom("status")` — status bar (outermost bottom, as today).
  5. `Panel::bottom("command_line")` — input line; when the log is open, the
     log is a resizable part of the same panel above the input line.
  6. `Panel::bottom("palette")` — palette strip.
  7. `Panel::left("tools")` — tool column (skipped while side panels are
     hidden).
  8. `Panel::left("node_tools")` — only while Edit nodes is active and side
     panels are shown.
  9. `Panel::right("dock")` — resizable; inside it, `Panel::bottom` for the
     lower group (resizable, giving the group splitter) and the rest for the
     upper group; each group draws its own title strip at the top and tab
     strip at the bottom with `Panel::top`/`Panel::bottom` inside the group,
     then a `ScrollArea` for the front panel's body.
  10. `CentralPanel` with `Frame::NONE` — canvas, last.
- **Hand-drawn tab strips** (stage a): a horizontal row of
  `selectable_value`-style widgets over a per-group enum, painted with the
  tab look above (custom `Painter` rects for the joined front tab and the
  `accent` marker). The front tab per group and the visible set live in a
  UI-state struct (for example `DockState`), not in the project.
- **Title strips**: a painted rect in `panel-title` plus a label and two
  small icon buttons; the focused-group line is a painted line.
- **Job bar above dialogs**: the bar is laid out in row 1, but while
  connected it is also drawn (or instead drawn) in an `egui::Area` on the
  top-most layer order, after every window, so no window covers it. While
  connected, dialogs must not use `egui::Modal` (its backdrop takes all
  input); they use windows that block only the canvas and panels, or a
  backdrop that leaves the job bar's rectangle uncovered (`dialogs.md`). A
  UI test in `crates/lasercraft-ui/tests/shell.rs` must click Stop with
  each kind of dialog open and see the stop action (M6).
- **Overflow**: toolbars measure their buttons before drawing (or use the
  previous frame's width) and move the ones that do not fit into the More
  popup; no wrapping layouts (`horizontal_wrapped` is not used for toolbars
  or the palette strip).
- **Actions**: Window menu items, the panels menu, close buttons, tab clicks
  and Reset layout emit `ui.*` front-end actions (they change view state, not
  the project, like `ui.toggle_panels` today). No business logic in the
  layout code.
- **Persistence** (stage p, later): store the `DockState` and sizes through
  eframe's `persistence` feature or our own settings file; owner decision
  and an `ARCHITECTURE.md` line. On load, a stored layout that does not fit
  the current screen or scale is ignored in favour of the default (the
  reference's display-scale bugs [R9] are the reason).
- **Docking crate** (stage b): `egui_dock` or `egui_tiles` would replace the
  hand-drawn groups; the title-strip and tab looks above still apply through
  the crate's style hooks. Not added without the owner.

## Deliberately omitted

Out of scope (decision 1: GRBL gantry lasers only), listed so nobody adds
a place for them:

- The reference's menu of third-party web tools and its built-in web
  browser; vendor menus and vendor bundles.
- (Not omitted, only deferred: a language menu, which `menus.md` places
  between Window and Help later.)
- The file-list tab that the reference adds to its dock for controllers
  with their own file storage, and the send-to-controller button of those
  controllers' job panel.
- Galvo-only panels and buttons, licensing panels, and the network bridge
  product's windows.

## Current state → target

From `current-ui.md` (commit 151eb26); paths relative to the repo root.

- Window title: "LaserCraft", then "‹name› — LaserCraft" with "• " when
  modified (`crates/lasercraft-app/src/main.rs:212-215`) → add "—
  ‹device profile›" from M6; " (simple layout)" later.
- No toolbar band (`crates/lasercraft-ui/src/shell.rs:240-295`) → rows 1
  and 2 as specified; row 1 at least for the job bar in M6.
- Panel creation order today: menu, status, console, palette, tools, layers,
  canvas (`shell.rs:244-277`) → add the toolbar rows after the menu and
  rename the bottom "console" panel to the command line; the order of the
  bottom panels stays.
- Command line: always open with its log, resizable (`shell.rs:247-250`,
  `556-579`) → collapsed to the input line by default, log opens upward
  with a toggle; errors also reach the Messages field.
- Tool column: six text buttons (`shell.rs:537-554`) → Drawing tools and
  Shape operations with icons (`toolbars.md`); Node tools column while Edit
  nodes is active.
- Right dock: one `Panel::right("layers")` with a "Layers" heading and a
  scroll area (`shell.rs:262-270`, `layers_panel.rs:76-316`) → a dock panel
  with a title strip "Layers"; from M6 a lower group (Machine) and tab
  strips.
- Palette strip wraps onto more lines when narrow
  (`layers_panel.rs:46-72`, `horizontal_wrapped`) → one line with a More
  button.
- Status bar order: project, work area, pointer, selection, with separators
  (`shell.rs:624-652`) → handle switches (later), pointer, selection/tool
  hint, messages, active setups, work area, project; pointer value kept in
  `text-muted` when the pointer leaves the canvas instead of disappearing.
- F12 hides tools and layers (`shell.rs:403-406`) → same, plus Node tools
  and the whole right dock, never the job bar.
- No Window menu, no Reset layout, no panels menu → as specified.
- Sizes are not remembered (no eframe `persistence`) → unchanged until
  stage p.
- "Unsaved changes" is an `egui::Modal` (`main.rs:185-210`) → while
  connected it must not block the job bar (`dialogs.md`).
- No theme is set; eframe follows the OS (current-ui.md 2) → LaserCraft's
  light theme by default (`visual-language.md`), so the layout looks the
  same on every desktop.

## Proposed changes to other specs

- **Proposed change to ui-shell.md**: replace its Layout section's region
  list with this file's default arrangement (menu bar, toolbar band,
  tool column, canvas, right dock, palette strip, command line, status
  bar); rename "tool palette" to Drawing tools and the bottom "Console" to
  Command line (`naming.md`); the command line starts collapsed with its log
  opening upward; new status bar field order (pointer, selection or tool
  hint, messages, active setups, work area, project); the job bar (M6);
  the Window menu with Toolbars and Panels check lists, the panels menu and
  Reset layout; the window title gains the device profile (M6).
- **Proposed change to ui-shell.md**: F12 hides the tool column (including
  Node tools) and the whole right dock; it never hides the job bar, toolbar
  band, palette strip, command line or status bar.
- **Proposed change to keymap.md**: add the front-end actions
  `ui.reset_layout`, `ui.show_panel <name>`, `ui.show_toolbar <name>` and
  `ui.toggle_command_log` (not bound to chords; listed so the keymap file can
  bind them); add a rule that the Pause and Ctrl+Break chords work even while
  a text field has keyboard focus, so the safety chords never depend on
  focus (today all shortcuts are ignored in text fields).
- **Proposed change to safety.md**: rule 7 names the job bar as the control
  that keeps Stop and Pause visible, and requires a UI test that Stop is
  clickable with every kind of dialog open.
- **Proposed change to cli.md** (or the app's argument list in ui-shell.md
  "Files"): `lasercraft --reset-layout` starts with the default arrangement
  (only useful once layouts persist).
- **Proposed change to machine-panels.md**: in its constant-power table
  (Job bar row) and its job bar contents table, the note "The laser may
  stay on" is never cut to its tooltip; it sits next to Pause and stays
  at every window width (this file, "Job bar"; safety rule 9).
- **Proposed change to canvas.md**: add a rule for what the view does when
  the canvas region changes size (panels shown or hidden, F12, Node tools
  column): keep the bed point under the centre, keep the top-left, or
  re-fit. This file defers to it.
- **Proposed change to ARCHITECTURE.md** (only when the owner decides):
  a line for a docking crate (stage b) and for eframe `persistence` or a
  settings file (stage p).

## Open questions

1. Docking stage b: does the owner want drag-to-redock at all, and if so
   `egui_dock` or `egui_tiles` (a new dependency)?
2. Layout persistence (stage p): eframe `persistence` or LaserCraft's own
   settings file (which would also hold preferences)? Per device profile or
   global (the reference docs do not say [R3])?
3. Should the Machine panel jump to the front of its group automatically on
   connect, as proposed, or never change the front tab by itself?
4. Job bar: is the sent percentage wanted there, or only the state word and
   the two buttons that decision 7 lists?
5. While connected, should dialogs that are modal today (Unsaved changes)
   become non-modal windows, or keep a backdrop with a hole for the job bar?
   `dialogs.md` must agree.
6. Align and distribute: independent of Arrange (both may be on, as
   proposed) or a replacement that turns Arrange off when shown?
7. Saved artwork: join the upper group as proposed, or dock on the left as
   the reference's examples show (which would break "only tools on the
   left")?
8. Recovery when a size preference makes the UI unusable: a "reset
   preferences" start flag, a menu item, or both?
9. Should the right dock be allowed on the left (a mirrored layout for
   left-handed use) in stage a already, as a preference?
10. Multiple windows: wanted at all, given the device-per-window rule?
11. Tab switching chord: none is proposed (Ctrl+PgUp/PgDn are drawing
    order); is one wanted, for example for jumping to Machine?
12. Canvas view on resize: when panels appear or hide, should the view keep
    its centre, keep its top-left corner or re-fit the bed? `canvas.md`
    decides (proposed change above).
13. Does the reference show Pause and Stop anywhere besides its job-control
    window? The docs read do not say; our job bar does not depend on it.

## Sources

All LightBurn pages read 2026-10-21, through the research notes for this
file (main window layout, status bar and preferences, machine windows, top
toolbars, left toolbars).

- [R1] https://docs.lightburnsoftware.com/2.1/Reference/UI/ — default full
  window map: regions, bands, right column groups and proportions, row 2
  contents, overflow chevron, title contents.
- [R2] https://docs.lightburnsoftware.com/2.1/GetStarted/UITour/ — region
  tour, reduced-mode window, tab strips at the bottom, layer and job panels
  in their groups, swatch row, stacked-button markers.
- [R3] https://docs.lightburnsoftware.com/2.1/Guides/CustomizingTheLightBurnWindow/
  — show and hide, right-click check list, side-panel toggle, docking,
  undocking, tabbing, stacking, drop highlight, resizing and minimum sizes,
  toolbar grips and overflow, nothing on the left by default, reset layout,
  splitter dots, floating window, truncated tabs.
- [R4] https://docs.lightburnsoftware.com/2.1/Reference/ResetToDefaultLayout/
  — what a layout reset restores and keeps; reset at launch.
- [R5] https://docs.lightburnsoftware.com/2.1/Reference/UI/WindowMenu/ —
  Window menu check lists and their order, side-panel toggle wording,
  check states in one screenshot (not necessarily defaults).
- [R6] https://docs.lightburnsoftware.com/2.1/Reference/BeginnerMode/ —
  reduced mode: what it hides and simplifies, title marker, one-way hiding.
- [R7] https://docs.lightburnsoftware.com/2.1/Reference/NewWindow/ — new
  main window command.
- [R8] https://docs.lightburnsoftware.com/2.1/Guides/MultipleLBInstances/ —
  one project and device per window, shared settings overwritten by the
  last window, one device per window.
- [R9] https://docs.lightburnsoftware.com/2.1/NewFeatures/NewIn2.1/ — dock
  and display-scale fixes, off-screen window fix, macOS menu placement in a
  2.1 screenshot, overflow chevron in a narrow window.
- [R10] https://docs.lightburnsoftware.com/2.1/Reference/UI/MenuToolbar/ —
  menu bar line and platform placement.
- [R11] https://docs.lightburnsoftware.com/2.1/Reference/UI/StatusBar/ —
  status row fields and order, pointer freeze, tinted notification bands,
  mode indicator, swatch row overflow.
- [R12] https://docs.lightburnsoftware.com/2.1/Reference/TransformControls/
  — the handle switches at the left of the status row.
- [R13] https://docs.lightburnsoftware.com/2.1/Reference/SettingsPreferences/
  — where the reduced mode is switched on.
- [R14] https://docs.lightburnsoftware.com/2.1/Reference/LaserWindow/ —
  job-control window: default place, tabs, state line, job buttons, title
  strip buttons.
- [R15] https://docs.lightburnsoftware.com/2.1/Reference/MoveWindow/ — jog
  window place and tab order in the upper group.
- [R16] https://docs.lightburnsoftware.com/2.1/Reference/ConsoleWindow/ —
  console place, G-code devices only.
- [R17] https://docs.lightburnsoftware.com/2.1/Reference/MacrosWindow/ —
  macros place, G-code devices only.
- [R18] https://docs.lightburnsoftware.com/2.1/GetStarted/JobControl/ —
  Pause, Stop and Start live in the job-control window.
- [R19] https://docs.lightburnsoftware.com/2.1/Reference/UI/MainToolbar/ —
  main toolbar place in row 1, grips and separators.
- [R20] https://docs.lightburnsoftware.com/2.1/Reference/UI/ArrangeToolbar/
  — arrange toolbar place, dropdowns opening below.
- [R21] https://docs.lightburnsoftware.com/2.1/Reference/UI/CreationToolbar/
  — left column, creation tools on top, grip, horizontal layout, reset.
- [R22] https://docs.lightburnsoftware.com/2.1/Reference/UI/ModifiersToolbar/
  — modifier tools in the lower half of the column.
- [R23] https://docs.lightburnsoftware.com/2.1/Reference/UI/ModesToolbar/ —
  special-mode buttons in row 2.
- [R24] https://docs.lightburnsoftware.com/2.1/Reference/EditNodes/ — the
  node strip appearing beside the creation tools.
- [R25] https://docs.lightburnsoftware.com/2.1/Reference/BooleanTools/ —
  flyout opening to the right; reduced-mode boolean helper.
- [R26] https://docs.lightburnsoftware.com/2.1/Reference/PrimaryShapes/ —
  stacked shape button: quick click versus press and hold.
- [R27] https://docs.lightburnsoftware.com/2.1/Reference/Dock/ — object
  docking toolbar at the far right of row 2, enabled from the Window menu.
- [R28] https://docs.lightburnsoftware.com/2.1/Reference/UI/Tooltips/ —
  hover tooltips (look unknown), referenced by the toolbar model's More and
  flyout rows.
- [R29] https://docs.lightburnsoftware.com/2.1/Reference/Distribute/ — a
  floating toolbar over the drawing area.
- [R30] https://docs.lightburnsoftware.com/2.1/GetStarted/ConnectingToLaser/
  — job-control window for G-code devices (three job buttons).
- [R31] https://docs.lightburnsoftware.com/2.1/Reference/CoordinatesOrigin/
  — job-control window in its group in full-window shots.
- [R32] https://docs.lightburnsoftware.com/2.1/Reference/UndoRedo/ — a 2.1
  full-window shot showing row 1 filled to about three quarters.
- [R33] https://docs.lightburnsoftware.com/2.1/Troubleshooting/ResetToDefaultSettings/
  — recovery when size settings make the interface unusable.
- [R34] https://docs.lightburnsoftware.com/2.1/Reference/Grouping/ — title
  bar with file name, unsaved star and device name.
- [R35] https://docs.lightburnsoftware.com/2.1/Reference/CutSelectedGraphics/
  — older macOS tab look as a segmented control.

Existing specs and documents used: `docs/spec/ui-shell.md` (regions, F12,
status bar, title), `docs/spec/safety.md` (rules 2, 7, 9, 10),
`docs/spec/keymap.md` and `crates/lasercraft-ui/keymap/default.keymap`
(F12, Alt+S, Pause, Ctrl+Break, Ctrl+PgUp/PgDn, Ctrl+H; chords not bound
are named as such), `docs/spec/canvas.md`, `docs/spec/device.md`,
`docs/clean-room.md`, `docs/linux-serial.md`, `docs/hardware-testing.md`,
`ARCHITECTURE.md`. Open standards: GRBL v1.1 (machine states, feed hold,
soft reset, `M3`/`M4`) via `safety.md` and `device.md`.

Own design: every LaserCraft name; the job bar and everything about it; the
title strip, tab, splitter and grip looks; single-panel groups without tab
strips; the command line and its collapsed state; the status bar's fields,
order and bands; the window title form; the toolbar fold order and the More
popup; the narrow and short window rules and the minimum window size; F12's
exact set; Reset layout's flag; the Window menu's toolbar order; Simple
layout's exact set and its full reversal; the multi-window preference rule;
the platform test matrix; the egui panel order and the staging.
