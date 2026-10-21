# Spec: style — windows for later milestones

Milestones: M7 (Image adjustments, Save adjustment preset, Trace to
vectors, Convert to image), M8 (Materials panel, Speed and power test, Line
interval test, Focus height test, No Z axis), M9 (Camera panel, Camera list,
Add camera, Camera setup, Camera lens calibration, Camera alignment, Rotary
attachment, Taper correction, Print registration, Registration failed);
Find center, the Saved artwork panel, the Text data panel (owner decision)
and Simple layout are later, with no milestone yet.

This file fixes what each panel, dialog, wizard and message that arrives
after M6 contains, in which order, how it looks (through tokens only), when
each control is enabled, what it says and what it does, so that these
windows fit the shell the other style files describe. It does not repeat
what they own: where each panel docks and its tab order are `layout.md`'s;
every label, tooltip rule, message shape and empty-state line is
`naming.md`'s (labels this file needs that naming.md does not list yet are
marked "proposed label" and collected under Proposed changes); colors, type
and sizes are the tokens defined in `visual-language.md`; icon artwork is
`icons.md`'s (this file only names each icon's concept); the menu rows that
open these windows are `menus.md`'s; the Job setups toolbar and the Capture
camera image button are `toolbars.md`'s; the job bar, the Machine and Jog
panels and the device dialogs are `machine-panels.md`'s; the camera
overlay, registration targets and tool-window previews drawn on the canvas
are `canvas.md`'s; the layer editor, swatch chips and the Images on this
layer section are `layers-panel.md`'s; the Properties panel and the Text
style toolbar are `property-bar.md`'s; the job preview window is
`job-preview.md`'s; dialog frames, footers and wizard chrome are
`dialogs.md`'s; `README.md` explains the tiers (Match, Echo, Own). The
behavior specs (`docs/spec/safety.md`, `docs/spec/layers.md`,
`docs/spec/device.md`, `docs/spec/keymap.md`, `docs/spec/text.md`) win
wherever they already decide something.

## How to read this file

- Every element has a **Reference** part (what the public docs show, each
  fact tagged with a source key such as [R1], listed under Sources), a
  **LaserCraft** part (the binding spec) and a **Tier** line (Match, Echo
  or Own, as `README.md` defines them).
- Milestone tags: *M7*–*M10* as in `CLAUDE.md`; *Later* = no milestone yet;
  *owner decision* = needs the owner before it is built.
- Every button, switch and field invokes a registry command or a `ui.*`
  front-end action (CLAUDE.md, architecture rules). Opening a window is a
  `ui.*` action; changing the project is an undoable command; moving the
  machine or running a job is a non-undoable machine command, exactly as
  in `machine-panels.md`.
- Shortcuts are quoted from `docs/spec/keymap.md`. A chord keymap.md lists
  for a later milestone is written with that milestone, for example
  "Alt+C (M9)", and is not shown in a menu or tooltip until
  `crates/lasercraft-ui/keymap/default.keymap` binds it. A chord keymap.md
  does not list is "not bound".
