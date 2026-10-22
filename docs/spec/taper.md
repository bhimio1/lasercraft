# Spec: taper correction

Milestone: M9. On a rotary, a tapered object (a cup, a tumbler) is
narrower at one end, so the same turn covers less surface there. Taper
correction narrows the design across the turn direction in proportion to
the object's diameter along its length, so the engraving keeps its
proportions on the object.

## Inputs

| Field | Meaning | Valid |
| --- | --- | --- |
| `top` | Which edge of the design goes on the object's top end: `top`, `left` or `right` | |
| `top_diameter` | Object diameter at its top end, mm | 1–1000 |
| `length` | Object length from top to bottom, mm | 1–2000 |
| `bottom_diameter` | Object diameter at its bottom end, mm | 1–1000 |

## The warp

- The **length axis** runs from the design edge named by `top` across the
  design (for `top`: downwards, document +y; for `left`: +x; for `right`:
  −x). The **turn axis** is the other one.
- `s` is a point's distance from that edge along the length axis. The
  object's diameter there is `d(s) = top_d + (bottom_d − top_d) × s /
  length` (beyond `length` the line continues).
- With `d_max = max(top_d, bottom_d)` (the diameter the rotary is set up
  with, `docs/spec/rotary.md`), each point moves along the turn axis
  toward the design's center line on that axis: `t' = c + (t − c) × d(s)
  / d_max`.
- Curves and ellipses are first turned into paths with points at most
  0.05 mm from the curve; straight segments are subdivided every 1 mm
  along the length axis so they bend with the taper. Shapes keep their
  ids, layers and groups; their transform is folded into the new path.
- Images are not warped: a selection with an image is refused ("Taper
  correction works on vector shapes; image ‹id› is selected.").
- The **correction** shown is `1 − min(d) / d_max` over the design, as a
  percentage.

## Command

`shape.taper <top|left|right> <top_diameter> <length> <bottom_diameter>
[ids]`: warps the selection (or `ids`). Undoable: the inverse restores
each shape's geometry and transform. Nothing selected is refused.

## In the app

Machine → "Taper correction…" opens the Taper correction dialog
(`docs/spec/style/later-windows.md`): Top of design (Top edge, Left edge,
Right edge), Top diameter (mm), Length (mm), Bottom diameter (mm), the
read-out "Correction ‹n›%", our own cup drawing, OK and Cancel. The canvas
previews the warped outlines dashed while the dialog is open; OK runs one
`shape.taper`. Enabled while the rotary is on; OK needs a selection.

## Sources

- Purpose, which end is the top, top diameter, length, bottom diameter,
  millimetres, live preview, OK applies to the selection:
  https://docs.lightburnsoftware.com/2.1/Reference/TaperWarp/ (read
  2026-10-22); widest end as the rotary's object diameter:
  https://docs.lightburnsoftware.com/2.1/Reference/RotaryMode/RotaryModeGCode/
- Own design: the linear diameter model, the warp formula, subdivision,
  refusing images, the command.
