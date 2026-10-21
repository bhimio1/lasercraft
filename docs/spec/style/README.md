# Spec: style guide (look and feel)

Milestones: all of them. M0–M5 (restyle of what exists today: theme tokens,
labels, menus, switches, palette strip, canvas and preview colors), M6 (job
bar, machine panels, the lower dock group, the rules every window follows
while connected), M7 (image windows), M8 (materials, tests, device
profiles), M9 (camera, rotary, print registration, job setups), M10 (help,
accessibility pass, app icon, release packages); the toolbar band, the icon
set, docking beyond fixed places, the dark theme and Simple layout are
later, with no milestone yet.

This file is the entry point of LaserCraft's UI style guide in
`docs/spec/style/`. It decides nothing about any single region or widget;
it states why the guide exists, the clean-room boundary as it applies to
look and feel, the three tiers every element is placed in, a one-glance
summary of the look, the index of the other thirteen files, how to use them
when building UI, a suggested build order, and two consolidated lists: every
"proposed change to ‹spec›" the other files raise, and every owner decision
and open question they leave. Where this file and another style file
disagree about a detail, the other file wins for its own topic and this
index is corrected; where any style file and an existing spec in
`docs/spec/` disagree, the existing spec wins until it is updated
(decision 15, below).

## Purpose

LaserCraft should feel familiar to people who already know LightBurn, so
that switching costs them as little as possible: the same regions in the
same places, the same control groups in the same order, the same gestures
and default shortcuts, the same information in each panel. That is the
point of the project (`CLAUDE.md`, `docs/clean-room.md` "Look and feel").

At the same time LaserCraft is an independent, GPL program built only from
LightBurn's public documentation, open standards and our own design. So the
guide copies arrangement and behaviour, never expression: every icon,
label, tooltip, message, color value, font and the palette are ours, and
the reference's most distinctive chrome is deliberately drawn differently.
The guide is detailed enough that a developer who has never seen the
reference can build the UI from these files alone, without guessing and
without looking at the reference program.

The guide describes the target. It changes no code and no other spec by
itself: each file says what differs from today's code ("Current state →
target") and lists the changes it needs in other specs ("Proposed changes
to other specs"), which a later session applies spec first, code second.

## The clean-room boundary, applied to look and feel

### What we took from the public docs, and how

- **Source.** Only pages under https://docs.lightburnsoftware.com/2.1/:
  their text and the screenshots shown on those pages, all read on
  2026-10-21 by a research pass that wrote notes in its own words. Where a
  page showed an animation, only its first frame was seen; anything later
  in it is known from the page text only. Several captures on 2.1 pages
  come from older releases; the notes record that where it matters.
