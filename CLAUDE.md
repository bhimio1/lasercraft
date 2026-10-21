# LaserCraft

A clean-room, pure-Rust app for designing and running jobs on laser cutters and
engravers. Workflow, layout and default shortcuts should feel familiar to
LightBurn users. Linux is a first-class target (LightBurn dropped it after
v1.7); Windows and macOS are supported too. Developed primarily on CachyOS.

## Read first

- `docs/clean-room.md`: binding rules about what we may and may not look at or copy.
- `ARCHITECTURE.md`: crate graph, threading model, command registry, dependencies.
- `docs/spec/`: behavior specs. **Write or update the spec before the code.**
- `docs/spec/safety.md`: binding laser safety requirements.
- `docs/spec/style/`: UI style guide (layout, names, toolbars, colors as tokens, icons).

## Clean-room rules (summary; full text in `docs/clean-room.md`)

- Work only from: LightBurn's public docs (docs.lightburnsoftware.com), behavior
  the owner describes from software they are licensed for, and open standards
  (GRBL's published docs, SVG, DXF, G-code, PNG/JPEG, USB serial).
- Never decompile, disassemble or inspect LightBurn binaries. Never use leaked source.
- Never copy LightBurn's icons, artwork, fonts, names, material library data or
  UI text. Describe behavior in our own words.
- No reading, parsing or writing `.lbrn`/`.lbrn2` unless the owner approves it.
  LaserCraft has its own format (`.lcproj`).
- Proprietary controller protocols (Ruida, Trocen, …) are out of scope until the
  owner decides which public sources are acceptable. Ask first.
- Every spec ends with a **Sources** section: a public LightBurn doc page by URL,
  an open standard, or "own design".
- If unsure whether something is allowed, stop and ask the owner.

## Safety rules (this software drives a real laser)

- The laser never fires except during a job, framing or test fire the user
  explicitly started. Framing defaults to laser-off moves.
- Every job is checked against the work area before sending; a move outside the
  bed is refused with a clear error.
- Every power value is clamped to the layer's and the device's maximum.
- Stop and Pause are always visible while connected and always work: feed hold
  `!`, soft reset `0x18`, resume `~`.
- Disconnect, stalled status replies or an alarm stop streaming and show the alarm.
- Tests never talk to real hardware: use the simulated GRBL device. Hardware
  tests are manual, marked as such, in `docs/hardware-testing.md`.
- No `unsafe` anywhere (`unsafe_code = "forbid"` workspace-wide).
- The device thread never blocks the UI. UI and device talk only through
  channels carrying fixed message types (`lasercraft_device::{Request, Event}`).

## Architecture rules

- **Every user-visible action is a command** in `lasercraft-commands`. UI,
  shortcuts, CLI and (later) MCP only invoke commands. Nothing else mutates the
  project. Undoable commands return their inverse invocation.
- Keyboard shortcuts are data: `crates/lasercraft-ui/keymap/default.keymap`.
  Defaults follow LightBurn's published hotkeys (`docs/spec/keymap.md`).
- The UI crate contains no business logic.
- Geometry and planning are pure and deterministic; prefer golden-file tests
  (input SVG + layer settings → expected G-code).
- Adding a dependency needs a one-line justification in `ARCHITECTURE.md`.
- Linux: document serial permissions (`uucp` on Arch, `dialout` on
  Debian/Ubuntu), detect "permission denied" and explain the fix in-app, and the
  Flatpak must request serial device access (`docs/linux-serial.md`).

## Quality gates (must pass before a session ends)

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI (`.github/workflows/ci.yml`) runs these on Linux and `cargo check` on Windows
and macOS.

## Working style

- One milestone per session. Plan first, write or update the spec, then code.
- Add a test for every command and every planner and G-code feature.
- Verify headlessly through `lasercraft-cli` and the simulator before building UI.
- Keep functions small and crates focused.
- Commit with clear messages, push to a branch, open a PR. Never push tags.

## File editing protocol

1. Use the native `Edit` tool first for localized changes.
2. Only if it fails on complex whitespace or quoting, use a precise
   `python3 - <<'EOF'` replacement through Bash, after stating the error.

## Milestones

- [x] M0 Workspace, docs, command registry, CLI, project save/load, undo/redo.
- [x] M1 Canvas with bed grid; shapes (rect, ellipse, line, polyline); select,
      move, scale, rotate; snapping; zoom/pan; LightBurn-style default shortcuts.
- [x] M2 Layers ("cuts") with colors; line, fill and image modes; speed,
      power min/max, passes, interval, output on/off; cuts/layers panel.
- [x] M3 SVG and DXF import; node editing; booleans (union, subtract,
      intersect); offset, weld, group/ungroup, align/distribute, arrays (grid,
      circular); text from system fonts.
- [x] M4 Toolpath planning: cut ordering (inner first, shortest travel); fill
      scanlines with overscan; kerf offset; tabs/bridges; preview with travel
      moves and time estimate.
- [x] M5 GRBL G-code export (M3/M4 dynamic power, S-max).
- [ ] M6 Device connection and GRBL streaming: character counting with
      ping-pong fallback, status polling; jog, home, set origin, job origin
      modes (absolute, current position, user origin); framing,
      start/pause/stop, console. All simulator-tested.
- [ ] M7 Images: grayscale, threshold, Floyd–Steinberg, Jarvis, Stucki,
      ordered/Bayer, halftone; brightness/contrast/gamma, negative; DPI/line interval.
- [ ] M8 Material library (our format, our data), test grids (speed × power),
      per-device profiles.
- [ ] M9 Camera overlay and alignment, rotary, print-and-cut (later, optional).
- [ ] M10 Parity checklist (`cargo xtask parity`) against the public docs;
      Flatpak, AppImage, .msi and .dmg release builds.
