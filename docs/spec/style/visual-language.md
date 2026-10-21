# Spec: style — visual language

Milestones: M1–M5 (restyle of what exists today: theme tokens, canvas and
preview colors, switches instead of check boxes, numbered swatches, fields,
menus, tooltips), M6 (job bar, machine state dots, progress bar, alarm and
job-refused banners, big job buttons), M7–M9 (image, materials, camera and
setup windows reuse this catalogue), M10 (accessibility pass, Display
preferences, packaging checks on every platform); the dark theme, the
Theme preference and the Display size preferences are later.

This file is the master for LaserCraft's look: it defines every color,
type and size token that the other style files use, the rules for what
each hue may mean, and one shared catalogue of widgets with all their
states. The other files in `docs/spec/style/` say *where* things go and
*what* they contain; this one says how they look. `README.md` (still to
be written) will explain the tiers (Match, Echo, Own) and the binding
decisions; `naming.md` holds every
label and tooltip; `layout.md` places the regions; `menus.md`,
`toolbars.md`, `property-bar.md`, `layers-panel.md`, `machine-panels.md`,
`canvas.md`, `dialogs.md`, `job-preview.md` and `later-windows.md` describe
their parts using the tokens and widgets below; `icons.md` draws the
pictograms. When another style file and this one disagree about a color, a
type size or a widget state, this file wins and the other file gets a
"proposed change" entry below.

## How to read this file

- **Tokens** are written in backticks (`accent`, `type-small`, `row`). No
  file in the guide writes a color value, a font size or a length. The
  values live only in code (one theme module, see "egui mapping"), where
  tests check the contrast rules of this file.
- Colors are described in plain words for the light theme and the dark
  theme. Sizes are relative ("a step smaller", "about twice `row`").
- Each widget section has a **Reference** part (what the public
  documentation shows, in general words, tagged [Rn]) and a **LaserCraft**
  part (the specification). The tier line says how close we stay: **Match**
  (same arrangement or behaviour), **Echo** (same idea, our own look),
  **Own** (ours alone).
- Statements about the reference come only from the research notes made
  from the public 2.1 documentation pages listed under Sources. Where the
  pages show nothing, this file says "unknown" and designs the thing itself
  ("own design").
- Line weights are named in italics, as in `canvas.md`: *hairline* (the
  thinnest line the display draws crisply), *thin* (borders, separators),
  *normal* (focus rings, outlines that carry meaning), *heavy* (the
  current-color mark, the Stop emphasis ring). They are relative to the UI
  scale like everything else.
- Corner words: *square* (no rounding), *slightly rounded* (one small
  radius shared by buttons, fields, popups, windows and swatches), *pill*
  (fully rounded ends: switches, state dots, the progress bar).

## Principles

### Familiar arrangement, own expression

A user coming from the reference should find every region, every control
group and every state where they expect it (Match), but the surface they
look at is LaserCraft's: our flat title strips, our blue accent, our switch
colors, our pictograms, our words. Where the reference has a signature
look (its colored panel title strips, its green and red switches, its heavy
black glyphs), we keep the *idea* only when it helps users and draw it our
own way (Echo), or drop it (Own). `docs/clean-room.md`: "when in doubt,
differentiate".

### Light, neutral, dense

- The default theme is light: light neutral grey chrome, white fields and
  lists, near-black text, a near-white bed. This is the one place where the
  whole impression should feel familiar (decision 13, Match [R1][R2]).
- Chrome is neutral grey with no hue of its own. Color on screen belongs
  first to the user's layers (the palette colors on the canvas and in the
  swatches) and second to state (selection, focus, danger, warning,
  success). Chrome that shouts would compete with both.
- Dense: many controls, small gaps, one control per row in forms, two-row
  toolbar band. Density never goes below the target-size and contrast rules
  under "Accessibility".

### One meaning per hue

Every hue used in chrome and overlays means exactly one thing, everywhere,
in every theme (decision 6). A user who learns "red means danger" on the
Stop button must never meet red meaning "selected", "laser off" or
"origin". The full table is under "Semantic color rules".

### Flat and calm

- No gradients, bevels, embossing, glossy highlights or textures anywhere:
  not on title strips, buttons, bars, swatches, the canvas or the progress
  bar.
- Shadows only under things that float above the window: menus, popups,
  tooltips, dialogs and (later) floating panels (`shadow`).
- Nothing blinks or pulses. Motion is limited to what carries information
  (see "Motion").

### Safety stays visible

Stop is always the strongest control on screen while connected (decision 7,
safety rule 7). Colors that warn about the laser (`danger`, `warning`)
never appear as decoration, so they keep their force.

### One look on every platform

egui draws every widget, so LaserCraft looks the same on Linux (X11 and
Wayland), Windows and macOS (decision 12). It does not imitate any
operating system's native widgets, title bars, button order or menu style.

## Reference: visual traits in general words

### What the public documentation shows

- **Overall**: a light theme in every capture; light grey chrome and
  toolbars, a near-white drawing area, white fields, lists and tables, dark
  regular-weight text [R1][R2][R6].
- **Panel title strips**: every docked panel carries a strip drawn as a strongly
  colored dark-to-warm gradient, with small light title text and two pale buttons (float, close); it is the only strongly colored
  chrome element [R2][R3][R19][R22].
- **Icons**: bold, heavy, solid black pictograms of one size per toolbar,
  without captions; a few carry one small colored accent (a teal corner
  marker on buttons that hold several tools, a red centre on a
  head-position crosshair); disabled icons turn pale grey [R2][R12][R16].
  The node-editing tool's icon turns red while active [R17].
- **Active tool**: a soft rounded background behind the icon, pale blue in
  the Windows captures and slightly darker grey in the macOS ones [R2][R16].
- **Switches**: tiny pill switches replace check boxes almost everywhere;
  bright green with the knob at one end when on, dark red with the knob at
  the other end when off, dark grey when disabled [R2][R6][R9][R29][R33].
- **Fields**: number fields are white boxes with tiny stacked up/down
  arrows at the right; the unit is a separate word after the box or in
  parentheses inside the label; a focused field gets a blue border
  [R2][R6][R21][R29]. Dropdowns are white boxes with a small chevron
  [R2][R6]. Sliders have a thin groove, a small thumb and the value as text
  at the right [R6][R30].
- **Labels** sit right-aligned directly left of their field, often in two
  columns of label and field pairs [R2][R22][R29].
- **Buttons**: light rectangles with a thin border and slight rounding,
  regular-weight dark text, an optional small icon left of the text; the
  job-control buttons are the same style, about twice as tall; jog buttons
  are small icon-only squares; the default dialog button has a blue outline
  on Windows and a blue fill on macOS [R2][R6][R22][R23][R29].
- **Radio buttons** are plain circles with a blue or dark centre when
  chosen; a 3×3 grid of them, inside a slightly recessed box, picks a point
  of a bounding box and greys out when it does not apply [R2][R22][R26].
- **Tabs**: dock groups have text-only tabs along their bottom edge, the
  chosen one lighter and joined to the panel [R2][R3][R19]; settings tabs
  sit on top and carry large black pictograms [R6]; other dialog tabs are
  text only [R10][R30].
- **Grips and separators**: dotted grip lines at the start of toolbars and
  on the splitters between docked groups; thin, sometimes dotted lines
  between button groups [R3][R12][R16]. A toolbar torn off the band floats
  as a strip with rounded ends and a soft shadow [R15].
- **Menus**: white panels with a thin grey border, a narrow icon column, a
  light-blue band on the row under the pointer, grey disabled rows,
  right-aligned shortcuts, small triangles for submenus, check marks for
  toggles [R8][R13][R14]. Release 2.1 added blank icons so labels line up
  [R32]. The macOS captures show dark translucent rounded menus instead
  [R37].
- **Tooltips**: resting the pointer on a control shows a short description,
  sometimes with its shortcut; the delay is described only as brief and the
  look is never shown [R2][R5].
- **Status strip**: a borderless light row; hints appear on a pale cyan
  band, device news and active modes on pale green bands [R4].
- **Lists and tables**: white, with a light grey header row; some tables
  alternate white and light grey rows; the chosen row is a solid blue band
  with white text in some captures, a pale blue fill with a darker outline
  in others, light grey when the list is not focused; the row under the
  pointer is very pale blue [R7][R8][R19][R25].
- **Swatches**: flat rectangles in each layer color with a centred
  two-digit number in black or white, whichever reads better; the current
  one carries a frame [R20]. The color chip of a chosen row is drawn tinted
  [R19].
- **Progress**: one line of plain status text; while busy, a long bar with
  a pale track fills with bright green from the left, with the percentage
  as text at the right [R22].
- **Errors**: console error lines in dark red text [R24]; red text for
  invalid entries in dialogs [R29][R30]; error message boxes with a large
  red stop sign [R31].
- **Canvas cues**: a flat very light background with a thin dark bed
  outline and faint grey grid; a red origin square with red axis letters, a
  green job-origin square, a green frame outline, a red enclosing selection
  box and a green crossing one [R26][R27][R28][R22].
- **Typography**: the operating system's ordinary sans-serif at a small
  size everywhere; no bold in panels, toolbars or menus; bold only for some
  dialog and wizard headings [R2][R6][R33].
- **Density**: compact and information-dense [R1][R2].
- **Platform**: dialogs use the native title bar and the platform's button
  order [R6][R29].
- **Themes and display options**: an application theme setting with
  system, light and dark choices that needs a restart; a separate dark
  background switch for the drawing area that leaves the chrome light;
  sliders for font size and toolbar icon size; a cursor and node size
  setting; a reduce-motion switch whose effect is not described; number
  labels on the swatches can be turned off; grid strength levels
  [R6][R27]. The dark theme is mentioned in the 2.1 release notes but no
  capture shows it [R32].
- **Accessibility**: the publisher states partial conformance with WCAG
  2.1 level AA and gives no details on contrast, keyboard use or screen
  readers [R11]. A reset procedure exists for when large font or icon
  settings make the interface unusable [R6][R36].

### What LaserCraft adopts and what it does not

| Reference trait | LaserCraft | Why |
|---|---|---|
| Light, neutral, dense chrome; white fields | **Adopted** (Match) | familiarity is the point of the project |
| Dark-to-red-to-orange gradient title strips with light text | **Not adopted.** Flat `panel-title` strip with `text`; the focused group gets a thin `accent` line (decision 2) | it is the reference's most distinctive chrome (trade dress risk, `docs/clean-room.md`); red and orange would also break "red means danger"; gradients are against our flat rule |
| Green-on, red-off pill switches | **Not adopted.** Pill switch with `accent` fill and knob right when on, neutral grey and knob left when off (decision 4) | red must only mean danger; a red "off" on dozens of harmless settings would numb users to the red Stop; green/red is the pair most often confused by red-green color vision deficiency, so the state must not rest on hue |
| Heavy black filled pictograms | **Not adopted.** Medium-weight outline pictograms, one optional `accent` detail (decision 17, `icons.md`) | icons and artwork are Own |
| Teal corner marker on multi-tool buttons | **Echo.** A small corner triangle in `text-muted` (`toolbars.md`) | teal is our `guide` hue only |
| Active tool on a pale blue rounded background | **Echo.** `accent-soft` background plus a thin `accent` outline | same idea, our tokens, and the outline survives low-contrast displays |
| Tool icon turning red while active | **Not adopted.** Icons never change color with state | red is danger |
| Solid saturated blue selected row with white text | **Not adopted.** `accent-soft` row with `text` kept dark | calmer, keeps every row the same text color, works in both themes |
| Tinted color chip in the chosen row | **Not adopted.** The chip always shows the true layer color (`layers-panel.md`) | the chip exists to identify the layer |
| Pale cyan hint band, pale green device band | **Echo.** Four status bands: `info-soft`, `success-soft`, `warning-soft`, `danger-soft` | same idea, extended to warnings and errors, our hues |
| Bright green progress fill | **Not adopted.** `accent` while running, `warning` while held, `danger` after a fault | green means ready or success; a running job is neither |
| Dark red error text | **Partly.** Errors use a `danger-soft` band with a `danger` mark and `text` words | colored body text is hard to read; the mark keeps the meaning |
| Red origin square and axis letters; green job-origin square; green frame outline | **Not adopted.** `origin-marker` (dark slate), job origin by shape in the same hue, `frame-preview` in amber (`canvas.md`) | red is danger, green is success |
| Red enclosing box, green crossing box | **Not adopted.** One `marquee` hue; solid for enclosing, dashed for touching (`canvas.md`) | one meaning per hue; line style instead of hue |
| Native title bars, platform button order, macOS dark menus | **Not adopted.** One drawn look and one button order everywhere (decision 12) | egui draws everything; one look is easier to learn, test and document |
| Large black pictograms in settings tabs | **Not adopted.** Small own pictograms in the Preferences tabs (`dialogs.md`) | Own icons; calmer tab row |
| Separate dark-background switch for the drawing area | **Not adopted.** The canvas surround follows the theme and the bed stays light in every theme (decision 13, `docs/spec/canvas.md`) | black layer 0 must stay visible |
| Settings that need a restart (theme, font size) | **Not adopted.** Theme and sizes apply at once | egui can restyle at run time |
| Bold only for dialog headings | **Adopted** (Match) | keeps chrome quiet |

