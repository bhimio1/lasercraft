# Spec: importing SVG and DXF

Milestone: M3.

## Command

| Command | Undo | Effect |
| --- | --- | --- |
| `project.import <path> [@color]` | delete | Read an SVG (`.svg`) or DXF (`.dxf`) file and add its vector art as path shapes. |

- The format is chosen by the file extension (case-insensitive). Anything
  else fails with "unsupported file type".
- Every curve becomes a `path` shape (`docs/spec/geometry.md`): lines stay
  lines, cubic Béziers stay cubic, quadratic Béziers are raised to cubics
  exactly, and circular or elliptical arcs become cubic pieces of at most
  90°. Shapes get the identity transform; their coordinates are document
  millimetres.
- One shape per drawn element (an SVG path, rectangle, circle, …; a DXF
  entity, after joining, below). Subpaths of one SVG element stay in one
  shape, so letter counters and other holes stay holes.
- When the file yields more than one shape, the new shapes are put in a new
  group (`docs/spec/arrange.md`), so the import moves as one object; ungroup
  to work on the pieces. The new shapes are selected.
- The import is one undo step; undo deletes the imported shapes.
- A file that yields no shapes fails with "nothing to import" and changes
  nothing. A file that cannot be read or parsed fails with the reason.
- Text in a file is not imported. Convert text to outlines in the program
  that made the file first. Raster images inside an SVG are skipped; PNG and
  JPEG files are imported as image shapes (`docs/spec/images.md`, M7). The CLI and console report how many shapes were added.

## Colors to layers

- Without `@color`, each shape goes to the layer whose palette color is
  closest to the element's color (smallest squared distance in RGB; ties go
  to the lower color number). An exact match always wins, so artwork drawn
  with LaserCraft's palette colors lands on exactly those layers. The
  palette is LaserCraft's own (`crates/lasercraft-project/src/layer.rs`).
- SVG: the stroke color is used when the element has a plain-color stroke,
  otherwise its plain fill color; gradients, patterns and no paint at all
  give layer 0.
- DXF: the entity's ACI color number (group code 62) is used; "by layer"
  (256) takes the color of the entity's layer from the LAYER table; "by
  block" (0) and missing colors take the inserting entity's color, or
  layer 0's default (7, black/white → palette black). ACI numbers map to RGB
  approximately: 1–9 the named colors, 10–249 hue steps of 15° in five
  shades with full and half saturation, 250–255 grays from dark to white.
- With `@color`, every imported shape goes to that layer.

## SVG

- Parsed with `usvg`, which resolves CSS, `style` attributes, `<use>`,
  `<symbol>`, nested `<svg>`, inheritance and `transform`s, and turns basic
  shapes (`rect` including rounded corners, `circle`, `ellipse`, `line`,
  `polyline`, `polygon`) and every path command into absolute lines and
  curves.
- Units: user units are CSS pixels at 96 per inch, so 1 user unit =
  25.4 / 96 mm. A root `width`/`height` in physical units with a `viewBox`
  scales the drawing so those units come out exact (`width="100mm"` is
  100 mm wide).
- Position: SVG and LaserCraft both put the origin at the top left with y
  growing downwards, so the page's top-left corner lands on document (0, 0),
  the rear-left corner of the bed in the default setup.
- Hidden elements (`display="none"`, `visibility="hidden"`) are not
  imported. Clip paths and masks are ignored: the full outlines are
  imported (they would otherwise be cut anyway).
- Elements with neither fill nor stroke are still imported (a laser outline
  is usually drawn as a hairline or invisible stroke).

## DXF

- ASCII DXF, read with LaserCraft's own parser of group-code/value pairs
  (binary DXF fails with "binary DXF is not supported").
- Units: header `$INSUNITS`: 1 inch, 2 foot, 4 mm, 5 cm, 6 m, 8 microinch,
  9 mil, 10 yard, 14 decimetre; 0 (unitless) or missing means millimetres;
  other values fail with "unsupported DXF units".
- Entities read in model space and inside inserted blocks:
  `LINE`, `LWPOLYLINE` (with bulges), `POLYLINE`/`VERTEX` (2D, with
  bulges), `CIRCLE`, `ARC`, `ELLIPSE` (full and partial), `SPLINE`, and
  `INSERT` (insertion point, x/y scale, rotation; nested up to 8 deep; the
  block's base point is honored; column/row arrays are not). A negative
  extrusion direction (z of group 230 < 0) mirrors an entity's x, as DXF's
  object coordinate system requires for 2D entities. Text, dimensions,
  hatches, points and 3D entities are skipped.
- Splines: rational and non-rational B-splines are evaluated with de Boor's
  algorithm at 16 points per knot span and imported as polylines. A spline
  with only fit points is imported as the polyline through them.
- Bulge `b` on a polyline vertex makes the segment to the next vertex a
  circular arc with included angle `4·atan(b)`, counter-clockwise for
  positive `b` (in DXF's y-up space).
- Position: DXF y grows upwards. The drawing is mirrored so that DXF
  (x, y) lands on document (x, H − y), where H is the work-area height:
  the DXF origin is the front-left corner of the bed, as on a machine with
  a front-left origin.
- Joining: open pieces of the same color whose ends meet within 0.01 mm are
  joined into one path, and a joined path whose ends meet is closed. Each
  joined result is one shape. CAD programs often draw a closed outline as
  separate lines and arcs; joining makes it closed so it can be filled or
  used in booleans.

## Sources

- Import, its shortcut (Ctrl+I) and the advice that text must be outlined
  before importing: LightBurn docs, "Creating and Importing Artwork",
  https://docs.lightburnsoftware.com/2.1/GetStarted/CreatingAndImportingArtwork/
- Imported shapes grouped and selected by default; DXF unit detection and
  closing nearly-touching ends on import; SVG pixel density settings:
  LightBurn docs, "Settings",
  https://docs.lightburnsoftware.com/2.1/Reference/SettingsPreferences/ and
  https://docs.lightburnsoftware.com/2.1/Reference/Grouping/
- Matching imported colors to the layer palette: LightBurn legacy docs,
  "Layer Colors", https://docs.lightburnsoftware.com/legacy/LayerColors
  (only the behavior is taken: LaserCraft matches against its own palette;
  LightBurn's color values are not used).
- All read 2026-10-21.
- SVG 1.1: paths, basic shapes, coordinate systems, units and `viewBox`:
  https://www.w3.org/TR/SVG11/paths.html, https://www.w3.org/TR/SVG11/shapes.html,
  https://www.w3.org/TR/SVG11/coords.html; CSS pixel = 1/96 in (CSS Values 3).
- DXF: Autodesk DXF reference (group codes, `$INSUNITS`, LINE, LWPOLYLINE,
  POLYLINE/VERTEX, CIRCLE, ARC, ELLIPSE, SPLINE, INSERT, LAYER table, ACI
  colors, bulge, object coordinate system).
- Nearest-color rule, grouping only multi-shape imports, document placement
  of DXF drawings, 0.01 mm join tolerance, spline sampling density: own
  design.