- Field widths, decimals and ranges follow `naming.md` ("Numbers in fields
  and read-outs"); units sit in the label, never typed after the number.
- "Proposed label" marks wording this file introduces; it becomes binding
  only when `naming.md` takes it over (Proposed changes).

---

## Conventions shared by every window in this file

### Dialog frame (summary of dialogs.md)

`dialogs.md` is the master for dialog frames, footers and wizard chrome.
This summary only lists what the windows below rely on; where the two ever
differ, `dialogs.md` wins.

- Every dialog, wizard and message is drawn by egui inside the main
  window, with the same look on Linux, Windows and macOS (README.md,
  platform). Its title repeats the command that opens it, without the
  ellipsis ("Image adjustments", not "Image adjustments…") (naming.md,
  Parts of speech).
- Footer: a strip along the bottom edge, buttons at its right end. The
  confirming button is the default (`accent` fill, keyboard Enter); Cancel
  or Close sits beside it; Esc is Cancel. A destructive button uses the
  `danger` style and is never the default.
- Wizard footer: Back, Skip (only on pages that may be skipped), Next or
  Finish, Cancel, as `dialogs.md` orders them (naming.md, wizard buttons).
  Next stays disabled until the page's requirement is met; its tooltip's
  second line says what is missing (naming.md, Tooltips).
- Tool windows that change shapes draw their result on the canvas as a
  live preview (dashed `drawing`, `canvas.md`); OK builds one undoable
  command; closing or Cancel runs nothing (ui-shell.md, M3 additions).
- **While a machine is connected, no window in this file blocks the job
  bar.** No `egui::Modal` backdrop may cover it, and Stop and Pause in the
  job bar stay clickable whatever is open (README.md, safety; safety.md
  rule 7; `layout.md`, "Job bar above dialogs"). Windows that run jobs
  (the three tests, Camera alignment's burn page) are ordinary movable
  windows, never modal.
- Resizable windows (Image adjustments, Trace to vectors, Convert to image,
  Camera setup and the camera wizards) have a minimum size at which every
  control stays reachable; below the width of two side-by-side groups,
  groups stack (own design).

### Shared look

- Body: `panel` fill; lists, trees, tables, picture panes and text fields
  are `field` with a thin `border`; alternating rows `field-alt`.
- Group boxes: a heading in `type-heading` over a frame with a thin
  `border` and `panel` fill (Echo of the reference's framed groups
  [R2][R5]; bold is kept for dialog headings only, `visual-language.md`).
- Collapsing sections: a full-width row with a disclosure marker and the
  section's name in `type-body`; closed by default unless the window
  section says otherwise (Echo [R9][R11]).
- Forms: labels right-aligned directly left of their fields, two columns
  of label and field pairs where the window is wide enough; no colon after
  a field label (naming.md, General voice).
- Switches, never check boxes (decision 4; `visual-language.md` draws
  them). Mutually exclusive choices of two to four items are choice dots
  (radio buttons) in one row; longer lists are dropdowns.
- Sliders: thin groove, the part left of the knob filled with `accent`,
  always paired with a number field at the right that shows and accepts
  the exact value (Echo [R5][R9]; the reference's slider color follows the
  operating system, ours is always `accent`).
- Selected list, tree or table row: `accent-soft`, shown the same whether
  or not the list has keyboard focus (`visual-language.md`, Tables and
  lists; Echo [R1]: the reference uses a strong blue band with white text;
  ours is the pale tint used everywhere).
- Job buttons inside dialogs (Job preview…, Frame, Start, Pause, Stop,
  Save G-code…): the big tier, `row-big` tall, `icon-big` left of the
  label, the same icon and look as in the Machine panel
  (`machine-panels.md`); Stop has the `danger` fill. Echo of the
  reference's large glyph-and-text buttons in its test windows [R2][R3][R4]
  [R13].
- Status icons (camera windows, wizard results): a round `success` badge
  with a check for "done", a `warning` badge with an exclamation mark for
  "needs attention" or "in progress", a `danger` badge with a cross for
  "failed" (own design; the reference uses green, yellow and red signs of
  its own shape [R10][R11]). Each badge has a text word beside it, so
  color is never the only signal (WCAG 2.x, use of color).
- Empty states: one line in `text-muted`, the text from `naming.md`
  ("Empty states").

### Jobs started from these windows (safety)

The three test windows and Camera alignment's burn page start real jobs.
They obey `docs/spec/safety.md` exactly like the Machine panel's Start
(`machine-panels.md`), and add nothing that bypasses it:

1. The laser fires only after the user clicks Start (or uses Alt+S (M6)
   while the window has focus). Opening a window, changing a field, Job
   preview…, Frame and Save G-code… never fire (rule 1).
2. Frame is laser-off; there is no low-power variant inside these windows
   and no modifier that turns the laser on (rule 2; README.md, safety).
3. The pattern's every move, including labels and borders, is checked
   against the work area before anything is sent; a refusal shows the
   "Cannot run: …" band (naming.md, Messages) inside the window and sends
   nothing (rule 5).
4. Every power value in the pattern, including the highest test value, is
   clamped to the device's Max power (%) and shown clamped: a field whose
   value exceeds it shows a `warning-soft` band "Warning: the device
   maximum is ‹m›%. ‹m›% is used." (rule 6, naming.md "Power clamped").
5. Patterns use dynamic power (M4) unless the user's base settings turn
   Constant power (M3) on; then Pause in the window reads "Pause (motion
   only)" with "The laser may stay on", and Stop gets the highlight ring
   (rule 9; `machine-panels.md`, Constant-power variant).
6. Pause and Stop in the window repeat the job bar's buttons with the same
   enablement as `machine-panels.md` (enabled in every connected state;
   Pause reads Resume while GRBL holds a job); the job bar stays the
   safety control (rule 7). An alarm, a stall or a disconnect
   stops the pattern and shows the Alarm banner in the Machine panel and
   job bar, never only inside the window (rules 10 and 11).
7. Closing a window while its job runs does not stop the job; the job bar
   keeps Pause and Stop (own design: closing a view never acts on the
   machine).

Tier for this block: **Own** (all of it implements safety.md).

---

## Materials panel (M8)

### Reference

- The reference's material library is a dock window in the lower tab group
  of the right column, the second tab after its job-control window; it can
  be closed and brought back from the Window menu and returns to its place
  on a layout reset [R1][R23][R24].
- Top row: a lead-in label, a wide dropdown of loaded libraries, then a
  rename button and an unload button that takes the library off the list
  without deleting its file [R1].
- Below, two columns of about equal width. Left: a tree with three levels,
  material name, then a thickness folder (or, for entries marked as having
  no thickness, a folder named by the entry's title), then the entries
  themselves, each led by a small outlined pictogram whose meaning the page
  does not explain; a full-width button under the tree opens a popup of
  library file actions [R1].
- Right: a small caption, then a stack of text buttons in three groups
  separated by gaps: assign the entry to the active layer (a one-time copy)
  and link the layer to the entry (the layer then follows later edits and
  becomes read-only in its settings editor); create an entry from the
  active layer (a taller two-line button), edit the entry's cut settings,
  overwrite the entry from the layer, edit its description; duplicate and
  delete [R1].
- Creating or describing an entry replaces the right column with a form in
  place: material name, thickness with a "no thickness" switch that
  enables a title field instead, description, then OK and Cancel stacked
  full width; the tree's selection turns pale meanwhile [R1].
- The popup holds, with icons: load, save, save as, create a new library,
  merge another library in [R1]. Version 2.1 moved the library buttons
  below the list and gave them icons [R25][R29].
- A saved library remembers the device it was made for and loads when that
  device is chosen; linked layers are re-linked by matching material,
  thickness or title, and description; a library on a shared folder can be
  used by several computers [R1].
- Not shown: disabled looks, right-click or drag in the tree,
  double-click [R1].

### LaserCraft

Panel title "Materials" (naming.md). Place: right dock, lower group, second
tab after Machine, shown by default (`layout.md`). Holds LaserCraft's own
library format and our own data (CLAUDE.md M8); the reference's library
files are never read (`docs/clean-room.md`). Milestone: M8.

Contents, top to bottom:

| # | Row | Contents, left to right |
|---|---|---|
| 1 | Library row | label "Library", dropdown of open libraries (stretches), "Rename library…", "Close library" |
| 2 | Browser | two columns: left the entry tree, right the button column (or the entry form, below) |
| 3 | Bottom row | under the tree: "Library ▸" (opens the Library menu); under the buttons: the target line |

**Library row**

- The dropdown lists every open library by its name; the active one is
  shown. Choosing one makes it active. A library with unsaved changes ends
  in " •" (the project title's unsaved mark, naming.md).
- Rename library… opens a small dialog "Rename library" with one field
  "Name" and OK, Cancel; it renames the library inside its file (own
  design; the reference's text says to rename the file outside the
  program for its art library [R19], which we do not copy).
- Close library takes the library off the list; the file stays on disk.
  With unsaved changes it asks first (naming.md, Confirmations): title
  "Close library?", body "‹name› has changes that are not saved.",
  buttons "Save library", "Discard" (`danger`), "Cancel".

**Entry tree (left column)**

- `field` with a thin `border`, filling the column's height, scrolling
  vertically.
- Three levels, each with a disclosure marker: material ("Plywood"), then
  thickness ("3.00 mm", naming.md number format) or, for entries with
  Thickness does not apply on, the entry's group name, then the entries,
  shown by their description.
- Each entry starts with a small `icon-small` mode pictogram in
  `text-muted` (line, fill or image; `icons.md`), so the user sees what
  kind of layer it holds (own meaning; the reference's leaf glyph is
  unexplained [R1]). A second pictogram, a small chain, marks entries that
  a layer of the open project follows.
- Selected entry `accent-soft`, also while the entry form is open
  (Echo [R1]; `visual-language.md`, Tables and lists).
- Sorted by material, then thickness, then description, each
  alphabetically and numerically (own design).
- Double-click on an entry = Apply to layer (own design; not documented
  for the reference [R1]).
- Right-click on an entry: a small menu with the same actions as the
  button column, in the same order (own design; menus.md may adopt it).

**Button column (right column)**

| # | Group | Button (exact label) | Enabled when | Action |
|---|---|---|---|---|
| 1 | Use | Apply to layer | an entry and a target layer exist | copy the entry's settings into the target layer; afterwards the two are independent (undoable command) |
| 2 | Use | Follow this entry | an entry and a target layer exist | tie the target layer to the entry; the layer editor shows it read-only (below) (undoable command) |
| — | gap one `row` high | | | |
| 3 | Edit | Save layer as material… | a target layer exists | open the entry form, prefilled from the layer; OK stores the layer's settings, operations included once they exist, as a new entry |
| 4 | Edit | Edit settings… | an entry is selected | open the Layer settings window (`layers-panel.md`) on the entry; OK writes back to the entry |
| 5 | Edit | Update from layer | an entry and a target layer exist | overwrite the entry's settings with the target layer's (asks first, below) |
| 6 | Edit | Edit description… | an entry is selected | open the entry form on the entry |
| — | gap one `row` high | | | |
| 7 | Copy and remove | Duplicate | an entry is selected | add a copy named "‹description› (copy)" |
| 8 | Copy and remove | Delete | an entry is selected | remove the entry (asks first, below) |

- Ordinary buttons (`row` tall), full column width, text only (Match: the
  reference's are text-only [R1]); Save layer as material… is one line,
  not the reference's taller two-line button (own design).
- Under the buttons, the **target line**: "Target layer" in `text-muted`,
  then the target layer's swatch chip (`layers-panel.md` chip look) and
  its name. The target layer is the layer being edited in the Layers panel
  (`layers-panel.md`); with none, the line reads "No layer selected" and
  buttons 1–3 and 5 are disabled with the second tooltip line "Click a
  layer in the Layers panel first." (naming.md, Tooltips).
- Tooltips (first line / second line):
  - Apply to layer: "Copy this material's settings into the target layer"
    / "The layer does not change when the entry changes later."
  - Follow this entry: "Keep the target layer tied to this material" /
    "The layer's settings become read-only and follow the entry."
  - Save layer as material…: "Store the target layer's settings as a new
    material entry".
  - Edit settings…: "Change this entry's settings".
  - Update from layer: "Replace this entry's settings with the target
    layer's".
  - Edit description…: "Change this entry's material, thickness and
    description".
  - Duplicate: "Copy this entry".
  - Delete: "Remove this entry from the library".
- Confirmations (naming.md): Update from layer asks "Replace material
  settings?" / "The settings of "‹description›" are replaced by those of
  layer "‹layer›". Layers that follow it change too." with "Replace
  settings" and "Cancel". Delete asks "Delete material entry?" / "The
  entry "‹description›" is removed from ‹library›. Layers that follow it
  keep their current settings and stop following it." with "Delete entry"
  (`danger`) and "Cancel".

**Entry form (replaces the button column in place)**

| # | Label | Widget | Notes |
|---|---|---|---|
| 1 | Material | text field | first tree level; suggests existing material names as you type |
| 2 | Thickness (mm) | number field | 2 decimals; disabled while switch 3 is on |
| 3 | Thickness does not apply | switch | off by default |
| 4 | Group name | text field | enabled only while switch 3 is on; it replaces the thickness level in the tree |
| 5 | Description | text field | the entry's name in the tree |
| 6 | (space) | | |
| 7 | OK, Cancel | buttons | side by side at the column's foot, OK the default |

- Labels are right-aligned in one column (Echo [R1], own labels from
  naming.md). OK is enabled when Material and Description are not empty
  and either Thickness (mm) is above zero or Group name is not empty.
- While the form is open the tree stays visible and browsable, but its
  selection keeps its `accent-soft` row; choosing another entry asks nothing and
  keeps the form (own design).

**Bottom row**

- "Library ▸" under the tree (ordinary button, the full tree width) opens
  the Library menu (`menus.md`: Open library…, Save library, Save library
  as…, New library, Merge library…). Echo of the reference moving these
  actions under the list in 2.1 [R25].

**Layers that follow an entry** (cross-reference)

- A layer that follows an entry shows, at the top of its layer editor, an
  `info-soft` band "Follows material ‹description› (‹library›)" with a
  button "Stop following"; its settings fields are read-only (proposed
  change to `layers-panel.md` and `docs/spec/layers.md`).
- Following is kept in the project as the library's name and the entry's
  material, thickness or group name and description; when the library is
  not open, the band reads "Follows a material from ‹library›, which is not
  open" in `warning-soft`, and the layer keeps its last settings (own
  design, after the reference's re-link idea [R1]).

**Device profiles and libraries**

- Each library may name a device profile; choosing that profile opens the
  library and makes it active (Echo [R1]). The library row's tooltip names
  the profile: "Used with device profile ‹name›".

**States**

- No library open: rows 1 and 2 are replaced by the empty-state line from
  `naming.md` ("No library is open. Use the Library menu to open or start
  one.") above the Library ▸ button.
- A library with no entries: the tree shows "No materials yet. Select a
  layer, then use Save layer as material…." in `text-muted` (proposed
  empty-state line).
- Narrow panel (below the width of two columns): the button column moves
  under the tree (own design).

Tier: **Match** (place, row order, two columns, the button groups and their
order, form in place, popup actions), **Echo** (tree levels, pale selection
while editing), **Own** (labels, pictograms, target line, confirmations,
following band, library-to-profile tooltip, format and data).

---

## Test patterns (M8)

The three test windows share one layout idea: settings on top, the job
buttons, then a footer. Each is opened from the Machine menu (`menus.md`)
and builds a temporary job: the pattern is not added to the project
(own design; whether it may also be placed on the canvas is an open
question). The pattern's position follows the Machine panel's Place job
at (`machine-panels.md`): with Absolute coordinates the window's center
fields place it; otherwise they are disabled and the job reference point
decides (Echo [R2][R3][R4]).

Shared parts of all three:

- **Job buttons** (big tier, see Shared look), in a group headed "Run" in
  `type-heading`:

| Button | Enabled when | Action |
|---|---|---|
| Job preview… | the settings are valid | open the job preview window (`job-preview.md`) on the pattern; works without a machine |
| Frame | connected and Ready | trace the pattern's bounding box with the laser off |
| Start | connected and Ready, settings valid | check the pattern against the work area, then run it |
| Pause / Resume | while connected (every connected state, exactly as `machine-panels.md`) | as in the Machine panel: reads Resume while GRBL holds a job; "Pause (motion only)" for constant power |
| Stop | while connected | as in the Machine panel |
| Save G-code… | the settings are valid | write the pattern as a G-code file (same file window as File → Save G-code…) |

- The job buttons sit in a grid of two columns: Job preview… and Frame,
  Start and Pause, Stop and Save G-code… (Echo of the reference's grid
  [R2]). Where a window is too narrow they flow into one column in the
  same order.
- Under the grid, a read-out "Pattern size ‹w› × ‹h› mm" in `type-small`,
  `text-muted` (Echo [R2]; proposed label).
- **Footer**: "Close" only (own design). Values are remembered per device
  profile as soon as they change, so there is nothing to confirm or
  revert (the reference's OK keeps and Cancel reverts [R3][R4]; we drop
  the distinction because the windows run jobs and a Cancel that undoes
  field edits after a burn would be confusing).
- **Presets** (Speed and power test only; see there).
- Every field has a range; an out-of-range value is shown with a `danger`
  border and the second tooltip line saying the range, and Start, Frame,
  Job preview… and Save G-code… are disabled until it is fixed.

### Speed and power test

#### Reference

- A wide window, about twice as wide as tall, opened from the machine
  tools menu [R2][R15].
- Top row: a presets dropdown with four small square icon buttons (save,
  delete, export, import), then a title field whose purpose the page does
  not explain [R2]. The presets list has a blank first row, a separator
  line for built-in presets, four built-in ones, a separator for user
  presets, then the user's [R2].
- Middle: three framed groups side by side. One for rows and one for
  columns, each with count, the parameter it varies (dropdown; speed down
  the rows and power across the columns by default), minimum, maximum, box
  height or width, and the center coordinate used only in absolute
  placement; the third holds the job buttons as large glyph-and-text
  buttons in two columns, with a machine-file save and a send button, and
  a read-only output size under them [R2].
- Bottom: three wide buttons, each starting with a layer-color chip with
  its two-digit number, that open the cut settings for the boxes, the
  labels and the border; switches for drawing the labels and the border;
  a close button at the right [R2].
- The grid defaults to ten by ten; varied parameters can be power, speed,
  interval or passes (plus pulse settings on other laser kinds); the run
  goes from the least risky box to the most; presets are kept with the
  preferences; the page warns about speed units [R2].
- The engraved result is a grid of boxes with the values printed along the
  left and bottom edges, axis titles, and a line of the constant settings
  above [R2].

#### LaserCraft

Title "Speed and power test" (naming.md). Opened by Machine → Speed and
power test… (`menus.md`; enabled always; Start inside needs a connection).
A movable, non-modal window, landscape. Milestone: M8 (CLAUDE.md: "test
grids (speed × power)").

| # | Part | Contents |
|---|---|---|
| 1 | Preset row | "Preset" dropdown (stretches), then four `icon`-sized square icon buttons: Save preset…, Delete preset, Import presets…, Export presets… (labels as tooltips) |
| 2 | Settings | two group boxes side by side: "Rows" and "Columns" |
| 3 | Run | the job buttons grid and the Pattern size read-out (shared parts) |
| 4 | Parts row | three wide buttons with swatch chips, then two switches |
| 5 | Footer | Close |

**Preset row**

- The dropdown's first row is "(none)" in `text-muted`; then a
  non-selectable heading "Built-in" in `type-small`, `text-muted`, our own
  built-in presets (own data, from the M8 material work; at least "Diode
  engraving" and "Diode cutting", proposed labels), then a heading
  "Yours" (proposed label) and the user's presets (Echo [R2] of the two
  headed sections; not the reference's dash-padded separators).
- Choosing a preset overwrites every field; it is not undoable but the
  previous values are kept until the window closes, so choosing "(none)"
  brings them back (own design).
- Save preset… opens a small dialog "Save preset" with "Name" and Save,
  Cancel. Delete preset asks "Delete preset?" with "Delete preset"
  (`danger`) and Cancel; built-in presets cannot be deleted (button
  disabled, second tooltip line "Built-in presets cannot be deleted.").
  Import presets… and Export presets… read and write LaserCraft's own
  preset file (format set by the M8 spec).
- Icon concepts (`icons.md`): a tray with an arrow down (save), a trash
  can (delete), a page with an arrow pointing in (import), a page with an
  arrow pointing out (export). The reference's title field is not copied
  (its purpose is unknown [R2]; Open questions).

**Rows and Columns groups** (same rows in each; proposed labels)

| # | Label | Widget | Rows default | Columns default | Range |
|---|---|---|---|---|---|
| 1 | Count | whole number | 10 | 10 | 2–20 |
| 2 | Setting | dropdown: Speed, Power, Interval, Passes | Speed | Power | the two groups cannot pick the same setting |
| 3 | From | number with the setting's unit in the label ("From (mm/s)", "From (%)", "From (mm)", "From") | 10 | 10 | the setting's range in `docs/spec/layers.md` |
| 4 | To | as From | 100 | 100 | as From |
| 5 | Box height (mm) / Box width (mm) | number | 5 | 5 | 1–100 |
| 6 | Center Y (mm) / Center X (mm) | number | bed center | bed center | inside the work area; disabled unless Place job at is Absolute coordinates |

- Labels right-aligned, one column of pairs per group (Echo [R2]).
- The From and To labels change their unit with Setting, so speed is
  always typed in mm/s (the reference warns about mixed speed units [R2];
  LaserCraft has one speed unit, `docs/spec/layers.md`).
- When Setting is Power, To is limited to the device's Max power (%) and
  the clamp warning shows (safety rule 6).
- Run order: the box with the least energy runs first (fastest speed,
  lowest power, smallest interval, fewest passes), then increasing (Match
  [R2]); the planner decides this, not the UI.

**Parts row**

| Button (proposed label) | Chip | Opens |
|---|---|---|
| Box settings… | the palette's color 00 | the Layer settings window (`layers-panel.md`) on a temporary layer whose settings fill every box; the two varied settings are overridden per box |
| Label settings… | the palette's color 01 | the same window for the value labels and axis titles |
| Border settings… | the palette's color 02 | the same window for the border around the grid |

- Each button is an ordinary-height button the width of a third of the
  row, starting with a swatch chip (`layers-panel.md` chip look, number
  and contrast rule) (Echo [R2]).
- Under Label settings… and Border settings…: switches "Draw labels" and
  "Draw border" (proposed labels), both on by default; off disables the
  matching button.
- The temporary layers are not added to the project's layer list; the job
  preview draws them in their chip colors.

**What the pattern contains** (planner output; the UI only previews it)

- A grid of Count × Count boxes, each filled with the base settings and
  the two varied values; values printed along the left edge for rows and
  the bottom edge for columns, each axis with its setting's name and unit;
  one line above the grid listing the settings that do not vary (Echo
  [R2]; text in LaserCraft's single-stroke label font chosen by the M8
  spec).

Tier: **Match** (opened from the machine menu, rows and columns groups and
their fields, run order, parts buttons with chips, switches for labels and
border, job buttons), **Echo** (preset sections, size read-out), **Own**
(labels, Close-only footer, remembered values, built-in presets, clamp
warning, layout of the job buttons in a "Run" group).

### Line interval test

#### Reference

- A small dialog, a little wider than tall, opened from the machine tools
  menu [R3][R15]; only an older screenshot exists [R3].
- Two columns of label, field and unit: speed, power and number of samples
  on the left; smallest interval, largest interval and sample size on the
  right; a separator; two radio choices for solid squares or dithered
  gradient squares; a row with preview, frame and start; a row with send
  and save; OK keeps the settings, Cancel reverts [R3].
- The result is a row of squares at different line intervals, each with
  its value printed beside it; placement follows the start-from mode and
  absolute placement centers it [R3].

#### LaserCraft

Title "Line interval test" (naming.md). Opened by Machine → Line interval
test… (`menus.md`). A small movable, non-modal window. Milestone: M8.

| # | Part | Contents |
|---|---|---|
| 1 | Settings | two columns of label and field pairs (below) |
| 2 | Separator | thin `border` line |
| 3 | Patch fill | choice dots in one row: "Solid", "Gradient" (proposed labels) |
| 4 | Run | job buttons and the Pattern size read-out (shared parts) |
| 5 | Footer | Close |

Settings (proposed labels except Speed and Max power, which are
naming.md's layer labels):

| Column | # | Label | Default | Range |
|---|---|---|---|---|
| left | 1 | Speed (mm/s) | 100 | as `docs/spec/layers.md` |
| left | 2 | Max power (%) | 30 | 0 to the device's Max power (%) |
| left | 3 | Patches | 8 | 2–20 |
| right | 4 | Smallest interval (mm) | 0.05 | 0.001–10, below Largest |
| right | 5 | Largest interval (mm) | 0.3 | 0.001–10 |
| right | 6 | Patch size (mm) | 10 | 2–50 |

- Each interval field shows a `type-small`, `text-muted` hint under it
  with the equivalent "‹n› lines per inch" (the layer editor's hint,
  `layers-panel.md`).
- Solid fills each patch with a fill scan; Gradient fills it with a
  dithered left-to-right ramp from white to black using the layer's image
  settings (own wording of the reference's idea [R3]).
- The pattern is a row of patches with each interval printed under its
  patch in `type-mono` digits on the engraving (Echo [R3]).
- The reference's send button is omitted (decision 1); its device-format
  save is our Save G-code….

Tier: **Match** (two columns of settings, fill choice, job buttons, result
layout), **Own** (labels, hints, Close-only footer).

### Focus height test

#### Reference

- A small dialog, wider than tall, opened from the machine tools menu,
  where its row is greyed when the device has no motorized Z axis
  [R4][R15]; only older screenshots exist [R4].
- Two columns: start height, end height and speed on the left; maximum
  power and steps on the right (lines = steps + 1); a switch for
  high-power lasers that turns the labels into a perforated cut; a row of
  preview, frame, machine-file save and a wider start; OK and Cancel with
  the same keep and revert meaning as the interval test [R4].
- Without a Z axis enabled in device settings, a plain message names the
  setting to turn on, with a single OK [R4][R32].
- The result is a set of lines, each at its own height with that height
  printed under it; placement follows the start-from mode [R4].

#### LaserCraft

Title "Focus height test" (naming.md). Opened by Machine → Focus height
test… (`menus.md`), which is enabled only when the device profile has
Motorized Z axis on; its disabled tooltip's second line reads "Needs a
motorized Z axis in the device profile." Milestone: M8.

| # | Part | Contents |
|---|---|---|
| 1 | Settings | two columns of label and field pairs (below) |
| 2 | Switch | "Dashed labels" (proposed label), right-aligned under the fields |
| 3 | Run | job buttons and the Pattern size read-out (shared parts) |
| 4 | Footer | Close |

Settings (proposed labels except Speed and Max power):

| Column | # | Label | Default | Range |
|---|---|---|---|---|
| left | 1 | Z from (mm) | −2 | −20 to 20 |
| left | 2 | Z to (mm) | 2 | −20 to 20, not equal to Z from |
| left | 3 | Speed (mm/s) | 20 | as `docs/spec/layers.md` |
| right | 4 | Max power (%) | 30 | 0 to the device's Max power (%) |
| right | 5 | Lines | 9 | 2–41 |

- Z values are offsets from the head's Z height when Start is clicked
  (own design: GRBL has no "material height" of ours to measure from; the
  reference measures from its layer material height [R4]). The read-out
  under the fields says "Line spacing ‹d› mm of Z" (proposed label).
- "Dashed labels" tooltip: "Cut the height labels as dashes so a strong
  laser does not burn them through" (own wording of [R4]).
- The pattern is a row of short lines, each at its Z, its Z offset printed
  under it (Echo [R4]).
- Z moves follow the Z jog rules of `machine-panels.md` (Z jog distance and
  speed); between lines the laser is off (safety rule 4).

Tier: **Match** (fields, labels switch, job buttons, Z-axis gate),
**Own** (labels, Z reference, read-out, Close-only footer).

### No Z axis (message)

#### Reference

- A plain message titled after the missing Z setting, with one line
  pointing to the device-settings place to enable it, an OK button at the
  bottom right and no icon in the body [R4].

#### LaserCraft

Title "No Z axis" (naming.md). M8.

- Shown only when the Focus height test is opened another way while the
  profile has no motorized Z axis (for example by typing its command in
  the Command line); the menu row itself is disabled (above).
- Body, one paragraph (naming.md, Messages: what happened, why, what to
  do): "Cannot open Focus height test: the device profile ‹name› has no
  motorized Z axis. Turn on Motorized Z axis in Edit device profile… →
  Work area, then try again."
- Footer: "Edit device profile…" (opens the Device profile dialog on its
  Work area tab, `machine-panels.md`) and "Close" (default).
- A `warning` badge at the left of the body (own design; the reference
  shows none [R4]).

Tier: **Match** (exists, points to the setting), **Own** (wording, badge,
the button that opens the setting).

---

## Image windows (M7)

### Image adjustments

#### Reference

- A large resizable window with the operating system's minimize, maximize
  and close buttons, landscape, opened by double-clicking an image, from
  its right-click menu, or with Alt+I [R5]; the tools menu shows the same
  chord [R35].
- Top, most of the height: two picture panes side by side with a thin line
  between them, the original on the left and the processed result on the
  right exactly as it will be engraved; dragging pans and the wheel zooms,
  both panes in step; under each, its pixel size; under the right pane a
  switch that shows the result inverted for marks that come out light
  [R5].
- A middle row: a reset-all button at the left; at the right, a presets
  dropdown with four small icon buttons (save, delete, import, export)
  [R5].
- Bottom: two framed groups. The narrower one mirrors the image settings
  of the layer and changes the whole layer: image mode, a negative switch,
  halftone cells per inch, line interval, halftone angle and resolution.
  The wider one changes only this image: contrast, brightness and gamma in
  one column, two sharpening settings in the other, each a slider with a
  number field [R5].
- OK (default) and Cancel at the bottom right [R5].
- Presets list: a separator for built-in presets, two built-in entries (a
  basic start and an inverted one), a separator for user presets, then the
  user's; choosing one overwrites the settings [R5].

#### LaserCraft

Title "Image adjustments" (naming.md). Opened by Tools → Image
adjustments… (`menus.md`; not bound: keymap.md lists no chord, Proposed
changes), the image's canvas menu (`menus.md`), double-clicking an image
on the canvas, and the button "Image adjustments…" under the image rows of
the Properties panel (`property-bar.md`). Enabled when exactly one image
is selected. A resizable window, landscape, opening at about two thirds of
the main window's size. Milestone: M7.

| # | Part | Contents |
|---|---|---|
| 1 | Picture panes | two panes side by side: "Original" and "As engraved" (proposed captions), each with a size line under it |
| 2 | Pane options | under the right pane, right-aligned: switch "Show inverted" |
| 3 | Preset row | left: "Reset all"; right: "Preset" dropdown and four icon buttons |
| 4 | Settings | two group boxes side by side: "For the whole layer" (narrower) and "For this image only" (wider) |
| 5 | Footer | OK (default), Cancel |

**Picture panes**

- Each pane is a `field` area with a thin `border` (own design; the
  reference draws them on the bare window background [R5]), its caption in
  `type-small`, `text-muted` above it.
- "As engraved" shows the processed image with the layer's method (for
  example Floyd–Steinberg) at the layer's interval, exactly as the
  planner will burn it; it redraws a short moment after the last change
  so dragging a slider stays smooth (own design).
- Drag pans and the wheel zooms about the pointer; the two panes always
  show the same region (Match [R5]). Middle-button double-click fits
  (own design, as in Trace to vectors [R6]).
- Under each pane, centered, `type-small`, `text-muted`: "‹w› × ‹h› px"
  and, for the right pane, "at ‹n› dpi".
- Show inverted: preview only, light marks on dark; tooltip "Preview light
  marks on a dark surface; the job is unchanged" (Echo [R5]).

**Preset row**

- Reset all: "Put every setting in this window back to how it was when the
  window opened" (own meaning; ordinary button).
- Preset dropdown: first row "(none)", heading "Built-in" with our own
  presets ("Photo, general" and "Photo, light marks on dark", proposed
  labels, own data), heading "Yours", then the user's (Echo [R5]).
- Icon buttons, square, `icon` size, in this order (naming.md labels as
  tooltips): Save preset… (opens Save adjustment preset), Delete preset,
  Import presets…, Export presets…. Delete asks "Delete preset?" with
  "Delete preset" (`danger`) and Cancel.

**For the whole layer** (changes the layer; the same fields as the
layer's Images on this layer section, `layers-panel.md`)

| Line | Left pair | Right pair |
|---|---|---|
| 1 | Method — dropdown: Grayscale, Threshold, Floyd–Steinberg, Jarvis, Stucki, Ordered (Bayer), Halftone | Negative — switch |
| 2 | Halftone frequency (lpi) — enabled only with Halftone | Interval (mm), hint "‹n› dpi" |
| 3 | Halftone angle (°) — enabled only with Halftone | (empty) |

- A one-line `info-soft` note at the group's top: "Changes every image on
  layer ‹name›." (own design).
- Method names are the readable ones from `naming.md`, never stored ids.

**For this image only** (the Properties panel's image rows,
`property-bar.md`)

| Line | Left column | Right column |
|---|---|---|
| 1 | Brightness | Sharpen radius |
| 2 | Contrast | Sharpen amount |
| 3 | Gamma | Denoise |

- Each row: label, slider, number field (Shared look). Ranges and neutral
  values come from the M7 image spec; double-clicking a slider puts it
  back to its neutral value (own design).
- Denoise is LaserCraft's addition (naming.md); the reference shows two
  sharpening rows only [R5].

**Footer and commands**

- OK applies both groups as one undoable command (layer settings and the
  image's settings together); Cancel discards everything (Match [R5]).
- While the window is open, the canvas keeps showing the image as it was;
  the panes are the preview (own design).

Tier: **Match** (place of entry points, two panes, presets, two groups
split by scope, OK/Cancel), **Echo** (inverted preview, size lines),
**Own** (labels, framed panes, built-in presets, Denoise, delayed redraw,
info note).

### Save adjustment preset

#### Reference

- Opened by the save-preset icon. A name field at the top right, a short
  centered instruction that unchecked items keep their current value when
  the preset is loaded, two framed groups mirroring the two settings
  groups with one switch per setting in two columns, a row to switch all,
  none or invert, then Save (default) and Cancel [R5].

#### LaserCraft

Title "Save adjustment preset" (naming.md). M7.

| # | Part | Contents |
|---|---|---|
| 1 | Name | label "Name" and a text field, full width |
| 2 | Hint | one line in `text-muted`: "Settings switched off here are left as they are when the preset is used." |
| 3 | Groups | "For the whole layer": switches Method, Negative, Halftone frequency, Interval, Halftone angle; "For this image only": switches Brightness, Contrast, Gamma, Sharpen radius, Sharpen amount, Denoise |
| 4 | Quick choice | right-aligned: lead-in "Switch on" in `text-muted`, buttons "All", "None", "Invert" (proposed labels) |
| 5 | Footer | Save (default), Cancel |

- Every switch is on by default. Each switch's label is the setting's
  label without its unit.
- Save is enabled when Name is not empty; a name that already exists asks
  "Replace preset?" with "Replace preset" and Cancel.

Tier: **Match** (name, two groups of switches, all/none/invert), **Own**
(labels, hint wording).

### Trace to vectors

#### Reference

- A resizable window, very wide and short, opened from the tools menu,
  the image's right-click menu, with Alt+T, or from the camera window's
  trace button [R6][R9][R35].
- Most of the height is a preview: the image with the would-be outlines
  drawn over it in a thin bright color, updating live; zoom and pan as on
  the canvas; dragging draws a boundary rectangle with corner handles that
  limits the trace to that area [R6].
- Under a separator, rows of controls: a lower and an upper brightness
  bound, each a number field and a slider; a switch to trace by
  transparency; a switch for an edge-finding mode meant for sketches and
  unevenly lit photos; then a speck-size limit, a smoothness value, a node
  reduction value, three buttons (fade the image, show the nodes, clear
  the boundary), a switch to delete the image afterwards, and OK and
  Cancel at the far right [R6].
- The default bounds are the lower half of the brightness range; the
  result is one group of paths [R6].
- A separate experimental command traces several files at once with
  default settings and writes them beside the sources, with no window of
  its own [R34].

#### LaserCraft

Title "Trace to vectors" (naming.md). Opened by Tools → Trace to vectors…
(`menus.md`; not bound, Proposed changes), the image's canvas menu, and
Trace camera image… in the Camera panel (M9). Enabled when exactly one
image is selected (or a camera picture exists, for the camera route). A
resizable landscape window. Milestone: M7.

| # | Part | Contents |
|---|---|---|
| 1 | Preview | the image with the traced outlines over it, most of the height |
| 2 | Separator | thin `border` line |
| 3 | Range row | "Brightness from" field and slider; "Brightness to" field and slider |
| 4 | Mode row | switches "Use transparency", "Follow edges (sketches)" |
| 5 | Clean-up row | "Drop specks under (px)", "Smoothness", "Node reduction" (number fields) |
| 6 | View row | toggle buttons "Dim image", "Show nodes"; button "Clear trace area"; switch "Delete image when done" |
| 7 | Footer | OK (default), Cancel |

- Rows 3 to 6 are laid out in two columns on wide windows and one column
  on narrow ones (own design; the reference packs them into three long
  rows [R6], which do not fit a narrow window).
- **Preview look**: the image on a `field` background; traced outlines in
  `drawing` (solid here, not dashed, because they are the main content of
  the window); nodes, while Show nodes is on, as `node` squares; the trace
  area as a `marquee` rectangle with `handle-fill` corner handles
  (own design; the reference uses bright magenta outlines and a reddish
  boundary [R6], which would clash with our layer colors and `danger`).
- Dim image fades the picture so the outlines stand out; Show nodes shows
  node squares. Both are latching toggle buttons: pressed look
  (`accent-soft` face) while on (own design; whether the reference's are
  latching is unknown [R6]).
- Drag in the preview draws the trace area; Clear trace area removes it;
  without an area the whole image is traced (Match [R6]).
- Pan and zoom as on the canvas (`canvas.md`); middle-button double-click
  fits the image.
- Fields (ranges and defaults are the M7 spec's; the defaults below are
  proposals):

| Label | Default | Tooltip (first / second line) |
|---|---|---|
| Brightness from | 0 | "Darkest brightness that counts as ink" / "0 is black, 255 is white." |
| Brightness to | 128 | "Lightest brightness that counts as ink" |
| Use transparency | off | "Trace the image's transparent edges instead of its brightness" |
| Follow edges (sketches) | off | "Trace along local contrast; good for sketches and unevenly lit photos" |
| Drop specks under (px) | 2 | "Ignore traced bits smaller than this many image pixels" |
| Smoothness | 1.0 | "0 gives only straight lines; higher values give more curves" |
| Node reduction | 0.2 | "Higher values use fewer nodes and follow the image less closely" |
| Delete image when done | off | "Remove the image after the trace is added" |

- OK adds one group of paths on the current layer as one undoable command
  (deleting the image in the same command when the switch is on) (Match
  [R6]). Cancel adds nothing.
- Trace several files… (Tools menu, later) has no window: it opens the
  file chooser and writes one SVG per image beside it with the last used
  settings (Echo [R34]).

Tier: **Match** (preview with live outlines, range, modes, clean-up
fields, view helpers, delete-after switch, trace area), **Own** (labels,
colors, two-column layout, latching look).

### Convert to image

#### Reference

- Opened from the edit menu with a chord, or from the right-click menu
  [R7]; the edit menu's screenshot and page text name different chords
  for it [R36].
- A large white preview pane with a thin border, updating live; at the
  bottom left a framed group with three radio choices (outlines only,
  everything filled, or by each shape's layer mode); at the bottom right a
  large resolution number with a large unit beside it and a slider with
  its minimum and maximum printed at the ends; OK and Cancel [R7].
- The new image is mid-grey, the vectors are deleted, and the image lands
  on the last selected layer [R7].

#### LaserCraft

Title "Convert to image" (naming.md). Opened by Edit → Convert to image…
(Ctrl+Shift+B (M7), keymap.md) and the canvas menu. Enabled when
something is selected. A resizable window, about square. Milestone: M7.

| # | Part | Contents |
|---|---|---|
| 1 | Preview | `field` pane with a thin `border`: the selection rendered at the chosen resolution, live |
| 2 | Draw as | group box "Draw as" with choice dots in a column: "Strokes only", "Filled", "By layer mode" |
| 3 | Resolution | group box with "Resolution (dpi)" number field and a slider under it, its range ends printed in `type-small`, `text-muted` |
| 4 | Read-out | "‹w› × ‹h› px, about ‹n› MB" in `type-small`, `text-muted` (own design) |
| 5 | Footer | OK (default), Cancel |

- Groups 2 and 3 sit side by side under the preview (Match [R7]); the
  resolution number is an ordinary field, not the reference's oversized
  one (own design).
- Default Draw as: By layer mode; default resolution: the target layer's
  image interval expressed as dpi (`docs/spec/layers.md`: dpi = 25.4 /
  interval) (own design).
- OK replaces the selected shapes with one image on the last selected
  shape's layer, grey as rendered, in one undoable command (Match [R7]).

Tier: **Match** (preview, three render choices, resolution with slider,
replace on OK), **Own** (labels, size read-out, defaults).

---

## Camera (M9)

All camera windows follow one model: a **camera** belongs to a device
profile and has a source (USB or network picture stream), a lens
correction and a bed alignment (Echo [R8][R9][R11]). The Camera panel is
where it is used; Camera list, Add camera, Camera setup and the two
wizards are where it is set up. Camera files, calibrations and alignments
use LaserCraft's own formats (set by the M9 spec); the reference's camera
files are never read (`docs/clean-room.md`).

### Camera panel

#### Reference

- The reference's camera window floats by default and can be docked; in a
  docked example it is a tab of the right column's upper group [R9][R14].
  The Window menu lists it [R24]; version 2.1 rebuilt the camera interface
  [R25].
- Top: two equal wide buttons with large glyphs: update the overlay
  (grab a picture and paint it under the workspace; with a selection, only
  that area; a head-mounted camera jogs and stitches several pictures) and
  trace the overlay (opens the trace window on the picture) [R9][R14].
- Overlay controls, in two variants: always-visible sliders for
  resolution and opacity plus width, height and X/Y shift fields; or those
  size and shift fields in a collapsed bar with only an opacity slider left
  visible below it [R9].
- A framed list with one block per camera. Its header has a disclosure
  triangle, a small green dot while the camera streams, a per-camera
  update button, an eye that includes or excludes the camera from the
  combined overlay (crossed out on red when excluded), the bold name, and
  three status glyphs at the right for connection, lens calibration and
  alignment, each red when missing and each opening the matching window or
  wizard [R9].
- Under the header, when expanded and connected, the live picture; a
  collapsed bar of camera settings (exposure, focus and white balance each
  with an automatic switch, then brightness, contrast, hue and backlight,
  then a reset button); and an advanced bar (preview frame rate with the
  current resolution, and buttons that insert the current picture into the
  workspace with or without lens correction) [R9].
- At the bottom: add a camera and import a camera, same wide style [R9].
- Right-clicking a camera's picture gives edit, remove and export; elsewhere
  a camera-buying helper, add and import [R9]. Under the workspace the
  overlay fades as opacity drops [R9].

#### LaserCraft

Panel title "Camera" (naming.md). Place: right dock, upper group, tab 7,
hidden until a camera is set up, then shown (`layout.md`). Milestone: M9.

| # | Part | Contents |
|---|---|---|
| 1 | Action row | two equal buttons, one `row` and a half tall, icon left of label: "Capture camera image", "Trace camera image…" |
| 2 | Opacity | "Overlay opacity (%)" slider and field, always visible |
| 3 | Overlay fit | collapsing section "Overlay fit", closed by default: "Overlay width (%)", "Overlay height (%)", "Overlay offset X (mm)", "Overlay offset Y (mm)" in two columns |
| 4 | Picture age | one line in `type-small`, `text-muted`: "Picture taken ‹time›" or "No picture yet" (proposed label) |
| 5 | Camera list | a `field` frame holding one block per camera (below), scrolling when taller than the panel |
| 6 | Bottom row | two equal buttons like row 1: "Add camera…", "Import camera…" |

- Row 1 buttons: Capture camera image (tooltip "Take a new camera picture
  for the overlay", chord Alt+C (M9) once bound; icon: camera with a small
  circular arrow, the same as the Standard toolbar's, `toolbars.md`);
  Trace camera image… (tooltip "Trace the camera picture into shapes";
  opens Trace to vectors on the picture; icon: camera with a pen). Trace
  camera image… is disabled until a picture exists ("Capture a camera
  picture first.").
- With a selection on the canvas, Capture camera image refreshes only the
  part of the overlay under the selection (Match [R9]); its tooltip's
  second line says so while something is selected.
- Overlay opacity and Overlay fit change only the view (front-end
  settings stored with the camera), not the project (own design).
- Picture age answers the canvas's stale-picture question (`canvas.md`):
  a picture taken before the last jog, frame or job reads "Picture taken
  before the head last moved" in `text`, led by an `icon-small` warning
  mark in `warning` (`visual-language.md`: `warning` is never a text
  color in the light theme; own design).
- No overlay resolution control: the reference shows an overlay
  resolution slider beside opacity in one variant [R9], but LaserCraft
  derives the overlay's scale from the camera's lens correction and bed
  alignment, so a second, hand-set resolution could only disagree with
  them (own design; listed under Deliberately omitted).

**Camera block** (one per camera of the active device profile)

- **Camera header** (naming.md, no visible label): a `panel-title` strip
  across the block:

| # | Item | Look | Interaction |
|---|---|---|---|
| 1 | Disclosure marker | `icon-small` | opens or closes the block |
| 2 | Streaming dot | small `success` dot while frames arrive; absent otherwise | tooltip "Receiving pictures" |
| 3 | Capture this camera | `icon-small` button, camera with circular arrow | captures only this camera's part of the overlay |
| 4 | Use in overlay | `icon-small` toggle: open eye when used; crossed eye in `text-muted` when not | tooltip "Use this camera in the overlay" (proposed label) |
| 5 | Name | the camera's name in `type-body`, left-aligned after the icons | double-click renames (own design) |
| 6 | Status badges | three `icon-small` pictograms at the right end: Source (a plug), Lens (a lens), Alignment (a framed bed) | each carries a corner badge: `success` check when set, `warning` mark when missing, `danger` cross when the source failed; clicking opens Camera setup at that section (Source) or the matching wizard (Lens, Alignment) |

- The badges have tooltips naming state and fix: "Source: connected",
  "Lens: not calibrated. Click to calibrate.", "Alignment: missing. Click
  to align." (own wording). The reference's red-background look for a
  missing item is not used; red stays for failures (README.md, semantic
  colors).
- **Live picture**: under the header, full block width, the lens-corrected
  stream at the panel's width; shown only while the block is open and the
  source is connected. Right-click opens the Camera menu (`menus.md`:
  heading, Edit…, Remove, Export…).
- **Picture settings** (naming.md): collapsing section, closed by default.
  One row per control, each a label, a slider and a number field, three of
  them with an "Auto" switch after the field (proposed labels):

| Row | Label | Auto switch | Notes |
|---|---|---|---|
| 1 | Exposure | yes | field and slider disabled while Auto is on |
| 2 | Focus | yes | as above |
| 3 | White balance | yes | as above |
| 4 | Brightness | no | |
| 5 | Contrast | no | |
| 6 | Hue | no | |
| 7 | Backlight compensation | no | |
| 8 | "Restore defaults" button, centered | | sets every row back to the camera's own defaults |

  Rows appear only for controls the camera reports (USB Video Class
  controls); a camera without any shows "This camera has no adjustable
  controls." in `text-muted` (own design; unknown for the reference [R9]).
- **Advanced** (naming.md): collapsing section, closed by default:
  "Preview rate" dropdown (1, 2, 5, 10, 15 frames per second; proposed
  label) with the current resolution "at ‹w› × ‹h›" in `text-muted` after
  it; "Insert picture" lead-in with two buttons "Corrected" and "As
  captured" (proposed labels) that place the current picture on the canvas
  as an image, with or without lens correction (undoable command).
- Right-click elsewhere in the panel: the second part of the Camera menu
  (`menus.md`: Add camera…, Import camera…). The reference's camera-buying
  helper is omitted (vendor product chooser, naming.md Omitted) [R33].
- Empty state (no camera for this profile): the naming.md line "No camera
  set up. Use Add camera… to add one." above the bottom row; rows 1–5 are
  hidden.
- Two or more cameras: the overlay combines every camera with Use in
  overlay on (Echo [R9]).

Tier: **Match** (place in the upper group, action row, opacity and fit
controls, per-camera blocks with header, live picture, picture settings,
advanced options, add and import at the bottom), **Echo** (status badges,
include toggle), **Own** (labels, badge colors and words, picture age,
"no controls" line, view-only overlay settings).

### Camera list

#### Reference

- The machine tools menu has an entry that opens a dialog listing the
  device's cameras: a caption, a white list whose rows show a large device
  glyph with the name and connection kind on two lines (selected row light
  grey), a status line, three equal buttons to add, edit and remove, then
  OK and Cancel [R15].

#### LaserCraft

Title "Camera list" (naming.md). Opened by Machine → Camera list…
(`menus.md`, always enabled). M9.

| # | Part | Contents |
|---|---|---|
| 1 | Caption | "Cameras for ‹device profile›" in `type-heading` (proposed label) |
| 2 | List | `field` list, two lines per camera: line 1 the name; line 2 in `text-muted` "USB · ‹device name›" or "Network · ‹address›"; at the left an `icon` pictogram of a plug or a network socket; at the right the three status badges of the Camera header |
| 3 | Status line | for the selected camera: "Connected at ‹w› × ‹h›", "Connecting…" or "No picture source", with the matching badge |
| 4 | Buttons | three equal buttons: "Add camera…", "Edit…", "Remove" |
| 5 | Footer | Close |

- Edit… (or double-click) opens Camera setup; Remove asks "Remove
  camera?" / "The camera "‹name›", its lens correction and its alignment
  are removed from this device profile." with "Remove camera" (`danger`)
  and Cancel. Edit… and Remove need a selection.
- Footer is Close only: every change here is already its own command, and
  Camera setup has its own OK (own design; the reference has OK and Cancel
  [R15]).
- Empty: "No camera set up. Use Add camera… to add one." (naming.md).

Tier: **Match** (list with two-line rows and pictograms, status line, add,
edit and remove), **Own** (labels, badges in the list, Close-only).

### Add camera (wizard)

#### Reference

- A large landscape wizard with a white page and a light bottom strip
  holding skip (on calibration and alignment pages), back, next or finish,
  and cancel at the right; back is disabled on the first page and next
  until a page is complete [R10][R12].
- Pages: choose the camera kind from three option cards (USB, network, a
  preset for a known machine), each with a large glyph, a bold title, one
  line of description and a select button [R10]; find the source: on the
  left a framed box with a large status icon, device and format dropdowns
  (or a stream address and frame rate for network cameras) and a status
  line, then key and value facts about the stream; on the right a large
  live preview [R10]; choose the mounting position from two large cards
  with simple drawings, above the bed or on the head [R14]; choose a lens
  calibration: create a new one, load one from a file, or use a camera
  known to need no correction, plus vendor presets [R10]; then the lens
  wizard, then the alignment wizard, then finish [R10].
- Importing a camera file opens the edit window with all three items
  marked good [R10].

#### LaserCraft

Title "Add camera" (naming.md). Opened by Add camera… (Camera panel,
Camera list, Camera menu). A resizable landscape window. Wizard chrome per
`dialogs.md` (summary above). M9.

| Page | Heading (`type-heading`, centered) | Contents | Next enabled when |
|---|---|---|---|
| 1 | "What kind of camera?" | two option cards (below): "USB camera" ("A camera on a USB cable"), "Network camera" ("A camera that serves pictures over the network") | a card is chosen |
| 2 | "Choose the picture source" | left half: Source box (below); right half: "Live view" pane | the source delivers pictures |
| 3 | "Where is the camera mounted?" | two option cards side by side: "Above the bed" ("Fixed over the work area; sees the whole bed"), "On the laser head" ("Moves with the head; sees a small area") | a card is chosen |
| 4 | "Lens correction" | option cards: "Calibrate now", "Load from file…", "No lens correction" (proposed labels) | a card is chosen |
| 5 | (the Camera lens calibration pages, unless page 4 chose otherwise) | | |
| 6 | (the Camera alignment pages) | Skip allowed: the camera is then added without alignment | |
| 7 | "Camera added" | a `success` badge, the camera's name field (prefilled "‹kind› camera ‹n›"), a summary of source, mounting, lens and alignment | Finish |

- **Option cards** (Echo [R10][R14]): full-width rounded rectangles with
  `field-alt` fill and a thin `border`; an `icon-big` pictogram at the left
  (`icons.md`: plug, network socket, camera over a bed, camera on a head,
  lens, folder); the title in `type-body` (not bold: bold is kept for
  headings), the description under it in `type-small`, `text-muted`. The
  whole card is clickable; the chosen card gets an `accent` outline,
  `accent-soft` fill and a filled choice dot at its left (own design; the
  reference uses a per-card select button [R10]).
  Mounting cards carry a simple own line drawing instead of an icon.
- **Source box** (page 2, Echo [R10]): a group box "Source" holding, left
  to right, a large status badge (`success`, `warning` while connecting,
  `danger` when no source), then for USB: "Device" dropdown (cameras found
  on this computer; first item "None") and "Format" dropdown (resolution,
  pixel format and frame rate); for network: "Address" text field (a
  still-picture or motion-JPEG URL) and "Pictures per second" number
  field; under them "Status: Connected | Connecting… | No picture source"
  (proposed labels). Under the box, a facts list with keys in `text-muted`:
  Resolution, Rate, Camera controls (proposed labels).
- **Live view** pane: `field` frame; while connecting, a dark band with
  "Connecting…" centered in `type-body`, `text-muted`; with no source, the pane is
  empty `field` with "No picture source" in `text-muted` (Echo [R11]).
- Vendor presets for known machines are omitted (vendor data, decision 1);
  so are vendor lens presets on page 4.
- Import camera… (not a wizard page): opens the file chooser for
  LaserCraft's own camera file, then Camera setup on the imported camera.

Tier: **Match** (page order and contents, option cards, source box with
live view, mounting choice, lens choice), **Echo** (card look), **Own**
(labels, chosen-card look, last page with name and summary, no vendor
presets).

### Camera setup

#### Reference

- A large landscape window that edits one camera, opened from the camera
  list, the camera's right-click menu, or after an import [R11]. Left
  column (about two fifths): four sections, each a bold heading over a
  framed box: the mount kind with a large source glyph, a name field and
  an export button, and lines for camera type and mount type; the camera
  source with a large status sign, device and format dropdowns (or a
  stream address), a status line and a collapsed source-info bar; the lens
  calibration with its status, a dropdown of the current calibration (or
  none) and a remove button, import and calibrate buttons, and an info
  bar; the workspace alignment with the same parts and an align button
  [R11].
- Right column: a large preview of the corrected picture, overlaid with a
  thin grid when calibrated and aligned; a dark band with large
  "connecting" text while connecting; blank with no source; under it a
  collapsed bar of advanced settings (how to identify a USB device, how
  many warm-up frames to drop, two experimental options) [R11].
- Finish and Cancel at the bottom [R11]. Alignments are kept per material
  height; the alignment must be redone when the camera moves, the lens
  calibration need not [R8][R13].

#### LaserCraft

Title "Camera setup" (naming.md), with the camera's name after a colon:
"Camera setup: ‹name›" (the Device profile dialog's pattern,
`machine-panels.md`). A resizable landscape window. M9.

Left column, four group boxes (headings in `type-heading`; proposed
labels):

| # | Group | Contents, in order |
|---|---|---|
| 1 | "Camera" | `icon-big` plug or network socket; "Name" field; "Export…" button; lines "Kind: USB camera" and "Mounted: above the bed" in `text-muted` |
| 2 | "Source" | status badge; for USB "Device" and "Format" dropdowns, for network "Address" field; status line; collapsing section "Source details" (Resolution, Rate, Camera controls) |
| 3 | "Lens correction" | status badge; dropdown of stored corrections (named "‹camera› ‹date and time›") or "None"; "Remove"; buttons "Import…", "Calibrate lens…"; collapsing section "Correction details" (name, camera, model, original resolution) |
| 4 | "Bed alignment" | status badge; dropdown of stored alignments (named with their material height when one is set) or "None"; "Remove"; buttons "Import…", "Align to bed…"; collapsing section "Alignment details" (name, lens correction used, mounting, material height, picture resolution) |

Right column:

| # | Part | Contents |
|---|---|---|
| 5 | "Live view" | the corrected picture; when lens correction and bed alignment are both set, a thin `grid-major`-style square grid over it in bed millimetres (own design: grid spacing shown in a corner label "grid 10 mm"); connecting band and empty state as in Add camera |
| 6 | "Advanced" | collapsing section, closed: "Identify device by" (choice dots "Port", "Name"), "Skip first pictures" (whole number, default 3), and experimental switches, each ending in "(experimental)" (naming.md), for any option the M9 spec adopts |
| 7 | Footer | OK (default), Cancel |

- Remove in group 3 or 4 asks first (naming.md, Confirmations) with
  "Remove correction" or "Remove alignment" (`danger`). Importing a lens
  correction that differs from the one the current alignment was made with
  shows a `warning-soft` band "This alignment was made with another lens
  correction. Align to bed again." (Echo [R11]).
- Export… writes LaserCraft's own camera file with source, correction and
  alignments.
- OK applies every change in the window as one change to the device
  profile; Cancel discards them (own design; the reference's footer says
  Finish [R11]).
- Calibrate lens… and Align to bed… run the two wizards inside this window
  (the pages replace the two columns until they finish), as the reference
  does [R12][R13].

Tier: **Match** (two columns, four sections and their parts, live view
with grid, advanced section), **Echo** (status badges, details sections),
**Own** (labels, title with name, OK/Cancel, warning band wording).

### Camera lens calibration (wizard)

#### Reference

- Runs inside the add or edit window; opened from the calibrate button or
  the lens status glyph [R12].
- Welcome page: a centered bold heading, a subtitle and an estimated time;
  three step cards side by side, each a rounded square with a numbered
  badge and a simple line drawing (get the printed pattern, capture
  pictures, choose a model), with a caption and a short description;
  footer skip, back, next, cancel [R12].
- Capture page: the camera picture on the left (about three quarters),
  with the area where the pattern must be held shown as a tinted rectangle
  and found pattern marks glowing; a collapsed bar to adjust the camera
  picture under it; on the right a framed side panel: a count of pictures
  taken out of five, an instruction, thumbnails of the captured pictures,
  capture and discard buttons, a progress bar, and a collapsed jog pad
  with distance fields [R12]. Five guided positions (four corners and the
  center); more may be added [R12].
- Model page: the corrected preview, then four option cards with radio
  dots (standard, extended, fish-eye, keep the original), a description of
  the highlighted one, a score that is only a hint, and a tips box [R12].
- Done page: a short success line and a large check sign, with finish
  [R12].

#### LaserCraft

Title stays the host window's ("Add camera" or "Camera setup: ‹name›").
Heading of the flow: "Camera lens calibration" (naming.md). M9. Pictures
are called pictures, never frames (naming.md decision).

| Page | Heading (proposed) | Main area | Side panel | Footer |
|---|---|---|---|---|
| 1 | "Correct the lens" | subtitle "About ten minutes"; three step cards: "Print the pattern" (with "Open the pattern file…" link button), "Take pictures", "Pick a model" | none | Back, Skip, Next, Cancel |
| 2 | "Take pictures of the pattern" | live picture with the target area shown as a `marquee` rectangle and found pattern points marked `success`; collapsing "Picture settings" under it (the Camera panel's section) | counter "Pictures taken: ‹n› of 5"; one instruction line; thumbnail grid (each thumbnail removable by selecting it and clicking Discard); buttons "Capture", "Discard"; progress bar (`accent`) with "‹n›%"; collapsing "Jog" with a small jog pad and "Jog distance (mm)", "Z jog distance (mm)" (naming.md) | Back, Next (enabled after five good pictures), Cancel |
| 3 | "Pick a lens model" | corrected live picture, collapsing "Picture settings" | four option cards with choice dots: "Standard", "Wide angle", "Fish-eye", "Keep previous" (proposed labels); a description of the chosen one; "Fit ‹n›%" with the hint "Higher is better, but compare the pictures too" (proposed label); a "Tips" box (`info-soft`) with up to four short lines | Back, Next, Cancel |
| 4 | "Lens corrected" | a large `success` badge, one line naming the stored correction | none | Finish |

- **Step cards** (page 1): three `field-alt` rounded squares side by side
  with a round `text-muted` number badge in a corner and our own simple
  line drawing (`icons.md` concepts: a printed sheet, a camera with
  stacked pictures, a straight and a curved frame); the caption under each
  in `type-body`, a one-line description in `type-small`, `text-muted`
  (Echo [R12]).
- **Pattern** (own design, open question): an openly documented
  calibration pattern that LaserCraft ships as a PDF and SVG; the reference
  uses a code-style pattern of its own, which we do not copy [R12].
- **Capture guidance**: five positions, four corners and the center, as
  the instruction line asks for them one by one ("Hold the pattern in the
  top-left part of the picture"); more pictures may be added (Match
  [R12]).
- **The Jog section moves the machine** with the laser off (safety rule
  3); it is enabled only while connected and Ready, and its pad repeats
  the Jog panel's keys and look in a smaller size (`machine-panels.md`).
- Skip on page 1 leaves the camera uncorrected (Lens badge `warning`).

Tier: **Match** (four pages, step cards, capture side panel with counter,
thumbnails, capture and discard, progress, jog section, model cards,
score, tips), **Echo** (card and badge looks), **Own** (labels, pattern,
colors).

### Camera alignment (wizard)

#### Reference

- Runs inside the add or edit window; opened from the align button or
  the alignment status glyph [R13][R14].
- A method choice (automatic or manual) is described but not shown; an
  overview page like the lens wizard's with three step cards for the
  automatic method (set up the pattern, burn it, check it) or four for the
  manual one (with a step to click the marks) [R13].
- Pattern setup: a left text column with short instructions and a bullet
  list; a big framed panel with a drawing of the bed and the pattern on
  it, which can be dragged into place: four numbered crosshair marks at
  the corners (and, for the automatic method, pairs of printed tags), a
  small center cross; a group of size fields (mark width, a preset
  dropdown, pattern width and height) [R13]. A head-mounted variant only
  lets the mark width change and draws a different pattern [R14].
- Burn page: a left column with three short steps (prepare material,
  enter settings, burn) and a collapsed machine-control bar; on the right
  the live picture, a laser settings group with speed and power, buttons
  to take settings from the material library or open the settings editor,
  and a job controls group of large buttons (preview, start, stop, frame,
  send, pause) [R13].
- Target pages: the large picture with a side panel showing the step
  title, instructions, a bold status line and discard and next buttons,
  then a collapsed machine-control bar. Automatic: found tags are
  outlined, at least four of eight are needed. Manual: zoom with the
  wheel and click the exact centers of marks one to four; each placed
  center shows a small cross and the crosses are joined into a box;
  discard removes the last [R13][R14].
- Check page: a straightened picture with a grid over it; next saves,
  discard retries [R13]. Head-mounted only: a first step to jog until the
  pattern is centered, and a last step to drag the sharp part of the
  picture [R14].
- Alignment must be absolute-placed, is kept per material height, and is
  redone whenever the camera moves [R8][R13].

#### LaserCraft

Heading of the flow: "Camera alignment" (naming.md); runs inside Add
camera or Camera setup (title unchanged). M9. **The burn page starts a
real job and follows `docs/spec/safety.md` through "Jobs started from
these windows" above, item by item.**

| Page | Heading (proposed) | Main area | Side panel | Footer |
|---|---|---|---|---|
| 1 | "How should the marks be found?" | two option cards: "Automatically" ("LaserCraft finds the burned marks in the picture"), "By clicking" ("You click the center of each mark") | none | Back, Next, Cancel |
| 2 | "Line up the camera with the bed" | subtitle with the estimated time; three or four step cards: "Place the pattern", "Burn the pattern", ("Click the marks"), "Check the result" | none | Back, Skip, Next, Cancel |
| 3 | "Place the pattern" | left text column (`type-small`): two short instruction lines and a short list of what makes a good pattern; right: a group "Pattern on the bed": a drawing of the bed (`bed` fill, `grid-minor` lines, `bed-edge` outline) with the pattern as a white rectangle carrying four numbered ring-and-cross marks (`text`), numbered clockwise from top left, and a small center cross; drag to move it | group "Pattern size": "Mark size (mm)", "Size" dropdown ("Default", "Custom"), "Pattern width (mm)", "Pattern height (mm)" (proposed labels) | Back, Next, Cancel |
| 4 | "Burn the pattern" | left column: three short numbered steps ("Put a flat scrap sheet on the bed", "Check the settings on the right", "Click Start") and a collapsing "Jog" (as in the lens wizard); right: live picture on top | group "Settings": "Speed (mm/s)", "Max power (%)", buttons "From materials…" (opens a small picker of the open library's entries) and "Layer settings…"; group "Run": Job preview…, Frame, Start, Pause, Stop (job buttons) | Back, Next (enabled once the job has finished), Cancel |
| 5 | "Find the marks" | the picture, zoomable with the wheel; automatic: found marks outlined in `drawing`, a `success` cross at the picture center; by clicking: each placed center is a `drawing` cross, the four joined by thin `drawing` lines into a box | box (`field-alt`, rounded) with "Step ‹n›: ‹title›", an instruction and a status line in `type-body` ("‹n› of 8 marks found", "Click mark ‹k›", "All marks placed"); buttons "Discard", "Next mark" (proposed); collapsing "Jog" | Back, Next, Cancel |
| 6 | "Check the alignment" | the straightened picture with a `grid-major`-style grid in bed millimetres | one line "If the grid lines up with the burned marks, click Finish." | Back, "Try again", Finish |

Head-mounted cameras (Mounted: on the laser head) add, before page 3, a
page "Center the head over the pattern" with the live picture and the
Jog section, and after page 5 a page "Mark the sharp area" where the user
drags a rectangle (`marquee` look) over the sharp part of the picture
(Match [R14]).

**Burn page and safety** (binding; implements `docs/spec/safety.md`):

- The pattern is a job: Start is enabled only while connected, Ready, with
  valid settings; it runs the same bounds check as every job (rule 5) and
  refuses with the "Cannot run: …" band inside the page.
- Max power (%) is clamped to the device's Max power (%) and the clamp
  warning shows (rule 6). Default settings come from the open library's
  matching entry when one exists, else a low default chosen by the M9 spec
  (own design).
- Frame is laser-off (rule 2). Pause and Stop repeat the job bar (rule 7)
  and are enabled in every connected state, as in `machine-panels.md`;
  with Constant power (M3) on in Layer settings…, Pause reads "Pause
  (motion only)" (rule 9).
- The pattern's position is always absolute: the page forces Absolute
  coordinates for this job only and says so in an `info-soft` line "The
  pattern is placed by bed coordinates." (Match [R13]; it does not change
  the Machine panel's Place job at).
- Back, Next and Cancel are disabled while the job runs or is held; the
  user must Stop or let it finish first (own design: leaving the page must
  never hide a running job's buttons, although the job bar still shows
  them).
- An alarm or stall during the burn shows the Alarm banner in the job bar
  and Machine panel and a `danger-soft` band on the page "The burn
  stopped: ‹reason›. Clear the alarm, then click Start again." (rule 10).
- The reference's send button is omitted (decision 1).

Tier: **Match** (method choice, overview, pattern placement with sizes,
burn page with settings and job buttons, mark finding in both methods,
check page, head-mounted extra steps), **Echo** (side panel box, status
line), **Own** (labels, colors, safety behavior, disabled navigation while
burning).

---

## Job setups (M9)

### Rotary attachment

#### Reference

- Opened from the machine tools menu with Ctrl+Shift+R, and from a button
  of the special-modes toolbar [R15][R22].
- When rotary is active, the job-control panel's start button gains a
  rotary mark, the status bar's modes indicator names it, and an optional
  switch in the job-control panel turns it on and off (shown only when a
  preference asks for it) [R25][R28][R30][R26]. The jog pad has two curved
  arrows for the rotary axis, whose distance is in degrees [R31]. A device
  setting stores the rotary setup with each project [R27]. The reduced
  mode hides the rotary switch [R21].
- The contents of the rotary setup window itself are not described in the
  pages read (Open questions).

#### LaserCraft

Title "Rotary attachment" (naming.md). Opened by Machine → Rotary
attachment… (Ctrl+Shift+R (M9), keymap.md) and the Job setups toolbar's
first button (`toolbars.md`). A small movable dialog. Milestone: M9.
**Everything inside the dialog is own design** (the reference's layout is
unknown); every label below is a proposed label.

| # | Part | Contents |
|---|---|---|
| 1 | Use | switch "Use rotary" (naming.md; the same setting as the Machine panel's switch) |
| 2 | Kind | choice dots in one row: "Rollers", "Chuck" |
| 3 | Axis | dropdown "Rotary axis": "Instead of Y", "A axis" (the second only when the controller reports an A axis) |
| 4 | Sizes | "Travel per turn (mm)" (the Y or A travel the controller makes for one full turn of the rotary's driven part); "Roller diameter (mm)" (Rollers only); "Object diameter (mm)" |
| 5 | Read-out | "Circumference ‹c› mm; the design's height maps to ‹d›° of turn" in `type-small`, `text-muted` |
| 6 | Test | button "Test rotation" (turns the object once forward and back at jog speed, laser off); enabled only while connected and Ready |
| 7 | Footer | OK (default), Cancel |

- OK stores the settings in the project and the device profile (own
  design, after the reference's store-with-project setting [R27]); the
  change is one undoable command for the project part.
- When Use rotary is on: the Job setups toolbar button is checked, the
  status bar's Active setups field reads "Rotary" (`layout.md`), Start
  carries the small cylinder mark (`machine-panels.md`), and the Jog
  panel shows its rotary arrows (M9, `machine-panels.md`).
- Test rotation is a jog move: it never turns the laser on (safety rule
  3); its tooltip says "Turn the object once and back with the laser
  off".

Tier: **Match** (entry points and chord, the on/off switch and where its
state shows), **Own** (the whole dialog's contents and labels).

### Taper correction

#### Reference

- Opened from the machine tools menu, where it is greyed in a gantry
  screenshot [R15][R17]. A small dialog: on the left, right-aligned labels
  with a dropdown for which side the object's top is on (top, left or
  right), and number fields for the top diameter, the length and the
  bottom diameter, then a read-only correction value; on the right a bold
  line drawing of a tapered cup marked with the three measurements; OK
  (default) and Cancel [R17].
- Values are in millimetres; the workspace previews the warp while values
  change; OK applies it to the selected art [R17].

#### LaserCraft

Title "Taper correction" (naming.md). Opened by Machine → Taper
correction… (`menus.md`: enabled while the rotary attachment is on; with
nothing selected OK is disabled and a `text-muted` line under the fields
reads "Select the art to correct", proposed label, own design). A small
dialog. M9.

| # | Part | Contents |
|---|---|---|
| 1 | Fields (left) | "Top of design" dropdown: "Top edge", "Left edge", "Right edge" (proposed values); "Top diameter (mm)"; "Length (mm)"; "Bottom diameter (mm)" (naming.md labels) |
| 2 | Read-out | "Correction ‹n›%" in `type-small`, `text-muted` (proposed label): how much narrower the design is drawn at its narrow end |
| 3 | Diagram (right) | our own simple line drawing of a cup in `text` strokes, its top diameter, length and bottom diameter marked with the letters of the field labels' first words |
| 4 | Footer | OK (default), Cancel |

- "Top of design" tooltip: "Which edge of the design goes on the cup's
  wider end" (own wording of [R17]).
- The canvas previews the warped selection live (`canvas.md`, tool window
  previews; Match [R17]); OK warps it as one undoable command.
- Lengths are always millimetres, as everywhere in LaserCraft today
  (decision 14).

Tier: **Match** (fields and their order, diagram beside them, live preview,
OK/Cancel), **Own** (labels, read-out wording, drawing).

### Print registration (wizard)

#### Reference

- Opened from a submenu of the machine tools menu (a wizard entry with
  Alt+0, and set first target, set second target, align scaled, align
  without scaling with Alt+1 to Alt+4, then a reset entry), with
  Alt+0, or from the special-modes toolbar [R16][R22].
- A small, roughly square wizard with three pages that replace each other:
  page 1 a big crosshair pictogram marked 1 with a two-line caption and a
  set button, a full-width reset button and close; page 2 the same for
  target 2, plus a column with a jog pictogram and a button that moves the
  head to where the second target is expected; page 3 two columns, scaled
  alignment and alignment without scaling, each with a pictogram, a
  caption and a button; choosing either closes the wizard [R16].
- Each target is one shape or one group (a crosshair made of two lines);
  set targets are drawn thick in one color for the first and another for
  the second; once aligned, both get a ring [R16]. The status bar shows the
  registration as an enabled mode on a pale green band [R16][R28]. The
  start-from mode is forced to absolute coordinates, and the preview shows
  the job turned to match the targets [R16].
- Errors are plain message boxes with one line and OK: more than one shape
  selected; the second target is the same graphic, the laser has not
  moved, or the targets are too close [R16].
- Preconditions: a gantry machine, connected, reporting position, homed,
  moved only by software, never by hand [R16].

#### LaserCraft

Title "Print registration" (naming.md). Opened by Machine → Print
registration ▸ Guided setup… (Alt+0 (M9)) and the Job setups toolbar's
second button (`toolbars.md`). The submenu's step items (Set target 1
Alt+1, Set target 2 Alt+2, Fit job to targets Alt+3, Place job on
targets (no scaling) Alt+4, Clear registration; all M9) do the same steps
without the wizard (`menus.md`). A small movable window, not modal. M9.

| Page | Main content | Buttons |
|---|---|---|
| 1. Target 1 | `icon-big` pictogram `set-target-1` (the registration sheet with its upper-left corner mark in `accent`; no digit); caption "Select the first target on the canvas, move the head over its printed mark, then click Set target 1." (proposed) | "Set target 1" (default); "Clear registration"; footer Close |
| 2. Target 2 | left column: pictogram `set-target-2` (the same sheet with its lower-right corner mark in `accent`; no digit), caption "Select the second target, move the head over its mark, then click Set target 2.", "Set target 2" (default); right column: jog pictogram (`icons.md`), caption "Move the head to where target 2 should be.", "Go to expected target 2" (proposed) | "Clear registration"; footer Back, Close |
| 3. Fit | two columns: "Fit job to targets" ("Moves, turns and scales the job to match both targets") and "Place job on targets (no scaling)" ("Moves and turns the job only"), each with its pictogram (`fit-to-targets`, `place-on-targets`), caption and button | "Clear registration"; footer Back, Close |

- Choosing either fit button applies it and closes the window (Match
  [R16]); the result is an undoable command on the project's registration
  setting, not a change to the shapes.
- Captions in `type-body`, centered under their pictogram, at most two
  lines; pictograms are our own and come from `icons.md`: `set-target-1`
  and `set-target-2` carry no digits and tell the targets apart by which
  corner mark carries the accent; `fit-to-targets` is a frame with two
  diagonal scale arrows; `place-on-targets` is a corner-tick mark with a
  short arrow sliding it into place.
- **Preconditions shown in the window**: at the top, an `info-soft` line
  lists what is missing ("Connect the machine first.", "Home the machine
  first.", "Select exactly one shape or group as the target.") and the
  set buttons stay disabled until it is gone (own design; the reference
  only explains this in its text [R16]).
- **Head moves**: Go to expected target 2 is a jog-like move with the
  laser off (safety rule 3); it is enabled only while connected and Ready.
  The user moves the head with the Jog panel, the jog chords or Move head,
  never by hand (the instruction line says so).
- **Canvas feedback**: Registration targets are drawn as `canvas.md`
  specifies (number, not set dashed, set solid); the reference's red,
  blue and green states are not copied (README.md, semantic colors).
- **Status**: while registration is in use, the status bar's Active
  setups field reads "Print registration" (`layout.md`; Echo [R28]), the
  Job setups button is checked (`toolbars.md`), and Place job at shows
  Absolute coordinates, disabled, with the tooltip "Print registration
  places the job by its targets." (`machine-panels.md` cross-reference;
  Match [R16]).
- The job preview draws the job turned and scaled as it will run (Match
  [R16]; `job-preview.md`).

Tier: **Match** (three pages and their actions, the submenu steps and
chords, absolute placement, preview of the fitted job), **Echo** (big
pictogram above a caption and one button), **Own** (labels, preconditions
line, non-modal window, target colors).

### Registration failed (message)

#### Reference

- Plain message boxes with the program's icon in the title bar, one line
  of text and OK; one titled after the feature (more than one shape
  selected; group them), one titled after a failed target (same graphic
  twice, laser not moved, targets too close) [R16].

#### LaserCraft

Title "Registration failed" (naming.md) for every case. M9. Body follows
naming.md's three-part message; footer: "OK" (default). A `danger` badge
at the left of the body (own design).

| Case | Message (proposed wording) |
|---|---|
| Not exactly one target selected | "Cannot set target ‹n›: ‹k› shapes are selected. Select one shape, or group the target's parts first." |
| Same target twice | "Cannot set target 2: it is the same shape as target 1. Select the second printed target." |
| Head did not move | "Cannot set target 2: the head is still where target 1 was set. Move the head over the second mark first." |
| Targets too close | "Cannot set target 2: the targets are only ‹d› mm apart. Use targets at least ‹m› mm apart, far from each other on the sheet." |
| No head position | "Cannot set target ‹n›: the machine did not report its position. Wait until it shows Ready, then try again." |
| Fit off the bed | "Cannot fit the job: the fitted job would go ‹d› mm past the ‹edge› edge of the bed. Check the targets." |

- While connected, the message is a window that does not block the job bar
  (Conventions above).

Tier: **Match** (the cases), **Own** (single title, wording, badge).

---

## Later windows

### Find center (wizard, later)

#### Reference

- Opened from the machine tools menu [R15][R18]. A plain dialog: a
  centered one-line instruction, a small square diagram of a circle with
  the current point highlighted, chords between the points, their
  bisectors and the computed center, one large full-width button to set
  the current point, and a footer with reset on the left and OK and Cancel
  on the right [R18].
- The last step shows the computed position in large text, a full-width
  button to move the head to the center, and a switch that adds a guide
  circle to the project on a tool layer [R18].
- The head must be moved by software, not by hand; points may be set in
  any order; the reference's guide circle is drawn dashed [R18].

#### LaserCraft

Title "Find center" (naming.md). Opened by Machine → Find center…
(`menus.md`, enabled while connected). A small movable window, not modal.
Later.

| Step | Instruction (proposed) | Diagram | Main button (proposed) |
|---|---|---|---|
| 1 | "Move the head to a point on the round part's edge." | our own drawing: a circle with three point positions; the current one `selection`, set ones `text` | "Set point 1" |
| 2 | "Move the head to a second point on the edge, well away from the first." | as above, point 1 set | "Set point 2" |
| 3 | "Move the head to a third point on the edge." | as above, points 1 and 2 set | "Set point 3" |
| 4 | "The center was found." | the circle with its center cross (`text`) and the three points | "Go to center" |

- Steps 1–3: the instruction in `type-body`, centered; the diagram an
  `icon-big`-sized square below it (own drawing, `icons.md`; the
  reference's colored chords and bisectors are not copied [R18]); the main
  button full width, ordinary height, the default. Footer: "Clear points"
  at the left, Cancel at the right.
- Step 4: "Center X (mm)" and "Center Y (mm)" read-outs in `type-body`
  with `type-mono` digits, "Diameter ‹d› mm" under them; "Go to center"
  (moves the head, laser off); switch "Add guide circle" (on by default;
  adds a Guide circle on guide layer 1, `canvas.md`, undoable); footer
  "Clear points" left, OK (default) and Cancel right. OK adds the circle
  if the switch is on; Cancel adds nothing.
- Set point ‹n› reads the live head position from status reports; it is
  enabled only while connected and Ready. All moves are jogs with the
  laser off (safety rule 3).
- Points can be set in any order: clicking a set point in the diagram
  makes it current again (own design; the reference allows any order
  [R18]).
- Problems use the naming.md message shape, shown as an inline
  `danger-soft` band in the window (own design): "Cannot set point ‹n›:
  the machine did not report its position. Wait until it shows Ready,
  then try again." / "Cannot set point ‹n›: it is only ‹d› mm from point
  ‹k›. Move farther along the edge." / "Cannot find a center: the three
  points lie almost on a line, or the center is off the bed. Clear the
  points and try again."

Tier: **Match** (three points, diagram, one main button, last step with
position, go to center and guide circle switch), **Own** (labels,
drawing, inline problems, any-order editing).

### Saved artwork panel (later)

#### Reference

- The reference's art library is a dock window hidden by default; when
  shown it docks on the left in the examples [R19][R29]. Wide panel [R19].
- Upper area: a narrow list of loaded art libraries on the left; on the
  right, a search field, a thumbnail-size slider with its size read-out,
  and a large thumbnail grid with each item's name under it, cut with an
  ellipsis when long [R19].
- Lower area: a group for the library file (new, load, unload) and a group
  for the artwork (add the selected art to the project at the view's
  center, import files, import the current selection, delete) [R19].
- Right-click in the grid: add the selection, rename, delete; dragging a
  thumbnail onto the canvas places it where dropped; dragging onto another
  library copies it, with Shift it moves; the library holds artwork only,
  never settings [R19].

#### LaserCraft

Panel title "Saved artwork" (naming.md). Place: hidden by default; when
first shown it joins the right dock's upper group as its last tab
(`layout.md`; the reference's left docking is not copied, so the left
stays tools only). Later. Saved artwork files are called *collections*
(proposed word: "library" already means the material library, naming.md
one meaning per word).

| # | Part | Contents |
|---|---|---|
| 1 | Search row | text field with hint "Search saved artwork" (proposed), then "Thumbnail size" slider (proposed) |
| 2 | Browser | left, about a quarter of the width: list of open collections; right: the thumbnail grid |
| 3 | Collection row | "New collection…", "Open collection…", "Close collection" (proposed labels) |
| 4 | Artwork row | "Place on canvas" (proposed; disabled with nothing chosen), "Add selection" (naming.md), "Import files…" (proposed), "Delete" (naming.md) |

- Narrow panel (the usual right-dock width): the collection list becomes a
  "Collection" dropdown above the grid (own design, so the panel works in
  a quarter-width dock); wide or floating panels show the list.
- Thumbnail grid: `field`; each item a square thumbnail of the artwork in
  its layer colors on `bed`, its name centered under it in `type-small`,
  ending in "…" when long; chosen item `accent-soft` behind thumbnail and
  name. Thumbnail size steps between about three and about eight `row`s
  (own design; no pixel sizes, README.md).
- Drag a thumbnail onto the canvas: places it where dropped (undoable).
  Drag onto another collection in the list: copies; with Shift held,
  moves (Match [R19]).
- Place on canvas places the chosen artwork at the center of the view
  (Match [R19]). Add selection stores the canvas selection in the current
  collection, asking for a name ("Name" field, OK, Cancel). Import files…
  adds SVG or DXF files, one item per file (`docs/spec/import.md`).
- Right-click in the grid: the Saved artwork menu (`menus.md`: Add
  selection, Rename, Delete).
- Delete asks "Delete saved artwork?" with "Delete" (`danger`) and Cancel.
- Holds shapes, text and images only; never layer settings (Match [R19]);
  items keep their layer numbers, and placing one uses the project's
  layers of those numbers.
- Empty: the naming.md line "Nothing saved yet. Select shapes, then
  right-click and choose Add selection."

Tier: **Match** (hidden by default, list plus searchable thumbnail grid,
size slider, file and artwork actions, drag behavior, right-click menu),
**Own** (place in the upper group, collection word, labels, narrow
layout).

### Text data panel (later, owner decision)

#### Reference

- The reference's variable-text window is docked behind its layer window,
  like its shape-properties window [R20]; it is hidden by default in the
  Window menu screenshot [R24].
- Left: a form with right-aligned labels and number fields for the current
  value, the start, the end and the step. Right: a grid of buttons aligned
  with those rows: previous and next; a wide test button that shows the
  substituted text while held; reset and a button that bakes the codes
  into plain text for good (Shift-click: only the selection); an automatic
  advance switch under them [R20].
- Below a thin rule, a CSV section: the loaded file's name and buttons to
  browse and clear [R20].
- The current value is used next; start and end wrap; the step is used by
  previous, next and auto-advance, which counts after each job is started,
  sent or saved; each text object's own offset is added to the current
  value [R20].

#### LaserCraft

Panel title "Text data" (naming.md). Place: right dock, upper group, tab
6, hidden by default (`layout.md`). Later; it depends on editable text
objects, an owner decision (proposed change to `docs/spec/text.md`).

| # | Part | Contents |
|---|---|---|
| 1 | Counter form (left) | "Current value", "First value", "Last value", "Increment" (whole-number fields) |
| 2 | Counter buttons (right, aligned with the rows) | row 1: "Previous", "Next"; row 2: "Show values" (spans both columns); row 3: "Back to first", "Freeze values"; row 4: switch "Count up after each job" |
| 3 | Rule | thin `border` line |
| 4 | CSV section | heading "CSV file" in `type-heading`; one row: the loaded file's name (or "No file" in `text-muted`), then "Choose file…" and "Remove file" at the right |

- Buttons are ordinary, icon left of label (`icons.md` concepts: left
  chevron, right chevron, eye, back-to-start arrow, and a "freeze"
  concept that icons.md chooses), Echo [R20].
- Field labels carry no colon (naming.md; the reference's have colons
  [R20]).
- Show values is a hold button: while pressed, every text shape whose Text
  source is not Plain shows its substituted text on the canvas (Match
  [R20]).
- Freeze values replaces the codes with their current text in every such
  text shape (with Shift: only the selected ones) as one undoable command,
  after asking "Freeze text values?" / "The selected text shapes keep
  their current text and stop changing." with "Freeze values" and Cancel
  (own design: asking first, because the reference's action is permanent
  [R20]; undo still works).
- Count up after each job: the counter advances by Increment each time a
  job is started or saved as G-code (naming.md); its tooltip says so.
- First value and Last value wrap: after Last value comes First value
  (Match [R20]).
- Each text shape's Counter offset (`property-bar.md`) is added to Current
  value.
- CSV: Choose file… opens the file chooser for a CSV file (RFC 4180);
  Remove file unloads it; the row with the Current value number is used.
- Empty state (no text shape uses data): the naming.md line "Select a text
  shape whose Text source is not Plain." under the CSV section; the panel
  stays usable.

Tier: **Match** (panel place behind the layer list, counter form beside
its buttons, CSV section below a rule, behavior), **Own** (labels, no
colons, confirmation before freezing).

### Simple layout (later)

Simple layout is specified in full in `layout.md` ("Simple layout"): its
switch in Preferences…, the " (simple layout)" title suffix, what it
hides, and that it never hides the job bar, Stop, Pause, the bounds-check
message or any safety text. For the windows in this file it changes only:

- Materials panel: Edit settings…, Update from layer and Merge library…
  are hidden; Apply to layer and Follow this entry stay (own design).
- Speed and power test: the preset icon buttons, Interval and Passes in
  the Setting dropdowns, and Label settings… and Border settings… are
  hidden; Box settings… stays (own design).
- Image adjustments: "For the whole layer" shows only Method and Negative
  (own design, after the reference hiding advanced layer settings [R21]).
- Camera panel: the Advanced section is hidden (own design).
- Everything else here is unchanged. Turning Simple layout off brings
  every hidden control back (`layout.md`).

Tier: **Own** (the set for these windows), **Match** (the idea, [R21]).

---

## Deliberately omitted (decision 1, safety and clean-room)

- Send buttons and controller-file saves in the test windows and the burn
  page (DSP-only transfers) [R2][R3][R4][R13].
- Pulse-frequency and pulse-width test settings (other laser kinds) [R2].
- The focus test's minimum-power field and controller-file save of the DSP
  variant [R4].
- Camera presets for known machines and vendor lens presets [R10].
- The Camera panel's overlay resolution slider [R9]: the overlay's scale
  follows the lens correction and bed alignment instead (Camera panel).
- The camera-buying helper and its window [R9][R33].
- The message about the reference's older camera file format (it concerns
  its own format; naming.md, Omitted) [R10].
- Network-camera bandwidth statistics (naming.md, Omitted).
- Cylinder correction, split marking and galvo lens calibration (galvo,
  decision 1) [R15][R22].
- Feeder setup (DSP-only; keymap.md lists Ctrl+Shift+F for M9: proposed
  change to keymap.md, as naming.md says).
- The reference's library and camera file formats, and its material data
  (`docs/clean-room.md`).
- Printing (black only, keep colors): not planned (keymap.md).

---

## egui implementation notes

- Picture panes (Image adjustments, Trace to vectors, Convert to image,
  camera views) are `egui::Image` widgets backed by textures the app
  hands the UI; the UI never decodes images or computes dithering itself
  (CLAUDE.md, the UI has no business logic). Image decoding needs the M7
  dependency decision (`image` crate) with its `ARCHITECTURE.md` line;
  camera capture needs the M9 dependency decision (a USB Video Class
  crate and an HTTP client for network cameras), also with their lines.
- Pan and zoom in panes reuse the canvas's view transform helpers
  (`canvas.md`); the two panes of Image adjustments share one transform.
- Windows that run jobs are `egui::Window`s, never `egui::Modal`, and are
  drawn below the job bar's top-most `egui::Area` (`layout.md`).
- Wizards are one window with a page enum; the footer is drawn with a
  bottom `Panel` inside the window so it never scrolls away.
- Option cards are `Frame`s with a click sense; the chosen card sets the
  page's state, and Next reads it.
- Collapsing sections are `CollapsingHeader`s with their open state kept
  in UI state, not in the project.
- Every button, slider commit and switch emits a `ui.*` action or a
  registry invocation; dragged sliders commit once when the drag ends, as
  the layer editor does today (ui-shell.md, Layout).

---

## Current state → target

From `current-ui.md` (audit at commit 151eb26); paths relative to the repo.

- None of the windows in this file exists yet: no Machine menu, no Camera,
  Materials, Saved artwork or Text data panel, no image objects on the
  canvas, no test windows (current-ui.md 1.3, 5). Each arrives with its
  milestone after its behavior spec.
- Dialogs today are fixed-size and not collapsible, and only one argument
  dialog can be open at a time
  (`crates/lasercraft-ui/src/dialogs.rs:115-118`) → the windows here are
  resizable where stated, and a test window, a camera wizard and an
  ordinary dialog may be open together.
- The only modal is "Unsaved changes", an `egui::Modal` that blocks all
  input (`crates/lasercraft-app/src/main.rs:185-210`) → while connected no
  window may block the job bar; every window in this file is non-modal.
- The layer editor's image method dropdown shows raw ids such as
  "floyd_steinberg" (`crates/lasercraft-ui/src/layers_panel.rs:272-314`)
  → Image adjustments and the layer section both show the readable names.
- The layer model has Constant power (M3) but the editor does not show it
  (`crates/lasercraft-project/src/layer.rs`, Layer::new) → it must exist
  before M8, because the test windows' Pause wording depends on it
  (safety rule 9).
- The keymap binds none of Ctrl+Shift+B, Alt+C, Ctrl+Shift+R, Alt+0 …
  Alt+4 yet (`crates/lasercraft-ui/keymap/default.keymap`) → each is bound
  by its milestone (keymap.md).
- No image textures, no image loader and no camera crate
  (current-ui.md 3) → dependency decisions at M7 and M9.
- No tab strips in the right dock (`crates/lasercraft-ui/src/shell.rs:262-270`)
  → the Materials, Camera, Text data and Saved artwork panels need the
  dock groups of `layout.md` (stage a) first.

## Proposed changes to other specs

- **Proposed change to naming.md**: add the labels this file marks
  "proposed label": the test windows' fields and parts (Count, Setting,
  From, To, Box height (mm), Box width (mm), Center X (mm), Center Y (mm),
  Box settings…, Label settings…, Border settings…, Draw labels, Draw
  border, Pattern size, Patches, Smallest interval (mm), Largest interval
  (mm), Patch size (mm), Patch fill: Solid, Gradient, Z from (mm), Z to
  (mm), Lines, Dashed labels, Run, Built-in, Yours, (none), Save preset
  dialog); Image adjustments' captions "Original" and "As engraved" and
  Save adjustment preset's "Switch on: All, None, Invert"; the Camera
  panel's Picture age line, Use in overlay, Picture settings rows (Exposure,
  Focus, White balance, Brightness, Contrast, Hue, Backlight compensation,
  Auto, Restore defaults), Advanced rows (Preview rate, Insert picture:
  Corrected, As captured); Add camera's cards and fields (USB camera,
  Network camera, Above the bed, On the laser head, Calibrate now, Load
  from file…, No lens correction, Source, Device, Format, Address,
  Pictures per second, Status, Live view); Camera setup's groups (Camera,
  Source, Lens correction, Bed alignment, Calibrate lens…, Align to bed…,
  Identify device by, Skip first pictures); the two wizards' page
  headings, model names (Standard, Wide angle, Fish-eye, Keep previous),
  Fit, Next mark, Try again, From materials…; Rotary attachment's fields
  (Rollers, Chuck, Rotary axis, Travel per turn (mm), Roller diameter
  (mm), Object diameter (mm), Test rotation); Taper correction's Top of
  design values and Correction read-out; Print registration's Go to
  expected target 2; Find center's Set point 1–3, Go to center, Add guide
  circle, Clear points, Center X (mm), Center Y (mm); Taper correction's
  "Select the art to correct" line; the Materials target line ("Target
  layer", "No layer selected"); Camera setup's collapsing sections
  ("Source details", "Correction details", "Alignment details") and fact
  lines ("Kind: …" with USB camera / Network camera, "Mounted: …" with
  above the bed / on the laser head); the camera list caption "Cameras for
  ‹device profile›"; the alignment wizard's page-3 labels ("Pattern on the
  bed", "Pattern size", "Mark size (mm)", "Size" with Default / Custom,
  "Pattern width (mm)", "Pattern height (mm)"); Rotary attachment's Rotary
  axis values ("Instead of Y", "A axis"); Saved artwork's
  collection words and buttons; the Registration failed messages; the
  Materials empty-tree line.
- **Proposed change to naming.md** (one meaning per word): "collection"
  for a Saved artwork file, so "library" keeps meaning the material
  library.
- **Proposed change to keymap.md**: consider binding Alt+I (Image
  adjustments…) and Alt+T (Trace to vectors…) at M7; the reference shows
  both chords [R5][R6][R35] but keymap.md lists neither, so they are "not
  bound" here (menus.md asks the same). Also drop Ctrl+Shift+F (Feeder
  setup, M9) from keymap.md: feeder setup is DSP-only and out of scope
  (decision 1, naming.md Omitted), so the chord stays not bound.
- **Proposed change to layers-panel.md and docs/spec/layers.md**: a layer
  can follow a material entry; its editor then shows the Follows material
  band and read-only fields (Materials panel above).
- **Proposed change to docs/spec/safety.md**: state that test patterns and
  the camera alignment burn are jobs under rule 1 and go through rules 5,
  6, 7 and 9 unchanged (this file already behaves so).
- **Proposed change to docs/spec/text.md**: the Text data panel, Text
  source and Counter offset (with editable text objects, owner decision;
  naming.md lists the same change).
- **Proposed change to docs/spec/ui-shell.md**: windows may be resizable,
  several may be open at once, and none blocks the job bar while connected
  (with `layout.md` and `dialogs.md`).
- **Proposed change to menus.md**: its Library menu note says
  `machine-panels.md` places the Library ▸ button; the button is placed in
  this file (Materials panel, bottom row).
- **Proposed change to device.md** (M9): a rotary setup (kind, axis,
  travel per turn, diameters) and camera list per device profile.

## Open questions

1. **Test patterns on the canvas**: should the three tests also offer to
   place the pattern on the canvas as shapes (so it can be moved or
   edited), or stay temporary jobs only, as proposed?
2. **Test window footers**: Close only with remembered values (proposed),
   or the reference's OK keeps / Cancel reverts?
3. **Speed and power test title field**: the reference has a title field
   whose purpose its page does not explain [R2]; leave it out (proposed)
   or add a "Title" printed above the grid?
4. **Focus height test Z reference**: offsets from the head's Z at Start
   (proposed), or absolute Z from a material height setting like the
   reference's [R4]?
5. **Rotary attachment contents**: the reference's rotary window was not
   in the pages read; the owner should confirm the proposed fields (kind,
   axis, travel per turn, diameters, test rotation), or describe what
   they need from their own machine.
6. **Built-in presets and materials**: which built-in test and image
   presets ship, and who writes their data (our own measurements, M8)?
7. **Calibration pattern**: which openly documented pattern does the lens
   wizard use (a chessboard, a published marker set), and under which
   license is the printable file shipped?
8. **Camera dependencies**: which crates for USB Video Class capture and
   network streams (M9), and how they behave inside the Flatpak sandbox
   (camera portal)?
9. **Camera panel default**: hidden until a camera is set up (as
   `layout.md` says), then shown once — or shown on every start after
   that?
10. **Saved artwork word**: is "collection" acceptable, or should the
    owner pick another word for a Saved artwork file?
11. **Text data**: does the owner want editable text objects at all
    (text.md), which this panel depends on?
12. **Simple layout set**: are the hidden controls listed for these
    windows the right ones?
13. **Following a material entry** when the library is not open: keep the
    last settings with a warning (proposed) or refuse to run the job?
14. **Image adjustments' Denoise**: keep LaserCraft's extra row, or wait
    until the M7 image spec decides the filter?

## Sources

All LightBurn pages read 2026-10-21, through the research notes for this
file (libraries, images and cameras notes; preview and device notes; main
window layout notes; menu notes) and, where named, the canvas, left
toolbar, property bar, machine window and preferences notes.

1. [R1] https://docs.lightburnsoftware.com/2.1/Reference/MaterialLibrary/ —
   material library panel: place, library row, tree levels, button
   groups, form in place, popup actions, device-linked libraries,
   re-linking, shared folders, unknowns.
2. [R2] https://docs.lightburnsoftware.com/2.1/Reference/MaterialTest/ —
   material test window: presets row and sections, rows and columns
   groups, job buttons, parts buttons with chips, switches, defaults, run
   order, unit warning, result layout.
3. [R3] https://docs.lightburnsoftware.com/2.1/Reference/IntervalTest/ —
   interval test window, fill choice, job buttons, OK and Cancel meaning,
   result.
4. [R4] https://docs.lightburnsoftware.com/2.1/Reference/FocusTest/ —
   focus test window, high-power labels switch, Z-axis message, Z
   reference, result.
5. [R5] https://docs.lightburnsoftware.com/2.1/Reference/AdjustImage/ —
   image adjustment window, entry points and chord, panes, presets,
   two settings groups, preset saving dialog.
6. [R6] https://docs.lightburnsoftware.com/2.1/Reference/TraceImage/ —
   trace window, preview, boundary, controls, defaults, result.
7. [R7] https://docs.lightburnsoftware.com/2.1/Reference/ConvertToBitmap/ —
   convert-to-image window, entry from the edit menu and right-click
   menu, render choices, resolution, result.
8. [R8] https://docs.lightburnsoftware.com/2.1/Reference/Cameras/ — camera
   overview: when alignment and calibration must be redone, per-height
   alignments.
9. [R9] https://docs.lightburnsoftware.com/2.1/Reference/Cameras/CamerasWindow/
   — camera panel: place, overlay buttons, overlay controls in two
   variants, camera blocks and header glyphs, picture settings, advanced,
   add and import, context menus, overlay under the workspace.
10. [R10] https://docs.lightburnsoftware.com/2.1/Reference/Cameras/AddACamera/
    — new-camera wizard frame and pages, option cards, source page,
    presets, lens choice, import, older-file message.
11. [R11] https://docs.lightburnsoftware.com/2.1/Reference/Cameras/EditCamera/
    — edit-camera window: columns, sections, status signs, preview states,
    advanced settings, footer.
12. [R12] https://docs.lightburnsoftware.com/2.1/Reference/Cameras/Calibration/
    — lens calibration wizard pages, capture panel, models, score, done
    page.
13. [R13] https://docs.lightburnsoftware.com/2.1/Reference/Cameras/Alignment/
    — alignment wizard pages, pattern setup, burn page and job buttons,
    target finding, check page, absolute placement.
14. [R14] https://docs.lightburnsoftware.com/2.1/Reference/Cameras/HeadMountedCamera/
    — mounting choice, head-mounted pattern and extra steps, docked
    camera tab, stitched capture.
15. [R15] https://docs.lightburnsoftware.com/2.1/Reference/UI/LaserToolsMenu/
    — machine tools menu rows for the tests, cameras, rotary, center
    finder, taper warp; greyed states; the camera list dialog.
16. [R16] https://docs.lightburnsoftware.com/2.1/Reference/PrintAndCut/ —
    print-and-cut submenu and chords, wizard pages, target feedback,
    status band, absolute placement, preview, error messages,
    preconditions.
17. [R17] https://docs.lightburnsoftware.com/2.1/Reference/TaperWarp/ —
    taper warp dialog fields, diagram, live preview, millimetres (through
    the left toolbar notes).
18. [R18] https://docs.lightburnsoftware.com/2.1/Reference/CenterFinder/ —
    center finder wizard, diagram, steps, last step, guide circle, any
    point order (through the canvas notes).
19. [R19] https://docs.lightburnsoftware.com/2.1/Reference/ArtLibrary/ —
    art library panel, place, list, search, size slider, grid, buttons,
    context menu, drag behavior, artwork only.
20. [R20] https://docs.lightburnsoftware.com/2.1/Reference/VariableText/ —
    variable text panel layout, buttons, CSV section, behavior (through
    the property bar notes).
21. [R21] https://docs.lightburnsoftware.com/2.1/Reference/BeginnerMode/ —
    reduced mode hiding the rotary switch and advanced layer settings.
22. [R22] https://docs.lightburnsoftware.com/2.1/Reference/UI/ModesToolbar/
    — special-modes toolbar buttons for rotary, print-and-cut and galvo
    items (through the left toolbar notes).
23. [R23] https://docs.lightburnsoftware.com/2.1/Reference/UI/ — default
    window map: library tab in the lower group.
24. [R24] https://docs.lightburnsoftware.com/2.1/Reference/UI/WindowMenu/ —
    window list entries for the library, camera, art library and
    variable text windows and their shown or hidden states.
25. [R25] https://docs.lightburnsoftware.com/2.1/NewFeatures/NewIn2.1/ —
    rebuilt camera interface, library buttons moved under the list with
    icons, rotary mark on Start.
26. [R26] https://docs.lightburnsoftware.com/2.1/Reference/SettingsPreferences/
    — preference that shows the rotary switch in the job panel (through
    the preferences notes).
27. [R27] https://docs.lightburnsoftware.com/2.1/Reference/DeviceSettings/BasicSettings/
    — device setting that stores the rotary setup with projects.
28. [R28] https://docs.lightburnsoftware.com/2.1/Reference/UI/StatusBar/ —
    enabled-modes indicator on a pale green band.
29. [R29] https://docs.lightburnsoftware.com/2.1/Guides/CustomizingTheLightBurnWindow/
    — art library docked on the left in examples; library buttons below
    the list.
30. [R30] https://docs.lightburnsoftware.com/2.1/Reference/LaserWindow/ —
    the optional rotary switch in the job panel (through the machine
    window notes).
31. [R31] https://docs.lightburnsoftware.com/2.1/Reference/MoveWindow/ —
    rotary jog arrows and distance in degrees (through the machine window
    notes).
32. [R32] https://docs.lightburnsoftware.com/2.1/Reference/DeviceSettings/DimensionsUnits/
    — the Z axis enable setting the focus test needs.
33. [R33] https://docs.lightburnsoftware.com/2.1/Reference/Cameras/CameraSelection/
    — the camera-buying helper (omitted).
34. [R34] https://docs.lightburnsoftware.com/2.1/Reference/MultiFileTraceImage/
    — batch tracing without a window.
35. [R35] https://docs.lightburnsoftware.com/2.1/Reference/UI/ToolsMenu/ —
    tools menu rows for adjusting and tracing images with their chords
    (through the menu notes).
36. [R36] https://docs.lightburnsoftware.com/2.1/Reference/UI/EditMenu/ —
    edit menu row for converting to an image, whose screenshot and page
    text give different chords (through the menu notes).

Existing LaserCraft specs used: `docs/clean-room.md`, `docs/spec/safety.md`
(rules 1–11), `docs/spec/ui-shell.md`, `docs/spec/keymap.md` and
`crates/lasercraft-ui/keymap/default.keymap`, `docs/spec/layers.md`,
`docs/spec/text.md`, `docs/spec/import.md`, `docs/spec/device.md`, and the
style files `naming.md`, `layout.md`, `menus.md`, `toolbars.md`,
`machine-panels.md`, `layers-panel.md`, `property-bar.md`, `canvas.md`,
`job-preview.md`.

Open standards: WCAG 2.x (use of color, contrast); USB Video Class
(USB-IF) for camera controls; RFC 4180 (CSV); SVG and DXF for saved
artwork imports; GRBL v1.1 documentation (jogging, laser mode `M3`/`M4`)
https://github.com/gnea/grbl/wiki; the published dithering algorithms
named in `CLAUDE.md` M7.

Own design: every label, tooltip and message in this file; every look
(tokens, badges, option cards, chips, panes, colors of previews and
targets); the Close-only test footers and remembered values; the pattern
being a temporary job; the target line and following band in Materials;
built-in presets and data; the Camera panel's picture age, badges and view-
only overlay settings; Camera setup's OK/Cancel; the calibration pattern;
the whole Rotary attachment dialog; the Print registration preconditions
line and non-modal window; Find center's inline problems; the Saved
artwork collection word, narrow layout and place in the upper group; the
Text data confirmation; the Simple layout set for these windows; and every
safety behavior above, which implements `docs/spec/safety.md`.
