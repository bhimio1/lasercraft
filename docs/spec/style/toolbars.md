# Spec: style — toolbars

Milestones: M1–M3 (the canvas tools and arrange commands that exist today
and get toolbar buttons), M4 (Job preview button), M6 (row 1 must exist for
the job bar; Move head tool, "To head position" and "Go to selection"
items; Edit device profile button with the M6 subset of its dialog), M8
(the full Device profile dialog behind that button), M9 (Job setups toolbar, Capture
camera image button); everything else here (the toolbar band itself, icons,
flyouts, Shape operations, Node tools, Align and distribute, Slide and nest)
is later, with no milestone yet.

This file fixes the contents of every toolbar: which buttons each one holds,
in which order and groups, what each button runs, when it is enabled or
shown as active, what its tooltip says, which buttons are flyouts and what
they hold, and how a toolbar button looks in each state. Where toolbars sit,
how rows fold and how toolbars are shown and hidden is `layout.md`'s (its
"Toolbar band", "Tool column", "Toolbar model" and "Narrow and short
windows" sections); this file only adds what is specific to each toolbar.
Labels come from `naming.md`; colors, type and sizes are only the tokens of
`visual-language.md`; icon artwork is `icons.md`'s (this file names each
icon and its concept). The fields of Position and size and Text style are in
`property-bar.md`; the job bar's buttons in `machine-panels.md`; the canvas
behaviour of each tool in `canvas.md`; the menus that repeat these actions
in `menus.md`; dialogs opened from buttons in `dialogs.md`; the preview
window in `job-preview.md`; the Camera panel in `later-windows.md`.
`README.md` explains the tiers (Match, Echo, Own).

## How to read this file

- **Order**: every toolbar table lists buttons in on-screen order: left to
  right for horizontal toolbars, top to bottom for vertical ones. A row
  "— separator —" is a group divider (thin `border` line, `layout.md`
  "Toolbar model").
- **Button label** is the button's name: what the More popup and a flyout
  row show next to the icon, what accessibility tools read, and what the
  first line of its tooltip names. Toolbar buttons draw no caption
  (icon-only), so the label is never painted on the button itself.
- **Tooltip** text is LaserCraft's own wording, built as `naming.md`
  "Tooltips" says: first line "‹what it does› (‹chord›)", optional second
  line in `text-muted`. The chord is read from the keymap at run time and
  shown with key symbols; a chord is shown only once
  `crates/lasercraft-ui/keymap/default.keymap` binds it. Where this file
  writes "(no chord)" nothing is bound and keymap.md lists nothing; "(Ctrl+K
  once bound)" means keymap.md lists the chord for a later milestone and the
  tooltip shows it from that milestone on.
- **Icon** gives a generic concept in our own words and the icon's name in
  `icons.md`, in lower-case kebab case. Icon names never reuse a size token
  name (`icon`, `icon-small`, `icon-big`). `icons.md` owns the artwork; the
  concepts here are own design and are a starting point, not a drawing.
- **Action** is the registry command or front-end action (`ui.*`, `view.*`)
  the button invokes, exactly as the keymap file would write it. Buttons
  only invoke actions (CLAUDE.md: the UI has no business logic). Names
  marked "(new)" do not exist yet; they are listed under "Proposed changes
  to other specs".
- **Enabled when** names the condition. The UI reads these conditions from
  view state it already has (selection count, clipboard, undo stacks,
  connection state) or from read-only queries the app offers; it never
  computes geometry to decide (CLAUDE.md, ARCHITECTURE.md). If a condition
  cannot be known cheaply (for example "all selected shapes are closed"),
  the button stays enabled and the command's own error message explains the
  refusal in the Messages field.
- **Milestone**: a button is drawn only once its action exists. Until then
  it is absent, not greyed (the same rule `layout.md` uses for the Window
  menu lists). "Now" means the action exists today (as a menu item, key or
  command) and the button appears with the toolbar.

## Toolbar button: look and states

### Reference

- Toolbar buttons are square, flat, icon-only and all the same size within
  a toolbar; they carry no caption and no visible frame at rest [R1][R32].
- Icons are dark monochrome pictograms; a few carry one small colour
  accent, such as the corner marker of buttons that hold several tools, and
  a red-centred crosshair for the head-position move target [R2][R14][R32].
- A button whose action cannot apply is drawn in a pale grey and cannot be
  clicked; this is the normal look of clipboard and arrange buttons with
  nothing selected, and of redo with nothing to redo [R1][R2][R32].
- The current canvas tool has a soft rounded background behind its icon; a
  pale bluish one in the Windows-style captures [R14][R17][R24][R32].
- The node-editing tool's own icon also changes colour while it is active
  [R19].
- Hover and pressed looks are not shown on any page; neither is the look of
  the pan button while pan mode is on [R1][R11].
- Resting the pointer on a control shows a short description, sometimes
  with the shortcut; F1 over a control opens its help page. The tooltip's
  look is not shown [R37][R32].
- A size preference lets the user make toolbar icons larger or smaller;
  the app's high-DPI scaling applies on top of it [R39][R35].

### LaserCraft

Size and spacing:

