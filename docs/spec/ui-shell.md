# Spec: main window shell

Look and feel, names and the target layout are in `docs/spec/style/`
(start with its README). Where it proposes changes to this spec, they are
labelled there and land here before code.

Milestones: M0 (shell), M1 (canvas, tools, editing menus), M2 (layers panel,
palette, notes), M6 (Laser panel and menu), M7 (images on the canvas, Adjust Image).

## Look

- The app installs LaserCraft's light theme at start instead of following
  the operating system's light or dark preference. Every color, type size
  and spacing comes from the tokens of `docs/spec/style/visual-language.md`,
  whose values live only in the UI crate's theme module (`theme.rs`), with
  tests for its contrast targets. A dark theme and a Theme preference are
  later.
- Settings are pill switches (accent and knob right when on, grey and knob
  left when off), never check boxes. Stop and other destructive buttons are
  filled `danger` with white labels.

## Layout

- Menu bar, in this order (`docs/spec/style/menus.md`; the tables live in
  the UI crate's `menus.rs`):
  - File: New; Open…, Import…, Project notes…; Save, Save as…, Save
    G-code…; Quit.
  - Edit: Undo ‹action›, Redo; Select all, Invert selection, Cut, Copy,
    Duplicate, Paste, Paste in place, Delete; Convert to path, Close path,
    Join paths.
  - Tools: Select, Line, Shape tools ▸ (Rectangle, Ellipse), Edit nodes,
    Text (the active tool is marked); Offset…, Weld, Union, Subtract,
    Intersect; Image adjustments….
  - Arrange: Group, Ungroup; Mirror horizontally / vertically, Rotate 90°
    clockwise / counter-clockwise; Align ▸, Distribute ▸ (spacing and
    centers, then Pack horizontally / vertically); Move on bed ▸; Grid
    array…, Circular array…; Break apart; Bring forward, Send backward,
    Bring to front, Send to back; Select next, Select previous.
  - Machine: Connect or Disconnect; Start, Pause, Stop; Frame, Frame
    outline; Home, Go to user origin, Set user origin, Clear user origin,
    Clear alarm.
  - View: Job preview…, Fit bed, Zoom in, Zoom out; Side panels (ticked
    while shown).
- Labels are sentence case. Undo reads "Undo ‹action›" with the readable
  action name from `lasercraft-commands` (`docs/spec/commands.md`, "Action
  names"), never a command id; plain "Undo", greyed, with nothing to undo.
- Each row has its own rule for when it can run (something selected, two or
  more objects, three or more for spacing, exactly two for the booleans, a
  group, an open path, an image, a connected and ready machine). A row that
  cannot run is greyed and its tooltip says what is missing ("Select two or
  more objects.", "Connect to a machine first.").
- Each item shows its chord from the keymap in symbol form (Ctrl+=, PgUp,
  Del, Esc, Alt+Left; Cmd and Option on macOS; `docs/spec/keymap.md`).
- Left: the drawing tools column, icon-only buttons one `icon` square each
  (`docs/spec/style/toolbars.md`, "Drawing tools"): Select, Line, Shape
  tools, Edit nodes, Text. Shape tools is a flyout: its face shows the last
  chosen member (Rectangle by default; Ctrl+R and Ctrl+E pick a member and
  update the face); a plain click picks the face member; a click in the
  lower-right corner marker or a right-click opens the popup with
  Rectangle and Ellipse (icon, label, chord). The active tool has an
  `accent-soft` background with a thin `accent` outline; hover shows
  `hover`. Tooltips read "‹Tool name› tool (‹chord›)" with a second line
  where the guide gives one. Clicking a tool button does not keep keyboard
  focus.
- Icons (`docs/spec/style/icons.md`) are our own SVG drawings in
  `crates/lasercraft-ui/icons/`, one per id, in a restricted subset (square
  24-unit view box; `path`, `line`, `polyline`, `polygon`, `rect`,
  `circle`, `ellipse`; stroke only, except small convex fills; at most one
  element with the id `accent`; no text, images, gradients, filters or
  masks). They are read once through our SVG importer and painted as
  strokes with round caps and joins in theme colors: base parts in `text`
  (`text-disabled` when disabled), the accent part in `accent`. Dashed
  strokes follow `stroke-dasharray`. Tests check every file against the
  subset and that every icon the UI names exists and stays inside the safe
  margin. `PROVENANCE.md` there records who drew each file.
- Right: the dock (see M6 additions). Its Layers panel lists the layers that
  have shapes in output order under a header row (Color, Name, Mode, Speed,
  power, Output, Show); a color with vectors and images has two rows, the
  image row reading "Image". Rows are one line: numbered swatch (hollow when
  hidden, faded when not output), name, a Line / Fill dropdown, "‹speed› /
  ‹power›%" with an "M3" tag for constant power, and Output and Show
  switches. Clicking anywhere on a row edits that layer; Shift+click
  selects its shapes; when every selected shape is on one layer, that layer
  is edited. ⏶ / ⏷ beside the list move the edited layer; Delete shapes,
  Copy settings and Paste settings (`layer.copy_settings`) sit below it.
  The editor starts with the layer's swatch and Name (Enter or leaving the
  field commits, Esc restores, an empty name is refused), then two
  label/field pairs per line with units in the labels and no suffixes:
  Mode | Speed (mm/s), Passes | Max power (%), Air assist | Constant power
  (M3) (with a warning band while on), then the line or fill settings.
  "Images on this layer" holds Method (readable names: Grayscale,
  Threshold, Floyd–Steinberg, Jarvis, Stucki, Atkinson, Ordered (Bayer),
  Halftone) | Interval (mm), Negative | Min power (%) (Grayscale only),
  Beam compensation (mm), Halftone frequency (lpi) | Halftone angle (°). A
  dragged value is committed as one command when the drag ends; typed
  values when entered.
- Bottom: the palette strip, one line of 30 square swatches numbered 00–29
  (black or white digits, whichever contrasts more); swatches that do not
  fit fold into a "»" More button. The current color has a heavy outline.
  Clicking a color with shapes selected moves them to that layer, otherwise
  it becomes the current color; the tooltip says which.
- F12 or View → Side panels hides and shows the tool palette and the layers
  panel. File → Project notes… (Ctrl+Alt+N) edits the notes.
- View → Job preview… (Alt+P) opens the job preview window
  (`docs/spec/preview.md`, M4). The shell emits `Action::Preview`; the app
  plans the job and hands the shell the moves and text to draw.
- Center: the canvas (`docs/spec/canvas.md`).
- Command line (bottom): a line where any command can be typed and run.
  Its log of each command with its message or error is collapsed by
  default and opens above the line with the "Log" toggle (or by clicking
  the status bar's message).
- Status bar, left to right: pointer position in mm (kept, muted, after the
  pointer leaves the canvas), the selection's count and size, Messages (an
  alarm, else the last log line; errors on `danger-soft`), the work area
  size and origin ("origin front left"), and the project name with a dot
  when modified and the file path.
- The canvas re-fits the bed when its size changes until the user zooms or
  pans.
- Window title: project name, prefixed with a dot when modified.

## M7 additions

- Images are drawn on the canvas as textures with a border in the layer
  color (`docs/spec/images.md`, "Canvas and panels"). The layer list says
  `image` or `line + image`; the editor's image section holds the method
  and its settings.
- Tools → Image adjustments… (Alt+I) opens the adjustments of the selected
  image; with none selected the console says so. The import prompt takes
  `.svg`, `.dxf`, `.png` and `.jpg`.

## M6 additions

- The right dock has two groups: the upper one with Layers, Jog and
  Console tabs along its bottom, the lower one with Machine; each group has
  a title strip with the front panel's name and a close cross. Window →
  Reset layout, Toolbars (Drawing tools, Palette) and Panels (Console, Jog,
  Layers, Machine) show and hide them. The machine panels
  (`docs/spec/laser.md`, "Panel layout") draw `LaserView` data the app
  builds from device events, and emit `device.*` invocations (run through
  the session like any command) or `Action::Laser` for what only the app
  remembers (user origin, port list, hiding bands).
- Machine menu (see Layout): Pause reads "Resume" while a job is held and
  "Pause (motion only)" while a constant-power (M3) job runs; Pause and
  Stop are enabled whenever a machine is connected. Clear alarm (`$X`) is
  enabled only in an alarm. Disconnect waits for the job to end.
- While a laser is connected, toolbar row 1 holds the job bar at its right
  end: state dot and word (a click shows the Machine panel), "‹n›% sent",
  "The laser may stay on" during an M3 job, Pause and Stop. It is drawn
  above every window (safety rule 7).
- Alarms, faults, refused jobs and permission help are bands in the Machine
  panel, not windows.

## M3 additions

- File → Import… asks for an `.svg` or `.dxf` path in the same kind of
  window as Open and runs `project.import`.
- Tools: weld, union, subtract, intersect, Offset… (a window for distance
  and corner style), convert to path, close path, join paths, break apart.
- Arrange: group, ungroup; Align, Distribute and Move on Bed submenus;
  Grid Array… and Circular Array… windows (columns, rows and gaps; count,
  center, sweep and whether copies turn).
- Each window builds one command when OK is pressed; closing it runs
  nothing.

## Files

- Open and Save As ask for a path in a small window (native dialogs: see
  `ARCHITECTURE.md`, deferred decisions). Save without a known path asks too.
- `lasercraft <file.lcproj>` opens that file at start.
- New, Open, Quit, closing the window, and typing `project.new` or
  `project.load` in the console all ask "Discard unsaved changes?" first when
  the project is modified.
- A failed load or save leaves the project and the known path unchanged and
  logs the error.

## Clipboard

- Copy and cut also put the copied shapes on the system clipboard as JSON.
  The windowing layer only reports the paste chord when the system clipboard
  holds something, so this keeps paste working; the paste itself always uses
  LaserCraft's own clipboard.

## Unsaved changes

- "Modified" compares the document with the one last saved or loaded;
  selecting shapes or copying them does not count as a change.

## Rules

- The UI crate only produces `Action`s; the app runs them as commands.

## Sources

- Own design. Menu entry names are generic terms, not copied labels.
- `docs/spec/style/menus.md` and `docs/spec/style/naming.md` (own design).
