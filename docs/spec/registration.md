# Spec: print registration

Milestone: M9. Print registration lines a job up with something already on
the bed (a printed sheet, an earlier engraving) by two target marks: the
user picks each target in the design and moves the head over the same
mark on the bed. LaserCraft then moves, turns and optionally scales the
job so the design's targets land on the measured positions.

## Targets

- A **target** is one object of the design: one shape or one top-level
  group (a crosshair is usually two lines grouped). Its **design point**
  is the center of its bounding box, document mm. Its **measured point**
  is the head's position when the target was set, machine mm.
- Targets are usually on a layer with output off, so they are not cut.
- Setting a target needs a connected machine that reports its position
  (the head position comes from the status reports, as for Current
  position, `docs/spec/laser.md`). The head must only be moved by jogging
  or Move head; moving it by hand loses the position. Homing first is
  advised (the dialog says so); LaserCraft cannot check it.

## Fit

Two fits, as a transform `T` of document points (`m₁`, `m₂` are the
measured points as document points, `d₁`, `d₂` the design points):

- **Fit job to targets** (`scaled`): the similarity (move, turn, uniform
  scale) that maps `d₁ → m₁` and `d₂ → m₂` exactly.
- **Place job on targets** (`rigid`): turn by the angle between `d₂ − d₁`
  and `m₂ − m₁`, no scaling, and move so the middle of the design points
  lands on the middle of the measured points.

While a fit is set:

- Every planned job (`device.start`, `device.frame`, `project.export_gcode`,
  the job preview, the CLI's `preview` and `export`) uses the project with
  every shape's transform preceded by `T`. Fills keep their scan angle on
  the bed. The project's shapes themselves do not change.
- Placement is always Absolute coordinates: a `from=current` or
  `from=user` on `device.start` / `device.frame` is ignored and the outcome
  says "Print registration places the job by its targets."
- The usual work area check applies to the fitted job.
- A scaled fit whose scale is off by more than 5% is refused ("the targets
  are ‹p›% farther apart on the bed than in the design; check the
  targets"); above 1% the outcome warns.

## Errors

| Case | Message |
| --- | --- |
| Not one object | "Cannot set target ‹n›: ‹k› objects are selected. Select one shape, or group the target's parts first." |
| Target 2 before target 1 | "Set target 1 first." |
| Same object twice | "Cannot set target 2: it is the same object as target 1. Select the second target." |
| Head did not move | "Cannot set target 2: the head is still where target 1 was set. Move the head over the second mark first." (less than 1 mm) |
| Too close | "Cannot set target 2: the targets are only ‹d› mm apart. Use targets at least 10 mm apart." (design or measured distance) |
| No head position | "Cannot set target ‹n›: the machine did not report its position. Wait until it is Ready, then try again." |
| Fit before both targets | "Set both targets first." |

## Project file

`registration` (optional): `{"targets": [{"object": <id>, "design":
[x, y], "machine": [x, y]}, …], "fit": "none" | "scaled" | "rigid"}`.
At most two targets. A target whose object no longer exists is shown as
missing and the fit is ignored until it is set again.

## Commands

All undoable (they change the project's `registration`).

| Command | Effect |
| --- | --- |
| `registration.set_target <1\|2> [id] head=X,Y` | Set a target from the selected object (or `id`) and the head position. Setting target 1 clears target 2 and the fit. |
| `registration.fit <scaled\|rigid>` | Set the fit. |
| `registration.clear` | Remove targets and fit. |
| `registration.restore <1\|2> <object> <x,y> <x,y>` | Put one target back exactly; undo uses it. |
| `registration.expected` | Move the head (laser off, jog speed) to where target 2 should be: target 1's measured point plus the design offset `d₂ − d₁`. Needs target 1, a selected second target and a Ready machine. Not undoable (a machine move). |

The app fills `head=` from the last status report, as for framing.

## In the app

- Machine → Print registration ▸ with Guided setup… (Alt+0), Set target 1
  (Alt+1), Set target 2 (Alt+2), Fit job to targets (Alt+3), Place job on
  targets (no scaling) (Alt+4), Clear registration.
- The Print registration window (`docs/spec/style/later-windows.md`):
  three pages (Target 1, Target 2 with "Go to expected target 2", Fit),
  a preconditions line, Clear registration, Back and Close. Choosing a fit
  closes it.
- The canvas marks targets with a numbered ring at their design point,
  dashed while not set and solid when set; with a fit, the ring also
  shows where the target lands (`docs/spec/style/canvas.md`).
- While a fit is set the status bar's Active setups field reads "Print
  registration" and Place job at is shown as Absolute coordinates,
  disabled.
- Errors appear in a "Registration failed" message window that never
  covers the job bar.

## Sources

- Two targets, center of a single shape or group, set with the head
  position, scaled and unscaled alignment, absolute placement, no moving
  by hand, error cases, jog to the expected second target, preview turns
  the job: https://docs.lightburnsoftware.com/2.1/Reference/PrintAndCut/
  (read 2026-10-22).
- Own design: the fit formulas, the scale limits, the minimum distances,
  the file format, the command set, the message wording.