- A toolbar button is one `icon` square (one `row` tall). Its pictogram is
  drawn inside it with a small even margin (the grid is `icons.md`'s).
- Buttons in a group sit `gap-tight` apart. A separator has `gap` on each
  side. Toolbars in a row sit `gap-wide` apart (`layout.md`).
- Job setups buttons are `icon-big` squares (they fill toolbar row 2's two
  lines); flyout and More popup rows use `icon-small`.
- No toolbar stretches its buttons to fill space; extra width is empty
  `panel` background.
- A later "Icon size" display preference (`naming.md`, Display preferences)
  scales `icon`, `icon-small` and `icon-big` together; the rules here are
  relative, so nothing else changes.

States (all colours are tokens; the full light and dark values are in
`visual-language.md`):

| State | Background | Pictogram | Extra | When |
|---|---|---|---|---|
| Normal | none (the toolbar's `panel` shows through) | strokes in `text`; at most one `accent` detail (`icons.md`) | no border | at rest |
| Hover | `hover`, slightly rounded corners | unchanged | tooltip after egui's normal delay | pointer over an enabled button |
| Pressed | `pressed`, same rounding | unchanged | none | primary button held on an enabled button |
| Active tool | `accent-soft`, same rounding | unchanged (the icon does not change colour) | thin `accent` outline around the rounded background | the button's canvas tool is the current tool |
| Checked | `accent-soft`, same rounding | unchanged | no outline | a toggle button that is on (Job setups while that setup is on) |
| Disabled | none | strokes and the accent detail in `text-disabled`; a flyout marker also turns `text-disabled` | hover shows the tooltip but no `hover` background; clicks do nothing | the action cannot run now |
| Keyboard focus | as the state underneath | unchanged | `accent` focus ring just outside the button | only if keyboard navigation of toolbars is adopted (Open questions) |

- Hover over an active or checked button keeps `accent-soft` (it does not
  switch to `hover`), so the active tool is always readable.
- No toolbar button uses `danger`, `warning` or `success`. Red belongs to
  Stop in the job bar and the Machine panel (decision 6); a pictogram that
  stands for the head never uses red (`icons.md`).
- Clicking a toolbar button never keeps keyboard focus on it: focus goes
  back to the canvas, so canvas shortcuts and the node keys keep working
  after a click (own design; egui would otherwise leave focus on the
  button).
- Right-click on a toolbar's background or on a non-flyout button opens the
  panels menu (`layout.md`); right-click on a flyout button opens its
  flyout (below).

Icon-only and text rules:

- Every toolbar button is icon-only (Match). Text appears in toolbars only
  as: field labels in Position and size, Text style and Shape operations'
  corner radius field; switch labels in Slide and nest; and in popups
  (flyouts, More), where each row shows icon, label and chord.
- Transition (own design): until `icons.md`'s set is drawn, a button may
  show its label in `type-small` instead of a pictogram; the tool column
  then widens to its longest label. This is today's state of the tool
  column (text buttons) and ends when icons land.

Tooltips (own design, `naming.md` "Tooltips"):

- Every button has one; disabled buttons too. A disabled button's second
  line says what is missing ("Select two or more objects.", "Needs a
  connected machine.").
- Tools read "‹Tool name› tool (‹chord›)" (today's form, kept).
- A flyout button's tooltip names the member on its face, then a second
  line "Hold, or click the corner, for more." (wording own design).
- F1 over a button opens that button's user guide page (Context help,
  M10).

Tier: **Echo** (the active-tool background and the disabled grey are the
reference's ideas; ours use our tokens and an outline), **Own** (all
colours, the outline, focus handling, tooltip wording, the transition rule).

## Flyout buttons

### Reference

- Some buttons hold a family of related tools. They carry a small coloured
  triangle in one corner as a marker; in the screenshots it sits at the
  lower right, while the tour text speaks of a small arrow in the lower
  left [R14][R17][R23][R32].
- A plain quick click runs the member chosen last; pressing and holding
  anywhere on the button, or clicking the corner marker, opens the list
  [R17][R23]. Picking a member makes it the button's new face [R23].
- The face of the shape button changes to the shape last chosen [R17][R19];
  the boolean button shows the operation last used [R23].
- In the vertical tool strip the list opens to the right of the button as a
  popup with rounded corners, one row per member with a small icon and a
  text name [R23]. The arrange toolbar's lists open directly below the
  button, left-aligned, in menu style: small icon, name, separators, a
  light-blue band on the hovered row and no shortcuts [R2][R8].
- Version 2.1 made these buttons slightly wider so their side is easier to
  hit [R35].
- When disabled, the marker turns grey together with the icon [R15][R23].
- The look of the open shape list is not shown [R17].

### LaserCraft

- **Face**: the button shows the icon of its current member. At first start
  (and after Reset layout) that is the default member named in each table
  below. Choosing a member makes it the face; the face is view state, kept
  for the session and saved with the layout once layouts persist
  (`layout.md`, stage p). It is not part of the project.
- **Marker**: a small solid right-angled triangle filling the lower-right
  corner of the button, in `text-muted` (never teal: teal is only `guide`;
  never `accent`: the accent detail belongs to the pictogram). Disabled:
  `text-disabled`. The button keeps the normal `icon` square; it is not
  wider (own choice; the hold and right-click gestures make the corner
  target less critical).
- **Opening** (any of):
  - press and hold the primary button on the flyout button for a short
    moment (own design: shorter than the tooltip delay);
  - click inside the marker corner (the lower-right quarter of the button);
  - right-click the button.
- **Plain click** anywhere else runs the face member: a tool becomes the
  current tool; a command runs once. If the face member is disabled but
  another member is enabled, a plain click opens the flyout instead.
- **Enabled state**: the button is enabled when at least one member is
  enabled; disabled only when all are.
- **Placement**: flyouts of vertical toolbars (tool column) open to the
  right of the button, their top edge level with the button's top; flyouts
  of horizontal toolbars open directly below the button, left edges
  aligned. If the popup would leave the window it shifts to stay inside
  (egui's popup placement).
- **Popup look** (`menus.md` drop-down look, so menus and flyouts are one
  family): `field` background, thin `border` outline, slightly rounded
  corners, the shadow `visual-language.md` gives popups. One row per
  member, `row` tall: a leading check column (a check mark in `text` on
  the member that is the current face), `icon-small` pictogram, label in
  `type-body`, chord right-aligned in `text-muted` when one is bound.
  Separators between member groups are thin `border` lines. The row under
  the pointer gets `accent-soft`. Disabled rows: label, icon and chord in
  `text-disabled`; hovering one shows its tooltip with the reason.
- **Headings**: a flyout with two blocks (Move on bed) shows each block's
  name as a non-clickable caption row in `type-small`, `text-muted`.
- **Closing**: choosing a row (which runs it and makes it the face),
  Esc, a click outside, or opening another popup. Esc here closes only the
  popup; it does not also switch the tool to Select.
- **Keyboard**: arrow keys move the row highlight and Enter chooses while
  the popup is open (egui menu behaviour); there is no chord that opens a
  flyout.
- **In the More popup**: a hidden flyout button appears as a submenu named
  after its flyout (Shape tools ▸, Booleans ▸, …) with the same rows
  (`layout.md` "Toolbar model").

Tier: **Echo** (corner marker, quick click versus hold, face follows the
last choice, open direction), **Own** (marker colour and shape, right-click,
the disabled-face rule, popup look, check column, headings).

## Toolbar band and tool column (summary)

### Reference

- Two full-width toolbar rows sit under the menu bar: the first holds the
  file and view toolbar and the arrange toolbar; the second, about twice as
  tall, holds the numeric position and size fields, the text options, the
  special-mode buttons and the object-docking toolbar [R31][R32][R1][R2].
- A narrow vertical strip at the left edge holds the creation tools above
  the modifier tools, each starting with a dotted grip; a node-action strip
  appears beside it while nodes are edited [R14][R15][R19][R31].
- Each toolbar starts with a dotted grip and splits its groups with thin
  lines; a toolbar that does not fit collapses behind a double chevron that
  shows the hidden buttons on click [R33][R31][R35].
- Toolbars are shown and hidden through checkable Window menu entries, and a
  layout reset restores them [R12][R14][R15].

### LaserCraft

The places, row heights, fold order, More button, grips, separators,
Window → Toolbars list and F12 behaviour are fixed in `layout.md`. In short,
with the names used below:

| Place | Toolbars, in order | Default |
|---|---|---|
| Row 1 | Standard, Arrange, Align and distribute; job bar at the right end | Standard and Arrange shown; Align and distribute hidden; job bar while connected (M6) |
| Row 2 | Position and size, Text style, Job setups (M9); Slide and nest at the right end | Slide and nest hidden |
| Tool column | Drawing tools above Shape operations; Node tools beside Drawing tools while Edit nodes is active | shown (F12 hides the column) |

Per-toolbar fold order, following `layout.md`'s rule that a toolbar folds
button by button from its end: each toolbar folds its buttons one at a time
from the last one backwards. A separator that ends up with no visible
button after it disappears too; in the More popup a separator is drawn
between folded buttons that came from different groups. The tool column
folds Shape operations from its bottom first, then Drawing tools from its
bottom; Select is never folded.

Tier: **Match** (bands, column, order), **Own** (separator handling in the
fold).

## Standard toolbar

### Reference

- Purpose: files, history, clipboard, view movement and zoom, a camera
  refresh, the job preview and the two settings dialogs [R1][R32].
- Default place: far left of row 1, directly before the arrange toolbar
  [R1][R31].
- Nineteen icon-only buttons in six groups: four file buttons (new, open,
  save, import); undo and redo; cut, copy, paste, delete; pan, zoom to the
  whole page, zoom in, zoom out, zoom to the selection and the camera
  refresh; preview; then program settings and device settings [R1][R9][R31].
  One capture and the page text list copy before cut; most captures show cut
  first [R1][R9].
- States: undo and redo grey when their history is empty; cut, copy and
  delete grey without a selection; paste grey with an empty clipboard; the
  camera button grey and unclickable unless a camera is in use [R1][R9][R10].
- Pan turns on a mode in which a left drag slides the view; Esc or picking
  another tool ends it; holding Space or dragging with the middle button pans
  as well [R1][R11].
- Paste puts the clipboard contents centred on the pointer; paste in place
  and duplicate are menu items, not buttons [R9].
- Zoom to the selection fits the selection, or every object when nothing is
  selected [R11]. The Window menu lists the same zoom and preview entries with
  the same icons [R12].

### LaserCraft

Purpose: the everyday project, history, clipboard and view actions, plus the
preview and the two settings dialogs. Default place: row 1, far left. Tier
of contents and order: **Match**; labels, tooltips and icons: **Own**.

| # | Button label (tooltip, our words) | Icon concept (`icons.md` name) | Action | Enabled when | Checked / active | Flyout members | Milestone |
|---|---|---|---|---|---|---|---|
| 1 | New ("Start an empty project (Ctrl+N)") | blank sheet with a small plus (`new-project`) | `ui.new` | always (asks about unsaved changes first, ui-shell.md) | — | — | Now |
| 2 | Open… ("Open a LaserCraft project (Ctrl+O)") | folder with its lid open (`open-project`) | `ui.open` | always | — | — | Now |
| 3 | Save ("Save the project (Ctrl+S)"; second line with no known path: "Asks where to save the first time.") | tray with an arrow coming down into it (`save-project`) | `ui.save` | always | — | — | Now |
| 4 | Import… ("Add SVG or DXF artwork to this project (Ctrl+I)") | sheet with an arrow coming in from the side (`import-art`) | `ui.import` | always | — | — | Now |
| — | — separator — | | | | | | |
| 5 | Undo ("Undo ‹action› (Ctrl+Z)", the action in readable words, e.g. "Undo move") | arrow curving back to the left (`undo`) | `edit.undo` | the undo stack is not empty | — | — | Now |
| 6 | Redo ("Redo the last undone action (Ctrl+Shift+Z)") | the same arrow mirrored (`redo`) | `edit.redo` | the redo stack is not empty | — | — | Now |
| — | — separator — | | | | | | |
| 7 | Cut ("Move the selection to the clipboard (Ctrl+X)") | open scissors (`cut`) | `edit.cut` | something is selected | — | — | Now |
| 8 | Copy ("Copy the selection to the clipboard (Ctrl+C)") | two overlapping sheets (`copy`) | `edit.copy` | something is selected | — | — | Now |
| 9 | Paste ("Paste the clipboard (Ctrl+V)"; second line "From this button the copy lands where the original was.") | clipboard board with a sheet (`paste`) | `ui.paste` | LaserCraft's clipboard holds shapes | — | — | Now |
| 10 | Delete ("Delete the selection (Del)") | waste bin (`delete`) | `shape.delete` | something is selected | — | — | Now |
| — | — separator — | | | | | | |
| 11 | Pan ("Pan tool: drag to move the view"; second line "Or drag with the middle button, or hold Space and drag.") | open hand (`pan`) | `ui.tool pan` (new) | always | active-tool look while the pan tool is current | — | Later |
| 12 | Fit bed ("Show the whole bed (Ctrl+0)") | the bed between two inward-pointing arrowheads (`fit-bed`) | `view.zoom_fit` | always | — | — | Now |
| 13 | Zoom in ("Zoom in (Ctrl+=)") | magnifier with a plus (`zoom-in`) | `view.zoom_in` | the view is not at its closest zoom (canvas.md limits) | — | — | Now |
| 14 | Zoom out ("Zoom out (Ctrl+-)") | magnifier with a minus (`zoom-out`) | `view.zoom_out` | the view is not at its farthest zoom | — | — | Now |
| 15 | Zoom to selection ("Zoom to the selection (no chord)"; second line "With nothing selected, fits every shape.") | magnifier over a dashed box (`zoom-selection`) | `view.zoom_selection` (new) | the project has at least one shape | — | — | Later |
| 16 | Capture camera image ("Take a new camera picture for the overlay (Alt+C once bound)") | camera with a small refresh arc (`camera-capture`) | `ui.camera_capture` (new; keymap.md lists Alt+C for M9) | a camera is set up and reachable | — | — | M9 |
| — | — separator — | | | | | | |
| 17 | Job preview… ("Preview the job's moves and time (Alt+P)") | an eye above a half-dashed path (`job-preview`) | `ui.preview` | always (the preview explains an empty job itself) | — | — | Now |
| — | — separator — | | | | | | |
| 18 | Preferences… ("Settings for LaserCraft itself (no chord)") | slider panel with two knobs (`preferences`) | `ui.preferences` (new) | always | — | — | Later |
| 19 | Edit device profile… ("Settings of the active machine (no chord)") | the gantry with a small pencil (`device-profile`) | `ui.edit_device_profile` (new) | a device profile is active (always from M6, which has one built-in profile) | — | — | M6 (the M6 subset of the Device profile dialog: work area, port, baud; `machine-panels.md`), M8 (all tabs) |

Notes:

- Save as…, Save G-code…, Paste in place, Duplicate, Select all and Invert
  selection stay menu-only, as in the reference [R1][R9]. Save G-code… also gets
  a Machine panel button in M6 (`naming.md`).
- Paste: today `ui.paste` pastes at the pointer when the pointer is over the
  canvas, otherwise in place (`crates/lasercraft-ui/src/shell.rs:383-386`).
  From the toolbar the pointer is on the button, so the toolbar button
  pastes in place. This differs from the reference, which centres the paste
  on the pointer [R9]; see Open questions.
- Pan is a canvas tool, not a command: it is the current tool until Esc or
  another tool is chosen (Match [R1]). The middle-button and Space gestures
  keep working with any tool (canvas.md) and do not change the current tool.
- Zoom to selection has no chord: keymap.md binds Ctrl+Shift+A to framing
  the selection with the laser (legacy page, M6), while the reference's
  Window menu shows that chord on zoom to selection [R12]. Owner decision
  (Open questions; `naming.md` lists the same question).
- Capture camera image is drawn from M9 on. With no camera set up it is
  disabled with the second line "Set up a camera first." (the reference
  greys it the same way [R1]).
- Zoom in and out are disabled only at the zoom limits (own design; the
  reference shows no disabled state for them [R1]).
- The device settings button moves the machine nowhere and sends nothing;
  opening it while connected is allowed (`dialogs.md` decides what may be
  edited while a job runs).

Deliberately omitted: nothing from the reference's set; the two settings
buttons are kept with our own dialogs.

## Arrange toolbar

### Reference

- Purpose: a compact subset of the arrange menu for placing objects on
  their own and against each other [R2][R32].
- Default place: row 1, directly after the file and view toolbar [R2][R31].
- Thirteen icon-only buttons in six groups: group and ungroup; flip
  vertically, flip horizontally, mirror across a line; align centres, a
  vertical-line align list, a horizontal-line align list; a vertical and a
  horizontal distribute list; make the same width, make the same height; one
  list for moving the selection or the laser [R2][R3][R4][R5][R6][R7].
- Only the horizontal-line align list (top, horizontal centres, bottom) and
  the move list are shown open; the other three lists are named in the text
  only, so their exact items are unknown [R2][R3][R4].
- The move list has two blocks: ten targets that move the selection (bed
  centre, four corners, the laser's position, four edge midpoints) and nine
  that send the laser to a point of the selection [R2][R8].
- Holding Ctrl or Cmd while clicking a move target moves the laser to that
  point of the selection instead [R8].
- Moving to the laser's position needs a connected machine that homes and
  reports its position reliably [R8].
- Make-same buttons resize every selected object to the anchor object's
  width or height; Shift-click keeps each object's proportions [R7].
- Packing objects edge to edge is menu-only, not on the toolbar; the
  arrange menu also holds items the toolbar leaves out (draw order, locking,
  the slide submenu, nesting, arrays and automatic grouping), while its two
  move submenus are repeated in the toolbar's last list [R4][R13][R2][R8].
- Align and make-same follow the same anchor rules (last added object, then
  drawing order) [R3][R7].

### LaserCraft

Purpose: group, mirror, align, distribute, match size and move on the bed,
with families behind flyouts. Default place: row 1, after Standard. Tier of
contents and order: **Match**; flyout contents where the reference does not
show them, labels, tooltips and icons: **Own**.

"Object" and "anchor object" are arrange.md's words. Enabled conditions
count top-level objects in the selection.

| # | Button label (tooltip, our words) | Icon concept (`icons.md` name) | Action | Enabled when | Checked / active | Flyout members | Milestone |
|---|---|---|---|---|---|---|---|
| 1 | Group ("Group the selected objects (Ctrl+G)") | a small square and circle inside one solid rounded box (`group`) | `group.create` | two or more objects selected | — | — | Now |
| 2 | Ungroup ("Split the selected group one level (Ctrl+U)") | the same box broken open, the shapes nudged apart (`ungroup`) | `group.ungroup` | a grouped object is selected | — | — | Now |
| — | — separator — | | | | | | |
| 3 | Mirror vertically ("Mirror top to bottom (Ctrl+Shift+V)") | a half shape and its mirror image across a dashed horizontal axis (`mirror-vertical`) | `shape.flip vertical` | something is selected | — | — | Now |
| 4 | Mirror horizontally ("Mirror left to right (Ctrl+Shift+H)") | the same across a dashed vertical axis (`mirror-horizontal`) | `shape.flip horizontal` | something is selected | — | — | Now |
| 5 | Mirror across a line ("Mirror the selection over a two-point line (Ctrl+Shift+M once bound)") | the same across a slanted dashed axis (`mirror-line`) | `shape.mirror_line` (new) | something is selected (the command explains what the line must be) | — | — | Later |
| — | — separator — | | | | | | |
| 6 | Align centers ("Center every object on the anchor object (Alt+Home)") | two boxes sharing one centre cross (`align-centers`) | `arrange.align center` | two or more objects selected | — | — | Now |
| 7 | Align on a vertical line (flyout; face default "Left edges") | stacked bars against a vertical line (face shows the member) | flyout | two or more objects selected | — | Left edges (`align-left`, `arrange.align left`, Alt+←); Vertical centers (`align-center-x`, `arrange.align center-x`, Alt+PgUp); Right edges (`align-right`, `arrange.align right`, Alt+→) | Now |
| 8 | Align on a horizontal line (flyout; face default "Top edges") | side-by-side bars against a horizontal line (face shows the member) | flyout | two or more objects selected | — | Top edges (`align-top`, `arrange.align top`, Alt+↑); Horizontal centers (`align-center-y`, `arrange.align center-y`, Alt+PgDn); Bottom edges (`align-bottom`, `arrange.align bottom`, Alt+↓) | Now |
| — | — separator — | | | | | | |
| 9 | Distribute vertically (flyout; face default "Vertical spacing") | stacked bars with equal-gap marks (face shows the member) | flyout | three or more objects selected | — | Vertical spacing (`distribute-v-spaced`, `arrange.distribute v-spaced`, no chord); Vertical centers (`distribute-v-centered`, `arrange.distribute v-centered`, no chord) | Now |
| 10 | Distribute horizontally (flyout; face default "Horizontal spacing") | side-by-side bars with equal-gap marks (face shows the member) | flyout | three or more objects selected | — | Horizontal spacing (`distribute-h-spaced`, `arrange.distribute h-spaced`, no chord); Horizontal centers (`distribute-h-centered`, `arrange.distribute h-centered`, no chord) | Now |
| — | — separator — | | | | | | |
| 11 | Match width ("Give every object the anchor object's width (no chord)") | two boxes with a shared width arrow (`match-width`) | `arrange.match_size width` (new) | two or more objects selected | — | — | Later |
| 12 | Match height ("Give every object the anchor object's height (no chord)") | two boxes with a shared height arrow (`match-height`) | `arrange.match_size height` (new) | two or more objects selected | — | — | Later |
| — | — separator — | | | | | | |
| 13 | Move on bed (flyout; face default "Center") | the bed with the selection square at the target (face shows the member) | flyout | block "Move on bed": something is selected; block "Go to selection": also connected, the machine idle and homed since it connected | — | see the table below | Now (first block); M6 (head items) |

Move on bed flyout, in order (two blocks, each with its caption row):

| # | Row label (tooltip) | Icon (`icons.md` name) | Action | Enabled when | Milestone |
|---|---|---|---|---|---|
| — | caption "Move on bed" | | | | |
| 1 | Center ("Move the selection to the middle of the bed") | the bed with a small square in its middle (`bed-center`) | `arrange.move_to_bed center` | something is selected | Now |
| 2 | Top left ("Move the selection into the bed's top-left corner") | the bed with the small square in that corner (`bed-top-left`) | `arrange.move_to_bed top-left` | something is selected | Now |
| 3 | Top right | (`bed-top-right`) | `arrange.move_to_bed top-right` | something is selected | Now |
| 4 | Bottom left | (`bed-bottom-left`) | `arrange.move_to_bed bottom-left` | something is selected | Now |
| 5 | Bottom right | (`bed-bottom-right`) | `arrange.move_to_bed bottom-right` | something is selected | Now |
| 6 | To head position ("Move the selection so its reference point is under the laser head"; second line "Uses the last position the machine reported.") | the bed with the head ring and the small square centred on it (`bed-head`) | `arrange.move_to_head` (new) | something is selected, connected, the machine has homed since it connected, and a position has been reported | M6 |
| 7 | Left ("Move the selection against the bed's left edge, centred along it") | the bed with the small square against that edge (`bed-left`) | `arrange.move_to_bed left` | something is selected | Later (the command exists; the menu lacks it today) |
| 8 | Right | (`bed-right`) | `arrange.move_to_bed right` | something is selected | Later |
| 9 | Top | (`bed-top`) | `arrange.move_to_bed top` | something is selected | Later |
| 10 | Bottom | (`bed-bottom`) | `arrange.move_to_bed bottom` | something is selected | Later |
| — | separator, then caption "Go to selection" | | | | |
| 11 | Center ("Send the laser head to the middle of the selection, laser off") | dashed selection box with the head ring at its centre (`goto-center`) | `machine.goto_selection center` (new) | something is selected, connected, machine idle (Ready), and homed since it connected | M6 |
| 12 | Top left | (`goto-top-left`) | `machine.goto_selection top-left` (new) | as row 11 | M6 |
| 13 | Top right | (`goto-top-right`) | `machine.goto_selection top-right` (new) | as row 11 | M6 |
| 14 | Bottom left | (`goto-bottom-left`) | `machine.goto_selection bottom-left` (new) | as row 11 | M6 |
| 15 | Bottom right | (`goto-bottom-right`) | `machine.goto_selection bottom-right` (new) | as row 11 | M6 |
| 16 | Top | (`goto-top`) | `machine.goto_selection top` (new) | as row 11 | M6 |
| 17 | Bottom | (`goto-bottom`) | `machine.goto_selection bottom` (new) | as row 11 | M6 |
| 18 | Left | (`goto-left`) | `machine.goto_selection left` (new) | as row 11 | M6 |
| 19 | Right | (`goto-right`) | `machine.goto_selection right` (new) | as row 11 | M6 |

Notes:

- The vertical-line align list and both distribute lists are not shown open
  in the reference; their members here are our own reading of what the
  family names imply (own design). The horizontal-line list matches the
  reference's [R2].
- Align centers stays a direct button and is not repeated in a flyout
  (Match [R2]). The two align flyouts' button labels ("Align on a vertical
  line", "Align on a horizontal line") are the names the More popup and the
  tooltip's first line use while the face member's own tooltip follows:
  "Align left edges with the anchor object (Alt+←)", and so on.
- Pack horizontally and Pack vertically (Alt+Shift+H, Alt+Shift+V) stay in
  the Arrange → Distribute submenu only (Match [R4]).
- Match width and height: the reference's Shift-click keeps proportions
  [R7]. LaserCraft adopts it as an argument of the new command
  (`arrange.match_size width keep`), which the button passes when Shift is
  held; its tooltip's second line says "Shift+click keeps each object's
  proportions." (own wording).
- Go to selection moves the head with the laser off (safety rules 1 and 3).
  The target is checked against the work area first; a target outside it is
  refused with an error that names the axis and the distance (own design,
  following the idea of safety rule 5). The rows are disabled while a job
  runs, frames, is paused, the machine is in an alarm, or the machine has
  not homed since it connected (homing rule under Drawing tools).
- The reference's Ctrl-click on a move target, which moves the laser
  instead of the objects [R8], is **not** adopted: machine motion only comes
  from rows that say so (own design for safety; Open questions).
- The Go to selection block repeats the Machine → Go to selection ▸
  submenu (`naming.md`; `menus.md` decides where that submenu lives).
- No arrange button is coloured. The reference's red-centred head target
  [R2] becomes the neutral head ring pictogram (`icons.md`).

## Align and distribute toolbar (wide variant)

### Reference

- A wide form of the arrange toolbar for large screens; it is unchecked in
  the Window menu capture and is turned on from there; the docs imply the
  short form is the default [R2][R12].
- Twenty-five buttons: the same grouping and mirror groups; align centres
  and all six edge and centre alignments as separate buttons (a faint
  divider between the vertical-line three and the horizontal-line three);
  make-same width and height; the four distribute actions; four corner
  targets, the bed centre and the laser position as separate buttons; and
  the full move list as the last button [R3][R4][R7][R8].
- Edge-midpoint targets stay inside the move list even in this form [R8].
- In the distribute group the order within each pair cannot be read from
  the captures [R4].

### LaserCraft

Purpose: the same actions as Arrange with every common action as its own
button, for wide windows. Default: hidden; when shown it sits in row 1 after
Arrange (`layout.md`). Both may be shown at once (Open questions, shared
with `layout.md`). Tier: **Match** (contents and order), **Own** (order
inside the distribute pairs, labels, icons).

All rows use the same label, tooltip, icon, action, enabled condition and
milestone as the matching Arrange row or flyout member above; only the
order is given here.

| # | Button label | Icon (`icons.md` name) | Action | Enabled when | Checked / active | Flyout members | Milestone |
|---|---|---|---|---|---|---|---|
| 1 | Group | `group` | `group.create` | as Arrange | — | — | Now |
| 2 | Ungroup | `ungroup` | `group.ungroup` | as Arrange | — | — | Now |
| — | — separator — | | | | | | |
| 3 | Mirror vertically | `mirror-vertical` | `shape.flip vertical` | as Arrange | — | — | Now |
| 4 | Mirror horizontally | `mirror-horizontal` | `shape.flip horizontal` | as Arrange | — | — | Now |
| 5 | Mirror across a line | `mirror-line` | `shape.mirror_line` (new) | as Arrange | — | — | Later |
| — | — separator — | | | | | | |
| 6 | Align centers | `align-centers` | `arrange.align center` | two or more objects | — | — | Now |
| 7 | Align left edges | `align-left` | `arrange.align left` | two or more objects | — | — | Now |
| 8 | Align vertical centers | `align-center-x` | `arrange.align center-x` | two or more objects | — | — | Now |
| 9 | Align right edges | `align-right` | `arrange.align right` | two or more objects | — | — | Now |
| — | — separator (lighter: `border` at reduced strength, own choice) — | | | | | | |
| 10 | Align top edges | `align-top` | `arrange.align top` | two or more objects | — | — | Now |
| 11 | Align horizontal centers | `align-center-y` | `arrange.align center-y` | two or more objects | — | — | Now |
| 12 | Align bottom edges | `align-bottom` | `arrange.align bottom` | two or more objects | — | — | Now |
| — | — separator — | | | | | | |
| 13 | Match width | `match-width` | `arrange.match_size width` (new) | two or more objects | — | — | Later |
| 14 | Match height | `match-height` | `arrange.match_size height` (new) | two or more objects | — | — | Later |
| — | — separator — | | | | | | |
| 15 | Distribute vertical spacing | `distribute-v-spaced` | `arrange.distribute v-spaced` | three or more objects | — | — | Now |
| 16 | Distribute vertical centers | `distribute-v-centered` | `arrange.distribute v-centered` | three or more objects | — | — | Now |
| 17 | Distribute horizontal spacing | `distribute-h-spaced` | `arrange.distribute h-spaced` | three or more objects | — | — | Now |
| 18 | Distribute horizontal centers | `distribute-h-centered` | `arrange.distribute h-centered` | three or more objects | — | — | Now |
| — | — separator — | | | | | | |
| 19 | Move to top left | `bed-top-left` | `arrange.move_to_bed top-left` | something selected | — | — | Now |
| 20 | Move to top right | `bed-top-right` | `arrange.move_to_bed top-right` | something selected | — | — | Now |
| 21 | Move to bottom left | `bed-bottom-left` | `arrange.move_to_bed bottom-left` | something selected | — | — | Now |
| 22 | Move to bottom right | `bed-bottom-right` | `arrange.move_to_bed bottom-right` | something selected | — | — | Now |
| 23 | Move to bed center | `bed-center` | `arrange.move_to_bed center` | something selected | — | — | Now |
| 24 | Move to head position | `bed-head` | `arrange.move_to_head` (new) | as Arrange's row 6 of the Move on bed flyout | — | — | M6 |
| 25 | Move on bed (flyout; face default "Left") | as Arrange | flyout | as Arrange | — | the full Move on bed flyout above (both blocks) | Now |

Notes:

- Inside each distribute pair, spacing comes before centres (own choice,
  because the reference's order cannot be read [R4]); vertical before
  horizontal follows the reference [R4].
- In this variant the More popup and tooltips use the longer labels shown
  in the table (with the verb), because the button is no longer inside a
  named flyout.
- Row 25's face defaults to "Left", the first edge target, because the
  centre and corners already have their own buttons (own choice).

## Job bar (summary)

### Reference

- The reference has no separate job strip: its start, pause and stop
  buttons live in the job-control panel at the lower right of the dock
  [R31][R32].

### LaserCraft

The job bar is LaserCraft's own always-visible strip at the right end of
row 1 while connected: machine state word, Pause/Resume and Stop. Its place,
contents, states and the rules that keep it visible over every dialog are
in `layout.md` ("Job bar"); its buttons' look and behaviour in detail are in
`machine-panels.md`. For this file it matters only that:

- with a constant-power (M3) job, Pause reads "Pause (motion only)" and
  Stop is highlighted (decision 7, safety rule 9, `machine-panels.md`);
- it is not a toolbar: it has no grip, no More button, no Window → Toolbars
  entry and never folds;
- toolbars to its left fold into More before it gives up any space;
- its Stop is the only `danger` control in the toolbar band.

Tier: **Own**. Milestone: M6.

## Position and size toolbar (summary)

### Reference

- Row 2, leftmost: two-line numeric fields for X and Y, a lock for keeping
  proportions, width and height, percentage scale fields, a nine-point
  reference grid, a rotation field and a units control (two stacked radio
  buttons in 2.1) [R38][R31][R35].
- All fields are greyed with nothing selected [R32].

### LaserCraft

Contents in order: X (mm) over Y (mm); Keep proportions (a lock toggle the
height of both lines); Width (mm) over Height (mm); Width (%) over Height
(%); Reference point (3×3 point grid); Rotation (°); mm / in. Every field,
its look, enabled rules, typing and dragging behaviour and the commands it
runs are in `property-bar.md`. Default place: row 2, left. Disabled with no
selection. Milestone: later (no numeric fields exist today, current-ui.md
1.11). Tier: **Match** (order), **Own** (look, labels).

## Text style toolbar (summary)

### Reference

- Row 2, right of the numeric fields: font and height, small style switches
  in columns, letter and line spacing, horizontal and vertical alignment, and
  a text-source mode with an offset; everything greyed unless text is
  selected [R30][R31][R32].

### LaserCraft

Contents in order: Font and Capital height (mm) (for the next text, from
the Add text window's defaults); Bold, Italic, All caps, Weld letters, Bend
letters along arc, Right to left (only with the Right-to-left text
preference); Letter spacing (%) over Line spacing (%); Horizontal
align over Vertical align; Text source over Counter offset. The rows beyond
font and height need editable text objects (proposed change to text.md,
owner decision). Details, enabled rules and the two-line layout are in
`property-bar.md`. Default place: row 2, after Position and size.
Milestone: later. Tier: **Match** (place, order), **Own** (labels, look).

## Job setups toolbar

### Reference

- A short strip of large icon-only buttons that open the setup dialogs or
  wizards for special processes; by default it sits in the second row
  between the text options and the object-docking toolbar [R16][R31].
- Its buttons depend on the machine type: for gantry machines a rotary
  setup, a print-and-cut registration and a split-marking setup that the
  text ties to galvo work [R16][R35].
- It is turned back on from the Window menu [R16].

### LaserCraft

Purpose: open the job setups (rotary attachment and print registration) and
show at a glance which ones are on. Default place: row 2, after Text style
(`layout.md`). Milestone: M9. Tier: **Match** (place, idea), **Own** (the
checked state, labels, icons).

| # | Button label (tooltip, our words) | Icon concept (`icons.md` name) | Action | Enabled when | Checked / active | Flyout members | Milestone |
|---|---|---|---|---|---|---|---|
| 1 | Rotary attachment… ("Set up a rotary attachment for round objects (Ctrl+Shift+R once bound)"; second line when on: "Rotary is on for jobs.") | a cylinder lying on two rollers (`rotary`) | `ui.rotary_setup` (new; keymap.md lists Ctrl+Shift+R for M9) | always (opens the Rotary attachment dialog) | checked while Use rotary is on | — | M9 |
| 2 | Print registration… ("Line the job up with a printed sheet using two targets"; second line "Also known as print-and-cut.") | a sheet with two target crosses (`registration`) | `ui.print_registration` (new; opens the Print registration wizard; the step chords Alt+0 … Alt+4 belong to its menu items, keymap.md) | always | checked while both targets are set and registration is in use | — | M9 |

- Buttons are `icon-big` squares, icon-only (Match [R16]); the checked state
  is our addition so the toolbar agrees with the Active setups status field
  (`layout.md`).
- Each button only opens a dialog or wizard; nothing here moves the machine
  or fires the laser. The setups' own steps that move the head follow the
  Machine panel's rules (`machine-panels.md`, `dialogs.md`).
- Deliberately omitted: the split-marking and cylinder-correction buttons
  (galvo, decision 1).

## Slide and nest toolbar

### Reference

- An optional toolbar at the far right of row 2 that pushes the selection
  left, right, up or down until it meets another object's outer edge or the
  bed edge; it is enabled from the Window menu when missing [R34][R31].
- Contents: four push buttons, then two stacked switches (move the
  selection as one group; keep nested parts with their container) and a
  labelled spacing field above a small spin box [R34].
- Version 2.1 adds a quick-nest button, an edge-margin field and a switch
  that stops nesting from turning parts [R35].
- The four push buttons are grey until something is selected; the spacing
  follows the numeric toolbar's units [R34].

### LaserCraft

Purpose: slide the selection until it touches something, and pack parts
close together on the bed. Default: hidden; right end of row 2
(`layout.md`). Milestone: later; sliding and nesting need a proposed change
to arrange.md (sliding is excluded by design there today). Tier: **Match**
(contents and order), **Own** (names, look, layout of the options).

| # | Button label (tooltip, our words) | Icon concept (`icons.md` name) | Action | Enabled when | Checked / active | Flyout members | Milestone |
|---|---|---|---|---|---|---|---|
| 1 | Slide left ("Move the selection left until it touches a shape or the bed edge (no chord)") | box with a motion arrow stopping at a wall on its left (`slide-left`) | `arrange.slide left` (new) | something is selected | — | — | Later |
| 2 | Slide right ("… right …") | the same, wall on the right (`slide-right`) | `arrange.slide right` (new) | something is selected | — | — | Later |
| 3 | Slide up ("… toward the top of the bed …") | the same, wall above (`slide-up`) | `arrange.slide up` (new) | something is selected | — | — | Later |
| 4 | Slide down ("… toward the bottom of the bed …") | the same, wall below (`slide-down`) | `arrange.slide down` (new) | something is selected | — | — | Later |
| — | — separator — | | | | | | |
| 5 | Nest ("Pack the selected parts closely on the bed to save material (no chord)") | several small shapes packed in a corner (`nest`) | `arrange.nest` (new) | two or more objects selected | — | — | Later |
| — | — separator — | | | | | | |
| 6 | Slide as one block (switch, label at its right; tooltip "Slide the whole selection together.") | — | sets an option the slide commands read (`arrange.slide` argument) | always | pill switch on/off (`visual-language.md`) | — | Later |
| 7 | Inner shapes follow outer (switch; tooltip "Shapes inside another keep their place relative to it.") | — | option of slide and nest | always | pill switch | — | Later |
| 8 | Keep rotation (switch; tooltip "Nesting does not turn parts.") | — | option of nest | always | pill switch | — | Later |
| — | — separator — | | | | | | |
| 9 | Gap (mm) (field; tooltip "Space left between parts. 0 or more.") | — | option of slide and nest | always | — | — | Later |
| 10 | Edge margin (mm) (field; tooltip "Space kept free along the bed edge. 0 or more.") | — | option of nest | always | — | — | Later |

- Layout: buttons 1–5 are `icon` squares on the upper line, centred in the
  two-line height; switches 6–8 stack in one column of two lines (6 over 7,
  then 8 at the top of the next column) and fields 9–10 stack in the last
  column, label right-aligned left of the field (density rule of
  `visual-language.md`). Own design.
- The options are UI preferences passed as command arguments, not project
  data (own design; the commands stay explicit and undoable).
- Fields follow the design units (mm by default, decision 14).

## Drawing tools

### Reference

- Purpose: select, create and edit objects on the canvas; it is the upper
  part of the left column, a single column of icon-only square buttons with
  no separators between them [R14][R31].
- Order in 2.1: select; draw lines; the stacked shape button (rectangle by
  default); edit nodes; trim; add tabs; text; position the laser; measure
  [R14][R17]. One capture lacks the laser-position pin, and older captures
  show rectangle, ellipse and polygon as three separate buttons instead of
  the stacked one [R14][R18][R19].
- The shape list holds rectangle, ellipse, triangle, pentagon, polygon,
  octagon, star and a two-radius star; the on-screen order of that list is
  not shown [R17].
- Esc returns to the select tool; it also ends line drawing and text entry
  [R14][R18][R30].
- After a shape is drawn it is selected and its handles work at once while
  the shape tool stays active; a settings switch can send the user back to
  select after each shape [R17].
- The laser-position tool sends the head to the clicked spot, needs a
  connection and a homed machine, and switches itself off after about ten
  seconds as a safeguard [R22].
- Tabs can only be placed on shapes of line-mode layers [R21]; trim works on
  plain paths, not groups [R24]; measure reads shapes and segments under the
  pointer or between two dragged points [R20].
- None of the creation tools is shown greyed in any capture [R14].

### LaserCraft

Purpose: choose the canvas tool. Default place: tool column, top. Tier:
**Match** (contents, order, the shape flyout, Esc to Select), **Own** (Move
head's one-shot rule, labels, icons, the look).

| # | Button label (tooltip, our words) | Icon concept (`icons.md` name) | Action | Enabled when | Checked / active | Flyout members | Milestone |
|---|---|---|---|---|---|---|---|
| 1 | Select ("Select tool (Esc)") | arrow pointer (`select`) | `ui.tool select` | always | active-tool look while current (the default tool) | — | Now |
| 2 | Line ("Line tool (Ctrl+L)"; second line "Click points; double-click or Enter to finish.") | slanted pen nib drawing a short stroke (`line`) | `ui.tool line` | always | active-tool look | — | Now |
| 3 | Shape tools (flyout; face default "Rectangle") | the face member's outline (`rect`, …) | flyout | always | active-tool look while any member is the current tool | Rectangle (`rect`, `ui.tool rect`, Ctrl+R, Now); Ellipse (`ellipse`, `ui.tool ellipse`, Ctrl+E, Now); — separator —; Triangle (`triangle`, `ui.tool triangle` (new)); Pentagon (`pentagon`, `ui.tool pentagon` (new)); Polygon (`polygon`, `ui.tool polygon` (new)); Octagon (`octagon`, `ui.tool octagon` (new)); Star (`star`, `ui.tool star` (new)); Alternating star (`alternating-star`, `ui.tool alternating_star` (new)); the last six later, no chords | Now (Rectangle, Ellipse); later (the rest) |
| 4 | Edit nodes ("Edit nodes tool (Ctrl+`)"; second line "Shows Node tools beside this column.") | path outline with small squares on its corners (`edit-nodes`) | `ui.tool nodes` | always | active-tool look; the Node tools column is shown | — | Now |
| 5 | Trim ("Trim tool: click a segment to cut it back to the next crossing (Ctrl+K once bound)") | two crossing lines, one end beyond the crossing dotted (`trim`) | `ui.tool trim` (new) | always (clicking a group or an unsuitable shape explains the refusal) | active-tool look | — | Later |
| 6 | Place tabs ("Place tabs tool: click a line-layer shape to add a tab (Ctrl+Tab once bound)") | outline with a short bridge gap (`place-tabs`) | `ui.tool tabs` (new) | always | active-tool look | — | Later |
| 7 | Text ("Text tool (Ctrl+T)"; second line "Click the bed to add text.") | a text cursor beside a few text lines (`text-tool`) | `ui.tool text` | always | active-tool look | — | Now |
| 8 | Move head ("Move head tool: click the bed to send the laser head there, laser off (Alt+L once bound)") | the head ring with a short arrow pointing at it (`move-head`) | `ui.tool move_head` (new) | connected, the machine idle (Ready) and homed since it connected | active-tool look | — | M6 |
| 9 | Measure ("Measure tool: hover or drag to read sizes (Alt+M once bound)") | a caliper (`measure`) | `ui.tool measure` (new) | always | active-tool look | — | Later |

Notes:

- Shape tools uses a flyout from the start, even while only Rectangle and
  Ellipse exist (Match [R17]); the separator keeps today's two shapes
  together above the later ones. Today the tool column shows Rectangle and
  Ellipse as separate text buttons (current-ui.md 1.5); see "Current state →
  target". New shape kinds need a proposed change to shapes.md. The order
  of the later shapes is own design (sides counted up, then stars).
- The Rectangle and Ellipse chords still pick those tools directly from any
  tool and update the flyout face (own design).
- Homing requirement (own design, after the reference's need for a homed
  machine [R22][R8]): Move head, To head position and every Go to selection
  row stay disabled until the machine has completed a homing cycle in the
  current connection, with the second line "Home the machine first." Without
  homing the reported position need not match the bed, so the work-area
  check of safety rule 5 could pass a move that is really outside the bed.
  How machines without homing switches are handled is an owner decision
  (Open questions).
- Move head (own design): disabled unless connected, Ready and homed; disabled
  again the moment a job, framing, homing or jog starts, and on an alarm or
  disconnect, after which the current tool falls back to Select. The click
  target is checked against the work area; outside it, nothing is sent and
  the Messages field says why (spirit of safety rule 5). The head moves
  laser off (safety rules 1 and 3) at the jog speed (`machine-panels.md`).
- The reference's ten-second auto-off for its laser-position tool [R22]
  becomes a stricter one-shot rule here (next section).
- Trim, Place tabs and Measure behaviours on the canvas are `canvas.md`'s
  (proposed changes there); Place tabs refuses shapes on fill or image
  layers in the Messages field (Match [R21]).

### Which tool returns to Select

| Tool | After a use | Esc |
|---|---|---|
| Select | — | stays Select (clears nothing by itself; Esc has no other meaning) |
| Line | stays Line after a finished line (canvas.md: a finished drawing selects the new shape and stays in the tool) | cancels an unfinished line (canvas.md); with no line in progress returns to Select |
| Shape tools (every member) | stays in the same shape tool; the new shape is selected and its handles work (canvas.md, Match [R17]) | returns to Select |
| Edit nodes | stays | with a node action armed: back to the node pointer (proposed, Node tools); otherwise returns to Select (nodes.md) |
| Text | stays Text after the Add text window closes (own design, matching canvas.md's drawing rule) | closes nothing while the window is open (the window's own Esc closes it); otherwise returns to Select |
| Trim, Place tabs, Measure | stay (several clicks are normal) | returns to Select |
| Move head | **returns to Select after one click** that sends a move (own design: one explicit click, one move; stricter than the reference's timed switch-off [R22]) | returns to Select |
| Pan (Standard toolbar) | stays | returns to Select (Match [R1]) |
| Start point (Shape operations) | stays | returns to Select (own design; the reference's Esc behaviour here is unknown) |
| Round corners (Shape operations) | stays | returns to Select (Match [R25]) |

- Commands run from any toolbar (booleans, align, arrays…) never change the
  current tool.
- A later preference "Return to Select after each shape" (own wording; the
  reference has an equivalent switch, off by default [R17]) would send Line,
  the shape tools and Text back to Select after each finished shape. It is
  off by default. Proposed change to canvas.md; Open questions.

## Node tools

### Reference

- While node editing is on, a second narrow strip with the same background
  and width appears directly right of the creation tools, between them and
  the ruler [R19].
- Its buttons are small line glyphs with small colour accents (green for
  adding, red for removing or breaking, cyan for smoothing); one button is
  the plain node pointer; hovering any button names its action; using one
  means clicking the button, then a node or line on the canvas [R19].
- The page lists the node actions only as a key table (delete node, insert,
  insert at the midpoint, make a corner, smooth, break, delete a line, make
  a line, make a curve, align by a segment, trim a line, extend a line); the
  strip's exact order and labels are not given and the capture is cut off
  [R19].

### LaserCraft

Purpose: one button per node action, for users who prefer clicking to the
node keys. Shown only while Edit nodes is the current tool, in its own
column right of the tool column (`layout.md`); never listed in Window →
Toolbars and not folded into Drawing tools. Tier: **Echo** (contextual
column, click-then-click use), **Own** (order, groups, labels, icons, the
armed state). Milestone: later (the six key actions exist today in
nodes.md; their buttons come with the toolbar; the other five actions are
later).

Use (own design, proposed change to nodes.md):

- The first button, Node pointer, is the normal node tool of nodes.md
  (drag nodes and handles, hover keys).
- Every other button **arms** its action: the button takes the active-tool
  look (the Node pointer loses it), the canvas cursor shows the action's
  badge (`canvas.md`), and the next click on a node or segment of a selected
  path applies the action there, exactly as its key would. The action stays
  armed for more clicks; Esc or the Node pointer button disarms it.
- The node keys (D I M L C B) keep working in every armed state and act on
  what is under the pointer.
- Buttons are enabled while at least one selected shape is a path; with
  only non-path shapes selected they are disabled with the second line
  "Convert the shape to a path first (Ctrl+Shift+C)."

| # | Button label (tooltip, our words) | Icon concept (`icons.md` name) | Action | Enabled when | Checked / active | Flyout members | Milestone |
|---|---|---|---|---|---|---|---|
| 1 | Node pointer ("Drag nodes and handles (Esc leaves Edit nodes)") | arrow pointer with a small square at its tip (`node-pointer`) | node tool, no action armed | always while Edit nodes is current | active-tool look when nothing is armed | — | Later |
| — | — separator — | | | | | | |
| 2 | Add node here ("Click a segment to add a node there (I)") | curve with a node and an `accent` plus (`node-add`) | arms `node.insert` at the clicked point | a path is selected | active-tool look while armed | — | Later (key: Now) |
| 3 | Add node at middle ("Click a segment to add a node at its middle (M)") | curve with a centred node and a plus (`node-add-middle`) | arms `node.insert … 0.5` | a path is selected | while armed | — | Later (key: Now) |
| 4 | Delete node ("Click a node or segment to remove it (D)") | curve with a node and a minus (`node-delete`) | arms `node.delete` (on a node) or `node.delete_segment` (on a segment) | a path is selected | while armed | — | Later (key: Now) |
| — | — separator — | | | | | | |
| 5 | Make straight ("Click a curved segment to straighten it (L)") | curve turning into a straight line (`node-straight`) | arms `node.to_line` | a path is selected | while armed | — | Later (key: Now) |
| 6 | Make curve ("Click a straight segment to make it a curve (C)") | straight line turning into a curve (`node-curve`) | arms `node.to_curve` | a path is selected | while armed | — | Later (key: Now) |
| 7 | Make node smooth ("Click a node to line up its handles (no chord)") | node with two aligned handles (`node-smooth`) | arms `node.smooth` (new) | a path is selected | while armed | — | Later |
| 8 | Make node sharp ("Click a node so its handles move separately (no chord)") | node with two handles at an angle (`node-sharp`) | arms `node.sharp` (new) | a path is selected | while armed | — | Later |
| — | — separator — | | | | | | |
| 9 | Break at node ("Click a node to open the path there (B)") | path with a gap at one node (`node-break`) | arms `node.break` | a path is selected | while armed | — | Later (key: Now) |
| 10 | Trim segment ("Click a segment to cut it back to the next crossing (no chord)") | segment ending in a small blade at a crossing (`segment-trim`) | arms `node.trim_segment` (new) | a path is selected | while armed | — | Later |
| 11 | Extend to crossing ("Click an open end to lengthen it to the next path (no chord)") | open end with a dashed extension to a line (`segment-extend`) | arms `node.extend` (new) | a path with an open end is selected | while armed | — | Later |
| — | — separator — | | | | | | |
| 12 | Level by segment ("Click a segment to turn the selection so it lies at 0°, 45° or 90° (no chord)") | slanted segment with a level mark (`segment-level`) | arms `node.level` (new) | a path is selected | while armed | — | Later |

- Order (own design): the plain pointer, then adding and removing, then
  segment and node types, then cutting and extending, then levelling. The
  reference's order is unknown [R19].
- Node keys are tool-local and not rebindable (nodes.md), so the tooltip
  shows the plain letter; S is not used because nodes.md gives smoothing no
  key.
- Icon accents: at most one `accent` detail per icon (decision 17), never
  green or red (decision 6 keeps those for machine state and danger); the
  reference's coloured marks are not copied.
- The column has no grip and no menu entry; if it is too short its last
  buttons fold into a More button at its bottom, like the tool column. It
  appears and disappears with the Edit nodes tool.

## Shape operations

### Reference

- Purpose: combine, alter and repeat shapes; it is the lower half of the
  left column, under its own dotted grip [R15][R31].
- Order in 2.1, in groups: offset; weld and a stacked boolean button
  (union, difference, intersection and a preview helper); grid array and
  circular array; set start point and the corner-rounding tool, with a
  "radius" caption and a small number field below the rounding button in
  the vertical layout (to its right when horizontal) [R15][R23][R25].
  Older captures show four separate weld and boolean buttons [R23].
- The offset and array buttons open dialogs [R27][R28][R29]; set start
  point and corner rounding are canvas modes, highlighted while active
  [R26][R25].
- Buttons that cannot act on the selection, and the radius caption and
  field, are drawn grey; the flyout marker greys with them [R15][R23][R26].
- Booleans need exactly two closed vector items; weld takes any number
  [R23].
- The boolean flyout opens to the right with its four rows [R23].

### LaserCraft

Purpose: the operations that make new geometry from the selection, plus the
start point and corner-rounding tools. Default place: tool column, below
Drawing tools (`layout.md`). Milestone: later for the toolbar (most actions
exist today in the Tools and Arrange menus). Tier: **Match** (contents,
groups, order, the field under the rounding tool), **Own** (labels, icons,
look).

| # | Button label (tooltip, our words) | Icon concept (`icons.md` name) | Action | Enabled when | Checked / active | Flyout members | Milestone |
|---|---|---|---|---|---|---|---|
| 1 | Offset… ("Outline the selection at a distance (Alt+O)"; second line "Opens the Offset window.") | shape with a parallel outline around it (`offset`) | `ui.offset` | something is selected (the command refuses open shapes, arrange.md) | — | — | Now |
| — | — separator — | | | | | | |
| 2 | Weld ("Merge the selected shapes into one outline (Ctrl+W)") | two overlapping shapes drawn as one outline (`weld`) | `path.weld` | something is selected | — | — | Now |
| 3 | Booleans (flyout; face default "Union") | the face member's icon | flyout | exactly two objects selected | — | Union (`union`, `path.union`, Alt++, Now); Subtract (`subtract`, `path.subtract`, Alt+-, Now); Intersect (`intersect`, `path.intersect`, no chord: Alt+* cannot be bound, keymap.md; Now); — separator —; Combine shapes… (`combine-shapes`, `ui.combine_shapes` (new), Ctrl+B once bound; later) | Now (first three); later (Combine shapes…) |
| — | — separator — | | | | | | |
| 4 | Grid array… ("Copy the selection in rows and columns (no chord)"; second line "Opens the Grid array window.") | 3×3 grid of small outlined squares (`grid-array`) | `ui.grid_array` | something is selected | — | — | Now |
| 5 | Circular array… ("Copy the selection around a center (no chord)"; second line "Opens the Circular array window.") | a centre dot and a dashed circle through four small diamonds, one solid (`circular-array`) | `ui.circular_array` | something is selected | — | — | Now |
| — | — separator — | | | | | | |
| 6 | Start point ("Start point tool: click a node to choose where its cut begins (no chord)") | a circle outline with a filled node dot and an arrowhead on the path just after it (`start-point`) | `ui.tool start_point` (new) | always (clicking a non-path explains the refusal) | active-tool look | — | Later |
| 7 | Round corners ("Round corners tool: click a corner to round it by the radius below (no chord)") | an L-shaped corner whose bend is a round arc, the old sharp corner dotted (`round-corners`) | `ui.tool round_corners` (new) | always | active-tool look | — | Later |
| 8 | Radius (mm) (field under button 7; tooltip "Radius used by Round corners. A negative value cuts the corner inward.") | — | sets the radius the tool passes to its command (UI preference, not project data) | while Round corners is current, or always (Open questions) | — | — | Later |

Notes:

- Union is the default face because it is the first boolean in our menus
  and keymap (own choice); the reference also opens its list with union
  [R23].
- Booleans' enabled rule follows arrange.md ("select exactly two objects");
  its tooltip's second line for the disabled state reads "Select exactly two
  closed objects.".
- Combine shapes… sits after a separator because it opens a window, while
  the three above it act at once (`naming.md` ellipsis rule).
- Radius (mm) layout: in the vertical column the label sits on its own line
  under the button in `type-small`, centred, and the field fills the column
  width below it; when the toolbar is laid out horizontally (stage b) the
  label and field sit right of the button on one line (Match [R15]). The
  field and its label turn `text-disabled` when the field is not usable.
- Start point and Round corners need canvas behaviour and commands that do
  not exist yet (proposed changes to canvas.md and planning.md for the start
  point, and to nodes.md or shapes.md for rounding).
- Not on this toolbar (menu-only, as in the reference): Convert to path,
  Close path, Join paths, Break apart, Split by shape.

## Simple layout and toolbars (later)

### Reference

- The reference's reduced interface forces the short arrange toolbar,
  collapses the move targets into their list, hides distribute and
  mirror-across-a-line, turns the align buttons into one button that opens
  an align helper, replaces the boolean buttons by one helper button, drops
  the add-tabs tool and hides set start point [R36].

### LaserCraft

Simple layout is later and optional (decision 11). `layout.md` owns its
exact set of hidden and replaced controls; this section only restates what
that set means for the toolbars, so the two files cannot drift. If it is
built:

- Align and distribute is forced off; its Window → Toolbars item is
  disabled while Simple layout is on (`layout.md`).
- Arrange:
  - buttons 6 to 8 (Align centers and the two Align flyouts) are replaced
    by one button, Align shapes… (tooltip "Line up the selected objects,
    with a live preview (no chord)"; icon `align-shapes`, a draft id until
    `icons.md` lists it; action `ui.align_shapes` (new); enabled when two
    or more objects are selected), which opens the Align shapes dialog
    (`naming.md`; its contents belong to `dialogs.md`);
  - Move on bed stays one flyout (button 13 already is one; nothing on the
    toolbar spells its targets out as separate buttons);
  - Mirror across a line and both Distribute flyouts are hidden;
  - Group, Ungroup, the two plain mirrors and Match width and height stay.
- Shape operations:
  - the Booleans flyout (Union, Subtract, Intersect) is replaced by one
    button, Combine shapes… (icon `combine-shapes`, action
    `ui.combine_shapes`, same tooltip and enabled rule as its flyout row);
  - Start point is removed; every other button, Round corners and its
    Radius (mm) field included, stays.
- Drawing tools: Place tabs is removed.
- Standard, Position and size, Text style, Job setups and Slide and nest are
  unchanged.
- The job bar, Stop and Pause are never hidden (`layout.md`).
- Leaving Simple layout restores every hidden button and swaps the
  replacement buttons back (`layout.md`: nothing is hidden for good).

Tier: **Match** (the idea of reduced toolbars and the replacement buttons
[R36]), **Own** (the exact set, owned by `layout.md`).

## egui implementation notes

Own design; no numbers.

- One small widget, a toolbar button, paints its own background from the
  state table, then the pictogram from the icons module (`icons.md`
  decides painted shapes versus our own SVG files), then the flyout marker.
  It returns egui's `Response`, so hover text, clicks and long presses come
  from egui (`Response::on_hover_text`, `clicked`, `secondary_clicked`, and a
  press-duration check through `is_pointer_button_down_on` with the input
  time).
- Toolbars are data: each toolbar is a list of entries (button, flyout,
  separator, field, switch) with label, icon name, invocation and an
  enabled-condition tag. The shell maps condition tags to view state; the
  same lists build the More popup, the Window → Toolbars list and tests.
- Flyouts and the More popup use egui's popup/menu machinery so menus and
  flyouts share one look (`menus.md`).
- Row 1 and row 2 are `Panel::top` panels created right after the menu bar
  panel; the tool column stays the `Panel::left("tools")` panel; Node tools
  is a second left panel created after it only while Edit nodes is current
  (`layout.md`, "egui implementation notes").
- The active-tool look replaces today's `selectable_label`
  (`crates/lasercraft-ui/src/shell.rs:537-554`).
- Tests (CLAUDE.md: a test for every command): a shell test per toolbar
  that clicking each button emits its invocation; that disabled conditions
  hold (Cut disabled with no selection, Booleans with one object); that a
  flyout's plain click runs its face and a choice changes the face; that
  Move head falls back to Select after one move (with the simulated
  device); and that a button whose action does not exist is not drawn.

## Current state → target

From `current-ui.md` (commit 151eb26); paths relative to the repo root.

- No toolbar band, no icons (current-ui.md 1.2, 3) → rows 1 and 2 with the
  toolbars above; icons per `icons.md`.
- Tool column: six text `selectable_label`s, Select, Line, Rectangle,
  Ellipse, Text, Edit Nodes (`crates/lasercraft-ui/src/shell.rs:17-24`,
  `537-554`) → Drawing tools in the order above, with Rectangle and Ellipse
  inside the Shape tools flyout and "Edit Nodes" written "Edit nodes"
  (sentence case, `naming.md`); Shape operations below it.
- Tool tooltips "‹label› tool (‹chord›)" with the chord printed as key
  names, e.g. "Ctrl+Backtick" (`shell.rs:540-551`, current-ui.md 1.4) →
  same form, chord shown with symbols (Ctrl+`).
- Active tool shown with egui's default selection style (`shell.rs:546`) →
  `accent-soft` background with an `accent` outline.
- Weld, Union, Subtract, Intersect, Offset… exist only in the Tools menu
  (`shell.rs:27-37`); Grid array… and Circular array… only in Arrange
  (`shell.rs:93-94`) → also on Shape operations.
- Group, Ungroup, Align ▸, Distribute ▸ and Move on bed ▸ exist only in the
  Arrange menu (`shell.rs:40-95`); Move on bed lists five targets though
  `arrange.move_to_bed` accepts nine (arrange.md) → Arrange toolbar with
  flyouts; the four edge targets added.
- Clipboard, undo, zoom and preview actions exist only in menus and keys →
  also on the Standard toolbar.
- Node actions exist only as keys in the node tool (nodes.md) → also as
  Node tools buttons.
- Menu enabled rules today are "has a selection" / "has a clipboard"
  (ui-shell.md) → the finer per-button conditions above (object counts),
  shared with the menus (`menus.md`).
- No Pan tool (pan is a gesture only, canvas.md) → a Pan tool on Standard.

## Proposed changes to other specs

- **Proposed change to ui-shell.md**: the Layout section gains the toolbar
  band and its toolbars (Standard, Arrange, Align and distribute, Position
  and size, Text style, Job setups, Slide and nest) and the tool column's
  Drawing tools, Node tools and Shape operations with their contents by
  reference to this file; "tool palette" becomes Drawing tools; enabled
  rules per action follow this file.
- **Proposed change to keymap.md**: list the new front-end actions so the
  keymap file can bind them (none bound by default): `ui.tool pan`,
  `ui.tool trim` (Ctrl+K later), `ui.tool tabs` (Ctrl+Tab later),
  `ui.tool move_head` (Alt+L, M6), `ui.tool measure` (Alt+M later),
  `ui.tool start_point`, `ui.tool round_corners`, `ui.tool triangle`,
  `pentagon`, `polygon`, `octagon`, `star`, `alternating_star`,
  `view.zoom_selection`, `ui.preferences`, `ui.edit_device_profile`,
  `ui.camera_capture` (Alt+C, M9), `ui.rotary_setup` (Ctrl+Shift+R, M9),
  `ui.print_registration`, `ui.combine_shapes` (Ctrl+B later),
  `ui.align_shapes` (Simple layout, later). Decide
  whether Ctrl+Shift+A means zoom to selection or frame the selection
  (Open questions).
- **Proposed change to canvas.md**: add the Pan tool (left drag pans until
  Esc or another tool), the Move head tool (one click, one laser-off move,
  then Select; refused outside the work area), the Trim, Place tabs,
  Measure, Start point and Round corners tools, the Text tool staying
  current after its window closes, the Esc table above, and the later
  "Return to Select after each shape" preference.
- **Proposed change to nodes.md**: the Node tools column, armed node
  actions (click a button, then a node or segment), Esc first disarming an
  action and then leaving the tool, and the new actions Make node smooth,
  Make node sharp, Trim segment, Extend to crossing and Level by segment
  with their commands (`node.smooth`, `node.sharp`, `node.trim_segment`,
  `node.extend`, `node.level`).
- **Proposed change to arrange.md**: `arrange.match_size <width|height>
  [keep] [ids]` (Match width and height; anchor object rules as align),
  `shape.mirror_line`, `arrange.move_to_head`, and sliding and nesting
  (`arrange.slide <left|right|up|down>` with its options, `arrange.nest`),
  which are excluded by design there today.
- **Proposed change to shapes.md**: the shape kinds triangle, pentagon,
  polygon, octagon, star and alternating star (as `naming.md` already
  proposes).
- **Proposed change to device.md** (M6): the head-moving commands behind
  Move head and Go to selection (`machine.goto_selection <point>`), laser
  off, refused outside the work area, while not Ready, and while the machine
  has not completed a homing cycle in the current connection (machines
  without homing: Open questions); `arrange.move_to_head` reads the reported
  position only under the same homing condition.
- **Proposed change to naming.md**: add the visible labels this file
  introduces that naming.md does not list yet, so they go through the naming
  review: the Arrange flyout labels Align on a vertical line, Align on a
  horizontal line, Distribute vertically and Distribute horizontally; the
  direct button Align centers; the distribute members Vertical spacing and
  Horizontal spacing; the Align and distribute button labels with a verb
  (Align left edges … Align bottom edges, Distribute vertical spacing …
  Distribute horizontal centers, Move to top left … Move to bed center, Move
  to head position); the Node tools button Node pointer; and the Slide and
  nest buttons Slide left, Slide right, Slide up and Slide down. Until
  naming.md lists them they are drafts.
- **Proposed change to icons.md**: add `node-pointer` (Node tools button
  1: an arrow pointer with a small square node at its tip; accent: the
  node) and `align-shapes` (the Simple layout Align shapes… button: the
  `align-centers` pair with three short list lines beside it, matching how
  `combine-shapes` marks a helper window; accent: none). Both are drafts
  here until icons.md lists them.
- **Proposed change to menus.md**: View → Zoom to selection is enabled
  when the project has at least one shape (not only when something is
  selected), because with nothing selected the command fits every shape;
  the toolbar button and the menu row then share one enabled rule.
- **Proposed change to planning.md**: per-shape start point and direction
  chosen with the Start point tool.
- **Proposed change to ui-shell.md** (paste): state that Paste from a
  toolbar button or menu pastes in place, from the keyboard at the pointer
  (today's behaviour, now written down), unless the owner prefers the view
  centre (Open questions).

## Open questions

1. Which milestone builds the toolbar band? M6 needs row 1 for the job bar;
   the other toolbars have no milestone yet.
2. Ctrl+Shift+A: zoom to selection (the reference's 2.1 Window menu) or
   frame the selection with the laser (keymap.md, legacy page, M6)? Until
   decided, Zoom to selection has no chord.
3. Should Ctrl+click on a Move on bed target send the head to that point of
   the selection, as the reference does [R8]? Proposed: no, for safety;
   the Go to selection rows cover it.
4. Align and distribute: independent of Arrange (both may be on) or a
   replacement that turns Arrange off? Same question as `layout.md`.
5. Paste from the toolbar: in place (today's code) or at the centre of the
   visible canvas? The reference pastes at the pointer [R9], which a
   toolbar click cannot do.
6. Move head: one click then Select (proposed), or stay armed with a
   timeout like the reference [R22]?
7. Is the "Return to Select after each shape" preference wanted?
8. Keyboard access to toolbars: Tab is bound to Select next, so egui's Tab
   focus order cannot reach toolbar buttons. Is an extra way (for example
   F10 to the menu bar, then arrow keys) wanted for accessibility, and
   should accesskit be enabled (a feature change in eframe)?
9. Icon names and concepts: `icons.md` owns them, and the concepts here
   follow it. Most ids in this file exist unchanged in `icons.md`, but not
   all: the Text tool's id is `text-tool` there (renamed from `text`,
   which is a color token; icon ids never reuse a token name), and
   `node-pointer` (Node tools button 1) and `align-shapes` (the Simple
   layout Align shapes… button) are not listed in `icons.md` yet (proposed
   change to icons.md below). Until icons.md lists them, those two ids are
   drafts.
10. The text-label transition before icons exist: acceptable, or should the
    toolbars wait for the icon set?
11. Radius (mm): always editable, or only while Round corners is current
    (the reference greys it when not usable [R15])?
12. Job setups: icon-only big buttons (proposed, as the reference) or big
    buttons with a short caption under the icon, which new users may find
    easier?
13. Capture camera image: keep it on Standard (as the reference) once the
    Camera panel has the same button, or drop it from Standard?
14. (Closed) Edit device profile… before M8: the button is shown from M6
    and opens the M6 subset of the Device profile dialog (work area, port,
    baud), as `menus.md` and `machine-panels.md` decide; M8 adds the other
    tabs.
15. Machines without homing switches: keep Move head, To head position and
    Go to selection disabled on them (proposed, safest), or allow them once
    the user confirms the head's position (for example after setting an
    origin), with a device-profile flag saying the machine has no homing?

## Sources

All LightBurn pages read 2026-10-21 through the research notes for this
file (top toolbars, left toolbars, main window layout, property bars; plus
the status bar and preferences notes for tooltips and icon size).

- [R1] https://docs.lightburnsoftware.com/2.1/Reference/UI/MainToolbar/ —
  file and view toolbar: place, the nineteen buttons and their groups,
  disabled states, pan mode and how it ends, camera button greyed without a
  camera, no button captions.
- [R2] https://docs.lightburnsoftware.com/2.1/Reference/UI/ArrangeToolbar/
  — compact arrange toolbar: place, thirteen buttons, the lists that open
  below, the horizontal-line align list, the move list's two blocks, the
  wide form's existence.
- [R3] https://docs.lightburnsoftware.com/2.1/Reference/Align/ — align
  actions and chords, the two align lists in the compact form, anchor rules,
  the wide form's align buttons.
- [R4] https://docs.lightburnsoftware.com/2.1/Reference/Distribute/ —
  distribute lists, packing being menu-only, the wide form's distribute
  group and its unreadable pair order.
- [R5] https://docs.lightburnsoftware.com/2.1/Reference/FlipMirror/ — the
  flip and mirror-across-a-line buttons.
- [R6] https://docs.lightburnsoftware.com/2.1/Reference/Grouping/ — group
  and ungroup buttons.
- [R7] https://docs.lightburnsoftware.com/2.1/Reference/MakeSameWidthHeight/
  — make-same buttons, anchor object, Shift-click keeps proportions.
- [R8] https://docs.lightburnsoftware.com/2.1/Reference/MoveSelectedObjects/
  — move targets, which are direct buttons in the wide form, the move list
  in both forms, Ctrl-click moves the laser, the head-position target's
  requirements.
- [R9] https://docs.lightburnsoftware.com/2.1/Reference/Clipboard/ —
  clipboard buttons and their order, paste at the pointer, menu-only paste
  in place and duplicate, disabled states.
- [R10] https://docs.lightburnsoftware.com/2.1/Reference/UndoRedo/ — undo
  and redo buttons and their disabled state; a 2.1 full-window capture of
  row 1.
- [R11] https://docs.lightburnsoftware.com/2.1/Reference/ZoomPan/ — pan
  mode, zoom buttons, zoom to the selection or to all objects.
- [R12] https://docs.lightburnsoftware.com/2.1/Reference/UI/WindowMenu/ —
  toolbar check entries, the wide arrange form unchecked in the capture, the zoom
  entries and their chords (including the zoom-to-selection chord).
- [R13] https://docs.lightburnsoftware.com/2.1/Reference/UI/ArrangeMenu/ —
  the arrange menu the toolbar draws from (draw order, locking, the slide
  submenu, nesting, arrays, automatic grouping, packing, move submenus).
- [R14] https://docs.lightburnsoftware.com/2.1/Reference/UI/CreationToolbar/
  — creation tools: place, order, icon-only buttons, active-tool background,
  stacked shape button, Esc to select, variant captures.
- [R15] https://docs.lightburnsoftware.com/2.1/Reference/UI/ModifiersToolbar/
  — modifier tools: place, groups, order, the radius caption and field,
  disabled looks.
- [R16] https://docs.lightburnsoftware.com/2.1/Reference/UI/ModesToolbar/ —
  special-mode buttons: place, large icon-only buttons, machine-type
  dependent contents.
- [R17] https://docs.lightburnsoftware.com/2.1/Reference/PrimaryShapes/ —
  shape list and its members, quick click versus hold, the face showing the
  last shape, staying in the tool, the return-to-select setting.
- [R18] https://docs.lightburnsoftware.com/2.1/Reference/DrawLines/ — line
  tool, Esc ends drawing, older tool order.
- [R19] https://docs.lightburnsoftware.com/2.1/Reference/EditNodes/ — node
  strip beside the creation tools, its use and accents, node actions and
  keys, active icon colour change.
- [R20] https://docs.lightburnsoftware.com/2.1/Reference/Measure/ — measure
  tool behaviour.
- [R21] https://docs.lightburnsoftware.com/2.1/Reference/AddTabs/ — tabs
  only on line-mode shapes.
- [R22] https://docs.lightburnsoftware.com/2.1/Reference/PositionLaser/ —
  laser-position tool: needs a connection and homing, timed switch-off.
- [R23] https://docs.lightburnsoftware.com/2.1/Reference/BooleanTools/ —
  weld and the boolean flyout, its rows and direction, face follows the
  last operation, requirements, disabled marker, older separate buttons.
- [R24] https://docs.lightburnsoftware.com/2.1/Reference/TrimShapes/ — trim
  tool, not on groups, active-tool background in captures.
- [R25] https://docs.lightburnsoftware.com/2.1/Reference/RadiusFillet/ —
  corner-rounding tool, highlighted while active, Esc exits, its field.
- [R26] https://docs.lightburnsoftware.com/2.1/Reference/SetStartPoint/ —
  start-point tool as a canvas mode, greyed radius field in captures.
- [R27] https://docs.lightburnsoftware.com/2.1/Reference/OffsetShapes/ —
  offset button opens a dialog.
- [R28] https://docs.lightburnsoftware.com/2.1/Reference/GridArray/ — grid
  array button opens a dialog.
- [R29] https://docs.lightburnsoftware.com/2.1/Reference/CircularArray/ —
  circular array button opens a dialog.
- [R30] https://docs.lightburnsoftware.com/2.1/Reference/Text/ — text tool,
  Esc finishes typing, the text options toolbar.
- [R31] https://docs.lightburnsoftware.com/2.1/Reference/UI/ — default
  window map: both toolbar rows, the left column, row 2's order.
- [R32] https://docs.lightburnsoftware.com/2.1/GetStarted/UITour/ — region
  tour: toolbar purposes, disabled grey, active-tool highlight, stacked
  button marker, tooltips with shortcuts and F1, numeric fields greyed.
- [R33] https://docs.lightburnsoftware.com/2.1/Guides/CustomizingTheLightBurnWindow/
  — grips, separators, overflow chevron, toolbar docking.
- [R34] https://docs.lightburnsoftware.com/2.1/Reference/Dock/ — object
  docking toolbar: place, buttons, switches, spacing field, greyed buttons.
- [R35] https://docs.lightburnsoftware.com/2.1/NewFeatures/NewIn2.1/ — wider
  stacked buttons, quick nest and its fields, units radio buttons, icon size
  choice, galvo split-marking button.
- [R36] https://docs.lightburnsoftware.com/2.1/Reference/BeginnerMode/ —
  what the reduced interface hides or simplifies in the toolbars.
- [R37] https://docs.lightburnsoftware.com/2.1/Reference/UI/Tooltips/ —
  tooltips on hover, some with shortcuts, F1 help; their look not shown.
- [R38] https://docs.lightburnsoftware.com/2.1/Reference/NumericEditsToolbar/
  — numeric position and size toolbar contents and place.
- [R39] https://docs.lightburnsoftware.com/2.1/Reference/SettingsPreferences/
  — toolbar icon size preference.

Existing specs and documents used: `docs/spec/ui-shell.md` (tool palette,
menu enabled rules, F12), `docs/spec/canvas.md` (tools, Esc, staying in the
tool after drawing, pan gestures, zoom limits), `docs/spec/keymap.md` and
`crates/lasercraft-ui/keymap/default.keymap` (every chord quoted; chords not
bound are marked "once bound" or "no chord"), `docs/spec/arrange.md`
(commands, object counts, anchor object), `docs/spec/nodes.md` (node keys
and commands), `docs/spec/shapes.md` (shape kinds, paste),
`docs/spec/text.md` (text tool window), `docs/spec/commands.md`,
`docs/spec/safety.md` (rules 1, 3, 5, 7), `docs/clean-room.md`, and the
style files `layout.md` and `naming.md`.

Own design: every label, tooltip and icon concept and name; the state
table and its tokens; focus returning to the canvas; the flyout marker's
look, right-click opening, the disabled-face rule, check column and
headings; separator handling when folding; the contents of the flyouts the reference does
not show; the distribute pair order; Match width's Shift argument; not
adopting Ctrl-click to move the head; Go to selection's checks; the
homing requirement for head moves; Move head's
one-shot rule and enabled conditions; the Esc table's choices for Text and
Node tools; the Node tools order, groups and armed actions; the Job setups
checked state; the Slide and nest option layout; the Simple layout effects;
the egui notes and tests.
