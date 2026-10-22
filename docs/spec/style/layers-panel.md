# Spec: style — layers panel, layer settings and palette strip

Milestones: M2 (Layers panel, layer editor and palette strip exist today and
are restyled here), M4 (kerf and tab fields), M5 (Constant power (M3)
switch, missing from the editor today), M6 (the constant-power note ties in
with the job bar's Pause wording), M7 (image rows, the image section,
readable method names), M8 (default buttons, Materials applying to a
layer); column headers, the Air column, the icon button column, Copy and
Paste settings, context menus, drag to reorder, guide swatches, the Layer
settings window and Operations are later (some are owner decisions, marked
as such).

This file fixes what the Layers panel, the layer editor under its list, the
Layer settings window and the Palette strip contain, in which order, how
each part looks (through tokens only), how it behaves and what it says.
Where these regions sit and how big they are is `layout.md`'s (right dock,
palette strip band, Simple layout); every label comes from `naming.md`;
colors, type and sizes are the tokens defined in `visual-language.md`; icon
artwork is `icons.md`'s (this file only names each icon and its concept);
the two context menus of the layer list are specified item by item in
`menus.md` (Layer menu, All layers menu) and only summarised here; dialog
frames, button order and the rule that no window covers the job bar are
`dialogs.md`'s; how shapes and moves are drawn in layer colors is
`canvas.md`'s and `job-preview.md`'s, summarised in the last section here;
the Materials panel is in `later-windows.md`; the job bar's constant-power
wording is in `machine-panels.md`. `README.md` explains the tiers (Match,
Echo, Own) used below. The behavior rules of layers themselves stay in
`docs/spec/layers.md`, which wins wherever it already decides something.

## How to read this file

- Every element has a **Reference** part (what the public docs show, each
  fact tagged with a source key such as [R1], listed under Sources), a
  **LaserCraft** part (the binding spec) and a **Tier** line.
- Milestone tags: *Now* = exists today (possibly needing the restyle this
  file describes); *M6*–*M10* as in `CLAUDE.md`; *Later* = no milestone yet;
  *owner decision* = needs the owner before it is built.
- Commands are the registry commands of `docs/spec/layers.md` and
  `docs/spec/commands.md`. Every change a widget makes is one of them; the
  UI never edits the project directly (CLAUDE.md, architecture rules).
- "The edited layer" is the layer whose row is selected in the list and
  whose settings the layer editor shows. "The current color" is the palette
  color new shapes get (`docs/spec/layers.md`). They are separate states.

---

## Palette strip

### Reference

- A strip of numbered color chips; each chip stands for one layer. The
  colors only tell operations apart and say nothing about the finished
  piece [R2][R3].
- By default it is a single toolbar row across the bottom of the main
  window, directly above the status row, with the chips packed to the left
  [R2][R13][R14]. It is a dockable toolbar: the docs also show it as a
  one-chip-wide column between the left tools and the drawing area, and it
  carries a dotted grip at its start [R2][R14]. If closed, it comes back from
  the window menu or a layout reset [R2][R18].
- Contents in order: thirty chips numbered with two digits from "00" to
  "29", then two special chips for non-output helper layers [R2][R14]. The
  window-menu page words the count differently from what the screenshots
  show [R18].
- Each chip is a small flat filled shape with its two-digit number centered
  in a small plain font; the number is light on dark chips and dark on light
  chips [R2][R14].
- The current chip has a visible frame around it [R2][R3]. One animation
  frame hints at a hover effect; the text does not confirm it [R2].
- When the window is too narrow, the chips at the right end are cut off and
  an overflow chevron appears at the strip's end [R3][R17].
- Clicking a chip with nothing selected picks the layer for newly drawn
  shapes; with shapes selected it moves them to that layer, and that layer
  then shows in the layer list [R2][R3][R14].
- The docs mention a habit of using red for cutting and black for engraving,
  while leaving the meaning of each color to the user [R2].
- Imported art is matched to the nearest palette color, so designs drawn in
  the palette's colors in other programs land on the intended layers [R2].
- The two special chips are layers with no settings that are never sent to
  the machine, for helper geometry such as a material outline, alignment
  marks, a path for text or an image mask [R2].
- The 2.1 changelog mentions the chips being redrawn when the font scale
  changes and a fix for chips growing larger than tool buttons [R15].
- Not described: right-click, double-click, Shift+click or dragging a chip,
  chip tooltips, a separator before the special chips [R2].

### LaserCraft

Placement and band: `layout.md` ("Palette strip"): one `row` tall, full
width, directly below the middle band and above the command line, swatches
packed from the left with `gap-tight`, one line only. Shown or hidden from
Window → Toolbars → Palette; F12 does not hide it. *Now* (restyle).

#### Contents in order

| # | Item | Label on it | Tooltip (first line / second line) | Milestone |
|---|---|---|---|---|
| 1–30 | Layer swatches 00 … 29, in color-number order (never in output order) | two-digit number "00" … "29" | "‹nn›: ‹layer name›" / with shapes selected "Click to put the selected shapes on this layer."; with nothing selected "Click to draw new shapes on this layer." | Now (numbers, second line: restyle) |
| — | a `gap-wide` space | — | — | Later (with guides) |
| 31 | Guide swatch 1 | guide pictogram, then "1" | "Guide layer 1: ‹name›" / "Shapes here are never output." | Later, owner decision |
| 32 | Guide swatch 2 | guide pictogram, then "2" | "Guide layer 2: ‹name›" / as above | Later, owner decision |
| end | More (only when swatches do not fit) | "»" | "More" | Now (replaces wrapping) |

- The swatch order is fixed by color number, so a color is always in the
  same place on the strip, whatever the project's output order (own design;
  the order of the list is output order, the strip is a key).
- "‹layer name›" is the layer's `name` (default "Layer NN",
  `docs/spec/layers.md`); the number in the tooltip is always two digits
  ("07: Layer 07", "07: Engrave logo").

#### Swatch look

| Part | Look | Tokens |
|---|---|---|
| Shape | a square with slightly rounded corners, a little less than one `row` tall so the strip has a thin margin above and below; all 30 the same size | — |
| Fill | the layer's palette color, flat, no gradient, no shading | palette data (`lasercraft_project::layer::palette`) |
| Number | two digits, centered, never bold | `type-small`; black or white per swatch (rule below) |
| Normal outline | a thin line just outside the fill, so pale colors stay readable against the strip | `border` |
| Hovered | outline turns darker; no raised or shadow effect | `text` (thin) |
| Pressed | fill unchanged; outline as hovered plus `pressed` behind the swatch's margin | `pressed` |
| Current color | a thick outline outside the swatch with a thin light ring between it and the fill, so the mark reads on swatch 00 (black) as well as on pale colors | outer `text`, inner ring `field` |
| Keyboard focus (later) | focus ring around the swatch, outside the current-color mark | `accent` |

