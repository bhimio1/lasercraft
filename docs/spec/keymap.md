# Spec: keyboard shortcuts

Milestones: M0 (file and edit), M1 (selection, drawing, transform, view), M2
(notes, side panels), M3 (import, text, nodes, groups, booleans, alignment).
Later chords are bound as their commands arrive; the table below says when.

## Behavior

- Shortcuts are data: `crates/lasercraft-ui/keymap/default.keymap` maps chords
  to invocations, one `<chord> = <invocation>` per line, `#` comments allowed.
  The UI only looks chords up.
- A user keymap can replace the default: `lasercraft --keymap <file>`.
- `Ctrl` means `Cmd` and `Alt` means `Option` on macOS, which is exactly how the
  published macOS list differs from the Windows/Linux one.
- Shortcuts are ignored while a text field has keyboard focus.
- A chord bound twice in one file is an error naming the line.
- Invocations are registry commands, or front-end actions: `ui.*` (files,
  tools, paste at the pointer, notes, side panels) and `view.*` (zoom). Front-end actions do not
  change the project; the actions they lead to are commands.
- Menu items show the chord bound to their action.

## Default bindings

"Bound" names the milestone that binds the chord. Action descriptions are our
own words. The source is the v2.0 cheat sheet unless marked *(legacy)*.

### Files and editing

| Chord | Action | Bound |
| --- | --- | --- |
| Ctrl+N / Ctrl+O | New / open project | M0 |
| Ctrl+S / Ctrl+Shift+S | Save / save as | M0 |
| Ctrl+Q | Quit | M0 |
| Ctrl+Z / Ctrl+Shift+Z | Undo / redo | M0 |
| Ctrl+I | Import a file | M3 |
| Alt+Shift+L | Save machine code (G-code) | M5 |
| Alt+X | Export vectors | later (export not built yet) |
| Ctrl+P / Ctrl+Shift+P | Print (black only / keep colors) | not planned |
| Ctrl+Alt+N | Project notes window (`ui.notes`) | M2 |
| Ctrl+A / Ctrl+Shift+I | Select all / invert selection | M1 |
| Ctrl+X / Ctrl+C / Ctrl+V | Cut / copy / paste (at the pointer) | M1 |
| Alt+V | Paste in place | M1 |
| Ctrl+D | Duplicate | M1 |
| Delete, Backspace | Delete selection | M1 |
| Tab / Shift+Tab | Select the next / previous object | M1 |
| Esc | Leave the current tool, back to selection | M1 |

### Drawing, view and arrangement

