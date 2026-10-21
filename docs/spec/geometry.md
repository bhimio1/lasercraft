# Spec: geometry primitives

Milestones: M0 (points, transforms, bounds), M3 base (paths, booleans,
offsets). Paths, curves, flattening,
booleans and offsets get their own specs in M1–M4.

## Coordinates

- Millimetres, `f64`. x grows to the right and y grows downwards, as in SVG.
  A positive rotation turns clockwise on screen.
- Affine transforms use SVG's `matrix(a b c d e f)` layout:
  `x' = a·x + c·y + e`, `y' = b·x + d·y + f`. `t1 * t2` applies `t2` first.
- `inverse` returns nothing for a transform whose determinant is (nearly) zero.
- `Rect` is an axis-aligned box. `Rect::transformed` gives the bounding box of
  the transformed corners, which bounds the transformed rectangle exactly.

## Paths (M3 base)

- A path is one or more subpaths. A subpath has a start point and segments,
  each a line to a point or a cubic Bézier (two control points, then the end
  point), and is open or closed. Quadratic curves and arcs are converted to
  cubics on import; circular arcs use pieces of at most 90°.
- In files a line segment is `[x, y]` and a cubic `[c1x, c1y, c2x, c2y, x, y]`.
- Bounds are exact: curve extrema are found from the derivative.
- Flattening subdivides each cubic until its control points lie within the
  tolerance of the chord, so the polyline stays within the tolerance of the
  curve.
- Affine transforms map Béziers to Béziers exactly.
- Node editing, joining and splitting paths: `docs/spec/nodes.md` (M3).

## Booleans and offsets (M3 base)

- Booleans (union, subtract, intersect, exclusive-or) and offsets work on
  flattened closed contours with the even-odd rule, through `i_overlay`.
- Union of several shapes treats each shape even-odd on its own (its holes
  stay holes) and then merges the shapes, so overlaps join.
- Offsets grow (positive) or shrink (negative) closed contours with miter,
  round or bevel corners; shrinking past the middle leaves nothing.

## Sources

- SVG 1.1 paths (cubic Béziers, arcs converted to curves):
  https://www.w3.org/TR/SVG11/paths.html
- SVG 1.1, "Coordinate Systems, Transformations and Units" (the `matrix`
  transform and the y-down user space): https://www.w3.org/TR/SVG11/coords.html
- Own design otherwise.
