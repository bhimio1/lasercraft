# Spec: device profiles

Milestone: M8. A device profile describes one machine: its bed, its
connection and the limits planning and G-code must respect. Profiles are
user data kept outside projects; a project names the profile it is made
for. LaserCraft drives GRBL over USB serial only, so every profile is a
GRBL profile.

## Fields

| Key | Meaning | Default | Valid |
| --- | --- | --- | --- |
| `name` | Shown in lists; unique (case-insensitive) | — | 1–60 characters, not only spaces |
| `width`, `height` | Work area, mm | 400, 400 | as `project.set_work_area` |
| `origin` | Machine origin corner: `front-left`, `front-right`, `rear-left`, `rear-right` | `front-left` | |
| `port` | Serial port to connect to; empty means "choose in the Machine panel" | empty | any text |
| `baud` | Baud rate | 115200 | 1–10 000 000 |
| `max_speed` | Top speed, mm/s (planning.md `max_speed`) | 100 | 1–10 000 |
| `acceleration` | mm/s² (planning.md `acceleration`) | 1000 | 1–100 000 |
| `max_power` | Device power ceiling, % (safety rule 6) | 100 | 0–100 |
| `s_max` | Full-power S value, must equal GRBL's `$30` | 1000 | 1–100 000 |
| `air` | Air assist code, `m7` or `m8` | `m8` | |

- One profile may be the **default**: it is chosen for new projects.
- Z axis, pointer offset, start/end G-code, scan shift correction, the
  separate travel speed and fill acceleration and the time-estimate scale
  of `docs/spec/style/machine-panels.md` are later; they are not stored yet.

## Where profiles live

- In the user's configuration folder, file `devices.json`:
  `{"format": "lasercraft-devices", "version": 1, "default": <name or
  null>, "profiles": [<profile>, …]}`, each profile an object with the keys
  above. Unknown keys are ignored when reading; a newer `version` is
  refused with a message.
- The configuration folder is `$LASERCRAFT_CONFIG_DIR` when set; otherwise
  `$XDG_CONFIG_HOME/lasercraft` or `~/.config/lasercraft` on Linux,
  `~/Library/Application Support/LaserCraft` on macOS and
  `%APPDATA%\LaserCraft` on Windows. The CLI takes `--config <dir>`;
  without it the CLI keeps user data in memory only, so scripts and tests
  never touch the user's files.
- Every profile command that changes something writes `devices.json` at
  once (there is no Save for profiles). A write failure is reported and
  the change stays in memory.
- Profile files for exchange (`.lcdevice`) hold one profile:
  `{"format": "lasercraft-device", "version": 1, "profile": {…}}`.

## The project's device

- A project names its device profile (`device` in the project file,
  `docs/spec/project-file.md`), or none.
- `project.set_device <name|none>` sets it and copies the profile's work
  area and origin into the project. It is undoable and restores both.
- New projects (`project.new`, and the app's first project) use the
  default profile when there is one.
- Planning, the job preview, `project.export_gcode` and `device.start` /
  `device.frame` / `device.test` use the named profile's `max_speed`,
  `acceleration`, `max_power`, `s_max` and `air`. Flags on a command still
  override them. With no profile, or one that does not exist, the planner
  defaults of `docs/spec/planning.md` and the export defaults apply; a
  missing profile is reported once as a warning in the preview.
- `s_max` reported by the controller (`$30`) still wins while connected
  (`docs/spec/laser.md`).

## Commands

None of these change the project except `project.set_device`; they are not
undoable (they change user data, which is saved at once).

| Command | Effect |
| --- | --- |
| `profile.add <name> [key=value …]` | Add a profile with the defaults above, then the given keys. The first profile becomes the default. |
| `profile.set <name> <key> <value>` | Change one field. |
| `profile.rename <old> <new>` | Rename; keeps the default mark. |
| `profile.duplicate <name> <new>` | Copy a profile under a new name. |
| `profile.delete <name>` | Remove it; if it was the default, no profile is default. |
| `profile.default <name\|none>` | Set or clear the default. |
| `profile.import <path>` | Add the profile in a `.lcdevice` file; a clashing name gets " (2)", " (3)", … |
| `profile.export <name> <path>` | Write one profile as `.lcdevice`. |
| `profile.list` | One line per profile: name, size, origin, port; the default marked. |
| `project.set_device <name\|none>` | As above; undoable. |

## In the app

- Machine panel, Device row (`docs/spec/style/machine-panels.md`, "9.
  Device row"): a "Device" dropdown listing the profiles ("‹name›" then
  "‹w› × ‹h› mm" in `text-muted`, the default with a "default" tag) and
  "Device profiles…". Choosing a profile runs `project.set_device` and
  fills the connection row's port and baud rate from the profile. Disabled
  while connected.
- Machine menu: "Device profiles…" and "Edit device profile…".
- **Device profiles** dialog: the list (two lines per profile), a summary
  of the selected one, buttons "New profile…", "Import…" and, for the
  selection, "Set as default", "Edit…", "Duplicate", "Remove", "Export…";
  footer "Close". Changes apply at once (profiles have no Save, so the
  style guide's OK/Cancel pair becomes Close: own decision, recorded in
  the Open questions). Remove asks "Remove device profile?" with "Remove
  profile" (`danger`) and "Cancel".
- **Device profile** dialog ("Device profile: ‹name›", or "New device
  profile"): tabs "Work area" (Width (mm), Height (mm), Origin corner as
  four choice dots), "Options" (Port, Baud rate, Full-power S value ($30),
  Max power (%), Air assist code M7 / M8) and "Motion" (Max speed (mm/s),
  Acceleration (mm/s²), with the note that they only time the estimate).
  OK applies every changed field as `profile.*` commands; Cancel runs
  nothing. While connected, Port and Baud rate are read-only.
- The window title is "• ‹project› — ‹device profile› — LaserCraft" when
  the project names a profile.

## Open questions

- Device scan and the New device profile wizard of the style guide are
  later; "New profile…" opens the Device profile dialog with defaults.
- Whether a project opened with another device's profile should offer to
  switch (style guide "Switch device profile?").

## Sources

- Device list, choosing the active device, default device, per-device work
  area, origin, speeds and power limits:
  https://docs.lightburnsoftware.com/2.1/Reference/Devices/ and
  https://docs.lightburnsoftware.com/2.1/Reference/DeviceSettings/BasicSettings/
  (read 2026-10-22).
- GRBL settings `$30`, `$110`/`$111`, `$120`/`$121`:
  https://github.com/gnea/grbl/wiki/Grbl-v1.1-Configuration
- Own design: the file formats, the configuration folder, the command set,
  saving at once, the Close-only dialog.