- **What the notes kept.** Which regions exist and where they sit, the
  order of controls in each toolbar and panel, which actions exist and how
  they behave, which states a control has, and general visual traits in
  plain words ("a light theme", "a strongly colored title strip", "small
  pill switches"), with relative sizes only.
- **What the notes did not keep.** No sentence of the docs, no color value,
  no size, no font, no icon drawing beyond a one-word concept ("folder",
  "magnifier"), no screenshot.
- **How the guide uses them.** Each style file states reference facts only
  from those notes, tags each with a citation key ([R1], …) and lists the
  page URLs under its Sources. Anything the notes do not show is either
  marked "own design" and designed by us, or listed as an open question.
  Nothing is filled in from memory.
- **Nothing copied, nothing stored.** No screenshot is stored, embedded,
  linked as an asset, traced or described in enough detail to recreate it.
  Each file is run through the guide's clean-room lint before it is
  accepted (shared runs of seven words with the fetched page texts, color
  codes, pixel and point sizes), and prose overlap it reports is rephrased.
- **Doc terms.** What the reference calls a window, toolbar or button
  appears only in `naming.md`'s mapping column and in Sources lists, to say
  where a fact came from. Everywhere else the guide uses LaserCraft's names
  and talks about "the reference's layer list", never using a doc term as
  if it were our label.
- **Scope.** GRBL gantry lasers only. Controller-specific, galvo-only,
  vendor, web-tool, network-bridge and licensing items of the reference are
  only listed as omitted in each file's "Deliberately omitted" section
  (decision 1).

### What is always ours

Icons and every other piece of artwork, the app icon and any logo, fonts
(we use the fonts egui bundles), every color value (the guide writes none;
they live only in the theme module), the layer palette's values, every
visible label, tooltip, message and empty-state line, and the window title
form. These are "Own" in every file, with no exceptions.

### Trade dress and the deliberate differences

`docs/clean-room.md` warns against close imitation of a distinctive
interface: "when in doubt, differentiate". The reference has a few
signature traits; the guide keeps the idea where it helps users and draws
it our own way, or drops it:

| Reference trait (general words) | LaserCraft instead | Owning file |
|---|---|---|
| Docked panel title strips drawn as a strongly colored dark-to-warm gradient with light text [R2][R3][R4] | A flat strip one step darker than the panel body (`panel-title`) with dark text (`text`); the focused panel shows a thin `accent` line along its lower edge; no gradients anywhere | `layout.md`, `visual-language.md` |
| Small pill switches, green when on and dark red when off [R2][R8] | Pill switch filled with `accent` and knob right when on, neutral grey with knob left when off, faded when disabled; red is kept for danger | `visual-language.md` |
| Blue-family selection and focus in some captures [R11][R12] | One accent, a medium blue (`accent`), used only for focus, active, selected and "on" | `visual-language.md` |
| The reference's own palette order and values [R5] | LaserCraft's computed palette (black first, then an even hue wheel alternating bright and dark); same strip arrangement | `layers-panel.md` |
| Heavy, solid black toolbar pictograms [R2][R9] | Medium-weight outline pictograms with rounded joins, dark neutral strokes, at most one `accent` detail | `icons.md` |

Other traits the guide does not adopt (red and green canvas boxes, a green
progress fill, a red origin marker, platform-native dialogs) are tabled
with their reasons in `visual-language.md` ("What LaserCraft adopts and
what it does not").

## The three tiers

Every element in every file carries one tier. It says how close we stay to
the reference.

| Tier | Meaning | We copy | We never copy | Examples |
|---|---|---|---|---|
| **Match** | Functional familiarity | the arrangement and behaviour | the expression (look, words, artwork) | which regions exist and their default places; the dense two-row toolbar band; the tool column on the left; tabbed dock groups on the right with tab strips along their bottom edge; the bottom palette strip of 30 numbered swatches; the order of controls in each toolbar and panel; what each panel shows; gestures; default shortcuts (`docs/spec/keymap.md`); the light neutral overall theme |
| **Echo** | Same idea, our own look | the idea only | anything visual beyond the idea | panel title strips; toggle switches; big job buttons about twice the normal height; the swatch strip's look; tinted status bands for hints and messages; stacked (flyout) tool buttons with a corner marker; icons beside menu items |
| **Own** | Wholly LaserCraft's | nothing | — | icons, logo and app icon; fonts; every color value; the palette values; every label, tooltip and message; the job bar; the command line; the window title form; the dark theme; Simple layout's exact set |

A file may give one element two tiers, one for its place and one for its
look (for example "Match (place), Own (look)"). Where a file writes
"Echo → Own", the reference's version was judged too distinctive and the
element became our own design (title strips, switches).

## The look at a glance

### Reference

- A light theme in every capture: light grey chrome, white fields and
  lists, dark text, a near-white drawing area; compact and dense [R1][R2].
- Under the menu bar, two full-width toolbar rows, the lower one about
  twice as tall because its fields sit in two lines [R1][R2].
- A narrow vertical tool strip on the left, the drawing area in the middle
  taking most of the width, and a column on the right about a quarter of
  the window wide holding two tabbed groups, the upper one a little more
  than half the height [R1][R2]. Nothing else is docked on the left by
  default [R3].
- Tab strips run along the bottom edge of the docked groups [R2][R3].
- One row of numbered color swatches across the full width, then a thin
  status row at the very bottom [R1][R2][R5][R6].
- The job-control panel sits in the lower right group and carries tall job
  buttons [R1][R7]. A simplified mode hides and simplifies parts of the window
  [R10].

### LaserCraft

The default arrangement, with LaserCraft names (simplified from
`layout.md`'s diagram, which is binding):

```
+------------------------------------------------------------------------+
| • Coaster sign — Shop diode — LaserCraft                 (OS title bar) |
| File  Edit  Tools  Arrange  Machine  View  Window  Help      (menu bar) |
| [Standard ..............][Arrange .........]      [Job bar (M6)]  row 1 |
| [Position and size][Text style][Job setups (M9)]   [Slide and nest] r 2 |
+------+--------------------------------------------+--------------------+
| Draw-|                                            | Layers             |
| ing  |                                            |  layer list        |
| tools|               Canvas                       |  layer editor      |
|------|        (bed sheet, grid, shapes)           |Layers|Jog|Console..|
| Shape|                                            | Machine            |
| opera|                                            |  state, job buttons|
| tions|                                            |Machine|Materials   |
+------+--------------------------------------------+--------------------+
| 00 01 02 ... 29                                             (Palette)  |
| > command line ................................................ [log]  |
| handle switches | pointer | selection / hint | messages | work area .. |
+------------------------------------------------------------------------+
```

In words:

- **Theme**: light and neutral (`window`, `panel`, `field`, `text`), dense,
  flat, calm; one blue accent; color on screen belongs first to the user's
  layers and second to state (`visual-language.md`). Tier: Match (overall
  impression), Own (every value).
- **Toolbar band**: two full-width rows. Row 1 is one `row` tall (Standard,
  Arrange; the Job bar at its right end while connected). Row 2 is two
  `row`s tall (Position and size, Text style, Job setups, Slide and nest).
  Tier: Match (place and order), Own (look and icons).
- **Tool column**: left edge, Drawing tools above Shape operations; Node
  tools appears beside it while Edit nodes is active. Tier: Match.
- **Canvas**: the bed as a near-white sheet (`bed`) in every theme on a
  slightly darker surround (`canvas`). Tier: Match (behaviour), Own (look).
- **Right dock**: about a quarter of the width; an upper group (Layers in
  front, then Jog, Console and later tabs) and a lower group (Machine in
  front, Materials), each with its tab strip along its bottom edge. Until
  M6 the Layers panel fills the whole dock. Tier: Match (arrangement), Echo
  (title strips), Own (look).
- **Palette strip**: one `row` tall, full width, 30 square numbered
  swatches in LaserCraft's palette, the current color outlined. Tier: Match
  (arrangement), Own (values and look).
- **Command line**: LaserCraft's typed-command line, collapsed to its input
  line by default, its log opening upward. Tier: Own.
- **Status bar**: thin, full width, `type-small`: handle switches (later),
  pointer position, selection size or tool hint, messages in tinted bands,
  active setups (M9), work area, project. Tier: Match (place), Echo (bands),
  Own (fields and order).
- **Job bar** (M6): while connected, a compact strip at the right end of
  toolbar row 1 with the machine state word, Pause/Resume and Stop. It stays
  visible when side panels are hidden, when the Machine panel is hidden or
  behind another tab, and above every dialog; no modal window covers or
  disables it. Stop is the strongest control on screen (`danger`). Tier:
  Own (safety, decision 7).

## File index

| File | What it decides |
|---|---|
| `README.md` | This index: purpose, clean-room boundary, tiers, summary, build order, consolidated proposed changes and open questions. |
| `naming.md` | Every visible name and the rules for all UI text (tooltips, messages, confirmations, empty states, numbers), with the doc-term mapping column. |
| `layout.md` | Where every region of the main window sits, relative sizes, defaults, showing, hiding, docking stages, resizing, reset, window title, narrow windows. |
| `menus.md` | The menu bar order and every item of every menu and context menu: separators, check items, chords, icon concept, enabled rule, milestone, command. |
| `toolbars.md` | Every toolbar's buttons in order and groups, flyouts, enabled and active rules, tooltips, button states, the tool column contents. |
| `property-bar.md` | Position and size and Text style toolbars, the Properties panel, the status bar's handle switches: fields, units, typing, stepping, commit rules. |
| `layers-panel.md` | The Layers panel, the layer editor, the Layer settings window and the Palette strip: contents, order, states, wording. |
| `machine-panels.md` | Machine panel, job bar buttons, Jog, Console, Macros, device dialogs and wizards, machine states and the safety rules in the UI. |
| `canvas.md` | The drawing area: surround, bed, grid, rulers, markers, shapes, selection, handles, selection boxes, hover and snap feedback, tool previews, node editing, cursors. |
| `dialogs.md` | Every dialog, message and secondary window: kinds, frames, form grid, button row, Enter and Esc, previews, errors, file prompts, and each dialog field by field. |
| `job-preview.md` | The job preview window: placement, parts, how each move kind is drawn, playback, legend, summary and bounds-check wording, keys. |
| `later-windows.md` | Windows arriving after M6: image adjustments (M7), materials and tests (M8), camera, rotary, print registration, job setups (M9), and later panels. |
| `visual-language.md` | The master for the look: every color, type and size token, semantic hue rules, the widget catalogue with all states, focus, motion, accessibility, dark theme, egui mapping. |
| `icons.md` | LaserCraft's pictograms: drawing rules, every icon id and its generic concept, cursors and badges, app icon direction, pipeline, the reviewer checklist against copies. |

## How to use the guide when building UI

1. **Spec first.** Before writing UI code, read the style file for the
   region and the existing spec it touches (`docs/spec/ui-shell.md`,
   `canvas.md`, `layers.md`, `preview.md`, `keymap.md`, `safety.md` …). If
   the style file proposes a change to that spec, apply the change to the
   spec in the same session, before the code (decision 15). Existing specs
   win until they are updated.
2. **Tokens only.** Use colors, type sizes and spacing only through the
   tokens of `visual-language.md` (`accent`, `panel-title`, `type-small`,
   `row`, `gap` …). Values exist in exactly one place in code, the theme
   module described under "egui mapping" there; never write a color or a
   size in a widget. A new token is added to `visual-language.md` first.
3. **Names from `naming.md`.** Every label, tooltip and message is quoted
   from `naming.md`, in sentence case. A label a file marks "proposed" is a
   draft until `naming.md` lists it. Never take wording from the reference.
4. **Icons from `icons.md`.** Use the icon id and concept listed there; the
   same icon in the toolbar and in its menu row. Until the icon set exists,
   buttons show text labels. Review every new drawing against the
   checklist in `icons.md`.
5. **Shortcuts from the keymap.** Show only chords that
   `crates/lasercraft-ui/keymap/default.keymap` binds; a chord
   `docs/spec/keymap.md` lists for a later milestone is described as such,
   never shown as working.
6. **Safety is not styling.** `docs/spec/safety.md` is binding: the job bar
   with Pause and Stop is visible and enabled whenever a device is
   connected, over every panel state and every dialog; Stop is the
   strongest control; framing is laser-off by default and any power during
   framing is shown; with a constant-power (`M3`) job Pause is labelled as
   motion-only and Stop is emphasised; no modifier key turns the laser on.
   Red (`danger`) and amber (`warning`) are never decoration.
7. **Commands only.** Every button, menu row and shortcut invokes a command
   from `lasercraft-commands`; the UI crate holds no business logic
   (`CLAUDE.md`). Enabled and disabled states follow the command's rules.
8. **One look everywhere.** egui draws everything on Linux (X11 and
   Wayland), Windows and macOS; menus stay inside the window; check
   fractional scaling on Linux (`layout.md` "Platform notes").
9. **Check the tier.** For an Echo or Own element, follow the LaserCraft
   description and do not "improve" it towards the reference.
10. **Record differences.** When code cannot follow a file yet, add the gap
    to that file's "Current state → target" instead of changing the target.

## Suggested implementation order

All of this is a proposal for the owner; milestones in `CLAUDE.md` stay
authoritative.

### Now (before or alongside M6, restyle of what exists)

- Theme module with the light theme and the tokens; stop following the
  operating system's theme (`visual-language.md`, proposed change to
  `ui-shell.md`).
- Fix the color clashes on the canvas and in the preview: one hue per
  meaning, `selection`, `marquee`, `drawing`, `move-unlit`, `move-travel`,
  `bed-edge` (`canvas.md`, `job-preview.md`).
- Labels: sentence case, renames (Drawing tools, Command line, "Undo
  ‹action›" with readable names) (`naming.md`, `menus.md`).
- Menu order and separators of File, Edit, Tools, Arrange and View
  (`menus.md`).
- Pill switches instead of check boxes; layer list header and row look;
  editor in two label/field columns; numbered square swatches on one line
  (`layers-panel.md`, `visual-language.md`).
- Command line collapsed by default; status bar field order (`layout.md`).
- Dialog frame, OK and Cancel, Enter and Esc, inline errors (`dialogs.md`).
- Job preview window restyle (`job-preview.md`).

### M6 (device connection and streaming)

- Toolbar row 1 exists at least for the job bar; the job bar itself with
  the state word, Pause/Resume and Stop, never covered (`layout.md`,
  `machine-panels.md`).
- Right dock stage a: fixed upper and lower groups with hand-drawn bottom
  tab strips and flat title strips; Machine, Jog and Console panels; show
  and hide; Reset layout; Window menu (`layout.md`, `menus.md`).
- Machine menu, machine states and dots, alarm and job-refused banners,
  Serial port access help, the M6 subset of the device profile dialog
  (`machine-panels.md`, `dialogs.md`).
- Head marker, job origin marker and frame path on the canvas
  (`canvas.md`).
- Window title with the device profile (`layout.md`).

### Later, no milestone yet

- The rest of the toolbar band (Standard, Arrange, Position and size, Text
  style), the tool column as icons, flyouts, Shape operations, Node tools
  (`toolbars.md`, `property-bar.md`); which milestone builds it is an open
  question.
- The icon set and its pipeline (`icons.md`), decided by the milestone that
  builds the toolbar band.
- Docking stage b (drag to redock) and layout persistence, both owner
  decisions with new dependencies (`layout.md`).
- Rulers, guide lines and the later canvas tools (`canvas.md`).
- Dark theme and Theme preference; Display size preferences
  (`visual-language.md`).
- Simple layout (`layout.md`, `toolbars.md`).

### M7 to M10

- M7: Image adjustments and the image parts of the layer editor and menus
  (`later-windows.md`, `layers-panel.md`, `menus.md`).
- M8: Materials panel, test patterns, device profiles dialogs and wizards
  (`later-windows.md`, `machine-panels.md`).
- M9: camera, rotary, print registration, Job setups toolbar
  (`later-windows.md`, `toolbars.md`).
- M10: Help menu and context help, accessibility pass, app icon, packaging
  checks on every platform (`menus.md`, `visual-language.md`, `icons.md`,
  `layout.md`).

## Current state → target

Each file has its own detailed section; the main gaps, from the audit of
the code at the M5 merge:

- **Theme**: nothing sets egui's visuals, style or fonts; the app follows
  the operating system's light or dark preference
  (`crates/lasercraft-ui`, `crates/lasercraft-app`). Target: LaserCraft's
  light theme from one theme module (`visual-language.md`).
- **Layout**: menu bar, a text tool column, the canvas, a right Layers
  panel, palette strip, an open console with its log, status bar
  (`crates/lasercraft-ui/src/shell.rs:240-295`). No toolbar band, no tab strips, no Window or Help
  menu. Target: `layout.md`'s arrangement.
- **Tools**: six text buttons (`shell.rs:537-554`). Target: Drawing tools
  with icons and tooltips (`toolbars.md`, `icons.md`).
- **Labels**: Title Case items and a raw command id in "Undo ‹command id›"
  (`shell.rs:447-472`). Target: `naming.md`.
- **Layers panel**: unlabeled check boxes for Output and Show, no header
  row (`crates/lasercraft-ui/src/layers_panel.rs:118-148`). Target: `layers-panel.md`.
- **Palette strip**: a wrapping row of unnumbered swatches
  (`layers_panel.rs:46-72`). Target: one line of numbered square swatches.
- **Canvas and preview colors**: orange means "selected" on the canvas but
  "laser off" in the preview; green means drawing, the touching box and
  the preview's bed edge (`crates/lasercraft-ui/src/canvas.rs:32-33`,
  `crates/lasercraft-ui/src/preview.rs:12-13`).
  Target: one meaning per hue (`canvas.md`, `job-preview.md`).
- **Status bar**: project, work area, pointer, selection
  (`shell.rs:624-652`). Target: `layout.md`'s order.
- **Dialogs**: OK only, typed file paths, a blocking Unsaved changes modal
  (`crates/lasercraft-ui/src/dialogs.rs`, `shell.rs:581-621`,
  `crates/lasercraft-app/src/main.rs:185-210`).
  Target: `dialogs.md`.
- **Icons**: none, and no window icon
  (`crates/lasercraft-app/src/main.rs:246-251`). Target:
  `icons.md`.
- **Machine UI**: does not exist yet (M6). Target: `machine-panels.md`.

## Proposed changes to other specs

Consolidated from every file, grouped by the spec that would change; the
proposing file is named in brackets. Each is a proposal under decision 15:
the named spec is updated first, in the session that builds the change.
Where two files propose the same change it appears once with both names.
Paths: `docs/spec/canvas.md` is the behaviour spec; `style/canvas.md` is the
style file.

### proposed change to docs/spec/ui-shell.md

- Replace the Layout section's region list with the default arrangement
  (menu bar, toolbar band, tool column, canvas, right dock, palette strip,
  command line, status bar); the job bar (M6); the Window menu with
  toolbar and panel check lists and Reset layout; the window title gains
  the device profile [layout, toolbars, property-bar].
- Renames and sentence case: "tool palette" → Drawing tools, the bottom
  "Console" → Command line, collapsed by default with its log opening
  upward; Title Case labels to sentence case; drawing-order items as verb
  phrases; "Job preview…" [layout, naming, job-preview].
- F12 hides the tool column (with Node tools) and the whole right dock,
  never the job bar, toolbar band, palette strip, command line or status
  bar [layout].
- New status bar field order and the handle switches [layout,
  property-bar].
- Menu list replaced by a pointer to `menus.md`; menu order and
  separators; Side panels as a check item; moves of Convert to path, Close
  path, Join paths and Break apart; Machine, Window and Help menus;
  context menus [menus].
- Toolbar band row 2 (Position and size, Text style); commit rule: Enter,
  Tab and focus loss commit, Esc reverts, an unchanged value runs nothing
  [property-bar].
- Toolbars and the tool column's contents by reference to `toolbars.md`;
  Paste from a button or menu pastes in place, from the keyboard at the
  pointer [toolbars].
- Layers panel header row, whole-row selection, icon button column,
  two-pair editor with units in labels and a Constant power (M3) switch;
  palette strip with two-digit numbers on one line with a More button
  [layers-panel].
- Dialog kinds, OK and Cancel in every argument window, Enter, Esc,
  Ctrl+Enter, errors keep the window open; tool windows draw a live
  `drawing` preview; file prompts gain Cancel, a prefilled field and
  "Replace file?"; Project notes gains Cancel; Unsaved changes gains Save
  (owner decision) [dialogs, canvas, naming].
- Windows may be resizable, several may be open, none blocks the job bar
  [later-windows].
- A "Machine (M6)" section pointing to `machine-panels.md`
  [machine-panels].
- Install LaserCraft's light theme instead of following the OS; a later
  Theme preference; a rule pointing to `visual-language.md`
  [visual-language].
- Drawing tools show icons with tooltips once the set exists; the main
  window sets an app icon [icons].

### proposed change to docs/spec/keymap.md

- Pause and Ctrl+Break (M6) act even while a text field has focus or a
  menu is open; they cannot be rebound (owner decision) [layout,
  machine-panels, dialogs, menus].
- New front-end actions listed so the keymap file can bind them (none bound
  by default): `ui.reset_layout`, `ui.show_panel`, `ui.show_toolbar`,
  `ui.toggle_command_log` [layout]; `ui.units`, `ui.handles` [property-bar];
  the `ui.tool …` actions, `view.zoom_selection`, `ui.preferences`,
  `ui.edit_device_profile`, `ui.camera_capture`, `ui.rotary_setup`,
  `ui.print_registration`, `ui.combine_shapes`, `ui.align_shapes`
  [toolbars].
- Action descriptions reworded in our vocabulary (chords unchanged);
  keymap.md:121 "the console" → the command line; Alt+Shift+S marked not
  bound and Ctrl+Shift+F not planned (out of scope); a chord display rule
  with symbols and Cmd/Option on macOS [naming, menus, machine-panels,
  later-windows].
- Consider Alt+I and Alt+T at M7 [menus, later-windows]; record the
  choice for Ctrl+Shift+A and for a Device profiles chord [menus,
  toolbars]; numeric-keypad jogging listed as not bound [machine-panels].
- Mouse and modifier rows: Alt while dragging shows alignment lines and
  widens snapping; Esc cancels a drag; Shift and Ctrl/Cmd also apply to
  selection boxes; S as a tool-local Line key (later); Ctrl while placing a
  Line point stops the automatic join; the Alt+Shift+W row worded "Show
  fills" [canvas].
- Ctrl+Enter presses the primary button in multi-line boxes; nudge
  bindings refer to named distances; a default user keymap location
  [dialogs].
- Focus-scoped preview keys (zoom chords, Esc closes, slider keys step
  moves while the slider has focus) [job-preview].
- Tab and Shift+Tab move keyboard focus outside the canvas and step objects
  only while the canvas has focus [visual-language].

### proposed change to docs/spec/safety.md

- Rule 7 names the job bar as the control that keeps Stop and Pause
  visible, enabled in every connected state including Connecting… and No
  response, with a UI test that Stop is clickable with every dialog open
  [layout, machine-panels]; and the no-backdrop-over-the-job-bar rule
  [dialogs].
- Rule 1: lines typed in the Console, sent by macros or set as start or end
  G-code that could turn the laser on are refused (owner decision)
  [machine-panels].
- Rule 2 names the Low-power frame window as the only laser-on framing path
  (only if the owner approves it) [dialogs].
- A rule for partial restarts (Run from here) [job-preview].
- Test patterns and the camera alignment burn are jobs under rule 1 and go
  through rules 5, 6, 7 and 9 [later-windows].

### proposed change to docs/spec/canvas.md

- The style file's view, grid levels, rulers, origin marker, selection
  look, Move and Skew handles, reference-point cross, locked look, hover,
  snap markers and kinds, guide lines, drawing previews, the later tools'
  canvas looks, Line tool finishing and joining, per-tool cursors, Show
  fills and Smooth curves, head marker, job origin marker, frame path,
  camera overlay, dark-theme halos, "Edit nodes" [style/canvas, naming,
  visual-language].
- Fixed colors replaced by canvas tokens [visual-language].
- Pointer position in bed coordinates; field values follow drags live
  [property-bar].
- Right-click opens the canvas menu, selecting an unselected object first;
  no menu while drawing [menus].
- Pan, Move head and the later tools; Text tool staying current; the Esc
  table; a "Return to Select" preference [toolbars].
- A rule for the view when the canvas region resizes [layout].
- Grid spacing preference [dialogs]; invalid barcode preview [dialogs].

### proposed change to docs/spec/preview.md

- Unlit moves `move-unlit`, travel `move-travel`, bed outline `bed-edge`;
  the bounds error as a `danger-soft` band with its point marked
  [job-preview, canvas, visual-language].
- h:mm:ss times, "unlit" wording, control labels Time (s), Show whole job,
  Show travel, Close; pan and zoom keys; later playback features; M6 rules
  (same job as Start, constant-power note, Run from here) [job-preview,
  naming].

### proposed change to docs/spec/layers.md

- Swatch gestures (Shift+click, double-click, Swatch menu); `min_power`
  only for Grayscale; image rows' settings; guide layers; Concentric fill;
  Operations and Mixed; extra tab and later fields; layer and all-layers
  context menus; a command behind Put line layers last [layers-panel,
  menus, naming].
- Guide-layer shapes never count in job bounds [canvas]; low-contrast
  colors drawn with a darker edge (owner decision) [canvas]; a General
  preference for saved layer defaults [dialogs]; a layer may follow a
  material entry [later-windows]; a per-shape power share [property-bar].

### proposed change to other behaviour specs

- **planning.md**: guide shapes outside job bounds [canvas]; per-subpath
  start point and direction [canvas, toolbars]; Cut order settings
  [dialogs, naming]; Place job at and the frame path planned like a job;
  `travel_speed`, `fill_acceleration`, `estimate_scale` [machine-panels];
  per-shape power share and run order [property-bar].
- **nodes.md**: node look; Node tools column and armed actions with the
  new node commands; "Edit nodes" [canvas, toolbars, naming].
- **shapes.md**: `shape.skew`; corner radius; polygon, star and
  alternating star kinds; barcodes; `shape.move_to`, `shape.resize`,
  `shape.rotate_at`; locked shapes [canvas, toolbars, naming, dialogs,
  property-bar, menus].
- **text.md**: editable text objects and the Text style toolbar's second
  stage, Text data, Arc handle, Put text on path (owner decision); Font
  browser; Add text's OK rule [naming, property-bar, canvas, dialogs,
  later-windows, menus].
- **arrange.md**: linked copies; sliding and nesting; Match width and
  height; mirror across a line; move to head; Lock and Unlock; path array,
  select circles, fit slots, close nearly closed paths, simplify [canvas,
  menus, naming, toolbars, dialogs].
- **geometry.md**: tolerances for close and simplify, arc segments
  [dialogs].
- **commands.md**: readable action names for Undo; a "can run" query for
  greying menu items; a dry run for previews [menus, dialogs].
- **device.md**: new `Request` and `Event` variants and app-level states
  [machine-panels]; state words and alarm message form [naming]; head
  moving commands with homing conditions [toolbars]; `job.start_from`
  [job-preview]; rotary setup and camera list per profile (M9)
  [later-windows].
- **gcode.md**: Place job at in Save G-code…; "After the job"; start and
  end lines checked by the verifier [machine-panels, naming].
- **project-file.md**: Cut order settings, Project notes switch, auto-save
  [dialogs]; Place job at and Job reference point [machine-panels]; shape
  fields `locked`, `run_order`, `power_share` [property-bar].
- **import.md**: "the console" → the command line; DXF units, close
  tolerance and SVG resolution as preferences [naming, dialogs].
- **cli.md**: `preview --svg` colors [job-preview]; `--reset-layout`
  [layout]; a flag ignoring saved display preferences [visual-language].

### proposed change to project documents

- **ARCHITECTURE.md** (owner decisions): a docking crate and layout
  persistence [layout]; native file dialogs with the XDG portal, a settings
  file, font outlines for the Font browser [dialogs]; the icon route's
  dependency line [icons]; a bundled bold face and the eframe `accesskit`
  feature [visual-language].
- **hardware-testing.md**: constant-power Pause wording and Stop, Stop with
  each dialog open, Console refusal, Clear alarm, unplugging while idle
  [machine-panels].
- **linux-serial.md**: the in-app box with Copy buttons and the
  ModemManager hint [machine-panels].

### proposed change between style files

- **naming.md**: add every label marked "proposed" in `dialogs.md`,
  `machine-panels.md`, `later-windows.md`, `layers-panel.md`,
  `property-bar.md`, `toolbars.md` and `visual-language.md`; "collection"
  for a Saved artwork file; "varies" for differing values; the
  constant-power note as `text` on `warning-soft` with a `warning` mark
  [those files]. `naming.md` in turn proposes small rewordings to
  `dialogs.md`, `later-windows.md`, `style/canvas.md` and
  `visual-language.md`, and adopts the other files' labels [naming].
- **icons.md**: corner and smooth `badge-line` variants [canvas];
  `node-pointer` and `align-shapes` [toolbars].
- **icons.md → others**: icon concepts and ids for `toolbars.md`,
  `menus.md`, `machine-panels.md`, `style/canvas.md`, `dialogs.md`,
  `later-windows.md`, `job-preview.md` [icons].
- **visual-language.md → others**: `text-on-strong`, `text-on-warning` and
  `backdrop` in `dialogs.md`, `machine-panels.md` and `layout.md`; job end
  lines and No response look in `machine-panels.md`; inverted preview
  tokens in `job-preview.md`; table header and guide swatch tokens in
  `layers-panel.md`; canvas accent family and guide layer tokens in
  `style/canvas.md`; group frames in `later-windows.md` [visual-language].
- **style/canvas.md → visual-language.md**: confirm the canvas uses of
  `selection` and the `danger` head-marker cases [canvas].
- **layout.md → machine-panels.md**: the constant-power note stays next to
  Pause at every width [layout].
- **later-windows.md → menus.md**: the Library ▸ button's placement note
  [later-windows]; **toolbars.md → menus.md**: Zoom to selection enabled
  when the project has shapes [toolbars].
- **README.md now exists**: the lines in `dialogs.md`, `visual-language.md`
  and `layout.md` saying it is still to be written can point here; the
  tiers are explained above (proposed by this file).

## Open questions

Deduplicated; each points to the file(s) that raise it (Q = that file's
open question number). Questions the files already closed are listed at
the end.

### Safety and machine

1. Low-power framing: offer it at all, its ceiling, diode only?
   (dialogs Q16, naming Q13, menus Q5, machine-panels Q15)
2. Test fire: anything beyond a placeholder? (naming Q12, machine-panels
   Q15)
3. Refuse, warn or trust Console, macro and start/end lines that could turn
   the laser on? (machine-panels Q3)
4. Make the Pause and Stop chords fixed? (dialogs Q14)
5. Quitting while a job runs; Disconnect while a job runs (dialogs Q2,
   menus Q6)
6. Where Clear alarm sits (menus Q4, naming Q10, machine-panels Q4)
7. Live head marker during a job, or frozen as in the reference
   (style/canvas Q6, machine-panels Q6); its color during a constant-power
   pause (style/canvas Q7)
8. Machines without homing switches (toolbars Q15, machine-panels Q17);
   Ctrl+click on Move on bed and an armed Move head (toolbars Q3, Q6)
9. Run from here: wanted, restart point, passes, re-homing (job-preview Q7)
10. Work area in the project or the device profile; pointer frame
    (machine-panels Q1, Q2; property-bar Q1)
11. Job bar's sent share; Machine to the front on connect; modal dialogs
    while connected (layout Q3–Q5, machine-panels Q9, Q10)
12. Automatic port scan, repeat framing end, keypad jogging, axis
    orientation, omitted device settings, storage of head positions and
    macros, work-offset mode, Z control (machine-panels Q5, Q7, Q8,
    Q11–Q13, Q18, Q19)
13. Forcing the send-and-wait streaming mode; auto focus (naming Q9, Q11)
14. Whether the reference shows Pause and Stop outside its job-control
    window (layout Q13; unknown from the docs, not needed by the job bar)

### Shortcuts and keyboard

15. Ctrl+Shift+A: framing the selection or Zoom to selection, and its label
    (menus Q1, naming Q2, Q3, toolbars Q2)
16. A chord for Device profiles…; Alt+I and Alt+T at M7 (menus Q3; proposed
    changes above)
17. Tab: object stepping versus focus navigation; list keys in the layer
    list (visual-language Q7, layers-panel Q8)
18. Keyboard access to the menu bar, toolbars, palette and property fields;
    `accesskit` (menus Q7, toolbars Q8, visual-language Q8, Q12,
    property-bar Q13, icons Q9)
19. A tab switching chord (layout Q11); a remove-from-selection box
    (style/canvas Q9); Ctrl's two meanings in the Line tool (style/canvas
    Q17); right-click finishing a line (menus Q10, style/canvas proposes
    yes); Esc closing the preview (job-preview Q4); access-key underlines
    (visual-language Q14)

