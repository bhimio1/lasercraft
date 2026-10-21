# Spec: geometry primitives

Milestone: M0 (points, transforms, bounds). Paths, curves, flattening,
booleans and offsets get their own specs in M1–M4.

## Coordinates

- Millimetres, `f64`. x grows to the right and y grows downwards, as in SVG.
  A positive rotation turns clockwise on screen.
- Affine transforms use SVG's `matrix(a b c d e f)` layout:
  `x' = a·x + c·y + e`, `y' = b·x + d·y + f`. `t1 * t2` applies `t2` first.
- `inverse` returns nothing for a transform whose determinant is (nearly) zero.
- `Rect` is an axis-aligned box. `Rect::transformed` gives the bounding box of
  the transformed corners, which bounds the transformed rectangle exactly.

## Sources

- SVG 1.1, "Coordinate Systems, Transformations and Units" (the `matrix`
  transform and the y-down user space): https://www.w3.org/TR/SVG11/coords.html
- Own design otherwise.
