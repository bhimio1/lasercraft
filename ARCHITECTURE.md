# LaserCraft architecture

## Goals

- Pure Rust, no `unsafe`, cross-platform (Linux first-class, Windows, macOS).
- Headless-testable: everything the app can do, `lasercraft-cli` can do.
- A single command registry is the only way to change a project. UI, shortcuts,
  CLI and a later MCP server are thin front ends over it.
- Geometry and planning are pure functions; device I/O is isolated on its own
  thread behind fixed message types.
- Safety checks (`docs/spec/safety.md`) sit in the pipeline, not in the UI, so
  every front end gets them.

## Crate graph

Solid edges exist today; `┄►` edges are planned and added by the milestone
named in brackets.

```
lasercraft-app ──► lasercraft-ui ──► lasercraft-commands ──► lasercraft-project ──► lasercraft-geom
lasercraft-ui, lasercraft-commands ──► lasercraft-geom
lasercraft-commands, lasercraft-ui ──► lasercraft-import (M3; the UI only lists font families)
lasercraft-import ──► lasercraft-geom
lasercraft-app ┄► lasercraft-device [M6], lasercraft-plan, lasercraft-gcode [M4/M5]

lasercraft-cli ──► lasercraft-commands, lasercraft-project
      └┄► lasercraft-plan, lasercraft-gcode, lasercraft-device (simulator) [M4–M6]

lasercraft-plan ┄► lasercraft-project, lasercraft-geom, lasercraft-raster [M4]
lasercraft-gcode ┄► lasercraft-plan [M5]
lasercraft-raster, lasercraft-device, lasercraft-geom: no workspace dependencies
```

Rules: `lasercraft-ui` never depends on `lasercraft-device`, `-plan` or
`-gcode` directly; it shows what the app hands it. `lasercraft-device` never
depends on `lasercraft-project`: it streams lines it is given.

| Crate | Responsibility | Status |
| --- | --- | --- |
| `lasercraft-geom` | Points, affine transforms, bounding boxes; later paths, Béziers, flattening, booleans, offsets. Pure, deterministic. | M1: `Point`, `Affine`, `Rect`, ellipse flattening, polyline queries, snapping, handle math |
| `lasercraft-project` | Document model and the JSON `.lcproj` format (`docs/spec/project-file.md`). Validating setters; no I/O beyond load/save. | M2: name, notes, work area, 30 layers with cut settings in output order, shapes (rect, ellipse, polyline) with transforms and a layer, hit tests, selection and clipboard as view state; compact JSON writer |
| `lasercraft-commands` | `Command` trait, `Registry`, `Invocation` text form, `Session` with undo/redo, batches and coalescing (`docs/spec/commands.md`). | M3: adds `project.import`, `shape.add_text`, `shape.to_path`, `node.*`, `group.*`, `path.*`, `arrange.*` |
| `lasercraft-import` | SVG and DXF to geometry; font lookup and text outlines. | M3: SVG (usvg), DXF (own reader), system fonts (fontdb), glyph outlines (ttf-parser) |
| `lasercraft-raster` | Image modes and dithering. | Empty until M7 |
| `lasercraft-plan` | Layers to ordered toolpaths, fills, kerf, tabs, time estimates. Pure, golden-tested. | Empty until M4 |
| `lasercraft-gcode` | GRBL-dialect output, with work-area and power checks. | Empty until M5 |
| `lasercraft-device` | Serial connection, GRBL streaming, status polling, simulated GRBL (`docs/spec/device.md`). | M0: message types, GRBL real-time bytes, state names, Linux permission help |
| `lasercraft-cli` | Headless front end (`docs/spec/cli.md`). | M2: `new`, `run`, `script`, `show` (layers and shapes), `commands` |
| `lasercraft-ui` | egui views and the keymap. Turns input into `Action`s; no business logic (`docs/spec/ui-shell.md`). | M2: shell, canvas (zoom/pan, grid, tools, gestures as a testable state machine, layer colors), layers panel and editor, palette, keymap |
| `lasercraft-app` | eframe entry point: owns the `Session`, runs actions, tracks the file path and unsaved changes. | M0 |

## Command registry

- `Command` trait: `name`, `summary`, `usage`, `execute(&mut Project, &[String])`.
- An `Invocation` is a command name plus string arguments. A shortcut, menu
  item, CLI line, script line and MCP tool call all become one. Its text form
  supports quoting, so paths and JSON can be arguments.
- `execute` returns an `Outcome`: a message, an optional **inverse
  invocation**, and a `reset_history` flag (new/load). On error the project is
  unchanged.
