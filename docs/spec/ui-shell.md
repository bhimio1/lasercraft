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
  Del, Esc, Alt+⏴; Cmd and Option on macOS; `docs/spec/keymap.md`).
- Left: the drawing tools (Select, Line, Rectangle, Ellipse, Text, Edit
  nodes); the active tool is highlighted.
- Right: the layers panel. It lists the layers that have shapes in output
  order (swatch, name, mode, speed and power, output and show switches);
  clicking a name opens it in the editor below, Shift+click selects its
  shapes. Move up / Move down / Delete shapes act on the edited layer. The
  editor shows the layer's settings (fill settings only in fill mode; image
  settings in a collapsed section). A dragged value is committed as one
  command when the drag ends; typed values when entered.
- Bottom: the palette strip of 30 colors; the current color for new shapes is
  outlined. Clicking a color with shapes selected moves them to that layer,
  otherwise it becomes the current color.
- F12 or View → Side panels hides and shows the tool palette and the layers
  panel. File → Project notes… (Ctrl+Alt+N) edits the notes.
- View → Job preview… (Alt+P) opens the job preview window
  (`docs/spec/preview.md`, M4). The shell emits `Action::Preview`; the app
  plans the job and hands the shell the moves and text to draw.
- Center: the canvas (`docs/spec/canvas.md`).
- Console (bottom, resizable): a line where any command can be typed and run,
  and a log of each command with its message or error.
- Status bar: project name, a dot when there are unsaved changes, the file
  path, the work area size and origin, the pointer position in mm, and the
  selection's count and size.
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

- The right dock has two tabs: Cuts / Layers and Laser. The Laser panel
  (`docs/spec/laser.md`, "Panel layout") draws `LaserView` data the app
  builds from device events, and emits `device.*` invocations (run through
  the session like any command) or `Action::Laser` for what only the app
  remembers (user origin, port list, closing the alarm and notice windows).
- Machine menu (see Layout): Pause reads "Resume" while a job is held and
  "Pause (motion only)" while a constant-power (M3) job runs; Pause and
  Stop are enabled whenever a machine is connected. Clear alarm (`$X`) is
  enabled only in an alarm. Disconnect waits for the job to end.
- While a laser is connected the status bar starts with Pause (or Resume)
  and Stop and the machine state, so they stay reachable whatever panel or
  window is open (safety rule 7).
- A "Laser alarm" window shows an alarm until closed, with Unlock and Home;
  a "Laser" window shows permission help and failed jobs.

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