## Color tokens

All tokens are Own design. "Light" and "Dark" give the color in plain
words; the code holds the values. "Used by" names the style files that use
the token most. Tokens marked *new* are added by this file (see the end of
this section for why).

### Chrome surfaces and text

| Token | Role | Light theme | Dark theme | Used by |
|---|---|---|---|---|
| `window` | background behind toolbars, menu bar, palette strip, status bar, inactive dock tabs | light neutral grey | dark neutral grey (charcoal) | layout.md, toolbars.md |
| `panel` | panel and toolbar body, dialog body, front dock tab, title strip buttons' resting background | light neutral grey, same family as `window`, a hair lighter | dark neutral grey, a hair lighter than `window` | every file |
| `panel-title` | panel title strip fill | one step darker neutral grey than `panel`; no gradient | one step lighter than `panel`, so the strip still stands apart | layout.md, dialogs.md |
| `field` | input fields, lists, tables, menus, popups, tooltips, button faces | white | very dark grey, darker than `panel`, so fields read as inset | every file |
| `field-alt` | alternating table rows, table header rows, read-only and disabled fields, disabled button faces, progress track | faint grey | a hair lighter than dark `field` | dialogs.md, layers-panel.md, machine-panels.md |
| `border` | outlines of fields, buttons, lists, popups and windows; separators; splitter lines | light-mid grey, dark enough to reach the outline contrast target against `panel` (see "Contrast targets") | mid-dark grey, reaching the same target against dark `panel` | every file |
| `text` | normal text, icon strokes, values, check ticks | near-black (not pure black) | near-white (not pure white) | every file |
| `text-muted` | units, hints, secondary labels, chords in menus and tooltips, inactive tab names, ruler numbers, stepper chevrons, grips, flyout markers | mid grey | light-mid grey | every file |
| `text-disabled` | disabled text, icons and markers | light grey: clearly visible, clearly inactive | dim grey, closer to `panel` | every file |
| `hover` | background of a hovered button, row, tab, chevron or title strip button | faint tint of `accent` over the surface | faint tint of `accent` over the dark surface (a slightly bluish lift) | every file |
| `pressed` | background of a pressed button | slightly stronger tint of `accent` | slightly stronger tint of `accent` | every file |
| `text-on-strong` *(new)* | labels and icons on `accent` and `danger` fills (primary button, Stop, destructive buttons, filled check boxes) | white | white | dialogs.md, machine-panels.md, layout.md |
| `text-on-warning` *(new)* | labels and icons on a `warning` fill (the laser-on frame button) | near-black | near-black | dialogs.md |

### Accent and states

| Token | Role | Light theme | Dark theme | Used by |
|---|---|---|---|---|
| `accent` | LaserCraft's accent (decision 3): focus rings, active-tool outline, selected tab marker, focused title strip line, switch "on" track, primary button fill, chosen radio dot and point-grid dot, checked check box fill, slider filled part, progress fill while running, state dot for motion states, splitter line on hover; on the canvas the family of `selection`, `marquee`, `drawing`, `node-selected`, `preview-head` (full list in "Semantic color rules") | medium blue | a lighter, brighter medium blue, so it keeps its contrast on dark surfaces | every file |
| `accent-soft` | active tool background, checked toolbar toggle, selected list or table row, the row under the pointer in menus, popups and dropdown lists, text selection in fields | pale blue | deep muted blue, dark enough that near-white `text` stays readable on it | toolbars.md, menus.md, layers-panel.md, dialogs.md |
| `danger` | Stop, alarm, errors, destructive buttons, invalid fields, the head marker while the laser may fire | strong red | strong red, a touch lighter | machine-panels.md, layout.md, dialogs.md, canvas.md |
| `danger-soft` | band behind error messages and the alarm banner; job bar background in Alarm; invalid field background | pale red | deep muted red (dark maroon-brown) | layout.md, machine-panels.md, property-bar.md |
| `warning` | paused, door open, no response, warnings, "laser may stay on" (constant power), frame preview outline, laser-on button fill | amber, deep enough (towards golden brown, never orange) to reach the non-text contrast target on `panel` and on `bed` | warm amber, a touch lighter | machine-panels.md, layout.md, canvas.md, dialogs.md |
| `warning-soft` | band behind warnings; job bar background during a constant-power job | pale amber | deep muted amber-brown | layout.md, job-preview.md, dialogs.md |
| `success` | connected and ready (state dot), finished, the success mark | green | lighter green | machine-panels.md, layout.md |
| `success-soft` | band behind success and ready messages | pale green | deep muted green | layout.md |
| `info-soft` | band behind hints in the status bar and in dialogs; Active setups field; drop zone fill | pale blue-grey, clearly greyer than `accent-soft` | deep blue-grey | layout.md, dialogs.md |

### Canvas

Everything drawn on the bed uses the same color in both themes, because the
bed itself is the same near-white sheet in both themes. Overlays that can
leave the bed (a selection box that extends past the edge, a marquee
started outside the bed) get a *hairline* halo in `handle-fill` in the dark
theme so they still read on the dark `canvas`.

| Token | Role | Light theme | Dark theme | Used by |
|---|---|---|---|---|
| `canvas` | area around the bed | neutral grey a little darker than `bed` | dark neutral grey, between `window` and `field` | canvas.md |
| `bed` | the work-area sheet | near-white | near-white (unchanged) | canvas.md, job-preview.md |
| `bed-edge` | outline of the work area | dark neutral, *thin* | same | canvas.md, job-preview.md |
| `grid-minor` | minor grid lines | very faint grey | same (they lie on the bed) | canvas.md |
| `grid-major` | major grid lines | faint grey | same | canvas.md |
| `ruler` | ruler band | `panel`-like grey | dark `panel`-like grey | canvas.md |
| `ruler-text` | ruler numbers and ticks | `text-muted` | `text-muted` (dark) | canvas.md |
| `origin-marker` | machine-origin marker and axis letters; job origin marker (hollow form) | strong dark slate: darker and heavier than `bed-edge`, not black, no semantic hue | same | canvas.md, machine-panels.md |
| `selection` | selected objects' bounding box and handle outlines; reference-point cue | a deeper shade of the `accent` family, dashed box | same | canvas.md, property-bar.md |
| `handle-fill` | fill of transform handles; halos behind markers | white | white | canvas.md |
| `marquee` | drag-select rectangle | `accent` outline with a very faint `accent` fill; solid while enclosing, dashed while touching | same, with the dark-theme halo off the bed | canvas.md |
| `guide` | snapping and alignment lines, measure read-outs; the only teal | deep teal, dark enough to reach the non-text target on `bed` | same | canvas.md |
| `drawing` | a shape being drawn; live previews of tool windows | `accent`, dashed | same | canvas.md, dialogs.md |
| `node` | node handles | white with a dark outline | same | canvas.md |
| `node-selected` | chosen node handles | filled `accent` | same | canvas.md |
| `start-marker` | path start point and direction marks; the start node | dark violet, distinct from `accent`, `selection` and the palette's darkest violet | same | canvas.md |
| `head-marker` | laser head position | dark neutral crosshair while the laser is off; drawn in `danger` with a faint `danger-soft` disc only while the laser may be firing | same | canvas.md, machine-panels.md |
| `frame-preview` | outline of the area a frame command will trace | `warning`, dashed | same | canvas.md, machine-panels.md |

### Job preview

| Token | Role | Light theme | Dark theme | Used by |
|---|---|---|---|---|
| `move-burn` | laser-on moves | the layer's color (palette data) | same | job-preview.md |
| `move-unlit` | laser-off moves inside a cut (overscan, unlit returns) | light grey, dotted (deepened only as far as needed to reach the non-text target on `bed`), lighter than `move-travel` but still at the non-text target on `bed` | same | job-preview.md |
| `move-travel` | rapid travel between cuts | mid grey, dashed; never red | same | job-preview.md |
| `preview-head` | simulated head | `accent` dot with a `handle-fill` halo | same | job-preview.md |
| `preview-sheet-inverted` *(new)* | the sheet in the preview's inverted view | near-black | near-black (independent of theme) | job-preview.md |
| `move-burn-inverted` *(new)* | burn moves in the inverted view | near-white, or grey levels with "Lighter for low power" | same | job-preview.md |

### Floating layers

| Token | Role | Light theme | Dark theme | Used by |
|---|---|---|---|---|
| `shadow` *(new)* | soft shadow under menus, popups, tooltips, dialogs, floating panels and floating toolbars | faint, soft, dark grey, short offset downward | a little stronger, near-black | menus.md, toolbars.md, dialogs.md, layout.md |
| `backdrop` *(new)* | translucent veil over the main window while a message window is shown (never over the job bar, `dialogs.md`) | translucent mid-dark grey that dims without hiding | translucent near-black, a little stronger | dialogs.md |

### Proposed guide layer colors (later, owner decision)

| Token | Role | Light theme | Dark theme | Used by |
|---|---|---|---|---|
| `guide-layer-1` *(proposed)* | fill of guide swatch 1 and stroke of its shapes | muted warm grey-brown (taupe) | same (palette-like data, theme independent) | layers-panel.md, canvas.md |
| `guide-layer-2` *(proposed)* | fill of guide swatch 2 and stroke of its shapes | muted grey-olive | same | layers-panel.md, canvas.md |

Both are deliberately desaturated so they cannot be mistaken for any of the
30 saturated palette colors, for teal `guide`, for the blue `selection`
family or for the dark slate `origin-marker`. They exist only if the owner
approves guide layers (`layers-panel.md` open questions).

### Why the new tokens

- `text-on-strong`, `text-on-warning`: other files wrote `field` for labels
  on strong fills, which means "white" only in the light theme; in the dark
  theme `field` is very dark and the Stop label would vanish. Labels on
  strong fills need a token that does not follow the theme.
- `preview-sheet-inverted`, `move-burn-inverted`: `job-preview.md`
  reused `text` and `bed` for its inverted view; `text` turns near-white in
  the dark theme and would un-invert the sheet. Answer to `job-preview.md`
  open question 8.
- `shadow`, `backdrop`: other files referred to "egui's shadow" and a veil
  (`dialogs.md` open question 5) without a token.
- No `job-origin-marker` token: the job origin marker shares
  `origin-marker` and differs by shape (hollow ring with a cross versus
  the filled corner square), as `canvas.md` specifies. One hue for "origin"
  keeps the canvas calm, and shape survives color vision deficiency better
  than a second dark hue would. Answer to `canvas.md` open question 7 and
  `machine-panels.md` question 14.

## Semantic color rules

### One meaning per hue

| Hue | Meaning | Tokens | Allowed uses (complete list) |
|---|---|---|---|
| Blue (accent) | in the UI: focus, the current or chosen item, "on", the main action, motion in progress; on the canvas: selection and what is being made | `accent`, `accent-soft`, `hover`, `pressed`; canvas `selection`, `marquee`, `drawing`, `node-selected`, `preview-head` | focus rings; active-tool outline and background; checked toolbar toggles; selected tab marker (dock tabs and page tabs); focused title strip line; switch on track; chosen radio dot; chosen point-grid dot; checked check box; primary button fill; selected list and table rows; the row under the pointer in menus, popups and dropdown lists; text selection in fields; slider filled part; progress fill while running; state dot for Running, Framing, Jogging, Homing; splitter line on hover; the highlight ring around Stop during a constant-power (M3) job (safety rule 9); one optional detail per pictogram (`icons.md`); the canvas tokens named here |
| Red (danger) | danger: something stops, failed, is destructive, or the laser may be firing | `danger`, `danger-soft` | Stop (job bar, Machine panel, its menu row icon detail); destructive dialog buttons; Alarm state dot and banner mark; error marks and bands; invalid field outline and background; job bar background in Alarm; head marker while the laser may fire; danger pictogram detail on Stop and delete-everything actions only (decision 17); the Danger message mark (`dialogs.md`) |
| Amber (warning) | caution: held, waiting, a risk the user must notice, "the laser may stay on" | `warning`, `warning-soft` | Paused, Door open and No response state dots; held progress fill; warning marks and bands; the constant-power job bar background and its note mark; the laser-on frame button fill (later); `frame-preview` on the canvas |
| Green (success) | ready, connected, done | `success`, `success-soft` | Ready state dot; success and "device found" bands; the finished-job mark |
| Teal (guide) | a transient snapping or alignment cue | `guide` | snapping and alignment lines and their read-outs on the canvas only |
| Violet (start) | where a cut starts and which way it runs | `start-marker` | start arrows, start rings, the start node |
| Dark slate (origin) | origins | `origin-marker` | machine origin marker, axis letters, job origin marker |
| Neutral greys | everything else | chrome tokens | chrome, text, borders, grids, travel and unlit moves |
| Layer colors | which layer a thing belongs to | palette data | swatch chips, shape strokes and fills, burn moves; nowhere else |

