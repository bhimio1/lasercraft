# Spec: images

Milestone: M7. Bitmap images become shapes on a layer; the layer's image
settings turn them into rows of dots or gray levels that the planner,
G-code export and the device stream like any other job.

## Image shapes

- An image is a shape with geometry `image`: a width and height in mm (its
  local rectangle (0, 0)–(width, height), like `rect`), a grayscale bitmap,
  and its own adjustments. Move, scale, rotate, mirror, duplicate, copy,
  group, align, arrange and delete work as for any shape; its transform
  places it.
- Images are stored as 8-bit grayscale (luminance). Color is not kept: a
  laser burns one shade. Transparent pixels are composited onto white
  (unburned). Luminance = 0.299 R + 0.587 G + 0.114 B (ITU-R BT.601).
- At most 25 000 000 pixels and 20 000 pixels per side.
- Images have no vector outline: line and fill modes, booleans, offset,
  weld, node editing, kerf and "convert to path" skip them or refuse them
  ("shape N is an image"). Selection, hit testing and snapping use their
  rectangle.

## Adjustments (per image)

| Setting | Meaning | Default | Valid |
| --- | --- | --- | --- |
| `brightness` | Added to every level, in % of full scale | 0 | −100–100 |
| `contrast` | Stretches levels around mid-gray: factor (100 + c) / (100 − c) | 0 | −99–99 |
| `gamma` | Level ↦ level^(1/gamma) on 0–1; above 1 lightens mid-tones | 1 | 0.1–10 |

Applied in that order on 0–1 levels, each result clamped to 0–1. The layer's
`image.negative` is applied after them. `image.adjust <id> [brightness=]
[contrast=] [gamma=]` changes them (undoable).

## From bitmap to dots

1. **Grid.** Rows are `image.interval` mm apart; dots along a row are the
   same distance apart (DPI = 25.4 / interval). The grid is aligned to the
   document origin, so neighbouring images share rows. It covers the image's
   bounding box.
2. **Sampling.** Each dot's center is mapped back through the image's
   transform into bitmap pixels. When the bitmap has more than one pixel per
   dot it is first averaged down (box filter) to about one pixel per dot;
   the value is then bilinearly interpolated. Dots outside the image are
   empty.
3. **Darkness.** d = 1 − level after adjustments (negative: d = level).
4. **Method** (`image.dither`), giving each dot a power fraction 0–1:

| Method | Rule |
| --- | --- |
| `grayscale` | d itself (0 is off). Power = `min_power` + d × (`max_power` − `min_power`); white dots are not burned. Levels are quantized to 1/255. |
| `threshold` | 1 when d ≥ `image.threshold` / 100, else 0. |
| `floyd_steinberg` | Error diffusion, weights 7 → right; 3, 5, 1 → below-left, below, below-right; /16. |
| `jarvis` | Error diffusion over two rows ahead, weights 7 5 / 3 5 7 5 3 / 1 3 5 3 1, /48. |
| `stucki` | Error diffusion, weights 8 4 / 2 4 8 4 2 / 1 2 4 2 1, /42. |
| `atkinson` | Error diffusion, 1/8 to right, right+2, below-left, below, below-right, two below; 6/8 of the error is spread. |
| `ordered` | 8 × 8 Bayer matrix, on when d > (index + 0.5) / 64. |
| `halftone` | Clustered dots on a screen of `image.halftone_cells` cells per inch at `image.halftone_angle` degrees: on when d exceeds the cell's spot threshold. |

   Error diffusion scans rows left to right, top to bottom, quantizing at 0.5
   and spreading the error to dots not yet visited (dots outside the image
   receive none). Dithered dots burn at `max_power`.

## Planning

- Per layer, after its vector shapes, every image on it is planned in drawing
  order with the layer's speed, passes, `bidirectional` and `overscan`
  settings, scanning horizontal rows from the front of the bed (bottom of the
  screen) to the back, like fill.
- A row that has no dots is skipped. Each row runs from its first to its last
  dot: burn moves over runs of dots with the same power (a run spans the dots'
  full width, so neighbouring dots join), unlit moves over gaps, overscan
  before and after with the laser off. Moves to the next row are travel.
- Grayscale runs change power where the level changes; `S` follows in the
  G-code. Dithered runs use `max_power`. Power is clamped as for every
  layer (safety rule 6).
- Images on a layer with `output` off are skipped. The preview shows the
  dots as burn moves.

## Import

- `project.import <file>` also takes PNG and JPEG (by extension). The image
  becomes one shape on the given color (default 0) and is selected.
- Size: from the file's resolution (PNG `pHYs`, JPEG JFIF density) when it
  states one; otherwise 254 DPI (0.1 mm per pixel). It is placed with its
  top-left corner at the document origin.

## File format

The project file version becomes 5 (older readers must refuse a file with an
image; files without images still load in version-4 readers is not
promised). Image geometry is `{ "kind": "image", "width": mm, "height": mm,
"pixels": [w, h], "brightness": 0, "contrast": 0, "gamma": 1, "png": "<base64
of an 8-bit grayscale PNG>" }`. The PNG is written with default compression,
so saving the same image twice gives the same text.

## Canvas

Images are drawn as their adjusted grayscale bitmap (not dithered), with a
thin border in the layer color; the job preview shows the result of the
method.

## Sources

- To be completed from the research notes.
- Error-diffusion kernels: R. W. Floyd and L. Steinberg, "An adaptive
  algorithm for spatial grey scale" (1976); J. F. Jarvis, C. N. Judice and
  W. H. Ninke (1976); P. Stucki (1981); Bill Atkinson's dither as published
  in public descriptions (e.g. Wikipedia, "Dither").
- Bayer matrix: B. E. Bayer, "An optimum method for two-level rendition of
  continuous-tone pictures" (1973).
- Luminance weights: ITU-R Recommendation BT.601.
- PNG: W3C/ISO 15948; JPEG/JFIF: ITU-T T.81 and the JFIF 1.02 note.
