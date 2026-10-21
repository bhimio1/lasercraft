# Spec: keyboard shortcuts

Milestone: M0 (file and edit chords). The rest of the published defaults are
bound as their commands arrive (mostly M1 and M3).

## Behavior

- Shortcuts are data: `crates/lasercraft-ui/keymap/default.keymap` maps chords
  to invocations, one `<chord> = <invocation>` per line, `#` comments allowed.
  The UI only looks chords up.
- A user keymap can replace the default: `lasercraft --keymap <file>`.
- `Ctrl` means `Cmd` on macOS, matching the published macOS column (⌘ where
  Windows/Linux use Ctrl).
- Shortcuts are ignored while a text field has keyboard focus.
- A chord bound twice in one file is an error naming the line.
- `ui.new`, `ui.open`, `ui.save`, `ui.save_as` and `ui.quit` are handled by the
  front end (they need the file path or a prompt). Everything else is a
  registry command.
- Menu items show the chord bound to their action.

## M0 bindings

| Chord | Action |
| --- | --- |
| Ctrl+N | New project (asks before discarding unsaved changes) |
| Ctrl+O | Open project |
| Ctrl+S | Save |
| Ctrl+Shift+S | Save as |
| Ctrl+Q | Quit (asks before discarding unsaved changes) |
| Ctrl+Z | Undo |
| Ctrl+Shift+Z | Redo |

Ctrl+Y is deliberately not bound: it is not in the published defaults.

## Published defaults to adopt later

Chords from the public page, with the action described in our own words and
the milestone that adds the command. Mouse gestures listed there (Shift+click
adds to the selection, Ctrl+click removes, Shift constrains a drag, Ctrl
disables snapping while dragging, middle-drag or Space+drag pans, the wheel
zooms about the cursor) are M1.

| Chord (Win/Linux) | Our action | Milestone |
| --- | --- | --- |
| Ctrl+I | Import a file | M3 |
| Alt+Shift+L | Save G-code | M5 |
| Alt+X | Export vectors | M3 |
| Ctrl+A / Ctrl+Shift+I | Select all / invert selection | M1 |
| Ctrl+X / Ctrl+C / Ctrl+V / Alt+V | Cut / copy / paste / paste at original position | M1 |
| Ctrl+D | Duplicate | M1 |
| Delete, Backspace | Delete selection | M1 |
| Esc | Back to the selection tool | M1 |
| Arrows; Shift+arrows; Ctrl+arrows | Nudge selection; large steps; small steps | M1 |
| Ctrl+L / Ctrl+R / Ctrl+E | Line / rectangle / ellipse tool | M1 |
| Ctrl+= / Ctrl+- / Ctrl+0 | Zoom in / out / fit the bed | M1 |
| Ctrl+G / Ctrl+U | Group / ungroup | M3 |
| Ctrl+Shift+H / Ctrl+Shift+V | Mirror horizontally / vertically | M1 |
| Alt+arrows, Alt+PgUp, Alt+PgDn | Align left/right/top/bottom, centers vertically/horizontally | M3 |
| PgUp / PgDn, Ctrl+PgUp / Ctrl+PgDn | Raise/lower in drawing order, to front/back | M1 |
| `.` / `,`, Ctrl+`.` / Ctrl+`,`, Ctrl+Shift+`.` / Ctrl+Shift+`,`, Shift+`>` / Shift+`<` | Rotate 90°, 10°, 5°, 45° clockwise / counter-clockwise (canvas focus) | M1 |
| L R T B C E P (canvas focus) | Align left, right, top, bottom, centers vertically, centers horizontally; move to the bed center | M3 |
| Ctrl+\` | Node editing | M3 |
| Ctrl+T | Text tool | M3 |
| Alt+O / Ctrl+W | Offset / weld | M3 |
| Alt++ / Alt+- / Alt+* | Union / subtract / intersect | M3 |
| Alt+B | Break a shape into its parts | M3 |
| Ctrl+Tab | Add tabs | M4 |
| Alt+P | Preview the job | M4 |
| Alt+Shift+W | Toggle outline/filled display | M2 |
| Ctrl+Shift+A / Alt+R | Frame the selection / frame its square bounds | M6 |
| Alt+L | Move the laser head to a clicked point | M6 |
| Pause / Ctrl+Break | Pause / stop the job | M6 |
| Alt+I / Alt+T | Adjust image / trace image | M7 |
| Ctrl+Shift+R | Rotary setup | M9 |
| Alt+0…4 | Print-and-cut steps | M9 |
| Alt+C | Refresh the camera overlay | M9 |

Conflicts to settle when binding (the page lists them under different
contexts): Alt+C is also "cut shapes" in the vector tools table, and the single
letters L R T B C E P are canvas-only. Node-editing single keys (S, L, C, D, I,
M, B, T, E while hovering) are M3. Pause and Stop also get on-screen buttons
that are always visible while connected (`docs/spec/safety.md`).

## Sources

- LightBurn documentation, "Hotkeys and Gestures":
  https://docs.lightburnsoftware.com/legacy/Hotkeys
  (`https://docs.lightburnsoftware.com/Hotkeys.html` redirects there; read
  2026-10-21). Only chord assignments are taken; action descriptions are our own.
- Keymap file format and `ui.*` actions: own design, carried over from
  MusicCraft.
