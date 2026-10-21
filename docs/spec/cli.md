# Spec: `lasercraft-cli`

Milestone: M0 (preview arrives with M4, export with M5).

## Subcommands

| Usage | Effect |
| --- | --- |
| `new <out.lcproj> [<width_mm> <height_mm>]` | Create an empty project file. |
| `run <project.lcproj> <command> [<command> …]` | Load the file, run each argument as one command line, and save back. If any command fails, nothing is written. Prints each command's message. |
| `script <script.lcs> [<project.lcproj>]` | Run a script, one command per line, starting from the given project or an empty one. Blank and `#` lines are skipped. Stops at the first error, reporting `file:line`. Prints each message and a final one-line summary. The script saves explicitly with `project.save`. |
| `show <project.lcproj> [--json]` | Print a summary (name, work area, shape count, then each shape's id, kind and bounds in drawing order), or the normalized JSON. |
| `preview <project.lcproj> [--svg <out.svg>] [--moves] [--max-speed <mm/s>] [--accel <mm/s²>] [--max-power <%>]` | Plan the job (`docs/spec/planning.md`) and print the start position, one line per layer, burn, unlit and travel distances with times, the total estimated time, the job bounds (overscan included), warnings, and whether the job fits the work area or the bounds error. `--moves` also lists every move (the planner's text dump). `--svg` writes the toolpath: the work area outline, burn moves in layer colors, unlit moves dashed orange, travel dashed red. The machine flags override the planner's default machine. A job that does not fit is reported, not an error: the refusal belongs to export and streaming. |
| `commands` | List every command's usage and summary, including `edit.*`. |
| `help` | Print usage. |

## Exit codes

- 0: success. 1: a command or file operation failed. 2: wrong arguments
  (usage is printed to stderr).

## Planned

- M5: `export <project.lcproj> <out.gcode>`: write GRBL G-code after the
  safety checks.
- M6: `stream --simulate <out.gcode>`: stream through the simulated GRBL device.

## Sources

- Own design.
