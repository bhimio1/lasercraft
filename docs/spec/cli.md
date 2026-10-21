# Spec: `lasercraft-cli`

Milestone: M0 (export and preview arrive with M4/M5).

## Subcommands

| Usage | Effect |
| --- | --- |
| `new <out.lcproj> [<width_mm> <height_mm>]` | Create an empty project file. |
| `run <project.lcproj> <command> [<command> …]` | Load the file, run each argument as one command line, and save back. If any command fails, nothing is written. Prints each command's message. |
| `script <script.lcs> [<project.lcproj>]` | Run a script, one command per line, starting from the given project or an empty one. Blank and `#` lines are skipped. Stops at the first error, reporting `file:line`. Prints each message and a final one-line summary. The script saves explicitly with `project.save`. |
| `show <project.lcproj> [--json]` | Print a summary (name, work area, shape count, then each shape's id, kind and bounds in drawing order), or the normalized JSON. |
| `commands` | List every command's usage and summary, including `edit.*`. |
| `help` | Print usage. |

## Exit codes

- 0: success. 1: a command or file operation failed. 2: wrong arguments
  (usage is printed to stderr).

## Planned

- M4: `preview <project.lcproj>`: plan the job and print the time estimate,
  travel distance and bounds, or write a preview image.
- M5: `export <project.lcproj> <out.gcode>`: write GRBL G-code after the
  safety checks.
- M6: `stream --simulate <out.gcode>`: stream through the simulated GRBL device.

## Sources

- Own design.