### Accent beyond decision 3

Decision 3 lists six uses of `accent` and says "nowhere else"; decision 6
gives blue the meaning "selection in the UI, focus, active". The complete
list above is the reading of both decisions together: every entry is either
a focus cue, a "this is the current/chosen/on item" cue, the main action,
or a "motion is in progress" cue. Entries not covered by decision 3's
literal list (selected rows, menu rows, text selection, choice dots,
check boxes, slider fill, progress and state dots, the splitter hover line,
the canvas selection family) are listed as an owner confirmation under
Open questions. Everything not in the list is forbidden: links, headings,
title strip fills, decorative lines, drop zones, info bands, chart colors.

### Forbidden uses

- **No gradients** anywhere.
- **No red or orange in chrome**: never on title strips, tabs, toolbars,
  menu bars, headers, separators, panel borders or focus cues. Orange is not
  used at all; amber is kept clearly on the golden side so it never reads as
  orange.
- **Red only for danger.** Never for "off", "origin", "travel", "remove from
  selection", "enclosing box", "active tool" or "error" text in normal body
  copy.
- **No green and red pair for on and off.** Switches are blue and grey and
  also move their knob.
- **Green only for ready and success.** Never for "on", "drawing", "bed
  edge", "job origin" or "frame".
- **Color is never the only carrier**: every colored state has a second cue
  (a word, a mark, a knob position, a line style, a shape). See
  "Accessibility".
- **No saturated solid selection bands with white text.** Selection is
  `accent-soft` with `text`.
- **No palette colors in chrome** except swatch chips and layer strokes.
  No layer color is ever used to mean a state.
- **No colored body text.** Colored words are limited to short state words
  that meet the text contrast target in that theme (see "Contrast
  targets"); messages are `text` on a soft band with a colored mark.
  `warning` is never used for text in the light theme (amber cannot reach
  the text target on light grey).
- **No translucency** except `backdrop`, `shadow`, the `marquee` fill, the
  `danger-soft` disc behind the firing head marker, and the faded shapes of
  layers with Output off.
- **No bevels, emboss, inner shadows or glossy fills.** Recessed areas are
  shown by `field` against `panel` and a *thin* `border`, never by shading.
- **Nothing flashes or blinks** (WCAG 2.3.1), including alarms.
- **Icons do not change color with state** (active, hovered, checked); the
  background changes instead.

### Layer colors and the chrome

- **Palette data stays data.** The 30 layer colors come from
  `lasercraft_project::layer::palette` (decision 5); they are not tokens and
  this guide does not change them.
- **Swatch chips** are the one widget that shows a layer color in chrome:
  the Palette strip, the color column of the layer list, the layer column
  of Layer settings, the job preview legend and any later list of layers
  use the same chip (see "Swatch chips" in the catalogue).
- **Contrast text on chips**: the number on a chip is black or white,
  whichever has the higher WCAG contrast ratio against the chip's fill,
  computed in code from the palette data (`layers-panel.md`). Black and
  white here are fixed, not `text`, so chips look the same in both themes.
- **Chips are never tinted** by selection, hover or focus; those states
  draw around the chip (outline, ring, row background), not on it.
- **Shapes keep their layer color** when selected; selection is drawn by
  the `selection` box and handles, never by recoloring the shape
  (`canvas.md`). Layers with Output off are drawn faded, by reduced opacity
  of the same color, not a grey substitute (`canvas.md`).
- **Distance between tokens and palette**: `selection`, `marquee`,
  `drawing`, `guide`, `start-marker`, `origin-marker`, `head-marker`,
  `frame-preview` and the semantic hues must each stay clearly apart from
  every palette color they could be drawn over or next to. A unit test in
  the theme module computes a perceptual color difference between each
  canvas token and every palette color and fails below a threshold kept in
  that test (values never in specs). Where a palette color sits close to a
  token (the palette's blues near `selection`, its violets near
  `start-marker`, its teals near `guide`), the token's line style (dashes,
  markers, rings) carries the distinction and the test documents the pair.
- **Low-contrast palette colors on the bed** (bright yellows; see
  current-ui.md 2) are an owner question (decision 5): `canvas.md` proposes
  a darker *hairline* edge for them instead of changing palette data. This
  file adds nothing to that proposal.

## Typography

### Fonts

- LaserCraft uses egui's bundled default fonts (eframe feature
  `default_fonts`): one proportional sans-serif for all UI text and one
  monospace font. They ship with egui under licenses already accepted with
  the egui dependency, look the same on every platform, and need no
  font-loading code. No platform font is used for chrome (decision 12).
  System fonts appear only as user choices for text shapes
  (`docs/spec/text.md`).
- egui's bundled set has a single weight per family. "Bold" in this guide
  therefore needs a bold face of the same family to be added as an asset
  (license recorded, owner decision, Open questions). Until then every place
  marked bold is drawn at its size in `text` with egui's strong emphasis,
  and headings rely on their larger size. Nothing else may imitate bold (no
  double drawing, no fake stroke).
- No italics in chrome. Placeholders and hints use `text-muted`, not a
  slant.

### Type tokens

| Token | Face | Relative size | Weight | Used for |
|---|---|---|---|---|
| `type-body` | proportional | the normal UI size (egui's body size at the user's Text size) | regular | labels, values, buttons, menu rows, tabs, title strips, tooltips, list rows, banners, dialog text |
| `type-small` | proportional | one step smaller than `type-body` | regular | units after values where shown, hints under fields, table headers, ruler numbers, swatch numbers, the status bar, the sent percentage, caption rows in popups, toolbar text fallback labels |
| `type-heading` | proportional | one step larger than `type-body` | bold (see Fonts) | section headings inside dialogs and settings windows, wizard page headings, the product name in About; nowhere else |
| `type-mono` | monospace | same size as `type-body` (at `type-small` where the field is `type-small`, e.g. the status bar pointer position) | regular | the Console log and its input line, the Command line log and input, G-code shown anywhere, live coordinate read-outs (status bar pointer, Machine panel position, Jog read-outs), preview time value, raw GRBL lines in banners' details |

- Steps are relative to `type-body` and scale with the Text size
  preference and the display scale; nothing has a fixed size.
- **Where bold is used**: `type-heading` only, plus the canvas axis letters
  (`canvas.md`). Never in menus, toolbars, title strips, tabs, buttons,
  table headers, status bar or panels (Match [R2][R6]).
- **Case**: sentence case everywhere (`naming.md`); never all capitals,
  never small capitals, no letter-spacing tricks.
- **One line per control**: labels, buttons, menu rows, tabs and table
  cells never wrap; when space runs out the text is cut with "…" and the
  full text becomes the tooltip. Only tooltips, banners, hints, dialog
  paragraphs and the Console wrap.

### Numbers

- Values in number fields are right-aligned inside the box, so decimal
  points line up in a column of fields (`property-bar.md`).
- Each quantity shows a fixed number of decimals (`naming.md`, Numbers), so
  a column never jumps between "10" and "10.25".
- Numeric table columns (speed, power, times, sizes) are right-aligned;
  text columns are left-aligned; headers follow their column's alignment.
- Read-outs that change while the user watches (pointer position, head
  position, elapsed time) use `type-mono` so digits keep their width.
- Units: in the label in parentheses ("Speed (mm/s)") for fields; after the
  value with a space in read-outs ("x 12.34 y 56.78 mm"); never inside an
  editable box (`naming.md`, Units).
- Minus signs and multiplication signs use the proper characters (−, ×)
  where egui's font has them; times use h:mm:ss (`naming.md`, Numbers).

## Size and spacing tokens

| Token | Meaning | Relative to | Used for |
|---|---|---|---|
| `row` | height of a normal button, field, dropdown, list row, tab, menu row (a little less for menu rows) | egui's interaction size at the user's Text size | almost everything |
| `row-big` | about twice `row` | `row` | the big job buttons of the Machine panel (Pause/Resume, Stop, Start; Frame, Frame outline) |
| `icon` | toolbar icon square, one `row` tall | `row` | toolbar buttons, jog pad buttons, More button |
| `icon-small` | small pictogram, about the height of a capital letter plus a little | `type-body` | menus, flyout and More rows, title strip buttons, icons inside ordinary buttons, marks in bands and banners, Preferences tabs |
| `icon-big` | large pictogram, about the height of `row-big` minus a margin | `row-big` | big job buttons, Job setups toolbar |
| `gap` | standard spacing between controls and between rows of a form | egui item spacing | most spacing |
| `gap-tight` | spacing inside a control group: label to field, button to button in a toolbar group, padding inside bands | about half `gap` | toolbars, forms, bands |
| `gap-wide` | spacing between groups, around window content, between toolbars in a row | about twice `gap` | dialogs, panels, toolbars |

Shared shapes and weights (named words, not tokens):

- *Slightly rounded*: one small radius for buttons, fields, dropdowns,
  popups, menus, tooltips, windows, swatches, bands and banners. Panels,
  title strips, the status bar and the toolbar band are *square*. Tabs are
  square except the two outer corners away from the panel, which are
  *slightly rounded*. Menus use the same radius as popups (answer to
  `menus.md`: one radius for every popup).
- *Pill*: switches (including the handle switches in the status bar),
  state dots (full circles), the progress bar and scrollbar handles.
- Borders are *thin*; focus rings and meaningful outlines are *normal*;
  the current-swatch mark and the constant-power Stop ring are *heavy*.

### Density rules

- **Forms**: one control per row; labels right-aligned, ending `gap-tight`
  before their control (Match [R2][R22][R29]); the label column is as wide
  as its longest label; rows are `row` tall with `gap` between them.
- **Two columns** of label and field pairs where a panel or dialog is wide
  enough; one column otherwise (`dialogs.md`, `machine-panels.md`).
- **Groups** are separated by `gap-wide` and, in dialogs, a `type-heading`
  heading; framed groups are used only where several groups sit side by
  side on one page (see "Group frames and section headings").
- **Margins**: panels and dialogs keep `gap-wide` around their content;
  toolbars keep a thin margin above and below their buttons.
- **No decorative empty space**: extra width in a toolbar or panel stays
  empty `panel`, controls are not stretched to fill it, except fields and
  dropdowns that this guide says stretch.
- **Minimum targets**: every clickable thing (including title strip
  buttons, stepper chevrons, tab close areas, swatches and the More button)
  has a hit area at least `row` high, even when its drawing is smaller
  (`icon-small`); this meets WCAG 2.2's minimum target size at the default
  scale.
- **Scaling**: all tokens derive from egui's interaction size and font
  sizes, so the Text size preference (egui zoom factor, later) and the
  display's scale factor grow and shrink everything together. Nothing is
  laid out with absolute positions.

## Widget catalogue

Each widget lists its states. Unless a widget says otherwise, these rules
hold for all:

- **Hover** uses `hover` and appears only on enabled widgets.
- **Keyboard focus** shows an `accent` focus ring, *normal* weight, just
  outside the widget's outline, following its corners; it appears only when
  focus came from the keyboard (Tab, arrow keys), never after a mouse click.
- **Disabled** draws text, icons and marks in `text-disabled`, keeps the
  widget in place, takes no clicks, and still shows its tooltip with a
  second line saying why (`naming.md`, Tooltips).
- **Error** is shown with `danger` outline and a `danger-soft` fill plus a
  message band; it never relies on color alone.

### Push button

Reference: light rectangles with a thin border and slight rounding,
centred regular-weight dark text, optional small icon left of the text;
equal widths in a dialog's button row; the default button outlined or
filled in blue depending on the platform [R2][R6][R22][R29].

LaserCraft:

- Shape: `row` tall, *slightly rounded*, width = label plus `gap` on each
  side (or the shared width of its group, `dialogs.md`). Optional
  `icon-small` left of the label with `gap-tight` between.
- Styles (`dialogs.md` decides where each is used):

| State | Normal | Primary | Destructive | Laser on (later) |
|---|---|---|---|---|
| Rest | `field` face, *thin* `border` outline, `text` label | `accent` fill, no outline, `text-on-strong` label | `danger` fill, no outline, `text-on-strong` label | `warning` fill, *thin* `text-on-warning` outline, `text-on-warning` label |
| Hover | `hover` face | fill one shade deeper | fill one shade deeper | fill one shade deeper |
| Pressed | `pressed` face | fill two shades deeper | fill two shades deeper | fill two shades deeper |
| Keyboard focus | as rest plus `accent` focus ring | as rest plus focus ring with a `panel` gap so it reads against the fill | as Primary | as Primary |
| Disabled | `field-alt` face, *thin* `border`, `text-disabled` label | `field-alt` face, *thin* `border`, `text-disabled` label (a disabled primary looks like any disabled button) | as Primary disabled | as Primary disabled |
| Default (Enter) | no extra mark: the primary style is the default button | — | — | — |

- One primary button per window at most (`dialogs.md`).
- Labels are verbs or nouns from `naming.md`; never all capitals.
- A button never keeps keyboard focus after a mouse click that ran a canvas
  action (focus returns to the canvas, `toolbars.md`); dialog buttons keep
  egui's normal focus.