### Look, theme and accessibility

20. The full accent list, including the canvas selection family
    (visual-language Q1, style/canvas Q1); hover by weight alone and the
    trim preview without red (style/canvas Q15, Q16)
21. Bundle a bold face (visual-language Q2)
22. Default theme light everywhere or follow the desktop; dark theme
    timing (visual-language Q3, Q4)
23. Palette data: 01 and 29 both red, faint yellows, darker edge rule,
    matching the reference's table for imports (layers-panel Q1–Q3,
    visual-language Q5, style/canvas Q10)
24. Guide layers, their colors and the word clash with the `guide` token
    (layers-panel Q4, visual-language Q6, naming Q7, style/canvas Q11)
25. Scrollbars, reduced motion from the OS, high contrast, the amber shade
    (visual-language Q10, Q11, Q13, Q15, icons Q10)
26. Swatch shape, used-layer marker, unfocused selection look, column order
    (layers-panel Q17, Q14, Q12, Q18)
27. Preview look: one color for burns, legend default, grid, head at the
    end (job-preview Q1, Q5, Q10, Q9)

### Icons

28. Implementation route (painted, own SVG through our importer, or an SVG
    crate), accent marking in source files, grid units, who draws the set
    (icons Q1–Q4)
29. Universal metaphors the reference also uses; digits in icons; the
    danger rule for deletes (icons Q5, Q8, Q7)
