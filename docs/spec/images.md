# Spec: images

Milestone: M7. Bitmap images become shapes on a layer; the layer's image
settings turn them into rows of dots or gray levels that the planner, G-code
export and the device stream like any other job.

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
- Images have no vector outline. Booleans, weld, offset and join refuse them
  ("shape N is an image; this works on vector shapes only"); convert to
  path, node editing, line and fill modes, kerf and tabs skip them.
  Selection, hit testing and snapping use their rectangle.

## Adjustments (per image)

| Setting | Meaning | Default | Valid |
| --- | --- | --- | --- |
| `brightness` | Added to every level, in % of full scale | 0 | −100–100 |
| `contrast` | Stretches levels around mid-gray by (100 + c) / (100 − c) | 0 | −99–99 |
| `gamma` | Level ↦ level^(1/gamma) on 0–1; above 1 lightens mid-tones | 1 | 0.1–10 |

- Applied in that order on 0–1 levels, each result clamped to 0–1. The
  layer's `image.negative` is applied after them.
- `image.adjust <id> [brightness=] [contrast=] [gamma=]` changes them; it is
  undoable. Values not given stay.
- Tools → Adjust Image… (Alt+I) opens a window for the first selected image
  with three sliders and Reset; OK runs `image.adjust` (values rounded to
  0.1, gamma to 0.01).

## From bitmap to dots

1. **Grid.** Rows are `image.interval` mm apart; dots along a row are the
   same distance apart (DPI = 25.4 / interval). The grid is aligned to the
   document origin, so neighbouring images share rows and screens line up.
   It covers the image's bounding box; dot (i, j) is centered on
   ((i + 0.5) × interval, (j + 0.5) × interval).
2. **Sampling.** Each dot's center is mapped back through the image's
   transform into bitmap pixels. When the bitmap has two or more pixels per
   dot it is first averaged down (box filter) to about one pixel per dot;
   the value is then bilinearly interpolated. Dots whose center is outside
   the image are empty.
3. **Darkness.** d = 1 − adjusted level (negative: d = adjusted level).
4. **Method** (`image.dither`) gives each dot a power fraction 0–1:

| Method | Rule |
| --- | --- |
| `grayscale` | d, quantized to 1/255; 0 is off. Power = `min_power` + d × (`max_power` − `min_power`); white dots are not burned. |
| `threshold` | 1 when d ≥ 0.5 (brightness moves the cut-off). For art that is already black and white. |
| `floyd_steinberg` | Error diffusion: 7 → right; 3, 5, 1 → below-left, below, below-right; ÷ 16. LaserCraft's general "dither" for photos. |
| `jarvis` | Error diffusion over two rows ahead: 7 5 / 3 5 7 5 3 / 1 3 5 3 1, ÷ 48. |
| `stucki` | Error diffusion: 8 4 / 2 4 8 4 2 / 1 2 4 2 1, ÷ 42. |
| `atkinson` | Error diffusion: 1 each → right, two right, below-left, below, below-right, two below, ÷ 8; only three quarters of the error is passed on. |
| `ordered` | 8 × 8 Bayer matrix on the global grid: on when d > (index + 0.5) / 64. |
| `halftone` | Clustered round dots on a screen of `image.halftone_cells` cells per inch at `image.halftone_angle` degrees. A cosine spot function, ranked so a cell's covered area follows d, is compared with d. |

   Error diffusion scans rows left to right, top to bottom, quantizing at 0.5
   and spreading the error only to dots inside the image not yet visited.
   Dithered and threshold dots burn at `max_power`.

## Planning

- Per layer, after its vector shapes, every image on it is planned in drawing
  order with the layer's speed, passes, `bidirectional` and `overscan`
  settings. Rows are horizontal and run from the bottom of the screen up,
  like fill. Images that cannot be rasterized (more than 60 000 000 dots)
  are skipped with a warning.
- A row without dots is skipped. A row runs from its first to its last dot:
  burn moves over runs of dots with the same power (a run spans its dots'
  full width, so neighbours join), unlit moves over gaps (the head keeps its
  speed), and overscan before and after with the laser off. One-way rows
  travel back to the next row's start.
- `image.dot_correction` shortens every burn run by that much, half at each
  end, to make up for the beam's width (at most one dot).
- Grayscale runs change power where the level changes; `S` follows in the
  G-code. Power is clamped to the layer and machine maximum as for every
  layer (safety rule 6); `min_power` is clamped to the same ceiling.