Tier: **Echo** (light bordered buttons, highlighted default), **Own**
(styles, tokens, one order everywhere). Milestone: restyle now.

### Big job button

Reference: the job-control window's main buttons are the ordinary button
style about twice as tall, in rows of equal widths, each with a bold glyph
left of its label; disabled ones turn grey [R22]. Hover and pressed looks
are unknown [R22].

LaserCraft:

- `row-big` tall, equal widths across a row, `icon-big` left of a
  `type-body` label, icon and label centred together (`machine-panels.md`).
- Faces: Pause/Resume, Start, Frame, Frame outline use the Normal style;
  **Stop** always uses the Destructive style (`danger` fill,
  `text-on-strong` icon and label) and is the only `danger`-filled big
  button, so it is the strongest control on screen (decision 7). The later
  laser-on frame button (`warning` fill, `dialogs.md`) always stays
  visually weaker than Stop: smaller, and never beside it.
- States as Push button. Additionally:
  - Constant-power job (safety rule 9): Pause reads "Pause (motion only)"
    with a `warning` mark beside its icon; Stop gains a *heavy* `accent`
    ring outside its fill (the highlighted action) in the Machine panel and
    the job bar.
  - Pause turns into Resume in Paused and Door open: the label and icon
    change, the face stays Normal; a `warning` mark sits at the label's
    start so the held state is visible on the button itself.
- The compact job bar buttons (`layout.md`) are ordinary `row`-tall
  buttons with `icon-small`, in the same styles.

Tier: **Echo** (tall job buttons with icons), **Own** (red Stop, warning
marks, ring). Milestone: M6.

### Icon button (in panels)

Reference: small square buttons with a light face, thin border and a
black glyph, used beside the layer list and in the jog pad [R19][R23].

LaserCraft:

- Square, one `row` on a side, `field` face, *thin* `border`, *slightly
  rounded*, pictogram `icon-small` (panel side columns) or `icon` (jog pad)
  in `text`. Hover `hover`, pressed `pressed`, focus ring, disabled as
  above.
- Buttons that belong together touch or sit `gap-tight` apart as a pair
  (up and down, copy and paste); pairs are separated by `gap`
  (`layers-panel.md`).
- Every icon button has a tooltip with its name and chord; there is no
  icon button without a name.
- Title strip buttons (float, close) and band close buttons (×) are a
  borderless variant: no face at rest, glyph in `text-muted`; hover shows a
  `hover` square and the glyph in `text`.

Tier: **Echo**. Milestone: with the layer list buttons (restyle) and M6.

### Toolbar button

Reference: square, flat, icon-only buttons of one size per toolbar, with no
frame at rest; disabled glyphs pale; the active tool on a soft rounded
background; hover and pressed looks not shown [R2][R12][R16].

LaserCraft (full table in `toolbars.md`, summarised here as the master):

| State | Background | Pictogram | Extra |
|---|---|---|---|
| Normal | none (`panel` shows) | `text` strokes, at most one `accent` detail | — |
| Hover | `hover`, *slightly rounded* | unchanged | tooltip after the delay |
| Pressed | `pressed` | unchanged | — |
| Active tool | `accent-soft` | unchanged | *thin* `accent` outline around the background |
| Checked toggle | `accent-soft` | unchanged | no outline |
| Hover on active or checked | stays `accent-soft` | unchanged | — |
| Disabled | none | strokes and accent detail in `text-disabled` | tooltip still shows, with the reason |
| Keyboard focus | as underneath | unchanged | `accent` focus ring (only if toolbar keyboard navigation is adopted, `toolbars.md`) |

- One `icon` square; no captions (until icons exist the transition label
  in `type-small` is allowed, `toolbars.md`).
- Toolbar buttons never use `danger`, `warning` or `success`.

Tier: **Echo** (flat icon-only, active background), **Own** (tokens,
outline, no color change of the icon). Milestone: when the toolbar band is
built.

### Flyout button and its marker

Reference: a button holding related tools carries a small colored triangle
in a corner; a quick click repeats the last choice, pressing and holding or
clicking the corner opens a list; the list opens to the right in the tool
column as a rounded popup with icon and name rows [R2][R16][R18]. Release
2.1 made such buttons slightly wider [R32].

LaserCraft:

- Same square and states as a toolbar button. The **marker** is a small
  solid right-angled triangle filling the lower-right corner, in
  `text-muted`; disabled `text-disabled`. Never teal, never `accent`
  (`toolbars.md`).
