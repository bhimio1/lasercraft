# Spec: rotary attachment

Milestone: M9. A rotary turns a round object under the head so its surface
can be engraved. LaserCraft supports rotaries that GRBL drives as its Y
axis (the Y motor cable is moved to the rotary). GRBL 1.1 has no A axis,
so "A axis" from the style guide is not offered.

## Settings

Stored in the project (`rotary` in the project file, `docs/spec/
project-file.md`), because they describe the object of this job:

| Key | Meaning | Default | Valid |
| --- | --- | --- | --- |
| `enabled` | Rotary output is on | false | |
| `kind` | `rollers` (the object rests on driven rollers) or `chuck` (the object is held and turned) | `rollers` | |
| `mm_per_turn` | Y travel GRBL makes for one full turn of the driven part (roller or chuck) | 100 | 1–10 000 |
| `roller_diameter` | Diameter of the driven roller, mm (rollers only) | 30 | 1–1000 |
| `object_diameter` | Diameter of the object, mm (the widest end of a taper) | 80 | 1–1000 |
| `mirror` | Turn the other way | false | |

- The **surface diameter** is `roller_diameter` for rollers (the roller's
  surface moves the object's surface the same distance) and
  `object_diameter` for a chuck.
- The **circumference** is `π × object_diameter` for both kinds; it is
  only a guide for rollers (the object may slip).

## Output

With `enabled` on, planning is unchanged (the design is planned as drawn,
in surface millimetres) and placing the job (`docs/spec/laser.md`) is
unchanged. Then, for G-code:

1. The **scale** is `k = mm_per_turn / (π × surface diameter)`, negated
   when `mirror` is on.
2. Every point's document Y is replaced by `y₀ + k × (y − y₀)`, where `y₀`
   is the Y of the job's placement point: the job origin point after
   placing (Current position and User origin), or the top edge of the job
   box where it is drawn (Absolute coordinates). X is unchanged.
3. The feed of every `G1` move is scaled so the surface still moves at the
   layer's speed: `F × axis length / surface length` (axis length in the
   rotary's millimetres). Rapid moves are not changed. The time estimate
   is the surface time, unchanged.
4. The work area check applies to X only. Y is a rotation and may go
   below 0 or past the bed; the verifier gets no Y limits. GRBL's soft
   limits (`$20=1`) may still raise alarm 2; the Rotary attachment dialog
   says so.
5. A design taller than the circumference wraps onto itself: the job
   still runs, with the warning "The design is ‹h› mm tall, more than the
   object's circumference ‹c› mm; it overlaps itself."
6. A job origin of Current position is the usual choice; with Absolute
   coordinates a warning reads "Rotary jobs usually start from the current
   position."

Framing traces the same transformed outline with the laser off. Test
patterns are planned on their own scratch project and ignore the rotary. The job
preview shows the design as drawn (the surface), with the line "Rotary: Y
turns ‹d›° for the design's height" (degrees = height / circumference ×
360).

## Commands

| Command | Effect |
| --- | --- |
| `rotary.set <key> <value> [key=value …]` | Change settings (keys above, `enabled` takes `on`/`off`). Undoable; the inverse restores every changed key. |
| `rotary.clear` | Forget the rotary settings (back to defaults, off). Undoable. |
| `device.rotary_test [speed=<mm/s>]` | Turn the driven part one full turn forward, pause one second, then back: a short laser-off program (`G91`, `G1 Y…`, `G4 P1`, `G1 Y−…`, `G90`) streamed like framing. Needs a connected, Ready machine; the settings need not be enabled. |

## In the app

- Machine → "Rotary attachment…" (Ctrl+Shift+R) opens the Rotary
  attachment dialog (`docs/spec/style/later-windows.md`): Use rotary,
  Rollers / Chuck, Travel per turn (mm), Roller diameter (mm) (rollers
  only), Object diameter (mm), Turn the other way, the read-out
  "Circumference ‹c› mm; the design's height turns ‹d›°", Test rotation,
  a hint about soft limits, OK and Cancel. OK runs one `rotary.set` with
  every changed key.
- While rotary is on, the status bar shows "Rotary" in its Active setups
  field and Start's tooltip ends with "(rotary)".

## Open questions

- Rotaries on a dedicated axis (grblHAL's A axis) and degree-based
  output.
- Splitting long rotary engravings into segments.

## Sources

- Rotary kinds, travel per rotation, roller and object diameter,
  circumference, test turn, mirror, Current position, soft limits:
  https://docs.lightburnsoftware.com/2.1/Reference/RotaryMode/RotaryModeGCode/
  (read 2026-10-22).
- GRBL jogging (`$J=`), soft limits `$20` and alarm 2:
  https://github.com/gnea/grbl/wiki/Grbl-v1.1-Jogging and
  https://github.com/gnea/grbl/wiki/Grbl-v1.1-Configuration
- Own design: the stored keys, scaling about the placement point, feed
  scaling, the warnings, the test pause, the commands.
