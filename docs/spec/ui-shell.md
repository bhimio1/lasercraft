# Spec: main window shell

Milestones: M0 (shell), M1 (canvas, tools, editing menus).

## Layout

- Menu bar: File (New, Open…, Save, Save As…, Quit); Edit (Undo, Redo, Cut,
  Copy, Duplicate, Delete, Invert Selection, Paste, Paste in Place, Select
  All); Tools; Arrange (rotate 90°, mirror, drawing order, select next and
  previous); View (zoom in, zoom out, fit bed). Undo names the command it
  would reverse. Items that need a selection or a clipboard are greyed out
  without one. Each item shows its shortcut from the keymap.
- Left: the tool palette (Select, Line, Rectangle, Ellipse); the active tool
  is highlighted.
- Center: the canvas (`docs/spec/canvas.md`).
- Console (bottom, resizable): a line where any command can be typed and run,
  and a log of each command with its message or error.
- Status bar: project name, a dot when there are unsaved changes, the file
  path, the work area size and origin, the pointer position in mm, and the
  selection's count and size.
- Window title: project name, prefixed with a dot when modified.

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
