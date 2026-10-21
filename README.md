# LaserCraft

A clean-room, pure-Rust app for designing and running jobs on laser cutters and
engravers, for Linux, Windows and macOS. Its workflow and default shortcuts aim
to feel familiar to people coming from LightBurn. It is an independent project,
not affiliated with or endorsed by LightBurn Software.

**Status: milestones M0–M4.** Draw shapes or import SVG and DXF; add text
from system fonts; edit nodes; combine shapes with booleans, weld and offset;
group, align, distribute and array them; put them on color layers with line
or fill settings, kerf and tabs; and preview the planned job with travel
moves and a time estimate. Everything is undoable and works headlessly from
the CLI. It does not drive a laser yet (G-code export is M5, the device M6).
See the milestone list in `CLAUDE.md`.

## Build and run

```sh
cargo build --release
target/release/lasercraft [project.lcproj]
target/release/lasercraft-cli help
target/release/lasercraft-cli script examples/m3-design.lcs
target/release/lasercraft-cli preview job.lcproj --svg toolpath.svg
```

On Linux the window needs `libxkbcommon-x11` and an OpenGL driver at runtime;
desktop installs normally have both. Serial access for controllers is covered
in `docs/linux-serial.md`.

## Documents

- `CLAUDE.md`: rules, quality gates, milestones.
- `ARCHITECTURE.md`: crates, threading, dependencies.
- `docs/clean-room.md`: provenance rules.
- `docs/spec/`: behavior specs, each with its sources.
- `docs/hardware-testing.md`: manual checks on real machines.

## License

GPL-3.0-or-later (`LICENSE`).
