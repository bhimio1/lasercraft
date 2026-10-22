# Spec: commands, undo and batches

Milestone: M0.

## Invocations

- Every change to a project is an invocation: a command name in `area.action`
  form plus string arguments.
- Text form: arguments are separated by whitespace. An argument that is empty or
  contains whitespace, `"` or `\`, or starts with `#`, is written in double
  quotes with `\"` and `\\` escapes. Outside quotes, a token starting with `#`
  starts a comment that runs to the end of the line. A closing quote must be
  followed by whitespace or the end of the line.
- Printing an invocation and parsing the result gives the same invocation.
- Lengths are millimetres. Numbers must be finite.

## Running

- A command either succeeds and returns a message, or fails and leaves the
  project and the history exactly as they were.
- Unknown names fail with "unknown command". Wrong argument counts fail with
  the command's usage line.

## Device commands

- `device.*` commands (`docs/spec/laser.md`) never change the project and are
  not undoable. Their outcome carries a device request (`Outcome::device`)
  that the front end delivers to the device thread or the simulator.

## User data commands (M8)

- Besides the project, a session holds the user's **workshop**: device
  profiles (`docs/spec/devices.md`), open material libraries
  (`docs/spec/materials.md`) and test presets (`docs/spec/test-patterns.md`).
  Commands run with both; most only touch the project.
- `profile.*`, `library.*`, `test.preset_*` and the library half of
  `material.*` change the workshop. Workshop changes are not undoable. When
  the workshop has a folder, profile and preset changes are written to it
  at once; libraries are written by `library.save`.
- A command that changes both (for example `material.update`, which also
  updates the layers that follow the entry) records only the project part
  in undo history.
- A failing command leaves the project and the workshop as they were.

## M9 commands

- `rotary.*` (`docs/spec/rotary.md`), `registration.*`
  (`docs/spec/registration.md`) and `shape.taper` (`docs/spec/taper.md`)
  change the project and are undoable.
- `camera.*` (`docs/spec/camera.md`) change the device profile's cameras
  (workshop data, saved at once, not undoable); `camera.pattern`,
  `camera.overlay` and `camera.list` change nothing.
- `device.rotary_test`, `device.camera_pattern`,
  `device.camera_pattern_frame` and `registration.expected` are device
  commands. `registration.set_target` reads the head position (`head=`)
  that the front end adds, as for framing.

## Undo and redo

- Undoable commands return an inverse invocation that restores the previous
  state exactly (numbers are written in Rust's shortest round-trip form).
- `edit.undo` runs the newest inverse; `edit.redo` re-runs the newest undone
  command. Running a new undoable command clears the redo stack.
- `edit.undo` with nothing to undo fails with "nothing to undo"; the same for redo.
- If an inverse fails (it should not), the entry stays on its stack.
- Undo and redo both restore the project's id allocator to where it was
  before the command first ran: after an undo the project is exactly as it
  was, and a redo recreates objects with the same ids.
- Commands that replace the whole document (`project.new`, `project.load`) clear
  both stacks. `project.save` is not an edit and is not recorded.

## Action names

- `action_name(invocation)` gives the readable name of what an undoable
  command did, lower case, for the Edit menu's "Undo ‹action›": "move",
  "rotate 90° clockwise", "align left edges", "change layer max power",
  "move layer down". It never shows a command id or a raw setting key.
- A batch reads as its first line; anything without a name reads
  "changes". A test checks every undoable command in the round-trip script
  has a name of its own.
- Redo stays plain "Redo".

## Batches

- `edit.batch "<line>" "<line>" …` runs the lines in order, all or nothing.
- If one fails, the lines already run are undone newest first and the batch
  fails with that error.
- A successful batch is one undo step whose inverse is a batch of the inverses
  in reverse order. Batches may nest.
- `edit.undo`, `edit.redo` and document-replacing commands cannot be batched.

## Gestures

- Front ends may run a continuous gesture (dragging a handle) with a gesture
  key. Consecutive runs of the same command with the same key become one undo
  step whose inverse restores the state from before the gesture.

## M0 commands

| Command | Undoable | Effect |
| --- | --- | --- |
| `project.new [<width_mm> <height_mm>]` | resets history | Empty project, optionally with a work area size. |
| `project.load <path>` | resets history | Open a `.lcproj` file. |
| `project.save <path>` | no | Write a `.lcproj` file. |
| `project.rename <name>` | yes | Rename; words are joined with spaces, whitespace trimmed. |
| `project.set_notes <text>` | yes | Replace the notes (one argument; quote it). |
| `project.set_work_area <width_mm> <height_mm>` | yes | Resize the work area, 1–10 000 mm per side. |
| `project.set_origin <front-left\|front-right\|rear-left\|rear-right>` | yes | Set the bed corner that is the machine origin. |
| `edit.undo`, `edit.redo`, `edit.batch …` | — | As above. |

M1 adds the `shape.*`, `select.*` and `edit.copy|cut|paste*` commands in
`docs/spec/shapes.md`. M2 adds `layer.*` and `shape.set_layer`
(`docs/spec/layers.md`). M3 adds `project.import` (`docs/spec/import.md`),
`shape.add_text` (`docs/spec/text.md`), `shape.to_path`,
`shape.set_geometry`, `node.*`, `path.close|join|break_apart`
(`docs/spec/nodes.md`), and `group.*`, `path.union|subtract|intersect|weld|offset`
and `arrange.*` (`docs/spec/arrange.md`).

## Sources

- Own design, carried over from the owner's MusicCraft command registry.
- Action names: `docs/spec/style/naming.md` ("Undo ‹action›", own design).
