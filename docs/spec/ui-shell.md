# Spec: main window shell

Milestone: M0. M1 replaces the work-area outline with the canvas.

## Layout

- Menu bar: File (New, Open…, Save, Save As…, Quit) and Edit (Undo, Redo).
  Undo names the command it would reverse. Disabled items are greyed out.
- Center: the work area drawn to scale, its size above it, and a dot at the
  origin corner. Screen up is the rear of the bed.
- Console (bottom, resizable): a line where any command can be typed and run,
  and a log of each command with its message or error.
- Status bar: project name, a dot when there are unsaved changes, the file
  path, and the work area size and origin.
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

## Rules

- The UI crate only produces `Action`s; the app runs them as commands.

## Sources

- Own design. Menu entry names are generic terms, not copied labels.