- **Number contrast rule** (own design, using the WCAG 2.x definitions of
  relative luminance and contrast ratio): for each swatch, compute the
  contrast ratio of black text and of white text against its fill and use
  whichever is higher. Because one of the two always reaches at least 4.5:1,
  every number meets WCAG's normal-text minimum. The choice is computed from
  the palette data in code (in `lasercraft-ui`'s `colors` module), never
  listed by hand.
- **Numbers are optional later**: the Display preference "Show swatch
  numbers" (naming.md; Later) hides the digits; the default is on.
- **Size follows the UI scale**: swatches are sized from the `row` token, so
  they grow and shrink with the text-size preference and the display scale,
  and are never larger than a toolbar button (own design; compare the
  reference's changelog fix [R15]).
- The current-color mark is not `accent`: `accent` is reserved for the uses
  listed in `visual-language.md` (README.md, accent decision), and a black
  and white mark reads on every palette color.
- Tier: **Echo** (numbered square swatches with contrast numbers and an
  outlined current color), **Own** (exact look, contrast rule, ring).

#### Guide swatches (Later, owner decision)

- Two extra swatches after a `gap-wide` space. Each shows a small guide
  pictogram (own concept: a dashed line with a set square corner; artwork in
  `icons.md`, id `guide-layer`) followed by "1" or "2" in `type-small`, so
  they cannot be mistaken for numbered output layers or for G-code words
  (naming.md, decisions log).
- Their fill colors are the proposed tokens `guide-layer-1` and
  `guide-layer-2` (`visual-language.md`, own design); they differ clearly
  from all 30 palette colors and from the canvas tokens `guide`,
  `selection` and `drawing`. The same tokens stroke the guide shapes on the
  canvas (`canvas.md`). The swatch number rule (contrast-chosen digit) does
  not apply: the pictogram and digit are drawn in `text` on the token fill.
- They follow every interaction rule below, except that their layers have
  no settings to edit and are never output, previewed or framed.
- Needs a proposed change to `docs/spec/layers.md` (it has 30 colors only).
- Tier: **Echo** (two non-output helper swatches at the end), **Own**
  (pictogram and labels).

#### Overflow

- The strip never wraps (today it does, `layers_panel.rs:53`). When the
  window is too narrow, the last swatches fold into a More button ("»") at
  the right end, as described in `layout.md` ("Toolbar model", "Narrow and
  short windows" step 3). The More popup shows the folded swatches in one
  row, same look, same interactions, same tooltips.
- The current color is never hidden: if the current swatch would fold, the
  More button itself shows the current-color mark (own design).
- Tier: **Echo** (overflow marker at the end), **Own** (popup, current mark
  on More).

#### Vertical strip (Later; docking stage b)

- When docked at the left or right edge (needs the docking crate,
  `layout.md` "Docking model"), the strip becomes one swatch wide with the
  swatches stacked top to bottom in the same order, guides at the bottom.
  Same look and interactions. Tier: **Match** (place option), **Own** (look).

#### Interactions

The first two rows are `docs/spec/layers.md`'s rules and are binding; the
rest are own design.

| Gesture | Effect | Command | Milestone |
|---|---|---|---|
| Click, shapes selected | Move the selected shapes to this layer; the current color does not change | `shape.set_layer ‹c›` | Now |
| Click, nothing selected | Make this the current color for new shapes; the mark moves | none (UI state) | Now |
| Shift+click | Select every shape on this layer (replacing the selection); a layer with no shapes does nothing | `layer.select ‹c›` | Later |
| Double-click | Open Layer settings on this layer (the first click has already acted as a click), only when the layer has shapes; otherwise it acts as a click | none (opens a window) | Later |
| Right-click | Open the Swatch menu (below) | — | Later |
| Hover | Tooltip after the usual delay (naming.md, Tooltips) | — | Now |
| Drag | Nothing (swatches cannot be reordered or dragged onto shapes) | — | — |

Swatch menu (right-click on a swatch; own design; labels are proposed
additions to `naming.md`, see Open questions):

| Item | Enabled when | Effect |
|---|---|---|
| Use for new shapes | always | Make this the current color without moving the selection |
| —— | | |
| Select shapes | the layer has shapes | `layer.select ‹c›` |
| Highlight shapes | the layer has shapes and Show is on | the same front-end action as the Layer menu's item (`menus.md`) |
| —— | | |
| Layer settings… | the layer has shapes (Later, with the window) | opens Layer settings on this layer |

- "Select shapes" and "Highlight shapes" reuse the Layer menu's labels and
  actions (`menus.md`), so one word means one thing.
- Tier: **Match** (click rules), **Own** (all other gestures and the menu).

#### Palette data (decision 5)

- LaserCraft keeps its own computed palette (`lasercraft_project::layer::
  palette`, `crates/lasercraft-project/src/layer.rs:41-51`): color 00 is
  black, then the hue wheel in even steps starting at red, alternating a
  bright and a darker shade so neighbours differ. This file does not change
  the data and never lists values.
- The reference's swatch order and colors are not adopted (README.md,
  palette decision). By coincidence our 00 is black and 01 a bright red, so
  the habit the docs mention (red to cut, black to engrave) still works with
  the first two swatches; LaserCraft does not suggest any meaning for a
  color in its UI text.
- Import keeps using `nearest_color` (`docs/spec/import.md`).
- Known issues are owner questions (Open questions 1–3).
- Tier: **Own**.

---

## Layers panel

### Reference

- The reference's layer list sits in the upper part of the right-hand dock
  by default, sharing a tabbed group whose tab strip runs along the bottom;
  the job-control window sits below it [R1][R13][R14]. It is reopened from
  the window menu or a layout reset [R1].
- Top to bottom: the dock's title strip, a table of layers (most of the
  height), a narrow column of icon buttons on the table's right, and a band
  of quick-edit fields under the table [R1][R13][R14].
- One row per layer used in the project, in run order; a color holding both
  images and vector shapes shows two rows, one for each kind [R1][R3].
- Run order is top to bottom by default unless the optimisation settings
  reorder it; rows are reordered by dragging or with up and down buttons
  [R1][R3].
- The 2.1 release made the window more compact, keeps the list's scroll
  position and the selected row in view, and shows power with one decimal
  when it is under 100 [R15].
- Double-clicking a layer opens the full settings editor; a hover hint on
  the list says so [R1][R4][R12].

### LaserCraft

Placement: `layout.md` ("Right dock"): the Layers panel is the first tab of
the right dock's upper group and is in front by default; until M6 it fills
the whole right dock. Panel title "Layers" in the panel's title strip
(`layout.md`, "Title strip"); the title strip replaces today's
`ui.heading("Layers")` (`layers_panel.rs:78`).

Anatomy, top to bottom (Later parts drawn; they are absent until built):

```
+--------------------------------------------------+
| Layers                                      [x]  |  title strip (layout.md)
+-------------------------------------------+------+
| Color | Name      | Mode   | Speed, power |  ... |  header row      button
|  [00] | Layer 00  | Line v | 10 / 20%     |  ... |  layer rows      column
|  [03] | Engrave   | Fill v | 200 / 35%    |  ... |  (list area,     [^]
|  [07] | Photo     | Image  | 150 / 40%    |  ... |   scrolls)       [v]
|                                           |      |                  [bin]
|                                           |      |                  [cp]
|                                           |      |                  [ps]
+-------------------------------------------+------+
| [03]  Name  [ Engrave                        ]   |  layer editor
|  Mode [Fill  v]          Speed (mm/s) [ 200 ]    |  (two label/field
| Passes [ 1 ]          Max power (%) [ 35 ]       |   pairs per line)
| Air assist (o)   Constant power (M3) (o)         |
| Interval (mm) [0.1]     Scan angle (°) [ 0 ]     |
| ▸ Images on this layer                           |
+--------------------------------------------------+
| Layers | Jog | Console | ...                     |  tab strip (layout.md)
+--------------------------------------------------+
```

(The diagram shows arrangement only; the Output, Show and Air columns sit
where "..." is drawn. The title strip has only the close button in docking
stage a; the float button joins it in stage b, as `layout.md` says.)

- The panel body is `panel`; the list area and the fields are `field`. A
  thin `border` line separates the list area from the layer editor.
- The panel body is one vertical scroll area (`layout.md`, "Narrow and short
  windows"); the list area has its own height rule (below) and scrolls
  inside when it has more rows than fit.
- Tier: **Match** (list with side buttons above a quick-edit form, in the
  upper right dock group), **Own** (look, names, no color bar).

### Column header row

#### Reference

- One plain header row under the title strip, light background, dark
  regular text, thin separators between cells, no sort markers [R1].
- The reference's columns, in order, are a name column, a color column, the
  mode, a speed and power summary, and three switches for output, visibility
  and air assist (air only on gantry machines) [R1][R13][R14].
- Right-clicking anywhere on the headers opens a menu that switches output,
  visibility or air on, off or inverted for every layer, and one that
  reorders layers so cutting runs last [R1].

#### LaserCraft (Later; needs column headers)

| # | Header (exact, naming.md) | Header tooltip | Cell widget | Cell width | Alignment |
|---|---|---|---|---|---|
| 1 | Color | "Layer color and number, as in the palette" | swatch chip with its number | one chip plus `gap-tight` | centered |
| 2 | Name | "Layer name. Click a row to edit the layer below." | text label | takes the remaining width (stretches) | left |
| 3 | Mode | "How the layer runs: Line traces outlines, Fill scans inside closed shapes" | compact dropdown, or plain text for Image and Mixed rows | wide enough for the longest mode name plus the dropdown marker | left |
| 4 | Speed, power | "Speed (mm/s) and Max power (%)" | read-only text | fits "10000 / 100%" in `type-body` | right |
| 5 | Output | "Include the layer in the job preview, G-code and jobs" | switch | one switch plus `gap` | centered |
| 6 | Show | "Draw the layer on the canvas" | switch | as Output | centered |
| 7 | Air | "Air assist for the layer" | switch | as Output | centered |

- LaserCraft puts the color first and the name second (glossary order);
  the reference has the name first (Echo, deliberate difference).
- Look: header cells on `field-alt` (the table header surface of
  `visual-language.md`), text in `type-small` and `text-muted`,
  regular weight; a thin `border` line under the row; thin `border`
  separators between header cells only (not between body cells). No sort
  markers: clicking a header does nothing (output order is the only order).
- The header stays at the top of the list area while the rows scroll
  (sticky header).
- When the panel is too narrow for all columns, the Name column shrinks
  first (names end in "…"); below that, the Speed, power column is dropped,
  then Mode (its value stays editable in the layer editor). Color, Output
  and Show never drop; Air drops before Mode (own design).
- Right-click anywhere on the header row opens the All layers menu
  (`menus.md`, "All layers menu"): output, show and air all on, all off or
  inverted, then Put line layers last. Each row there is one `edit.batch`,
  so one undo restores every switch.
- Air column (Later, as in naming.md): the editor already has Air assist
  now; the column joins when the header row is built.
- Tier: **Echo** (the same seven pieces of information, order adapted),
  **Own** (labels, look, narrow rules).

### Layer rows

#### Reference

- Rows about as tall as the mode dropdown, on a white background with no
  grid lines or stripes; empty space below the last row stays blank [R1].
- The name cell shows the layer name (default a letter plus the two-digit
  number), the color cell a chip matching the palette swatch with its number
  [R1].
- Vector rows have a mode dropdown with three outline and fill modes; image
  rows show their mode as fixed text; layers with several sub-layers show a
  combined mode word and are edited only in the editor [R1][R10].
- The summary shows speed and power separated by a slash; it is edited
  through the fields under the list or in the editor [R1][R12].
- Output, visibility and air are small pill switches; layers not set to
  output are drawn faded on the canvas; hidden layers still run when output
  is on [R1][R12].
- A hovered row gets a very pale blue band; the selected row is a blue band
  (or a light blue fill with an outline on newer Windows captures), and
  turns light grey when the window loses focus; the chip in a selected row
  is drawn tinted [R1][R3][R12].
- Shift with a left click on a row selects every object on that layer;
  Shift with a right click flashes them [R1].

#### LaserCraft

Which rows exist (binding, `docs/spec/layers.md`): one row per layer that
has shapes, in output order (the job runs top to bottom). Layers with no
shapes have no row. From M7 a color holding both images and vector shapes
has two rows: its vector row, then its image row directly below.

Cells, left to right:

| Column | Shows | Widget and look | Interaction | Milestone |
|---|---|---|---|---|
| Color | the layer's swatch with its two-digit number | a chip: same look as a palette swatch (fill, number contrast rule, thin `border` outline) but a little smaller, so it fits inside one `row` with a margin; never tinted by row selection | part of the row (click, double-click and right-click act on the row) | Now (numbered: restyle) |
| Name | `name`, one line; a long name ends in "…" | `type-body`, `text`; on an image row followed by an image pictogram (`icons.md`, id `image-row`) in `text-muted` | click, Shift+click, double-click (below); tooltip: the full name, then "Shift+click selects its shapes." | Now (restyle) |
| Mode | the readable mode name | vector rows: a compact dropdown one `row` tall with `field` background, thin `border`, a small down marker at its right; the open list is a `field` popup with the item under the pointer on `accent-soft`. Choices: Line, Fill (Concentric fill later). Image rows: the plain word "Image" in `text`, aligned with the dropdown text. Mixed rows (later): the plain word "Mixed" | choosing a value commits at once | Now (labels "Line"/"Fill": restyle) |
| Speed, power | "‹speed› / ‹max power›%" with `speed` in mm/s and `max_power` in %, each shown with at most one decimal and trailing ".0" dropped ("10 / 20%", "12.5 / 7.5%"); when Constant power (M3) is on, a small "M3" tag follows: a `warning-soft` chip, slightly rounded, with "M3" in `type-small` and `text` (never `warning` text, which `visual-language.md` keeps for marks and fills) | read-only text, `type-body`, `text`, right-aligned | none of its own (edit in the layer editor); tooltip "Speed ‹s› mm/s, Max power ‹p›%"; with the tag, second line "Constant power (M3): the laser may stay on while paused." | Now (format: restyle) |
| Output | `output` | switch (look below) | click toggles | Now (checkbox today) |
| Show | `show` | switch | click toggles | Now (checkbox today) |
| Air | `air` | switch | click toggles | Later (column) |

Switch look (README.md switch decision; drawn as in `visual-language.md`):
a small pill one text line tall; on = `accent` fill with the knob at the
right, off = neutral grey with the knob at the left; disabled = faded.
The reference's green and dark-red switch colors are not used (Echo → Own).

Row height and spacing:

- Every row is one `row` tall (the in-row dropdown sets the height); rows
  touch, with no gap and no separator lines between them.
- No alternating stripes (today's grid is striped, `layers_panel.rs:87`);
  the background is `field` (own choice, Match with the reference's plain
  rows).

Row states:

| State | Look | Notes |
|---|---|---|
| Normal | `field` background, `text` | |
| Hovered | `hover` across the whole row | |
| Edited (selected) | `accent-soft` across the whole row, text stays `text`; the chip keeps its true color | one row at most; shown the same whether or not the panel has keyboard focus (own design: the edited layer matters even while the canvas has focus) |
| Edited + keyboard focus in the list (later) | as Edited plus a thin `accent` focus outline around the row | |
| Output off | Name, Mode and Speed, power drawn in `text-muted`; the chip drawn faded, as the canvas fades the layer's shapes (`docs/spec/canvas.md`) | the Output switch shows off |
| Show off | the chip drawn hollow: only its outline in the layer color and the number in `text`, on the row background | the Show switch shows off; mirrors "not drawn on the canvas" |
| Output off and Show off | both looks combined | |
| Guide layer row (later, owner decision) | after all output rows, below a thin `border` line; Mode cell shows "Guide" in `text-muted`; Speed, power, Output and Air cells are empty; Show works | guide layers are never output |

- The chip is never tinted by selection (unlike the reference) so the row
  always shows the true layer color (own design).

Row interactions:

| Gesture | Where | Effect | Command | Milestone |
|---|---|---|---|---|
| Click | anywhere on the row except its dropdown and switches | the row becomes the edited layer; the layer editor shows it | none (UI state) | Now (today only on the name, `layers_panel.rs:121-130`) |
| Shift+click | same area | select every shape on the layer (replacing the selection); the edited layer does not change | `layer.select ‹c›` | Now |
| Double-click | same area | open Layer settings on this layer. Until that window exists, a double-click acts as a single click | none | Later |
| Right-click | anywhere on the row | the row becomes the edited layer, then the Layer menu opens (`menus.md`, "Layer menu": Output only this layer, Hide other layers, Select shapes, Highlight shapes) | per item, `menus.md` | Later |
| Drag | press on the row's Color or Name cell and move vertically | reorder: an insertion line in `text` between rows shows the drop place; the dragged row is drawn on `pressed`; dropping runs one command; Esc cancels | `layer.set_order ‹c› …` with the new full order | Later |
| Mode dropdown | Mode cell | change the mode | `layer.set ‹c› mode line\|fill` | Now |
| Switch click | Output, Show, Air cells | toggle | `layer.set ‹c› output\|show\|air on\|off` | Now (Air later) |
| Hover | list | tooltip per cell (table above); hovering empty space below the rows shows "Double-click a layer to open its settings." once Layer settings exists | — | Later |

- A switch or dropdown click acts on its own row without changing the
  edited layer (own design: toggling output on a layer does not move the
  editor).
- The reference's Shift+right-click flash is not copied; Highlight shapes
  is in the Layer menu (`menus.md`).
- The edited layer follows the canvas: when the selection changes and every
  selected shape is on one layer, that layer's row becomes the edited layer
  (own design). Otherwise the edited layer stays. If the edited layer loses
  its last shape, the first row becomes edited (today's rule,
  `layers_panel.rs:83-85`).
- Scrolling: the list keeps its scroll position across project changes and
  scrolls just enough to keep the edited row visible when it changes
  (Match [R15]).
- Keyboard: no list-specific keys in this file; global chords keep working
  (arrows nudge, Del deletes the selection). See Open questions.
- Tier: **Match** (one row per used layer in output order, two rows for
  mixed image and vector colors, mode dropdown, switches, Shift+click,
  drag and double-click), **Echo** (summary text, switch idea), **Own** (row
  look, states, hollow chip for hidden layers, following the canvas).

### List area height

- The list area is as tall as its rows, but at least four rows and at most
  about half of the panel's height; with more rows it scrolls inside (own
  design). The header stays visible.
- Below the last row, the rest of the list area is plain `field` (Match
  [R1]).
- The side button column always has the full height of the list area.

### Side button column

#### Reference

- A narrow column right of the table, one small square icon button wide,
  with plain black glyphs on light button faces, in three groups separated
  by gaps: up and down (reorder), then delete (the layer and every object
  on it, with a confirmation that Shift skips), then copy and paste of
  layer settings [R1]. Older frames show extra arrow buttons there [R19].
- Copy takes the selected layer's settings; Paste puts them on the layer
  selected afterwards [R1].

#### LaserCraft (icon column Later; today a text button row)

Place: a vertical column directly right of the list area, inside the panel,
as wide as one `icon-small` button plus `gap-tight` on each side; the
buttons start at the top of the header row. Buttons are icon-only, square,
one `row` tall, in the normal button look of `visual-language.md`
(the panel icon button of `visual-language.md`: `field` face, thin
`border`, slightly rounded, `hover`, `pressed`, disabled on a `field-alt`
face with `text-disabled` strokes).

| Order | Button (label, used in tooltips) | Icon concept (`icons.md` id) | Tooltip (first line / second line) | Enabled when | Command |
|---|---|---|---|---|---|
| 1 | Move up | a chevron pointing up (`layer-up`) | "Move up: run this layer earlier" | an edited layer exists and is not the first row | `layer.move ‹c› up` |
| 2 | Move down | a chevron pointing down (`layer-down`) | "Move down: run this layer later" | an edited layer exists and is not the last row | `layer.move ‹c› down` |
| — | `gap-wide` | | | | |
| 3 | Delete shapes | a waste bin (`delete`, the same icon as Delete in the Standard toolbar) | "Delete shapes: remove every shape on this layer" / "Its settings stay. Undo brings the shapes back." | an edited layer exists | `layer.delete ‹c›` |
| — | `gap-wide` | | | | |
| 4 | Copy settings | two overlapping sheets (`settings-copy`) | "Copy settings: remember this layer's settings" | an edited layer exists | none (UI state: remembers the source color) |
| 5 | Paste settings | a clipboard with a sheet (`settings-paste`) | "Paste settings: apply the settings of layer ‹nn› here" / "Every setting except the name." | settings were copied and the edited layer is a different layer | `layer.copy_settings ‹from› ‹c›` |

- Move up and Move down sit touching as a pair, and so do Copy settings and
  Paste settings (Match [R1]).
- Delete shapes never asks for confirmation: `layer.delete` is undoable,
  and LaserCraft only confirms what cannot be undone (`naming.md`,
  Confirmations). This deliberately differs from the reference. The icon
  uses normal strokes, not `danger` (README.md, icons decision reserves
  `danger` for Stop and delete-everything actions).
- Paste settings copies the source layer's settings as they are when Paste
  settings is clicked (that is what `layer.copy_settings` does); the
  tooltip names the source so this is visible. The remembered source is UI
  state, cleared when a different project is loaded.
- Pressing a button does not move keyboard focus away from where it was.
- Today: a horizontal row of three small text buttons under the list
  (`layers_panel.rs:97-109`) with no tooltips; Copy and Paste settings do
  not exist in the UI yet although the command does.
- Tier: **Match** (the five actions, their order and grouping), **Own**
  (icons, labels, no confirmation).

### Layer editor (below the list)

#### Reference

- A band under the table with a slightly different light grey background,
  holding two columns of right-aligned labels beside their fields [R1].
- Left column: a wide flat bar filled with the layer's color, the pass
  count, and the scan-line interval (greyed when the layer is a line layer);
  right column: speed (its unit follows the user's speed unit), maximum
  power, and minimum power, which is only offered for DSP machines or for
  images in the grayscale method [R1][R12][R13].
- Spin boxes with small stacked arrows; an arrow at its limit is drawn
  lighter [R1]. Double-clicking a field selects its value so typing
  replaces it [R12].
- The reduced mode leaves out the minimum power field [R14].

#### LaserCraft

The layer editor (no visible label, naming.md) is the form for the edited
layer. It holds every setting that exists today, so nothing needs Layer
settings until that window is built.

Look (own design):

- Background `panel`, separated from the list area by a thin `border`
  line; fields `field` with a thin `border`.
- Layout: label/field pairs, two pairs per line (left pair, right pair).
  Each label is right-aligned directly left of its field, in `type-body`,
  `text`; units are in the label in parentheses ("Speed (mm/s)"), never as
  a suffix in the field (naming.md, Units). Fields in one grid column share
  one width. `gap` between a label and its field, `gap-wide` between the
  left and right pair.
- When the panel is too narrow for two pairs, the form becomes one pair per
  line in the same reading order (left pair first, then right pair, line by
  line).
- Rows are one `row` tall with `gap-tight` between them.
- Switches sit where a field would, with their label right-aligned on the
  left like any other pair (same alignment rule, own choice; the reference
  usually puts switch labels to the right).
- Disabled pairs show label and field in `text-disabled`; the tooltip says
  why (naming.md, Tooltips).
- A muted hint line (`type-small`, `text-muted`) may follow a field, aligned
  with the field column.

#### Header line

| Part | Look | Behavior |
|---|---|---|
| Chip | the edited layer's swatch with its number, as in the list | none |
| Name | label "Name", single-line text field across the rest of the line | 1–60 characters (`docs/spec/layers.md`); commits on Enter or when focus leaves; Esc restores the stored name; an empty name is refused and the field returns to the stored name |

- No wide color bar (the reference has one); the chip identifies the layer
  (own design).

#### Field order, all modes

| Line | Left pair | Right pair |
|---|---|---|
| 1 | Mode — dropdown: Line, Fill (Concentric fill later). On an image row: plain text "Image" | Speed (mm/s) |
| 2 | Passes | Max power (%) |
| 3 | Air assist — switch | Constant power (M3) — switch |
| 3a | constant-power note band (only while Constant power (M3) is on; spans both pairs) | |

#### Line mode lines (vector rows in Line mode)

| Line | Left pair | Right pair |
|---|---|---|
| 4 | Kerf offset (mm) | Tabs per shape |
| 5 | (empty) | Tab width (mm) — only while Tabs per shape is above 0 (`docs/spec/layers.md`) |

#### Fill mode lines (vector rows in Fill mode)

| Line | Left pair | Right pair |
|---|---|---|
| 4 | Interval (mm), with the hint "‹n› lines per inch" below the field | Scan angle (°) |
| 5 | Both directions — switch | Overscan — switch |
| 6 | Crosshatch — switch | (empty) |

#### Images on this layer (collapsed section)

- A collapsing section under the mode lines, titled "Images on this layer"
  (current heading and label, naming.md), closed by default on vector rows
  and open on image rows (M7). Its header is a disclosure marker plus the
  title in `type-body`, `text`.
- Contents, two pairs per line:

| Line | Left pair | Right pair |
|---|---|---|
| 1 | Method — dropdown: Grayscale, Threshold, Floyd–Steinberg, Jarvis, Stucki, Ordered (Bayer), Halftone | Interval (mm), with the hint "‹n› dpi" below the field |
| 2 | Negative — switch | Min power (%) — enabled only while Method is Grayscale |

- On an image row (M7) the Line or Fill lines are not shown, and the
  section cannot be closed; the shared lines 1–3 still edit the layer's
  speed, power, passes, air and constant power (one layer, two rows; see
  Open questions).
- Min power (%) moves here from the shared lines (today it is always shown,
  `layers_panel.rs:204-214`), because `docs/spec/layers.md` says it only
  matters on GRBL for grayscale images. Its range stays 0 to Max power (%).
- The method dropdown shows readable names, never the stored ids (today it
  shows "floyd_steinberg", `layers_panel.rs:279-290`).

#### Field widgets, ranges and tooltips

Ranges and defaults come from `docs/spec/layers.md`; decimals from
`naming.md` ("Numbers in fields and read-outs"). Number fields are egui
drag values: drag sideways to scrub, click to type. Clicking into a number
field selects its whole value so typing replaces it (Match [R12], own
mechanism).

| Label | Setting | Widget | Range / decimals | Tooltip (first line / second line) | Milestone |
|---|---|---|---|---|---|
| Name | `name` | text field | 1–60 characters | "Name shown in the list and in messages" | Now |
| Mode | `mode` | dropdown | Line, Fill | "Line traces outlines; Fill scans inside closed shapes" / "Open shapes on a fill layer are skipped." | Now (labels) |
| Speed (mm/s) | `speed` | number | above 0 to 10 000, 1 decimal | "Top speed for this layer. Above 0 to 10 000." / "Short moves may not reach it." | Now (unit in label) |
| Max power (%) | `max_power` | number | 0–100 and not below Min power (%), 1 decimal | "Peak power for this layer. 0–100." / "Never above the device's Max power (%)." | Now (unit in label) |
| Passes | `passes` | number, whole | 1–1000 | "How many times the layer runs. 1–1000." | Now |
| Air assist | `air` | switch | — | "Turn air assist on while this layer runs." | Now (switch) |
| Constant power (M3) | `constant_power` | switch | — | "Keep the set power while the head slows or stops (M3)." / "Off uses dynamic power (M4), which turns the laser off when the head stops." | M5 (add) |
| Kerf offset (mm) | `kerf` | number | −5 to 5, 3 decimals | "Shift the cut on closed shapes to allow for the beam width. −5 to 5." / "Positive moves the cut outward from the part." | Now |
| Tabs per shape | `tabs` | number, whole | 0–100 | "Uncut bridges left on each closed outer shape. 0 is off." | Now |
| Tab width (mm) | `tab_width` | number | 0.01–50, 3 decimals | "Length of each bridge. 0.01–50." | Now |
| Interval (mm) | `interval` | number | 0.001–10, 3 decimals; also accepts "‹n›lpi" or "‹n›dpi" typed | "Distance between scan lines. 0.001–10." / "You can also type 254lpi." | Now |
| Scan angle (°) | `angle` | number | −360 to 360, 2 decimals | "Direction of the scan lines; 0 is horizontal." | Now |
| Both directions | `bidirectional` | switch | — | "Burn on the way back too; off returns with the laser off." | Now (switch) |
| Overscan | `overscan` | switch | — | "Run on past the line ends with the laser off." | Now (switch) |
| Crosshatch | `crosshatch` | switch | — | "Add a second pass at 90°." | Now (switch) |
| Method | `image.dither` | dropdown | seven methods | "How the image is turned into dots or shades" | Now (names: M7) |
| Interval (mm) (image) | `image.interval` | number | 0.001–10, 3 decimals; "‹n›dpi" accepted | "Distance between image lines. 0.001–10." | Now |
| Negative | `image.negative` | switch | — | "Burn the light parts instead of the dark ones." | Now (switch) |
| Min power (%) | `min_power` | number | 0 to Max power (%), 1 decimal | "Power for the lightest shade of a grayscale image. 0 to Max power (%)." / disabled: "Used only by the Grayscale method." | Now (moved) |

- Typed text such as "254lpi" is passed to the command unchanged; the
  command converts it (`docs/spec/layers.md`). The UI does not compute
  conversions other than the display hints.
- The hints ("‹n› lines per inch", "‹n› dpi") are display only (25.4 divided
  by the interval, whole number), in `type-small`, `text-muted`.
- Out-of-range input is clamped to the range and the tooltip states the
  range (naming.md). A Max power (%) typed below Min power (%) is sent
  unchanged; the command refuses it (`docs/spec/layers.md`: lowering
  max_power below min_power fails), the field returns to the stored value
  and the Messages field shows the command's error, e.g. "Max power cannot
  go below Min power (‹p›%). Lower Min power first." (own wording). The UI
  never decides this rule itself.
- Power is shown as stored; the device clamp is applied at export and in
  the job preview's warnings (`docs/spec/safety.md` rule 6, naming.md
  "Power clamped" message), never by the editor.

#### Constant power (M3) and its note

- The switch is a normal switch (on = `accent`; decision 4 keeps the switch
  colors the same for every switch).
- While it is on, a note band spans both pairs directly below line 3: a
  `warning-soft` band with a small `warning` mark at its left and the text,
  in `type-small`, `text`: "The laser may stay on while the job is paused.
  Stop turns it off." (own words; the fixed safety phrase "The laser may
  stay on" from naming.md comes first).
- The list row shows the "M3" tag as a `warning-soft` chip with `text`
  (Layer rows table), so a
  constant-power layer is visible without opening the editor.
- The job side of this rule (Pause labelled "Pause (motion only)", Stop
  highlighted while a job uses `M3`) is `machine-panels.md`'s and
  `docs/spec/safety.md` rule 9's; this file only marks the layer.
- The switch exists in the model since M5 but not in the editor
  (current-ui.md 1.6); adding it is part of this restyle.
- Tier: **Echo** (a per-layer constant-power switch), **Own** (label, note,
  tag).

#### Commit rules (binding, from `docs/spec/ui-shell.md`)

- A dragged number is previewed in the form and committed as one
  `layer.set` when the drag ends (one undo step); a typed number commits on
  Enter or when focus leaves the field; Esc while typing restores the stored
  value and commits nothing.
- Dropdowns and switches commit at once, one `layer.set` per change.
- The name commits on Enter or focus loss (Header line).
- While a field is being dragged or typed in, outside changes (undo, the
  command line) do not overwrite it; once the edit ends the form shows the
  project's values again (today's draft rule, `layers_panel.rs:150-156`).
- Switching the edited layer while a typed value is unconfirmed commits it
  first (focus leaves the field).
- Nothing in the editor plans, estimates or clamps against the device; it
  only emits commands (CLAUDE.md, the UI crate has no business logic).

#### Simple layout (Later)

`layout.md` ("Simple layout") lists the Layers panel among the reduced
parts. In Simple layout the layer editor hides Min power (%), Constant power
(M3), Kerf offset (mm), Tabs per shape, Tab width (mm), Scan angle (°), Both
directions, Overscan and Crosshatch; Mode, Speed (mm/s), Max power (%),
Passes, Air assist, Interval (mm) and the image Method, Interval and
Negative stay. Own rule: a hidden field whose value differs from its
default stays visible (so a constant-power or kerf setting can never act
unseen), and turning Simple layout off shows everything again. Unlike the
reference's reduced mode [R16], values are never forced. The Air column and
the header context menu stay.

Tier for the layer editor: **Match** (a quick-edit form of the main
settings under the list, two label/field columns, labels right-aligned),
**Echo** (constant-power switch), **Own** (field order, grouping, look,
hints, note band, Simple layout rule).

### Empty state

- With no layer that has shapes, the panel shows only one line in the list
  area: "Draw a shape to start a layer." (today's text, naming.md, Empty
  states) in `type-body`, `text-muted`, left-aligned with the list's first
  column, with `gap` above it.
- No header row, no side button column and no layer editor are drawn in
  the empty state (today: no editor, `layers_panel.rs:79-85`).
- The palette strip still works: choosing the current color there needs no
  layer row.
- Tier: **Own**.

### Layers panel narrow and short

- The panel's minimum width is what Color, Name (a few characters), Output,
  Show and the side button column need; the dock enforces it (`layout.md`,
  "Minimum sizes").
- Short: the list area keeps at least four rows; the layer editor scrolls
  with the panel body.

---

## Layer settings (window, Later)

### Reference

- The full editor for one layer, opened by double-clicking a layer in the
  list (or an entry of the material library). Speed, power and mode are
  always at the top; the rest is split into tabs; what shows depends on the
  device type and the mode [R4][R12].
- A separate window, taller than wide, with the system title bar [R4].
- A narrow column down its left side lists the project's layers as stacked
  chips in list order; clicking one switches the editor to that layer; the
  current chip is marked [R4][R6][R10].
- A header with a color bar, the name field and a whole-layer output switch;
  then a strip of sub-layer tabs with add, duplicate and delete buttons, a
  sub-layer name and a per-sub-layer output switch [R4][R5][R10].
- Essential settings for GRBL-type devices: speed, maximum power with an air
  switch beside it, a constant-power switch, and the mode [R5].
- For line and fill modes, two tabs (an everyday tab and an advanced tab)
  hold the rest; images use one titled group instead; groups are separated
  by thin lines and inactive features show a small grey status note after
  the field [R4][R6][R7][R8][R9].
- Fill and image pages show a small diagram of the scan pattern [R7][R9];
  the concentric variant shows a spiral and greys the fill-only rows [R8].
- A bottom row with three default buttons on the left and OK and Cancel on
  the right (in the platform's order) [R4][R11].
- Up to eleven sub-layers per layer, run left to right; in the list such a
  layer shows a combined mode [R10][R1].

### LaserCraft

Name: Layer settings (naming.md). A floating, non-modal egui window; it
never covers or blocks the job bar (`dialogs.md`; `docs/spec/safety.md`
rule 7). Opened by double-clicking a layer row, double-clicking a palette
swatch whose layer has shapes, or Layer settings… in the Swatch menu.
Title: "Layer settings — ‹nn› ‹name›" (own design; the title updates when
the column switches layers). Resizable, taller than wide on first open;
size and place are remembered for the session (persistence across runs is
`layout.md`'s later decision). Tier: **Echo** (separate full editor reached
by double-click), **Own** (everything below unless marked).

Anatomy:

```
+----------------------------------------------------------+
| Layer settings — 03 Engrave                          [x] |
+------+---------------------------------------------------+
| [00] |  [03]  Name [ Engrave              ]  Output (o)  |  header
| [03] |  [Fill 1][Line 2] (+)(dup)(-)      (owner decision)|  operations
| [07] |  Speed (mm/s) [ 200 ]      Max power (%) [ 35 ]   |  essentials
|      |  Air assist (o)     Constant power (M3) (o)       |
|      |  [warning band when M3 is on]                     |
|      |  Mode [Fill v]                                    |
|      |  [ Basics ][ Advanced ]                           |  tabs
|      |  +---------------------------------------------+  |
|      |  | mode-specific groups                        |  |
|      |  +---------------------------------------------+  |
+------+---------------------------------------------------+
| [Restore default][Save as default][Save as default for…] |
|                                           [OK] [Cancel]  |
+----------------------------------------------------------+
```

### Swatch column

- Place: the full height of the window's left side, one chip wide plus
  `gap-tight` on each side, on `field`, with a thin `border` line on its
  right; it scrolls when the project has more layers than fit.
- Contents: the project's used layers as chips (palette swatch look), in
  output order, top to bottom; guide layers are not listed.
- The layer shown in the window has the current-color style mark (thick
  `text` outline with a `field` ring) — the same mark as the palette's
  current color, because it means "this one".
- A chip whose layer has unsaved changes in this window shows a small dot in
  `text` at its top right corner (own design).
- Click: switch the window to that layer; the edits made so far stay as
  pending edits for their layer. Tooltip: "‹nn›: ‹name›".
- Tier: **Match** (column of chips on the left that switches layers),
  **Own** (look, pending-edits dot).

### Header and essentials

| Line | Contents |
|---|---|
| 1 | chip; Name (text field); at the right end Output — switch with its label on the left |
| 2 | Operations strip (owner decision; below) |
| 3 | Speed (mm/s) — Max power (%) |
| 4 | Air assist — Constant power (M3); line 4a: the constant-power note band while it is on (same as in the layer editor) |
| 5 | Mode — dropdown (plain text "Image" for an image entry) |

Same widgets, ranges, tooltips and tokens as the layer editor. Tier:
**Match** (essentials always on top), **Own** (order and look).

### Operations (Later, owner decision)

- Only if the owner approves several settings sets per layer (proposed
  change to `docs/spec/layers.md`). Then line 2 holds the Operations tab
  strip: one tab per operation, left to right in run order, each showing the
  operation name (default "‹mode› ‹n›", e.g. "Fill 1", "Line 2") with a
  small mode pictogram before it (`icons.md`: `mode-line`, `mode-fill`,
  `mode-image`). The open tab is `field` and joins the content below; other
  tabs are `panel` with `text-muted` text; a thin `accent` marker on the
  open tab (README.md, accent uses).
- At the right end: Add operation, Duplicate operation, Delete operation,
  icon-only (`icons.md`: `operation-add`, `operation-duplicate`,
  `operation-delete`). Delete operation is disabled while only one
  operation exists. Tooltips say what each does in our words.
- Below the strip: Operation name (text field) and Output this operation
  (switch, disabled while only one operation exists).
- Tabs reorder by dragging. A layer with more than one operation shows
  "Mixed" in the list's Mode cell (plain text) and its Mode is changed only
  here.
- Tier: **Echo** (several settings sets as tabs on one layer), **Own**
  (name, look, labels).

### Basics and Advanced tabs

- A tab pair labelled "Basics" and "Advanced" under the essentials, for
  every mode (own design; the reference uses one group for images). The
  Advanced tab is shown only when the mode has at least one advanced field
  that exists in LaserCraft at that time.
- Tab look as for the Operations strip; the content area is `panel` with a
  thin `border`, groups inside separated by thin `border` lines.
- Inside a group: label/field pairs, two per line, labels right-aligned, as
  in the layer editor. A switch that turns a feature on sits first in its
  group; the group's other fields are disabled (`text-disabled`) while it is
  off. No grey status notes after fields: the disabled look and the
  tooltip carry that meaning (own design, replacing the reference's notes).
- Field groups per mode (labels from naming.md; every field after "Now" is
  a proposed change to `docs/spec/layers.md` before it is built):

| Mode | Tab | Group | Fields in order | Milestone |
|---|---|---|---|---|
| Line | Basics | Passes and kerf | Passes; Kerf offset (mm) | Now (fields exist in the panel) |
| Line | Basics | Tabs | Tabs (switch); Tab placement (Automatic, By hand); Tab spacing (mm) or Tabs per shape; Tab width (mm); Max tabs per shape; Power across tabs (%); Clear hand-placed tabs (button); No tabs on inner shapes (switch) | Now for Tabs per shape and Tab width (mm); the rest Later |
| Line | Basics | Height | Z offset (mm); Lower Z each pass (mm) — shown only for a device profile with a motorised Z axis | Later |
| Line | Basics | Dashed cut | Dashed cut (switch); Dash on (mm); Dash off (mm) | Later |
| Line | Advanced | Dwell | Dwell at start (ms); Dwell at end (ms); Laser on during dwell (switch); Dwell power (%) | Later |
| Line | Advanced | Overcut | Overcut (mm) | Later |
| Line | Advanced | Lead-in and lead-out | Lead-in length (mm); Lead-out length (mm); Lead angle (°); Lead shape (Straight, Arc) | Later |
| Fill | Basics | Scanning | Interval (mm) with the lpi hint; Scan angle (°); Turn angle each pass (°); Both directions; Crosshatch; Overscan | Now (fields exist in the panel); Turn angle each pass (°) Later |
| Fill | Basics | Passes and height | Passes; Z offset (mm); Lower Z each pass (mm) (Z only with a motorised Z axis) | Now for Passes; Z later |
| Fill | Basics | Fill order | Fill order: All at once, By group, One shape at a time (radio buttons) | Later |
| Fill | Advanced | Edges and gaps | Ramp length (mm); Ramp outer edges too (switch, disabled while Ramp length (mm) is 0); Skip blank areas | Later |
| Concentric fill | Basics, Advanced | as Fill | as Fill, but Both directions, Crosshatch, Overscan, Scan angle (°) and Turn angle each pass (°) are disabled with the tooltip "Not used by Concentric fill."; Ramp length (mm), Ramp outer edges too and Skip blank areas likewise | Later (mode itself later) |
| Image | Basics | Image | Method; Interval (mm) with the dpi hint; Negative; Min power (%) (Grayscale only); Passes | Now for existing fields (M7 for images) |
| Image | Basics | Scanning | Both directions; Overscan; Scan angle (°); Turn angle each pass (°) | M7 for Both directions, Overscan and Scan angle (°) (the layer's existing `bidirectional`, `overscan` and `angle`, applied to images: proposed change to layers.md); Turn angle each pass (°) Later |
| Image | Advanced | Fine tuning | Use image as is; Beam compensation (mm); Halftone frequency (lpi); Halftone angle (°) (the halftone fields enabled only for Halftone) | M7 or later |
| Image | Advanced | Height and edges | Z offset (mm) (shown only for a device profile with a motorised Z axis); Ramp length (mm) (enabled only for the Threshold method; tooltip when disabled: "Used only by the Threshold method.") | Later |

- Reference basis for the rows added above: the reference's fill and image
  settings both offer a scan direction, sweeping both ways and a beam-off
  run past the line ends [R7][R9]; image settings also carry a height
  offset and an edge power ramp that works only with the threshold method
  [R9]; fill ramping has a second switch that also slopes the outer edges,
  greyed while the ramp length is zero [R7]; and a per-pass step that
  rotates the scan direction after each pass, zero meaning every pass runs
  the same way, is described for image layers and seen in a galvo fill
  capture [R9][R10]. Where the reference places the per-pass step is
  unknown, so its spot right after Scan angle (°) is own design.
- Field tooltips for the added rows (own words): Turn angle each pass (°)
  "Rotate the scan direction by this much after every pass; 0 keeps every
  pass the same."; Ramp outer edges too "Slope the outside edges as well,
  not only the inner walls."; Ramp length (mm) "Fade the power over this
  distance at the edges of the engraving; 0 is off."; Z offset (mm) "Raise
  or lower the head for this layer." Scan angle (°), Both directions and
  Overscan reuse the tooltips of the layer editor.
- Every power field (Max power (%), Min power (%), Dwell power (%), Power
  across tabs (%)) is clamped like every power value (`docs/spec/safety.md`
  rule 6). Laser on during dwell is part of the job (rule 1).
- A scan-pattern pictogram beside the Scanning group is optional own design
  (Open questions); if built, it is our own small drawing that follows Both
  directions and Scan angle (°), drawn with `text` strokes on `field`.

### Buttons

- One full-width row at the bottom, below the swatch column too.
- Left group (M8, per-device layer defaults): Restore default, Save as
  default, Save as default for all layers. Tooltips: "Load this color's
  saved settings for the current device", "Store these settings as this
  color's default for the current device", "Store these settings as the
  default for every color on the current device". Restore default changes
  only the pending edits (it is undone by Cancel). Shift+click on Restore
  default loads LaserCraft's built-in layer defaults instead (the Default
  column of `docs/spec/layers.md`, Settings: Line mode, speed 10 mm/s, Max
  power 20%, Min power 0%, one pass, interval 0.1 mm, and every optional
  feature off), again only into the drafts; the tooltip's second line
  says "Shift+click: load LaserCraft's built-in defaults instead.", as in
  the Cut order dialog (`dialogs.md`). The reference offers the same split
  between the color's saved default and a built-in one on Shift+click
  [R11]; the built-in values are ours. The two Save buttons
  write the device profile at once and say so in the Messages field (their
  effect is not part of the project's undo). Until M8 the group is absent.
- Right group: OK (primary: `accent` fill) and Cancel; order and keys
  (Enter, Esc) are `dialogs.md`'s, the same on every platform (README.md,
  platform decision).
- Commit model (from `docs/spec/ui-shell.md`, M3 additions: a window builds
  one command when OK is pressed): the window edits drafts. OK runs one
  `edit.batch` with one `layer.set_all ‹c› ‹json›` per layer changed in this
  window (one undo step); Cancel or closing the window runs nothing. While
  the window is open, the Layers panel and the canvas show the stored
  settings, not the drafts.
- If a layer changes underneath an open window (undo, the command line),
  that layer's drafts are discarded and its fields reload; a line in
  `info-soft` at the top of the window says "Layer ‹nn› changed elsewhere;
  its fields were reloaded." (own design).
- Tier: **Match** (default buttons left, confirm and cancel right),
  **Own** (labels, commit model).

### Deliberately omitted (decision 1)

DSP-only rows (PWM frequency override, dot mode, the U axis offset, the
speed and power "default" switches and the DSP minimum power layout), galvo
settings (frequency, pulse width, galvo-only fill rows) and any
vendor-specific defaults.

---

## Layer colors elsewhere

How the layer color shows outside this file's regions (each owned by the
file named; repeated here so the meaning stays one):

| Where | How the layer color appears | Owner |
|---|---|---|
| Canvas shapes | strokes in the layer color; heavier line when selected (not recolored); output-off layers faded; hidden layers not drawn and not clickable (`docs/spec/canvas.md`, `docs/spec/layers.md`) | `canvas.md` |
| Canvas, Show fills on (Later) | fill layers drawn filled with a lighter strength of the layer color | `canvas.md` |
| Canvas, guide layers (Later) | strokes in the guide color, dashed, never faded by Output | `canvas.md` |
| Canvas, Highlight shapes (Later) | the layer's shapes briefly marked; look decided in canvas.md, never by changing the layer color | `canvas.md`, `menus.md` |
| Job preview | `move-burn` = the layer color; unlit and travel moves use `move-unlit` and `move-travel`, never a layer color | `job-preview.md` |
| Palette strip, layer rows, layer editor header, Layer settings column | the swatch chip: fill, two-digit number with the contrast rule, thin `border` outline | this file |
| Materials panel (M8) | the target layer's chip beside Apply to layer | `later-windows.md` |
| Messages | a layer is named as layer "‹name›" in text, never by color alone (naming.md) | `naming.md` |

- Chrome never uses a layer color for meaning (danger, warning, selection):
  layer colors only tell layers apart (`docs/spec/layers.md`); semantic
  colors are the tokens of `visual-language.md`.
- Tier: **Match** (shapes and burn moves in their layer color), **Own**
  (everything else).

---

## egui implementation notes

- No new dependency: the list is an `egui::Grid` (or rows of
  `ui.horizontal` with fixed column widths) inside an `egui::ScrollArea`;
  the sticky header is drawn above the scroll area with the same column
  widths. `egui_extras::TableBuilder` would need a new crate and an
  `ARCHITECTURE.md` line; it is not needed.
- Swatches and chips are painted (`Painter::rect_filled`,
  `rect_stroke`, centered text with the chosen black or white); one helper
  draws them for the palette, the list, the editor header and Layer
  settings (today `swatch()`, `layers_panel.rs:31-42`).
- The switch is one painted widget shared with every panel
  (`visual-language.md`); it replaces `checkbox` calls
  (`layers_panel.rs:135-146`, `390-402`).
- Whole-row click: allocate the row rect with `Sense::click()` behind the
  cell widgets so dropdowns and switches keep their own clicks.
- Drag to reorder: egui's built-in drag-and-drop payloads
  (`dnd_drag_source`, `dnd_drop_zone`) are enough; the drop emits one
  `layer.set_order`.
- The More popup of the palette strip uses the toolbar model's popup
  (`layout.md`).
- Layer settings is an `egui::Window` (not `egui::Modal`, which blocks all
  input; README.md, safety decision).
- Contrast choice: a small pure function in `colors.rs` (WCAG relative
  luminance of the sRGB value), unit-tested over all 30 colors.
- Labels a test looks for: none in this file today
  (`crates/lasercraft-ui/tests/shell.rs` looks for a layer name, which stays
  drawn in the Name column).

---

## Current state → target

From current-ui.md 1.6, 1.7 and the code at commit 151eb26.

Palette strip (`layers_panel.rs:31-72`, `shell.rs:251-257`):

1. Swatches are unnumbered painted squares (`layers_panel.rs:31-42`) →
   two-digit numbers in `type-small`, black or white by the contrast rule.
2. `horizontal_wrapped` lets the strip wrap to more lines
   (`layers_panel.rs:53`) → one line with a More button.
3. Size and spacing are fixed values in code (`layers_panel.rs:54`, `61`) →
   derived from `row` and `gap-tight`.
4. Current swatch outlined in the theme's strong text color, others in weak
   text color (`layers_panel.rs:34-40`) → current: thick `text` outline with
   a `field` ring; others thin `border`; hover darkens the outline.
5. Tooltip "{n}: {name}" with an unpadded number (`layers_panel.rs:61`) →
   "‹nn›: ‹name›" plus the second line saying what a click will do.
6. Click rules unchanged (`layers_panel.rs:62-67`); Shift+click,
   double-click and the Swatch menu are new (Later).
7. No guide swatches → two guide swatches (Later, owner decision).

Layers panel (`layers_panel.rs:76-316`, `shell.rs:262-270`):

8. `ui.heading("Layers")` (`layers_panel.rs:78`) → the panel title strip
   (`layout.md`).
9. Striped five-column grid without headers (`layers_panel.rs:86-95`) →
   no stripes; header row Color, Name, Mode, Speed, power, Output, Show,
   Air (Later); sticky header.
10. Unnumbered small swatch in rows (`layers_panel.rs:120`) → numbered chip;
    hollow when Show is off; faded when Output is off.
11. Only the name is clickable, with hover text "Click to edit; Shift+click
    selects its shapes" (`layers_panel.rs:121-130`) → the whole row;
    tooltip as in the Layer rows table; selected row `accent-soft`.
12. Summary "line 10 mm/s 20%" with the raw lowercase mode
    (`layers_panel.rs:131-134`) → a Mode dropdown cell ("Line", "Fill") and
    a "10 / 20%" Speed, power cell, with the "M3" tag when constant power
    is on.
13. Unlabeled checkboxes with hover "Output" and "Show"
    (`layers_panel.rs:135-146`) → switches under headers; Air column added
    (Later).
14. Text buttons "Move up", "Move down", "Delete shapes" in a row under the
    list (`layers_panel.rs:97-109`) → icon column right of the list with
    tooltips; Copy settings and Paste settings added (Later).
15. The edited layer does not follow the canvas selection → it does when all
    selected shapes share a layer.

Layer editor (`layers_panel.rs:150-402`):

16. One label/field pair per line (`layers_panel.rs:162-164`) → two pairs
    per line, labels right-aligned, one pair per line when narrow.
17. Units as field suffixes (" mm/s", " %", " mm", "°";
    `layers_panel.rs:191`, `200`, `211`, `239`, `251`, `329`, `349`) → units in
    the labels ("Speed (mm/s)", "Max power (%)", "Kerf offset (mm)",
    "Interval (mm)", "Scan angle (°)", "Tab width (mm)").
18. Mode combo shows "line"/"fill" (`layers_panel.rs:175-178`) → "Line",
    "Fill".
19. Min power always shown (`layers_panel.rs:204-214`) → in the image
    section, enabled for Grayscale only.
20. Booleans are checkboxes with the label in the left column
    (`layers_panel.rs:390-402`) → switches, label right-aligned on the left.
21. No Constant power (M3) field although the model has it
    (current-ui.md 1.6) → switch, note band and list tag.
22. Image method shows raw ids (`layers_panel.rs:279-290`) → readable
    names; the section opens by itself on image rows (M7).
23. The "lines per inch" hint (`layers_panel.rs:243-245`) stays, in
    `type-small`, `text-muted`, below the field; the image interval gains a
    "dpi" hint.
24. Nothing styles the panel today (no `Visuals` set; current-ui.md 2) →
    all looks from `visual-language.md` tokens.
25. No Layer settings window → Layer settings (Later).

Unchanged: the empty-state text and its weak style
(`layers_panel.rs:80-82`), the draft and commit logic
(`layers_panel.rs:150-156`, `356-388`), default names "Layer NN", the canvas
fade of output-off layers (`canvas.rs:254-255`) and burn moves in the layer
color (`ui/preview.rs:207-209`).

---

## Proposed changes to other specs

Each is labelled per decision 15; the named spec is updated first, in the
session that builds the change.

- **Proposed change to ui-shell.md** (Layout, "Right" paragraph): the
  layers panel gets a header row (Color, Name, Mode, Speed, power, Output,
  Show, Air), whole-row selection, an icon button column (Move up, Move
  down, Delete shapes, Copy settings, Paste settings), a two-pair layer
  editor with units in the labels and a Constant power (M3) switch, and
  double-click opening Layer settings (later); "clicking a name opens it in
  the editor" becomes "clicking a row".
- **Proposed change to ui-shell.md** (Layout, "Bottom" paragraph): the
  palette strip shows two-digit numbers, stays one line with a More button,
  and its tooltip names what a click does.
- **Proposed change to layers.md** (Colors are layers): Shift+click,
  double-click and the right-click Swatch menu on palette swatches; the
  edited layer following the canvas selection is UI behavior and needs no
  spec change.
- **Proposed change to layers.md** (Settings): the editor shows
  `min_power` only for the Grayscale image method; image rows edit the same
  speed, power, passes, air and constant-power settings as the vector row of
  that color (or, if the owner prefers, images get their own; see Open
  questions).
- **Proposed change to layers.md**: guide layers (two non-output layers
  after the 30; owner decision), Concentric fill mode, Operations and the
  Mixed mode (owner decision), the new tab, Z, dashed-cut, dwell, overcut,
  lead-in/out, fill order, ramp and image fields listed under Layer
  settings (each a later change, as naming.md already notes).
- **Proposed change to layers.md** (Commands): a command behind Put line
  layers last (already proposed in `menus.md`).
- **Proposed change to naming.md** (layer setting labels): add "Turn angle
  each pass (°)" (fill and image scanning; rotates the scan direction by
  this many degrees on every pass, 0 keeps it fixed; Echo, Later) and "Ramp
  outer edges too" (switch after Ramp length (mm), disabled while the ramp
  is 0; Own wording, Later), each with its doc term in the mapping column.
