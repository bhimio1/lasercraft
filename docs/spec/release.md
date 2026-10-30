# Release builds

LaserCraft ships as one program, `lasercraft` (the `lasercraft-app`
crate), packaged four ways. Everything packaging needs lives in
`packaging/`; the release workflow (`.github/workflows/release.yml`)
builds all four on GitHub's runners.

## Identity

| Item | Value |
| --- | --- |
| Name | LaserCraft |
| Application id | `io.github.bhimio1.LaserCraft` (reverse DNS of the repository's home) |
| Summary | Design and run jobs on laser cutters and engravers |
| Version | the workspace version (`Cargo.toml`), e.g. `0.1.0` |
| License | GPL-3.0-or-later |
| Icon | `packaging/icons/lasercraft.svg` (own design: a beam striking a sheet), rendered to PNG sizes at build time |
| Categories | Graphics, Engineering |

The release profile builds with thin LTO, one codegen unit and stripped
symbols. Release builds on Windows use the GUI subsystem, so no console
window opens; debug builds keep the console for logs.

## Packages

| Package | Built on | How | Notes |
| --- | --- | --- | --- |
| AppImage, `LaserCraft-<version>-x86_64.AppImage` | Linux | `packaging/linux/appimage.sh`: an AppDir (binary, desktop entry, AppStream metadata, icons, `AppRun`) turned into an AppImage by `appimagetool` | Runs on any current x86-64 desktop Linux; serial access needs the user in `uucp` or `dialout` (`docs/linux-serial.md`). |
| Flatpak bundle, `LaserCraft-<version>-x86_64.flatpak` | Linux | `packaging/flatpak/io.github.bhimio1.LaserCraft.yml` with `flatpak-builder`, then `flatpak build-bundle` | Runtime `org.freedesktop.Platform` 24.08. The manifest installs the binary built by the workflow. Permissions below. |
| Windows installer, `LaserCraft-<version>-x86_64.msi` | Windows | `packaging/windows/lasercraft.wxs` with the WiX Toolset v5 (`wix build`): per-machine install to `Program Files\LaserCraft`, a Start menu shortcut, upgrades replace older versions (fixed upgrade code) | Serial ports need no driver beyond the board's. |
| macOS disk image, `LaserCraft-<version>-universal.dmg` | macOS | `packaging/macos/dmg.sh`: a universal (Apple silicon and Intel) binary from `lipo`, a `LaserCraft.app` bundle with `Info.plist` and an `.icns` icon, then `hdiutil` | Not signed or notarized yet (Open questions); the camera usage string is in `Info.plist`. |

### Flatpak permissions

| Permission | Why |
| --- | --- |
| `--device=all` | Serial ports (the laser) and USB cameras; Flatpak has no narrower device permission for serial ports. Required by `CLAUDE.md`. |
| `--share=network` | Network cameras. |
| `--socket=wayland`, `--socket=fallback-x11`, `--share=ipc`, `--device=dri` | The window and OpenGL. |
| `--filesystem=home` | Projects, imports and exports are opened by typed path, not through a portal. |

## Release workflow

- Runs on a pushed tag `v*` (pushed by the owner; contributors and agents
  never push tags), by hand (`workflow_dispatch`), and on pull requests
  that change `packaging/` or the workflow (so packaging changes are built
  before they merge).
- Jobs: `linux` (packaging check, release build, AppImage), `flatpak`
  (the Linux binary in a Flatpak bundle, built in Flathub's builder
  image), `windows` (MSI), `macos` (DMG). Each uploads its package as a
  workflow artifact.
- On a tag, a last job creates a draft GitHub release with the four
  packages and SHA-256 sums; the owner publishes it. A plain version tag
  (`v1.2.3`) is a full release marked Latest; a tag with anything after the
  version (`v1.2.3-beta.1`, `v1.0.0-rc1`) is marked as a pre-release.

## Checks

- `packaging/check.sh` (also run by CI on Linux) checks that the version
  in the AppStream metadata's newest release matches the workspace
  version, that the desktop entry names the binary and icon, and that the
  Flatpak manifest requests `--device=all`.

## Open questions

- Code signing for Windows and notarization for macOS (needs the owner's
  certificates).
- Flathub submission (needs building from source inside the Flatpak
  builder with vendored crates).

## Sources

- Own design: identity, package layout, scripts.
- Desktop Entry Specification 1.5 (freedesktop.org); AppStream metadata
  (freedesktop.org); AppImage specification and `appimagetool`
  (AppImage project); Flatpak manifest and `build-finish` permissions
  (docs.flatpak.org); WiX Toolset v5 and its v4 schema (wixtoolset.org); Apple
  bundle `Info.plist` keys and `hdiutil` (developer.apple.com).
