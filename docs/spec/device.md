# Spec: device connection, GRBL streaming and the simulator

Milestones: M0 defined the message types and constants; M6 implements the
connection, streaming, jogging, homing, framing and the simulated device.
Safety rules referenced as "rule n" are in `docs/spec/safety.md`.

## Layers

`lasercraft-device` has four parts, each testable on its own:

1. **Protocol text** (`grbl`, `reply`): real-time bytes, parsing GRBL's
   replies (`ok`, `error:n`, `ALARM:n`, `<status>`, the banner, `[MSG:…]`,
   `[OPT:…]`, `$n=v`) and the meaning of alarm and error codes, in our words.
2. **Controller** (`controller`): a state machine with no I/O and no clock of
   its own. It takes requests, received lines and the current time; it gives
   bytes to write and events. All streaming, polling, stall, pause and stop
   logic lives here, so tests drive it step by step.
3. **Simulator** (`sim`): a simulated GRBL 1.1 laser controller, also without
   I/O, running on simulated time.
4. **Device thread** (`thread`): owns the port (serial or simulator) and a
   controller, and talks to the UI only through channels.

## Threading and messages

- `spawn()` starts one device thread and returns a handle with a
  `Sender<Request>` and a `Receiver<Event>`. Neither side shares mutable state.
  The UI only ever does non-blocking `try_recv`.
- The thread wakes at least every 5 ms. Each wake it handles every pending
  request (Pause, Resume and Stop first, rule 8), reads what the port has,
  advances the controller and writes what it produced.
- Ports: a serial path (`/dev/ttyUSB0`, `COM3`, …) at the requested baud
  (default 115200, 8N1, no flow control), or `simulator`, a simulated machine
  running in real time.

### Requests

| Request | Effect |
| --- | --- |
| `Connect { port, baud }` | Open the port and run the handshake (below). |
| `Disconnect` | Stop anything running (as Stop) and close the port. |
| `Stream { kind, gcode }` | Stream a program; `kind` is `Job` or `Frame`. |
| `Pause` / `Resume` / `Stop` | See "Pause, resume and stop". |
| `Jog { x, y, feed, relative }` | `$J=G21 G91 X… Y… F…` (relative) or `G90` (absolute). |
| `Home` | `$H`. |
| `Unlock` | `$X`. |
| `Console(line)` | Send a line typed by the user. |

Anything that moves the head (`Stream`, `Jog`, `Home`) is refused with a
`Rejected` event unless the machine is idle and nothing else is running. A
`Jog` is also accepted while jogging. `Unlock` and `Console` are refused while
a stream runs.

### Events

| Event | Meaning |
| --- | --- |
| `Connected { port }` | The port is open and GRBL answered. |
| `Ready(Info)` | Handshake done: firmware version, receive buffer size, laser mode (`$32`), S-max (`$30`), max travel (`$130`, `$131`), homing enabled (`$22`). |
| `Disconnected { reason }` | The port closed (on request, or because it failed). |
| `PermissionDenied { port, help }` | Opening failed for lack of permission (Linux help text, `docs/linux-serial.md`). |
| `Status { state, position }` | A status report; position is the work position, mm. |
| `Alarm { code, message }` | GRBL reported an alarm. |
| `Stalled` | No status reply within the stall timeout. |
| `Console { sent, text }` | A line we sent (`sent = true`) or received. |
| `Progress { kind, acknowledged, total }` | Lines GRBL has accepted so far. |
| `Finished { kind, outcome }` | A stream ended: `Completed`, `Stopped` or `Failed(reason)`. |
| `Rejected(reason)` | A request was refused; nothing was sent. |

## Handshake

1. On connect, wait up to 3 s for the GRBL banner (`Grbl 1.1h ['$' for help]`;
   opening the port usually resets the board). If none arrives, send `?`; a
   status reply means GRBL was already running. Neither within 3 s more:
   disconnect with "no reply from a GRBL controller".
2. Send `$I` and `$$` one at a time (send-response). From `[OPT:…,blocks,rx]`
   take the receive buffer size (default 128). From `$$` take `$32`, `$30`,
   `$22`, `$130`, `$131`.
3. Emit `Ready`. Status polling starts.

Nothing in the handshake moves the head or touches the laser (rule 1). If a
reply is missing or malformed the value is unknown.

**Laser mode.** With `$32=0` GRBL does not turn the laser off during `G0`
travel. A `Job` stream is therefore refused unless `$32=1` was read:
"laser mode is off ($32=0): the laser would stay on during travel moves; set
$32=1 in the console". Framing is allowed (it never turns the laser on).

## Streaming

- **Character counting** (default): the controller keeps the byte count of
  every line sent and not yet answered (a FIFO, `\n` included). A line is sent
  only if the count plus its length fits the receive buffer. Each `ok` or
  `error:n` removes the oldest line.
