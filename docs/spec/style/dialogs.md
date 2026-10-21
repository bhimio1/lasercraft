# Spec: style — dialogs and secondary windows

Milestones: M0 (Unsaved changes, the Open project and Save project as
prompts), M2 (Project notes), M3 (Offset, Grid array, Circular array, Add
text, the Import vector art prompt), M5 (the Save G-code prompt), M6 (No
selection, and the rules every dialog follows while a machine is
connected), M8 (Before you start and its list editor), M9 (the Camera tab
of Preferences), M10 (Quick reference with its About tab). Path array,
Combine shapes, Measurements, Select circles, History, Cut order,
Preferences, Keyboard shortcuts, Restore preferences, Nesting failed, Fit
slots to thickness, Barcode and Font browser are later, with no milestone
yet. Low-power frame is later, optional, and needs the owner's decision.

This file fixes how every LaserCraft dialog, message and secondary window is
built: which kind of window each one is and what it blocks, the title, the
frame, where it opens, the form grid, field widths, the button row, Enter
and Esc, live previews on the canvas, inline errors, message boxes, wizards
and the file prompts. It then specifies each dialog that is not covered
elsewhere, field by field. `visual-language.md` explains the tiers (Match,
Echo, Own) until `README.md`, still to be written, takes them over. Every label comes from `naming.md` (labels this file needs and
`naming.md` does not yet list are collected under "Proposed changes to other
specs"); colors, type and sizes are only the tokens defined in
`visual-language.md`; pictograms are named here by concept and drawn as
`icons.md` says. Where the job bar sits and why nothing may cover it is
`layout.md`'s; the dialog-opening menu items are in `menus.md` and the
buttons in `toolbars.md`; the shared numeric field widget (anatomy, states,
steppers, dragging a value) is `property-bar.md`'s; how previews and
markers are drawn on the canvas is `canvas.md`'s; the job preview window is
`job-preview.md`'s; Layer settings is `layers-panel.md`'s; the device,
GRBL, macro and saved-position dialogs and the Device scan wizard are
`machine-panels.md`'s (their contents; their frames, buttons and keys follow
this file); image, camera, material and test windows are in
`later-windows.md`. The behaviour specs in `docs/spec/` win wherever they
already decide something.

## How to read this file

- Each element has a **Reference** part (what LightBurn's public docs show,
  every fact tagged with a citation key such as [R3], listed under
  Sources), a **LaserCraft** part (the binding spec) and a **Tier** line.
- Labels in double quotes or in the label column of a table are exact UI
  text. Tooltips are our own words; a chord is shown only when
  `crates/lasercraft-ui/keymap/default.keymap` binds it, read from the keymap
  at run time and drawn with key symbols (`naming.md`, Tooltips).
- "Field" means a single-line input; "box" a multi-line text input;
  "dropdown" a closed list (egui `ComboBox`); "switch" the pill toggle of
  `visual-language.md`; "choice" a set of round choice dots (radio
  buttons). "Draft" means the values a window holds before its primary
  button runs anything.