| Chord | Action | Bound |
| --- | --- | --- |
| Ctrl+L / Ctrl+R / Ctrl+E | Line / rectangle / ellipse tool | M1 |
| Ctrl+= / Ctrl+- / Ctrl+0 | Zoom in / zoom out / fit the bed | M1 |
| Arrows | Nudge selection (1 mm) | M1 |
| Shift+arrows / Ctrl+arrows | Nudge in large (10 mm) / small (0.1 mm) steps | M1 |
| `.` / `,` | Rotate 90° clockwise / counter-clockwise | M1 |
| Shift+`.` / Shift+`,` | Rotate 45° clockwise / counter-clockwise | M1 |
| Ctrl+`.` / Ctrl+`,` | Rotate 15° clockwise / counter-clockwise *(legacy: 10°)* | M1 |
| Ctrl+Shift+`.` / Ctrl+Shift+`,` | Rotate 1.5° clockwise / counter-clockwise *(legacy: 5°)* | M1 |
| Ctrl+Shift+H / Ctrl+Shift+V | Mirror horizontally / vertically | M1 |
| PgUp / PgDn | One step forward / backward in drawing order | M1 |
| Ctrl+PgUp / Ctrl+PgDn | To front / to back | M1 |
| Ctrl+T | Text tool | M3 |
| Ctrl+\` | Node editing | M3 |
| Ctrl+W / Ctrl+B | Weld / boolean helper | M3 / later |
| Alt++ / Alt+- / Alt+* | Union / subtract / intersect | M3 / M3 / not bindable (below) |
| Alt+Shift+C / Ctrl+K / Alt+B | Cut shapes / trim / break apart | later / later / M3 |
| Alt+O / Ctrl+Shift+C / Ctrl+Shift+B | Offset (window) / convert to path / convert to bitmap | M3 / M3 / M7 |
| Alt+J / Alt+Shift+O / Alt+D | Join open paths / optimize / delete duplicates | M3 / later / later |
| Ctrl+G / Ctrl+U | Group / ungroup | M3 |
| Alt+←/→/↑/↓ | Align left / right / top / bottom | M3 |
| Alt+PgUp / Alt+PgDn / Alt+Home | Align centers vertically / horizontally / both | M3 |
| Alt+Shift+H / Alt+Shift+V | Move together horizontally / vertically | M3 |
| L R T B C E (canvas focus) | Align left / right / top / bottom / centers vertically / horizontally *(legacy)* | not bound (below) |
| P (canvas focus) | Move to the bed center *(legacy)* | not bound (below; Arrange → Move on Bed) |
| Ctrl+Shift+M / Ctrl+2 | Mirror across a line / two-point rotate and scale | later |
| Alt+M | Measure | later |
| Alt+Shift+W | Toggle outline / filled display (with the fill preview) | M4 |
| F12 | Toggle side panels (`ui.toggle_panels`) | M2 |
| F1 | Help for the item under the pointer | M10 |

### Machine (all need a connected device)

| Chord | Action | Bound |
| --- | --- | --- |
| Ctrl+Tab | Add tabs | M4 |
| Alt+P | Preview the job | M4 |
| Alt+L | Move the head to a clicked point | M6 |
| Ctrl+Shift+[ / Ctrl+Shift+] | Jog down / up | M6 |
| Ctrl+Alt+[ / Ctrl+Alt+] | Jog left / right | M6 |
| Ctrl+H | Home (on macOS this chord is also the system "hide app") | M6 |
| Alt+S / Alt+Shift+S | Start the job / send it without starting | M6 |
| Alt+R / Alt+Shift+R | Frame the job / frame its outline | M6 |
| Ctrl+Shift+A | Frame the selection *(legacy)* | M6 |
| Pause / Ctrl+Break *(legacy)* | Pause / stop; also always-visible buttons | M6 |
| Alt+C | Capture a camera image | M9 |
| Ctrl+Shift+R / Ctrl+Shift+F | Rotary / feeder setup | M9 |
| Alt+0 … Alt+4 | Print-and-cut steps | M9 |

### Mouse and modifiers (M1)

- Click selects; Shift+click adds; Ctrl+click toggles.
- Dragging a box from left to right selects objects fully inside it; from
  right to left selects every object it touches.
- Mouse wheel zooms about the pointer. Pan with the middle button, or hold
  Space and drag *(legacy)*. The v2.0 list also pans with Shift and mouse
  movement; LaserCraft keeps Shift for selection and constraint instead.
- While dragging: Shift keeps the move horizontal, vertical or at 45°; Ctrl
  turns snapping off.
- Resizing keeps the aspect ratio unless Shift is held; Ctrl resizes from the
  center.
- Drawing a rectangle or ellipse: Ctrl draws from the center, Shift makes
  width and height equal.

M3 notes:

- `+` is Shift+= on many layouts, so union is bound as both `Alt+Plus` and
  `Alt+Shift+Plus`. egui has no key for `*` (Shift+8 on US layouts, other
  keys elsewhere), so Alt+* cannot be bound reliably; intersect is in the
  Tools menu and the console.
- The legacy single-letter canvas keys (L R T B C E P) are not bound: in the
  node tool the letters D I M L C B act on the node or segment under the
  pointer (`docs/spec/nodes.md`), and plain letters would also fire while
  typing. The Alt chords above cover alignment.
- Alt+O opens the offset window; Ctrl+I the import path prompt; Ctrl+T and
  Ctrl+\` switch to the text and node tools.

Not adopted: Ctrl+Y (not a published default). Alt+C means "capture camera
image" in v2.0 and "cut shapes" moved to Alt+Shift+C, so the legacy Alt+C
conflict is gone.

## Sources

- LightBurn 2.0 Windows hotkey cheat sheet, rev. 2.0, 13 Nov 2025:
  https://docs.lightburnsoftware.com/2.0/download/LightBurnWindowsHotkeysV2.0.pdf
- LightBurn 2.0 macOS hotkey cheat sheet:
  https://docs.lightburnsoftware.com/2.0/download/LightBurnMacHotkeysV2.0.pdf
- LightBurn documentation, "Hotkeys and Gestures" (legacy):
  https://docs.lightburnsoftware.com/legacy/Hotkeys
  (`/Hotkeys.html` redirects there).
- All read 2026-10-21. Only chord assignments and gestures are taken; action
  descriptions are our own. M3 chords are also listed on the LightBurn 2.1
  pages cited in `docs/spec/arrange.md` and `docs/spec/nodes.md` (Align,
  Boolean Tools, Grouping, Offset Shapes, Edit Nodes, Break Apart, Auto-Join). Nudge step sizes are our own choice.
- Keymap file format and `ui.*`/`view.*` actions: own design, carried over from
  MusicCraft.
