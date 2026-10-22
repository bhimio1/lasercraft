# Spec: cameras

Milestone: M9. A camera above the bed shows the material under the design
on the canvas (the **overlay**), so a design can be placed on a part that
is already in the machine. Before it can do that, each camera's **lens
correction** (how its lens bends straight lines) and its **bed alignment**
(where the picture lies on the bed) must be known.

Scope of M9: cameras fixed **above the bed**, USB or network, one or more
per device profile. Cameras mounted on the laser head (stitched captures)
are later (Open questions).

## Camera data

Cameras belong to a device profile (`docs/spec/devices.md`): the profile
gains `cameras`, a list of:

| Key | Meaning |
| --- | --- |
| `name` | Unique within the profile; 1–60 characters |
| `source` | `{"kind": "usb", "device": <name>, "index": <n>, "format": "<w>x<h>"}`, `{"kind": "network", "url": <http(s) URL>, "rate": <pictures per second>}` or `{"kind": "simulated"}` |
| `mount` | `bed` (above the bed); `head` is refused until head-mounted cameras exist |
| `lens` | The lens correction or `null` (below) |
| `alignments` | List of bed alignments (below); the first is in use |
| `use` | Included in the overlay (default true) |
| `opacity` | Overlay opacity, 0–100 % (default 70) |
| `fit` | `{"width": %, "height": %, "dx": mm, "dy": mm}`, fine adjustment of the overlay (default 100, 100, 0, 0) |
| `skip` | Pictures to drop after the source starts (default 3) |

Profiles without `cameras` read as an empty list. A camera file for
exchange (`.lccam`) holds one camera: `{"format": "lasercraft-camera",
"version": 1, "camera": {…}}`.

## Lens correction

A pinhole camera with lens distortion. Picture pixels `(u, v)` and
normalized camera rays `(x, y)` are related by the focal lengths `fx, fy`,
the center `cx, cy` and a **model**:

| Model | Shown as | Distortion of a ray `(x, y)`, `r² = x² + y²` |
| --- | --- | --- |
| `standard` | Standard | radial `k1, k2` and tangential `p1, p2` (Brown–Conrady) |
| `wide` | Wide angle | as standard plus radial `k3` |
| `fisheye` | Fish-eye | equidistant: angle `θ = atan(r)`, `θd = θ(1 + k1θ² + k2θ⁴ + k3θ⁶ + k4θ⁸)` (Kannala–Brandt) |

Stored as `{"model": …, "width": w, "height": h, "fx", "fy", "cx", "cy",
"k": [...], "p": [...], "rms": <pixels>, "created": "<RFC 3339>"}`. A
picture of another size is scaled to `width × height` first.

### Calibration

1. The user prints LaserCraft's **calibration sheet** (own design, written
   by `camera.pattern`): a grid of 9 × 7 black dots on white, pitch 20 mm
   at 100 % print size, dot diameter 8 mm, with three larger dots (12 mm)
   at grid positions (4, 3), (5, 3) and (4, 4) that show the sheet's
   center, its x direction and its y direction. The printed size does not
   matter (the lens correction does not depend on it); only flatness and
   contrast do.