- Tokens are written in backticks (`accent`, `row`, `gap-wide`). No color
  or size is given in any other form. Relative sizes ("about a third of the
  main window wide") are targets for the first open, not measurements.
- Defaults marked "today" are the values the code uses now
  (`crates/lasercraft-ui/src/dialogs.rs:56-86`); the others are own design.

---

## Conventions

### Kinds of window

#### Reference

- The tool dialogs float over the main window with the canvas still visible
  around them [R1][R3][R5]. Some draw their result on the canvas while they
  are open and redraw it when a value changes [R3][R4][R5][R29].
- The notes do not say whether these dialogs block the canvas or other
  windows while open; no screenshot shows the canvas being edited with one
  open (unknown).
- The job preview is a separate top-level window that can be resized from a
  grip in its corner [R31]. The setup wizards are separate wide windows
  [R27][R28].
- Short message boxes carry one or two lines of text and a single
  confirming button [R7][R8].

#### LaserCraft

Every dialog and secondary window is one of six kinds. The kind decides
what stays usable around it (own design; the reference's blocking rules are
unknown).

| Kind | Members | What stays usable while it is open | How it closes |
|---|---|---|---|
| Tool window | Offset, Grid array, Circular array, Path array, Combine shapes, Close nearly closed paths, Simplify, Fit slots to thickness, Barcode, Add text, Select circles, History | Pan and zoom on the canvas, every view action (zoom chords, F12, panel tabs, View menu), the job bar and every Machine panel control that does not change the project. Everything that would change the project or the selection (canvas editing, menu and toolbar commands, Layers panel edits, editing chords) is disabled; such menu rows and buttons show the second tooltip line "Close the ‹title› window first." | primary button, Cancel, the title strip's close button (same as Cancel), Esc |
| Settings window | Preferences, Keyboard shortcuts, Cut order, Project notes, Restore preferences, Font browser, Edit list | Everything. The window only edits its draft; nothing changes until its primary button | primary button, Cancel, close button (= Cancel), Esc |
| Info window | Measurements, Quick reference (and Job preview, `job-preview.md`) | Everything | "Close", close button, Esc |
| Message | Unsaved changes, No selection, Nesting failed, Replace file?, Switch device profile?, Before you start, Low-power frame, and `machine-panels.md`'s confirmations | Nothing except the job bar and the Pause and Stop chords. A backdrop dims the rest of the main window | one of its buttons; close button and Esc = its Cancel; in a message with no Cancel, they press its safe button (the one with focus that runs or changes nothing: OK in No selection, "Keep ‹current name›" in Switch device profile?, OK when it has only OK) |
| Wizard | Device scan and New device profile (`machine-panels.md`), the camera, registration and centre-finding wizards (`later-windows.md`) | As a tool window, plus the machine controls a page asks for | Finish, Cancel, close button (= Cancel), Esc |
| File prompt | Open project, Save project as, Import vector art, Save G-code | As a settings window | OK, Cancel, close button (= Cancel), Esc |

Rules for every kind (own design):

- At most one tool window or wizard is open at a time. Asking for a second
  one (menu, button or chord) brings the open one to the front and focuses
  it instead; the open one is never cancelled silently. Today only one of
  the four argument windows can be open (`dialogs.rs`), which matches.
- Each settings and info window exists once; asking for it again brings it
  to the front.
- Messages never stack. A message that arrives while another is shown waits
  and appears when the first closes, in the order they were raised.
- "Modal" in this guide means the Message kind's backdrop. LaserCraft never
  uses `egui::Modal` while a machine is connected, because its backdrop
  takes every click, including the job bar's (`layout.md`, egui notes).
- No window ever opens by itself while the user is typing, except a
  message raised by an action the user just took (own design).

Tier: **Own** (the six kinds and their rules; the reference's are unknown).

### Frame and title

#### Reference

- On Windows each tool dialog has a light title strip with a small icon at
  its left (the tool's own toolbar pictogram where it has one, otherwise
  the program's logo), the text "‹tool name› - ‹program name and version›"
  and only a close button at its right [R1][R2][R3][R4][R5]. A few dialogs
  add a "?" help button beside the close button [R6][R19][R29].
- On macOS the title strip shows only the tool's name, centred, with the
  system's window buttons at the left [R1][R3].
- Dialog bodies are a flat light grey with dark regular text; no colored
  bands appear inside them [R1][R3][R14].

#### LaserCraft

- Every dialog is drawn by egui inside the main window (decision 12: one
  look on every platform), so its title strip is LaserCraft's own chrome,
  not the operating system's.
- Title strip: `panel-title` background, a little shorter than `row`
  (as `layout.md` and `visual-language.md` draw every title strip), the title in
  `type-body` `text` at the left, the close button (a small cross,
  `icon-small`, `text-muted`; `hover` background under the pointer) at the
  right. No icon before the title and no app name or version in it (the
  window is already inside LaserCraft). No "?" button: context help is F1
  over any control, including dialog fields (M10, keymap.md).
- The window that has keyboard focus shows a thin `accent` line along the
  lower edge of its title strip, the same marker as a focused dock panel
  (decision 2).
- Body: `panel` background, a thin `border` outline, slightly rounded
  corners and the `shadow` token under it (`visual-language.md` sets its
  strength). Content sits inside a `gap-wide` margin on every side.
- Windows are never collapsible (today's `collapsible(false)`,
  `dialogs.rs:115-117`); a double-click on the title strip does nothing.
- Title wording (`naming.md`, Parts of speech): the title repeats the
  command that opens the window without its ellipsis ("Offset", "Grid
  array", "Add text"), or is a noun when no single command opens it
  ("Preferences", "History"). Confirmations use a short question naming the
  action ("Replace file?") or a noun for a state ("Unsaved changes").
  Sentence case; never "LaserCraft" in the title; never a trailing
  ellipsis.
- A window shown later as a separate operating-system window (egui
  multi-viewport, later) takes the OS title "‹title› — LaserCraft".

Tier: **Echo** (close-only title strip, plain light body), **Own** (strip
look, focus line, no icon, title rules).

### Where windows open, their size, and remembering them

#### Reference

- Tool dialogs are shown placed beside the shapes they act on so the shapes
  and the dialog are both visible [R3][R5].
- Sizes differ per dialog: wide and short (the offset dialog, the
  slot-resizing dialog), tall and narrow (the grid array dialog, the undo
  history window), small and square (the copy-along-path dialog)
  [R1][R3][R4][R6][R12].
- The settings dialog keeps one size on all of its tabs, so short tabs
  leave empty space above the buttons [R14].
- Only the job preview shows a resize grip; the Windows dialog title strips
  have no maximize button [R31][R1].
- Whether dialogs remember their position or size is not stated (unknown).

#### LaserCraft

First open, by kind (own design):

| Kind | Where it first opens |
|---|---|
| Tool window | At the right edge of the canvas area, its top just below the canvas's upper edge, so the selection usually stays visible. If the window would cover the selection bounds there, it opens at the left edge of the canvas instead; if both would cover them, at the right. |
| Add text | Beside the click point, to its right and below it, moved inward so it fits; the click point's `drawing` cross (`canvas.md`) stays visible. |
| Message | Centred horizontally over the main window, in its upper third. |
| Settings, info, file prompt, wizard | Centred over the canvas area. |

- Size: each window sizes to its content on first open. Widths are relative
  to content, never to the screen, except where a section below says
  otherwise. A window never grows larger than the canvas area; when content
  is taller, the body scrolls and the button row stays fixed at the bottom.
- Resizable windows (by dragging an edge or corner, with egui's resize
  corner mark drawn in `text-muted`): Project notes, History, Keyboard
  shortcuts, Restore preferences, Font browser, Quick reference, Edit list.
  All others keep the size their content gives them (today's
  `resizable(false)`, `dialogs.rs:117`).
- Tabbed windows (Preferences, Barcode, Quick reference) keep one size on
  every tab, the size of their largest tab (Match [R14]).
- Wizards keep one size on every page, the size of their largest page.
- Windows are constrained to the main window. While a machine is connected
  they are also kept below toolbar row 1, so none can be dragged over the
  job bar (the bar is drawn above them anyway; `layout.md`).
- Within one run, a window reopens where and at the size it was last left
  (egui keeps window state by id for the session). Across runs nothing is
  remembered until layout persistence exists (`layout.md`, stage p; owner
  decision).
- Everything scales with egui's display scale; no window has a size in
  device pixels.

Tier: **Echo** (beside the shapes, one size per tabbed window), **Own**
(placement rules, resizable set, constraint, session memory).

### Form grid and section headings

#### Reference

- Most forms place each label right-aligned directly to the left of its
  control, forming a label column and a control column [R1][R2][R6][R5].
- Rows that hold only a switch are usually left-aligned with the switch
  before its words [R2][R3][R4]; one dialog mixes both orders [R1].
- Related controls are grouped in slightly recessed rounded boxes with a
  plain heading just above each box, sometimes several boxes side by side
  [R3][R1][R14]; one older dialog uses a classic titled frame [R6]; thin
  full-width rules separate stacked parts of one dialog [R1].
- Section captions in the settings dialog are regular weight, not bold
  [R14]. Spacing is dense: one control per text line, more space between
  groups than between rows [R14][R1].

#### LaserCraft

The grid (Echo of the label/control columns; own details):

| Part | Rule |
|---|---|
| Label column | Labels right-aligned, `type-body` in `text`, no trailing colon, the unit in parentheses as part of the label ("Distance (mm)", `naming.md` Units). The column is as wide as the longest label in its group. |
| Gap | `gap-tight` between a label and its control. |
| Control column | Controls left-aligned. Fields keep the dialog's numeric width (below); text fields and boxes stretch to the content's right edge. |
| Rows | One `row` tall; `gap-tight` between rows of one group; `gap-wide` between groups. |
| Switch rows | The switch sits at the start of the control column with its label to its right in `type-body` `text`; the label column stays empty. Clicking the label toggles the switch. This keeps every control in one column (own; the reference puts switches at the far left). |
| Switch with a value | A switch whose setting needs a number ("Last copy size (%)") has its field on the same row, after the label; the field is disabled while the switch is off. |
| Choice rows | A choice with three or fewer short options lays them out side by side in the control column; longer ones stack, one option per row. The group's label, if any, sits in the label column on the first row. |
| Choice with a field | When each option carries its own value ("Fixed count", "Fixed spacing (mm)"), each option is its own row: the choice dot and its label in the label column, right-aligned, the field in the control column; a field is disabled unless its option is chosen. |
| Hints | A hint sits under its control, in the control column, in `type-small` `text-muted` ("Negative moves inward"). |
| Read-only summaries | A summary line under the form, left-aligned with the content edge, in `type-body` `text-muted`, above the error band and the button row. |

Section headings and groups:

- A heading in `type-heading` `text` stands above each named group,
  left-aligned with the content edge, `gap` above its first row. This is the
  only place `type-heading` is used (`visual-language.md`: dialog section
  headings only); wizard page headings use it too, as section headings of a page.
- Groups have no boxes or frames (own; differs from the reference's
  recessed boxes): headings and `gap-wide` spacing separate them. Where two
  stacked parts of a window have no heading (a form, then a summary), a thin
  `border` rule spans the content width between them.
- Wide windows (Preferences, Cut order, Measurements, Restore preferences)
  place groups in two columns, left column first; each column has its own
  label column. Narrow windows use one column.
- No colored bands inside a dialog except the error and warning bands
  (below) and the Low-power frame's warning band.

Field widths (own design, extending `property-bar.md`'s rule):

- All numeric fields in one window share one width: enough for the longest
  value the field ranges allow, with sign and the shown decimals
  (`naming.md`, Numbers), plus the stepper.
- Text fields and boxes stretch to the content's right edge. File path
  fields scroll horizontally when the path is longer.
- Dropdowns are as wide as their longest option plus the chevron, but never
  narrower than the numeric width; font dropdowns stretch like text fields.
- Boxes have a stated height in text lines (each section says how many).

Tier: **Match** (right-aligned labels left of controls, headings over
groups, dense rows), **Echo** (switch rows, two-column groups), **Own**
(no group boxes, every token, width rules).

### Fields, dropdowns, choices and switches

#### Reference

- Number fields are white boxes with a thin grey border and two tiny
  stacked arrow buttons at their right edge; values show a fixed number of
  decimals; a unit is a separate small label after the box or a suffix
  inside the value [R1][R2][R4][R6][R30].
- The mouse wheel over a number field changes it [R3]. Disabled fields have
  pale text and arrows; the focused field gets a blue border or ring
  [R1][R5].
- Dropdowns are white fields with a small down chevron; the open list is
  white with a blue band on the current row [R5].
- Choices are standard round radio buttons [R1][R3][R4][R6]. Switches are
  small pills, green when on and dark red when off, used instead of
  checkboxes [R1][R2][R3][R4][R14].

#### LaserCraft

- Number fields are the shared field widget of `property-bar.md` (Shared
  field behaviour): its anatomy, its states (Normal, Hover, Focused,
  Invalid, Disabled, Read-only), the steppers, the step sizes, Up and Down
  keys, wheel only when focused, and dragging a value. The unit is in the
  label, never inside the field (`naming.md`).
- One difference in dialogs: a commit (Enter, Tab, focus loss, a stepper
  click, the end of a drag) updates the window's draft and its preview; it
  runs no command and makes no undo step. The primary button runs one
  command for the whole draft (`docs/spec/ui-shell.md`, M3 additions).
- Whole-number fields show no decimals; their ranges are given per field
  below. Out-of-range input is clamped (`naming.md`, Numbers).
- Dropdowns: `field` box with a thin `border` outline, the current option in
  `type-body` `text`, a small chevron in `text-muted` at the right. The open
  list is a `field` popup whose rows follow `menus.md`'s row look (pointer
  row and current row in `accent-soft`, the current row with a tick).
  Options are readable names, never raw values (`naming.md`).
- Choices: a small circle in `field` with a `border` ring; the chosen one
  holds a filled `accent` dot. Disabled choices are drawn faded in
  `text-disabled`. Clicking the label chooses it.
- Switches: the pill of decision 4 (`visual-language.md`): `accent` fill
  with the knob right when on, neutral grey with the knob left when off,
  faded when disabled. Never a checkbox; today's Circular array checkbox
  becomes a switch (`dialogs.rs:175`).
- Boxes (multi-line text): `field` background, `border` outline, `type-body`
  `text`, a vertical scrollbar when the text is longer than the box; the
  Focused state matches the field's.

Tier: **Echo** (stepped fields, pill switches, round choices), **Own**
(tokens, draft-only commits, no unit suffixes).

### Button row

#### Reference

- Buttons are text-only rectangles of equal width at the bottom right of
  the dialog [R1][R2][R3][R4][R6][R14].
- On Windows the order is OK then Cancel, with Apply before OK where a
  dialog has it; on macOS Cancel comes before OK [R1][R3][R6][R30].
- The default button (OK) has a blue outline on Windows and a filled blue
  background on macOS [R1][R3][R16].
- Extra actions that do not close the dialog sit at the bottom left
  [R13]. The unsaved-changes prompt offers three choices: save first, do
  not save, or cancel [R20]; its button labels and which one is the
  default are unknown. Simple message boxes have one OK button, sometimes on a
  slightly darker bottom strip [R7][R8][R26].

#### LaserCraft

Placement and order (one order on every platform, decision 12; Match of the
reference's Windows order):

- The button row is the last row of the window, below a thin `border` rule
  across the content width, with `gap` above and below.
- The right group is right-aligned, in this order: Apply (when the window
  has it), the primary button, any alternative actions, Cancel. Examples:
  "OK", "Cancel"; "Apply", "OK", "Cancel"; "Save", "Discard", "Cancel";
  wizards "Back", "Next", "Cancel".
- The left group, left-aligned, holds actions that do not close the window
  ("Save as default", "Restore default", "Reset all", "Import keymap…",
  "Export keymap…", "Copy values") and, in Project notes, a switch.
- All buttons of one group share one width: the widest label of the group
  with `gap` on each side. Height `row`. Text only, no icons (Match).

Button styles:

| Style | Fill | Label | Border | Used for |
|---|---|---|---|---|
| Primary | `accent` | `type-body` in `text-on-strong` | none | the one main action of the window: OK, Save, Save notes, Start job, Next, Finish, Use font, Restore backup, Place the parts that fit |
| Normal | `field` | `type-body` in `text` | thin `border` | Cancel, Apply, alternatives, left-group actions |
| Destructive | `danger` | `type-body` in `text-on-strong` | none | actions that lose work or data: Discard, Replace, Remove profile (`naming.md`, Confirmations) |
| Laser on | `warning` | `type-body` in `text-on-warning` | thin `text-on-warning` outline | only the Low-power frame's frame button |
| Hover | as the style, one step stronger (`hover` over Normal; a slightly darker shade of the fill for the others, `visual-language.md`) | unchanged | unchanged | pointer over an enabled button |
| Pressed | `pressed` over Normal; a darker shade for the others | unchanged | unchanged | while the mouse button is held |
| Disabled | `field-alt` | `text-disabled` | thin `border` | the action cannot run now; the tooltip's second line says why |
| Keyboard focus | as the style | unchanged | an `accent` focus ring just outside the button | the button has keyboard focus (never shown for mouse clicks) |

- Exactly one primary button per window, or none when every choice is
  risky (Low-power frame).
- Labels (`naming.md`): plain forms use "OK" and "Cancel"; windows that
  only show things use "Close"; confirmations use verbs naming the result
  and "Cancel", never "Yes", "No" or "OK".
- Contrast: label against fill at least 4.5:1, a button's outline or fill
  against `panel` at least 3:1, and the focus ring visible (WCAG 2.2
  1.4.3, 1.4.11, 2.4.7); `visual-language.md` picks the shades that meet
  these targets.

Tier: **Match** (bottom-right group, OK then Cancel, Apply first, extras at
the left, text-only equal buttons), **Own** (single order on every
platform, the styles and their tokens).

### Keyboard: focus, Enter, Esc and Tab

#### Reference

- Whether Enter confirms and Esc cancels a dialog, the tab order, and when
  typed values take effect are not stated in the notes (unknown) [R1].
- Fields are shown focused on open in some dialogs: the notes box and the
  hotkey editor's search field have the focus border when they open
  [R19][R15].

#### LaserCraft

All own design:

- On open, keyboard focus goes to the first field of the window with its
  whole value selected (so typing replaces it). Windows without fields
  focus their primary button. Messages focus their safe button: the
  primary when it is safe (Save, OK), otherwise Cancel. Before you start
  and Low-power frame focus Cancel.
- Enter, with focus in a single-line field, on a switch, on a choice or on
  no widget: commits the field and presses the primary button if it is
  enabled. With focus on a button: presses that button.
- Enter in a box (multi-line text) starts a new line; Ctrl+Enter presses
  the primary button.
- Enter never presses a Destructive or Laser-on button, and never presses
  Start job; those need a click or Space on the focused button.
- Esc, in order: closes an open dropdown list; else, if a field holds typed
  text not yet committed, reverts that field (`property-bar.md`); else
  presses Cancel (or Close; in a message with no Cancel, its safe button,
  see Kinds of window). During key
  capture in Keyboard shortcuts, Esc ends the capture only.
- Tab and Shift+Tab move focus in reading order: groups of the left column
  top to bottom, then the right column; rows top to bottom; controls in a
  row left to right; then the left button group, then the right group in
  its visual order. Focus stays inside the window that has it until the
  window closes or the user clicks elsewhere.
- Space toggles the focused switch, chooses the focused choice and presses
  the focused button.
- Main-window chords while a window has focus: ignored while a text field
  has focus (keymap.md); otherwise view chords (zoom, F12) work, and
  project-changing chords are disabled while a tool window, wizard or
  message is open (see Kinds). The Pause and Stop chords always work, even
  with focus in a field (proposed change to keymap.md, already raised in
  `layout.md`).
- No access-key underlines (`menus.md`).

Tier: **Own**.

### Live previews on the canvas

#### Reference

- Offset draws the outline it would make on the canvas and redraws it as
  settings change; in one capture the original is shown dashed and the
  preview outlines as solid thin dark lines [R3].
- The copy-along-path dialog shows its copies on the canvas while options
  change [R4];
  the bar code redraws on the canvas while its text is typed, and invalid
  data replaces it with a large cross [R5]; the boolean helper previews an
  operation when its button is hovered or clicked [R9]; the taper warp
  previews while values change [R29].
- Whether the grid array dialog and the circular array dialog preview on
  the canvas is not stated [R1][R2].

#### LaserCraft

- Every tool window that creates or changes geometry draws its result live:
  Offset, Grid array, Circular array, Path array, Combine shapes, Fit slots
  to thickness and Barcode. Add text draws only its click-point cross today
  (`canvas.md`; a text preview is proposed under Open questions).
- Look (`canvas.md`, Tool window previews): the result is drawn as *normal*
  dashed lines in `drawing`; the original keeps its Selected look and its
  selection bounds. A Barcode whose data is invalid shows a cross through
  its box in `danger` instead of the code (Echo [R5]; proposed change to
  `canvas.md`).
- History is the exception: it shows the real project at the chosen point,
  drawn normally, because it really steps the history (see History).
- The preview updates on every commit in the window (Enter, Tab, focus
  loss, a stepper click), on every switch, choice or dropdown change and
  continuously while a value is dragged. It is not updated on each keystroke
  of a typed number (same rule as `property-bar.md`).
- The UI computes nothing (CLAUDE.md, the UI crate has no business logic):
  the window hands the app the invocation its draft would run; the app runs
  it on a scratch copy of the project and hands back the shapes to draw and
  a summary line. A dry run that fails draws no preview and shows its
  message in the window's error band. (Proposed change to ui-shell.md and
  commands.md.)
- Cancel, the close button and Esc remove the preview at once. OK replaces
  it with the real result, which is selected as the command decides
  (`arrange.md`).
- While a tool window is open the canvas can be panned and zoomed so the
  preview can be checked from close up.

Tier: **Echo** (live previews that follow the fields), **Own** (which
windows, the dry-run path, timing, tokens).

### Validation and inline errors

#### Reference

- The bar code dialog counts remaining characters under the text box, turns
  the count red and negative when over the limit, shows a red line naming
  bad characters, and crosses out the code on the canvas [R5].
- The slot-resizing dialog lists unsupported items in plain text under its
  choices
  [R6]. Fields that cannot apply are greyed in place [R1][R4][R13].

#### LaserCraft

Three levels, each shown where it happens (own design):

1. **Field.** Out-of-range values are clamped as they are committed
   (`naming.md`); text that cannot be read as a number puts the field in
   its Invalid state (`property-bar.md`). In a dialog the message for it
   goes to the window's error band, not to the status bar.
2. **Form.** A rule across fields (a grid needs at least two blocks, "Up to
   (mm)" smaller than the diameter, text with nothing visible) shows its
   message in the error band and disables the primary button; the primary
   button's tooltip repeats the message as its second line.
3. **Command.** When the dry run or the real run fails, the message from
   the command is shown in the error band in `naming.md`'s message form
   ("Cannot ‹verb›: ‹why›. ‹next step›."), and the window stays open with
   its draft, so the user can fix it. Today the window closes and the error
   only reaches the command line log (proposed change to ui-shell.md).

The error band:

- Sits across the content width just above the button rule, `gap` from the
  form. `danger-soft` background, a small `danger` mark (`icon-small`,
  `mark-error` in `icons.md`) at its left, text in `type-body` `text`.
- Warnings use the same band in `warning-soft` with the `warning` mark (`mark-warning`) and
  the lead-in "Warning: " (tests look for it, `naming.md`). Warnings never
  disable the primary button.
- Shows at most three messages, then "and ‹n› more".
- Appears and disappears without moving the button row: the window grows by
  the band's height while it is shown (no layout jump of the form above).
- Controls that cannot apply are disabled in place, never hidden; their
  tooltip's second line says what enables them ("Turn on Last copy size (%)
  first.").

Tier: **Echo** (greyed dependent controls, inline red text), **Own** (three
levels, the band, keeping the window open on errors).

### While a machine is connected and while a job runs

This section applies `docs/spec/safety.md` rule 7 and decision 7 to every
window (own design, binding for all style files).

- No dialog, message, wizard, menu or backdrop covers or blocks the job
  bar. The bar is drawn above every window; windows are kept below toolbar
  row 1 while connected; the Message backdrop leaves the bar's area out and
  lets its clicks through (`layout.md`).
- No `egui::Modal` while connected (its backdrop takes every click).
- The Pause and Stop chords work whichever window has focus, including a
  text field (proposed change to keymap.md).
- Opening a window never moves the head and never turns the laser on. Only
  an explicit button inside a window does (Start job, the Low-power frame's
  frame button, a wizard page's burn or move button), and each such button
  follows `docs/spec/safety.md` (rules 1, 2 and 5).
- While a job or framing runs:
  - Windows that change the machine or move the head refuse to open: the
    Device profile and Device profiles dialogs, GRBL settings, the wizards,
    Low-power frame, Saved head positions. Their menu rows and buttons are
    disabled with the second tooltip line "Not while a job runs."
  - If one is already open when the job starts (for example a Device
    profile left open), it turns read-only: every control shows its
    Read-only state, the primary button is disabled, and a `warning-soft`
    band says "Settings cannot be changed while a job runs." Cancel still
    closes it.
  - Project windows (tool windows, Cut order, Project notes) stay usable;
    the running job keeps the plan it started with. Cut order shows the
    warning "Warning: changes apply to the next job." while a job runs.
  - Preferences stay usable.
- An alarm, a stall or a disconnect while a window is open leaves the
  window open; the alarm shows in the job bar and the Machine panel
  (`machine-panels.md`). A wizard page that needs the connection shows the
  message in its error band and disables Next.

Tier: **Own**.

### Message boxes

#### Reference

- An error box: a large red stop-sign shape holding a white cross at the
  left, one short line of text, and a single OK at the bottom right with
  the default outline, inside a softly shadowed box [R7].
- An information box with no icon: two lines of plain text and one OK on a
  slightly darker bottom strip; wide and short [R8].
- A warning box (missing fonts in a project): a large yellow circle with an
  exclamation mark, the message, and OK on a darker footer strip [R26].
- The unsaved-changes prompt offers three choices: save first, do not
  save, or cancel [R20]. Opening a file saved for another device asks
  whether to switch devices, answered Yes or No [R20]. The icons, the
  button labels, their order and the default button of these two prompts
  are unknown (the notes do not record them).

#### LaserCraft

Anatomy, top to bottom (own design):

| Part | Rule |
|---|---|
| Title strip | as every window; the title is the message's name ("No selection") |
| Icon | `icon-big`, at the left of the text, top-aligned with its first line |
| Text | right of the icon, `gap-wide` from it: the first sentence says what happened (`type-body` `text`); the next sentences say why and what to do (`type-body` `text`), following `naming.md`'s three-part message form. Wraps at about a third of the main window's width; never scrolls |
| Details (optional) | a short list in `type-small` `text-muted` under the text (skipped items, file names) |
| Button row | as every window |

Kinds, each with an own icon (concepts for `icons.md`; outline style,
decision 17):

| Kind | Icon concept | Icon token | Used by |
|---|---|---|---|
| Error | `message-error`: a circle outline holding an exclamation mark | `danger` | Nesting failed (nothing fits, no arrangement), Registration failed (`later-windows.md`) |
| Warning | `message-warning`: a triangle outline holding an exclamation mark | `warning` | Unsaved changes, No selection, Nesting failed (some parts do not fit), Replace file? |
| Question | a circle holding a question mark | `accent` | Switch device profile? (M8) |
| Info | `message-info`: a circle outline holding the information mark (a dot over a short stem; no letters inside icons) | `accent` | notices that must be acknowledged (rare; most notices go to the Messages field instead) |
| Task | none | — | Before you start, Low-power frame (their content carries its own marks) |

- Backdrop: while a message is shown, the rest of the main window is
  covered by a translucent veil that dims it and takes its clicks, except
  the job bar's area, which stays clear, undimmed and clickable. The veil
  is drawn in `backdrop` (`visual-language.md`).
- No sounds; messages never time out; a message shows its buttons only
  (never an extra "Don't show again" switch, own design).
- Most notices are not messages: success and info go to the status bar's
  Messages field (`layout.md`, `naming.md`); a message is used only when the
  user must choose or acknowledge before going on.

Tier: **Echo** (icon at the left, short text, buttons at the bottom right,
three-button unsaved prompt), **Own** (icons, kinds, backdrop, wording).

### Wizards

#### Reference

- The device discovery wizard is a wide window: an inset page on the right
  and three equal buttons at the bottom right, "go back", "next" and
  "cancel", the first greyed on the first page; the add button is greyed
  while the results list is empty [R27]. It also carries
  a large faded brand watermark at its left [R27].
- While scanning, a small separate box with its own title strip floats over
  the page with a one-line message and a blue progress bar showing its
  percentage [R27].
- The manual setup wizard has a header with a small back arrow and the
  wizard's name underlined, a white page and a footer strip with Next and
  Cancel; one capture uses a classic style with back and next buttons
  carrying arrow marks; Next stays greyed until a required field is filled;
  a final review page has Back and Finish [R28].
- Wizard prompts are drawn noticeably larger than body text and their hints
  smaller [R27][R28].
- A step indicator is not shown in the notes (unknown).

#### LaserCraft

Layout, top to bottom (own design):

| Part | Look |
|---|---|
| Title strip | the wizard's name (`naming.md`: "Device scan", "New device profile", …) |
| Step indicator | one line listing the steps by name, "1 Connect · 2 Scan · 3 Name and size · …", in `type-body`; the current step in `text` with an `accent` underline marker, done steps in `text` with a small tick before the number, later steps in `text-muted`. Not clickable. When the line does not fit it shows "Step ‹i› of ‹n›: ‹name›" instead |
| Page heading | `type-heading` `text`, a short question or instruction ("Where is the machine origin?") |
| Page hint | under the heading, `type-small` `text-muted` |
| Page body | the form grid of this file |
| Error band | as every window |
| Button row | right group "Back", "Next" (primary), "Cancel"; on the last page "Next" becomes "Finish" (primary); a "Skip" (Normal) appears before Next only on pages that can be skipped |

- Back is disabled on the first page. Next and Finish are disabled until
  the page is complete; their tooltip says what is missing (Echo [R28]).
- Progress (a scan, a capture) is shown inside the page, never in a
  separate floating box: a bar in `field` with an `accent` fill and a line
  of text under it (`machine-panels.md`, Device scan). Own.
- Enter presses Next or Finish when enabled; Esc and the close button
  press Cancel. Cancel never asks; nothing is saved until Finish (Echo of
  the reference's device list, where Cancel drops everything new [R32]).
- No artwork, watermark or logo on any page (the reference's is vendor
  branding).
- A page that moves the head offers it through an explicit button on the
  page; a page that burns (camera alignment, M9) follows
  `docs/spec/safety.md` like a job (`later-windows.md`).

Tier: **Match** (Back, Next or Finish, Cancel; disabled until complete),
**Echo** (larger prompts with smaller hints), **Own** (step indicator,
inline progress, no artwork).

### File prompts now, native file dialogs later

#### Reference

- Saving uses the operating system's own file dialog, with a file type
  dropdown that lists the project format [R20].

#### LaserCraft

- Now: a small window asking for a typed path (ARCHITECTURE.md defers
  native file dialogs). The four prompts are specified under "Dialogs that
  exist today".
- Later (owner decision; a crate such as `rfd`, with the XDG desktop portal
  on Linux and inside Flatpak; needs an ARCHITECTURE.md line): Open…,
  Save as…, Import… and Save G-code… open the native dialog of the
  platform, starting in the folder last used for that kind of file, with
  these filters: "LaserCraft project (*.lcproj)", "Vector art (*.svg,
  *.dxf)", "G-code (*.gcode, *.nc)". The typed prompt stays as the fallback
  when no portal is available and is what the UI tests use.
- The native dialog is the one place where the OS's look appears; it is not
  imitated in egui (decision 12).

Tier: **Match** (native dialogs later), **Own** (typed prompt, filters,
fallback).

---

## Dialogs that exist today

### Offset

#### Reference

- Opened from the Tools menu, the offset button or Alt+O; a Ctrl+click on
  the button repeats the last offset without the dialog [R3].
- Wider than tall. A distance field at the top, then three groups side by
  side: the direction (outward, inward, or both, which makes two outlines;
  inward needs closed shapes), the corner style (round, flat cut or sharp)
  and options (ignore shapes nested in others, select the result, delete
  the originals, simplify the result), then OK and Cancel [R3].
- The result is drawn on the canvas while the dialog is open; the dialog is
  placed beside the shape [R3].

#### LaserCraft

- Title "Offset". Kind: tool window. Opened by Tools → Offset… (Alt+O) and
  the Offset button of Shape operations (`toolbars.md`, which also owns the
  Ctrl+click repeat). The menu row is enabled when something is selected.
- Contents:

| # | Label | Control | Default | Range | Tooltip |
|---|---|---|---|---|---|
| 1 | Distance (mm) | number field; hint "Negative moves inward" | 1.00 (today), then the last value used in this run | −1000 to 1000 (own) | "How far the new outline lies from the shapes. Negative values shrink them." |
| 2 | Corners | dropdown: Round, Miter, Bevel | Round (today) | — | "How the corners of the new outline are shaped." |

- Buttons: "OK", "Cancel".
- Preview: the new outline in `drawing` (Conventions). OK runs `path.offset
  ‹distance› ‹round|miter|bevel›` on the selection (`arrange.md`): the
  inputs are kept and the new outline is selected.
- Errors (worded per `naming.md` from `arrange.md`'s messages): open shapes
  in the selection → "Cannot offset: the selection has open shapes. Close
  them with Close path, or select closed shapes only."; an empty result →
  "Cannot offset: nothing is left at this distance. Use a smaller negative
  distance."
- Distance 0 is allowed (it rebuilds the outline, `arrange.md`); the hint
  stays the same.
- Not offered (arrange.md decides the behaviour): the both-sides direction
  and ignoring nested shapes (open: Open questions, item 6); deleting the
  originals and simplifying the result (left out by design: Deliberately
  omitted).

Tier: **Match** (one distance, a corner choice, live preview, OK and
Cancel), **Echo** (a signed distance instead of a direction choice),
**Own** (labels, layout).

### Grid array

#### Reference

- Opened from the Arrange menu or the grid button; tall and narrow, the
  largest of the array dialogs [R1].
- Two side-by-side columns headed for the horizontal and the vertical
  direction, each with: copy count, total size (an alternative way to set
  the count), spacing, and an alternate-column or alternate-row shift; then
  whether spacing is measured centre to centre or edge to edge; switches to
  reverse direction, shift by half and mirror alternate columns or rows
  [R1].
- Centred rows below for a random rotation with a seed, auto-incrementing
  variable text, and linked ("virtual") copies; a centred read-only line
  with the total size and count, updating live; switches to group and to
  select the result; OK and Cancel [R1].

#### LaserCraft

- Title "Grid array". Kind: tool window. Opened by Arrange → Grid array…
  and the Grid array button of Shape operations. Enabled when something is
  selected.
- Contents, two groups side by side without headings (Echo of the
  reference's two direction columns; the labels say the direction):

| # | Column | Label | Control | Default | Range | Tooltip |
|---|---|---|---|---|---|---|
| 1 | left | Columns | whole-number field | 3 (today) | 1–100 (today) | "Blocks across, the original included." |
| 2 | left | Column gap (mm) | number field | 5.00 (today) | −1000 to 1000 | "Space between neighbouring blocks, edge to edge. Negative values overlap them." |
| 3 | right | Rows | whole-number field | 2 (today) | 1–100 (today) | "Blocks down, the original included." |
| 4 | right | Row gap (mm) | number field | 5.00 (today) | −1000 to 1000 | as Column gap |
| 5 | full width, later | Make linked copies | switch | off | — | "Copies follow later edits to the original." (Later; proposed change to arrange.md, `naming.md`) |

- Summary line (from the app's dry run): "‹n› blocks, ‹w› × ‹h› mm overall"
  (Echo of the reference's total line [R1]; own words).
- Buttons: "OK", "Cancel".
- Preview: every new block in `drawing`. OK runs `arrange.grid ‹columns›
  ‹rows› ‹x_gap› ‹y_gap›` (`arrange.md`); originals and copies end up
  selected.
- Form rules: Columns × Rows equal to 1 → "Cannot make the array: it needs
  at least two blocks." with OK disabled; more than the command's copy
  limit → "Cannot make the array: it would make ‹n› shapes, more than the
  limit of 10 000." (`arrange.md`).
- Values are remembered for the run.
- Not offered (`arrange.md` leaves them out by design): total size as
  input, centre-to-centre spacing, shifts, reversing, mirroring, random
  rotation, auto-incrementing text, grouping the result. Open question.

Tier: **Echo** (two direction columns, count and gap per direction, total
line), **Own** (labels, layout, one spacing mode).

### Circular array

#### Reference

- Small and compact, a little wider than tall [R2].
- Rows for a start angle, an end angle with the number of copies on the
  same row, and the angle step between copies (linked to the other two);
  a centre given as two fields; switches to use the last-selected object's
  position as the centre, to rotate the copies to face the centre, to
  auto-increment variable text, to group and to select the result; OK and
  Cancel on the last row [R2].

#### LaserCraft

- Title "Circular array". Kind: tool window. Opened by Arrange → Circular
  array… and the Circular array button. Enabled when something is
  selected.
- Contents:

| # | Label | Control | Default | Range | Tooltip |
|---|---|---|---|---|---|
| 1 | Total count | whole-number field; hint "Includes the original" | 6 (today) | 2–1000 | "Blocks in the finished ring, the original included." |
| 2 | Center X (mm) | number field | centre of the work area (today) | the bed's extent and a margin around it (own) | "Point the blocks turn around." |
| 3 | Center Y (mm) | number field | centre of the work area (today) | as Center X | as Center X |
| 4 | Sweep (°) | number field | 360.00 (today) | −360 to 360 | "Angle the blocks spread over, clockwise. 360 spaces them evenly round a full turn." |
| 5 | Rotate copies | switch | on (today) | — | "Turn each block as it goes round. Off keeps every block upright." |

- Summary line (from the dry run): "One block every ‹a›°".
- Buttons: "OK", "Cancel".
- Preview: every new block in `drawing`, and the centre as a small cross in
  `drawing` (as `canvas.md` marks centres while drawing).
- OK runs `arrange.circular ‹count› ‹cx› ‹cy› ‹sweep› ‹rotate|upright›`
  (`arrange.md`).
- Not offered: start and end angles (Sweep replaces them), using the
  last-selected object as the centre, auto-incrementing text, grouping and
  selecting the result (always selected). Open question for the centre.

Tier: **Echo** (count, centre, angle, rotate switch), **Own** (Sweep instead
of start and end, labels).

### Add text

#### Reference

- The reference's text tool needs no dialog: a click on the canvas starts
  typing there, Enter starts a new line and Esc finishes; by default the
  click point becomes the text's centre [R26].
- Text can later be edited with the text tool or by a double-click; a
  setting makes the double-click open a text edit dialog instead, and the
  right-click menu offers one too; that dialog's look is not shown [R26].
- The font list comes from the computer's installed fonts plus single-line
  fonts from a folder; a font manager opens from the font box's right-click
  menu [R26].

#### LaserCraft

- Title "Add text" (`docs/spec/text.md`: a click with the Text tool opens a
  small window at that point). Kind: tool window. Opened by a click on the
  canvas while the Text tool (Ctrl+T) is active; placed beside the click
  point (Conventions).
- Contents:

| # | Label | Control | Default | Range | Tooltip |
|---|---|---|---|---|---|
| 1 | Text | box, three text lines tall, scrolls when longer | empty | — | "The words to add. Enter starts a new line." |
| 2 | Font | dropdown of installed families with typing to filter (the font field rules of `property-bar.md`); later a "Font browser…" button at its right | the first installed of the families `text.md` lists, then the last used in this run | — | "Font family, from the fonts installed on this computer." |
| 3 | Capital height (mm) | number field | 10.00 (`text.md`), then the last used | 0.1–1000 (`text.md`) | "Height of capital letters. 0.1–1000." |

- Buttons: "OK", "Cancel". Enter in the Text box starts a new line;
  Ctrl+Enter presses OK.
- OK is disabled while the text has nothing visible to draw, with the
  tooltip line "Type some text first." (the command would fail with "nothing
  to draw", `text.md`).
- OK runs `shape.add_text ‹x› ‹y› ‹height› ‹font› ‹text› @‹current color›`
  (`text.md`). A font that is not installed → "Cannot add text: the font
  ‹name› is not installed. Choose another font." in the error band.
- Canvas: the click point's small cross in `drawing` until the window
  closes (`canvas.md`).
- When the Text style toolbar exists (`property-bar.md`), its font and
  capital height are this window's defaults.

Tier: **Own** (a window instead of typing on the canvas, `text.md`'s
decision), **Match** (Ctrl+T, click to place).

### Project notes

#### Reference

- Opened from the File menu or Ctrl+Alt+N; free text kept in the project
  file [R19].
- A small window, wider than tall: a "notes" caption, a large text box that
  has focus on open, and a bottom row with a switch to show the notes when
  the file opens at the left and OK and Cancel at the right. OK saves,
  Cancel throws the edit away; with the switch on the notes appear when
  the project is opened [R19].

#### LaserCraft

- Title "Project notes". Kind: settings window, resizable. Opened by File
  → Project notes… (Ctrl+Alt+N, `ui.notes`).
- Contents:

| # | Part | Control | Notes |
|---|---|---|---|
| 1 | (no label) | box, about ten text lines tall (today's height, `shell.rs:307`), full content width, grows when the window is resized | focused on open with the caret at the end of the text |
| 2 | left of the button row, later | switch "Show when the project opens" | stored in the project (proposed change to project-file.md); off by default |

- Buttons: right group "Save notes" (primary; today's label), "Cancel"
  (new; today only the close button discards).
- Enter starts a new line; Ctrl+Enter presses Save notes; Esc presses
  Cancel.
- Save notes runs `project.set_notes ‹text›` (one undo step) only when the
  text changed; otherwise it just closes.
- With "Show when the project opens" on, opening the project shows this
  window after the load, with focus on Cancel so a key press does not edit
  the notes.

Tier: **Match** (box, switch at the left, confirm and cancel at the right),
**Own** (labels, behaviour details).

### Unsaved changes

#### Reference

- The prompt offers three choices: save first, do not save (dropping the
  changes), or cancel the action [R20]. Its icon, button labels, button
  order and default button are unknown.

#### LaserCraft

- Title "Unsaved changes" (today's title). Kind: message, warning icon.
  Raised by New, Open…, Quit, closing the main window, and typing
  `project.new` or `project.load` in the command line while the project is
  modified (`ui-shell.md`).
- Text: "‹project name› has changes that are not saved." then one of "Save
  them before starting a new project?", "Save them before opening another
  project?", "Save them before quitting?".
- Buttons, right group: "Save" (primary; owner decision, `naming.md`),
  "Discard" (destructive), "Cancel". Without Save (today) the row is
  "Discard", "Cancel", and focus and Enter go to Cancel.
- Save: saves to the known path, or opens the Save project as prompt when
  there is none; after a successful save the action that raised the
  message continues; if the save fails or the prompt is cancelled, nothing
  else happens and the project stays open and modified.
- Discard: drops the changes and continues the action. Cancel: does
  nothing.
- While connected the message uses the Message backdrop, not
  `egui::Modal` (today's code, `crates/lasercraft-app/src/main.rs:190`).
- Quitting while a job runs is a separate question (Open questions).

Tier: **Match** (three choices: save, do not save, cancel), **Own**
(labels, order, text, icon, flow after Save).

### File prompts: Open project, Save project as, Import vector art, Save G-code

#### Reference

- The reference uses the operating system's file dialogs [R20]; it has no
  typed-path prompt.

#### LaserCraft

- Kind: file prompt. Titles and prompt lines are today's
  (`crates/lasercraft-ui/src/shell.rs:586-589`); a prompt line above a field
  may end in a colon (`naming.md`).

| Title | Opened by | Prompt line | Field prefilled with | On OK |
|---|---|---|---|---|
| Open project | File → Open… (Ctrl+O) | "Path to a .lcproj file:" | the folder last used for projects, ending in a separator | open the project (after Unsaved changes when needed) |
| Save project as | File → Save as… (Ctrl+Shift+S); Save with no known path | "Path to a .lcproj file:" | the current file's path, else "‹project name›.lcproj" in the last project folder | save to that path |
| Import vector art | File → Import… (Ctrl+I) | "Path to an .svg or .dxf file:" | the folder last used for imports | `project.import ‹path›` |
| Save G-code | File → Save G-code… (Alt+Shift+L) | "Path to the .gcode file to write:" | the project's path with the extension ".gcode", else "‹project name›.gcode" in the last G-code folder | `project.export_gcode ‹path›` |

- Layout: the prompt line in `type-body`, the field under it across the
  content width, the error band, the button row. The window is about half
  the main window wide so ordinary paths fit; longer ones scroll in the
  field.
- Buttons: "OK" (primary), "Cancel" (new; today only the close button
  cancels). OK is disabled while the field is empty or blank.
- Enter presses OK (today too, `shell.rs:597`); Esc presses Cancel.
- Errors stay in the prompt: when the app reports that the file cannot be
  read or written, the prompt stays open with "Cannot open ‹path›: ‹OS
  reason›. Check the path and try again." (`naming.md`) in its error band.
  Today the prompt closes first and the error only reaches the command line
  log (proposed change to ui-shell.md, Files).
- Save project as and Save G-code, when the path names an existing file,
  first raise the message "Replace file?": warning icon; text "‹file name›
  already exists in ‹folder›. Replacing it overwrites its contents."; buttons
  "Replace" (destructive), "Cancel" (focus and Enter). Own design; proposed
  change to ui-shell.md.
- The folders last used are remembered for the run (UI state), and across
  runs once settings persist (`layout.md`, stage p).
- Later: the native dialogs of "File prompts now, native file dialogs
  later".

Tier: **Own** (typed path, prefill, overwrite question).

---

## Later dialogs

### Path array

#### Reference

- Opened from the Arrange menu after selecting the object to copy first
  and Shift+clicking the path second; the menu item is greyed when the
  selection does not fit (one vector or one group to copy, one ungrouped
  vector as the path) [R4].
- Small, roughly square. Rows: a choice between a number of copies and a
  spacing between copies (spacing usually leaves some path unused), each
  with its field; switches to turn copies along the path, to scale the
  copies step by step to a final percentage (its field greyed while off), to
  auto-increment variable text and to select the result; OK and Cancel. The
  copies are previewed on the canvas while options change [R4].

#### LaserCraft

- Title "Path array". Kind: tool window. Opened by Arrange → Path array…
  (`menus.md`). Enabled when exactly two objects are selected and the one
  selected last (the anchor, `arrange.md`) is a single path; otherwise
  disabled with the second tooltip line "Select the shapes to copy, then
  Shift+click the path." Later.
- Contents (choice-with-field rows, Conventions):

| # | Label | Control | Default | Range | Tooltip |
|---|---|---|---|---|---|
| 1 | Fixed count | choice dot + whole-number field | chosen; 5 | 2–1000 | "Spread this many copies evenly along the path." |
| 2 | Fixed spacing (mm) | choice dot + number field | 10.00 | 0.01–10 000 | "Place a copy every this many millimetres along the path, centre to centre." |
| 3 | Follow path direction | switch | off | — | "Turn each copy to follow the path." |
| 4 | Last copy size (%) | switch + number field (disabled while off) | off; 100.0 | 1–1000 | "Grow or shrink the copies step by step so the last one has this size." |

- Summary line (dry run): "‹n› copies" with Fixed count, "‹n› copies,
  ‹d› mm of path left over" with Fixed spacing.
- Buttons: "OK", "Cancel". Preview of every copy in `drawing`; the path
  keeps its Selected look.
- OK runs `arrange.path_array` (new; proposed change to arrange.md); the
  copies are selected.
- Not offered: auto-incrementing text (later with Text data, `text.md`
  proposal); a switch to select the result (always selected).

Tier: **Echo** (count or spacing choice, follow and scale switches,
preview), **Own** (labels, anchor rule).

### Combine shapes

#### Reference

- A small floating window with a row of five large square buttons, each a
  big pictogram above a two-line caption: union, intersection, the two
  subtraction orders, and a reset; then a thin rule, two lines of
  instructions, and Cancel and OK with OK as the default [R9].
- Hovering or clicking an operation previews its result on the canvas;
  reset puts the original shapes back [R9].
- Booleans need two closed vector items [R9].

#### LaserCraft

- Title "Combine shapes". Kind: tool window. Opened by Tools → Combine
  shapes… and the Booleans flyout (`toolbars.md`); Ctrl+B once keymap.md
  binds it (later). Enabled when exactly two objects are selected
  (`arrange.md`).
- Contents, top to bottom:

| # | Part | Contents |
|---|---|---|
| 1 | Result buttons | one row of five square buttons, each about `row-big` tall: an `icon-big` pictogram (`icons.md`) above its caption in `type-small`: "Union", "Intersect", "First minus second", "Second minus first", "Reset" |
| 2 | Rule | thin `border` rule |
| 3 | Instructions | two lines in `type-small` `text-muted`: "Point at a result to preview it, click it to choose it." and "First and second follow the order you selected the shapes in." |

- Result button states: Normal (`field` fill, `border` outline); Hover
  (`hover`; the canvas previews that result); Chosen (`accent-soft` fill,
  `accent` outline; the preview stays when the pointer leaves); Disabled
  (`text-disabled`, when the shapes are open: second tooltip line "Both
  shapes must be closed.").
- When the pointer leaves the buttons the preview returns to the chosen
  result, or to none.
- Reset clears the choice and the preview. Buttons: "OK" (disabled until a
  result is chosen; tooltip "Choose a result first."), "Cancel".
- OK runs `path.union`, `path.intersect` or `path.subtract` with the two
  object ids in the order that gives the chosen result ("Second minus
  first" swaps them; `arrange.md` removes the last id given).

Tier: **Echo** (row of large picture buttons with hover preview, reset,
OK and Cancel), **Own** (captions, instructions, states).

### Close nearly closed paths (later)

#### Reference

- The reference's Edit menu has an item, greyed while no open path is
  selected, that opens a dialog for closing open paths whose two ends lie
  within a distance the user chooses [R34].
- The dialog's look, fields, defaults and buttons are not described in the
  notes (unknown).

#### LaserCraft

- Title "Close nearly closed paths" (the menu row without its ellipsis,
  `naming.md` title rule). Kind: tool window, not resizable, small. Opened
  by Edit → Close nearly closed paths… (`menus.md`), enabled when a selected
  path is open. No chord (`menus.md`: none). Later.
- Contents, top to bottom:

| # | Part | Contents |
|---|---|---|
| 1 | Tolerance (mm) | number field, focused on open with its text selected; range 0.01–100; default the value last used in this run, else today's DXF join tolerance (`import.md`), so imported and edited paths close alike; hint "Ends closer than this are joined" |
| 2 | Summary | one line in `type-small` `text-muted`, updated as the value changes: "‹n› of ‹m› open paths will close." or, when none qualify, "No open path has its ends within ‹t› mm." |

- Preview (as Live previews on the canvas): every path that will close is
  drawn with its closing segment in `drawing`; open paths that will stay
  open keep their normal look, so the difference is the new segments only.
- Buttons: right group "OK", "Cancel". OK is disabled while the summary
  says no path will close, with the second tooltip line "No open path is
  close enough to close." Enter presses OK; Esc cancels.
- OK runs one undoable command (new, later: `path.close_within ‹tolerance›`
  over the selected ids; proposed change to arrange.md). Each qualifying
  path gets one straight segment from its last node to its first; nothing
  else moves. Closed paths and paths whose ends are farther apart are left
  as they are and are not reported as errors.
- Errors keep the window open in the error band: "Tolerance must be
  between 0.01 and 100 mm."

Tier: **Echo** (a distance dialog from the Edit menu that closes nearly
closed paths), **Own** (field, range, default, summary, preview, labels).

### Simplify (later)

#### Reference

- The reference's Edit menu has an item, shown with Alt+Shift+O, that
  opens a dialog which smooths the selected shapes or refits them to arcs
  and lines within a tolerance [R34].
- The dialog's look, fields, defaults and buttons are not described in the
  notes (unknown).

#### LaserCraft

- Title "Simplify". Kind: tool window, not resizable, small. Opened by Edit
  → Simplify… (`menus.md`), enabled when something is selected; Alt+Shift+O
  once keymap.md binds it (later; keymap.md calls the action "optimize",
  proposed change in `naming.md`). Later.
- Contents, top to bottom:

| # | Part | Contents |
|---|---|---|
| 1 | Tolerance (mm) | number field, focused on open with its text selected; range 0.001–10; default the value last used in this run, else 0.05; hint "How far the result may stray from the original" |
| 2 | Fit arcs | switch, off; on: runs of nodes that lie on a circle within the tolerance become arc segments. Shown only once `geometry.md` keeps arcs as segments (today arcs become curves on import); until then the row is absent, not greyed (Open questions) |
| 3 | Summary | two read-outs side by side in `type-small` `text-muted`, updated with the preview: "Nodes before: ‹n›" and "Nodes after: ‹n›" |

- Preview: the simplified result in `drawing` over the originals, which
  are drawn dashed as in Offset's preview (Echo of the reference's offset
  preview, [R3]), so the change in shape is visible at a glance. When the
  result has as many nodes as the original, the summary adds "Nothing to
  simplify at this tolerance." and OK is disabled with that text as its
  second tooltip line.
- Buttons: right group "OK", "Cancel". Enter presses OK; Esc cancels.
- OK runs one undoable command (new, later: `path.simplify ‹tolerance›
  ‹arcs›` over the selected ids; proposed change to geometry.md and
  arrange.md). Text, images and barcodes in the selection are skipped and
  listed in the Messages field ("Skipped 2 objects that are not paths.").
- Errors keep the window open in the error band: "Tolerance must be
  between 0.001 and 10 mm."

Tier: **Echo** (a tolerance dialog from the Edit menu that reduces nodes,
with an optional fit to arcs), **Own** (fields, ranges, defaults, node
read-outs, preview, labels).

### Measurements

#### Reference

- A small floating window: two columns of label and value pairs, labels
  right-aligned, values left-aligned. The left column counts nodes, straight
  and curved segments and gives the perimeter, width, height, whether the
  path is closed, and below them the area; the right column gives the
  hovered segment's length, start and end points, their difference, the
  angle, and the arc radius and centre (empty when the segment is not a
  true arc) [R10].
- The bottom row holds the area value, a wide button showing the unit that
  switches units, and OK [R10].
- Hovering highlights the shape and the segment on the canvas; dragging
  measures between two points [R10].

#### LaserCraft

- Title "Measurements". Kind: info window. Shown when the Measure tool
  becomes active (Tools → Measure; Alt+M once bound, later) and hidden when
  another tool is chosen; Close hides it until the tool is chosen again.
- Contents: two groups side by side.

| Group | Label | Value |
|---|---|---|
| Shape (heading) | Nodes | count |
| | Straight segments | count |
| | Curved segments | count |
| | Kind | "Closed path", "Open path" or "‹n› paths" |
| | Width (mm) | bounds width |
| | Height (mm) | bounds height |
| | Perimeter (mm) | total length of all outlines |
| | Area (mm²) | even-odd area; "—" for open paths |
| Segment (heading) | Length (mm) | length of the hovered or measured segment |
| | From (mm) | "x, y" |
| | To (mm) | "x, y" |
| | Change (mm) | "Δx, Δy" |
| | Angle (°) | direction from From to To, clockwise from the +x axis (screen words, `geometry.md`) |
| | Circle radius (mm) | radius when the segment is a circular arc; "—" otherwise |
| | Circle centre (mm) | "x, y"; "—" otherwise |

- Values in `type-mono` `text`, right-aligned in their column, decimals per
  `naming.md` (two for lengths and angles); "—" in `text-muted` when a value
  does not apply.
- Empty state: "Point at a shape to measure it." in `text-muted`.
- Buttons: left group "Copy values" (puts every label and value on the
  clipboard as text lines), right group "Close".
- Units follow the design units (millimetres now; the "mm / in" toggle of
  the Position and size toolbar later); there is no unit button here (own;
  the reference has one).
- The numbers come from the app (geometry crate), never computed in the UI.
  The highlight colors on the canvas are `canvas.md`'s.

Tier: **Echo** (two columns of measures, shape and segment), **Own**
(labels, Copy values, Close instead of OK, no unit button).

### Select circles

#### Reference

- A small dialog: a diameter field, a maximum diameter field that is greyed
  unless the range choice is made, a choice between an exact size and a
  range, and OK and Cancel [R11].

#### LaserCraft

- Title "Select circles". Kind: tool window (it changes the selection).
  Opened by Edit → Select circles…, enabled when the project has shapes.
  Later.
- Contents:

| # | Label | Control | Default | Range | Tooltip |
|---|---|---|---|---|---|
| 1 | Diameter (mm) | number field | 10.00, then the last used | 0.01–10 000 | "Diameter to look for, or the smallest of the range." |
| 2 | (no label) | choice, side by side: "One size", "Size range" | One size | — | "Select circles of exactly this size, or of every size in a range." |
| 3 | Up to (mm) | number field; hint "Largest diameter to select"; disabled unless Size range | the diameter | 0.01–10 000 | "Largest diameter to select." |

- Form rule: Up to below Diameter → "Cannot select circles: Up to (mm) is
  smaller than Diameter (mm)."
- Buttons: "OK", "Cancel". OK runs `select.circles ‹d› [‹max›]` (new
  command; proposed change to arrange.md or a selection spec): the
  selection becomes every matching circle; "One size" matches within the
  planner's flattening tolerance (0.01 mm, `planning.md`; own choice).
  The status bar's Messages field reports "Selected ‹n› circles."
- None found: the window stays open with "Cannot select circles: no circle
  has that size." in the error band.
- What counts as a circle (ellipses with equal axes, paths whose outline is
  a circle) and how grouped circles are treated (selections widen to whole
  groups, `arrange.md`) are Open questions.

Tier: **Match** (diameter, exact or range, maximum greyed), **Own** (labels,
tolerance, feedback).

### History

#### Reference

- A narrow, tall window: one line of instructions, then a scrolling list
  with one row per recorded action, each row a small round arrow icon and
  the action's name; a row for the current state carries a play-triangle
  marker, rows before it undo icons and rows after it redo icons; the
  selected row is a blue band; OK and Cancel [R12].
- Choosing a row previews the project in that earlier state; OK keeps it,
  undoing everything after it [R12].

#### LaserCraft

- Title "History". Kind: tool window, resizable. Opened by Edit → History…
  (`menus.md`), enabled when at least one action is in the history. Later.
- Contents:

| # | Part | Contents |
|---|---|---|
| 1 | Instructions | `type-small` `text-muted`: "Click an action to see the project as it was right after it. OK keeps that point; Cancel returns to where you were." |
| 2 | List | `field` background with `border`, scrolling, oldest action at the top. First row "Start" (the project as opened or created). Then one row per action: an `icon-small` back-arrow for actions that are done, a forward-arrow in `text-muted` for undone ones (the redo stack), then the action's readable name (`naming.md`: the same words as "Undo ‹action›", e.g. "Move", "Weld", "Grid array"). The row of the current point is drawn in `text` with a small `accent` triangle at its left; rows after it in `text-muted`. The chosen row is `accent-soft`. |

- Choosing a row (click, or Up and Down keys) asks the app to undo or redo
  to that point; the canvas shows the real project there. These steps are
  the existing `edit.undo` and `edit.redo` commands, run by the app, not
  new history entries (`commands.md`).
- Buttons: "OK" (keeps the chosen point; the actions after it stay on the
  redo stack until a new command clears them, `commands.md`), "Cancel"
  (returns to the point that was current when the window opened).
- Empty state: "No actions yet." (`naming.md`). Opens scrolled so the
  current row is visible.

Tier: **Echo** (list of actions with a current marker, preview by
choosing, OK and Cancel), **Own** (labels, Start row, icons, tokens).

### Cut order

#### Reference

- A tall dialog opened from a button in the laser panel [R13].
- At the top a master switch for optimizing and reordering output, and a
  rule; then a caption saying the ordering rules are optional and their
  order can change [R13].
- An ordering group: three buttons on the left that add a rule (by layer,
  by group, by each shape's priority number) or move it to the top when it
  is already there, a list in the middle showing the active rules in order
  (by default layer then priority), and a wide remove button on the right.
  Rules nest: the first splits the shapes into sets, the next splits each
  set [R13].
- A heading for further options and a large group: a column of switches
  (cut inner shapes first, cut in one sweep direction with four direction
  choices beside the first rows, reduce travel, hide backlash, reduce
  direction changes, choose the best start point, prefer corners, choose
  the best direction, remove overlapping lines with a distance field on the
  same row) [R13].
- Footer: set as defaults and reset to defaults at the left (Shift+click
  restores the factory values), OK and Cancel at the right; the values are
  stored in the project [R13].

#### LaserCraft

- Title "Cut order". Kind: settings window, two columns. Opened by the
  Machine panel's "Cut order…" button (`machine-panels.md`). Later;
  `docs/spec/planning.md` fixes the order today, so the window needs a
  proposed change to planning.md first.
- Contents, top to bottom:

| # | Part | Contents |
|---|---|---|
| 1 | Master switch | switch "Reorder cuts", full width; tooltip "Let the planner choose the order shapes are cut in. Off cuts them in drawing order." Off disables everything below (values kept) |
| 2 | Rule | thin `border` rule |
| 3 | Heading "Rules" | then a caption in `type-small` `text-muted`: "Each rule sorts the shapes inside the groups made by the rule above it." |
| 4 | Rule builder | left: three stacked Normal buttons "Add layer rule", "Add group rule", "Add run-order rule"; middle: a `field` list as tall as the three buttons, one rule per row; right, top-aligned: "Remove rule" (disabled with no row chosen) |
| 5 | Heading "Options" | |
| 6 | Options | left column of switches; right column, beside the first switch rows, the sweep choices |

Rule rows (the list's wording, proposed to `naming.md`):

| Rule | Row text | Effect |
|---|---|---|
| Layer | "By layer order" | layers in output order (`layers.md`), as today |
| Group | "Groups kept together" | every group's shapes are cut together; ungrouped shapes count as one group each (Echo [R13]) |
| Run order | "By run order (lowest first)" | each shape's "Run order" property (Properties panel, later) |

- Adding a rule that is already listed moves it to the top (Match [R13]).
  Default list: "By layer order" (today's behaviour; the run-order rule
  needs the Properties panel first).

Options, in order (labels from `naming.md`):

| # | Label | Control | Default | Tooltip |
|---|---|---|---|---|
| 1 | Inner shapes first | switch | on (today, `planning.md`) | "Cut shapes that lie inside another before the outer one." |
| 2 | Shortest travel | switch | on (today: nearest next shape) | "Pick the nearest shape next to cut down travel." |
| 3 | Sweep in one direction | switch, with the choices From top, From bottom, From left, From right in the right column (disabled while off; From top chosen) | off | "Work across the job in one direction." |
| 4 | Backlash-friendly order | switch | off | "Order cuts so that slack in the drive does not show at start and end points." |
| 5 | Keep head direction | switch | off | "Prefer next cuts that keep the head moving the same way." |
| 6 | Start anywhere on closed shapes | switch | on (today) | "A closed shape may start at any of its nodes." |
| 7 | Prefer starting at corners | switch (disabled unless 6 is on) | off | "Start closed shapes at sharp corners where possible." |
| 8 | Allow reversing paths | switch | on (today: open paths may run backwards) | "The planner may cut a path in the other direction." |
| 9 | Skip duplicate lines | switch, with "Within (mm)" field on the same row (disabled while off) | off; 0.01 | "Cut lines that lie on top of each other only once." |

- Buttons: left group "Save as default" (stores these values in the
  preferences for new projects), "Restore default" (loads them into the
  draft; Shift+click loads LaserCraft's own defaults, said in its tooltip's
  second line); right group "OK", "Cancel".
- OK runs `project.set_cut_order ‹settings›` (new; one undo step; stored
  in the project, proposed change to project-file.md and planning.md).
- Which switches another switch forces (the reference says the backlash
  option forces some) is unknown; when LaserCraft defines it, a forced
  switch shows Disabled with the second tooltip line "Set by Backlash-
  friendly order." Open question.

Tier: **Echo** (master switch, rule builder with its list and remove
button, option switches with sweep choices, defaults buttons at the left),
**Own** (labels, defaults, layout details).

### Preferences

#### Reference

- The settings dialog opens from the Edit menu (the application menu on
  macOS) or a gears button in the main toolbar; it applies to the whole
  program, whatever machine is chosen [R14].
- A fixed-size window with five tabs whose captions carry large bold
  pictograms; each tab holds plain captions above thin framed panels, laid
  out in one or two columns, sometimes side by side; OK and Cancel at the
  bottom right, no Apply [R14].
- Tab contents: editor behaviour switches, auto-save interval, start-button
  guards, job log, file-dialog choice, and text defaults; units, snapping,
  grid look and spacing, click and snap distances and nudge steps; display
  switches, cursor and node size, curve quality, icon and font size sliders
  and the application theme; import options for DXF, SVG and plotter files,
  output curve tolerance and a print line width; camera capture options
  [R14].

#### LaserCraft

- Title "Preferences". Kind: settings window, not resizable, one size on
  every tab. Opened by Edit → Preferences… on every platform (`menus.md`)
  and the Standard toolbar's preferences button (`toolbars.md`). Later.
- Tab strip across the top of the content, left-aligned, tabs as wide as
  their captions: an `icon-small` pictogram then the caption in
  `type-body`. Selected tab: `field` background joined to the page below
  and an `accent` marker line along its top edge (decision 3); others
  `panel` with `text`. Tabs, in order: "General", "Grid and snapping",
  "Display", "Import and export", "Camera" (M9; absent before). Tab
  icons (`icons.md`): `prefs-general` (an outlined switch), `prefs-grid`
  (a grid with a dot on its centre crossing), `prefs-display` (a split
  window), `prefs-import-export` (a sheet with an arrow in and an arrow
  out), `camera`.
- Buttons: right group "Apply", "OK", "Cancel". Apply and OK run the front-
  end action `ui.set_preferences ‹settings›` (new; preferences are not
  project data and not undoable); Cancel discards changes not yet applied.
  Every setting takes effect at once, without a restart (egui can change
  themes, scale and fonts at run time).
- Storage: LaserCraft's settings file (owner decision with layout
  persistence, `layout.md`).

General tab:

| Group (heading) | Label | Control | Default | Notes |
|---|---|---|---|---|
| Saving | Auto-save every (min) | whole-number field; hint "0 turns it off" | 0 | own default until auto-save is specified (proposed change to project-file.md) |
| Saving | Keep a job log | switch | off | one line per job (date, device profile, time taken) in a CSV file in LaserCraft's data folder |
| Starting jobs | Block Start while the screen is off | switch | on | owner decision whether this is fixed behaviour (`naming.md`) |
| Starting jobs | Start button skips keyboard focus | switch | on | as above; the Start button never keeps focus, so Enter or Space typed later cannot start a job |
| Text | Default font | dropdown with typing to filter | `text.md`'s default family | |
| Text | Default capital height (mm) | number field | 10.00 (`text.md`) | 0.1–1000 |
| Text | Single-line font folder (SHX)… | Normal button, then the chosen folder path in `type-small` `text-muted`, then a small "Clear folder" button | none | later, with single-line fonts |
| Text | Right-to-left text | switch | off | later; text.md has no right-to-left shaping today |
| Text | Double-click edits text | switch | off | later; proposed change to text.md (editable text) |
| Layers | New projects use saved layer defaults | switch | off | M8, with the per-device layer defaults (`layers-panel.md`, Buttons): on, File → New project and the first start load every color's saved default for the current device profile; off, they keep the layer settings last used, as today. Echo [R14] |
| Layout | Simple layout | switch | off | later, optional (decision 11) |

Grid and snapping tab:

| Group (heading) | Label | Control | Default | Notes |
|---|---|---|---|---|
| Grid | Grid spacing (mm) | number field; hint "0 means automatic" | 0 (automatic) | 0 keeps `canvas.md`'s adaptive levels (minor step from the 1, 2, 5 sequence, major always ten minor steps); a value above 0 fixes the minor step at that spacing and major lines still fall every ten minor steps, so the rule of `canvas.md` holds either way; needs the proposed change to canvas.md |
| Grid | Grid strength | dropdown: Off, Faint, Medium, Strong | Faint | how dark the grid lines are drawn (`grid-minor`, `grid-major`) |
| Snapping | Snap to grid | switch | on | |
| Snapping | Snap to shapes | switch | on | Ctrl while dragging still turns snapping off (keymap.md) |
| Snapping | Snap spacing (mm) | number field | 10.00 (today's grid snap, `canvas.md`) | independent of Grid spacing |
| Snapping | Snap distance (px) | whole-number field | today's snap radius (`canvas.md`) | a screen distance, independent of zoom; the code measures it in egui logical points, which scale with the display (the label says px as `canvas.md` and `naming.md` do) |
| Snapping | Click tolerance (px) | whole-number field | today's click tolerance (`canvas.md`) | as Snap distance (px) |
| Nudge distances | Small (mm) | number field | 0.10 (keymap.md, Ctrl+arrows) | |
| Nudge distances | Normal (mm) | number field | 1.00 (arrows) | |
| Nudge distances | Large (mm) | number field | 10.00 (Shift+arrows) | the three fields sit on one row, Echo [R14] |

Display tab:

| Group (heading) | Label | Control | Default | Notes |
|---|---|---|---|---|
| Interface | Theme | dropdown: Light, Dark, Follow system | Light (decision 13) | the bed stays light in every theme |
| Interface | Text size | dropdown: Smaller, Normal, Larger, Largest | Normal | scales `type-*` together |
| Interface | Icon size | dropdown: Smaller, Normal, Larger | Normal | scales `icon`, `icon-small`, `icon-big` together |
| Canvas | Handle size | dropdown: Normal, Larger, Largest | Normal | handles and nodes on the canvas (Echo [R14]) |
| Canvas | Show swatch numbers | switch | on | the two-digit labels on palette swatches (decision 5) |
| Canvas | Reduce motion | switch | off | stops the moving selection dashes and slide-in animations (`naming.md`) |

- Show fills and Smooth curves stay View menu items (`menus.md`), not
  preferences.
- If Text size or Icon size makes the window unusable, Cancel within the
  same window restores the previous values; a start-up reset is an open
  question in `layout.md` (question 8).

Import and export tab:

| Group (heading) | Label | Control | Default | Notes |
|---|---|---|---|---|
| DXF | DXF units | dropdown: Detect, Millimetres, Centimetres, Metres, Inches, Feet | Detect (today: from the file's units header, `import.md`) | proposed change to import.md |
| DXF | DXF close tolerance (mm) | number field | today's join tolerance (`import.md`) | |
| SVG | SVG resolution | dropdown: 96, 72, Detect (SVG user units per inch) | 96 (`import.md`, the CSS pixel) | options name numbers, not programs (`naming.md`) |
| Output | Output curve tolerance (mm) | number field | 0.01 (`planning.md`) | how closely output follows curves |

Camera tab (M9): its contents are set by the camera spec of M9
(`later-windows.md`); the reference's tab holds a capture-system choice,
the default overlay color mode, a preview quality slider and frame-rate
limits [R14]. Open question.

Not offered (see Deliberately omitted for the reasons): the update check,
the system-clipboard switch, opening files in a running copy, the error
sound for failed sends, ignoring shapes outside the bed, the file-dialog
choice, antialiasing, high-resolution scaling switches, curve quality, the
dark canvas, rotary and cylinder switches, the plotter-file and print
settings. The reference's other import and export switches [R14] are
not offered either, each for its own reason:

- Importing hidden layers from one vendor's drawing format: LaserCraft
  imports SVG and DXF only (`import.md`); there is no such format to read.
- Importing onto the reference's tool layers: guide layers are an owner
  decision (`naming.md`, `layers-panel.md` guide swatches); if adopted, the
  switch is decided with them.
- Ignoring DXF text: `import.md` decides what DXF entities are read; a
  switch is a proposed change there if text import is ever added.
- DXF export as arcs or as lines: LaserCraft has no DXF export
  (`import.md` is import only); decided with any future export spec.
- HPGL and plotter-file precision and print line width: no plotter import
  and no printing.

Open questions: inverted wheel zoom, returning to Select after
drawing, a full-window cursor crosshair, a bed-centre cross, and the
reference's import switches (group, select and centre imported shapes).

Tier: **Match** (one fixed-size tabbed window, sections in columns, the
content areas), **Echo** (tab pictograms, settings groups), **Own** (labels,
defaults, Apply, no restart, no frames).

### Keyboard shortcuts

#### Reference

- The hotkey editor opens from the preferences submenu; a medium window in
  two columns [R15].
- Left: a list of command categories mirroring the menus and toolbars, and
  a full-width reset-all button under it. Right: a search field (focused on
  open), a two-column table of commands and shortcuts with a sortable
  header and alternating rows, a line of instructions, a shortcut field that
  captures key presses with a clear button, export and import buttons, and
  OK and Cancel [R15].

#### LaserCraft

- Title "Keyboard shortcuts". Kind: settings window, resizable, a bit wider
  than tall. Opened by Edit → Keyboard shortcuts… (`menus.md`). Later
  (keymap.md: a user keymap replaces the default).
- Contents:

| # | Column | Part | Contents |
|---|---|---|---|
| 1 | left (about a third) | Category list | `field` list: "All commands", then one row per menu (File, Edit, Tools, Arrange, Machine, View, Window, Help), then "Canvas keys" (nudge, rotate and drawing-order keys) and "Machine keys" (jog keys); the chosen row `accent-soft` |
| 2 | left | Reset all | Normal button under the list, as wide as the list; asks nothing but only changes the draft |
| 3 | right | Search | field with the hint "Search commands"; focused on open; filters by command name and by chord text |
| 4 | right | Table | columns "Command" and "Shortcut" (`type-body`; header in `text-muted` on `field-alt`; click a header to sort); rows alternate `field` and `field-alt`; the chosen row `accent-soft`; chords drawn with key symbols (`naming.md`); a command with no chord shows "—" in `text-muted` |
| 5 | right | Instructions | `type-small` `text-muted`: "Choose a command, click the field below and press the keys." |
| 6 | right | Shortcut row | label "Shortcut", a field with the hint "Press keys" (disabled until a command is chosen), then "Clear shortcut" |

- Buttons: left group "Import keymap…", "Export keymap…"; right group "OK",
  "Cancel".
- Capture: clicking the Shortcut field starts capture (`accent` focus
  ring); the next chord (modifiers plus one key) is captured and shown; Esc
  ends capture without change. Modifier keys alone are not a chord.
- Conflicts: a chord already used shows the warning "Warning: Ctrl+R is
  already used by Rectangle tool. OK moves it to ‹command›." The keymap
  never ends with one chord bound twice (keymap.md makes that an error).
- The Pause and Stop chords are fixed (safety.md rule 7): their rows are
  shown in `text-disabled` with the tooltip "Fixed: always pauses or stops
  the job." Owner decision (Open questions).
- OK writes the user keymap file and applies it at once; Cancel drops the
  draft. Export keymap… writes the draft to a chosen file; Import keymap…
  reads one into the draft and lists problems in the error band by line
  ("Line 12: Ctrl+R is bound twice."), keymap.md's format.
- Empty search: "No command matches "‹text›"." (`naming.md`).

Tier: **Match** (categories, search, command and shortcut table, capture
field with clear, reset, export and import), **Own** (labels, fixed safety
chords, conflict handling).

### Restore preferences

#### Reference

- Reached from a preferences submenu with import, export, open-folder and
  backup items [R16].
- A window with two side-by-side panes and a divider: a list of automatic
  backups, each a date and time with its file size, the chosen one a solid
  band; and details of the chosen backup: when it was made and a bold
  heading over a bulleted list of the device profiles it holds; Cancel and
  a load button, load being the default [R16].

#### LaserCraft

- Title "Restore preferences". Kind: settings window, resizable. Opened by
  Edit → Restore preferences…, enabled when an automatic backup exists
  (`menus.md`). Later.
- Contents: two panes side by side with a draggable divider (a thin
  `border` line that shows `hover` under the pointer).

| Pane | Contents |
|---|---|
| Left | `field` list, newest first, rows "‹date› ‹time› (‹size›)"; the chosen row `accent-soft` |
| Right | "Saved on ‹date› at ‹time›" in `type-body`; heading "Device profiles" (`type-heading`) with the profile names under it (M8), each "‹name› · ‹w› × ‹h› mm"; then "Keyboard shortcuts: changed" or "Keyboard shortcuts: default" |

- Buttons: "Restore backup" (primary; disabled until a row is chosen),
  "Cancel".
- Restore backup first saves the current preferences as a new backup, so a
  restore can itself be undone by restoring that one (own design); then it
  replaces the preferences, device profiles (M8) and keymap with the chosen
  backup, and the Messages field says "Preferences restored from the backup
  of ‹date›."
- When backups are made and how many are kept are open questions.

Tier: **Echo** (list of dated backups with sizes, details pane with the
device list), **Own** (labels, safety backup before restoring).

### Quick reference and About (M10)

#### Reference

- One window with three text tabs: a short list of common mouse and
  keyboard shortcuts, getting-started tips, and version information; a
  large white text area with a bold underlined heading over short
  paragraphs; a single OK [R17].
- It opens from the Help menu's about item and its quick-help item, which
  shows F1 beside it [R17][R18]; pressing F1 over a control instead opens
  that control's online page, and how the two uses of F1 are told apart is
  not stated [R33][R17].

#### LaserCraft

- Title "Quick reference". Kind: info window, resizable, wider than tall.
  Opened by Help → Quick reference… (Shortcuts tab) and Help → About
  LaserCraft… (About tab) (`menus.md`). No chord: F1 is context help for the
  item under the pointer (keymap.md, M10). M10; the About tab may come
  earlier (`menus.md` question 12).
- Tabs (text only, same tab look as Preferences): "Shortcuts", "Tips",
  "About".
- Shortcuts tab: built from the active keymap at run time, so it always
  matches the chords in use. Sections with `type-heading` headings named as
  keymap.md's tables ("Files and editing", "Drawing, view and arrangement",
  "Machine", "Mouse and modifiers"); rows "‹chord›" in `type-mono` and the
  action in `type-body`. A left button "Keyboard shortcuts…" opens the
  editor.
- Tips tab: short tips, each a `type-heading` line and one or two sentences
  in `type-body` (our own text; examples: set up a device profile first;
  every button has a tooltip; menus show chords; F1 opens help for the item
  under the pointer; Window → Reset layout restores the panels; Esc
  returns to Select).
- About tab: "LaserCraft" in `type-heading`, version and build in
  `type-body`; the licence line "LaserCraft is free software under the GNU
  General Public License, version 3 or later, and comes with no warranty.";
  the main libraries used (from ARCHITECTURE.md) in `type-small`; buttons
  "License…", "Copy system info", "Report a problem" (`menus.md` actions).
  Above the name, the app icon of `icons.md` (App icon) at `icon-big`,
  once the owner approves it (M10); until then nothing stands there.
- Buttons: "Close".

Tier: **Match** (one window with shortcut, tips and about tabs), **Own**
(labels, generated shortcuts, no chord, content).

### Low-power frame (later, optional, owner decision)

#### Reference

- A modifier-click on a frame button traces the frame with the laser
  firing; the docs mark it as dangerous and diode-only [R23].
- A device setting turns the laser on during every frame at a percentage
  set beside it, greyed unless a fire button is enabled [R22]; framing power
  and speed come from the move panel's fields [R23].
- With continuous framing on, a dialog stays open while framing repeats and
  its OK ends it [R25].

#### LaserCraft

Safety first (`docs/spec/safety.md` rule 2, decision 7): framing is
laser-off by default; this window is the only way to frame with the laser
on, it is opt-in on every use, and it always shows the power. There is no
modifier-click, no setting that makes every frame fire, and no chord.

- Title "Low-power frame". Kind: message (backdrop; the job bar stays
  free). Opened only by Machine → Frame at low power… (`menus.md`),
  enabled when connected, Ready, and the job has output. Disabled while a
  job or framing runs. Later, optional, only if the owner approves.
- Contents, top to bottom:

| # | Part | Contents |
|---|---|---|
| 1 | Warning band | `warning-soft`, `warning` triangle mark, `type-body` `text`: "The laser will be on while the head traces the frame. Wear eye protection rated for your laser and keep the enclosure closed." |
| 2 | Trace | choice, side by side: "Bounding box" (the Frame path), "Outline" (the Frame outline path, the tight convex outline); default the one last used in this run, else Bounding box |
| 3 | Frame power (%) | number field, **empty every time the window opens**; hint "Up to ‹cap›%"; range 0.1 to the cap; the cap is the lower of the device's Max power (%) and a framing ceiling the owner sets (Open questions) |
| 4 | Speed | read-only line in `type-body` `text-muted`: "Speed: ‹v› mm/s (Jog speed)" |

- Buttons: right group "Frame at ‹p›%" (Laser-on style; its label shows the
  typed power; disabled while the field is empty), "Cancel" (focus).
  Enter never presses the frame button.
- Before the button runs, the frame path is checked against the work area
  like a job (safety rule 5); a refusal shows in the error band.
- The frame runs once, whatever Repeat framing says (`machine-panels.md`),
  with dynamic power (`M4`), so a Pause turns the beam off (safety.md rule
  9). The power is clamped (rule 6).
- While it runs: the job bar's state reads "Framing" followed by "laser on
  ‹p›%" in `text` with a `mark-warning` before it (no `warning` text,
  `visual-language.md`); the head marker uses `danger` (`head-marker` in
  `visual-language.md`); the path is drawn in `frame-preview` (`canvas.md`); Stop
  and Pause work as always.
- The typed power is not remembered after the window closes.
- New command `machine.frame_lit ‹box|outline› ‹power›` (proposed change to
  safety.md and the M6 device spec, owner decision).

Tier: **Own**.

### Before you start (M8)

#### Reference

- A device setting turns on a job checklist, with an edit button beside the
  switch; starting a job then shows the checklist to confirm first; Start
  can also show a warning about risky settings first [R22][R23][R24].
- The checklist dialog's look is not shown (unknown).

#### LaserCraft

- Title "Before you start". Kind: message (task, no icon). Raised by Start
  (button, menu, Alt+S once bound) while the device profile's "Confirm
  before start" is on (`machine-panels.md`, Options tab). M8.
- Contents: a line "Check each item before the job starts." in `type-body`;
  then one switch per checklist item, the item's text as its label, all
  off when the window opens; a long list scrolls.
- Buttons: "Start job" (primary, but never pressed by Enter; disabled until
  every item is on, tooltip second line "Switch on every item first."),
  "Cancel" (focus).
- Nothing moves and the laser stays off until Start job (safety rule 1);
  then the normal Start path continues (bounds check, No selection, and so
  on).
- The list is edited by "Edit list…" beside "Confirm before start" in the
  device profile: a settings window titled "Edit list" (from the button,
  `naming.md` title rule; Open question) with one box, one item per line,
  about eight lines tall, resizable; buttons "OK", "Cancel"; stored in the
  device profile.

Tier: **Echo** (checklist confirmed before Start), **Own** (switches, the
enabling rule, labels).

### No selection (M6)

#### Reference

- With the cut-selected-graphics switch on and nothing selected, start,
  send, save and preview show a small warning box with one line and two
  buttons: one stops but keeps the switch on, the other stops and turns
  the switch off [R21].

#### LaserCraft

- Title "No selection" (`naming.md`). Kind: message, warning icon. Raised
  by Start, Frame, Frame outline, Save G-code… and Job preview while
  "Selected shapes only" is on and nothing is selected (`machine-panels.md`).
- Text: "Selected shapes only is on, but nothing is selected." then "Select
  the shapes to use, or use all shapes."
- Buttons: "OK" (primary, focus: keeps the switch on, runs nothing; also
  what Esc and the close button do), "Use all shapes" (Normal: turns the
  switch off, runs nothing; the user starts again). The labels are
  `naming.md`'s and `machine-panels.md`'s; "OK" is allowed here because
  the message is a notice, not a confirmation: neither button loses work
  or runs anything (`naming.md` question 19 asks whether the owner prefers
  a verb).

Tier: **Match** (two choices, nothing runs), **Own** (labels, text).

### Nesting failed (later)

#### Reference

- The quick nest reports problems in message boxes titled as unsuccessful:
  a red stop icon, one line and OK; the causes are a container that is not
  axis-aligned, no container chosen, some parts not fitting (OK still nests
  those that fit), and no possible arrangement [R7].

#### LaserCraft

- Title "Nesting failed". Kind: message. Raised by Nest (Slide and nest
  toolbar, `toolbars.md`, which nests the selected parts on the bed). Later.

| Case | Icon | Text | Buttons |
|---|---|---|---|
| Nothing fits | error | "None of the selected parts fit on the bed with the current Gap (mm) and Edge margin (mm). Make the gaps smaller or select fewer parts." | "OK" |
| Some parts fit | warning | "‹n› of ‹m› parts do not fit on the bed." then "Place the parts that fit and leave the others where they are?" | "Place the parts that fit" (primary), "Cancel" |
| No arrangement | error | "No arrangement was found for these parts. Switch off Keep rotation or make the gaps smaller, then try again." | "OK" |

- The container cases do not apply (LaserCraft nests on the bed).
- A nest that succeeds raises no message; it is one undo step.

Tier: **Echo** (one message per failure, partial nesting offered), **Own**
(cases, labels, text).

### Fit slots to thickness (later)

#### Reference

- Opened from the Tools menu on a selection; wider than tall; a "?" button
  in the title strip [R6].
- Three rows of fields with a millimetre unit after each: the old material
  thickness to look for, the new thickness, and a tolerance; a titled group
  with three choices in one row (slot depth, slot width, tab height); a
  message area listing unsupported items; then Apply (keeps the dialog
  open), OK and Cancel; changes already applied stay after Cancel and need
  undo [R6].

#### LaserCraft

- Title "Fit slots to thickness". Kind: tool window. Opened by Tools → Fit
  slots to thickness… (`menus.md`), enabled when something is selected.
  Later.
- Contents:

| # | Label | Control | Default | Range | Tooltip |
|---|---|---|---|---|---|
| 1 | Current thickness (mm) | number field | 3.00, then the last used | 0.1–100 | "Material thickness the slots were drawn for." |
| 2 | New thickness (mm) | number field | 3.00, then the last used | 0.1–100 | "Thickness of the material you will cut." |
| 3 | Tolerance (mm) | number field; hint "Slots this much off still count" | 0.10 | 0–10 | "How far a slot may differ from Current thickness and still be changed." |
| 4 | Change | choice, side by side: "Slot depth", "Slot width", "Tab height" | Slot width | — | "Which edges of the matching slots and tabs move." |

- Notes area under the form (only when needed): `type-small` `text-muted`
  lines for skipped items ("Grouped shapes are skipped. Ungroup them
  first.").
- Summary line (dry run): "‹n› slots found, ‹k› will change."
- Buttons: right group "Apply", "OK", "Cancel". Apply runs the command and
  keeps the window open (one undo step per Apply); Cancel closes without
  undoing applied steps, and its tooltip second line says "Changes already
  applied stay; use Undo." (Match [R6]).
- Preview: the moved edges in `drawing`.
- New command `path.fit_slots` (proposed change to arrange.md).

Tier: **Match** (three values, one choice, Apply, OK, Cancel), **Own**
(labels, defaults, summary).

### Barcode (later)

#### Reference

- Created from the Tools menu by dragging a box on the canvas; the dialog
  opens by itself on release and reopens by double-clicking the code or
  from the right-click menu when one code alone is selected [R5].
- Taller than wide, beside the canvas. At the top a code type dropdown
  defaulting to QR or the type used last; then three tabs for raw content,
  network login and contact details, the last two greyed for other types
  [R5].
- The content tab: a large text box for two-dimensional codes and a single
  line for linear codes, a remaining-characters counter under it (red and
  negative when over the limit), an error line naming bad characters, a
  read-only box showing the evaluated text, a text-mode dropdown and a
  variable offset field, switches shown per type (control characters,
  printed digits, forcing a square), and sliders for error correction and
  extra spacing [R5].
- The network tab holds the network name, password, a hidden-network switch
  and the authentication type; the contact tab seven name and address
  fields; filling either writes the raw content, the last tab edited wins
  [R5].
- OK finishes; Cancel deletes a code just drawn. The code redraws on the
  canvas while typing; invalid data shows a large cross [R5].

#### LaserCraft

- Title "Barcode". Kind: tool window, one size on every tab. Opened by
  Tools → Barcode… (then drag a box on the canvas; the window opens on
  release), by a double-click on a barcode, and by "Edit barcode…" in the
  canvas menu (`menus.md`). Later; proposed change to shapes.md (a barcode
  shape kind).
- Contents, top to bottom:

| # | Part | Contents |
|---|---|---|
| 1 | Code type | label "Code type", dropdown of the symbologies the chosen library supports, by their standard names (for example QR code, Data Matrix, Code 128, EAN-13); default QR code, then the last used |
| 2 | Tab strip | "Text", "Wi-Fi", "Contact" (`naming.md`); Wi-Fi and Contact are disabled unless the type is QR code, tooltip "Only for QR codes." Tab look as Preferences, without icons |
| 3 | Tab page | below |

Text tab:

| # | Label | Control | Notes |
|---|---|---|---|
| 1 | (no label) | box about five lines tall for two-dimensional codes; a single-line field for linear codes | the encoded text; when filled from another tab, a `text-muted` line above it says "Built from the Wi-Fi tab" or "Built from the Contact tab" |
| 2 | (no label) | counter, right-aligned under the box, `type-small` `text-muted`: "‹used› of ‹max› characters"; over the limit `danger`: "‹n› characters too many" | |
| 3 | Text source | dropdown: Plain, Date and time, From CSV, Counter, Layer speed and power | later, with Text data (`text.md` proposal); Plain until then |
| 4 | Counter offset | whole-number field | later, as Text source |
| 5 | Encoded text | read-only box (`field-alt`), shown when Text source is not Plain or control codes are converted | proposed label |
| 6 | Convert control codes | switch | turns names such as ‹GS› into the control characters; proposed label |
| 7 | Show digits | switch, linear codes only, default on | prints the human-readable digits under the code; proposed label |
| 8 | Keep square | switch, Data Matrix only, default on | proposed label |
| 9 | Error correction | dropdown: Low, Medium, Quartile, High (the QR code standard's levels); for types that have levels | a dropdown instead of the reference's slider (own) |

Wi-Fi tab: Network name (field), Password (field), Hidden network (switch),
Security (dropdown of the network security kinds the Wi-Fi QR text format
names; Open question). Contact tab: First name, Last name, Address, Phone,
Note, Email, Web address (fields, `naming.md`). Labels right-aligned, fields
stretched (form grid). Editing either tab writes the Text tab's text; the
last tab edited wins (Match [R5]).

- Buttons: "OK", "Cancel". Cancel on a barcode just drawn removes the drawn
  box (nothing is created); on an existing barcode it keeps it unchanged.
- Preview: the code in `drawing` inside its box; invalid data replaces it
  with a cross in `danger` and the error band says "Cannot make a ‹type›:
  the characters ‹list› are not allowed in this code type."
- Commands `shape.add_barcode` and `shape.set_barcode` (proposed change to
  shapes.md).

Tier: **Echo** (type dropdown, three tabs, counter, live code, cross on
invalid data), **Own** (labels, dropdown for error correction, layout).

### Font browser (later)

#### Reference

- A font manager opened from the font box's right-click menu: a full-width
  search field with a preview zoom field at its right; on the left a narrow
  column of categories with counts (all, favourites, recent, active,
  inactive, and two font kinds), the chosen one a solid band; on the right a
  scrolling list, one row per font: a small kind mark, the font's name drawn
  large in that font, a sample sentence under it in the same font, and at
  the right a favourite star and a dot that hides the font [R26].

#### LaserCraft

- Title "Font browser". Kind: settings window, resizable. Opened by the
  "Font browser…" button beside the Font field of Add text, the Font
  field's right-click menu and the Text style toolbar (`property-bar.md`).
  Later.
- Contents:

| # | Part | Contents |
|---|---|---|
| 1 | Top row | field with the hint "Search fonts" (focused on open), then "Preview size" dropdown: Small, Medium, Large |
| 2 | Left column (about a sixth) | category list: "All fonts (‹n›)", "Favorites (‹n›)", "Recently used (‹n›)", later "Single-line (‹n›)" when a single-line font folder is set; chosen row `accent-soft` |
| 3 | Right area | `field` list, scrolling; each row: the family name in `type-body` `text` at the left, under it a sample line drawn in that family at the chosen preview size, at the right a star button (outline in `text-muted` when off, filled `text` when on; tooltip "Mark as favorite"); the chosen row `accent-soft` |

- The sample line shows the text typed in Add text when there is some,
  otherwise "LaserCraft 0123456789" (own).
- Buttons: "Use font" (primary), "Cancel". A double-click on a row is Use
  font. Use font sets the Font field of the window or toolbar that opened
  the browser (a draft change there, no command by itself).
- Favorites and recently used fonts are kept in the preferences.
- Hiding fonts is not offered (Open question).
- Samples are drawn from the font's outlines supplied by the app (the UI
  crate uses `lasercraft-import` only to list families today, ARCHITECTURE.md;
  proposed change there).

Tier: **Echo** (search, categories with counts, rows previewed in their own
face, favourite star), **Own** (labels, sample text, no hiding).

### Smaller messages from this file

| Name | Raised by | Kind and icon | Text | Buttons | Milestone |
|---|---|---|---|---|---|
| Replace file? | Save project as, Save G-code prompts, when the file exists | message, warning | "‹file name› already exists in ‹folder›. Replacing it overwrites its contents." | "Replace" (destructive), "Cancel" (focus) | with native dialogs or earlier (proposed) |
| Switch device profile? | opening a project saved with another device profile (M8) | message, question | "This project was saved for ‹name›. The active device profile is ‹current›." | "Switch to ‹name›" (primary), "Keep ‹current name›" (focus; also Esc and the close button) (`naming.md`) | M8 |

The reference asks a similar device-switch question with Yes and No [R20];
LaserCraft's buttons name the result (`naming.md`, Confirmations).

---

## egui implementation notes

- `crates/lasercraft-ui/src/dialogs.rs` grows from one `Dialog` enum into
  one draft struct per window plus a small set of shared helpers: the
  window frame (title strip, focus line, constraint), the form grid
  (`egui::Grid` with two columns, the first right-aligned through
  `Layout::right_to_left` per cell), the error band, and the button row.
  Every window returns an `Action` or an `Invocation` when its primary
  button runs; none changes the project itself (CLAUDE.md).
- Window: `egui::Window::new(title).collapsible(false).resizable(…)
  .constrain_to(rect)` where `rect` is the main window below toolbar row 1
  while connected; `default_pos` and `pivot` place the first open as
  "Where windows open" says. egui keeps each window's position by id for
  the run.
- Button row: drawn in a bottom strip inside the window
  (`TopBottomPanel::bottom` inside the window's `Ui`, or a final
  `ui.horizontal` after a separator). With a right-to-left layout egui adds
  items from the right, so the code adds Cancel first and the primary last
  to get "OK, Cancel" on screen.
- Keys: the window reads Enter and Esc with `ui.input_mut(|i|
  i.consume_key(…))` only while it has focus (its layer is the top
  focused layer), so a key never acts on two windows. Ctrl+Enter is read
  the same way inside boxes.
- Focus order follows widget creation order, so widgets are added in
  reading order; the first field calls `request_focus()` on the first frame
  the window is shown.
- Message backdrop: an `egui::Area` covering the main window except the job
  bar's rect, drawn below the message window and above everything else,
  painting the veil and sensing clicks; the job bar's own `Area` is drawn
  on the top-most order after it (`layout.md`). No `egui::Modal` while
  connected; replace today's `egui::Modal` in
  `crates/lasercraft-app/src/main.rs:190` before M6 ships.
- Previews: the shell emits a new action carrying the draft's invocation;
  the app runs it on a scratch copy of the project and hands back shapes
  and a summary line; the canvas draws them with the `drawing` style
  (`canvas.md`). Proposed change to ui-shell.md and commands.md.
- Tests (`crates/lasercraft-ui/tests/shell.rs`): for every window, focus is
  in the first field on open; Enter runs the primary invocation; Esc and
  the close button run nothing; the primary is disabled on a form error;
  an error from the app keeps the window open with the band; with each
  window kind open, a click on Stop in the job bar reaches the stop action
  (`layout.md`). Renamed labels update the tests that look for them
  (`naming.md`).
- Accessibility: egui's AccessKit is off today (current-ui.md); when it is
  enabled, every field's label is its accessible name (later, owner
  decision).

## Deliberately omitted

- Window icons, the program's logo and its version in dialog title strips,
  and the "?" help buttons (context help is F1, M10).
- The macOS button order and the native title bars (decision 12: one look
  on every platform).
- The device discovery wizard's brand watermark and every vendor logo.
- The project-file warnings box about missing fonts: LaserCraft stores text
  as outlines, so a project never needs a font to open (`text.md`).
- Laser-on framing by a modifier-click or by a device setting (decision 7;
  only the opt-in Low-power frame window, if approved).
- The setting that ignores shapes outside the bed: a job outside the bed is
  refused with a clear error (CLAUDE.md safety rules).
- The external nesting-website message (`naming.md`, Omitted); Nest works
  locally.
- Preferences items: the update check (packages and Flatpak update the
  program), the system-clipboard switch (copy always uses both clipboards,
  `ui-shell.md`), opening files in a running copy, the failed-send sound
  (proprietary controllers), the file-dialog choice, antialiasing,
  high-resolution scaling and integer rounding (egui follows the OS
  scale), curve quality (View → Smooth curves), the dark canvas (decision
  13), rotary and cylinder switches (M9 setups, galvo), the plotter-file
  import precision and print line width (not planned).
- Grid array's total-size input, centre-to-centre spacing, shifts,
  reversing, mirroring and random rotation; the offset's delete-originals
  and simplify options (`arrange.md` leaves them out by design; Open
  questions).
- Auto-incrementing variable text in the array windows (later with Text
  data).

## Current state → target

From `current-ui.md` (commit 151eb26); paths relative to the repo root.

- Argument windows (`crates/lasercraft-ui/src/dialogs.rs:97-121`): titles
  "Offset", "Grid array", "Circular array", "Add text" are kept. Each shows
  its fields in `ui.horizontal` rows with the label before the value and a
  lone "OK" button (`dialogs.rs:121`) → the form grid with right-aligned
  labels, an error band, and the button row "OK", "Cancel"; Enter and Esc
  as in Conventions; focus in the first field.
- Offset (`dialogs.rs:147-156`): "Distance (negative: inward)" with a " mm"
  suffix and a combo "Corners" showing lowercase "round", "miter", "bevel"
  → "Distance (mm)" with the hint "Negative moves inward"; "Corners" with
  Round, Miter, Bevel.
- Grid array (`dialogs.rs:157-162`): "Columns", "Rows", "Gap between
  columns", "Gap between rows" with " mm" suffixes → "Columns", "Column gap
  (mm)" left, "Rows", "Row gap (mm)" right; a summary line.
- Circular array (`dialogs.rs:163-176`): "Count (with the original)",
  "Center x", "Center y", "Sweep" with a "°" suffix, and the checkbox "Turn
  the copies with the circle" → "Total count" (hint "Includes the
  original"), "Center X (mm)", "Center Y (mm)", "Sweep (°)", the switch
  "Rotate copies"; a summary line.
- Add text (`dialogs.rs:177-191`): "Text:" with a colon, a box two lines
  tall, a combo "Font", "Capital height" with a " mm" suffix → "Text" (no
  colon), a box three lines tall, "Font" with typing to filter, "Capital
  height (mm)"; OK disabled with nothing visible; Ctrl+Enter.
- No previews today → live previews in `drawing` for Offset and the arrays
  (needs the dry-run path).
- Errors after OK close the window and go to the command line log → the
  window stays open with the error band.
- Project notes (`crates/lasercraft-ui/src/shell.rs:297-319`): a box ten
  rows tall and "Save notes"; closing discards → add "Cancel" and
  Ctrl+Enter; later the "Show when the project opens" switch.
- File prompts (`shell.rs:581-621`): prompt line, field, "OK", close button;
  Enter confirms → add "Cancel", prefill the field, keep the prompt open
  on errors, add "Replace file?".
- Unsaved changes (`crates/lasercraft-app/src/main.rs:185-210`): an
  `egui::Modal` with a heading, "The project has changes that are not
  saved. Discard them?" and "Discard", "Cancel" → the Message kind with a
  warning icon, the new text, "Save" (owner decision), "Discard"
  (destructive), "Cancel"; the backdrop leaves the job bar free.
- No theme is set (current-ui.md 2) → every window uses the tokens of
  `visual-language.md`.
- Nothing else in this file exists yet.

## Proposed changes to other specs

- **Proposed change to ui-shell.md**: replace "Each window builds one
  command when OK is pressed; closing it runs nothing" with this file's
  window kinds, button row (OK and Cancel in every argument window), keys
  (Enter, Esc, Ctrl+Enter), and the rule that an error keeps the window
  open with the message; tool windows draw a live `drawing` preview (also
  raised by `canvas.md`); file prompts gain Cancel, a prefilled field and
  "Replace file?"; Project notes gains Cancel; Unsaved changes gains Save
  (owner decision, `naming.md`) and continues the pending action after a
  successful save.
- **Proposed change to commands.md**: a dry run of an invocation on a
  scratch copy of the project that returns the resulting shapes and a
  summary without touching history, for previews.
- **Proposed change to keymap.md**: Ctrl+Enter presses the primary button
  while a multi-line box has focus; Pause and Stop chords work while a
  dialog field has focus (already raised by `layout.md`); the nudge
  bindings refer to the three named distances so Preferences can change
  them; a default location for the user keymap written by Keyboard
  shortcuts, loaded at start unless `--keymap` is given; the Pause and Stop
  chords cannot be rebound (owner decision).
- **Proposed change to arrange.md**: `arrange.path_array`, `select.circles`
  (or a selection spec) and `path.fit_slots`; the grid's linked copies
  (already raised by `naming.md`).
- **Proposed change to planning.md**: the Cut order settings (master
  switch, rules, the nine options) replace today's fixed order, with
  today's behaviour as the defaults.
- **Proposed change to project-file.md**: store the Cut order settings and
  the Project notes switch "Show when the project opens"; auto-save files
  (with General → Auto-save every (min)).
- **Proposed change to shapes.md**: a barcode shape kind with
  `shape.add_barcode` and `shape.set_barcode`.
- **Proposed change to canvas.md**: an invalid barcode's preview is a cross
  through its box in `danger`.
- **Proposed change to arrange.md** and **proposed change to geometry.md**
  (later): `path.close_within ‹tolerance›` and `path.simplify ‹tolerance›
  ‹arcs›` for the Close nearly closed paths and Simplify windows, each one
  undoable step; geometry.md decides whether arc segments are kept.
- **Proposed change to docs/spec/canvas.md**: the Grid spacing (mm)
  preference, 0 meaning the adaptive levels of `canvas.md` (style), a value
  above 0 fixing the minor step with major lines every ten.
- **Proposed change to layers.md** (M8): the General preference "New
  projects use saved layer defaults", with today's last-used behaviour as
  the default.
- **Proposed change to text.md**: the Font browser, and font samples
  drawn from outlines supplied by the app; Add text's OK disabled before
  the command would fail with "nothing to draw".
- **Proposed change to import.md**: DXF units, DXF close tolerance and SVG
  resolution become preferences with today's behaviour as defaults
  (already raised by `naming.md`).
- **Proposed change to safety.md** (only if the owner approves Low-power
  frame): rule 2 names the Low-power frame window as the only laser-on
  framing path, with the empty power field on every open, the cap, one
  pass, `M4`, and the bounds check; and rule 7 names the Message backdrop
  rule (no backdrop over the job bar).
- **Proposed change to ARCHITECTURE.md** (owner decision): a native file
  dialog crate with the XDG portal; the settings file for preferences; the
  UI receiving font outlines from the app for the Font browser.
- **Proposed change to naming.md**: add the labels this file needs that
  it does not list yet: "Replace file?", "Replace"; the measurement labels
  ("Shape", "Segment", "Nodes", "Straight segments", "Curved segments",
  "Kind", "Width (mm)", "Height (mm)", "Perimeter (mm)", "Area (mm²)",
  "Length (mm)", "From (mm)", "To (mm)", "Change (mm)", "Angle (°)",
  "Circle radius (mm)", "Circle centre (mm)", "Copy values"; each is a
  generic geometric term checked against `docs/clean-room.md`, and naming.md
  should record them as such); "Up to (mm)" hint; Cut order's "Rules",
  "Options", the three rule rows; the Preferences group headings
  ("Saving", "Starting jobs", "Text", "Layers", "Layout", "Grid", "Snapping", "Nudge
  distances", "Interface", "Canvas", "DXF", "SVG", "Output") and the size
  values ("Smaller", "Normal", "Larger", "Largest"); Keyboard shortcuts'
  "All commands", "Canvas keys", "Machine keys", "Command", "Shortcut";
  "Restore backup"; Fit slots to thickness's "Current thickness (mm)", "New
  thickness (mm)", "Tolerance (mm)", "Change" with "Slot depth", "Slot
  width", "Tab height"; Barcode's "Code type", "Encoded text", "Convert
  control codes", "Show digits", "Keep square", "Error correction"; Font
  browser's "Search fonts", "Preview size", "All fonts", "Favorites",
  "Recently used", "Single-line", "Use font"; Low-power frame's "Trace",
  "Bounding box", "Outline", "Frame power (%)", "Frame at ‹p›%";
  "Start job"; "Place the parts that fit"; "New projects use saved layer
  defaults"; Grid spacing (mm)'s hint "0 means automatic"; Close nearly
  closed paths' "Tolerance (mm)" (`naming.md` already uses that word for
  it) and its summary wording, Simplify's "Tolerance (mm)", "Fit arcs",
  "Nodes before", "Nodes after"; "Edit list" as a title; the
  History row "Start".

## Open questions

1. Unsaved changes: add "Save" beside Discard and Cancel (`naming.md`
   question 1)? If yes, after Save the pending action continues as
   specified.
2. Quitting or closing the window while a job runs: refuse with a message,
   ask to stop the job first, or allow it (streaming would stop)? This file
   proposes a message "Stop the job first" with Stop and Cancel, pending the
   owner.
3. Tool windows: is locking project editing while one is open (pan and zoom
   still allowed) acceptable, or should the canvas stay fully editable and
   the preview follow the changes?
4. Button order: keep one order on every platform (OK then Cancel), as
   decided here from decision 12, or follow macOS order on macOS?
5. Answered: `visual-language.md` defines `backdrop` for the Message veil
   and sets its strength in both themes.
6. Offset: add a both-sides direction and an "ignore nested shapes"
   switch (`arrange.md` would change)? Deleting the originals and
   simplifying the result stay out by design (Deliberately omitted).
7. Grid array: add any of the reference's extra options (total size as
   input, centre-to-centre spacing, shifts, mirroring, random rotation)?
8. Circular array: default centre at the work area's centre (today) or at
   the selection's centre? Add a "use the anchor object's centre" control?
9. Add text: draw a live preview of the text outline in `drawing` while the
   window is open?
10. Select circles: what counts as a circle, and should matching circles
    inside a group select the whole group (arrange.md widens selections) or
    be taken out of it?
11. Cut order: which options force others (backlash-friendly order), and is
    the run-order rule wanted before the Properties panel exists?
12. Preferences: should "Block Start while the screen is off" and "Start
    button skips keyboard focus" be fixed behaviour instead of switches
    (`naming.md`)? Auto-save's default? Add inverted wheel zoom, returning
    to Select after drawing, a full-window crosshair, a bed-centre cross,
    the import switches?
13. Preferences, Camera tab (M9): which settings belong there?
14. Keyboard shortcuts: make the Pause and Stop chords fixed as proposed?
15. Restore preferences: when are backups made and how many are kept?
16. Low-power frame: offer it at all (`naming.md` question 13)? If yes,
    what framing ceiling (a percentage of full power) caps the field, and
    should it be limited to diode lasers as the reference is?
17. Before you start: title of the list editor ("Edit list" by the title
    rule, or a clearer noun)?
18. Barcode: which library and which symbologies; the Wi-Fi security
    choices; the reference's extra spacing option for retail codes.
19. Font browser: offer hiding fonts from the font lists?
20. Simplify: offer "Fit arcs" at all (it needs arc segments in
    `geometry.md` and arc output in the G-code writer), and is 0.05 mm a
    good default tolerance?
21. Close nearly closed paths: is today's DXF join tolerance the right
    default, or should it have its own?
22. Native file dialogs: which crate, and keep the typed prompt in the
    menus as an alternative?
23. "Replace file?": ask before overwriting with the typed prompt now, or
    only once native dialogs (which ask themselves) arrive?

## Sources

All LightBurn pages are part of the public 2.1 documentation, read
2026-10-21 through the style guide's research notes (tool dialogs, chrome
and preferences, preview and device, property bars, left toolbars, menus,
machine windows, top toolbars, canvas). They support the Reference parts
only; no wording is taken from them.

1. [R1] https://docs.lightburnsoftware.com/2.1/Reference/GridArray/ — grid array dialog: opening, size and placement, two direction columns, rows, centred rows, total line, switches, OK then Cancel with the default outline, macOS order, focus ring, greyed field, title strip, Enter and Esc unknown
2. [R2] https://docs.lightburnsoftware.com/2.1/Reference/CircularArray/ — circular array dialog: size, rows, centre fields, switches, buttons on the last row, title strip, form column
3. [R3] https://docs.lightburnsoftware.com/2.1/Reference/OffsetShapes/ — offset dialog: opening, Ctrl+click repeat, distance, three groups, live preview with the original dashed, placement beside the shape, wheel over a field, macOS order
4. [R4] https://docs.lightburnsoftware.com/2.1/Reference/CopyAlongPath/ — copy along path: selection order, greyed menu item, rows and switches, greyed scale field, live preview, small square size
5. [R5] https://docs.lightburnsoftware.com/2.1/Reference/BarCode/ — bar code dialog: creation by dragging, reopening, type dropdown, three tabs, counter, error line, evaluated text, switches, sliders, last tab wins, Cancel deletes a new code, live canvas update and cross on invalid data, focus border, dropdown list look
6. [R6] https://docs.lightburnsoftware.com/2.1/Reference/ResizeSlots/ — resize slots dialog: fields with units, titled group of three choices, message area, Apply, OK, Cancel, applied changes stay, "?" button
7. [R7] https://docs.lightburnsoftware.com/2.1/Reference/QuickNest/ — quick nest error boxes: icon, one line, single OK, shadow, the four causes
8. [R8] https://docs.lightburnsoftware.com/2.1/Reference/NestSelected/ — information box without icon, two lines, OK on a darker strip (the feature itself is omitted)
9. [R9] https://docs.lightburnsoftware.com/2.1/Reference/BooleanTools/ — boolean helper window: five picture buttons, rule, instructions, Cancel and OK, hover and click preview, reset, closed shapes
10. [R10] https://docs.lightburnsoftware.com/2.1/Reference/Measure/ — measurement window: two columns of measures, unit button, OK, hover highlights, drag measuring
11. [R11] https://docs.lightburnsoftware.com/2.1/Reference/Selection/ — select circles dialog: diameter, maximum greyed unless range, exact or range, OK and Cancel
12. [R12] https://docs.lightburnsoftware.com/2.1/Reference/UndoRedo/ — undo history window: instructions, list with undo, redo and current markers, OK and Cancel, preview on choosing
13. [R13] https://docs.lightburnsoftware.com/2.1/Reference/OptimizationSettings/ — cut optimization dialog: master switch, rule buttons, list and remove, nesting rules, options and sweep choices, defaults buttons at the left, stored in the project
14. [R14] https://docs.lightburnsoftware.com/2.1/Reference/SettingsPreferences/ — settings dialog: entry points, global scope, fixed size, tabs with pictograms, captions over framed panels, every tab's contents, OK and Cancel without Apply, regular-weight captions
15. [R15] https://docs.lightburnsoftware.com/2.1/Reference/EditHotkeys/ — hotkey editor: two columns, categories, reset all, focused search, table, instructions, capture field with clear, export and import, OK and Cancel
16. [R16] https://docs.lightburnsoftware.com/2.1/Reference/ManagingPreferences/ — preferences submenu and the backup window: two panes, dated backups with sizes, details with device list, Cancel and load
17. [R17] https://docs.lightburnsoftware.com/2.1/Reference/HelpAndNotes/ — help window: three text tabs, text area with a heading, single OK, entry points and F1
18. [R18] https://docs.lightburnsoftware.com/2.1/Reference/UI/HelpMenu/ — about and quick-help items open the help window; F1 shown beside the quick-help item
19. [R19] https://docs.lightburnsoftware.com/2.1/Reference/ShowNotes/ — project notes: menu and Ctrl+Alt+N, stored in the project, box focused on open, show-on-open switch at the left, OK and Cancel, "?" button
20. [R20] https://docs.lightburnsoftware.com/2.1/Reference/FileManagement/ — unsaved-changes prompt with three buttons, device-switch question, native save dialog with a file-type list
21. [R21] https://docs.lightburnsoftware.com/2.1/Reference/CutSelectedGraphics/ — nothing-selected warning with its two buttons and the actions that raise it
22. [R22] https://docs.lightburnsoftware.com/2.1/Reference/DeviceSettings/BasicSettings/ — job checklist setting with its edit button; laser-on framing setting with a percentage, greyed unless the fire button is on
23. [R23] https://docs.lightburnsoftware.com/2.1/Reference/LaserWindow/ — Start may show a warning or a checklist first; modifier-click framing with the laser on; framing speed and power from the move panel
24. [R24] https://docs.lightburnsoftware.com/2.1/GetStarted/JobControl/ — warning and checklist before a job starts
25. [R25] https://docs.lightburnsoftware.com/2.1/GetStarted/FramingBeginner/ — continuous framing with a dialog whose OK ends it
26. [R26] https://docs.lightburnsoftware.com/2.1/Reference/Text/ — text typed on the canvas, the edit-text dialog setting and menu item, font sources, the font manager window, the missing-font warning box
27. [R27] https://docs.lightburnsoftware.com/2.1/GetStarted/FindMyLaser/ — discovery wizard: wide window, inset page, footer buttons with the first greyed, floating progress box, add enabled after a choice, larger prompts and smaller hints, watermark
28. [R28] https://docs.lightburnsoftware.com/2.1/GetStarted/CreateManually/ — manual setup wizard: header with a back arrow and title, footer with Next and Cancel, classic variant, Next greyed until filled, review page with Back and Finish, prompt sizes
29. [R29] https://docs.lightburnsoftware.com/2.1/Reference/TaperWarp/ — small dialog with a "?" button and a canvas preview while values change
30. [R30] https://docs.lightburnsoftware.com/2.1/Reference/DeviceSettings/ — device settings dialog: units inside some values, OK and Cancel on Windows and the macOS order
31. [R31] https://docs.lightburnsoftware.com/2.1/Reference/Preview/ — the preview as a separate resizable window with a grip
32. [R32] https://docs.lightburnsoftware.com/2.1/Reference/Devices/ — device list: OK keeps every change, Cancel drops everything including new devices
33. [R33] https://docs.lightburnsoftware.com/2.1/Reference/UI/Tooltips/ — F1 over a control opens its online page
34. [R34] https://docs.lightburnsoftware.com/2.1/Reference/UI/EditMenu/ — the Edit menu items that open a dialog to close open paths within a chosen distance (greyed with no open path) and one to smooth or refit shapes to arcs and lines within a tolerance, with Alt+Shift+O

Existing specs and documents used: `docs/spec/ui-shell.md` (windows build
one command on OK, closing runs nothing; typed-path windows; Unsaved
changes; Files), `docs/spec/arrange.md` (offset, grid and circular array
commands, their rules, limits and messages; anchor and subtract order),
`docs/spec/text.md` (Add text window, defaults, ranges, errors),
`docs/spec/planning.md` (fixed cut order, flattening tolerance),
`docs/spec/canvas.md` (grid, snapping and click tolerances),
`docs/spec/import.md` (DXF units, SVG resolution, join tolerance),
`docs/spec/commands.md` (undo, redo, one step per command),
`docs/spec/keymap.md` and `crates/lasercraft-ui/keymap/default.keymap`
(bound chords: Ctrl+O, Ctrl+Shift+S, Ctrl+I, Ctrl+T, Ctrl+Alt+N, Alt+O,
Alt+Shift+L; later chords Ctrl+B, Alt+M; nudge distances; shortcuts ignored
in text fields), `docs/spec/safety.md` (rules 1, 2, 5, 6, 7, 9, 10),
`docs/spec/layers.md`, `docs/spec/geometry.md`, `docs/clean-room.md`,
`ARCHITECTURE.md` (deferred file dialogs, crate graph), the audit
`current-ui.md`, and the style files `layout.md`, `naming.md`, `menus.md`,
`toolbars.md`, `property-bar.md`, `canvas.md`, `layers-panel.md` and
`machine-panels.md`. Open standards: WCAG 2.2 (contrast minimum 1.4.3,
non-text contrast 1.4.11, focus visible 2.4.7),
https://www.w3.org/TR/WCAG22/; the QR code standard's error correction
levels (ISO/IEC 18004); SVG and CSS pixel units (W3C, via `import.md`).

Own design: the six window kinds and what each blocks; one tool window at a
time and queued messages; the title strip look, the focus line and title
rules; placement, resizable set, constraint below row 1 and session memory;
the form grid with switches in the control column, choice-with-field rows,
hints, summaries and no group boxes; field width rules; draft-only commits;
the button order on every platform, the button styles and contrast targets;
focus on open, Enter, Ctrl+Enter, Esc and Tab rules; which windows preview,
the dry-run path and preview timing; the three validation levels and the
error band; every rule for windows while connected and while a job runs;
the message anatomy, kinds, icons and backdrop; the wizard step indicator,
inline progress and button rules; the typed prompts' prefill, error and
overwrite handling and the native-dialog filters; every label, tooltip,
message, default, range and summary in the dialog sections; the Cut order
rule rows and defaults; the Preferences groups, values and defaults; the
Keyboard shortcuts categories, conflict and fixed-chord rules; the restore
safety backup; the Quick reference contents; the whole Low-power frame
window; Before you start's switches and list editor; the Nesting failed
cases; the Fit slots summary; the Barcode layout and dropdown; the Font
browser sample and layout; the Close nearly closed paths and Simplify
windows (fields, ranges, defaults, summaries, previews); the layer-defaults
preference and the reasons for the omitted import and export switches; and the egui notes and tests.
