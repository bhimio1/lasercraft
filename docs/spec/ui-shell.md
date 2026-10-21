# Spec: main window shell

Milestones: M0 (shell), M1 (canvas, tools, editing menus), M2 (layers panel,
palette, notes), M6 (Laser panel and menu), M7 (images on the canvas, Adjust Image).

## Layout

- Menu bar: File (New, Open…, Save, Save As…, Quit); Edit (Undo, Redo, Cut,
  Copy, Duplicate, Delete, Invert Selection, Paste, Paste in Place, Select
  All); Tools; Arrange (rotate 90°, mirror, drawing order, select next and
  previous); View (zoom in, zoom out, fit bed). Undo names the command it
  would reverse. Items that need a selection or a clipboard are greyed out
  without one. Each item shows its shortcut from the keymap.
- Left: the tool palette (Select, Line, Rectangle, Ellipse, Text, Edit
  Nodes); the active tool is highlighted.
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
- F12 or View → Side Panels hides and shows the tool palette and the layers
  panel. File → Project Notes… (Ctrl+Alt+N) edits the notes.
- View → Job Preview… (Alt+P) opens the job preview window
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
- Tools → Adjust Image… (Alt+I) opens the adjustments of the selected
  image; with none selected the console says so. The import prompt takes
  `.svg`, `.dxf`, `.png` and `.jpg`.

## M6 additions

- The right dock has two tabs: Cuts / Layers and Laser. The Laser panel
  (`docs/spec/laser.md`, "Panel layout") draws `LaserView` data the app
  builds from device events, and emits `device.*` invocations (run through
  the session like any command) or `Action::Laser` for what only the app
  remembers (user origin, port list, closing the alarm and notice windows).
- Laser menu: Laser Panel, Start, Frame, Frame Outline, Pause / Resume, Stop,
  Home, Unlock, Disconnect.
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
