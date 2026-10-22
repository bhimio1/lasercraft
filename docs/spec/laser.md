# Spec: running jobs — laser panel, job origin, framing, jogging, console

Milestone: M6. Builds on `docs/spec/device.md` (connection and streaming),
`docs/spec/gcode.md` (program text) and `docs/spec/safety.md`.

## Device commands

Every device action is a registry command named `device.*`, so the UI,
shortcuts, console, CLI and a later MCP server all go through the same code.
They never change the project and are not undoable. A device command returns
its `Request` in the outcome (`Outcome::device`); the front end hands it to
the device thread (the app) or to the simulator (the CLI). Building the
program is pure library code (`lasercraft_gcode::{export, frame}`).

| Command | Request |
| --- | --- |
| `device.connect <port> [baud]` | `Connect` (default 115200; `simulator` for the simulated machine) |
| `device.disconnect` | `Disconnect` |
| `device.start [placement] [s_max=] [air=] [max_speed=] [accel=] [max_power=]` | `Stream { Job }` |
| `device.frame [outline] [placement] [speed=<mm/s>] [head=X,Y]` | `Stream { Frame }` |
| `device.pause` / `device.resume` / `device.stop` | `Pause` / `Resume` / `Stop` |
| `device.jog <up\|down\|left\|right> <mm> [speed=<mm/s>]` | `Jog`, relative |
| `device.move_to <x> <y> [speed=<mm/s>]` | `Jog`, absolute machine position |
| `device.home` / `device.unlock` | `Home` / `Unlock` |
| `device.send <line>` | `Console` |

Placement arguments: `from=absolute|current|user` (default absolute),
`at=X,Y` (the head position or user origin, machine mm; required unless
absolute) and `anchor=<name>` (default `bottom-left`). The app fills in `at`
(from the last status report or the stored user origin), `head` and `s_max`
(from `$30`) when the user did not give them. Default speed: 50 mm/s.

## Coordinates

- Machine coordinates are GRBL's work coordinates (`G54`), in mm, measured
  from the work area's origin corner as in `docs/spec/gcode.md`. LaserCraft
  never writes work offsets (`G10`, `G92`); the user origin is kept by the
  app.
- Positions shown in the panel are the last reported machine coordinates.
- "Up" on screen is towards the rear of the bed. Jog arrows move the head
  the way the arrow points on screen, whatever the origin corner.

## Start From and job origin

Where the job runs (the "Start From" setting, kept per app session):

| Mode | The job is placed… |
| --- | --- |
| Absolute | where it is drawn on the bed (M5 behavior). |
| Current position | so that the job origin point sits at the head's position when Start or Frame is pressed. The head returns there at the end. |
| User origin | so that the job origin point sits at the user origin. |

- The **job origin** is one of nine points of the job's box (the smallest
  rectangle around everything that burns): the four corners, the four edge
  midpoints or the center. It is chosen with a 3 × 3 grid of buttons that is
  disabled in Absolute mode. Default: bottom left (front left on screen).
- **Set origin** stores the head's current position as the user origin;
  **Clear origin** forgets it; **Go to origin** moves the head there with a
  laser-off jog.
- Current position needs a known head position (a status report since
  connecting); user origin needs a stored origin. Otherwise Start and Frame
  are refused with a message saying what is missing.
- The placed job is checked against the work area exactly like an absolute
  one (rule 5): overscan, kerf and the return move included. A job that does
  not fit is refused before anything is sent.
- Implementation: `Options::placement` in `lasercraft-gcode`. The plan is
  shifted in document space, then checked and written as usual.
- Every job starts with a `G0` to its first point, even when the plan begins
  with a cut there: the head may be anywhere when a job starts.

## Framing

- **Frame** traces the job box; **Frame outline** traces the convex hull of
  everything that burns ("rubber band"). Both use the job placement above.
- The frame program contains no `M3`/`M4`: only `G0` to the first corner,
  `G1 … F` around the shape at the framing speed, back to the first corner,
  then, when the head position is known, `G0` back to where the head was, and
  `M5 S0`. The laser cannot fire (rule 2). It is verified like a job and must
  measure zero burn length.
- In Current position mode a job's last move is the `G0` back to the start,
  before the closing `M5 S0`.
- Framing speed is the panel's move speed (also used by jogging).
- A laser-on framing option is not offered in M6.

## Jogging and homing

- Arrow buttons and shortcuts jog by the panel's distance (default 10 mm,
  0.01–1000) at its speed (default 50 mm/s, 1–machine max) using `$J=` with
  relative moves; jogs never turn the laser on (rule 3).
- A jog that would leave the work area (known position, plus the distance)
  is shortened to the edge; with no known position it is sent unchanged and
  GRBL's own soft limits apply.
- **Home** sends `$H`. **Unlock** sends `$X` (shown with the alarm, with a
  warning that the position may be wrong).
- Moving actions are refused while a stream runs or the machine is not idle
  (`docs/spec/device.md`).

## Start, pause, stop

