# Spec: device thread and messages

Milestone: M0 defines the message types and constants; M6 implements the
serial connection, streaming and the simulator.

## Threading

- One device thread per connection owns the port. The UI sends `Request`s and
  receives `Event`s over `std::sync::mpsc` channels; neither side shares
  mutable state with the other. The UI never blocks on the device.

## Messages (M0)

- `Request`: `Connect { port, baud }`, `Disconnect`, `Pause`, `Resume`, `Stop`.
  M6 adds streaming a job, jogging, homing, framing, setting origin and raw
  console lines.
- `Event`: `Connected`, `Disconnected { reason }`, `PermissionDenied { port, help }`,
  `Status { state, position }`, `Alarm { code, message }`, `Stalled`.
- `MachineState` parses the state field of a GRBL status report (`Idle`, `Run`,
  `Hold`, `Jog`, `Alarm`, `Door`, `Check`, `Home`, `Sleep`); a `:n` sub-state
  is ignored. Other text is not a state.

## GRBL constants (M0)

- Real-time bytes: status query `?`, feed hold `!`, cycle start/resume `~`,
  soft reset `0x18`.
- Serial receive buffer: 128 bytes, used by character-counting streaming.

## Permission denied (M0 text, M6 detection)

- When opening a port fails with "permission denied" on Linux, the user sees
  which group owns serial ports on their distribution and the command to join
  it (`docs/linux-serial.md`): `uucp` on Arch and derivatives, `dialout` on
  Debian/Ubuntu and Fedora. The family is read from `/etc/os-release`
  (`ID`, `ID_LIKE`). Inside Flatpak the text also explains device access.

## Planned (M6)

- Character-counting streaming with a send-and-wait ("ping-pong") fallback,
  status polling with `?` at a fixed interval and a stall timeout, alarms and
  errors per `docs/spec/safety.md`, and a simulated GRBL device implementing
  the same interface.

## Sources

- GRBL v1.1 interface (status reports, streaming protocols, 128-byte buffer):
  https://github.com/gnea/grbl/wiki/Grbl-v1.1-Interface
- GRBL v1.1 commands (real-time commands): https://github.com/gnea/grbl/wiki/Grbl-v1.1-Commands
- os-release format: https://www.freedesktop.org/software/systemd/man/latest/os-release.html
- Serial device groups: Arch Wiki, "Users and groups" (group `uucp`), and
  Debian's `dialout` group convention. Own design otherwise.
