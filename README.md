# LaserCraft

A clean-room, pure-Rust app for designing and running jobs on laser cutters and
engravers, for Linux, Windows and macOS. Its workflow and default shortcuts aim
to feel familiar to people coming from LightBurn. It is an independent project,
not affiliated with or endorsed by LightBurn Software.

**Status: milestones M0–M6.** Draw shapes or import SVG and DXF; add text
from system fonts; edit nodes; combine shapes with booleans, weld and offset;
group, align, distribute and array them; put them on color layers with line
or fill settings, kerf and tabs; preview the planned job with travel moves
and a time estimate; export verified GRBL G-code; and run jobs on GRBL lasers
over USB serial: connect, home, jog, frame with the laser off, start, pause
and stop, place the job at the head or a user origin, and use the console.
Every device feature is tested against a simulated GRBL controller, which
the app also offers as a port. Everything works headlessly from the CLI.
See the milestone list in `CLAUDE.md`.

## Build and run

```sh
cargo build --release
target/release/lasercraft [project.lcproj]
target/release/lasercraft-cli help
target/release/lasercraft-cli script examples/m3-design.lcs
target/release/lasercraft-cli preview job.lcproj --svg toolpath.svg
target/release/lasercraft-cli export job.lcproj job.gcode --s-max 1000
target/release/lasercraft-cli simulate job.lcproj device.home "device.frame" \
    "device.start from=current anchor=center"
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