- Images on a layer with `output` off are skipped. The job preview shows the
  dots as burn moves.

## Import

- `project.import <file>` also takes PNG and JPEG files (by extension:
  `.png`, `.jpg`, `.jpeg`). The image becomes one shape on the given color
  (default 0), placed with its top-left corner at the document origin, and
  is selected. One undo step deletes it.
- Size: from the file's resolution when it states one (PNG `pHYs` in pixels
  per metre, JPEG JFIF density per inch or per cm); otherwise 254 DPI
  (0.1 mm per pixel). An image larger than the work area is scaled down,
  keeping its proportions, to fit.

## File format

The project file version is 5 (`docs/spec/project-file.md`). Image geometry:

```json
{ "kind": "image", "width": 12.0, "height": 8.0, "pixels": [120, 80],
  "brightness": 0.0, "contrast": 20.0, "gamma": 1.2, "png": "iVBORw0…" }
```

`png` is the base64 of an 8-bit grayscale PNG of the bitmap. A loaded image
keeps its text, and a new one is encoded the same way every time, so saving
an unchanged image gives the same file. `pixels` must match the PNG.

## Canvas and panels

- Images are drawn as their adjusted grayscale bitmap (not dithered, at most
  2048 pixels per side on screen) with a thin border in the layer color;
  faded when the layer's output is off.
- The layer editor's "Images on this layer" section (open when the layer
  has images) holds the method, interval with its DPI, negative, both
  directions and overscan (for line layers; fill layers show them above),
  dot correction, and the halftone screen for halftone.

## Not in M7

Scan angles other than horizontal for images, sketch and newsprint modes,
pass-through, ramp, enhance (sharpening), convert to bitmap, trace image,
masks, cropping, saving the processed bitmap, and BMP, GIF, TIFF, TGA or
WebP import.

## Sources

- Image modes (threshold, ordered, Atkinson, dither, Stucki, Jarvis,
  halftone with cells per inch and angle, grayscale between min and max
  power), negative image, bidirectional scanning, overscan, DPI = 25.4 /
  interval, dot width correction (shortens runs, up to the interval):
  https://docs.lightburnsoftware.com/2.1/Reference/CutSettingsEditor/ImageMode/
- Min power used for grayscale only on GRBL:
  https://docs.lightburnsoftware.com/2.1/Reference/CutSettingsEditor/SharedSettings/
- Adjust Image (per-image gamma, contrast, brightness; Alt+I; Tools menu):
  https://docs.lightburnsoftware.com/2.1/Reference/AdjustImage/,
  https://docs.lightburnsoftware.com/2.1/Reference/UI/ToolsMenu/
- Images on a layer split from vectors, border in the layer color:
  https://docs.lightburnsoftware.com/2.1/Reference/CutsLayersWindow/,
  https://docs.lightburnsoftware.com/2.1/Explainers/ImagesVsVectors/
- Image import formats: https://docs.lightburnsoftware.com/2.1/Reference/FileManagement/
- Scan from the bottom up at angle 0:
  https://docs.lightburnsoftware.com/2.1/Guides/PerfectImageEngraving/
- Error-diffusion kernels: R. W. Floyd and L. Steinberg, "An adaptive
  algorithm for spatial grey scale" (1976); J. F. Jarvis, C. N. Judice and
  W. H. Ninke (1976); P. Stucki (1981); Atkinson as described in
  https://en.wikipedia.org/wiki/Atkinson_dithering and
  https://en.wikipedia.org/wiki/Error_diffusion
- Bayer matrix: B. E. Bayer (1973); construction and threshold as in
  https://en.wikipedia.org/wiki/Ordered_dithering
- Clustered-dot halftone screens: https://en.wikipedia.org/wiki/Halftone
- Luminance weights: ITU-R Recommendation BT.601.
- PNG: W3C/ISO 15948 (`pHYs`); JPEG: ITU-T T.81; JFIF 1.02 (APP0 density).
- All read 2026-10-21. Own design: grayscale storage, the size and pixel
  limits, the adjustment formulas and ranges, the document-aligned grid, box
  filter and bilinear sampling, the threshold at 50 %, Floyd–Steinberg as the
  general dither, the cosine spot and its ranking, halftone defaults (50
  cells per inch, 45°), raster order and row planning, the 254 DPI default
  and fit-to-bed import, placement at the origin, the file encoding, and the
  2048-pixel display limit.
