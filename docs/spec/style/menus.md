# Spec: style — menus

Milestones: M0–M5 (the File, Edit, Tools, Arrange and View menus that exist
today: order, separators, sentence case, check items, icon gutter, chord
display), M6 (Machine menu, Move head, Go to selection, head-position items),
M7 (image items and the image part of the canvas menu), M8 (tests, device
profiles, the device and materials-library menus), M9 (camera, rotary, print
registration), M10 (Help menu) and later items as marked in each table.

This file fixes the menu bar and every menu LaserCraft draws: the order of the
menu titles, every item of every menu in on-screen order with its separators,
submenus and check items, the chord each item shows, its icon concept, when it
is enabled, its milestone and the command it runs; then the context menus
(canvas, image, linked copies, layer row, layer list header, palette swatch,
panels, camera, saved artwork, device row, materials library, materials
entry, console log, font field), owning the full row tables of most and
pointing to the file that owns the rest; then how a menu looks and behaves. It does not define names: every label is quoted
from naming.md, which also owns tooltip and message wording rules. The tiers
(Match, Echo, Own), the scope and the binding style decisions are in
README.md; where regions sit is in layout.md; toolbar buttons that repeat menu
items are in toolbars.md; the layer list and its columns are in
layers-panel.md; the Machine, Jog and Console panels are in machine-panels.md;
right-click and drawing behavior on the canvas are in canvas.md; the dialogs
that menu items open are in dialogs.md, job-preview.md and later-windows.md;
color, type and size tokens are in visual-language.md; icon drawings are in
icons.md.

## How to read the tables

- **Item** is the exact label from naming.md, in sentence case. "…" ends an
  item that opens a dialog or window (naming.md, Ellipsis). "▸" marks a
  submenu; its rows follow directly, indented with "↳". "☐/☑" marks a check
  item (unchecked / checked by default); "◉" marks one choice of a set where
  exactly one is active. A row reading "——" is a separator. A row in
  *italics* is a non-clickable heading row.