30. App icon direction and a wordmark (icons Q6)
31. Collapsing triangle, badge sizing, the small motion-only Pause mark
    (icons Q11–Q13)
32. Text labels until icons exist; icon-only Job setups buttons (toolbars
    Q10, Q12)

### Layout and windows

33. Which milestone builds the toolbar band (toolbars Q1)
34. Drag-to-redock and its crate; layout persistence and its scope (layout
    Q1, Q2)
35. Align and distribute beside or instead of Arrange (layout Q6, toolbars
    Q4)
36. Saved artwork's dock place (layout Q7); Camera panel default
    (later-windows Q9); Properties hidden by default (property-bar Q17)
37. Recovery from unusable sizes; mirrored layout; multiple windows (layout
    Q8–Q10, menus Q13, visual-language proposed cli.md flag)
38. Canvas view on resize (layout Q12); preview in its own OS window
    (job-preview Q2)
39. Simple layout's hidden set (later-windows Q12)

### Names and wording

40. Save in Unsaved changes (dialogs Q1, naming Q1); "Redo ‹action›"
    (menus Q15)
41. Acceptance of the labels proposed outside `naming.md` (machine-panels
    Q16, layers-panel Q7, dialogs and later-windows proposed lists)
42. "Command line" versus another word; aligning `CLAUDE.md`'s
    "print-and-cut" with Print registration; the quoting exception in
    `naming.md` (naming Q15, Q4, Q18)