- `Session` owns the project, registry and undo/redo stacks of invocations.
  `edit.undo`, `edit.redo` and `edit.batch` are handled by `Session` itself;
  batches are all-or-nothing and undo as one step. `run_coalesced` merges a
  continuous gesture (a drag) into one undo step.
- Redo restores the project's id allocator to its value when the command first
  ran, so recreated objects get the ids later history entries refer to.
- Naming: `area.action` (`project.set_work_area`). Lengths are millimetres.
- Device actions (connect, jog, frame, start, pause, stop) will be commands too
  (M6), but they are never undoable, and they act on the device thread through
  `Request` messages rather than on the project.

## Threading model

- **UI/main thread** owns the `Session` (project, registry, history) and the
  egui state. Commands run here, synchronously; they are fast and do no
  device I/O.
- **Device thread** (M6) owns the serial port (or the simulator) and the
  streaming state machine. It is the only code that reads or writes the port.
- The two talk only through two `std::sync::mpsc` channels carrying the fixed
  message enums `lasercraft_device::Request` (UI → device) and
  `lasercraft_device::Event` (device → UI). No shared mutable state, no locks
  held across I/O. The UI drains events with `try_recv` once per frame and
  never blocks; the device thread asks egui for a repaint when it sends one.
- Pause and Stop are `Request`s that the device thread handles before any
  queued streaming work, and they go out as GRBL real-time bytes (`!`, `0x18`),
  which GRBL acts on even mid-line and with a full receive buffer.
- **Planning** (M4) may run on a worker thread for large jobs. It takes an
  immutable snapshot of the project and returns a plan through a channel, so a
  long plan never freezes the UI.
- The CLI uses the same channels with the simulated device, which is how M6 is
  tested without hardware.

## Testing strategy

- Unit tests for every command, including error paths. A round-trip test runs
  every undoable command, then undoes, redoes and undoes again, comparing the
  project with snapshots; another test fails when a new command is not
  classified into it.
- Canvas gestures are a state machine over plain data
  (`lasercraft_ui::interaction`) with unit tests; `crates/lasercraft-ui/tests/`
  drives the whole shell through egui headlessly with synthetic pointer and
  key events and applies the emitted commands to a real session.
- Golden files: `crates/lasercraft-project/tests/golden/` (file format),
  `crates/lasercraft-cli/tests/golden/` (script output). Planner and G-code
  goldens follow in M4/M5.
- CLI integration tests spawn the real binary.
- Device tests (M6) run against the simulated GRBL device only.
  Hardware checks are manual (`docs/hardware-testing.md`).

## Dependencies

| Dependency | Why |
| --- | --- |
| `serde`, `serde_json` (`float_roundtrip`, `preserve_order`) | The `.lcproj` JSON format (geom's `Point` and `Affine` derive it too) and JSON arguments in undo inverses. `float_roundtrip` makes parsing bit-exact, so saving and undo never drift; `preserve_order` keeps field order for the compact file writer. |
| `i_overlay` | Pure-Rust polygon booleans and offsets (union, subtract, intersect, weld, offset, kerf); MIT. |
| `egui` | Immediate-mode UI used by `lasercraft-ui`; simple to keep free of business logic, and it runs headlessly, so UI tests drive the real shell with synthetic input. |
| `eframe` (`glow`, `x11`, `wayland`, `default_fonts`) | egui's native window and OpenGL backend for Linux (X11 and Wayland), Windows and macOS. |

| `usvg` (no default features) | SVG import: resolves CSS, `<use>`, transforms and units and turns every element into absolute lines and Béziers; text and font features are off, so no font stack is pulled in. Apache-2.0 OR MIT. |
| `fontdb` (`std`, `fs`, `fontconfig`; no `memmap`) | Finds installed fonts by family name for text (M3). MIT. |
| `ttf-parser` (`std` only) | Reads glyph outlines, advances, cap height and `kern` pairs from TrueType/OpenType fonts. MIT OR Apache-2.0. |

DXF is read by our own small parser (`lasercraft-import/src/dxf.rs`): the
available crates pull in large dependency trees for a format whose ASCII
form is simple group-code pairs.

Planned, each to be justified here when added: `serialport` (M6, serial I/O),
`image` (M7).

M3 notes: node editing, joining and splitting live in `lasercraft-geom`
(`nodes`); groups are a chain of group ids on each shape
(`docs/spec/arrange.md`); text is stored as outline paths
(`docs/spec/text.md`).

## Deferred decisions

- File dialogs: M0 asks for a path in a text field. A native dialog crate
  (e.g. `rfd`) needs a portal on Flatpak; decide in M1.
- Whether the MCP server is a separate crate (as in MusicCraft).
- Where device profiles live (M8): project file, user config, or both.
