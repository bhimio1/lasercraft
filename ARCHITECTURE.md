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
lasercraft-commands, lasercraft-ui ──► lasercraft-import (M3; the UI lists font families and reads its icon SVGs)
lasercraft-import ──► lasercraft-geom
lasercraft-app ──► lasercraft-plan (M4)
lasercraft-commands ──► lasercraft-gcode ──► lasercraft-plan (M5: project.export_gcode)
lasercraft-commands ──► lasercraft-device (M6: device.* commands return a Request)
lasercraft-app ──► lasercraft-device (M6: the device thread)
lasercraft-commands ──► lasercraft-workshop ──► lasercraft-project (M8: device profiles, material libraries, test presets)
lasercraft-app, lasercraft-cli ──► lasercraft-workshop (M8: the configuration folder)
lasercraft-camera ──► lasercraft-geom (M9: lens model, calibration, alignment, overlay)
lasercraft-workshop, lasercraft-commands, lasercraft-cli, lasercraft-ui, lasercraft-app ──► lasercraft-camera (M9: cameras in device profiles, camera.* commands, `camera` subcommand, Camera panel and windows, picture sources)
lasercraft-ui ──► lasercraft-workshop (M8: the Materials panel and device windows read profiles, libraries and presets; changes go through commands)

lasercraft-cli ──► lasercraft-commands, lasercraft-project, lasercraft-plan (M4)
lasercraft-cli ──► lasercraft-device (M6: simulate)

