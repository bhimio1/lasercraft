# Spec: `lasercraft-cli`

Milestone: M0 (preview arrives with M4, export with M5, simulate with M6).

## Subcommands

| Usage | Effect |
| --- | --- |
| `new <out.lcproj> [<width_mm> <height_mm>]` | Create an empty project file. |
| `run <project.lcproj> <command> [<command> …]` | Load the file, run each argument as one command line, and save back. If any command fails, nothing is written. Prints each command's message. |
| `script <script.lcs> [<project.lcproj>]` | Run a script, one command per line, starting from the given project or an empty one. Blank and `#` lines are skipped. Stops at the first error, reporting `file:line`. Prints each message and a final one-line summary. The script saves explicitly with `project.save`. |
| `show <project.lcproj> [--json]` | Print a summary (name, work area, shape count, then each shape's id, kind and bounds in drawing order, and `group a/b` for grouped shapes, outermost first), or the normalized JSON. |
| `export <project.lcproj> <out.gcode> [--s-max <n>] [--air m7\|m8] [--finish] [--max-speed <mm/s>] [--accel <mm/s²>] [--max-power <%>]` | Run `project.export_gcode` (`docs/spec/gcode.md`): plan, refuse jobs that leave the work area or burn nothing, write verified GRBL G-code, and print its line count, burn length, highest `S` and estimated time. `--s-max` must equal GRBL's `$30`. Refusals exit with code 1 and write nothing. |
| `preview <project.lcproj> [--svg <out.svg>] [--moves] [--max-speed <mm/s>] [--accel <mm/s²>] [--max-power <%>]` | Plan the job (`docs/spec/planning.md`) and print the start position, one line per layer, burn, unlit and travel distances with times, the total estimated time, the job bounds (overscan included), warnings, and whether the job fits the work area or the bounds error. `--moves` also lists every move (the planner's text dump). `--svg` writes the toolpath: the work area outline, burn moves in layer colors, unlit moves dashed orange, travel dashed red. The machine flags override the planner's default machine. A job that does not fit is reported, not an error: the refusal belongs to export and streaming. |
| `simulate <project.lcproj> [--locked] [--user-origin X,Y] <command> [<command> …]` | Connect the real streaming controller to the simulated GRBL (`docs/spec/device.md`) on simulated time, with the work area as its travel, and run each argument as a command line, typically `device.*` (`docs/spec/laser.md`). The head position, user origin and `$30` are filled in as the app does. Each request plays out before the next command: streams until they finish, everything else until the machine rests. Prints each command's message, then stream outcomes, refusals, alarms and non-routine replies, and finally the laser-on length and stretch count, head position, state and simulated time. `--locked` starts the machine in alarm (homing required). A failing command exits with 1; the project file is never written. |
| `commands` | List every command's usage and summary, including `edit.*`. |
| `help` | Print usage. |

## Exit codes

- 0: success. 1: a command or file operation failed. 2: wrong arguments
  (usage is printed to stderr).

## Sources

- Own design.