2. Pictures are taken with the sheet in five places: the four corners of
   the picture and its center ("Hold the sheet in the top-left part of the
   picture", …). More pictures may be added; each must show at least 20
   dots and the three large ones.
3. **Dot finding**: the picture is turned to gray, thresholded against its
   local mean (a 31-pixel window), and dark connected regions are kept
   when they are roughly round (filled at least 60 % of their bounding
   ellipse) and of a plausible size. Their centers are the region's
   centroid. The three large dots are the three largest regions that form
   a right angle with matching spacing. From them the grid grows outwards:
   each next dot is predicted from its neighbours and accepted when a
   found dot lies within 30 % of the local pitch.
4. **Fitting** (Zhang's method): a homography per picture from the
   sheet's grid coordinates to pixels, a closed-form first guess of `fx,
   fy, cx, cy`, then all intrinsics, distortion terms and each picture's
   pose refined together by Levenberg–Marquardt to minimise the
   reprojection error. The result's `rms` is that error in pixels.
   Pictures of a sheet lying flat, all seen from straight above, cannot
   separate the focal length from the camera's height; weak priors keep
   the focal length near the first guess (or the picture's longer side),
   the pixels square and the center near the picture's middle. The
   correction of straight lines, which is what the overlay needs, does
   not depend on that choice.
5. All three models are fitted; the wizard shows each one's **fit** as
   `100 / (1 + rms)` percent ("Higher is better, but compare the pictures
   too") and the user picks one. Five pictures with too few dots, a fit
   that does not converge or an `rms` above 5 pixels is refused with a
   reason.

## Bed alignment

A projective map (homography) from lens-corrected picture coordinates to
document millimetres on the bed, stored as `{"name": …, "height": <mm>,
"matrix": [9 numbers], "lens": "<created of the lens correction used>",
"width": w, "height_px": h, "created": …}`.

A camera without a lens correction ("No lens correction" in the wizard)
uses its pictures as they are: alignment and overlay treat it as a lens
without distortion, `lens` is empty and `width`/`height_px` are 0 when
the picture size was not known (then the size of each picture is used).

1. LaserCraft burns the **alignment pattern** (own design): four marks,
   each a filled ring (outer diameter the mark size, inner diameter 60 %
   of it) with a filled dot (25 %) at its center, centered on the corners
   of a rectangle and numbered clockwise from the top left, plus one dot
   at the rectangle's center. The marks are filled (engraved), not
   traced, so they are thick enough for the camera to see. The rectangle
   is centered on the bed and is 70 % of the bed by default (Pattern width
   (mm), Pattern height (mm), Mark size (mm), default 12; the pattern must
   be at least three marks wide and high and fit the bed).
   The burn is a job like any other: Absolute coordinates, the bounds
   check, power clamped to the device maximum, line mode, default speed
   and power from a "Camera alignment" entry of the open material library
   when there is one, else 100 mm/s at 20 % in fill mode at 0.1 mm
   (safety rules 1, 5, 6).
2. **Find the marks**, either:
   - automatically: dark regions not touching the picture's edge with
     exactly one hole (a ring) that holds a dark dot, the four largest of
     similar size, ordered clockwise from the one nearest the picture's
     top-left; their centers are the regions' darkness-weighted centers;
     or
   - by clicking: the user clicks the four mark centers in order (top
     left, top right, bottom right, bottom left) on the zoomable,
     lens-corrected picture; Discard removes the last one.
3. The homography is computed from the four lens-corrected points to the
   four mark centers on the bed (direct linear transform, normalized).
   Four points that are not a convex quadrilateral in the same turning
   order as on the bed are refused ("The marks are not in order. Click
   them clockwise from the top left.").
4. The check page shows the corrected, aligned picture with a 10 mm grid.
   The center dot must land on the bed point it was burned at.
5. `height` is the material height the alignment was made at (default 0);
   a new alignment replaces one with the same height. The first alignment
   in the list is in use; others can be chosen.

An alignment made with another lens correction than the camera's current
one is marked "made with another lens correction" and still used.

## Overlay

- Built from each used camera's latest picture: for every overlay pixel
  (a bed grid of 0.5 mm, at most 2000 pixels on the longer side) the
  alignment maps the bed point back to a corrected picture point, the lens
  model distorts it to a raw pixel, and the color is sampled bilinearly.
  Bed points the camera does not see are transparent.
- `fit` then scales the overlay about the bed center and shifts it.
- Cameras are drawn in list order; later ones cover earlier ones.
- The overlay is a view, never part of the project, and never sent to the
  laser.
- **Capture camera image** (Alt+C) takes a new picture from every used
  camera and rebuilds the overlay; with shapes selected, only the
  selection's bounding box (plus 5 mm) is replaced.
- The picture age shows when it was taken; a picture taken before the head
  last moved (a jog, frame or job) is marked stale (the head may hide
  part of the bed).

## Sources (picture providers)

Run in the app on a thread per camera; the UI only gets pictures.

- **USB**: through the operating system's camera interface (Video4Linux2
  on Linux, Media Foundation on Windows, AVFoundation on macOS). The
  device list gives names and formats; the camera's own controls
  (exposure, focus, white balance, brightness, contrast, hue, backlight
  compensation) are shown when it reports them.
- **Network**: an `http://` or `https://` URL that returns either one JPEG
  picture per request (polled at `rate`) or a motion-JPEG stream
  (`multipart/x-mixed-replace`).
- **Simulated** (own design, for tests and demos, like the simulated GRBL
  port): renders the bed (a light surface with a 10 mm grid) through a
  known lens and pose, with everything the simulated laser has burned
  drawn in dark brown; while the lens wizard asks for a position, the
  calibration sheet is shown there.
- A source that fails shows "No picture source: ‹reason›"; the app retries
  every 5 seconds while the camera is shown.

Pictures never leave the computer.

## Commands

Camera data is user data (saved at once, not undoable), as device
profiles. Every camera command acts on the project's device profile; with
none: "Cameras belong to a device profile. Choose one in the Machine
panel first."

| Command | Effect |
| --- | --- |
| `camera.add <name> <usb\|network\|simulated> [device=… index=… format=WxH \| url=… rate=…]` | Add a camera. |
| `camera.remove <name>` | Remove it with its correction and alignments. |
| `camera.rename <old> <new>` | |
| `camera.set <name> <key> <value>` | `use`, `opacity`, `fit_width`, `fit_height`, `fit_dx`, `fit_dy`, `skip`, `device`, `index`, `format`, `url`, `rate`. |
| `camera.pattern <path.svg>` | Write the calibration sheet. |
| `camera.calibrate <name> <standard\|wide\|fisheye> <picture> …` | Find the sheet in each picture file (PNG or JPEG), fit the model and store the lens correction. The outcome names the fit and how many dots each picture gave. |
| `camera.set_lens <name> <json\|path.lccam\|none>` | Store a lens correction (the wizard's choice, or the one in a camera file) or remove it. |
| `camera.align <name> <json>` | Store an alignment from `{"picture": [[u, v] ×4], "bed": [[x, y] ×4], "height": h}`, picture points raw (the lens correction is applied). |
| `camera.align_find <name> <picture> [pattern json]` | Find the four marks automatically in a picture file and store the alignment. |
| `camera.use_alignment <name> <alignment name>` / `camera.remove_alignment <name> <alignment name>` | |
| `camera.import <path>` / `camera.export <name> <path>` | `.lccam` files; a clashing name gets " (2)", … |
| `camera.overlay <name> <picture> <out.png>` | Write the overlay of one picture (headless check). |
| `camera.list` | One line per camera with its source, correction and alignment state. |
| `device.camera_pattern [json] [speed=<mm/s>] [power=<%>]` | Burn the alignment pattern on the connected laser (a job; Absolute coordinates). `json`: `{"width", "height", "mark"}` mm. |
| `device.camera_pattern_frame [json]` | Frame it with the laser off. |
| `image.insert_picture <path> <x> <y> <w> <h>` | Add a picture file as an image shape covering that bed rectangle (used by "Insert picture"); undoable. |

The CLI gains `camera pattern <out.svg>`, `camera calibrate <model>
<pictures…>` (prints the lens correction as JSON) and `camera overlay
<lens.json> <alignment.json> <picture> <out.png>`.

## In the app

As `docs/spec/style/later-windows.md`, "Camera (M9)", with these choices:

- Camera panel (upper dock group, after Console; Window → Panels lists
  it): Capture camera image, Trace camera image… (disabled: "Tracing
  comes later."), Overlay opacity (%), Overlay fit, picture age, one block
  per camera (header with streaming dot, capture, use, name and the three
  badges; live picture; Picture settings when the camera reports
  controls; Advanced with Preview rate and Insert picture Corrected / As
  captured), Add camera…, Import camera….
- Camera list (Machine → Camera list…), Add camera (wizard: kind, source
  with live view, mounting (Above the bed only; On the laser head
  disabled with "Later"), lens correction, the lens and alignment pages,
  name), Camera setup (four groups and the live view with a 10 mm grid),
  the lens calibration and camera alignment pages as the style guide
  draws them. The alignment burn page follows "Jobs started from these
  windows" item by item.
- The canvas draws the overlay under the grid and the shapes
  (`docs/spec/style/canvas.md`).

## Open questions

- Head-mounted cameras (stitching several pictures while jogging).
- AprilTag-style automatic marks; our marks are found by shape only.
- Alignments interpolated between material heights.
- Tracing the overlay into shapes (needs Trace to vectors).
- USB camera controls: the app does not read them yet, so Picture settings
  says the camera has no adjustable controls.
- Moving the alignment pattern by dragging it on the bed drawing (today
  only its size is set; it stays centered).

## Sources

- Camera purposes, lens calibration once per camera, alignment redone when
  the camera moves, five pictures (four corners and center), several
  models with a score that is only a hint, alignment by burning a pattern
  of four crosshair marks found automatically or clicked in order,
  absolute placement, a check view with a grid, alignments per material
  height, several cameras, network cameras:
  https://docs.lightburnsoftware.com/2.1/Reference/Cameras/,
  https://docs.lightburnsoftware.com/2.1/Reference/Cameras/Calibration/,
  https://docs.lightburnsoftware.com/2.1/Reference/Cameras/Alignment/,
  https://docs.lightburnsoftware.com/2.1/Reference/Cameras/CamerasWindow/
  (read 2026-10-22).
- Z. Zhang, "A Flexible New Technique for Camera Calibration", IEEE PAMI
  22(11), 2000 (homographies, closed-form intrinsics, refinement).
- D. C. Brown, "Decentering Distortion of Lenses", Photogrammetric
  Engineering 32(3), 1966 (radial and tangential terms).
- J. Kannala and S. S. Brandt, "A Generic Camera Model and Calibration
  Method for Conventional, Wide-Angle, and Fish-Eye Lenses", IEEE PAMI
  28(8), 2006.
- R. Hartley and A. Zisserman, *Multiple View Geometry*, 2nd ed., ch. 4
  (normalized direct linear transform for homographies).
- K. Levenberg (1944) and D. Marquardt (1963), damped least squares.
- Motion JPEG over HTTP: `multipart/x-mixed-replace` (RFC 2046 multipart
  syntax); JPEG: ITU-T T.81; USB Video Class 1.5 controls.
- Own design: the calibration sheet and its dot finding, the ring-and-dot
  alignment marks and their finding, the weak priors that settle the
  focal length when every picture looks straight down at a flat sheet, the fit score, the data formats, the simulated
  camera, the command set.