- The open flyout is a popup in the menu look (see "Menus and context
  menus"), with a check tick on the member shown on the face.
- Behaviour (hold, corner click, right-click, face follows the last
  choice): `toolbars.md`.

Tier: **Echo**. Milestone: with the toolbar band.

### Toggle switch

Reference: tiny pill switches used instead of check boxes in panels,
dialogs, toolbars and the status strip; on is bright green with the knob at
one end, off dark red with the knob at the other, disabled dark grey; the
label usually follows the switch [R2][R6][R9][R19][R29][R33][R35].

LaserCraft (decision 4):

- Shape: a horizontal *pill* about one capital-letter height plus a little
  tall and about twice as wide as tall, with a round knob inside one end;
  the switch sits left of its label with `gap-tight` between; clicking the
  label toggles it too.
- States:

| State | Track | Knob | Label |
|---|---|---|---|
| Off | neutral grey (`text-muted` family, reaching the non-text target on `panel`) | `field`-white knob at the **left**, *thin* `border` ring | `text` |
| On | `accent` | white knob at the **right** | `text` |
| Hover (off) | track one shade darker | unchanged | unchanged |
| Hover (on) | `accent` one shade deeper | unchanged | unchanged |
| Pressed | knob grows slightly wider toward the other side | — | — |
| Keyboard focus | `accent` focus ring around the whole pill | — | — |
| Disabled off | track `text-disabled` | knob left, faded | `text-disabled` |
| Disabled on | track a faded `accent` (`accent-soft` family) | knob right, faded | `text-disabled` |
| Varies (several objects differ, Properties later) | neutral grey track | knob in the centre | `text` |

- The knob position carries the state independently of color (WCAG 1.4.1).
- Switch change commits at once (one command or front-end action), like a
  click; there is no Apply needed unless the window says so (`dialogs.md`).
- The knob slides between ends over a short animation; with Reduce motion
  it jumps (see "Motion").
- White knobs are fixed white in both themes, so the dark-theme switch
  keeps a light knob on a dark or blue track.

Tier: **Echo → Own** (decision 4). Milestone: replaces today's check boxes
in the layer list and editor and the Circular array window (restyle now).

### Check box

Reference: the docs show check marks and empty boxes only in the window
menu's toggle rows; panels and dialogs use switches instead [R13][R2].

LaserCraft:

- Settings never use check boxes: every on/off setting is a switch.
- A check box is used only to **pick items from a list for a one-off
  action** (for example which layers a later export or test grid includes,
  or which checklist items the user has confirmed if the owner prefers
  boxes there), where a row of switches would wrongly suggest lasting
  settings.
- Look: a *slightly rounded* square about `icon-small`, `field` fill with a
  *thin* `border`. Checked: `accent` fill with a `text-on-strong` tick.
  Varies (some of a group): `accent` fill with a short dash. Hover: border
  `text-muted`. Focus: `accent` ring. Disabled: `field-alt` fill,
  `text-disabled` tick and border.
- Menu toggle rows do not draw a box: they show a tick in the gutter
  (`menus.md`).

Tier: **Own**. Milestone: whenever such a list first appears.

### Radio button ("choice")

Reference: plain circles; the chosen one has a blue or dark centre;
disabled ones grey [R2][R6][R26].

LaserCraft:

- A circle about `icon-small` across, `field` fill, *thin* `border` ring.
  Chosen: a filled `accent` dot inside. Hover: ring `text-muted`. Focus:
  `accent` ring around the circle. Disabled: `field-alt` fill,
  `text-disabled` ring, chosen dot `text-disabled`.
- Label to the right with `gap-tight`; clicking the label chooses.
- Choices in one group are stacked one per row, or in one row when there
  are two or three short options (`dialogs.md`).
- The chosen dot plus the label carry the state; the dot is a filled shape,
  not only a color.

Tier: **Echo**. Milestone: with the first window that needs it.

### 3×3 point grid

Reference: a 3×3 grid of round radios inside a slightly recessed box picks
which point of the job's or the selection's bounding box is used; the whole
grid greys out when the placement mode makes it meaningless [R22][R26].

LaserCraft (`property-bar.md`'s Reference point and `machine-panels.md`'s
Job reference point):

- Nine small circles on a square grid, about as tall as two `row`s, each a
  choice dot as above, joined by *hairline* `border` lines that outline the
  bounding box they stand for (so it reads as "points of a box", not nine
  loose options). No recessed box, no shading: the grid sits on a `field`
  square with a *thin* `border`.
- Chosen point: filled `accent` dot, a little larger than the others.
- Hover: the dot under the pointer gets a `hover` disc; its tooltip names
  the point ("Reference point: top left", proposed label, `naming.md`).
- Disabled: all dots and lines `text-disabled`; the chosen dot stays filled
  in `text-disabled` so the stored choice is still visible.
- Keyboard (if adopted, `property-bar.md` question): arrow keys move the
  choice; the focus ring surrounds the whole grid.

Tier: **Echo** (3×3 point picker), **Own** (box outline, no recess).
Milestone: Position and size toolbar (later); M6 Machine panel.

### Number field (spin box)

Reference: a white box with tiny stacked up/down arrows at its right; fixed
decimals; the unit as a word after the box or in parentheses in the label;
a blue border on focus; disabled grey; the mouse wheel changes the value
[R2][R6][R21][R29].

LaserCraft (master look; behaviour and the full state table in
`property-bar.md`):

| State | Box | Text | Stepper |
|---|---|---|---|
| Normal | `field`, *thin* `border`, *slightly rounded*, `row` tall | value `text`, right-aligned | two chevrons stacked in the box's right end, `text-muted` |
| Hover | outline `text-muted` | unchanged | chevron under the pointer on `hover` |
| Stepper pressed | unchanged | unchanged | chevron on `pressed` |
| Focused (editing) | `accent` focus ring; caret `text` | value selected on focus, selection in `accent-soft` | shown |
| Invalid | `danger` outline, `danger-soft` fill | `text` | shown |
| Varies | `field` | placeholder "varies" (`naming.md`; not "mixed", which is a layer mode) in `text-muted` | shown |
| Read-only | `field-alt` | `text-muted` | hidden |
| Disabled | `field-alt`, *thin* `border` | empty or as specified, label `text-disabled` | hidden |

- The label carries the unit in parentheses (`naming.md`); the box never
  shows a unit suffix.
- Dragging on the value to scrub (egui `DragValue`) stays available but
  shows the horizontal-resize cursor so it is discoverable; the drag is one
  gesture and one undo step (`docs/spec/ui-shell.md`).

Tier: **Echo**. Milestone: restyle of today's `DragValue`s now.

### Text field

Reference: white boxes with a thin border; a blue focus border; grey
placeholder text in parentheses in the console input [R9][R24][R7].

LaserCraft:

- `field` box, *thin* `border`, *slightly rounded*, `row` tall (multi-line
  boxes: a stated number of lines, `dialogs.md`); text `type-body` `text`,
  left-aligned; placeholder in `text-muted`, no parentheses, written as a
  hint ("Search commands").
- States as Number field without the stepper: Hover outline `text-muted`;
  Focused `accent` ring with the caret in `text` and selection in
  `accent-soft`; Invalid `danger` outline and `danger-soft` fill; Read-only
  `field-alt` with `text` (still selectable for copying); Disabled
  `field-alt` with `text-disabled`.
- Console and Command line inputs use `type-mono`.
- A search field shows a small magnifier pictogram (`icon-small`,
  `text-muted`) at its left and a clear "×" at its right while it holds
  text.

Tier: **Echo**. Milestone: restyle now.

### Dropdown

Reference: a white box with the current choice and a small down chevron at
the right; the open list is white with a blue band on the row under the
pointer, sometimes with white text; an older platform shows double-arrow
steppers [R2][R6][R25][R30].

LaserCraft:

- Closed: `field` box, *thin* `border`, *slightly rounded*, `row` tall;
  current option in `type-body` `text`, left-aligned; a small chevron in
  `text-muted` at the right. Hover: outline `text-muted`. Open: outline
  `accent`, chevron points up. Focus: `accent` ring. Disabled: `field-alt`,
  `text-disabled`.
- Open list: a popup in the menu look; the row under the pointer and the
  current row on `accent-soft` with `text` (never white text on solid
  blue); the current row also carries a tick in the gutter so it is not
  shown by color alone; a scrollbar appears when the list is longer than
  about twelve rows.
- Options are readable names, never raw ids (`naming.md`; today's
  `floyd_steinberg` and lowercase `line` are listed under "Current state").
- Typing a letter while the list is open jumps to the next option starting
  with it (own design; implement with egui's popup and our key handling).

Tier: **Echo**. Milestone: restyle now.

### Slider

Reference: a thin groove with a small light rounded-rectangle thumb (blue
at rest in one capture) and the value or level name as text to the right
[R6][R30].

LaserCraft:

- Groove: a *thin* `border` line across the slider's width; the part from
  the start to the knob filled `accent` (the "active value").
- Knob: a small circle in `field` (white in both themes) with a *thin*
  `border` ring; hover: ring `text-muted`; dragging: ring `accent`;
  focus: `accent` focus ring; disabled: groove and knob `text-disabled`,
  no fill.
- Value: a compact number field to the right (egui's slider value) in
  `type-mono` when it is a live quantity (`job-preview.md`), or a level
  word in `text-muted` for stepped sliders (Text size, Icon size later).
- Label right-aligned to the left like every label.

Tier: **Echo**. Milestone: job preview time slider (restyle now), M7
image sliders.

### Tab strips

Reference: dock groups have plain text tabs along their bottom edge, the
chosen one lighter and joined to the panel above, others light grey; long
captions are cut with an ellipsis in a narrow column; one older platform
draws a centred segmented control instead [R2][R3][R19][R22]. Dialog tabs
sit on top of a framed page; settings tabs add large pictograms [R6][R10]
[R30].

LaserCraft has two forms with one look, mirrored:

| Part | Dock tab strip (bottom of a group, `layout.md`) | Page tabs (top of a dialog page, `dialogs.md`) |
|---|---|---|
| Strip background | `window` | `panel` |
| Chosen tab | `panel` fill joined to the panel above (no line between), name `text`, a short `accent` marker line along its **lower** edge | `field` (or the page color) joined to the page below, name `text`, `accent` marker line along its **top** edge |
| Other tabs | `window` fill, name `text-muted`, *thin* `border` between tabs | `panel` fill, name `text`, *thin* `border` between tabs |
| Hover | `hover` fill | `hover` fill |
| Keyboard focus | `accent` focus ring inside the tab | same |
| Disabled tab | name `text-disabled`, no hover; tooltip says why | same |
| Content | text only, `type-body`, sized to text with `gap` padding | optional `icon-small` then text (Preferences only) |
| Too narrow | captions shortened with "…", then a "»" More button listing the rest | the window keeps its size (dialogs are not narrow) |

- The marker line is on the tab's edge away from the content in both
  forms, so "chosen" is shown by fill, joining and a line, not by color
  alone.
- No close buttons on tabs; no segmented-control style on any platform.

Tier: **Match** (bottom text tabs on dock groups, top tabs in dialogs),
**Own** (look). Milestone: dock tabs M6 (stage a); page tabs with the first
tabbed dialog.

### Panel title strip

Reference: a strip with a dark-to-red-to-orange fill, small light title
text at the left and two pale buttons (float, close), at the right on
Windows and at the left on macOS [R2][R3][R19][R22]. Not adopted, see the
table above.

LaserCraft (decision 2; placement and buttons in `layout.md`):

- Fill `panel-title`, flat, *square*, a little shorter than `row`, full
  width of the group.
- Title: the front panel's name in `type-body`, `text`, regular weight,
  left-aligned with `gap` padding.
- Buttons at the right on every platform: float (stage b) and close, the
  borderless icon button variant (`icon-small` in `text-muted`; `hover`
  square and `text` glyph on hover).
- **Focused group**: a *normal* `accent` line along the strip's lower edge,
  full width. Only one group shows it at a time; canvas focus removes it.
- Dialog and tool windows use the same strip (`dialogs.md`), so every
  title in LaserCraft looks alike.
- Drag cursor (stage b): the move cursor over the strip.

Tier: **Echo → Own**. Milestone: M6 (stage a) when the second dock panel
arrives; dialogs at their restyle.

### Splitter and its grip

Reference: the splitter between stacked dock groups carries a short row of
small dots [R3][R19].

LaserCraft:

- A *thin* `border` line with a short centred grip of a few small dots in
  `text-muted` (a row of dots on horizontal splitters, a column on vertical
  ones).
- Hover and drag: the line turns `accent` and the pointer becomes the
  matching resize cursor. Double-click returns the splitter to its default
  (`layout.md`).
- The hit area is wider than the line (at least `gap` on each side).

Tier: **Match** (dotted grips), **Own** (look). Milestone: M6.

### Separator

Reference: thin, sometimes dotted, lines between groups of toolbar buttons;
thin full-width rules between groups of settings [R12][R16][R21].

LaserCraft:

- A *thin* `border` line, solid (never dotted), with `gap` on each side:
  vertical between toolbar groups and between status bar fields;
  horizontal between parts of a dialog or panel that have no heading.
- Separators are decorative and do not need to meet the outline contrast
  target; groups must also be recognisable by spacing.

Tier: **Match** (placement), **Own** (solid line). Milestone: now.

### Toolbar grip

Reference: each toolbar starts with a dotted grip line used to drag it
[R3][R12][R16].

LaserCraft: in stage b only (drag-to-redock, owner decision), a short line
of small dots in `text-muted` at the toolbar's start, perpendicular to the
toolbar; hover shows the move cursor and a `hover` background behind the
dots. In stage a no grip is drawn (`layout.md`).

Tier: **Match** (idea), **Own** (look). Milestone: later (stage b).

### More button

Reference: a toolbar that does not fit collapses and shows an arrow; a
click shows the hidden buttons for a moment [R3]; a "»" mark is seen at the
end of the second toolbar row in some captures [R12].

LaserCraft (behaviour in `layout.md`, "Toolbar model"):

- One `icon` square at the clipped end, glyph "»" in `text`, the toolbar
  button states (hover, pressed, focus). Tooltip "More".
- While its popup is open the button shows the Checked look
  (`accent-soft`).
- Popup: the menu look; each row the hidden button's icon, name and chord;
  hidden flyout buttons become submenus.
- The same widget ends a dock tab strip whose tabs do not fit.

Tier: **Echo**. Milestone: with the toolbar band.

### Menus and context menus

Reference: white panels with a thin grey border and compact rows, a narrow
icon column, right-aligned shortcuts, light grey separators, a light-blue
band on the row under the pointer, grey disabled rows, small triangles for
submenus, check marks or empty boxes for toggles, grey non-clickable
captions; macOS shows dark translucent rounded menus [R8][R13][R14][R37];
release 2.1 reserves an icon slot on every row [R32].

LaserCraft (full rules in `menus.md`; master look here):

- Popup: `field` background, *thin* `border`, *slightly rounded* (the one
  popup radius), `shadow`; `gap-tight` padding top and bottom.
- Row: a little less than `row` tall; columns: gutter (`icon-small`, always
  reserved), label `type-body` `text`, chord `type-body` `text-muted`
  right-aligned, submenu triangle in `text-muted` at the far right.
- Row under the pointer or keyboard highlight: `accent-soft` across the
  full width, label stays `text`.
- Disabled row: label, icon and chord `text-disabled`; still highlighted
  under the pointer so its tooltip (with the reason) can be read.
- Toggle rows: a tick in `text` in the gutter when on, empty gutter when
  off. Choice rows (one of several): a filled dot in the gutter.
- Captions: non-clickable rows in `type-small` `text-muted`.
- Separators: *thin* `border` lines inset by `gap-tight` from both sides.
- Destructive rows keep `text` labels; only their pictogram may carry the
  `danger` detail (decision 17). No colored menu text.
- Menu bar: titles in `type-body` `text` on `window`; an open or hovered
  title on `accent-soft`; no access-key underlines unless the owner adopts
  them (`menus.md`).
- Context menus look the same and open at the pointer.

Tier: **Match** (structure and columns), **Echo** (highlight band), **Own**
(look, no translucency, one look everywhere). Milestone: restyle now.

### Tooltips

Reference: resting on a control shows a short description, sometimes with
its shortcut; F1 over a control opens its help page; the delay is only
called brief, and the look is never pictured [R2][R5].

LaserCraft:

- Look (own design): a popup with `field` background, *thin* `border`,
  *slightly rounded*, `shadow`; text `type-body` `text`; `gap-tight`
  padding.
- Content (`naming.md`, Tooltips): first line what the control does,
  followed by the chord in parentheses in `text-muted` with key symbols
  ("Zoom in (Ctrl+=)"); optional second line in `text-muted` with a reason
  (why disabled) or a tip. No icons, no headings, no links.
- Width: wraps at roughly forty-five characters so a tooltip never becomes
  a wide strip; at most three lines.
- Placement: below and to the right of the pointer, shifted to stay inside
  the window; for toolbar buttons, below the button.
- Timing: appears after egui's standard tooltip delay while the pointer
  rests; when moving from one control to the next while a tooltip is shown,
  the next one appears at once (egui behaviour). Disappears on pointer
  move-away, click, key press or scroll.
- Disabled controls show tooltips too.
- Canvas: tooltips appear only where `canvas.md` specifies (markers), never
  over ordinary shapes.
- Keyboard: when a control receives keyboard focus, its tooltip appears
  after the same delay (own design; needs our own code in egui, Open
  questions).

Tier: **Match** (description plus chord on hover), **Own** (look, timing
rules, reason line). Milestone: restyle now; keyboard tooltips later.

### Status bands

Reference: the status strip's message area is a filled box: pale cyan for
hints, pale green for device news; a small tinted box at the far right
lists active special modes [R4].

LaserCraft (the Messages field in `layout.md`; wording in `naming.md`):

| Kind | Band | Text | Mark (`icon-small`, left of the text) | Clears |
|---|---|---|---|---|
| Hint | `info-soft` | `text` | none (hints are not alerts) | when the tool or hint changes |
| Success | `success-soft` | `text` | a tick in `success` | at the next action, or after a short while |
| Warning | `warning-soft` | `text` | a triangle with "!" in `warning` (outlined in `text` in the light theme for contrast) | when the cause goes away |
| Error | `danger-soft` | `text` | a circle with "!" in `danger` | at the next successful action of the same kind |

- A band hugs its text with `gap-tight` padding, *slightly rounded*, one
  line, cut with "…" and the full text in its tooltip.
- The kind is carried by the mark's shape and the lead-in word
  ("Warning: ", "Cannot ‹verb›: ") as well as by the tint.
- Bands appear without animation (Reduce motion or not); a success band
  that clears after a while simply disappears.
- The Active setups field (M9) uses an `info-soft` band without a mark.

Tier: **Echo**. Milestone: Messages field later; preview warnings now.

### Banners

A banner is a larger, multi-line band inside a panel or window, with
buttons. Used for the Alarm banner and the Job refused band (proposed label) in the Machine
panel (`machine-panels.md`), the error and warning bands of dialogs
(`dialogs.md`), the job preview's status lines (`job-preview.md`) and the
Linux serial-permission help (`machine-panels.md`).

- Full width of its panel's content, *slightly rounded*, soft fill of its
  kind (`danger-soft`, `warning-soft`, `info-soft`, `success-soft`), a
  *thin* outline in the same hue's strong token, `gap` padding.
- Left: the kind's mark (`icon-small`). Then up to three lines of
  `type-body` `text`; the lead-in words ("Alarm 2: ", "Cannot run: ") in
  the strong hue only if that meets the text target in the current theme,
  otherwise in `text`. Then a row of ordinary push buttons (Normal style),
  right-aligned; an optional borderless "×" dismisses it where the banner
  allows.
- Raw machine lines (a GRBL reply) inside a banner use `type-mono`.
- Banners never animate, never flash, and stay until their rule clears
  them (safety rule 10: alarms stay until cleared).

Tier: **Own**. Milestone: M6 (alarm), preview bands now (restyle of
"Cannot run" and "Warning:" lines).

### Machine state dot

- A small filled circle (*pill*) before the machine state word in the job
  bar and the Machine panel: `success` for Ready; `accent` for Running,
  Framing, Jogging, Homing; `warning` for Paused, Door open, No response;
  `danger` for Alarm; `text-muted` otherwise (`machine-panels.md`).
- The state word is always shown beside it (the dot never stands alone).
- Steady; no pulsing even while running.

Tier: **Own**. Milestone: M6.

### Progress bar

Reference: a long bar with a pale track and a thin border fills with bright
green from the left, with the percentage as text at the right; the status
line shows elapsed and estimated remaining time [R22].

LaserCraft:

- Track: `field-alt`, *thin* `border`, *pill* ends, about half `row` tall,
  stretching across the free width.
- Fill from the left: `accent` while the job streams; `warning` frozen while
  held (Paused, Door open); `danger` frozen after a fault (Alarm, No
  response, connection lost); `text-muted` frozen after a user Stop.
- Text: "‹n›% sent" in `type-small` `text-muted` at the right of the bar
  (it counts lines sent, not work done, `naming.md`); elapsed and remaining
  time in `type-mono` before the bar (`machine-panels.md`).
- Finished: the bar disappears and a line with a `success` tick and
  "Finished in ‹h:mm:ss›" remains until the next action.
- No stripes, no shimmer, no indeterminate animation: when progress is
  unknown (planning, later) the bar is replaced by the word "Planning…" (proposed label,
  `naming.md`).

Tier: **Echo** (bar with percentage), **Own** (colors by state, "sent").
Milestone: M6.

### Tables and lists

Reference: white list areas; tables with a light grey header row, a sort
caret, and in some places alternating white and light grey rows; the row
under the pointer very pale blue; the chosen row a solid blue band with
white text, or a pale blue fill with a darker outline, light grey when the
list is not focused; the layer list shows no grid lines and comfortable
rows [R7][R8][R19][R25].

LaserCraft:

| Part or state | Look |
|---|---|
| List area | `field` with a *thin* `border`, *square* inside a panel, *slightly rounded* in dialogs |
| Header row | `field-alt` background, labels `type-small` `text-muted`, aligned like their column; *thin* `border` separators between header cells only; a sort marker (small triangle in `text-muted`) only on tables that sort |
| Body row | `row` tall, `type-body` `text`, no vertical grid lines |
| Alternating rows | `field` and `field-alt`, only in tables of more than a few columns where rows are hard to follow (keyboard table, saved positions); off in the layer list |
| Hover | `hover` across the row |
| Selected (chosen) | `accent-soft` across the row, text stays `text`, chips keep their true color; shown the same whether or not the list has keyboard focus (own design: the chosen layer matters while the canvas has focus) |
| Selected + keyboard focus | as Selected plus a *thin* `accent` outline around the row |
| Disabled or inactive row | text `text-muted` (Output off in the layer list, `layers-panel.md`) |
| Empty list | one line in `text-muted` centred in the area, saying what to do ("Draw a shape to start a layer.") |
| Drag to reorder (later) | the dragged row lifts with `shadow`; a *normal* `accent` line shows where it will land |

- Rows are never coloured by content (no red rows for errors); a row that
  needs attention shows a mark in its first cell.

Tier: **Echo**. Milestone: layer list restyle now; other tables with their
windows.

### Swatch chips

Reference: flat rectangles in the layer color with a centred two-digit
number, black or white for contrast; the current color has a frame; one
capture hints at a raised hover look [R19][R20].

LaserCraft (strip specifics in `layers-panel.md`):

| State | Look |
|---|---|
| Normal | flat fill in the layer color, *slightly rounded*, a *thin* `border` outline just outside so pale colors read on any surface; number in `type-small`, black or white by the contrast rule |
| Hover | outline `text`, no raise, no shadow |
| Pressed | `pressed` behind the chip's margin |
| Current color | *heavy* `text` outline with a thin `field` ring between it and the fill |
| Keyboard focus (later) | `accent` focus ring outside the current mark |
| Show off (layer list) | hollow: outline in the layer color, number in `text`, row background inside |
| Output off (layer list) | fill faded to match the faded canvas shapes |
| Number hidden (Display preference, later) | fill only |

- Chips are the same in both themes (fill and number); only the outline
  and rings follow the theme tokens.
- A chip is never tinted for any state.
- In legends (job preview) and small lists, the chip shrinks to
  `icon-small` and drops its number.

Tier: **Echo**. Milestone: numbered swatches now (decision 5).

### Group frames and section headings

Reference: settings pages put a plain caption above a panel with a thin
border; some dialogs use recessed, slightly rounded boxes with a heading
above [R6][R34].

LaserCraft:

- Dialogs (`dialogs.md`): a `type-heading` heading above each named group;
  no frames; groups separated by `gap-wide`.
- Multi-column pages (the Device profile dialog, Materials, later windows):
  where groups sit side by side, each group is a frame with a *thin*
  `border`, *slightly rounded*, on `panel` (never `field-alt`), with its
  `type-heading` heading above the frame.
- Panels: no headings inside dock panels except collapsing sections.
- Collapsing section: a full-width row with a small triangle (right when
  closed, down when open) in `text-muted` and the name in `type-body`
  `text`; hover `hover`. Opening and closing do not animate under Reduce
  motion.

Tier: **Echo** (headed groups), **Own** (no recess, frames only side by
side). Milestone: with each window.

### Scrollbars

- Solid (not floating): a *thin* track in `field-alt` along the area's edge
  and a *pill* handle in `text-muted`, `text` on hover and drag, shown
  whenever the content overflows (own design: always-visible bars make
  hidden content discoverable).
- The canvas has no scrollbars (pan and zoom, `docs/spec/canvas.md`).

Tier: **Own**. Milestone: restyle now.

### Windows and messages

- Dialog and tool window frames, message windows and their marks are
  specified in `dialogs.md`; they use `panel` bodies, the title strip
  above, `shadow`, *slightly rounded* corners and the `backdrop` for
  message windows. The Danger message mark uses `danger`; the Warning mark
  `warning`; the Question and Info marks `accent` (a mark is a pictogram
  detail, within the one-detail rule).

### Drop zone (stage b, later)

Reference: while a panel is dragged over a valid place, the area is
highlighted in blue or light grey [R3].

LaserCraft: the target area is filled with `info-soft` at partial opacity
and outlined with a *normal* dashed `text-muted` line; the zone under the
pointer turns its outline `text`. Not `accent` (reserved), not a solid
fill (the content below stays visible).

Tier: **Own**. Milestone: later (docking crate, decision 8).

### Canvas chips

Small read-outs drawn on the canvas next to the pointer (drag read-out,
guide coordinate, `canvas.md`): `field` background with a *thin* `border`,
*slightly rounded*, `type-small` in `text`, values in `type-mono`; never
colored backgrounds; they follow the pointer without easing.

Tier: **Own**. Milestone: with the canvas features that use them.

## Focus and keyboard indication

- **Who has focus**: exactly one of the canvas, a field or control, or a
  popup. The focused dock group shows the `accent` line on its title strip;
  a focused control shows the `accent` focus ring; the canvas shows no ring
  (its focus is implied when no other control has it, and shortcuts work
  there).
- **Visible only from the keyboard**: focus rings appear after Tab,
  Shift+Tab or arrow navigation, not after mouse clicks (egui behaviour).
- **Tab and the canvas**: `default.keymap` binds Tab and Shift+Tab to
  selecting the next and previous object. When the canvas has focus, Tab
  does that; when a text field has keyboard focus, shortcuts are ignored
  (`keymap.md`) and Tab and Shift+Tab move focus between controls in reading order (top to bottom,
  left to right, label column skipped), as egui does. Esc leaves a field
  and returns focus to the canvas (`property-bar.md`). Extending this to
  any non-canvas control or dialog is a proposed change to `keymap.md`
  (below) and Open question 7.
- **Focus order** in each window follows the visual order; dialogs put
  initial focus on their first field, never on a destructive button.
- **Ring look**: *normal* weight, `accent`, following the widget's
  corners, separated from filled widgets by a `panel` gap; never clipped by
  a scroll area (scroll the widget into view first).
- **Menus and popups**: arrow keys move the `accent-soft` highlight; Enter
  chooses; Esc closes only the popup.
- **Job buttons**: keyboard focus never rests on Start, Pause or Stop by
  default (`layout.md`); they are reached by their chords (Alt+S later,
  Pause key, Ctrl+Break, `machine-panels.md`).

Tier: **Own** (the reference's keyboard focus look is unknown beyond blue
field borders [R9][R24]). Milestone: rings now; full keyboard pass M10.

## Motion

- Default: minimal and short. Allowed motion:
  - the switch knob sliding to its new end;
  - popups and tooltips appearing (no slide, at most a very short fade);
  - the slow drift of the selection dashes on the canvas (`canvas.md`);
  - the progress fill growing as lines are sent;
  - panels shown with F12 appearing at once (no slide-in by default).
- Never: bouncing, pulsing, blinking, attention shakes, animated
  backgrounds, spinners that run while nothing happens.
- **Reduce motion** (Display preference, later, `naming.md`): switches
  jump, popups and collapsing sections appear at once, the selection dashes
  stand still, egui's animation time is zero. Until the preference exists,
  motion follows the defaults above; if egui can report the operating
  system's reduced-motion setting on a platform, LaserCraft follows it
  (Open questions).
- No motion runs for longer than a few seconds unless it shows ongoing work
  (progress) or the user can stop it (Reduce motion stops the dashes;
  WCAG 2.2.2).
- The canvas repaints for motion only while something moves on it
  (`canvas.md`), so an idle window uses no CPU.

Tier: **Own** (the reference's reduce-motion effect is unknown [R6]).

## Accessibility

### Contrast targets (WCAG 2.x level AA)

The theme module's tests check these pairs in both themes; a failing pair
fails `cargo test`.

| Pair | Target |
|---|---|
| `text` on `window`, `panel`, `panel-title`, `field`, `field-alt`, `hover`, `pressed`, `accent-soft` and every `-soft` band | at least 4.5:1 (normal text); aim well above it for `text` on `field` |
| `text-muted` on `window`, `panel`, `field`, `field-alt`, `accent-soft` | at least 4.5:1 (it carries units, chords and hints) |
| `text-on-strong` on `accent` and `danger`; `text-on-warning` on `warning` | at least 4.5:1 |
| colored state words (`danger` "Alarm") on their background | at least 4.5:1, otherwise the word is drawn in `text` and the dot carries the hue |
| `border` against `panel` and `window` (field, button, list outlines) | at least 3:1 (WCAG 1.4.11 non-text contrast) |
| `accent` (focus ring, switch on, chosen dots) against `panel`, `window` and `field` | at least 3:1 |
| switch off track against `panel` and `field` | at least 3:1 |
| `success`, `warning`, `danger` marks and dots against their surfaces | at least 3:1 |
| canvas tokens (`selection`, `marquee` outline, `drawing`, `guide`, `start-marker`, `origin-marker`, `head-marker`, `frame-preview`, `node` outline, `node-selected`, `bed-edge`, `move-unlit`, `move-travel`) against `bed` | at least 3:1 |
| `ruler-text` on `ruler` | at least 4.5:1 |
| swatch numbers on their chips | at least 4.5:1 (guaranteed by the black-or-white rule) |
| `text-disabled` | exempt (WCAG exempts inactive controls), but must stay legible as text |
| `grid-minor`, `grid-major`, separators | exempt (decorative); grids have a Grid strength preference later |

### Color is never the only carrier (WCAG 1.4.1)

- Switch: knob position plus color.
- Chosen radio, point grid dot, check box: filled shape plus color.
- Chosen tab: fill, joining and marker line.
- Selected row: background plus (with focus) outline; the edited layer is
  also named in the layer settings below the list.
- Bands and banners: mark shape and lead-in word plus tint.
- Machine state: word plus dot.
- Marquee enclosing vs touching: solid vs dashed.
- Default vs chosen start point: outline vs filled triangle (`canvas.md`).
- Preview moves: solid (burn), dotted (unlit), dashed (travel).
- Layers: number on the chip plus color; the layer name in tooltips.

### Keyboard reachability (WCAG 2.1.1)

Every action is a command with a menu row (`menus.md`) and often a chord,
so everything a toolbar button, panel button or canvas gesture does can
also be done from the keyboard through the menus. Panels and dialogs are
Tab-navigable; toolbars and the palette strip are not in the Tab order
(their actions are in menus), unless the owner adopts toolbar navigation
(`toolbars.md`).

### Size and scaling (WCAG 1.4.4)

- egui scales with the display's scale factor (including fractional
  scaling on Linux, X11 and Wayland) and with egui's zoom factor. The Text
  size preference (later) sets the zoom factor; every token grows with it,
  and layouts must stay usable up to double the default size (panels
  scroll, toolbars fold into More, `layout.md`).
- egui's own keyboard zoom stays off (`shell.rs:242`), because Ctrl+= and
  Ctrl+- zoom the canvas (`default.keymap`).
- A way back from an unusable size (the reference documents a reset for
  that case [R6][R36]): Window → Reset layout does not reset sizes; instead the
  Preferences window keeps a "Restore default sizes" action (proposed label, `naming.md`) and the
  `lasercraft` command line accepts a flag to start with default display
  preferences (own design, proposed change to `docs/spec/cli.md`).

### Targets and pointer

- Hit areas at least `row` high (see Density rules); adjacent small targets
  (stepper chevrons) are each at least half `row` and never overlap.
- Cursors: egui's standard cursors (arrow, text, resize, move, grab,
  not-allowed, crosshair); tool cursors with badges are `canvas.md`'s and
  `icons.md`'s.

### Screen readers

egui can expose its widgets to assistive technology through AccessKit
(eframe's `accesskit` feature, not enabled today). Every widget in this
catalogue has a text label or tooltip that can serve as its accessible
name. Enabling it is an owner decision with an `ARCHITECTURE.md` line
(Open questions).

### Statement

LaserCraft makes no conformance claim until an accessibility review has
been done (M10). The reference's publisher claims partial WCAG 2.1 AA
conformance without detail [R11]; we aim for the AA targets above.

Tier: **Own**.

## Dark theme (own design, later)

- **Choice**: Theme preference with Light (default), Dark and Follow
  system (`naming.md`, decision 13). Light is the default on every
  platform, whatever the desktop uses (proposed change to
  `docs/spec/ui-shell.md`; today eframe follows the OS). Changes apply at
  once, no restart.
- **What changes**: every chrome, accent and state token switches to its
  dark column above: dark grey `window` and `panel`, a lighter
  `panel-title`, very dark inset `field`s, near-white `text`, a brighter
  `accent`, deep muted soft bands. Icons are drawn from tokens, so their
  strokes turn near-white with `text`.
- **What stays**: the `bed` stays near-white and everything drawn on it
  keeps its light-theme color (canvas tokens), so layer 0 (black) and every
  palette color look the same on the sheet in both themes. Swatch chips,
  their numbers, switch knobs, `handle-fill`, `text-on-strong`,
  `text-on-warning` and the preview's inverted sheet are theme independent.
- **The surround**: `canvas` turns dark grey, so the light bed reads as a
  sheet on a dark table. Canvas overlays that cross the bed edge get a
  *hairline* `handle-fill` halo (see "Canvas" tokens). `ruler` follows the
  chrome.
- **No separate dark canvas switch** (the reference's [R27] is not
  adopted): the bed must stay light.
- **Shadows** get a little stronger; the `backdrop` a little darker.
- **Testing**: every contrast pair above is tested in both themes; manual
  check on Linux with a dark desktop (GNOME, KDE) and a light one.
- **Not before**: the light theme is complete and all colors go through the
  theme module.

Tier: **Own**.

## Platform notes

- One look on Linux, Windows and macOS: egui draws all widgets, menus live
  inside the window (decision 9), the title strip buttons are at the right,
  dialog buttons are in one order (`dialogs.md`).
- The operating system's own title bar of the main window (and, later,
  of separate OS windows) is the only native chrome. LaserCraft does not
  draw a custom main title bar.
- Chords are shown with the platform's modifier names where `naming.md`
  says so (Cmd on macOS), the only platform difference in text.
- Linux: check fractional scaling (for example a scale between one and
  two), X11 and Wayland, and both a light and a dark desktop theme; the
  light theme must not change with the desktop unless Follow system is
  chosen.

## egui mapping

Names refer to egui 0.37 (`Visuals`, `Style`, `Spacing`, `Interaction`,
`Options`); check them on each egui upgrade. No values are written here.

### Where the tokens live

- One module in `lasercraft-ui` (proposed `theme.rs`) holds a `Tokens`
  struct with one field per token and two constructors (light, dark). It is
  the only place in the codebase with color values; today's constants in
  `canvas.rs`, `preview.rs` and `colors.rs` move into it.
- At start-up the app builds egui `Visuals` and `Style` from the tokens and
  installs them for both egui themes (`Context::set_visuals_of`,
  `set_style_of`) and sets the theme preference (`Context::set_theme`) to
  light, dark or system.
- Painting code (canvas, preview, switches, chips, point grid, icons) reads
  the `Tokens` for the current theme; it never uses egui's built-in color
  constants.
- Tests in that module: the contrast table above, the palette distance
  check, and that every token has both theme values.

### Visuals

| egui concept | Token |
|---|---|
| `dark_mode` | which column (light or dark) |
| `panel_fill` | `panel` (side and top panels); the toolbar band and status bar paint `window` themselves |
| `window_fill` | `panel` (dialog bodies) |
| `window_stroke` | `border` |
| `window_shadow`, `popup_shadow` | `shadow` |
| `window_corner_radius`, `menu_corner_radius` | the *slightly rounded* radius |
| `extreme_bg_color` | `field` (text edit backgrounds) |
| `faint_bg_color` | `field-alt` (striped grids) |
| `code_bg_color` | `field-alt` |
| `text_edit_bg_color` (where available) | `field` |
| `selection.bg_fill` | `accent-soft` (selected labels, text selection) |
| `selection.stroke` | `text` for text on selected rows; focus rings are painted from `accent` by our helper where egui would use this stroke |
| `hyperlink_color` | `text` (links, if any, are underlined `text`, never accent) |
| `warn_fg_color` | `warning` (used only for marks; see forbidden uses) |
| `error_fg_color` | `danger` |
| `widgets.noninteractive` | `bg_stroke` `border` (separators), `fg_stroke` `text` (labels) |
| `widgets.inactive` | `bg_fill` and `weak_bg_fill` `field`, `bg_stroke` `border`, `fg_stroke` `text` |
| `widgets.hovered` | `bg_fill` `hover`, `bg_stroke` `text-muted`, `fg_stroke` `text` |
| `widgets.active` | `bg_fill` `pressed`, `bg_stroke` `accent`, `fg_stroke` `text` |
| `widgets.open` | `bg_fill` `accent-soft` (open menu titles, open dropdowns) |
| `widgets.*.corner_radius` | the *slightly rounded* radius |
| `striped` | off by default; tables that alternate turn it on |
| `slider_trailing_fill` | on; the filled part is painted `accent` (by our slider helper if egui's trailing fill follows `selection.bg_fill`) |
| `handle_shape` | circle |
| `text_cursor` | `text` |
| `collapsing_header_frame` | off |
| `interact_cursor` | none (buttons keep the arrow cursor) |
| `button_frame` | on; toolbar buttons are frameless buttons drawn by our toolbar helper |

### Style, spacing and interaction

| egui concept | Token |
|---|---|
| `text_styles`: `Body`, `Button` | `type-body` |
| `text_styles`: `Small` | `type-small` |
| `text_styles`: `Heading` | `type-heading` |
| `text_styles`: `Monospace` | `type-mono` |
| `spacing.interact_size` (height) | `row` |
| `spacing.item_spacing` | `gap` (horizontal and vertical) |
| `spacing.button_padding` | `gap-tight` horizontal |
| `spacing.window_margin`, `menu_margin` | `gap-wide`, `gap-tight` |
| `spacing.icon_width`, `icon_spacing` | `icon-small`, `gap-tight` (check box and radio size) |
| `spacing.scroll` | solid scroll bars (`ScrollStyle::solid` or equivalent) |
| `spacing.slider_width`, `combo_width`, `text_edit_width` | derived from `row` per window (`dialogs.md` width rules) |
| `interaction.tooltip_delay` | egui's default (kept) |
| `interaction.show_tooltips_only_when_still` | on |
| `animation_time` | short; zero under Reduce motion |
| `Options::zoom_factor` | the Text size preference (later) |
| `Options::zoom_with_keyboard` | off (already, `shell.rs:242`) |

### Widgets LaserCraft paints itself

egui has no pill switch, swatch chip, 3×3 point grid, state dot, status
band, banner, title strip, dock tab strip, flyout marker or More button in
this look. Each is a small helper function in `lasercraft-ui` that paints
with `Painter` from the tokens and returns an egui `Response`, so it
behaves like any widget (hover, focus, tooltips). egui's `Checkbox` is not
used for settings; `RadioButton`, `TextEdit`, `DragValue`, `ComboBox`,
`Slider`, menus and tooltips are used with the styled `Visuals` above.

## Current state → target

| Today (current-ui.md) | Target | Where specified |
|---|---|---|
| No `Visuals`, `Style` or fonts set; eframe follows the OS theme, so on a dark desktop the whole UI is egui's default dark (current-ui.md 2) | Tokens installed at start; light theme by default; Theme preference later | this file; proposed change to ui-shell.md |
| egui default fonts and sizes everywhere | same fonts; `type-*` tokens set the sizes and styles; bold face is an open question | this file |
| Canvas surround uses egui's "extreme background", near-black in dark mode (`canvas.rs:229`) | `canvas` token | canvas.md |
| Bed constant near-white (`canvas.rs:29`) | `bed`, unchanged in look | canvas.md |
| Grid lines dark grey at fixed alphas, bed border dark grey (`canvas.rs:30`, `233-234`) | `grid-minor`, `grid-major`, `bed-edge` | canvas.md |
| Machine origin is a red-orange dot (`canvas.rs:34`, `235-241`) | `origin-marker` (dark slate) with axis letters | canvas.md |
| Selection box and handles warm orange (`canvas.rs:32`, `364-378`); handles filled with the extreme background | `selection` box, `handle-fill` handles | canvas.md |
| Nodes and Bézier handles orange, first node solid orange (`canvas.rs:334-348`) | `node`, `node-selected`, `start-marker` | canvas.md |
| Enclosing marquee blue solid, touching marquee green dashed, both with fill (`canvas.rs:31`, `33`, `386-405`) | one `marquee` hue, solid vs dashed | canvas.md |
| Shape being drawn soft green (`canvas.rs:412-434`) | `drawing`, dashed | canvas.md |
| Preview bed outline green (`preview.rs:12`) | `bed-edge` | job-preview.md |
| Preview unlit moves orange dashed, travel red dashed (`preview.rs:13-14`) | `move-unlit` dotted, `move-travel` dashed grey | job-preview.md; proposed change to preview.md |
| Preview head royal blue dot (`preview.rs:15`) | `preview-head` | job-preview.md |
| "Cannot run: …" in deep red text (`preview.rs:16`, `141`), "Warning: …" in weak text | `danger-soft` banner with mark and `text` words; `warning-soft` banner | job-preview.md, naming.md |
| Swatches unnumbered; current outlined in strong text color, others in weak text (`layers_panel.rs:31-61`) | numbered chips with contrast numbers, `border` outline, *heavy* current mark | layers-panel.md |
| Output and Show are unlabeled check boxes; editor booleans and Circular array use check boxes (`layers_panel.rs:118-148`, `390-402`; `dialogs.rs:175`) | pill switches | layers-panel.md, dialogs.md |
| Mode and image method dropdowns show raw ids (`line`, `floyd_steinberg`) | readable names | layers-panel.md, naming.md |
| Tool palette is a list of text `selectable_label`s with egui's selection fill (`shell.rs:537-554`) | toolbar buttons with own icons, active-tool look | toolbars.md, icons.md |
| No icons anywhere | own pictograms per decision 17 | icons.md |
| Menus use egui defaults; chords shown as key names ("Ctrl+Equals") | menu look of this file; chords with symbols | menus.md, naming.md |
| Out-of-range palette fallback mid grey (`colors.rs:12`) | stays an internal fallback, drawn with `text-muted` | this file |
| Layer list grid striped by egui (`layers_panel.rs:86-95`) | layer list without stripes; `accent-soft` edited row | layers-panel.md |
| `egui::Modal` for unsaved changes blocks all input (`main.rs:185-210`) | message window with `backdrop` that leaves the job bar clickable | dialogs.md |
| Hard-coded colors in three files (`canvas.rs`, `preview.rs`, `colors.rs`) | all colors from the theme module; tests check contrast | this file |

## Proposed changes to other specs

- **Proposed change to docs/spec/ui-shell.md**: the app installs
  LaserCraft's light theme at start instead of following the operating
  system's theme; a Theme preference (Light, Dark, Follow system) comes
  later. Add "Theme and look: `docs/spec/style/visual-language.md`" to its
  rules.
- **Proposed change to docs/spec/canvas.md**: replace the fixed colors in
  its text with references to the canvas tokens, and state that on-bed
  colors are theme independent and overlays leaving the bed get a halo in
  the dark theme.
- **Proposed change to docs/spec/preview.md**: unlit moves `move-unlit`
  (dotted grey), travel `move-travel` (dashed grey), the "Cannot run"
  error as a `danger-soft` band with a mark (also proposed by `canvas.md`
  and `job-preview.md`).
- **Proposed change to docs/spec/cli.md**: a start-up flag that ignores
  saved display preferences (default theme and sizes), as the way back from
  an unusable size.
- **Proposed change to dialogs.md**: labels on Primary and Destructive
  buttons use `text-on-strong` (not `field`); the Laser on button's label
  and outline use `text-on-warning` (not `text`); the Message veil uses the
  new `backdrop` token (answers its open question 5).
- **Proposed change to machine-panels.md**: labels and icons on the Stop
  fill use `text-on-strong` (not `field`); the No response state word is
  drawn in `text` with the `warning` dot (amber text misses the text target
  in the light theme); framed groups use `panel`, not `field-alt`, inside
  the frame; no `job-origin-marker` token (answers its question 14).
- **Proposed change to machine-panels.md** (job end and notes): a
  finished job hides the bar and shows a `success` tick with "Finished in
  ‹h:mm:ss›" (instead of a full bar in `success`); "Stopped at ‹n›% sent"
  and "Stopped by alarm at ‹n›% sent" are drawn in `text`, with the frozen
  bar in `text-muted` or `danger` respectively and, for the alarm line, a
  `danger` mark, not `danger`-colored text (this file forbids colored body
  text); the constant-power note row ("`type-small`, `warning`") becomes
  `type-small` `text` on `warning-soft` with a `warning` mark, the same
  change proposed for `naming.md`.
- **Proposed change to docs/spec/keymap.md**: Tab and Shift+Tab move
  keyboard focus between controls while any non-canvas control or dialog
  has keyboard focus (not only a text field); they select the next and
  previous object only while the canvas has focus (Open question 7).
- **Proposed change to layout.md**: the job bar's Stop label uses
  `text-on-strong`; drop zones use the look under "Drop zone" here.
- **Proposed change to job-preview.md**: the inverted view uses
  `preview-sheet-inverted` and `move-burn-inverted` (answers its open
  question 8).
- **Proposed change to layers-panel.md**: the layer list header row uses
  `field-alt` (not `panel`), as every table header; guide swatches use
  `guide-layer-1` and `guide-layer-2` if the owner approves guide layers.
- **Proposed change to canvas.md** (style): the job origin marker keeps
  `origin-marker` (answers its open question 7); the accent canvas family
  is confirmed as listed here (its open question 1, subject to the owner's
  confirmation below); guide layer strokes use the proposed guide layer
  tokens (its open question 11).
- **Proposed change to naming.md**: the constant-power pause note is
  `text` on `warning-soft` with a `warning` mark, not `warning`-colored
  text (severity table). Proposed labels to add to its tables (all Own):
  "Restore default sizes" (Preferences action, later); "Planning…"
  (progress word while the job length is unknown, later); "varies"
  (placeholder in a number field, and the name of the switch and check box
  state, when selected objects differ, later; already listed there, kept
  here so the state name matches `property-bar.md`);
  "Reference point: ‹point›" tooltips on 3×3 point grid dots, such as
  "Reference point: top left" (M6); "Job refused" band (Machine panel,
  M6, already used by `machine-panels.md`).
- **Proposed change to later-windows.md**: framed groups follow "Group
  frames and section headings" (frames only where groups sit side by side).
- **Proposed change to ARCHITECTURE.md** (if the owner approves): a bold
  face of the bundled UI font as an asset with its license; the eframe
  `accesskit` feature with a one-line justification.

## Open questions

1. **Accent list.** Confirm the complete list of `accent` uses under
   "Semantic color rules", which extends decision 3's six uses with
   selected rows, menu highlights, text selection, choice dots, check
   boxes, the slider fill, running progress and state dots, the splitter
   hover line, the highlight ring around Stop during a constant-power (M3)
   job, and the canvas selection family (all read as "focus, chosen
   or active" from decision 6).
2. **Bold face.** Bundle a bold variant of egui's UI font (license to be
   recorded) so `type-heading` and the axis letters are really bold, or
   keep headings larger only?
3. **Default theme.** Light on every platform (this file, decision 13),
   or follow the desktop by default as eframe does today?
4. **Dark theme timing.** Which milestone ships the dark theme and the
   Theme preference (not before all colors go through the theme module)?
5. **Palette issues** (decision 5): layers 1 and 29 are both reds; bright
   yellows are faint on the bed. Change palette data, or keep the darker
   edge proposed in `canvas.md`?
6. **Guide layer colors.** Approve `guide-layer-1` (taupe) and
   `guide-layer-2` (grey-olive) if guide layers are approved.
7. **Tab key.** Tab selects the next object when the canvas has focus and
   moves keyboard focus elsewhere. Is that split acceptable, or should
   object stepping move to another chord so Tab is purely focus navigation?
8. **Toolbar keyboard navigation**: put toolbars and the palette strip in
   the Tab order (`toolbars.md` question), or rely on menus?
9. **Tooltips on keyboard focus**: worth the custom code?
10. **Scrollbars**: solid always-visible bars (this file) or egui's
    floating bars that appear on hover?
11. **Reduced motion from the OS**: if egui does not report the system
    setting on some platform, is the Display preference alone enough?
12. **Screen reader support**: enable eframe's `accesskit` feature (a
    dependency decision with an `ARCHITECTURE.md` line), and when?
13. **High contrast**: offer a later high-contrast variant of the light
    theme (darker `border`, `text-muted` closer to `text`, stronger grid),
    or rely on the AA targets and the Grid strength preference?
14. **Access-key underlines in menus**: adopt egui-drawn underlines
    (`menus.md`) or not?
15. **Warning color in the light theme**: the amber must be deep enough to
    reach the non-text target on `panel` and `bed` while still reading as
    amber, not orange or brown. If no shade satisfies both, prefer contrast
    and add a *thin* `text` outline to warning marks (as this file already
    does for the status band mark).

## Sources

All LightBurn documentation pages below were read on 2026-10-21 through the
research notes for this guide (main window layout, top toolbars, left
toolbars, layers, machine windows, chrome and preferences, canvas, tool
dialogs). Only general visual traits were taken; no values, artwork or
sentences.

- [R1] https://docs.lightburnsoftware.com/2.1/Reference/UI/ — the
  full-window default arrangement, the light overall theme, compact
  density.
- [R2] https://docs.lightburnsoftware.com/2.1/GetStarted/UITour/ — panel
  title strip look, icon style and accents, active-tool background, pill
  switches and their colors, number fields with arrows, dropdowns, push and
  job buttons, radios in the 3×3 grid, right-aligned labels, bottom tabs,
  typography, disabled grey, tooltips with shortcuts, the multi-tool corner
  marker.
- [R3] https://docs.lightburnsoftware.com/2.1/Guides/CustomizingTheLightBurnWindow/
  — dock title strips and buttons, splitter dots, toolbar grips, drop
  highlights, overflow arrow, separators.
- [R4] https://docs.lightburnsoftware.com/2.1/Reference/UI/StatusBar/ —
  borderless status strip, pale cyan hint band, pale green device band,
  tinted mode box.
- [R5] https://docs.lightburnsoftware.com/2.1/Reference/UI/Tooltips/ —
  tooltip content and context help; look and delay not shown.
- [R6] https://docs.lightburnsoftware.com/2.1/Reference/SettingsPreferences/
  — switches, radios, number fields, dropdowns, sliders, plain captions
  over framed panels, tabs with large pictograms, default button outline,
  application theme choices and restart, font size and icon size sliders,
  link to the reset path when sizes make the UI unusable,
  cursor and node size, reduce motion, dark background option, swatch
  labels, grid strength levels, native dialog look, bold only in rich text.
- [R7] https://docs.lightburnsoftware.com/2.1/Reference/EditHotkeys/ —
  table with grey header and sort caret, alternating rows, focused search
  field, unfocused selection grey.
- [R8] https://docs.lightburnsoftware.com/2.1/Reference/ManagingPreferences/
  — menu look with icon column and separators; solid blue selected list
  row with white text.
- [R9] https://docs.lightburnsoftware.com/2.1/Reference/ShowNotes/ — blue
  focus border on a text box; switch off look.
- [R10] https://docs.lightburnsoftware.com/2.1/Reference/HelpAndNotes/ —
  text-only dialog tabs, chosen tab lighter and joined.
- [R11] https://docs.lightburnsoftware.com/2.1/Licensing/AccessibilityStatement/
  — partial WCAG 2.1 AA claim without details.
- [R12] https://docs.lightburnsoftware.com/2.1/Reference/UI/MainToolbar/ —
  flat frameless icon-only toolbar buttons, bold black glyphs, pale
  disabled glyphs, faint dotted group separators, the overflow mark.
- [R13] https://docs.lightburnsoftware.com/2.1/Reference/UI/WindowMenu/ —
  menu check marks and empty boxes, grey captions, light-blue hover band.
- [R14] https://docs.lightburnsoftware.com/2.1/Reference/UI/ArrangeMenu/ —
  submenu triangles, grey disabled rows, right-aligned shortcuts.
- [R15] https://docs.lightburnsoftware.com/2.1/Reference/Distribute/ —
  a floating toolbar with rounded ends and a soft shadow.
- [R16] https://docs.lightburnsoftware.com/2.1/Reference/UI/CreationToolbar/
  — tool column, active-tool background in two platform looks, corner
  marker, grips.
- [R17] https://docs.lightburnsoftware.com/2.1/Reference/EditNodes/ — the
  node tool's icon turning red while active.
- [R18] https://docs.lightburnsoftware.com/2.1/Reference/BooleanTools/ —
  the flyout popup with rounded corners and icon-plus-name rows.
- [R19] https://docs.lightburnsoftware.com/2.1/Reference/CutsLayersWindow/
  — gradient title strip with float and close, header row, row hover and
  selection looks, tinted chip, switches in rows, small icon buttons.
- [R20] https://docs.lightburnsoftware.com/2.1/Reference/UI/ColorPalette/ —
  numbered flat swatches, black or white numbers for contrast, current
  swatch frame, possible hover raise.
- [R21] https://docs.lightburnsoftware.com/2.1/Reference/CutSettingsEditor/LineMode/
  — units in parentheses in labels, groups greyed together, thin rules
  between groups.
- [R22] https://docs.lightburnsoftware.com/2.1/Reference/LaserWindow/ —
  status line, green progress fill with percentage, big job buttons about
  twice ordinary height with glyphs, red-centred crosshair icon, disabled
  grey buttons, recessed 3×3 grid, frame outline on the canvas.
- [R23] https://docs.lightburnsoftware.com/2.1/Reference/MoveWindow/ —
  small square icon-only jog buttons, the three button size tiers.
- [R24] https://docs.lightburnsoftware.com/2.1/Reference/ConsoleWindow/ —
  log in the proportional font, error lines in dark red, focused input
  border.
- [R25] https://docs.lightburnsoftware.com/2.1/Reference/Devices/ — solid
  blue selected list rows with white text; default button outline.
- [R26] https://docs.lightburnsoftware.com/2.1/Reference/CoordinatesOrigin/
  — 3×3 grid disabled look and chosen dot; red origin square and axis
  letters; green job-origin square.
- [R27] https://docs.lightburnsoftware.com/2.1/Reference/UI/Workspace/ —
  flat light canvas, dark background option that leaves the chrome light.
- [R28] https://docs.lightburnsoftware.com/2.1/Reference/Selection/ — red
  enclosing and green crossing selection boxes.
- [R29] https://docs.lightburnsoftware.com/2.1/Reference/GridArray/ —
  dialog form layout, switches, equal-width buttons at the bottom right,
  focus ring on fields, red error text.
- [R30] https://docs.lightburnsoftware.com/2.1/Reference/BarCode/ —
  sliders with value text, tabs on a framed page, dropdown list highlight,
  red over-limit text.
- [R31] https://docs.lightburnsoftware.com/2.1/Reference/QuickNest/ —
  error message boxes with a red stop sign.
- [R32] https://docs.lightburnsoftware.com/2.1/NewFeatures/NewIn2.1/ —
  blank menu icons for alignment, wider multi-tool buttons, the dark theme
  mentioned without a capture.
- [R33] https://docs.lightburnsoftware.com/2.1/Reference/BeginnerMode/ —
  disabled switch look, bold dialog headings.
- [R34] https://docs.lightburnsoftware.com/2.1/Reference/OffsetShapes/ —
  recessed rounded group boxes with plain headings.
- [R35] https://docs.lightburnsoftware.com/2.1/Reference/Dock/ — pill
  switches inside a toolbar, popup menu look.
- [R36] https://docs.lightburnsoftware.com/2.1/Troubleshooting/ResetToDefaultSettings/
  — a reset path when large font or icon settings make the interface
  unusable.
- [R37] https://docs.lightburnsoftware.com/2.1/Reference/ViewStyle/ — a
  macOS capture of a menu drawn dark, translucent and rounded.

Existing specs used: `docs/clean-room.md` (look and feel rules),
`docs/spec/safety.md` (rules 7, 9, 10), `docs/spec/ui-shell.md`,
`docs/spec/canvas.md` (bed stays light in every theme),
`docs/spec/keymap.md` and `crates/lasercraft-ui/keymap/default.keymap`
(Tab, Shift+Tab, Esc, F12, Ctrl+=, Ctrl+-, Alt+S, Pause, Ctrl+Break,
F1; the M6 and M10 rows of `keymap.md`), `docs/spec/layers.md`
(palette is data), `docs/spec/preview.md`, and the other files of
`docs/spec/style/` (tokens and states they already use).

Open standards: WCAG 2.1 and 2.2 (W3C), success criteria 1.4.1 (use of
color), 1.4.3 (contrast minimum), 1.4.4 (resize text), 1.4.11 (non-text
contrast), 2.1.1 (keyboard), 2.2.2 (pause, stop, hide), 2.3.1 (three
flashes), 2.4.7 (focus visible), 2.5.8 (target size minimum), and WCAG's
definitions of relative luminance and contrast ratio. egui and eframe
0.37 documentation for the names in "egui mapping".

Own design: every token and its plain-word color in both themes, the new
tokens (`text-on-strong`, `text-on-warning`, `shadow`, `backdrop`,
`preview-sheet-inverted`, `move-burn-inverted`, proposed guide layer
colors), the semantic hue table and forbidden uses, the accent list, all
widget looks and states (flat title strip, blue and grey pill switch,
check box policy, point grid outline, banners, state dots, progress colors
by state, chips never tinted, solid scrollbars, drop zones, canvas chips),
type tokens and the bold policy, line weight and corner words, density
rules, focus rules, motion limits, contrast targets and tests, the dark
theme, and the egui mapping.