43. No selection's buttons (naming Q19); Recent projects' clear row
    (menus Q9, naming Q20); saved artwork rows in the canvas menu (menus
    Q8); "collection" (later-windows Q10); Edit list title (dialogs Q17);
    preset names (naming Q21)
44. Locale formats and a Language menu; Check for updates; Help menu items
    before M10; time format in CLI and UI; Play wording (naming Q14, Q17,
    menus Q12, job-preview Q14, Q3)

### Features and behaviour

45. Editable text objects (naming Q5, property-bar Q7, later-windows Q11)
46. Operations and the Mixed mode (naming Q6, layers-panel Q5)
47. Start guards as switches or fixed (naming Q8, dialogs Q12)
48. Material library files and following a material (naming Q16,
    later-windows Q13)
49. Dialog details: tool windows locking editing, button order on macOS,
    Offset, Grid and Circular array options, Add text preview, Select
    circles, Cut order, Preferences contents and Camera tab, backups,
    Barcode, Font browser, Simplify, Close tolerance, native dialogs,
    "Replace file?" timing (dialogs Q3, Q4, Q6–Q13, Q15, Q18–Q23)
50. Canvas details: marching paths, rotate zones, bottom and right rulers,
    off-bed marking, automatic tabs, equal spacing and auto-scroll, stale
    camera picture, drag read-out chip (style/canvas Q2–Q5, Q8, Q12–Q14)