- **Shortcut** is the chord `crates/lasercraft-ui/keymap/default.keymap`
  binds to the item's action today, written the way the menu must show it
  (naming.md, Chords in menus): Ctrl+=, Ctrl+-, Ctrl+\`, `.`, `,`, PgUp, PgDn,
  Del, Esc, Alt+←. On macOS Cmd replaces Ctrl and Option replaces Alt
  (keymap.md). "—" means no chord. A chord keymap.md lists for a later
  milestone is written with that milestone ("Ctrl+K (later)", "Ctrl+H (M6)").
  A chord the reference shows but keymap.md does not list is written
  "not bound". The menu never hard-codes a chord: it asks the keymap for the
  chord bound to the item's action, so a user keymap changes the menus too
  (keymap.md, "Menu items show the chord bound to their action").
- **Icon** is the id of LaserCraft's own icon in icons.md's icon table
  (README.md, icons decision). icons.md owns the concept and the drawing;
  this file only says which id a row shows. "none" leaves the icon gutter
  blank. An item that also has a toolbar button uses exactly the same icon
  as that button (toolbars.md).
- **Enabled when** is the condition under which the item can be clicked.
  Outside it the item is drawn disabled and stays visible. A row is left out
  instead of disabled in exactly three cases: rows of a milestone not yet
  shipped (see Menu look and behavior), the conditional blocks of context
  menus (image rows, linked copies, barcode; see Context menus, Shared
  rules), and Window and Panels menu rows for toolbars and panels the
  current build does not have.
- **Milestone**: *Now* = exists today (perhaps under another label or in
  another menu, see Current state → target); *M6*–*M10* as in CLAUDE.md;
  *Later* = no milestone yet; *owner decision* = needs the owner first.
- **Action** is the registry command or the `ui.*` / `view.*` front-end
  action the item invokes, exactly as `crates/lasercraft-ui/src/shell.rs`
  and `default.keymap` spell it today. "new command (Mn)" means the command
  does not exist yet and is named by the spec of that milestone. Per
  CLAUDE.md every item only invokes; nothing in a menu changes the project
  by itself.

Facts about the reference program come only from the style guide's research
notes, taken from the public LightBurn 2.1 documentation listed in Sources;
each is tagged with its source key. Where the notes say nothing, the text
says "own design" or the point is listed under Open questions.

## Menu bar

### Reference

- A single thin strip runs across the full window width directly under the
  title bar and above the first toolbar row; the titles are plain text,
  packed to the left, and the rest of the strip is empty [R1] [R2].
- The 2.1 Windows screenshots show nine titles: the file, edit, tools and
  arrange menus, a machine-tools menu, a web-tools menu that is new in 2.1,
  the window menu, a language menu and the help menu, in that order [R1]
  [R2] [R27]. A 2.1 macOS screenshot has no web-tools menu [R16]. Devices set
  up from a manufacturer's bundle add an extra vendor menu [R27].
- On macOS the menus sit in the system menu bar at the top of the screen,
  after the application-name menu [R1] [R16].
- On Windows several titles show an underlined access letter [R1] [R4]. The
  2.1 changelog says the menu access keys used to clash with shortcuts and
  that this was fixed [R27].
- An open title is drawn as a filled box (a saturated blue in 2.1 Windows
  screenshots, pale blue with a thin outline in older ones, a grey pill on
  macOS) [R4] [R12] [R21] [R22] [R23]. No disabled title was seen [R3]–[R11].
- Clicking a title opens its menu; hovering a submenu row opens the submenu
  beside it [R4]. Right-clicking the menu bar itself is not described.

### LaserCraft

Placement: the first band of the main window, full width, directly under the
OS title bar, drawn by egui inside the window on every platform, macOS
included (README.md, menus decision; layout.md). It is a little less than
one `row` tall (layout.md), holds `type-body` titles, and sits on `window`.

Titles, left to right:

| # | Title | Holds (our words) | Shown from | Tier |
|---|---|---|---|---|
| 1 | File | projects, import, G-code file, export, notes, quit | Now | Match |
| 2 | Edit | undo and history, clipboard and selection, path clean-up, selection helpers, image commands, preferences | Now | Match |
| 3 | Tools | canvas tools, then shape and image operations | Now | Match |
| 4 | Arrange | grouping, mirror and rotate, align and distribute, slide and move on bed, arrays, outline, drawing order, lock, select next/previous | Now | Match |
| 5 | Machine | connection, job control, framing, head moves, print registration, camera, rotary, tests, device profiles | M6 | Own (name), Match (position) |
| 6 | View | job preview, zoom, fills and curve display, snapping, side panels | Now | Own |
| 7 | Window | reset layout, check lists of toolbars and panels | Later (docking stage a) | Match |
| 8 | Help | about, license, quick reference, user guide, reporting a problem | M10 (About may come earlier) | Match |

- A title appears only once its menu has at least one built item. Today the
  bar reads File, Edit, Tools, Arrange, View.
- Machine takes the reference's fifth position, where its machine-tools
  menu sits [R1], so a switcher finds it in the same place. It holds more than
  that menu: LaserCraft also puts connection, job control, framing and head
  moves there (see Machine menu).
- View is LaserCraft's own menu (README.md, menus decision; ui-shell.md
  already has it). It sits immediately before Window because its items are
  the ones the reference keeps at the top of its window menu [R9]. Where the
  reference has its web-tools menu, LaserCraft has nothing (omitted, scope).
- A Language menu would go between Window and Help, as in the reference [R1];
  it is not planned while there are no translations (README.md, menus
  decision; naming.md open question 14).
- Titles are single capitalised words (naming.md), drawn in `text`,
  `type-body`, separated by `gap-wide`, with no icons.
- States (own design): idle titles have no background; the title under the
  pointer gets the `hover` background; the title of the open menu gets the
  `accent-soft` background with `text` text, so it reads as one family with
  the hovered menu row. Titles are never disabled.
- Interactions (egui's menu bar behavior, kept): a click opens the menu, a
  second click or Esc or a click elsewhere closes it; while one menu is open,
  moving the pointer over another title opens that one instead.
  Right-clicking the menu bar does nothing; the Panels menu opens from the
  places layout.md lists (see Context menus).
- No access-key underlines and no Alt+letter menu access (own design): the
  default keymap already uses many Alt+letter chords (Alt+P, Alt+O, Alt+B,
  Alt+J, Alt+V today; Alt+S, Alt+R, Alt+L, Alt+M, Alt+X, Alt+C later), the
  reference had to repair exactly this kind of clash [R27], and egui does not
  offer mnemonics. Keyboard access to the bar is an open question.
- Tier: Match (position, order of the shared titles), Own (Machine and View
  names, the look of the open title, no mnemonics).

## File menu

### Reference

The reference's file menu holds, in order: new; a separator; open a second
program window; recent projects and recent templates submenus; open; import;
project notes; a separator; save, save as, save as template, save the machine
file (its label names the device's file type) and export; a separator;
preferences-file and bundle submenus; a separator; two print commands; a
separator; save a processed image and save the camera background picture; a
separator; exit [R3] [R4] [R26]. Shortcuts shown include Ctrl+N, Ctrl+O,
Ctrl+I, Ctrl+Alt+N, Ctrl+S, Ctrl+Shift+S, Alt+Shift+L, Alt+X, Ctrl+P,
Ctrl+Shift+P and Ctrl+Q [R3]. Small icons sit on new, the second-window row,
open, import, the notes row, all four save rows, export, both print rows and
exit; the two recent submenus, the preferences and bundle submenus and the
two image rows have none [R3]. The recent-projects
submenu opens on hover and lists one row per file with its full path; the
list holds up to two dozen entries [R3] [R4]. The image and camera-picture
rows were disabled in the 2.1 screenshot [R3]. Preferences-file actions and
bundles live in this menu [R17] [R46]; printing goes to the system print
dialog [R45].

### LaserCraft

| Item | Shortcut | Icon | Enabled when | Milestone | Action |
|---|---|---|---|---|---|
| New | Ctrl+N | `new-project` | always (asks about unsaved changes, ui-shell.md) | Now | `ui.new` |
| —— | | | | | |
| New window | — | `new-window` | always | Later, optional | new front-end action (multi-viewport) |
| Recent projects ▸ | — | none | at least one recent project exists | Later | — (opens submenu) |
| ↳ ‹file name› (one row per project, newest first) | — | none | the file still exists (else disabled, tooltip gives the path) | Later | `ui.open` with the path |
| Recent templates ▸ | — | none | at least one recent template exists | Later | — (opens submenu) |
| ↳ ‹file name› (one row per template, newest first) | — | none | the file still exists | Later | new front-end action |
| Open… | Ctrl+O | `open-project` | always (asks about unsaved changes) | Now | `ui.open` |
| Import… | Ctrl+I | `import-art` | always | Now | `ui.import` |
| Project notes… | Ctrl+Alt+N | `project-notes` | always | Now | `ui.notes` |
| —— | | | | | |
| Save | Ctrl+S | `save-project` | always (asks for a path the first time) | Now | `ui.save` |
| Save as… | Ctrl+Shift+S | `save-as` | always | Now | `ui.save_as` |
| Save as template… | — | none | always | Later | new front-end action |
| Save G-code… | Alt+Shift+L | `save-gcode` | always; the bounds check reports problems when it runs | Now | `ui.export_gcode` |
| Export vector art… | Alt+X (later) | `export-art` | the project has shapes | Later | new command (later) |
| —— | | | | | |
| Export image as engraved… | — | `save-image` | exactly one image is selected | M7 | new command (M7) |
| Save camera image… | — | none | a camera picture exists | M9 | new command (M9) |
| —— | | | | | |
| Quit | Ctrl+Q | `quit` | always (asks about unsaved changes) | Now | `ui.quit` |

Notes:

- Grouping follows the reference (Match): starting work, then saving and
  writing files, then image outputs, then quitting [R3]. Differences: the
  preferences-file submenu moves to the Edit menu next to Preferences… (one
  place for everything about preferences, naming.md); bundles and printing are
  omitted (see Omitted items); the machine-file row always says "Save G-code…"
  because every profile is GRBL (README.md, scope).
- Recent projects ▸ rows show the file name in `text` and, after a `gap`,
  its folder in `text-muted`, so long paths stay readable; the full path is
  the row's tooltip (own design; the reference shows whole paths [R4]). The
  list keeps the ten most recent files (own design). There are no numbers in
  front of the rows: in the reference they serve as access keys [R4], which
  LaserCraft does not use. When the list is empty the parent row is disabled.
- Save G-code… stays in File as today and also becomes a Machine panel
  button at M6 (naming.md).
- Export image as engraved… and Save camera image… show only from their
  milestones; the separator above them appears with the first of them.
- Tier: Match (order and grouping), Own (labels, icons, recent-file rows).

## Edit menu

### Reference

The reference's edit menu holds, in order: undo (its label names the action
it would reverse), redo, an undo-history list (new in 2.1); a separator; select
all, invert selection, cut, copy, duplicate, paste, paste in place, delete; a
separator; eight path clean-up commands (convert to path, convert to bitmap,
close path, close with a tolerance, auto-join, optimize, delete duplicates,
reverse direction); a separator; seven selection helpers (open shapes, open
shapes on fill layers, all shapes of the current layer, flash the current
layer, contained shapes, smaller shapes, circles by size); a separator; an
image submenu; a separator; the settings window [R5] [R16] [R42]. Undo,
redo, the selection pair, the clipboard rows except paste in place, delete
and the settings row carry icons; the clean-up and helper rows do not [R5]. Shortcuts shown include Ctrl+Z,
Ctrl+Shift+Z, Ctrl+A, Ctrl+Shift+I, Ctrl+X, Ctrl+C, Ctrl+D, Ctrl+V, Alt+V,
Ctrl+Shift+C, Ctrl+Shift+B, Alt+J, Alt+Shift+O and Alt+D [R5]. Redo is
disabled when there is nothing to redo, and clean-up rows that do not suit the
selection are disabled [R5]. The image submenu (reload, replace, replace to
fit) only works with one ungrouped image selected [R18]. One page places the
settings window under the edit menu and another under the file menu; the
research notes guess at a platform difference, but the docs do not explain
it [R13].

### LaserCraft

| Item | Shortcut | Icon | Enabled when | Milestone | Action |
|---|---|---|---|---|---|
| Undo ‹action› | Ctrl+Z | `undo` | there is something to undo | Now | `edit.undo` |
| Redo | Ctrl+Shift+Z | `redo` | there is something to redo | Now | `edit.redo` |
| History… | — | `history` | at least one action is in the history | Later | new front-end action (later) |
| —— | | | | | |
| Select all | Ctrl+A | `select-all` | the project has shapes | Now | `select.all` |
| Invert selection | Ctrl+Shift+I | `invert-selection` | the project has shapes | Now | `select.invert` |
| Cut | Ctrl+X | `cut` | something is selected | Now | `edit.cut` |
| Copy | Ctrl+C | `copy` | something is selected | Now | `edit.copy` |
| Duplicate | Ctrl+D | `duplicate` | something is selected | Now | `shape.duplicate` |
| Paste | Ctrl+V | `paste` | LaserCraft's clipboard holds shapes | Now | `ui.paste` |
| Paste in place | Alt+V | `paste-in-place` | LaserCraft's clipboard holds shapes | Now | `edit.paste_in_place` |
| Delete | Del | `delete` | something is selected | Now | `shape.delete` |
| —— | | | | | |
| Convert to path | Ctrl+Shift+C | none | a selected shape is not already a path | Now (in Tools today) | `shape.to_path` |
| Convert to image… | Ctrl+Shift+B (M7) | none | something is selected | M7 | new command (M7) |
| Close path | — | none | a selected path is open | Now (in Tools today) | `path.close` |
| Close nearly closed paths… | — | none | a selected path is open | Later | new command (later) |
| Join paths | Alt+J | none | at least two open paths are selected | Now (in Tools today) | `path.join` |
| Simplify… | Alt+Shift+O (later) | none | something is selected | Later | new command (later) |
| Delete duplicates | Alt+D (later) | none | the project has shapes | Later | new command (later) |
| Reverse direction | — | none | something is selected | Later | new command (later) |
| —— | | | | | |
| Select open shapes | — | none | the project has shapes | Later | new command (later) |
| Select open shapes on fill layers | — | none | the project has shapes | Later | new command (later) |
| Select the current layer's shapes | — | none | the current layer has shapes | Later | `layer.select ‹current color›` |
| Highlight the current layer's shapes | — | none | the current layer has shapes | Later | new front-end action (later) |
| Add enclosed shapes to selection | — | none | something is selected | Later | new command (later) |
| Add smaller shapes to selection | — | none | something is selected | Later | new command (later) |
| Select circles… | — | none | the project has shapes | Later | new command (later) |
| —— | | | | | |
| Image ▸ | — | none | exactly one image is selected and it is not in a group | M7 | — (opens submenu) |
| ↳ Reload from file | — | none | as the parent; the image's file exists | M7 | new command (M7) |
| ↳ Replace… | — | none | as the parent | M7 | new command (M7) |
| ↳ Replace and fit… | — | none | as the parent | M7 | new command (M7) |
| —— | | | | | |
| Preferences… | — | `preferences` | always | Later | new front-end action (later) |
| Keyboard shortcuts… | — | `keyboard-shortcuts` | always | Later | new front-end action (later) |
| Restore preferences… | — | none | an automatic backup exists | Later | new front-end action (later) |
| Preferences backup ▸ | — | none | always | Later | — (opens submenu) |
| ↳ Export preferences… | — | none | always | Later | new front-end action (later) |
| ↳ Import preferences… | — | none | always | Later | new front-end action (later) |
| ↳ Open preferences folder | — | `open-project` | always | Later | new front-end action (later) |

Notes:

- Order and grouping follow the reference (Match) [R5]. The undo label names
  the action in readable words ("Undo move", naming.md), never a command id.
  Redo stays the plain label "Redo", as naming.md gives it; whether it should
  also name its action is an open question.
- The clean-up group moves Convert to path, Close path and Join paths here
  from today's Tools menu, where the reference keeps them [R5]. Break apart
  moves to Arrange (see there).
- The last group keeps everything about preferences together (naming.md):
  the reference splits it between this menu and a file-menu submenu [R5]
  [R17]. Preferences… is in Edit on every platform, macOS included (see macOS
  note).
- Highlight the current layer's shapes is a view action, not a command: it
  briefly marks the shapes on the canvas (canvas.md owns the look). Select the
  current layer's shapes uses the existing `layer.select` command with the
  current palette color.
- Enabled-when rules for the clean-up group: an item is enabled when the
  command could succeed on the current selection (see Proposed changes,
  commands.md); the conditions in the table are the target.
- Tier: Match (order, groups, which items exist), Own (labels, icons, the
  preferences group).

## Tools menu

### Reference

The reference's tools menu starts with the canvas tool modes (select, draw
lines, a shape-tools submenu, edit nodes, trim, add tabs, edit text, position
the laser, measure, create a barcode), continues with offset, weld, the three
boolean commands and a boolean helper window, has its only separator after
that boolean group, and ends with operations on the selection or on images
(cut shapes, adjust image, trace image, a batch trace marked experimental,
text on a path, image mask, crop, slot resizing, two warps, tangent circle)
[R6]. With nothing selected the tool modes stay enabled while the operations
are greyed; the batch trace stays enabled because it works on files [R6]. The
tool modes and the boolean rows carry icons; the later rows mostly do not
[R6]. Shortcuts shown include Ctrl+L, Ctrl+\`, Ctrl+K, Ctrl+Tab, Ctrl+T,
Alt+L, Alt+M, Alt+O, Ctrl+W, Alt++, Alt+-, Alt+*, Ctrl+B, Alt+Shift+C, Alt+I
and Alt+T, and the article and screenshot disagree on some of them [R6]. The
shape-tools submenu lists rectangle (Ctrl+R), ellipse (Ctrl+E), triangle,
pentagon, polygon, octagon, star and a two-radius star; its on-screen order is
not shown [R6] [R19].

### LaserCraft

| Item | Shortcut | Icon | Enabled when | Milestone | Action |
|---|---|---|---|---|---|
| ◉ Select | Esc | `select` | always | Now | `ui.tool select` |
| ◉ Line | Ctrl+L | `line` | always | Now | `ui.tool line` |
| Shape tools ▸ | — | `shape-tools` | always | Now | — (opens submenu) |
| ↳ ◉ Rectangle | Ctrl+R | `rect` | always | Now (top level today) | `ui.tool rect` |
| ↳ ◉ Ellipse | Ctrl+E | `ellipse` | always | Now (top level today) | `ui.tool ellipse` |
| ↳ ◉ Triangle | — | `triangle` | always | Later | `ui.tool` with a new tool (later) |
| ↳ ◉ Pentagon | — | `pentagon` | always | Later | `ui.tool` with a new tool (later) |
| ↳ ◉ Polygon | — | `polygon` | always | Later | `ui.tool` with a new tool (later) |
| ↳ ◉ Octagon | — | `octagon` | always | Later | `ui.tool` with a new tool (later) |
| ↳ ◉ Star | — | `star` | always | Later | `ui.tool` with a new tool (later) |
| ↳ ◉ Alternating star | — | `alternating-star` | always | Later | `ui.tool` with a new tool (later) |
| ◉ Edit nodes | Ctrl+\` | `edit-nodes` | always | Now | `ui.tool nodes` |
| ◉ Trim | Ctrl+K (later) | `trim` | always | Later | `ui.tool` with a new tool (later) |
| ◉ Place tabs | Ctrl+Tab (later) | `place-tabs` | always | Later | `ui.tool` with a new tool (later) |
| ◉ Text | Ctrl+T | `text-tool` | always | Now | `ui.tool text` |
| ◉ Move head | Alt+L (M6) | `move-head` | connected, the machine idle (Ready), and homed since it connected | M6 | `ui.tool` with a new tool (M6) |
| ◉ Measure | Alt+M (later) | `measure` | always | Later | `ui.tool` with a new tool (later) |
| Barcode… | — | `barcode` | always | Later | new front-end action (later) |
| —— | | | | | |
| Offset… | Alt+O | `offset` | something is selected | Now | `ui.offset` |
| Weld | Ctrl+W | `weld` | something is selected | Now | `path.weld` |
| Union | Alt++ | `union` | exactly two objects are selected | Now | `path.union` |
| Subtract | Alt+- | `subtract` | exactly two objects are selected | Now | `path.subtract` |
| Intersect | — (Alt+\* cannot be bound, keymap.md) | `intersect` | exactly two objects are selected | Now | `path.intersect` |
| Combine shapes… | Ctrl+B (later) | `combine-shapes` | exactly two objects are selected | Later | new front-end action (later) |
| —— | | | | | |
| Split by shape | Alt+Shift+C (later) | none | at least two objects are selected | Later | new command (later) |
| Image adjustments… | not bound | `image-adjust` | an image is selected | M7 | new front-end action (M7) |
| Trace to vectors… | not bound | `trace-image` | an image is selected | M7 | new front-end action (M7) |
| Trace several files… (experimental) | — | none | always | Later | new front-end action (later) |
| Put text on path | — | none | one text and one path are selected | Later | new command (later; text.md) |
| Clip image with shape | — | none | one image and one closed shape are selected | M7 | new command (M7) |
| Crop image | — | none | one image and one closed shape are selected | M7 | new command (M7) |
| Fit slots to thickness… | — | none | something is selected | Later | new front-end action (later) |
| Perspective warp | — | none | something is selected | Later | new command (later) |
| Mesh warp | — | none | something is selected | Later | new command (later) |
| Add tangent circle | — | none | at least three circles are in the project | Later | new command (later) |

Notes:

- The tool modes are one set where exactly one is active, so the active tool
  shows the choice dot in the icon gutter instead of its icon (own design;
  the reference shows no active mark in this menu). The same tool is
  highlighted in the Drawing tools toolbar (toolbars.md). A tool chosen from
  the Shape tools submenu marks both its own row and the submenu row.
- Order follows the reference (Match) [R6] with one own change: a separator
  between the tool modes and the operations, because LaserCraft's tool list
  ends with a dialog item (Barcode…) and the long menu reads better split in
  three. The reference's single separator after the boolean group is kept.
- Shape tools ▸ is the submenu the reference has [R6]; LaserCraft's on-screen
  order is the reference's article order [R19] with our labels (the
  reference's on-screen order is unknown). Until more shapes exist the
  submenu holds only Rectangle and Ellipse, which are top-level rows today.
- Trace several files… carries "(experimental)" in its label as naming.md
  requires, instead of the reference's flask icon and maturity marker [R6].
- Intersect shows no chord: keymap.md explains that egui cannot bind `*`.
- Image adjustments… and Trace to vectors… show "not bound": the reference
  page shows chords for them [R6] but keymap.md does not list any (see
  Proposed changes, keymap.md).
- Union, Subtract, Intersect and Combine shapes… follow arrange.md: exactly
  two objects, closed shapes; Subtract removes the object selected second.
- Tier: Match (order, the separator after the booleans, enabling pattern),
  Own (labels, icons, the extra separator, the active-tool dot).

## Arrange menu

### Reference

The reference's arrange menu, top to bottom: group, ungroup, auto-group; a
separator; flip horizontal and vertical, mirror across a line, rotate a quarter
turn each way, two-point rotate and scale; a separator; align and distribute
submenus; a separator; nesting through an online service, quick local
nesting, a submenu that slides the selection until it touches something, a
submenu that moves the selection to a bed position, a submenu that moves the
laser head to a point of the selection; a separator; grid array, circular
array, copy along a path; a separator; a tight outline around the selection
and break apart; a separator; four drawing-order rows; lock and unlock [R7]
[R40] [R41]. With nothing selected everything that acts on the selection is
greyed but the submenu rows, auto-group and the online nesting stay enabled
[R7]. Group, ungroup, the flips and rotations, quick nesting, the arrays and
the locks carry icons [R7]. Shortcuts shown include Ctrl+G, Ctrl+U,
Ctrl+Shift+H, Ctrl+Shift+V, Ctrl+Shift+M, `.`, `,`, Ctrl+2, Alt+B, PgUp,
PgDown, Ctrl+PgUp and Ctrl+PgDown [R7]. The align submenu's commands and
chords are listed on its own page (left, right, top, bottom, vertical and
horizontal centers, both centers) but the submenu is not pictured [R20]. The
distribute submenu has four even-spacing rows, a separator and two
move-together rows (Alt+Shift+H, Alt+Shift+V) [R21]. The slide submenu has
four directions with small icons and no chords [R22]. The bed-position submenu
lists the laser position, the page center, four corners and four edges in that
order, without chords; holding Ctrl while choosing moves the laser instead
[R23]. The head-to-selection submenu lists the selection's center, four
corners and four edges [R24].

### LaserCraft

| Item | Shortcut | Icon | Enabled when | Milestone | Action |
|---|---|---|---|---|---|
| Group | Ctrl+G | `group` | at least two objects are selected | Now | `group.create` |
| Ungroup | Ctrl+U | `ungroup` | a selected object is a group | Now | `group.ungroup` |
| Group nested shapes | — | none | the project has shapes | Later | new command (later) |
| —— | | | | | |
| Mirror horizontally | Ctrl+Shift+H | `mirror-horizontal` | something is selected | Now | `shape.flip horizontal` |
| Mirror vertically | Ctrl+Shift+V | `mirror-vertical` | something is selected | Now | `shape.flip vertical` |
| Mirror across a line | Ctrl+Shift+M (later) | `mirror-line` | something is selected | Later | new front-end action (later) |
| Rotate 90° clockwise | `.` | `rotate-cw` | something is selected | Now | `shape.rotate 90` |
| Rotate 90° counter-clockwise | `,` | `rotate-ccw` | something is selected | Now | `shape.rotate -90` |
| Rotate and scale by two points | Ctrl+2 (later) | `rotate-two-points` | something is selected | Later | new front-end action (later) |
| —— | | | | | |
| Align ▸ | — | none | at least two objects are selected | Now | — (opens submenu) |
| ↳ Left edges | Alt+← | `align-left` | as the parent | Now | `arrange.align left` |
| ↳ Right edges | Alt+→ | `align-right` | as the parent | Now | `arrange.align right` |
| ↳ Top edges | Alt+↑ | `align-top` | as the parent | Now | `arrange.align top` |
| ↳ Bottom edges | Alt+↓ | `align-bottom` | as the parent | Now | `arrange.align bottom` |
| ↳ Vertical centers | Alt+PgUp | `align-center-x` | as the parent | Now | `arrange.align center-x` |
| ↳ Horizontal centers | Alt+PgDn | `align-center-y` | as the parent | Now | `arrange.align center-y` |
| ↳ Centers | Alt+Home | `align-centers` | as the parent | Now | `arrange.align center` |
| Distribute ▸ | — | none | at least two objects are selected | Now | — (opens submenu) |
| ↳ Horizontal spacing | — | `distribute-h-spaced` | at least three objects are selected | Now | `arrange.distribute h-spaced` |
| ↳ Horizontal centers | — | `distribute-h-centered` | at least three objects are selected | Now | `arrange.distribute h-centered` |
| ↳ Vertical spacing | — | `distribute-v-spaced` | at least three objects are selected | Now | `arrange.distribute v-spaced` |
| ↳ Vertical centers | — | `distribute-v-centered` | at least three objects are selected | Now | `arrange.distribute v-centered` |
| ↳ —— | | | | | |
| ↳ Pack horizontally | Alt+Shift+H | `pack-horizontal` | at least two objects are selected | Now | `arrange.move_together horizontal` |
| ↳ Pack vertically | Alt+Shift+V | `pack-vertical` | at least two objects are selected | Now | `arrange.move_together vertical` |
| Match width | — | `match-width` | at least two objects are selected | Later | new command (later) |
| Match height | — | `match-height` | at least two objects are selected | Later | new command (later) |
| —— | | | | | |
| Nest | — | `nest` | at least one object is selected | Later | new command (later) |
| Slide ▸ | — | none | something is selected | Later | — (opens submenu) |
| ↳ Left | — | `slide-left` | as the parent | Later | new command (later; arrange.md) |
| ↳ Right | — | `slide-right` | as the parent | Later | new command (later) |
| ↳ Up | — | `slide-up` | as the parent | Later | new command (later) |
| ↳ Down | — | `slide-down` | as the parent | Later | new command (later) |
| Move on bed ▸ | — | none | something is selected | Now | — (opens submenu) |
| ↳ Center | — | `bed-center` | as the parent | Now | `arrange.move_to_bed center` |
| ↳ Top left | — | `bed-top-left` | as the parent | Now | `arrange.move_to_bed top-left` |
| ↳ Top right | — | `bed-top-right` | as the parent | Now | `arrange.move_to_bed top-right` |
| ↳ Bottom left | — | `bed-bottom-left` | as the parent | Now | `arrange.move_to_bed bottom-left` |
| ↳ Bottom right | — | `bed-bottom-right` | as the parent | Now | `arrange.move_to_bed bottom-right` |
| ↳ To head position | — | `bed-head` | something is selected, connected, the machine homed since it connected, and a position reported | M6 | `arrange.move_to_head` (new, M6) |
| ↳ Left | — | `bed-left` | as the parent | Later (the command exists; the row is missing today) | `arrange.move_to_bed left` |
| ↳ Right | — | `bed-right` | as the parent | Later | `arrange.move_to_bed right` |
| ↳ Top | — | `bed-top` | as the parent | Later | `arrange.move_to_bed top` |
| ↳ Bottom | — | `bed-bottom` | as the parent | Later | `arrange.move_to_bed bottom` |
| —— | | | | | |
| Grid array… | — | `grid-array` | something is selected | Now | `ui.grid_array` |
| Circular array… | — | `circular-array` | something is selected | Now | `ui.circular_array` |
| Path array… | — | `path-array` | one path and something to copy are selected | Later | new front-end action (later) |
| —— | | | | | |
| Outline around selection | — | `outline-selection` | something is selected | Later | new command (later) |
| Break apart | Alt+B | `break-apart` | something is selected | Now (in Tools today) | `path.break_apart` |
| —— | | | | | |
| Bring forward | PgUp | `bring-forward` | something is selected | Now | `shape.raise` |
| Send backward | PgDn | `send-backward` | something is selected | Now | `shape.lower` |
| Bring to front | Ctrl+PgUp | `bring-to-front` | something is selected | Now | `shape.to_front` |
| Send to back | Ctrl+PgDn | `send-to-back` | something is selected | Now | `shape.to_back` |
| Lock | — | `lock` | something unlocked is selected | Later | new command (later) |
| Unlock | — | `unlock` | something locked is selected | Later | new command (later) |
| —— | | | | | |
| Select next | Tab | none | the project has shapes | Now | `select.next` |
| Select previous | Shift+Tab | none | the project has shapes | Now | `select.previous` |

Notes:

- Order and groups follow the reference (Match) [R7]. LaserCraft
  differences: the head-to-selection submenu moves to the Machine menu as
  Go to selection ▸, because it moves the machine, not the design (the
  reference keeps it here [R7] [R24]); the online nesting is omitted and the
  local one is called Nest [R40] [R41]; Match width and Match height sit after
  Distribute ▸ because they are the menu form of toolbar buttons that sit with
  align and distribute (toolbars.md); Select next and Select previous get
  their own last group (they are LaserCraft's Tab gestures, keymap.md).
- Today's menu puts rotate and mirror first and the drawing order and
  select-next rows above Group; the target moves them as in the table.
- Move on bed ▸ uses the same rows in the same order as the Move on bed
  block of toolbars.md's flyout, which follows the order of the reference's
  toolbar list of these targets [R23]: the bed center, the four corners, the
  head position, then the four edges. The reference's own arrange submenu
  puts the head position first [R23]; we keep one order so that a flyout
  folded into the More popup (layout.md) and this submenu read the same. The
  edge rows are Later, as in naming.md and toolbars.md, although
  `arrange.move_to_bed` already takes `left`, `right`, `top` and `bottom`
  (arrange.md), so they can ship as soon as the owner wants them.
- To head position is enabled only when something is selected, a machine is
  connected, it has homed since it connected and it has reported a position
  (toolbars.md's homing rule). While it is disabled for want of homing its
  tooltip's second line reads "Home the machine first." "Top" is the rear of the bed, as
  on screen (canvas.md). The reference's Ctrl-while-choosing trick [R23] is
  not copied: moving the head is Go to selection ▸ in the Machine menu (own
  design: one gesture, one meaning).
- Align ▸ needs two objects and Distribute ▸ three (arrange.md); the parent
  of Distribute ▸ is enabled from two objects so the two Pack rows can be
  reached. Every align, distribute and pack row keeps the anchor object in
  place (arrange.md); the submenu tooltips say so in our words, for example
  "Line up the left edges with the anchor object".
- Slide ▸ is the reference's direction submenu with our label (naming.md);
  arrange.md excludes sliding today (see Proposed changes).
- Tier: Match (order, submenus, chords), Own (labels, icons, the moved items).

## Machine menu

### Reference

The reference's machine-tools menu for gantry devices holds, top to bottom:
a print-and-cut submenu; a separator; a camera list; a separator; rotary
setup (Ctrl+Shift+R); a separator; focus test, interval test, material test,
center finder and taper warp; a separator; the device list (Alt+Shift+D in the
screenshot), the active device's settings and the controller's settings [R8].
Its rows change with the device type, and galvo devices show other rows [R8]
[R47]. The focus test is disabled without a motorised Z axis [R8]. The
print-and-cut submenu has a guided wizard and four registration steps on
Alt+0 to Alt+4, then a separator and a reset row; the two target rows are
disabled until they make sense [R25]. A feeder setup row appears only for two
families of proprietary controllers [R25] [R43]. Connecting, starting,
pausing, stopping and framing are not in any menu; they live in the
reference's laser panel, whose device button reconnects when right-clicked
[R49]. The 2.1 changelog mentions a new shortcut for the device list that
differs from the menu screenshot [R27].

### LaserCraft

The Machine menu exists from M6. It gathers everything that talks to the
machine, so that every machine action is findable in one menu even when the
Machine panel is hidden (own design). Its lower part keeps the reference's
groups in the reference's order (Match).

| Item | Shortcut | Icon | Enabled when | Milestone | Action |
|---|---|---|---|---|---|
| Connect | — | `connect` | not connected and a port is chosen | M6 | new command (M6) |
| Disconnect | — | `disconnect` | connected and no job or framing is running | M6 | new command (M6) |
| —— | | | | | |
| Start | Alt+S (M6) | `start` | connected, ready, and the job has output | M6 | new command (M6) |
| Pause / Resume / Pause (motion only) | Pause (M6) | `pause` / `resume` / `pause-motion-only` | connected (always, safety.md rule 7) | M6 | `machine.pause` / `machine.resume` (new, M6; machine-panels.md) |
| Stop | Ctrl+Break (M6) | `stop` (`danger`) | connected (always, safety.md rule 7) | M6 | new command (M6) |
| —— | | | | | |
| Frame | Alt+R (M6) | `frame` | connected, ready, and the job has output | M6 | new command (M6) |
| Frame outline | Alt+Shift+R (M6) | `frame-outline` | connected, ready, and the job has output | M6 | new command (M6) |
| Frame at low power… | — | `frame-low-power` | connected, ready, and the job has output | Later, optional, owner decision | new front-end action |
| —— | | | | | |
| Home | Ctrl+H (M6) | `home` | connected and ready or in alarm (homing clears it, GRBL) | M6 | new command (M6) |
| Go to user origin | — | `go-user-origin` | connected, ready, and a user origin is set | M6 | new command (M6) |
| Set user origin | — | `set-user-origin` | connected and ready | M6 | new command (M6) |
| Clear user origin | — | `clear-user-origin` | a user origin is set | M6 | new command (M6) |
| Go to selection ▸ | — | none | something is selected, connected, the machine idle (Ready), and homed since it connected | M6 | — (opens submenu) |
| ↳ Center | — | `goto-center` | as the parent | M6 | new command (M6) |
| ↳ Top left | — | `goto-top-left` | as the parent | M6 | new command (M6) |
| ↳ Top right | — | `goto-top-right` | as the parent | M6 | new command (M6) |
| ↳ Bottom left | — | `goto-bottom-left` | as the parent | M6 | new command (M6) |
| ↳ Bottom right | — | `goto-bottom-right` | as the parent | M6 | new command (M6) |
| ↳ Top | — | `goto-top` | as the parent | M6 | new command (M6) |
| ↳ Bottom | — | `goto-bottom` | as the parent | M6 | new command (M6) |
| ↳ Left | — | `goto-left` | as the parent | M6 | new command (M6) |
| ↳ Right | — | `goto-right` | as the parent | M6 | new command (M6) |
| Clear alarm | — | none | connected and in alarm | M6, owner decision (naming.md question 10) | new command (M6) |
| —— | | | | | |
| Run G-code file… | — | `run-gcode` | connected and ready | Later | new front-end action (later) |
| —— | | | | | |
| Print registration ▸ | — | `registration` | connected | M9 | — (opens submenu) |
| ↳ Guided setup… | Alt+0 (M9) | none | connected | M9 | new front-end action (M9) |
| ↳ Set target 1 | Alt+1 (M9) | `set-target-1` | connected, ready, and a registration is in progress | M9 | new command (M9) |
| ↳ Set target 2 | Alt+2 (M9) | `set-target-2` | target 1 is set | M9 | new command (M9) |
| ↳ Fit job to targets | Alt+3 (M9) | none | both targets are set | M9 | new command (M9) |
| ↳ Place job on targets (no scaling) | Alt+4 (M9) | none | both targets are set | M9 | new command (M9) |
| ↳ —— | | | | | |
| ↳ Clear registration | — | none | a target is set | M9 | new command (M9) |
| —— | | | | | |
| Camera list… | — | `camera` | always | M9 | new front-end action (M9) |
| Capture camera image | Alt+C (M9) | `camera-capture` | a camera is set up | M9 | new command (M9) |
| —— | | | | | |
| Rotary attachment… | Ctrl+Shift+R (M9) | `rotary` | always | M9 | new front-end action (M9) |
| —— | | | | | |
| Focus height test… | — | `focus-test` | the device profile has a motorized Z axis | M8 | new front-end action (M8) |
| Line interval test… | — | `interval-test` | always | M8 | new front-end action (M8) |
| Speed and power test… | — | `power-test` | always | M8 | new front-end action (M8) |
| Find center… | — | `find-center` | connected | Later | new front-end action (later) |
| Taper correction… | — | `taper-correction` | rotary attachment is on | M9 | new front-end action (M9) |
| —— | | | | | |
| Device profiles… | not bound | `device-profiles` | always | M8 | new front-end action (M8) |
| Edit device profile… | — | `device-profile` | always | M6 (work area, port, baud), M8 (all tabs) | new front-end action (M6) |
| GRBL settings… | — | `grbl-settings` | connected and ready | Later | new front-end action (later) |

Notes:

- Why the top part exists (own design, differs from the reference):
  the reference has no menu rows for connecting, job control, framing or
  homing [R8] [R49]. LaserCraft adds them so a user who hid the Machine panel
  still finds every machine action by name. The menu rows are a convenience,
  never the safety control: the always-visible Job bar (README.md, safety
  decision; layout.md) is what safety.md rule 7 relies on, and it stays
  reachable while this menu is open (see Menu look and behavior).
- Pause row: one row, enabled in every connected state, whose label follows
  machine-panels.md's state table. It reads "Pause" in every connected state
  except the two below (Ready, Jogging, Homing and Alarm included; in Alarm
  it never reads Resume, safety.md rule 10), "Resume" (`resume` icon) while
  GRBL holds a job, and "Pause (motion only)" while a constant-power (M3)
  job runs, with the `pause-motion-only` icon and the tooltip "The laser may
  stay on" (naming.md, safety wording; safety.md rule 9). During an M3 job the Stop row is the highlighted
  action: when the Machine menu opens it is the pre-focused row, so it
  carries the `accent-soft` band and the `accent` focus ring and Enter
  stops at once (own design for safety.md rule 9's "highlighted"; a
  `danger-soft` band is not used because visual-language.md does not list
  menu rows among that token's uses).
- Stop row: `stop` is the only `danger` icon in the menus apart from
  `delete-stored` (README.md, icons decision; icons.md). It is enabled whenever a
  machine is connected, even while idle, because soft reset is always safe.
- Frame and Frame outline are laser-off (safety.md rule 2) and their
  tooltips say so ("Trace the job's bounding box with the laser off").
  Frame at low power… only exists if the owner approves it; it opens the
  Low-power frame dialog, which asks for and shows the power each time
  (dialogs.md; safety.md rule 2). There is no modifier on Frame that fires the
  laser (README.md, safety decision).
- Ctrl+Shift+A: keymap.md binds it at M6 to framing the selection (legacy
  source), which has no menu row yet (naming.md question 3); it is an open
  question here too.
- Go to selection ▸ uses the same rows in the same order as the Go to
  selection block of toolbars.md's flyout: center, the four corners, then
  Top, Bottom, Left, Right, as in the picture of the reference's
  head-to-selection submenu [R24]. naming.md's list of its rows gives the
  labels only, not an order. Its tooltip says the head moves with the laser
  off.
- Homing rule (from toolbars.md): Move head, every Go to selection row and
  To head position stay disabled until the machine has finished a homing
  cycle in the current connection, because before that the reported
  position need not match the bed and the work-area check (safety.md) could
  let a move outside the bed through. The disabled tooltip's second line
  reads "Home the machine first." Machines without homing switches are an
  owner decision (toolbars.md, Open questions).
- Device profiles…: the reference shows a chord on its device-list row, and
  its changelog names a different one [R8] [R27]; keymap.md binds neither, so
  the row shows "not bound" (Open questions).
- Feeder setup, galvo rows, the controller's file list and vendor rows are
  omitted (see Omitted items). The reference's camera-selection help is not in
  this menu either.
- Before connecting, only rows that need no connection are enabled: Connect,
  Clear user origin (when a user origin is set), Capture camera image (when
  a camera is set up), Camera list…, Rotary attachment…, the three test rows
  (Focus height test… only with a motorized Z axis), Taper correction… (when
  the rotary attachment is on) and the two device profile rows.
- Tier: Own (title, the top part), Match (the lower groups and their order,
  the print registration chords), Own (labels, icons).

## View menu

### Reference

The reference has no view menu. The items LaserCraft puts here sit at the top
of the reference's window menu: the job preview (Alt+P), zoom to page
(Ctrl+0), zoom in (Ctrl+=), zoom out (Ctrl+-) and zoom to the selection
(Ctrl+Shift+A); a separator; a greyed heading row followed by four exclusive
display styles (outline or filled, each coarse or smooth), a row that swaps
outline and filled (Alt+Shift+W in the screenshot, Alt+W in the text) and a
row that hides and restores the side-docked windows (F12) [R9] [R12] [R13].
The zoom rows and preview carry icons; the display styles show check boxes on
Windows and a bare tick on macOS [R9] [R12]. The display styles change only
the drawing, never the output [R12].

### LaserCraft

| Item | Shortcut | Icon | Enabled when | Milestone | Action |
|---|---|---|---|---|---|
| Job preview… | Alt+P | `job-preview` | always (works without a device) | Now | `ui.preview` |
| Fit bed | Ctrl+0 | `fit-bed` | always | Now | `view.zoom_fit` |
| Zoom in | Ctrl+= | `zoom-in` | always | Now | `view.zoom_in` |
| Zoom out | Ctrl+- | `zoom-out` | always | Now | `view.zoom_out` |
| Zoom to selection | — (owner decision on Ctrl+Shift+A) | `zoom-selection` | something is selected | Later | new front-end `view.*` action (later) |
| —— | | | | | |
| ☑ Show fills | Alt+Shift+W (later) | none | always | Later | new front-end `view.*` action (later) |
| ☑ Smooth curves | — | none | always | Later | new front-end `view.*` action (later) |
| —— | | | | | |
| ☑ Snap to grid | — | none | always | Later | new front-end action (later) |
| ☑ Snap to shapes | — | none | always | Later | new front-end action (later) |
| —— | | | | | |
| ☑ Side panels | F12 | none | always | Now (plain item today) | `ui.toggle_panels` |

Notes:

- Order follows the top of the reference's window menu (Match) [R9]: the
  preview first, then fitting, zooming and zooming to the selection; then the
  display settings; then the side-panel switch. Today's order (zoom in, zoom
  out, fit bed, side panels, job preview) changes to this order.
- Display settings: instead of four exclusive styles and a separate swap row
  [R9] [R12], LaserCraft has two independent check items (naming.md). Show
  fills on draws fill layers solid; off draws strokes only. Smooth curves on
  draws curves with more segments. Alt+Shift+W toggles Show fills once it is
  bound (keymap.md: later). Neither changes the job, the same rule as the
  reference [R12].
- Snap to grid and Snap to shapes are check items here so they can be turned
  off without opening Preferences… (own design; canvas.md owns snapping
  behavior; snapping is on by default and Ctrl suspends it while dragging,
  keymap.md).
- Side panels is a check item: ticked while the tool column and the right
  dock are shown. F12 hides both together (ui-shell.md). It never hides the
  Job bar (README.md, safety decision).
- Default states: Show fills off, so every shape stays pickable by its
  stroke as canvas.md requires; Smooth curves on (own choice: correctness
  over speed on today's hardware); both snaps on; Side panels on.
- Tier: Own (a separate View menu and its two check items), Match (contents
  and order taken from the reference's window menu, chords).

## Window menu

### Reference

The reference's window menu starts with a row that puts every toolbar and
window back where it was at installation (holding Shift while starting the
program does the same) [R9] [R15] [R14]. After the view rows (see View menu)
come two check lists separated by a rule: first every toolbar, then every
dockable window, each row ticked while that toolbar or window is shown;
clicking a row toggles it, and closing a window with its close button removes
its tick [R9] [R14]. The toolbar list reads: main, numeric fields, creation
tools, text options, modifiers, arrange, arrange (long form), color palette,
docking, modes [R9]. The window list is roughly alphabetical [R9]. The same
check list opens when right-clicking any toolbar or window title strip [R14].
The screenshot's ticks reflect one user's setup, not the defaults [R9].

### LaserCraft

The Window menu appears with docking stage a (README.md, docking decision;
layout.md). Until then View holds everything that can be shown or hidden.

| Item | Shortcut | Icon | Enabled when | Milestone | Action |
|---|---|---|---|---|---|
| Reset layout | — | none | always | Later | `ui.reset_layout` (front-end action, layout.md) |
| —— | | | | | |
| *Toolbars* | | | | Later | heading row |
| ☑ Standard | — | none | always | Later | `ui.show_toolbar standard` (proposed in layout.md) |
| ☑ Arrange | — | none | always | Later | as above, with the toolbar's name |
| ☐ Align and distribute | — | none | always | Later | as above |
| ☑ Position and size | — | none | always | Later | as above |
| ☑ Text style | — | none | always | Later | as above |
| ☑ Job setups | — | none | always | M9 | as above |
| ☐ Slide and nest | — | none | always | Later | as above |
| ☑ Drawing tools | — | none | always | Later | as above |
| ☑ Shape operations | — | none | always | Later | as above |
| ☑ Palette | — | none | always | Later | as above |
| —— | | | | | |
| *Panels* | | | | Later | heading row |
| ☐ Camera | — | none | always | M9 | `ui.show_panel camera` (proposed in layout.md) |
| ☑ Command line | — | none | always | Later | as above |
| ☑ Console | — | none | always | M6 | as above |
| ☑ Jog | — | none | always | M6 | as above |
| ☑ Layers | — | none | always | Later | as above |
| ☑ Machine | — | none | always | M6 | as above |
| ☑ Macros | — | none | always | Later | as above |
| ☑ Materials | — | none | always | M8 | as above |
| ☑ Properties | — | none | always | Later | as above |
| ☐ Saved artwork | — | none | always | Later | as above |
| ☐ Text data | — | none | always | Later | as above |

Notes:

- Reset layout restores layout.md's default arrangement: every default
  toolbar and panel shown, in its default dock and tab order, others hidden;
  preferences, device profiles and the project are untouched (Match [R15]).
  It runs the front-end action `ui.reset_layout`, which is not an undoable
  command. Holding a key at start-up is not copied; instead layout.md's
  start flag `lasercraft --reset-layout` starts with the default
  arrangement (own design, layout.md).
- Both lists and their order are fixed by layout.md ("Window menu check
  lists"): the toolbar list is spatial (top row, second row, left column,
  bottom), not the reference's order [R9]; the panel list is alphabetical by
  LaserCraft's names. naming.md maps each name to the reference's.
- The ticks shown are the defaults from layout.md's default arrangement:
  Align and distribute, Slide and nest, Saved artwork and Text data start
  hidden, and Camera stays hidden until a camera is set up (then it is
  shown).
- The two heading rows are drawn in `text-muted`, `type-small`, not
  clickable, never highlighted (Echo of the reference's greyed heading row
  [R9]). Each Panels row's tooltip says what the panel does (naming.md), for
  example Machine: "Laser connection and job control".
- Not listed, because they cannot be hidden: the Job bar (README.md, safety
  decision), the menu bar and the Status bar. Node tools is not listed: it
  shows itself while Edit nodes is active (toolbars.md).
- Rows appear only for toolbars and panels that exist in the current build;
  the heading row of an empty list is not drawn.
- The reference keeps zoom, display styles and the side-panel switch in this
  menu [R9]; LaserCraft keeps them in View (README.md, menus decision).
- Tier: Match (reset row, two check lists), Own (labels, heading rows,
  toolbar and panel order).

## Help menu

### Reference

The reference's help menu holds, top to bottom: about, the license text,
links to its forum and feature-request site, a quick help window (F1), the
online and printable documentation and video tutorials, help for a
third-party design program; a separator; a support-data generator and a
camera-selection helper; a separator; an update check; a separator; license
management; a separator; a debug log and developer tools [R10]. The quick
help and about rows open one tabbed window with shortcut, usage and about
tabs [R44]. No row was disabled in the screenshot [R10].

### LaserCraft

| Item | Shortcut | Icon | Enabled when | Milestone | Action |
|---|---|---|---|---|---|
| About LaserCraft… | — | `about` | always | M10 (may come earlier) | new front-end action |
| License… | — | `license` | always | M10 | new front-end action |
| Report a problem | — | `report-problem` | always | M10 | new front-end action (opens the browser) |
| —— | | | | | |
| Quick reference… | — | `quick-reference` | always | M10 | new front-end action |
| User guide | — | `user-guide` | always | M10 | new front-end action (opens the browser) |
| —— | | | | | |
| Copy system info | — | `copy-system-info` | always | Later | new front-end action |
| —— | | | | | |
| ☐ Debug log | — | none | always | Later | new front-end action |

Notes:

- Order follows the reference's first rows (Match) [R10]: about and license
  first, then the community link, then help windows and documentation. Vendor
  services, license management, the update check and developer tools are
  omitted (see Omitted items).
- About LaserCraft… opens the Quick reference window on its About tab, as the
  reference's about row opens its help window [R44] [R10].
- Quick reference… has no chord: F1 is context help for the item under the
  pointer (keymap.md, M10), not this window.
- Report a problem opens the project's issue tracker; Copy system info puts
  version, OS, display scale and serial details on the clipboard for a
  report, and confirms in the Status bar (own design).
- Debug log is a check item, ticked while logging (own design; whether the
  reference ticks it is unknown).
- Tier: Match (position, first groups), Own (labels, the support items).

## Context menus

### Shared rules (own design)

- A context menu opens on the secondary click (right-click; on macOS also
  Ctrl+click as egui reports it) at the pointer, using egui's context-menu
  popup and the same look as drop-down menus (see Menu look and behavior).
- Its rows are ordinary menu items: same labels, same chords in the
  shortcut column, same icons and the same enabling rules as the same item in
  the menu bar. A context menu never offers an action that has no menu-bar or
  panel equivalent.
- Rows that do not apply to what was clicked are left out of a context menu
  rather than disabled when they belong to a conditional block (image items,
  linked copies, barcode); the core rows stay and are disabled as in the menu
  bar (Echo of the reference, which adds blocks only when they apply [R33]
  [R34]).
- Like every menu, a context menu closes with Esc, a click outside, or after
  an item runs.

### Canvas menu

#### Reference

Right-clicking the canvas offers the clipboard commands, including paste in
place and duplicate [R28]. A screenshot of the menu over an image shows, top
to bottom: a submenu listing the windows; cut, copy, paste and duplicate with
icons and chords; delete; select all; group and ungroup; lock; convert to
path and convert to bitmap; the preview (monitor icon); then three image rows
(trace, adjust, save processed) and a row that shows the properties window,
with thin separators between clusters [R32]. A barcode adds an edit row after
the conversion rows when it is the only selected object [R34]; an original of
a virtual array adds a block of array rows after the conversion rows and
before the preview [R33]. The preview is also reachable here [R36], locking
has a pair of rows [R37], and the art-library page mentions two rows that
store the selection in an art library [R30]. A macOS capture shows a light
grey rounded menu with a bright blue hover bar [R33].

#### LaserCraft (Later)

Right-clicking an unselected object first selects it (as a plain click
would); right-clicking a selected object or empty canvas keeps the selection
(own design). While a shape is being drawn, right-click opens no menu (see
Open questions about finishing a line, as the reference does [R50]).

| Item | Shortcut | Icon | Shown / enabled when | Milestone | Action |
|---|---|---|---|---|---|
| Window ▸ | — | none | shown when the Window menu exists; same rows as Window → Toolbars and Panels | Later | — (opens submenu) |
| —— | | | | | |
| Cut | Ctrl+X | `cut` | enabled with a selection | Later | `edit.cut` |
| Copy | Ctrl+C | `copy` | enabled with a selection | Later | `edit.copy` |
| Paste | Ctrl+V | `paste` | enabled when the clipboard holds shapes; pastes at the right-click point | Later | `ui.paste` |
| Paste in place | Alt+V | `paste-in-place` | enabled when the clipboard holds shapes | Later | `edit.paste_in_place` |
| Duplicate | Ctrl+D | `duplicate` | enabled with a selection | Later | `shape.duplicate` |
| Delete | Del | `delete` | enabled with a selection | Later | `shape.delete` |
| Select all | Ctrl+A | `select-all` | enabled when the project has shapes | Later | `select.all` |
| —— | | | | | |
| Group | Ctrl+G | `group` | enabled with two or more objects | Later | `group.create` |
| Ungroup | Ctrl+U | `ungroup` | enabled when a selected object is a group | Later | `group.ungroup` |
| Lock | — | `lock` | enabled with an unlocked selection | Later | new command (later) |
| Unlock | — | `unlock` | enabled with a locked selection | Later | new command (later) |
| —— | | | | | |
| Convert to path | Ctrl+Shift+C | none | enabled as in Edit | Later | `shape.to_path` |
| Convert to image… | Ctrl+Shift+B (M7) | none | enabled with a selection | M7 | new command (M7) |
| Edit barcode… | — | `barcode` | shown only when exactly one barcode and nothing else is selected | Later | new front-end action (later) |
| *(linked copies block, see below)* | | | shown only for an original with linked copies | Later | |
| —— | | | | | |
| Job preview… | Alt+P | `job-preview` | always enabled | Later | `ui.preview` |
| *(image block, see Image menu)* | | | shown only when an image is selected | M7 | |

Notes:

- Order follows the reference's capture [R32] (Match): window list,
  clipboard, grouping and locking, conversion with its conditional rows, the
  preview, then image rows. Differences: the reference's row that shows the
  properties window is reached through Window ▸ (the Properties panel row);
  the art-library rows wait for an owner decision on their labels (Open
  questions).
- Paste here places the clipboard at the right-click point, which is what
  `ui.paste` does with the pointer position today (ui-shell.md, keymap.md).
- Tier: Echo (the idea and order of the canvas menu), Own (labels, icons,
  selection rule).

### Image menu (block in the canvas menu)

#### Reference

Right-clicking an image shows trace (Alt+T), adjust (Alt+I) and save
processed rows after the preview row of the canvas menu [R32].

#### LaserCraft (M7)

| Item | Shortcut | Icon | Shown / enabled when | Milestone | Action |
|---|---|---|---|---|---|
| —— | | | shown with the block | M7 | |
| Image adjustments… | not bound | `image-adjust` | shown when exactly one image is selected | M7 | new front-end action (M7) |
| Trace to vectors… | not bound | `trace-image` | as above | M7 | new front-end action (M7) |
| Export image as engraved… | — | `save-image` | as above | M7 | new command (M7) |

Tier: Echo (block and order), Own (labels).

### Linked copies block (in the canvas menu)

#### Reference

For an original that belongs to a virtual array, the canvas menu gains rows
that reopen the array settings, duplicate the whole array, turn the clones
into ordinary copies, take the selected originals out, add other selected
objects (shown only when the selection mixes array and non-array objects),
and remove the array while keeping the originals [R33]. A macOS capture shows
five of these rows [R33].

#### LaserCraft (Later)

| Item | Shortcut | Icon | Shown / enabled when | Milestone | Action |
|---|---|---|---|---|---|
| Edit linked copies… | — | none | shown when the right-clicked object is an original with linked copies | Later | new front-end action (later) |
| Duplicate with linked copies | — | none | as above | Later | new command (later) |
| Unlink copies | — | none | as above | Later | new command (later) |
| Take out of linked copies | — | none | as above, with several originals selected | Later | new command (later) |
| Add to linked copies | — | none | shown only when the selection mixes originals with linked copies and other objects | Later | new command (later) |
| Remove linked copies | — | `delete` | as the first row | Later | new command (later) |

Notes: arrange.md excludes linked copies today (see Proposed changes).
Remove linked copies deletes the copies and keeps the original; because Undo
brings them back it uses the ordinary `delete` icon in normal strokes, not
`danger` (icons.md: only `stop` and `delete-stored` are `danger`). Tier: Echo
(block, order), Own (labels).

### Layer menu (right-click on a layer row)

#### Reference

Right-clicking a layer row offers, in order: turn this layer's output off,
turn every other layer's output off, hide this layer, hide every other layer,
select all shapes on the layer, and flash the layer's shapes; Shift with a
right-click flashes them directly [R29]. Separators and icons are not shown
[R29].

#### LaserCraft (Later)

| Item | Shortcut | Icon | Enabled when | Milestone | Action |
|---|---|---|---|---|---|
| Output only this layer | — | none | the project has more than one used layer | Later | `edit.batch` of `layer.set ‹c› output on` and `layer.set ‹other› output off` for every other used layer |
| Hide other layers | — | none | the project has more than one used layer | Later | `edit.batch` of `layer.set ‹c› show on` and `layer.set ‹other› show off` |
| —— | | | | | |
| Select shapes | — | `select-all` | the layer has shapes | Later | `layer.select ‹c›` |
| Highlight shapes | — | none | the layer has shapes and Show is on | Later | new front-end action (later) |

Notes:

- The reference's single-layer rows (output off, hide this layer) are left
  out: the row's own Output and Show switches already do that with one click
  (layers-panel.md); LaserCraft keeps the "only this one" rows, which save
  many clicks (naming.md).
- Each row that changes several layers is one `edit.batch`, so one undo step
  restores every switch (commands.md).
- Shift+click on a layer name already selects its shapes (ui-shell.md); the
  reference's Shift+right-click flash is not copied (own design: Highlight
  shapes is in the menu).
- The separator between switch rows and shape rows is own design (unknown in
  the reference).
- Tier: Echo (idea and the shown rows), Own (labels, which rows exist).

### All layers menu (right-click on the layer list header)

#### Reference

Right-clicking the column headers offers three triples (output on for all, off
for all, inverted; shown for all, hidden for all, inverted; air on for all,
off for all, inverted) and a row that reorders the layers so the weaker
operations run first and line layers run last [R29].

#### LaserCraft (Later; needs column headers, layers-panel.md)

| Item | Shortcut | Icon | Enabled when | Milestone | Action |
|---|---|---|---|---|---|
| Output all layers | — | none | the project has used layers | Later | `edit.batch` of `layer.set ‹c› output on` for every used layer |
| Output no layers | — | none | as above | Later | `edit.batch` of `layer.set ‹c› output off` |
| Invert output | — | none | as above | Later | `edit.batch` flipping `output` on every used layer |
| —— | | | | | |
| Show all layers | — | none | as above | Later | `edit.batch` of `layer.set ‹c› show on` |
| Hide all layers | — | none | as above | Later | `edit.batch` of `layer.set ‹c› show off` |
| Invert shown | — | none | as above | Later | `edit.batch` flipping `show` |
| —— | | | | | |
| Air on for all layers | — | none | as above | Later | `edit.batch` of `layer.set ‹c› air on` |
| Air off for all layers | — | none | as above | Later | `edit.batch` of `layer.set ‹c› air off` |
| Invert air | — | none | as above | Later | `edit.batch` flipping `air` |
| —— | | | | | |
| Put line layers last | — | none | at least two used layers | Later | new command (later) that computes the order and runs `layer.set_order` |

Notes: the separators between the triples are own design. Put line layers
last must be a command, not UI code, because it computes an order from layer
settings (CLAUDE.md: the UI contains no business logic). Tier: Echo (rows and
order), Own (labels, separators).

### Swatch menu (right-click on a palette swatch)

#### Reference

The research notes for this file do not describe a right-click menu on the
reference's color strip.

#### LaserCraft (Later)

layers-panel.md (palette strip, Swatch menu) owns this menu, its labels
(proposed additions to naming.md) and its effects; the row table is repeated
here so every menu is listed in one place.

| Item | Shortcut | Icon | Enabled when | Milestone | Action |
|---|---|---|---|---|---|
| Use for new shapes | — | none | always | Later | none (UI state: the current color) |
| —— | | | | | |
| Select shapes | — | `select-all` | the layer has shapes | Later | `layer.select ‹c›` |
| Highlight shapes | — | none | the layer has shapes and Show is on | Later | the Layer menu's front-end action |
| —— | | | | | |
| Layer settings… | — | none | the layer has shapes (with the window) | Later | opens Layer settings on this layer |

Notes: Select shapes and Highlight shapes are the Layer menu's rows, with the
same labels, icon and action (layers-panel.md). If layers-panel.md changes
this menu, that file wins. Tier: Own.

### Panels menu (right-click on a title strip, a tab or a toolbar area)

#### Reference

Right-clicking any toolbar or any window's title strip opens the same check
list as the window menu [R14]; its exact contents are not shown [R14].

#### LaserCraft (Later, with the Window menu)

layout.md ("Panels menu") owns the triggers and the contents; this is the
row table. It opens on a right-click on a title strip, a tab, the empty part
of a toolbar row, a toolbar's grip or separator, the palette strip's empty
end, or the tool column's empty part (layout.md). The menu bar is not a
trigger.

| Item | Shortcut | Icon | Enabled when | Milestone | Action |
|---|---|---|---|---|---|
| *Toolbars* and its check rows | — | none | as in the Window menu | Later | as in the Window menu |
| —— | | | | | |
| *Panels* and its check rows | — | none | as in the Window menu | Later | as in the Window menu |
| —— | | | | | |
| ☑ Side panels | F12 | none | always | Later | `ui.toggle_panels` |
| Reset layout | — | none | always | Later | `ui.reset_layout` |

Notes: the check rows are exactly the Window menu's rows in the same order;
Side panels is the same check item as View → Side panels; Reset layout is
put last here (layout.md) so the check lists start at the pointer. Tier:
Match (same check list), Own (the last two rows and their placement).

### Camera menu (right-click in the Camera panel)

#### Reference

Right-clicking a camera's live picture opens a menu headed by the camera's
name in bold, then edit, remove and export rows with icons; right-clicking
elsewhere in the panel offers a camera-selection helper, add a camera and
import a camera [R31].

#### LaserCraft (M9)

On a camera's picture:

| Item | Shortcut | Icon | Enabled when | Milestone | Action |
|---|---|---|---|---|---|
| *‹camera name›* | | | | M9 | heading row |
| —— | | | | | |
| Edit… | — | `edit` | always | M9 | new front-end action (M9) |
| Remove | — | `delete-stored` (`danger`) | always | M9 | new command (M9) |
| Export… | — | `export-art` | always | M9 | new front-end action (M9) |

Elsewhere in the panel:

| Item | Shortcut | Icon | Enabled when | Milestone | Action |
|---|---|---|---|---|---|
| Add camera… | — | `camera-add` | always | M9 | new front-end action (M9) |
| Import camera… | — | `import-art` | always | M9 | new front-end action (M9) |

Notes: the heading row uses the same `text-muted`, `type-small` heading style
as the Window menu (own design; the reference sets it in bold [R31], and
LaserCraft keeps bold for dialog headings only, visual-language.md). The
camera-selection helper is omitted (vendor product chooser). Tier: Match
(rows and order), Own (labels, icons, heading style).

### Saved artwork menu (right-click in the Saved artwork panel)

#### Reference

Right-clicking the thumbnail area offers adding the selection, renaming and,
after a separator, deleting; rename and delete act on the item under the
pointer and are disabled on empty space [R30].

#### LaserCraft (Later)

| Item | Shortcut | Icon | Enabled when | Milestone | Action |
|---|---|---|---|---|---|
| Add selection | — | `add` | something is selected on the canvas | Later | new command (later) |
| Rename | — | none | the pointer is on an item | Later | new front-end action (later) |
| —— | | | | | |
| Delete | — | `delete` | the pointer is on an item | Later | new command (later) |

Tier: Match (rows, order, separator), Own (labels, icons).

### Device menu (right-click on a row in Device profiles)

#### Reference

Right-clicking an entry in the device list opens a plain menu without icons:
duplicate, a custom-icon row, rename, a separator, then sort by name and sort
by type [R35].

#### LaserCraft (M8)

| Item | Shortcut | Icon | Enabled when | Milestone | Action |
|---|---|---|---|---|---|
| Duplicate | — | none | a profile is under the pointer | M8 | new front-end action (M8): opens New device profile prefilled |
| Rename | — | none | a profile is under the pointer | M8 | new command (M8) |
| —— | | | | | |
| Sort by name | — | none | at least two profiles | M8 | new front-end action (M8) |

Notes: the custom-icon row and sort by type are omitted (naming.md: every
profile is GRBL, and device logos are vendor material). Tier: Echo (rows,
order, plain look), Own (labels).

### Library menu (Materials panel button)

#### Reference

A button at the bottom of the reference's material library opens a small
popup with icons: load, save, save as, create new and merge a library [R48].

#### LaserCraft (M8)

A popup menu opened by the Library ▸ button of the Materials panel
(later-windows.md places the button, Materials panel bottom row):

| Item | Shortcut | Icon | Enabled when | Milestone | Action |
|---|---|---|---|---|---|
| Open library… | — | `open-project` | always | M8 | new front-end action (M8) |
| Save library | — | `save-project` | the open library has unsaved changes | M8 | new command (M8) |
| Save library as… | — | `save-as` | a library is open | M8 | new front-end action (M8) |
| New library | — | `new-project` | always | M8 | new command (M8) |
| Merge library… | — | `merge-library` | a library is open | M8 | new front-end action (M8) |

Notes: order follows the reference [R48]; icons reuse the File menu's ids
for the same ideas (own design, icons.md). Tier: Match (rows, order), Own
(labels, icons).

### Material entry menu (right-click on a Materials entry)

#### Reference

The research notes for this file do not describe one.

#### LaserCraft (M8)

later-windows.md (Materials panel) defines it as a menu holding the same
actions as the panel's button column, in the same order, and leaves its
adoption here open; menus.md adopts it with this table. Labels, enabling and
effects are later-windows.md's button column.

| Item | Shortcut | Icon | Enabled when | Milestone | Action |
|---|---|---|---|---|---|
| Apply to layer | — | none | an entry and a target layer exist | M8 | as the button (later-windows.md) |
| Follow this entry | — | none | an entry and a target layer exist | M8 | as the button |
| —— | | | | | |
| Save layer as material… | — | none | a target layer exists | M8 | as the button |
| Edit settings… | — | `edit` | an entry is selected | M8 | as the button |
| Update from layer | — | none | an entry and a target layer exist | M8 | as the button |
| Edit description… | — | `edit` | an entry is selected | M8 | as the button |
| —— | | | | | |
| Duplicate | — | none | an entry is selected | M8 | as the button |
| Delete | — | `delete-stored` (`danger`) | an entry is selected | M8 | as the button (asks first) |

Notes: right-clicking an entry first selects it, so "an entry is selected"
holds for the clicked entry (own design). The separators stand where the
button column has its gaps. Icons follow icons.md (`edit` for both edit
rows, `delete-stored` for Delete); the buttons themselves stay text-only
(later-windows.md). Tier: Own.

### Console log menu (right-click in the Console log)

#### Reference

The research notes for this file do not describe one.

#### LaserCraft (M6)

machine-panels.md (Console, Log) owns it; this is the row table.

| Item | Shortcut | Icon | Enabled when | Milestone | Action |
|---|---|---|---|---|---|
| Copy | Ctrl+C | `copy` | log text is selected | M6 | copies the selected lines to the system clipboard (front-end) |
| Select all | Ctrl+A | none | the log has lines | M6 | selects every line of the log (front-end) |
| —— | | | | | |
| Clear log | — | none | the log has lines | M6 | empties the view only, not the saved history (front-end) |

Notes: Ctrl+C and Ctrl+A act on the log only while it has focus, as egui's
text handling provides; they are not keymap entries. Select all here has no
icon because it selects text, not shapes (own design). The separator is own
design. Tier: Own.

### Font field menu (right-click in the font field)

#### Reference

Right-clicking the reference's font box shows a text-editing menu (undo,
redo, cut, copy, paste, delete, select all) and, set apart, a row that opens
its font manager [R38].

#### LaserCraft (Later)

property-bar.md ("Font field") owns it; this is the row table. The field's
tooltip says "Right-click for the font browser." (property-bar.md).

| Item | Shortcut | Icon | Enabled when | Milestone | Action |
|---|---|---|---|---|---|
| Cut | Ctrl+X | `cut` | text is selected in the field | Later | egui text cut |
| Copy | Ctrl+C | `copy` | text is selected in the field | Later | egui text copy |
| Paste | Ctrl+V | `paste` | the system clipboard holds text | Later | egui text paste |
| Select all | Ctrl+A | none | the field is not empty | Later | egui text select all |
| —— | | | | | |
| Font browser… | — | none | always | Later (with the dialog, dialogs.md) | new front-end action (later) |

Notes: the chords are the ones egui handles inside a focused text field;
they edit the field's text, never the canvas selection. Cut, Copy and Paste
reuse the clipboard ids for the same idea; Select all and Font browser…
have no icon (own design; icons.md has no font-browser id). Until the Font
browser exists, its separator and row are left out (later-milestone rule).
Tier: Echo (text editing rows, then a font row set apart [R38]), Own
(rows, labels, icons).

### Text field menu (every other text field)

#### Reference

The research notes for this file only describe the font box's menu (see
Font field menu) [R38].

#### LaserCraft

Only the font field of the Text style toolbar has a context menu
(property-bar.md, "Font field"); see Font field menu above. Every other text
field has none: egui's text fields have no built-in context menu, and the
editing chords work inside fields as egui provides them (own design;
whether to add a generic one is an open question). Tier: Own.

## Menu look and behavior

### Reference

Drop-down menus on Windows are plain light panels with a thin grey border,
square corners and compact rows [R3]–[R11]. Each row has a narrow icon
column, the label, a shortcut column and, on submenu rows, a small solid
triangle at the far right; submenu rows never show a chord [R3] [R5] [R7]. The
2.1 changelog mentions adding missing blank icons to the main window's menus
[R27]; our reading is that this keeps labels lined up. Separators
are thin light grey rules with a small inset [R3]. Disabled rows stay visible,
with label, shortcut and icon greyed [R5] [R6] [R7]. The row under the pointer
is a full-width band: saturated blue with white text in 2.1, pale blue with
dark text in older captures [R4] [R21] [R22] [R23] [R24] [R26]. Check items
on Windows draw a small box that holds a tick; the language menu marks the
active entry with a dot [R9] [R11]. Several labels show access-key
underlines [R3] [R5] [R6]. Menu rows tend to reuse the toolbar button's icon,
and menus contain nearly every toolbar action [R39]. On macOS the same menus
are dark, translucent and rounded, with symbol chords [R12]. Keyboard
navigation, hover delays and menu tooltips are not described.

### LaserCraft

Every rule below is own design unless marked; the tokens are defined in
visual-language.md.

Panel:

- Background `field`, outline `border`, square or barely rounded corners
  (visual-language.md decides the rounding for all popups), egui's popup
  shadow kept faint. One look on every OS (README.md, platform decision): no
  translucency, no imitation of the macOS menu.
- Width: the longest label plus the shortcut column plus `gap-wide`; a
  submenu is sized on its own.
- Inner padding `gap-tight` at the top and bottom.

Rows, left to right (Echo of the reference's four columns):

1. **Gutter**: `icon-small` wide, always reserved in every menu, so labels
   line up whether or not a row has an icon (Echo of the blank icons the
   reference's 2.1 changelog mentions [R27], read as an alignment aid). It holds the row's icon, or the check tick, or the choice dot,
   or nothing.
2. **Label**: `type-body` in `text`, one line, never wrapped, never bold.
3. **Shortcut**: `type-body` in `text-muted`, right-aligned at the right edge
   (egui's `Button::shortcut_text` places it there); `gap-wide` between the
   longest label and the column.
4. **Submenu arrow**: a small right-pointing triangle in `text-muted` at the
   right edge, on submenu rows only; a submenu row never shows a chord
   (Match [R3]).

- Row height: compact, a little less than `row`, with `gap-tight` between
  rows (density: compact, visual-language.md).
- Icons: `icon-small`, the same drawing as the toolbar button (icons.md),
  strokes in `text`; at most one `accent` detail; `danger` only on `stop`
  (Stop) and `delete-stored` (Remove, deletes Undo cannot bring back),
  following icons.md.

Marks:

- **Check item**: a tick drawn in the gutter in `text` when on; an empty
  gutter when off. No boxes (own design, differs from the reference's boxes
  [R9]). A check item has no icon, so the gutter is free for the tick.
- **Choice dot** (one of a set, such as the active tool): a small filled dot
  in the gutter in `text`; the others show their icons.
- **Heading row**: `type-small` in `text-muted`, not clickable, no hover band.

States:

- **Hovered or keyboard-focused row**: full-width `accent-soft` band, label
  and shortcut stay `text` / `text-muted` (own design; the reference's 2.1
  saturated band with white text is not copied [R4]). A keyboard-focused row
  also shows egui's focus ring in `accent`.
- **Disabled row**: label, icon and shortcut all in `text-disabled`; still
  shown (Match [R5]); it still gets the `accent-soft` band under the pointer
  or keyboard focus, as visual-language.md requires, so its tooltip can be
  read, but choosing it does nothing. Its tooltip says why in one short sentence
  in our words, written by naming.md's tooltip rules, for example "Select two
  objects first" or "Connect to a machine first".
- **Rows of later milestones** are not drawn at all until their milestone
  ships (own design: no placeholder rows, no "coming soon"). A separator that
  would end up at the top, at the bottom or next to another separator is
  dropped.
- **Highlighted row** (only the Stop row during a constant-power job): the
  row is pre-focused when the Machine menu opens, so it shows the
  `accent-soft` band and the `accent` focus ring before the pointer moves
  (see the Machine menu notes).
- **Open submenu**: its parent row keeps the `accent-soft` band while the
  submenu is open.

Separators: a thin line in `border` with a `gap-tight` inset on both sides
where egui allows; a full-width line is acceptable.

Labels and tooltips:

- Labels and the ellipsis rule come from naming.md. A label that names a
  run-time value (Undo ‹action›, Recent projects rows, the camera heading)
  is built by the app from readable names, never from command ids.
- Every row has a tooltip only when it adds something: what a non-obvious
  item does (for example Pack horizontally: "Push the objects together in a
  row, keeping the anchor object in place"), a full path, a disabled reason,
  or a safety note (Pause (motion only): "The laser may stay on"). Tooltips
  wait for egui's normal hover delay.

Behavior:

- Opening: click a title (or right-click for context menus). Submenus open on
  hover after egui's short delay and to the right of the parent row, aligned
  with it; egui moves them left or up near the screen edge.
- Closing: a click on a row runs it and closes the menu; Esc closes the open
  menu; a click outside closes it. A click on a disabled row does nothing and
  keeps the menu open.
- Keyboard: Tab and Shift+Tab move between rows and Enter or Space runs the
  focused row, as far as egui's focus handling supports it in popups; Up and
  Down, Left and Right inside menus, and a key that focuses the menu bar, are
  targets for later and depend on what egui offers (Open questions). There are
  no access keys (see Menu bar).
- Chords while a menu is open: the Pause and Stop chords always act at once
  (safety.md rule 7). Any other bound chord closes the open menu and then runs
  its action, so a chord never acts on a half-open menu.
- Safety: an open menu or submenu never covers the Job bar. The shell checks
  where egui placed the popup and, if it would overlap the Job bar, opens it
  so the Job bar stays clear. A first click on the Job bar while a menu is
  open runs Pause or Stop immediately (it also closes the menu); it must not
  be swallowed as a "click outside". The shell test suite gets a test for
  both (README.md, safety decision).
- Menus never open modal dialogs over the Job bar either; dialogs.md owns
  that rule for the dialogs that menu items open.
- Accessibility: egui's AccessKit support is off today (current-ui.md); when
  it is turned on, menus expose their labels and states through it (later,
  owner decision, needs an ARCHITECTURE.md line if a feature is added).

Tier: Echo (four columns, separators, disabled rows kept, hover band, check
marks), Own (every token, the tick without box, no mnemonics, the safety
rules).

### macOS note

- The menu bar is drawn inside the window on macOS too (README.md, menus
  decision). LaserCraft does not populate the system menu bar beyond what
  eframe puts there by default; Quit stays in File, Preferences… in Edit and
  About LaserCraft… in Help on every platform (own design; the reference
  moves its menus to the system bar [R1], and where it puts settings, about
  and exit on macOS is unknown [R13]).
- Chords are shown with Cmd and Option instead of Ctrl and Alt (keymap.md);
  naming.md decides whether symbols or words are drawn.
- Ctrl+H (Home, M6) is also the system's hide-the-app chord on macOS
  (keymap.md); the Machine menu still shows it, and the user guide explains
  the clash.

## Omitted items

Left out on purpose (README.md, scope; naming.md, Omitted):

- The reference's web-tools menu and its built-in browser, and vendor menus
  from manufacturer bundles [R1] [R27].
- The bundle submenu in the file menu [R46] and the two print commands [R45]
  (keymap.md: not planned).
- Machine-file variants for proprietary controllers; the row always says
  Save G-code… [R3].
- Feeder setup [R43] and every galvo-only row: cylinder correction, repeat
  marking, galvo lens calibration and the galvo camera rows [R8] [R47].
- Online nesting through a web service; local nesting is Nest [R41].
- License management, the update check, the vendor's forum, feature-request,
  printable-documentation and video links, help for a third-party design
  program, the camera-selection helper, network-camera statistics and the
  developer tools [R10].
- The language menu for now (later, see Menu bar) [R11].
- The custom-icon row and sort by type in the device menu [R35].
- The camera-selection helper in the camera menu [R31].
- The reference's maturity marker on experimental items; LaserCraft writes
  "(experimental)" [R6].
- Sending a job without starting it (keymap.md's Alt+Shift+S), which only DSP
  controllers do (naming.md, Omitted).

## Current state → target

From current-ui.md 1.4 and `crates/lasercraft-ui/src/shell.rs`:

1. **Labels** use Title Case today ("Save As…", "Project Notes…", "Invert
   Selection", "Edit Nodes", "Fit Bed", "Side Panels", "Job Preview…",
   shell.rs:17-117, 432-499); target: sentence case from naming.md.
2. **Undo label** shows the raw command id ("Undo shape.move",
   shell.rs:447-450); target: readable action names (naming.md), which needs a
   display name per command (see Proposed changes, commands.md).
3. **File menu** has no separators and puts Import… after Save as…
   (shell.rs:432-441); target: the table order with three separators (Import…
   and Project notes… after Open…, Save G-code… after Save as…, Quit alone).
4. **Edit menu** order is Undo, Redo, separator, Cut, Copy, Duplicate,
   Delete, Invert selection, Paste, Paste in place, Select all
   (shell.rs:447-472); target: Select all and Invert selection first, then
   Cut, Copy, Duplicate, Paste, Paste in place, Delete; then the clean-up
   group.
5. **Invert selection** is greyed with no selection (it is in
   `SELECTION_EDITS`, shell.rs:96-103); target: enabled whenever the project
   has shapes.
6. **Tools menu** holds Convert to path, Close path, Join paths and Break
   apart (shell.rs:27-37); target: the first three move to Edit, Break apart
   to Arrange. Rectangle and Ellipse move into Shape tools ▸. The active tool
   gets the choice dot. A separator between tool modes and operations stays.
7. **Arrange menu** starts with rotate, mirror, drawing order and select
   next/previous, then a separator, then group and the submenus
   (shell.rs:40-117, 483-491); target: the table order with its separators.
   "Forward/Backward/To Front/To Back" become Bring forward, Send backward,
   Bring to front, Send to back (naming.md).
8. **Enabling** of every Arrange and path item is "something is selected"
   (shell.rs:483-491, 505-519); target: the conditions in the tables (two
   objects for Group, Align ▸ and the booleans; three for the spacing rows).
9. **Move on bed ▸** has five rows (shell.rs:76-89); target: the order of
   toolbars.md's flyout, To head position at M6 after the corners, and later
   Left, Right, Top, Bottom, which `arrange.move_to_bed` already accepts.
10. **View menu** order is Zoom in, Zoom out, Fit bed, separator, Side
    panels, Job preview… (shell.rs:492-499); target: Job preview…, Fit bed,
    Zoom in, Zoom out first; Side panels becomes a check item with a tick.
11. **Chords** print key names ("Ctrl+Equals", "Ctrl+Backtick", "Period",
    "PageUp") and always "Ctrl+" even on macOS (keymap.rs Display,
    current-ui.md 1.4); target: symbols as naming.md says, Cmd/Option on
    macOS. Delete shows the first chord bound (Del).
12. **No icons and no gutter** (current-ui.md 3); target: the reserved gutter
    with ticks, dots and icons as in the tables, drawn by the method
    icons.md chooses.
13. **Hover and open-title look** are egui defaults following the OS theme
    (current-ui.md 2); target: the tokens above in both themes.
14. **No Machine, Window or Help menu and no context menus** (current-ui.md
    1.4); target: as specified, each at its milestone.
15. **No tests** check menu order today; target: the shell tests
    (`crates/lasercraft-ui/tests/shell.rs`) check the menu-bar title order,
    that a menu never covers the Job bar, and that the Job bar takes the
    first click while a menu is open (M6).

## Proposed changes to other specs

Each is written up here and must be applied to that spec before code
(README.md, existing specs win).

- **Proposed change to ui-shell.md**: replace the Layout section's menu list
  with a pointer to this file; adopt the File, Edit, Tools, Arrange and View
  order and separators above, Side panels as a check item, and the moves of
  Convert to path, Close path and Join paths to Edit and Break apart to
  Arrange; add the Machine (M6), Window and Help menus and the context menus;
  sentence case (naming.md).
- **Proposed change to keymap.md**: show chords with symbols and Cmd/Option on
  macOS (naming.md); mark Alt+Shift+S not bound and Ctrl+Shift+F not planned
  (naming.md, Omitted); consider binding Alt+I (Image adjustments…) and Alt+T
  (Trace to vectors…) at M7, both shown on the reference's tools menu page
  [R6]; record the owner's choice for Ctrl+Shift+A (framing the selection or
  Zoom to selection) and whether Device profiles… gets a chord; state that the
  Pause and Stop chords act even while a menu is open.
- **Proposed change to commands.md**: give each command a readable action
  name for "Undo ‹action›" (and for "Redo ‹action›" if the owner wants it,
Open questions), and a way to ask whether a
  command can run on the current project and selection, so menus grey items
  without repeating each command's rules in the UI.
- **Proposed change to canvas.md**: right-click opens the canvas menu;
  right-clicking an unselected object selects it first; no menu while a shape
  is being drawn.
- **Proposed change to arrange.md**: Slide ▸ (sliding until touching), Match
  width and height, Lock and Unlock, linked copies and Nest, all excluded or
  absent today.
- **Proposed change to layers.md**: the layer and all-layers context menus
  and the command behind Put line layers last.
- **Proposed change to shapes.md and text.md**: the shape kinds of Shape
  tools ▸ and Put text on path (as naming.md already proposes).

## Open questions

1. Ctrl+Shift+A: framing the selection (keymap.md, M6) or Zoom to selection
   (the reference's window menu [R9])? If framing, which Machine menu row shows
   it (naming.md question 3)?
2. Go to selection ▸ in the Machine menu (this file) or in Arrange as in the
   reference [R7]?
3. Device profiles…: bind a chord? The reference shows one on its menu and
   names another in its changelog [R8] [R27].
4. Clear alarm: in the Machine menu as specified, or only in the Alarm
   banner and Console (naming.md question 10)?
5. Frame at low power…: offer it at all (naming.md question 13)?
6. Disconnect while a job runs: disabled (this file) or allowed after an
   automatic Stop?
7. Keyboard access to the menu bar (a key such as F10 that focuses File,
   arrow-key navigation inside menus): build it with our own code if egui
   lacks it?
8. Labels for storing the canvas selection in Saved artwork from the canvas
   menu (the reference has two rows [R30]); naming.md has none yet. Proposed:
   one row "Add to saved artwork".
9. Recent projects ▸: add a row that clears the list (needs a label in
   naming.md), and is ten entries the right length?
10. Right-click while drawing a line: should it finish the line, as the
    reference does [R50], instead of doing nothing?
11. Default states of Show fills (off here, as in canvas.md) and Smooth
    curves (on here).
12. Help menu milestone: About LaserCraft… and License… before M10, since
    GPL notices are useful early?
13. New window: wanted at all (multi-viewport; the reference uses a second
    window to edit one project while cutting another, and all its windows
    share preferences [R26])?
14. A text-field context menu (cut, copy, paste) for every field, beyond the
    font field's: worth adding beyond egui's chords?
15. Redo: keep the plain "Redo" (naming.md) or show "Redo ‹action›" like
    Undo? If the latter, it is a proposed addition to naming.md.

## Sources

LightBurn public documentation, version 2.1, all read 2026-10-21 through the
style guide's research notes. Only facts are taken; no wording.

1. [R1] https://docs.lightburnsoftware.com/2.1/Reference/UI/MenuToolbar/ — menu bar titles and order, macOS system menu bar, access-key underlines, web-tools menu
2. [R2] https://docs.lightburnsoftware.com/2.1/Reference/UI/ — main window map: menu bar position and the titles in the 2.1 screenshot
3. [R3] https://docs.lightburnsoftware.com/2.1/Reference/UI/FileMenu/ — file menu rows, icons, chords, separators, disabled rows, menu anatomy
4. [R4] https://docs.lightburnsoftware.com/2.1/Reference/FileManagement/ — recent-projects submenu on hover, open-title and hover band look, access keys
5. [R5] https://docs.lightburnsoftware.com/2.1/Reference/UI/EditMenu/ — edit menu rows, icons, chords, disabled states
6. [R6] https://docs.lightburnsoftware.com/2.1/Reference/UI/ToolsMenu/ — tools menu rows, the single separator, enabling pattern, chords, experimental marker
7. [R7] https://docs.lightburnsoftware.com/2.1/Reference/UI/ArrangeMenu/ — arrange menu rows, separators, submenus, chords, enabling
8. [R8] https://docs.lightburnsoftware.com/2.1/Reference/UI/LaserToolsMenu/ — machine-tools menu rows for gantry devices, device-dependent rows, focus test needing Z
9. [R9] https://docs.lightburnsoftware.com/2.1/Reference/UI/WindowMenu/ — window menu: reset row, preview and zoom rows, view-style heading and rows, side panels, toolbar and window check lists
10. [R10] https://docs.lightburnsoftware.com/2.1/Reference/UI/HelpMenu/ — help menu rows and groups
11. [R11] https://docs.lightburnsoftware.com/2.1/Reference/UI/LanguageMenu/ — language menu and its active-entry dot
12. [R12] https://docs.lightburnsoftware.com/2.1/Reference/ViewStyle/ — display styles, display only, macOS menu rendering
13. [R13] https://docs.lightburnsoftware.com/2.1/Reference/ZoomPan/ — zoom chords; settings location differing by platform
14. [R14] https://docs.lightburnsoftware.com/2.1/Guides/CustomizingTheLightBurnWindow/ — check list toggling, right-click check list on toolbars and title strips, reset on start
15. [R15] https://docs.lightburnsoftware.com/2.1/Reference/ResetToDefaultLayout/ — what the layout reset restores and leaves alone
16. [R16] https://docs.lightburnsoftware.com/2.1/Reference/UndoRedo/ — undo history row, macOS menu bar without the web-tools menu
17. [R17] https://docs.lightburnsoftware.com/2.1/Reference/ManagingPreferences/ — preferences-file submenu
18. [R18] https://docs.lightburnsoftware.com/2.1/Reference/ImageOptions/ — image submenu rows and their enabling rule
19. [R19] https://docs.lightburnsoftware.com/2.1/Reference/PrimaryShapes/ — shape-tools submenu contents
20. [R20] https://docs.lightburnsoftware.com/2.1/Reference/Align/ — align commands and chords
21. [R21] https://docs.lightburnsoftware.com/2.1/Reference/Distribute/ — distribute submenu rows and separator; older hover look
22. [R22] https://docs.lightburnsoftware.com/2.1/Reference/Dock/ — slide-direction submenu; older hover look
23. [R23] https://docs.lightburnsoftware.com/2.1/Reference/MoveSelectedObjects/ — bed-position submenu order, Ctrl moves the laser; older hover look
24. [R24] https://docs.lightburnsoftware.com/2.1/Reference/MoveLaserToSelection/ — head-to-selection submenu
25. [R25] https://docs.lightburnsoftware.com/2.1/Reference/PrintAndCut/ — print-and-cut submenu rows, chords, separator, disabled target rows
26. [R26] https://docs.lightburnsoftware.com/2.1/Reference/NewWindow/ — second program window (edit one project while cutting another, shared preferences), older file menu
27. [R27] https://docs.lightburnsoftware.com/2.1/NewFeatures/NewIn2.1/ — web-tools and vendor menus, access-key clash fix, missing blank menu icons added, device-list chord
28. [R28] https://docs.lightburnsoftware.com/2.1/Reference/Clipboard/ — clipboard rows in the canvas right-click menu
29. [R29] https://docs.lightburnsoftware.com/2.1/Reference/CutsLayersWindow/ — layer row and header context menus
30. [R30] https://docs.lightburnsoftware.com/2.1/Reference/ArtLibrary/ — art-library context menu, canvas rows that store the selection
31. [R31] https://docs.lightburnsoftware.com/2.1/Reference/Cameras/CamerasWindow/ — camera panel context menus
32. [R32] https://docs.lightburnsoftware.com/2.1/Reference/AdjustImage/ — canvas context menu over an image
33. [R33] https://docs.lightburnsoftware.com/2.1/Reference/GridArray/ — virtual-array rows in the canvas menu, macOS context menu look
34. [R34] https://docs.lightburnsoftware.com/2.1/Reference/BarCode/ — edit-barcode row in the canvas menu
35. [R35] https://docs.lightburnsoftware.com/2.1/Reference/Devices/ — device list context menu
36. [R36] https://docs.lightburnsoftware.com/2.1/Reference/Preview/ — preview row in the canvas menu
37. [R37] https://docs.lightburnsoftware.com/2.1/Reference/LockShapes/ — lock and unlock rows in the arrange and right-click menus
38. [R38] https://docs.lightburnsoftware.com/2.1/Reference/Text/ — font box context menu
39. [R39] https://docs.lightburnsoftware.com/2.1/GetStarted/UITour/ — menus repeat toolbar actions and icons, shortcuts at the right
40. [R40] https://docs.lightburnsoftware.com/2.1/Reference/QuickNest/ — local nesting row
41. [R41] https://docs.lightburnsoftware.com/2.1/Reference/NestSelected/ — online nesting row
42. [R42] https://docs.lightburnsoftware.com/2.1/Reference/Selection/ — select-circles row
43. [R43] https://docs.lightburnsoftware.com/2.1/Reference/FeederSetup/ — feeder setup only for proprietary controllers
44. [R44] https://docs.lightburnsoftware.com/2.1/Reference/HelpAndNotes/ — tabbed help window opened from the help menu
45. [R45] https://docs.lightburnsoftware.com/2.1/Reference/Print/ — print commands use the system print dialog
46. [R46] https://docs.lightburnsoftware.com/2.1/Reference/UserBundles/ — bundle submenu
47. [R47] https://docs.lightburnsoftware.com/2.1/Reference/CalibrateGalvoLens/ — galvo-only lens calibration row
48. [R48] https://docs.lightburnsoftware.com/2.1/Reference/MaterialLibrary/ — material library popup menu
49. [R49] https://docs.lightburnsoftware.com/2.1/Reference/LaserWindow/ — job control and the device button live in the laser panel; right-click reconnects
50. [R50] https://docs.lightburnsoftware.com/2.1/Reference/DrawLines/ — right-click finishes a line being drawn

Existing specs and files used: `docs/clean-room.md`, `docs/spec/safety.md`
(rules 2, 7, 9), `docs/spec/ui-shell.md`, `docs/spec/keymap.md`,
`crates/lasercraft-ui/keymap/default.keymap`, `docs/spec/arrange.md`,
`docs/spec/canvas.md`, `docs/spec/device.md`, `docs/spec/commands.md`,
`crates/lasercraft-ui/src/shell.rs`, the command list in
`crates/lasercraft-commands/src/cmds/`, and the other style files
(README.md, naming.md, layout.md, icons.md, property-bar.md,
layers-panel.md, machine-panels.md, later-windows.md). Open standards: GRBL v1.1 commands and
laser mode (https://github.com/gnea/grbl/wiki/Grbl-v1.1-Commands,
https://github.com/gnea/grbl/wiki/Grbl-v1.1-Laser-Mode) for homing in the
alarm state, feed hold and the M3 warning; WCAG 2.2 contrast targets apply to
`text-muted` on `field` through visual-language.md.

Own design: the Machine and View menus and the Machine menu's top part; every
label (from naming.md), tooltip, icon concept and token use; moving Go to
selection ▸ to Machine; the extra Tools separator and the active-tool dot;
two View check items instead of four styles; Snap rows in View; the Window
menu's heading rows; the Recent projects row format; every
context-menu selection rule, separator and placement not shown in the
reference; the tick without a box; no access keys; hiding rows of later
milestones; the chord rules while a menu is open; the rule that menus never
cover or block the Job bar; the macOS placement of Quit, Preferences… and
About LaserCraft….