lasercraft-plan ──► lasercraft-project, lasercraft-geom (M4)
lasercraft-project ──► lasercraft-raster (M7: image bitmaps; re-exported as lasercraft_project::raster)
lasercraft-raster, lasercraft-device, lasercraft-geom: no workspace dependencies
```

Rules: `lasercraft-ui` never depends on `lasercraft-device`, `-plan` or
`-gcode` directly; it shows what the app hands it (the job preview gets plain
`lasercraft_ui::Preview` data that the app builds from a `Plan`; the test
pattern windows send `TestRequest`s that the app checks and plans into
`TestInfo`). `lasercraft-device` never
depends on `lasercraft-project`: it streams lines it is given.

| Crate | Responsibility | Status |
| --- | --- | --- |
| `lasercraft-geom` | Points, affine transforms, bounding boxes; later paths, Béziers, flattening, booleans, offsets. Pure, deterministic. | M1: `Point`, `Affine`, `Rect`, ellipse flattening, polyline queries, snapping, handle math; tracing: simplified and smoothed outlines from pixel edges |
| `lasercraft-project` | Document model and the JSON `.lcproj` format (`docs/spec/project-file.md`). Validating setters; no I/O beyond load/save. | M2: name, notes, work area, 30 layers with cut settings in output order, shapes (rect, ellipse, polyline) with transforms and a layer, hit tests, selection and clipboard as view state; compact JSON writer; M7: image shapes (format 5) and image layer settings |
| `lasercraft-commands` | `Command` trait, `Registry`, `Invocation` text form, `Session` with undo/redo, batches and coalescing (`docs/spec/commands.md`). | M3: adds `project.import`, `shape.add_text`, `shape.to_path`, `node.*`, `group.*`, `path.*`, `arrange.*` |
| `lasercraft-import` | SVG and DXF to geometry; font lookup and text outlines. | M3: SVG (usvg), DXF (own reader), system fonts (fontdb), glyph outlines (ttf-parser) |
| `lasercraft-raster` | Images: grayscale bitmaps, PNG/JPEG decoding, adjustments, dot grid and dithering (`docs/spec/images.md`). Pure, deterministic. | M7: decoding with file resolution, PNG encoding, base64, brightness/contrast/gamma, document-aligned sampling, grayscale, threshold, Floyd–Steinberg, Jarvis, Stucki, Atkinson, Bayer, halftone; tracing: ink map, clean-up and pixel-edge outlines (`docs/spec/trace.md`) |
| `lasercraft-plan` | Layers to ordered toolpaths, fills, kerf, tabs, time estimates. Pure, golden-tested (`docs/spec/planning.md`). | M4: `plan(&Project, &Machine) -> Plan` (burn/unlit/travel moves), inner-first nearest-neighbour order, kerf, automatic tabs, even-odd fill with overscan and crosshatch, trapezoidal time estimate, `Plan::check_bounds`, text dump |
| `lasercraft-gcode` | GRBL-dialect output (`docs/spec/gcode.md`): refuses off-bed and empty jobs, writes `M4`/`M3` laser-mode G-code, and checks every program with an independent verifier that replays GRBL laser rules. | M5; M6: job placement (Start From, nine-point job origin), laser-off framing (box, convex hull) |
| `lasercraft-device` | Serial connection, GRBL streaming, status polling, simulated GRBL (`docs/spec/device.md`). | M6: reply parser, I/O-free `Controller` (character counting with send-response fallback, polling, stalls, hold-then-reset stop, spindle stop on pausing `M3` jobs, laser-mode check), simulated GRBL with a laser-on log, `Bench`, device thread (serial and simulator) |
| `lasercraft-camera` | Pictures, the lens model and its calibration from pictures of the calibration sheet, alignment marks and bed alignment, overlays, a synthetic camera for tests and the simulated camera (`docs/spec/camera.md`). Pure: capturing pictures is the app's. | M9 |
| `lasercraft-workshop` | The user's data outside projects: device profiles, material libraries (`.lcmat`), test presets, the starter library, their JSON formats and the configuration folder (`docs/spec/devices.md`, `docs/spec/materials.md`, `docs/spec/test-patterns.md`). | M8 |
| `lasercraft-cli` | Headless front end (`docs/spec/cli.md`). | M2: `new`, `run`, `script`, `show` (layers and shapes), `commands`; M4: `preview` (stats, `--moves`, `--svg`, machine flags); M5: `export`; M6: `simulate` |
| `lasercraft-ui` | egui views and the keymap. Turns input into `Action`s; no business logic (`docs/spec/ui-shell.md`). | M2: shell, canvas (zoom/pan, grid, tools, gestures as a testable state machine, layer colors), layers panel and editor, palette, keymap; M4: job preview window (draws `Preview` data), kerf and tab fields; M6: Laser panel (draws `LaserView` data), status-bar Stop/Pause, alarm window; M8: Materials panel, Device row and device profile windows, test pattern windows (settings edited as JSON; `serde_json` for that), follow band in the layer editor; M9: rotary, taper and print registration windows, Camera panel, camera list and setup windows (add, lens, alignment), picture settings, Trace to vectors window, scan controls, overlay and registration targets on the canvas |
| `lasercraft-app` | eframe entry point: owns the `Session`, runs actions, tracks the file path and unsaved changes. | M0; M6: owns the device thread's handle, builds `LaserView` from device events, keeps the user origin; M8: loads the workshop from the configuration folder, answers test windows with planned patterns; M9: camera threads (USB with picture settings, network, simulated camera over the bed or on the head that sees the simulator's burns), lens fitting, mark finding, the overlay, bed scans with a head camera, and trace previews |

## Command registry

- `Command` trait: `name`, `summary`, `usage`, `execute(&mut Context, &[String])`
  where `Context` holds the project and the user's workshop (M8; before M8
  commands saw only the project). Table commands that need only the project
  keep their `fn(&mut Project, &Args)` body; workshop commands use
  `fn(&mut Context, &Args)`.
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
- Device actions (connect, jog, frame, start, pause, stop, …) are `device.*`
  commands too (M6). They never change the project and are not undoable: each
  returns a `lasercraft_device::Request` in `Outcome::device`, which the front
  end delivers (the app to its device thread, `lasercraft-cli simulate` to the
  simulator). `with_context` fills in what only the front end knows (head
  position, user origin, `$30`).

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
  long plan never freezes the UI. M4 itself plans on the UI thread, only while
  the preview window is open and the session's revision has changed.
- The streaming logic is the I/O-free `Controller` (requests, received lines
  and the time in; bytes and events out). The device thread wraps it with a
  port and a clock; tests and `lasercraft-cli simulate` wrap it with the
  simulator on simulated time (`Bench`), which is how M6 is tested without
  hardware.
- **Camera threads** (M9): the app runs one thread per camera in use. Each
  owns its source (USB, network or the simulated camera), takes pictures at
  the panel's preview rate and hands them to the UI through a channel; the
  UI never waits for a camera. Lens fitting and overlays are pure functions
  in `lasercraft-camera`.

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
  `crates/lasercraft-cli/tests/golden/` (script and preview output),
  `crates/lasercraft-plan/tests/golden/` (fixture project + machine → plan
  dump; regenerate with `LASERCRAFT_UPDATE_GOLDEN=1` and review the diff).
  G-code goldens follow in M5.
- CLI integration tests spawn the real binary.
- Device tests (M6) run against the simulated GRBL device only.
  Hardware checks are manual (`docs/hardware-testing.md`).

## Dependencies

| Dependency | Why |
| --- | --- |
| `serde`, `serde_json` (`float_roundtrip`, `preserve_order`) | The `.lcproj` JSON format (geom's `Point` and `Affine` derive it too) and JSON arguments in undo inverses, workshop files, and the test windows' settings (the UI edits them as JSON). `float_roundtrip` makes parsing bit-exact, so saving and undo never drift; `preserve_order` keeps field order for the compact file writer. |
| `i_overlay` | Pure-Rust polygon booleans and offsets (union, subtract, intersect, weld, offset, kerf); MIT. |
| `egui` | Immediate-mode UI used by `lasercraft-ui`; simple to keep free of business logic, and it runs headlessly, so UI tests drive the real shell with synthetic input. |
| `eframe` (`glow`, `x11`, `wayland`, `default_fonts`) | egui's native window and OpenGL backend for Linux (X11 and Wayland), Windows and macOS. |
| `usvg` (no default features) | SVG import: resolves CSS, `<use>`, transforms and units and turns every element into absolute lines and Béziers; text and font features are off, so no font stack is pulled in. Apache-2.0 OR MIT. |
| `fontdb` (`std`, `fs`, `fontconfig`; no `memmap`) | Finds installed fonts by family name for text (M3). MIT. |
| `ttf-parser` (`std` only) | Reads glyph outlines, advances, cap height and `kern` pairs from TrueType/OpenType fonts. MIT OR Apache-2.0. |
| `png` | Reads and writes PNG for image import and the project file's embedded bitmaps (M7). Pure Rust; already in the tree through eframe. MIT OR Apache-2.0. |
| `zune-jpeg` (`std` only, no SIMD features) | Decodes JPEG to grayscale for image import (M7). Pure Rust; already in the tree. MIT OR Apache-2.0 OR Zlib. |
| `serialport` (no default features) | Opens and lists serial ports on Linux, Windows and macOS (M6). Without its `libudev` feature it lists Linux ports from sysfs, so no C library is needed. MPL-2.0. |
| `nokhwa` (no default features; `input-native`) | Opens USB cameras through each platform's own API (V4L2, Media Foundation, AVFoundation) for the app only (M9). Its decoding features are off (they pull in a C JPEG library); the app converts frames itself with `zune-jpeg`. Apache-2.0. |
| `ureq` 2 | Fetches still pictures and MJPEG streams from network cameras over HTTP(S) in the app (M9). Small, blocking, which suits one thread per camera; rustls, no C. MIT OR Apache-2.0. |

DXF is read by our own small parser (`lasercraft-import/src/dxf.rs`): the
available crates pull in large dependency trees for a format whose ASCII
form is simple group-code pairs.


M3 notes: node editing, joining and splitting live in `lasercraft-geom`
(`nodes`); groups are a chain of group ids on each shape
(`docs/spec/arrange.md`); text is stored as outline paths
(`docs/spec/text.md`).

## Deferred decisions

- Icons (decided with the drawing tools icons): own SVG files in a
  restricted subset, embedded in `lasercraft-ui` and read with
  `lasercraft_import::svg::parse_styled`, painted with egui's painter in
  theme colors (`docs/spec/style/icons.md`, option 2). No new dependency;
  `egui_extras`' SVG loader and icon fonts were not chosen.
- File dialogs: M0 asks for a path in a text field. A native dialog crate
  (e.g. `rfd`) needs a portal on Flatpak; decide in M1.
- Whether the MCP server is a separate crate (as in MusicCraft).
- Device profiles (decided in M8): in the user configuration folder
  (`devices.json`); a project names its profile (`docs/spec/devices.md`).