51. Layers details: image rows' settings, delete confirmation, paste
    timing, edited layer following the canvas, scan pictogram, Air column,
    Layer settings OK and Cancel (layers-panel Q6, Q9–Q11, Q13, Q15, Q16)
52. Property details: reference point default and storage, empty fields,
    handle switches' reach, expressions, inch units, new text placement,
    weld letters, several shapes, power share, run order, typed node
    coordinates, read-only facts, wheel over fields, star point curve
    (property-bar Q2–Q6, Q8–Q12, Q14–Q16, Q18)
53. Toolbar details: paste position, Return to Select, Radius field, camera
    capture on Standard (toolbars Q5, Q7, Q11, Q13)
54. Preview details: Save image and its encoder, left drag, click to jump,
    per-layer table (job-preview Q6, Q11–Q13)
55. Later windows: tests on the canvas, test footers, title field, Z
    reference, rotary contents, built-in presets, calibration pattern,
    camera crates, Denoise (later-windows Q1–Q8, Q14)
56. Menus: Show fills and Smooth curves defaults, text-field context menu
    (menus Q11, Q14)

### Closed in the files

- The Message veil token (`backdrop`, dialogs Q5); no `job-origin-marker`
  token (machine-panels Q14); the slider fill is `accent` (job-preview
  Q15); Edit device profile… from M6 (toolbars Q14).

