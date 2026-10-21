# Spec: laser safety

Milestone: binding from M0; each rule is implemented by the milestone named
and must have tests there. This software drives a real laser.

## Firing

1. The laser fires only during a job, framing or test fire that the user
   explicitly started. Nothing fires on connect, on load, at startup, after an
   alarm is cleared, or as a side effect of another command. (M5, M6)
2. Framing traces the job bounds with laser-off moves by default. A low-power
   framing option, if added, is opt-in per use and shows its power. (M6)
3. Jogging and homing never turn the laser on. (M6)
4. G-code we generate turns the laser off (`M5`, `S0`) at the end of every job,
   and uses `G0` (which GRBL's laser mode never fires on) for travel. (M5)

## Limits

5. Before anything is sent, every move of the planned job (including overscan,
   lead-ins and the job origin offset) is checked against the work area. Any
   point outside it refuses the whole job with an error naming the axis, the
   distance outside and the shape involved. Nothing partial is sent. (M4–M6)
6. Every power value is clamped to the layer's maximum and the device's maximum
   (and S-max scaling is applied after clamping). A power above either limit in
   the input never produces a higher `S` word in the output. (M2, M5)

## Stop and pause

7. While connected, Stop and Pause are always visible and enabled, whatever
   panel or dialog is open. (M6)
8. Pause sends GRBL's feed hold `!`; Resume sends `~`; Stop stops streaming
   immediately and sends soft reset `0x18`. These are real-time bytes, so they
   bypass the streaming queue and GRBL acts on them mid-line. The device
   thread handles them before any queued work. (M6)
9. GRBL's documentation says a feed hold stops motion but not the spindle
   output, and that `M3` (constant power) keeps the laser at its set power
   while stopped, whereas `M4` (dynamic power) turns it off when not moving.
   Therefore: G-code uses `M4` by default (M5); a job that uses `M3` labels
   Pause as "motion only, the laser may stay on" and makes Stop the
   highlighted action (M6). (Decided 2026-10-21, owner delegated the call.)

## Faults

10. Disconnect, a status reply that does not arrive within the stall timeout,
    an `ALARM:` message, or an `error:` response stops streaming and shows the
    alarm or error to the user with its GRBL meaning. Streaming never resumes
    by itself. (M6)
11. GRBL's docs warn that with character counting, lines already in its receive
    buffer still run after an error. So jobs are checked before streaming (our
    own checks, optionally GRBL's `$C` check mode), and on an error we also
    send feed hold and soft reset. (M6)

## Process

12. No `unsafe` code. The device thread never blocks the UI (`ARCHITECTURE.md`).
13. Automated tests never touch real hardware; they use the simulated GRBL
    device, which parses the stream, tracks position and laser state, honors
    the 128-byte receive buffer, and can raise alarms. Hardware checks are
    manual (`docs/hardware-testing.md`). (M6)
14. The simulator asserts rule 1 in every test: the laser is on only while a
    started job, framing or test fire is running.

## Sources

- GRBL v1.1 real-time commands, feed hold and soft reset:
  https://github.com/gnea/grbl/wiki/Grbl-v1.1-Commands
- GRBL v1.1 laser mode (`M3`/`M4`, `G0` never fires, `S0`):
  https://github.com/gnea/grbl/wiki/Grbl-v1.1-Laser-Mode
- GRBL v1.1 interface: streaming, the 128-character receive buffer, errors
  during character counting, `$C`: https://github.com/gnea/grbl/wiki/Grbl-v1.1-Interface
- The project owner's safety requirements (session 1, 2026-10-21).