- **Start** builds the program with `lasercraft_gcode::export` (S-max from
  the controller's `$30` when known, otherwise the export default), then
  streams it. Every refusal (bounds, nothing to burn, laser mode off, not
  idle, missing position) is shown and nothing is sent.
- **Pause / Resume** is one button; **Stop** next to it. Both are visible and
  enabled whenever a device is connected, in the Machine panel and in the
  job bar (rule 7). When the running job uses `M3`, the pause button reads
  "Pause (motion only)", the job bar says "The laser may stay on", and the
  tooltip says LaserCraft switches the laser off once the head has stopped.
- Progress: elapsed time (h:mm:ss), a bar, and "‹n›% sent" (lines accepted
  by the controller; the machine may still be working). After a stream:
  "Job finished in ‹h:mm:ss›.", "Job stopped at ‹n›% sent." or "Job stopped
  by a fault at ‹n›% sent.". The plan's remaining-time estimate is later.

## Console

- Shows every line sent (`>` prefix) and received, newest last, limited to the
  last 500 lines. "Show all messages" also shows status reports and polling
  (in muted text, sent ones with `→`), which are hidden by default. Not
  connected, it says "Not connected. Messages from GRBL appear here once you
  connect."
- A text field sends a line to the controller (`Console` request; refused
  while a stream runs). This is the user's explicit action; the console does
  not filter G-code.

## Alarms and faults

- An `Alarm` shows a band in the Machine panel: "Alarm ‹n›: ‹meaning›.
  Streaming stopped. …, then click Clear alarm." with Clear alarm (`$X`),
  Home (`$H`) and Show in console. It stays until GRBL leaves the alarm
  state; the status bar's Messages field repeats it.
- A failed stream, a stall and a connection lost during a job show a fault
  band with Show in console and Hide, and a line in the log. A refused
  Start or Frame shows "Cannot run: ‹reason›" with Show in job preview and
  Hide. Permission denied shows the Serial port access help with Try again
  and Hide.
- Nothing offers Resume after an alarm or a fault.

## Panel layout

Three dock panels (`docs/spec/style/machine-panels.md`):

- **Machine** (lower dock group): state line (dot, state word, "on ‹port›
  at ‹baud› baud"), progress; Pause / Resume, Stop, Start; Frame, Frame
  outline, Save G-code…; bands (constant power, refused job, alarm, fault,
  serial port access); Home, Go to user origin; "Place job at" (Absolute
  coordinates, Current position, User origin) and the "Job reference point"
  3 × 3 grid; connection row (Connect / Disconnect, Port, Baud rate,
  Refresh ports).
- **Jog** (upper group tab): Head position read-out (click copies it into
  Move head to), Move head to X (mm) / Y (mm) / Go, the jog pad (arrows
  around Home), Jog distance (mm), Jog speed (mm/s), Set user origin, Clear
  user origin. Jog arrows move the head in screen directions, so their
  tooltips say "Jog up" and so on rather than an axis sign.
- **Console** (upper group tab): input line ("Type a GRBL command"), the
  "Show all messages" switch and the log.

State words, derived by the app from the connection, GRBL's state and its
own stream: Not connected, Connecting…, Ready, Running, Framing, Paused,
Jogging, Homing, Alarm, Door open, Checking, Sleeping, No response. One
rule (`Enabled::of`) decides which controls work, for the panels, the job
bar and the Machine menu: Pause and Stop whenever connected; Start and
Frame when ready with an output layer that has shapes; Home when ready or
in alarm; the jog pad when ready or jogging; Clear alarm only in alarm;
Disconnect not while a stream runs.

Names: "Start From" is now "Place job at", the job origin grid "Job
reference point", "Set / Clear / Go to origin" "Set / Clear / Go to user
origin", "Unlock" "Clear alarm" (command `device.unlock` unchanged).

Open owner decisions, not applied: jogs past the bed shortened (today) or
refused; a homing requirement for Go and Go to user origin; refusing
laser-on console lines.

## Shortcuts

Shortcuts are bound to front-end actions that fill in the panel's settings
and run the command:

| Keys | Action | Binding |
| --- | --- | --- |
| Alt+S | Start | `ui.device_start` |
| Alt+R | Frame | `ui.device_frame` |
| Alt+Shift+R | Frame outline | `ui.device_frame_outline` |
| Pause | Pause / resume | `ui.device_pause` |
| Ctrl+Pause | Stop | `device.stop` |
| Ctrl+H | Home | `device.home` |
| Ctrl+Shift+] / Ctrl+Shift+[ | Jog up / down | `ui.jog_up`, `ui.jog_down` |
| Ctrl+Alt+[ / Ctrl+Alt+] | Jog left / right | `ui.jog_left`, `ui.jog_right` |

## Command line

`lasercraft-cli simulate <project.lcproj> [--locked] [--user-origin X,Y]
<command> …` runs device commands against the simulated GRBL through the
real controller (`docs/spec/cli.md`). Only the simulator is reachable from
the CLI in M6; real ports are used from the app.

## Sources

- Laser window: Start, Stop, Pause/Resume, the Pause key and Ctrl+Break, two
  frame buttons (box and rubber band), framing speed from the move speed,
  laser-on framing off by default, Home, Go to origin, device status and
  progress: https://docs.lightburnsoftware.com/2.1/Reference/LaserWindow/
- Start From modes, the nine-point job origin on the job's bounding box,
  current position returning to the start, user origin set from the head
  position: https://docs.lightburnsoftware.com/2.1/Reference/CoordinatesOrigin/
- Framing with each Start From mode:
  https://docs.lightburnsoftware.com/2.1/GetStarted/FramingBeginner/
- Move window: jog arrows with distance and speed, Set/Clear origin, jog
  shortcuts: https://docs.lightburnsoftware.com/2.1/Reference/MoveWindow/
- Console window: sent and received lines, typed commands, a "show all"
  switch: https://docs.lightburnsoftware.com/2.1/Reference/ConsoleWindow/
- GRBL v1.1 jogging and commands (see `docs/spec/device.md`).
- All read 2026-10-21. Own design: the device commands and their arguments, not writing work offsets, jog clamping, the frame program, the
  default distance/speed/anchor, the console limit, the panel layout, the
  alarm window, the `M3` pause label, and the CLI.