## Sources

All LightBurn pages read on 2026-10-21 through the research notes of this
guide; facts above are those the other style files cite from the same
pages.

- [R1] https://docs.lightburnsoftware.com/2.1/Reference/UI/ — default
  full-window arrangement, bands, right column groups and proportions,
  light theme, density.
- [R2] https://docs.lightburnsoftware.com/2.1/GetStarted/UITour/ — region
  tour, tab strips at the bottom of dock groups, swatch row, panel title
  strip look, switches and their colors, icon style.
- [R3] https://docs.lightburnsoftware.com/2.1/Guides/CustomizingTheLightBurnWindow/
  — showing, hiding and docking panels, dock title strips, nothing docked
  on the left by default.
- [R4] https://docs.lightburnsoftware.com/2.1/Reference/CutsLayersWindow/
  — the gradient title strip on the layer panel.
- [R5] https://docs.lightburnsoftware.com/2.1/Reference/UI/ColorPalette/ —
  the numbered swatch row.
- [R6] https://docs.lightburnsoftware.com/2.1/Reference/UI/StatusBar/ — the
  status row and its tinted message bands.
- [R7] https://docs.lightburnsoftware.com/2.1/Reference/LaserWindow/ — the
  job-control panel's place and its tall job buttons.
- [R8] https://docs.lightburnsoftware.com/2.1/Reference/SettingsPreferences/
  — pill switches and other controls in the light theme.
