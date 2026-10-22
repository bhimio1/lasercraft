# Trace to vectors

Turns the dark parts (or the edges) of an image, or of the camera
picture, into closed paths. The window is drawn as
`docs/spec/style/later-windows.md`, "Trace to vectors".

## Settings

One JSON object; every key optional (defaults in brackets):

| Key | Meaning |
| --- | --- |
| `from` [0], `to` [128] | Brightness range that counts as ink, 0–255, after the image's adjustments (`docs/spec/images.md`). `from` ≤ `to`. |
| `transparency` [false] | Trace what is not transparent instead. Images keep no transparency: import puts transparent pixels on white (`images.md`), so here every pixel that is not pure white (255, before adjustments) counts as ink. |
| `edges` [false] | Follow edges (sketches): a pixel is ink when it is darker than the mean of its neighbourhood (a square of side 1/20 of the image's shorter side, at least 5 pixels) by more than 12 levels; the brightness range is not used. Good for pencil lines and unevenly lit photos. |
| `speck` [2] | Drop specks under this many pixels, 0–10000: ink regions smaller than that are dropped, and holes smaller than that are filled. |
| `smoothness` [1.0] | 0–1.33. 0 gives straight lines only; higher values turn gentler bends into curves (below). |
| `reduction` [0.2] | Node reduction, 0–5: how far (in pixels, plus half a pixel) the outline may stray from the pixel edges to save nodes. |
| `area` [none] | `[x0, y0, x1, y1]` in image pixels: trace only inside this rectangle. |
| `delete` [false] | Delete the image when done (image tracing only). |

## Method (own design)

1. **Ink map**: each pixel is ink or not, by the rule above; outside
   `area` nothing is ink.
2. **Clean-up**: ink regions (pixels joined at edges or corners) smaller
   than `speck` pixels are removed; then holes (non-ink regions joined at
   edges, not reaching the image border) smaller than `speck` are filled.
3. **Outlines**: the borders between ink and non-ink pixels are followed
   along pixel edges into closed loops; where two ink pixels touch only at
   a corner they count as joined. Outer borders and hole borders run in
   opposite directions.
4. **Fewer nodes**: each loop is first replaced by the midpoints of its
   pixel edges, which turns a pixel staircase into a straight line (or a
   smooth bend); corners between two straight runs of 3 pixels or more
   are kept as they are, so square corners stay exact. The other points
   are smoothed lightly (each becomes the mean of itself and up to two
   neighbours on either side, never reaching past a kept corner), which
   spreads the single-pixel jogs of gentle curves. Then the loop is
   simplified (Douglas–Peucker) with a tolerance of `0.5 + reduction`
   pixels.
5. **Curves**: at each remaining node the outline turns by some angle.
   Nodes turning by less than `60° × smoothness` are smooth: their tangent
   follows the neighbouring nodes. The others are corners. A segment with
   a smooth end becomes a cubic Bézier whose handles are a third of the
   segment long along the end tangents; a segment between two corners
   stays straight.
6. **Placement**: pixel corner `(u, v)` lies at the image's local
   `(u · width / pixels across, v · height / pixels down)`, then the
   image's transform; for the camera picture, the overlay's bed mapping.

Each outer loop with the holes inside it becomes one path shape; with
more than one shape, they are put in one new group. At most 200 000 nodes
in all, else "The trace is too detailed: raise Drop specks or Node
reduction, or trace a smaller area."; nothing found gives "Nothing to
trace: no pixels fall in the brightness range."

## Commands

| Command | Effect |
| --- | --- |
| `image.trace <id> [json] [@color]` | Trace image shape `<id>`; the paths go on layer `@color` (default the image's layer). With `delete`, the image is removed in the same step. Undoable. |
| `image.trace_picture <path> <x> <y> <w> <h> [json] [@color]` | Trace a picture file (PNG or JPEG) lying on the bed rectangle `x, y, w, h` mm (the camera route: the app writes the overlay picture and passes its bed rectangle). Undoable. |

The outcome names the number of paths and nodes.

## In the app

- Tools → Trace to vectors…, the image's canvas menu, and Trace camera
  image… (Camera panel) open the window. The preview is computed by the
  app (as for test windows) whenever the settings change, and drawn by
  the window: the picture with the traced loops (flattened) over it.
- Dragging in the preview draws the trace area; Clear trace area removes
  it. Dim image fades the picture; Show nodes draws the nodes.
- OK runs `image.trace` (or, for the camera, `image.trace_picture` on the
  current overlay picture, cropped to the bed) with the window's settings
  and the current layer; Cancel adds nothing.

## Sources

- Trace window, live preview, brightness range, transparency and sketch
  modes, speck, smoothness and node-reduction settings, trace area, delete
  after tracing, tracing the camera picture:
  https://docs.lightburnsoftware.com/2.1/Reference/TraceImage/
- Method: own design, from textbook techniques (region labelling,
  boundary following on the pixel grid, Douglas–Peucker simplification,
  cubic Bézier handles along tangents).