- **Send-response** ("ping-pong"): one line at a time; the next is sent after
  the previous one is answered. Used:
  - when chosen in the connection settings;
  - always for `$` system commands and setting writes, during the handshake
    and from the console (GRBL's docs require it for EEPROM writes);
  - automatically for the rest of the connection if a reply arrives with no
    line outstanding (the counts no longer match), with a console note.
- Lines are trimmed; comments and empty lines are not sent. A line longer
  than 80 characters is refused before streaming starts.
- Real-time bytes are written at once and never counted.
- Push messages (status, `[MSG:…]`, `ALARM:`, the banner, `$n=v`) never count
  as answers.
- `Progress` is emitted at most every 100 ms and at the end. A stream
  completes when every line is answered `ok` and the machine reports `Idle`.

## Status polling and stalls

- `?` every 200 ms while connected (GRBL's docs suggest at most 5 Hz).
- Reports parse `MPos` or `WPos`; `WCO` is cached and work position = `MPos`
  − `WCO`.
- **Stall**: no status reply for 2 s (configurable) emits `Stalled`, and a
  running stream ends `Failed("the controller stopped answering")` after
  feed hold and soft reset are sent (rule 10). GRBL answers no status queries
  while homing, so the stall timer is suspended until `$H` is answered, up to
  a homing timeout of 90 s.

## Errors and alarms during a stream (rules 10, 11)

- `error:n`: the controller immediately sends feed hold `!`, then soft reset
  `0x18` once the machine reports `Hold:0` or `Idle` (or after 1 s), and ends
  the stream `Failed("line n: error n: meaning")`. Lines already in GRBL's
  buffer are discarded by the reset.
- `ALARM:n`: GRBL has already stopped. The stream ends `Failed`, `Alarm` is
  emitted with the code's meaning, nothing more is sent until the user
  unlocks or homes.
- Disconnect: the stream ends `Failed`, `Disconnected` is emitted.
- Streaming never resumes by itself.

## Pause, resume and stop (rules 7–9)

- **Pause** (`!`): GRBL decelerates and holds. If the running program uses
  `M3` (constant power, which keeps the laser on while stopped), the
  controller also sends spindle stop `0x9E` as soon as the hold completes
  (`Hold:0`); GRBL turns the spindle back on before resuming. Pause while not
  running is ignored.
- **Resume** (`~`).
- **Stop**: sending lines stops at once and feed hold `!` is sent; soft reset
  `0x18` follows as soon as the machine reports `Hold:0` or `Idle`, or after
  1 s. Resetting after the hold keeps GRBL's position (a reset in motion
  loses it). The stream ends `Stopped`. Stop works in every state, also with
  nothing streaming (it then cancels jogs and homing).

## Simulator

`Sim` behaves like GRBL 1.1 in laser mode, closely enough to test the
controller:

- 128-byte receive buffer; bytes arriving when it is full are dropped and
  recorded as an overflow (a test failure). 15-block planner queue: a motion
  line is taken from the receive buffer only when a block is free; it is then
  answered `ok`.
- Parses the words the exporter writes plus `G91`, `G4 P`, `$J=…`, `$H`,
  `$X`, `$I`, `$$`, `$G`; anything else is `error:20` (or `error:3` for an
  unknown `$` command). Motion in alarm is `error:9`; a jog outside the
  travel is `error:15`; `$J=` without `F` is `error:16`.
- Moves at the programmed feed (rapids at 6000 mm/min) with a fixed
  deceleration time for holds; tracks machine position, spindle mode and `S`.
- The laser is on exactly while a `G1` block runs with `M3`/`M4` and `S` > 0,
  or while held mid-`G1` under `M3` without spindle stop. Every lit stretch is
  recorded (`LaserLog`), so tests assert the laser fired only inside a job.
- Status reports `<State|MPos:x,y,z|FS:f,s>` with `WCO` in the first report
  and every 10th; no reply while homing.
- Faults on demand: raise an alarm, stop answering (stall), disconnect.
- Real-time bytes: `?`, `!` (holds; cancels a jog), `~`, `0x18` (reset:
  in motion → `ALARM:3` and lost position, else position kept; prints the
  banner), `0x9E` (spindle stop in hold).
- Settings answer `$$` with `$22`, `$30`, `$32`, `$130`, `$131` (configurable).

## Permission denied

When opening a port fails with "permission denied" on Linux, the user sees
which group owns serial ports on their distribution and the command to join
it (`docs/linux-serial.md`): `uucp` on Arch and derivatives, `dialout` on
Debian/Ubuntu and Fedora. The family is read from `/etc/os-release` (`ID`,
`ID_LIKE`). Inside Flatpak the text also explains device access.

## Sources

- GRBL v1.1 interface (responses, push messages, character counting and
  send-response streaming, the 128-byte buffer, errors during character
  counting, status report fields and `WCO`, polling rate, no status while
  homing, `[OPT:…]`): https://github.com/gnea/grbl/wiki/Grbl-v1.1-Interface
- GRBL v1.1 commands (real-time commands `?` `!` `~` `0x18` `0x9E`, `$H`,
  `$X`, `$I`, `$$`, reset behavior): https://github.com/gnea/grbl/wiki/Grbl-v1.1-Commands
- GRBL v1.1 jogging (`$J=` syntax, errors 15 and 16, feed hold cancels jogs):
  https://github.com/gnea/grbl/wiki/Grbl-v1.1-Jogging
- GRBL v1.1 configuration (`$10`, `$22`, `$30`, `$32`, `$130`, `$131`):
  https://github.com/gnea/grbl/wiki/Grbl-v1.1-Configuration
- GRBL alarm and error code tables:
  https://github.com/gnea/grbl/tree/master/doc/csv
- os-release format: https://www.freedesktop.org/software/systemd/man/latest/os-release.html
- Serial device groups: Arch Wiki, "Users and groups" (group `uucp`), and
  Debian's `dialout` group convention.
- All read 2026-10-21. Own design: the controller/simulator split, the
  timings (5 ms wake, 200 ms poll, 2 s stall, 1 s reset fallback, 90 s
  homing), the hold-then-reset Stop, spindle stop on pause for `M3`, refusing
  jobs without `$32=1`, the automatic send-response fallback, the simulator's
  motion model and the request/event set.