- [R9] https://docs.lightburnsoftware.com/2.1/Reference/UI/MainToolbar/ —
  heavy black icon-only toolbar buttons.
- [R10] https://docs.lightburnsoftware.com/2.1/Reference/BeginnerMode/ —
  the reduced mode.
- [R11] https://docs.lightburnsoftware.com/2.1/Reference/ManagingPreferences/
  — a solid blue selected list row.
- [R12] https://docs.lightburnsoftware.com/2.1/Reference/ShowNotes/ — a
  blue focus border on a text box.

Existing documents used: `docs/clean-room.md` (allowed and forbidden
sources, look and feel, "when in doubt, differentiate"), `CLAUDE.md`
(milestones, safety and architecture rules), `docs/spec/safety.md` (rules
1, 2, 7, 9), `docs/spec/ui-shell.md`, `docs/spec/keymap.md` and
`crates/lasercraft-ui/keymap/default.keymap` (F12 bound; Pause and
Ctrl+Break planned for M6 and not bound yet), and every file in
`docs/spec/style/` (their proposed changes and open questions are
consolidated above). Open standards: WCAG 2.x, through
`visual-language.md`.

Own design: the structure of this guide; the three tiers and their
application; every LaserCraft name, label and message; the flat title
strip, the blue accent, the blue and grey switch, the palette values, the
icon style and every icon; the job bar; the command line; the tokens and
their plain-word colors; the dark theme; the suggested build order; the
grouping of proposed changes and open questions.
