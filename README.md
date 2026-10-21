# LaserCraft

A clean-room, pure-Rust app for designing and running jobs on laser cutters and
engravers, for Linux, Windows and macOS. Its workflow and default shortcuts aim
to feel familiar to people coming from LightBurn. It is an independent project,
not affiliated with or endorsed by LightBurn Software.

**Status: milestone M0.** The project model, command registry with undo/redo,
`.lcproj` files, a CLI and a minimal window exist. It does not drive a laser
yet. See the milestone list in `CLAUDE.md`.

## Build and run

```sh
cargo build --release
target/release/lasercraft [project.lcproj]
target/release/lasercraft-cli help
target/release/lasercraft-cli script examples/m0-basics.lcs
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
