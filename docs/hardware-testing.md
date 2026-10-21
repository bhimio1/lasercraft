# Hardware testing (manual)

**These checks are manual. They run on a real machine, by a person standing at
it. Automated tests never talk to hardware; they use the simulated GRBL device.**

Since M6 the app connects to GRBL controllers, so these checks can be run.
The list grows with each milestone that touches the machine.

## Before any test

- Wear laser safety glasses rated for your laser's wavelength.
- Know where the machine's physical emergency stop or power switch is, and keep
  a hand near it.
- Use scrap material, ventilation and the enclosure lid if the machine has one.
- Record: date, LaserCraft commit, OS, controller and GRBL version (`$I`),
  laser module, and `$30`/`$32` settings.

## Checklist (from M5 and M6)

| # | Check | Expected |
| --- | --- | --- |
| H1 | Connect, then wait 30 s | Laser never fires; status updates arrive |
| H2 | Unplug USB while idle | "Disconnected" shown; Stop/Pause hidden |
| H3 | Frame a job | Head traces the bounds with the laser off |
| H4 | Start a job, press Pause | Motion stops; with `M4` the laser is off |
| H5 | Resume | Job continues from where it paused |
| H6 | Press Stop mid-job | Motion and laser stop at once; GRBL may report an alarm; streaming does not resume |
| H7 | Trigger a limit switch or alarm mid-job | Streaming stops; the alarm is shown |
| H8 | Send a job larger than the bed | Refused before anything is sent |
| H9 | Linux without serial group membership | Permission help naming the right group |
| H10 | (from M5) `$$` shows `$32=1` and `$30` equal to the export's `--s-max`; stream an exported file with `$C` (check mode) on | GRBL reports no errors; nothing moves or fires |
| H11 | (from M5) Run a small exported square at low power on scrap | Cut matches the design's position from the origin corner; laser off on travel |

### Added with M6

| # | Check | Expected |
| --- | --- | --- |
| H12 | Connect with `$32=0`, press Start | Refused, naming `$32`; Frame still works with the laser off |
| H13 | Home (Ctrl+H) on a machine with switches; then jog with the arrows and Ctrl+Shift+] / Ctrl+Alt+[ | Homing completes without a "stopped answering" message; jogs move the way the arrows point; laser off |
| H14 | Frame and Frame outline (Alt+R, Alt+Shift+R) | Box and convex outline traced at the panel speed, laser off, head returns to where it was |
| H15 | Start From "Current position", job origin center | Job centered on the head's position; head returns there at the end |
| H16 | Set origin, jog away, Start From "User origin" | Job placed at the stored origin |
| H17 | A layer with constant power (`M3`), pause mid-cut | Motion stops, then the laser goes off; Resume relights it and continues |
| H18 | Stop mid-job (Ctrl+Pause or the red button) | Motion stops, no alarm 3 when the hold completed first; nothing more runs |
| H19 | A long fill job | Streams without stalls or buffer errors; progress bar reaches the end |
| H20 | Console: `$$`, `$I`, `?` | Answers shown; status reports only with "Show all" (except the typed `?`) |

Record results in the PR that touched the behavior, with the details above.