- No change to keymap.md: this file binds no new chords and shows none
  that `crates/lasercraft-ui/keymap/default.keymap` does not bind.

---

## Open questions

1. **Palette: 01 and 29 look alike.** Both are bright reds because the hue
   wheel ends just short of where it starts (current-ui.md 2). Change the
   palette data (owner decision; decision 5 says this guide does not)?
2. **Palette: bright yellows on the bed.** The bright yellows (around 05
   and 07) have low contrast on the near-white `bed`. Options: change the
   data, or draw canvas strokes of low-contrast colors with a thin darker
   edge (canvas.md would own that). Owner decision.
3. **Import compatibility.** Should the palette match the reference's
   published color table so imported files land on the "same" layers? That
   would copy palette values the clean-room rules treat as Own (decision 5)
   and needs the owner (layers notes, Unknowns).
4. **Guide layers.** Approve the two guide layers, their colors (must differ
   from every palette color and the canvas tokens) and the guide pictogram?
5. **Operations and Mixed.** Approve several settings sets per layer, with
   the run-order and output rules described above?
6. **Image rows' settings.** Should an image row share speed, power and
   passes with the vector row of its color (one layer, as the model is
   today) or get its own? Affects `docs/spec/layers.md` and M7.
7. **New labels for naming.md.** "Use for new shapes" and "Layer
   settings…" (Swatch menu), the Layer settings title form "Layer settings —
   ‹nn› ‹name›", the "M3" tag, the header tooltips and the empty-area hint
   "Double-click a layer to open its settings.", the region name "Swatch
   menu" (no visible label), the two-digit swatch tooltip "‹nn›: ‹layer
   name›" (replacing the glossary's current "‹n›: ‹layer name›") and the
   swatch tooltip second lines ("Click to put the selected shapes on this
   layer." / "Click to draw new shapes on this layer.") are own wording to
   be added to `naming.md`.
8. **Keyboard in the list.** egui moves focus with Tab, but Tab is bound to
   select the next object; arrows nudge and Del deletes the selection. Do we
   add list keys (Up/Down to change the edited layer, F2 to rename) while
   the list has focus, and how is focus shown and left? Accessibility needs
   an answer before the header row is built.
9. **Delete shapes without confirmation.** This follows naming.md
   (undoable actions never ask). The reference asks by default; does the
   owner want an optional confirmation preference?
10. **Paste settings timing.** Our Paste uses the source layer's settings as
    they are when Paste is clicked (what `layer.copy_settings` does), not as
    they were when Copy was clicked. Acceptable, or should Copy capture a
    snapshot (a new command taking JSON, e.g. reusing `layer.set_all`)?
11. **Edited layer follows the canvas.** Own design; some users may find the
    editor switching layers surprising. Keep, or make it a preference?
12. **Unfocused selection look.** We keep `accent-soft` for the edited row
    even when the list is not focused (the reference greys it). Confirm.
13. **Scan-pattern pictogram.** Build an own small live drawing of the scan
    direction in Layer settings, or leave it out?
14. **Used-layer marker on swatches.** Should palette swatches whose layer
    has shapes carry a small marker so the strip shows which layers are in
    use? Not in the reference notes; own idea.
15. **Air column for machines without software air.** Hide the column when
    the device profile has no air-assist code (M8), or always show it?
16. **Layer settings OK/Cancel.** The draft-and-OK model follows
    ui-shell.md's window rule; the reference's behavior (live or on OK) is
    unknown from the docs. Confirm, including that the default Save buttons
    write the device profile immediately.
17. **Swatch shape.** Decision 5 says square; the reference's chips are
    slightly wider than tall. Square is kept; confirm.
18. **Column order.** Color before Name (glossary order) differs from the
    reference's name-first order. Confirm, since switchers may look for the
    name first.

---

## Sources

All LightBurn pages read 2026-10-21, through the research notes for this
file (palette, layer list and layer editor notes; main window layout notes;
the current UI audit).

1. [R1] https://docs.lightburnsoftware.com/2.1/Reference/CutsLayersWindow/
   — layer list place, columns and their meaning, row look and states,
   side buttons and their actions, quick-edit fields, header and row
   context menus, Shift gestures, drag reorder, two rows for image and
   vector, output fading, delete confirmation.
2. [R2] https://docs.lightburnsoftware.com/2.1/Reference/UI/ColorPalette/ —
   palette strip place, docking variants, numbering, contrast numbers,
   current-chip frame, click rules, special non-output chips, import color
   matching, the red/black habit.
3. [R3] https://docs.lightburnsoftware.com/2.1/GetStarted/ColorsAndLayers/
   — colors as layers, click rules, clipped strip end, hovered row look,
   current chip frame on Windows, run order.
4. [R4] https://docs.lightburnsoftware.com/2.1/Reference/CutSettingsEditor/
   — full editor purpose, window shape, left layer column, tab split, bottom
   buttons.
5. [R5] https://docs.lightburnsoftware.com/2.1/Reference/CutSettingsEditor/SharedSettings/
   — header (name, output), essential settings for GRBL-type devices,
   constant-power switch.
6. [R6] https://docs.lightburnsoftware.com/2.1/Reference/CutSettingsEditor/LineMode/
   — line-mode tabs and groups, inactive-feature notes, layer column in a
   second capture.
7. [R7] https://docs.lightburnsoftware.com/2.1/Reference/CutSettingsEditor/FillMode/
   — fill-mode tabs and rows, scan diagram, linked interval and lines per
   inch.
8. [R8] https://docs.lightburnsoftware.com/2.1/Reference/CutSettingsEditor/OffsetFillMode/
   — concentric variant greying fill-only rows, spiral diagram.
9. [R9] https://docs.lightburnsoftware.com/2.1/Reference/CutSettingsEditor/ImageMode/
   — image settings as one group, scan diagram, methods, interval and dpi.
10. [R10] https://docs.lightburnsoftware.com/2.1/Reference/CutSettingsEditor/SubLayers/
    — sub-layer tabs, their buttons, name and output switch, run order,
    count limit, combined mode word in the list.
11. [R11] https://docs.lightburnsoftware.com/2.1/Reference/CutSettingsEditor/WorkingWithDefaults/
    — default buttons and their meaning, bottom button row.
12. [R12] https://docs.lightburnsoftware.com/2.1/GetStarted/CutSettingsBeginner/
    — double-click opens the editor, hover hint, speed/power summary,
    double-click selects a field's value, interval greyed for line layers,
    hidden layers still run.
13. [R13] https://docs.lightburnsoftware.com/2.1/Reference/UI/ — default
    window map: layer list in the upper right group, palette row at the
    bottom, quick-edit fields.
14. [R14] https://docs.lightburnsoftware.com/2.1/GetStarted/UITour/ —
    layer list contents and tabs, palette row look and count, reduced-mode
    quick fields.
15. [R15] https://docs.lightburnsoftware.com/2.1/NewFeatures/NewIn2.1/ —
    more compact layer window, kept scroll position and visible selection,
    one-decimal power in the summary, chip size fix, chips redrawn on font
    scale change.
16. [R16] https://docs.lightburnsoftware.com/2.1/Reference/BeginnerMode/ —
    what the reduced mode hides or forces in layer settings.
17. [R17] https://docs.lightburnsoftware.com/2.1/Reference/UI/StatusBar/ —
    overflow chevron at the palette row's end.
18. [R18] https://docs.lightburnsoftware.com/2.1/Reference/UI/WindowMenu/ —
    palette and layer list entries in the window menu, layer count wording.
19. [R19] https://docs.lightburnsoftware.com/2.1/Guides/CustomizingTheLightBurnWindow/
    — older frames with extra arrow buttons in the layer list's button
    column.

Existing specs and documents used: `docs/spec/layers.md` (colors as layers,
palette clicks, used layers only, output order, settings, ranges, defaults,
commands, `min_power` only for grayscale on GRBL), `docs/spec/ui-shell.md`
(layers panel, palette, commit rules, window commit rule),
`docs/spec/canvas.md` (faded and hidden layers, light bed),
`docs/spec/preview.md` (burn moves in layer color), `docs/spec/safety.md`
(rules 1, 6, 7, 9), `docs/spec/commands.md` (`edit.batch`),
`docs/spec/planning.md` (kerf and tabs), `docs/spec/import.md` (nearest
color), `docs/spec/keymap.md` and `crates/lasercraft-ui/keymap/default.keymap`
(no layer chords; global chords unchanged), `docs/clean-room.md`, and the
style files `layout.md`, `menus.md`, `naming.md`, `toolbars.md`. Open
standards: WCAG 2.x (relative luminance, contrast ratio, the 4.5:1 minimum
for text); GRBL v1.1 laser mode (`M3` constant power, `M4` dynamic power)
via `docs/spec/safety.md`.

Own design: LaserCraft's palette values and swatch look (square, ring
mark, contrast rule, hover and pressed looks), fixed color-number order of
the strip, the More popup's current mark, the swatch tooltip's second line,
Shift+click, double-click and the Swatch menu on swatches, guide swatch
pictogram and labels; the column order, header labels and tooltips, narrow
column rules, row look and states (no stripes, `accent-soft` selection,
untinted chips, hollow chip for hidden layers, faded chip for output off,
"M3" tag), whole-row click, the edited layer following the canvas, list
height rule, icon concepts and tooltips of the button column, no delete
confirmation, Paste settings naming its source; the layer editor's field
order, two-pair layout with switches labelled on the left, hint lines, the
image section holding Min power (%), the constant-power note band, the
Simple layout rule that keeps non-default fields visible; the Layer
settings window's title, pending-edits dot, Basics/Advanced for every mode,
no inactive-feature notes, field groups, commit model and reload notice;
the egui notes.
