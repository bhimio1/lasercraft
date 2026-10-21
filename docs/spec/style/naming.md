# Spec: style — names and wording

Milestones: M0–M5 (renames and sentence case of labels that exist today), M6
(machine, jog, console and job wording), M7 (image wording), M8 (materials,
tests, device profiles), M9 (camera, rotary, print registration), M10 (help,
context help) and later items as marked in each row.

This file fixes every visible name in LaserCraft's interface (menu titles and
items, toolbars, panels, dialogs, buttons, switches, fields, status fields and
canvas features) and the rules for writing any further UI text: tooltips,
messages, confirmations, empty states and numbers. The other files in
`docs/spec/style/` use these names and do not define their own: README.md
(tiers, scope and the binding style decisions), layout.md (where each region
sits), menus.md (menu order and grouping), toolbars.md, property-bar.md,
layers-panel.md, machine-panels.md, canvas.md, dialogs.md (dialog structure and
button placement), job-preview.md, later-windows.md, visual-language.md (color,
type and size tokens) and icons.md. Where another style file needs a label, it
quotes it from here (labels first proposed by another file are reviewed in
"Labels adopted from the other style files"); where this file needs a look, it names a token from
visual-language.md or points to the file that owns that element.

## Why doc terms appear here, and only here

`docs/clean-room.md` allows LightBurn to be named in our docs to say where
public information came from and which workflow we want to make familiar; it
forbids copying the reference's feature and product names or UI text into the
app. To show provenance, each table below has a "Public docs term" column: the
name the reference's public documentation uses for the same thing. That column
is a record of where the idea came from, so reviewers can check that our label
is not a copy. It is never a source of wording, it never appears in the app,
and no other style file repeats it. Doc terms also appear in the Omitted list,
in the Naming decisions (where a rejected draft has to be compared with the
doc caption) and in Sources. Everywhere else in the style guide, and in every
label, tooltip and message, only the "LaserCraft name" column is used.

## How to read the tables

- **LaserCraft name** is the exact label drawn in the UI, already in sentence
  case. "‹…›" marks a value filled in at run time. Rows marked "(no visible
  label)" name a thing for specs, the user guide and tooltips only.
- **Kind** says what sort of element it is (menu item, switch, field, …).
- **What it is** is our own one-line description; tooltips may build on it.
- **Public docs term** is provenance only (see above).
- **Tier** is this file's naming tier (defined in the Naming rules below);
  README.md (tiers) defines the look tiers used by the other style files.
- **Notes** give the reason for the name, today's label when it changes, the
  milestone, and any spec the name affects.

All facts about the reference in the mapping column come from the style
guide's research notes, which were taken from the public LightBurn 2.1
documentation pages listed in Sources. Nothing about the reference is added
from memory.

## Naming rules

- **Our own words.** Every visible label, tooltip and message is LaserCraft's own design
  (README.md, tiers: Own). A doc label that is a coinage, a compound feature or product name, a
  slash-joined or parenthesised panel or toolbar name, a branded mode or wizard name, or a vendor
  name never becomes a LaserCraft label. A label made by swapping, dropping or reordering one word
  of a distinctive multi-word doc caption counts as a copy.
- **Plain words are fine.** When a single plain generic word, or a stock phrase used across design
  and CNC software ("Zoom in", "Select all", "Bring to front", "Continuous jog", "Save to file…"),
  is simply the natural English for the thing, we use it even if the docs use it too: Layers,
  Console, Preview, Undo, Align, Offset, Weld, Home, Frame, Start, Stop, Pause, Grid, Rulers.
  Such words are not ownable, and avoiding them would hurt users.
- **Findable.** Choose the word a laser user would type into a search or look for in a menu. Keep
  the meaning, the place and the shortcut; change only the wording. When our name differs a lot from
  what switchers expect, the tooltip says what the thing does in plain words and may name a generic
  search term ("continuous jog", "print-and-cut"); it never quotes doc text.
- **Our model's vocabulary.**
  - Each palette color is a *layer* (never "cut" or "color" for the layer itself).
  - Drawn things are *shapes*. An *object* is a shape or a whole top-level group, where a group
    counts as one (arrange.md; visible in "select exactly two objects" and keymap.md's "Select the
    next / previous object").
  - The *job* is what is sent; the *work area* is the setting and the *bed* the drawn sheet; the
    *head* is the moving laser head.
  - The three origins are the *machine origin* (the bed corner where GRBL's zero is), the *user
    origin* (a head position the user stored) and the *job origin* (the point on the bed where the
    job's reference point lands).
  - A *reference point* is the chosen point of a bounding box (the 3×3 grids).
  - The *anchor* is arrange.md's anchor object: the object that align, subtract and pack keep fixed.
  - An *operation* is one settings set inside a layer (later, owner decision).
  - *G-code* is always spelled that way; a saved machine is a *device profile*.
- **One meaning per word.**
  - "Frame" only means tracing the job with the head (zooming says "Zoom to selection").
  - "Dock" only means panel docking (pushing shapes until they touch is "Slide").
  - "Palette" only means the color strip (the left tools are "Drawing tools").
  - "Console" only means the GRBL console (LaserCraft commands are typed in the "Command line").
  - "Cut" never names a layer or a shape-splitting tool ("Split by shape").
  - "Outline" only means the tight convex outline around shapes (Frame outline, Outline around
    selection). The outlines-only view is "Show fills" switched off.
  - "Lock" only applies to shapes; GRBL's alarm lock is cleared with "Clear alarm".
  - "Step" only means motor steps (steps per mm). Jog moves have a *distance*, a layer has
    *operations*, history lists *actions*.
  - "Mode" only means a layer mode (Line, Fill, Image, Mixed). Job placement is "Place job at";
    rotary and registration are *setups*; the reduced interface is a *layout*.
  - "Gap" only means spacing between shapes or copies (Column gap, Gap (mm)).
  - "Camera" is the panel; the dialog that lists cameras is "Camera list".
- **Screen words and machine words.** Top, bottom, left and right describe the design and the
  canvas (Move on bed, align, sweep direction); the top of the screen is the rear of the bed
  (canvas.md). Front, rear, left and right describe the machine (origin corner, status bar
  "origin front-left").
- **Sentence case** for menu items, buttons, panel and dialog titles, switch labels and tooltips.
  Proper names and standards keep their case: LaserCraft, GRBL, G-code, SVG, DXF, CSV, SHX, Wi-Fi,
  Floyd–Steinberg. Menu bar titles are single capitalised words. Today's Title Case labels (Save As…,
  Paste in Place, Fit Bed, Edit Nodes, Side Panels, Job Preview…) become sentence case. That is a
  style fix, not a rename, and is not repeated row by row; it is a proposed change to ui-shell.md,
  preview.md, canvas.md, nodes.md and text.md, which spell those labels in Title Case.
- **Ellipsis** (the single character "…") ends any menu item or button that opens a dialog or a
  separate window (Offset…, Save as…, Job preview…). Items that act at once, toggle something, or
  show/hide a docked panel get none. Dialog and window titles never end in an ellipsis.
- **Parts of speech.** Buttons and menu items that act are verbs or verb phrases (Start,
  Save G-code…, Set user origin). A button or item that only opens a named dialog may carry that
  dialog's name plus "…" (Cut order…, Device profiles…). A dialog or window title repeats the command
  that opens it, without the ellipsis (Add text, Calibrate steps per mm), or is a noun when no single
  command opens it (Preferences). Panels and toolbars are nouns (Layers, Jog). Switches name what is
  on when they are on (Show travel, Selected shapes only). Dropdown labels are nouns or a short
  lead-in (Port, Place job at).
- **Units** go in the label in parentheses: (mm), (mm/s), (mm/s²), (%), (°), (s), (ms), (min),
  (px), (lpi), (dpi). Fields carry no unit suffix. Millimetres and mm/s by default (README.md, units);
  inches are a later preference. Screen distances are in px (canvas.md).
- **Chords in menus and tooltips** are shown with key symbols, not key names: Ctrl+=, Ctrl+-,
  Ctrl+`, `.`, `,`, PgUp, PgDn, Del, Esc; on macOS Cmd and Option replace Ctrl and Alt. Today the
  code prints "Ctrl+Equals", "Ctrl+Backtick", "Period", "PageUp" and "Ctrl+" on macOS
  (current-ui.md 1.4); proposed change to keymap.md (chord display).
- **Toolbars** appear under Window → Toolbars by their short name ("Standard", "Position and
  size"); prose says "the Standard toolbar". Panels appear under Window → Panels by their title;
  the entry's tooltip says what the panel does in plain words.
- **No brand or vendor names** in any label, including third-party program names in options (an
  SVG resolution option names the number, not the program). No doc-style maturity markers; an
  experimental feature says "(experimental)".
- **No internal ids** as labels: never a command id ("Undo shape.move") or a raw setting value
  ("floyd_steinberg", "line"). Menus and lists show readable names.
- **Safety wording is fixed** by `docs/spec/safety.md`: for a constant-power (M3) job Pause reads
  "Pause (motion only)" with the note "The laser may stay on", and Stop is the highlighted action.
  Laser-off framing labels never suggest firing; the opt-in low-power frame always says the laser
  is on and shows its power.
- **Prose names.** Rows marked "(no visible label)" are names for specs, the user guide and
  tooltips only. Nobody draws them as captions.
- **Doc terms** appear only in this file's "Public docs term" column, in the Omitted list, in
  the Naming decisions and in Sources. That column records provenance; it is never a source of wording.
- **Tier column** (this file's meaning, for reviewers): *Match* = our label uses the same words as
  the doc label, allowed only because they are plain generic words or stock phrases; *Echo* = our
  label keeps a doc word or the doc's structure but is reworded; *Own* = no doc word beyond plain
  function words. In README.md (tiers) terms every label is Own design either way.
- **Renames touch tests and specs.** Renaming a label a test looks for ("Job preview", "Cannot
  run:", "Warning:") updates `crates/lasercraft-ui/tests/shell.rs` in the same change. Renaming
  something an existing spec names is written up as "proposed change to <spec>" (README.md, existing specs win).
- **Proposed spec changes in this file** (README.md, existing specs win; each spec is updated
  before code; summarised again under "Proposed changes to other specs"):
  - ui-shell.md: "tool palette" → Drawing tools; the command "Console" → Command line; sentence
    case; Unsaved changes gains a Save button (owner decision).
  - keymap.md (action descriptions only, chords unchanged): Alt+Shift+C "cut shapes" → split by
    shape; Alt+Shift+O "optimize" → simplify; Ctrl+B "boolean helper" → combine shapes; Ctrl+Shift+B
    "convert to bitmap" → convert to image; Alt+Shift+W "toggle outline / filled display" → show or
    hide fills; Alt+Shift+H/V "move together" → pack horizontally / vertically; Alt+0 … Alt+4
    "print-and-cut steps" → print registration steps (CLAUDE.md M9 also says "print-and-cut"; the
    owner may align that wording); keymap.md:121 "the console" → the command line; Alt+Shift+S and
    Ctrl+Shift+F are not bound (see Omitted); chord display with symbols.
  - import.md:29 "the console" → the command line.
  - text.md: editable text objects (Text style toolbar beyond next-text defaults, Text data,
    text properties, Arc handle, Put text on path, Double-click edits text). Owner decision.
  - arrange.md: sliding (object docking) and linked copies (virtual arrays), both excluded by design
    in its Sources today.
  - layers.md: Concentric fill mode, Operations and the Mixed mode, guide layers, extra tab settings.
  - shapes.md: polygon, star and alternating-star shapes, rectangle corner radius, barcodes.
  - planning.md: the Cut order dialog's switches beyond today's fixed order.
  - gcode.md: "After the job" (stay, go to origin, go to end position) replaces the on/off finish
    option; G-code tab switches that change output.
  - import.md: DXF units, SVG resolution and tolerances become preferences (today fixed).
- **Milestone tags** in Notes: *Now* = exists today; *M6*–*M10* as in CLAUDE.md; *Later* = no
  milestone yet; *owner decision* = needs the owner before it is built.


## Regions, panels and toolbars

| LaserCraft name (exact UI label) | Kind | What it is (our words, one line) | Public docs term (provenance only) | Tier (Match/Echo/Own) | Notes (why this name; current label if changed; milestone if future) |
|---|---|---|---|---|---|
| Main window (no visible label) | window | The single top-level window: menu bar, toolbar band, tool column, canvas, right dock, palette, status bar | Main Window (default layout) | Match | Arrangement only; egui draws it on every OS (README.md, platform). Now (partly). |
| • ‹project› — ‹device profile› — LaserCraft | window title | OS title text: a dot when unsaved, the project name, the active device profile, the app name | title bar / Window title | Own | Keeps today's form ("• ‹name› — LaserCraft"; README.md, window title); device profile added with M6/M8; "(simple layout)" suffix later. |
| Menu bar (no visible label) | menu bar | Text menus under the title, drawn inside the window on all platforms, macOS included | Menus / Menu Bar / Menu Toolbar | Match | README.md (menus). Now. |
| Toolbar band (no visible label) | region | Two full-width toolbar rows under the menu bar | (top toolbar rows) | Own | Later; needs a ui-shell.md section first. |
| Standard | toolbar | Row 1 left: new, open, save, import; undo, redo; cut, copy, paste, delete; pan, fit bed, zoom in, zoom out, zoom to selection, capture camera image; job preview; preferences, device profile | Main Toolbar ("Main") | Own | Stock software name for the everyday toolbar. Later. |
| Arrange | toolbar | Row 1, after Standard (the Job bar sits at the row's right end while connected): group, ungroup, mirror, align, distribute, match width and height, move on bed, mostly as flyouts | Arrange Toolbar ("Arrange") | Match | Same word as our Arrange menu. Later. |
| Align and distribute | toolbar (variant) | Wide variant with one button per align, match-size, distribute and move-on-bed action | Arrange (long) | Own | Off by default; listed under Window → Toolbars by this name. Draft: "Arrange (all buttons)". Later. |
| Position and size | toolbar | Row 2 left: X, Y, width, height, scale, keep-proportions lock, reference point grid, rotation, units | Numeric Edits Toolbar | Own | Users look for "position" and "size". Later (no numeric fields today). |
| Text style | toolbar | Row 2: font and capital height for the next text; style switches and spacing for selected text once text stays editable | Text Options Toolbar | Own | Draft: "Text". Not a bare "Text", which is the tool's name. Font and height for the next text: Later (they are Add text dialog defaults today). Everything for selected text: Later; proposed change to text.md (editable text objects, owner decision). |
| Job setups | toolbar | Row 2: large buttons that open the rotary and print-registration setups | Modes Toolbar | Own | M9. The status field says "Active setups" (same word). |
| Slide and nest | toolbar | Row 2 far right, off by default: slide left, right, up, down; Nest; the slide-and-nest options | Docking Toolbar | Own | Off by default (Window → Toolbars turns it on). Avoids "dock", which means panel docking here. Later; proposed change to arrange.md (sliding is currently excluded by design). |
| Tool column (no visible label) | region | Narrow column at the left edge holding Drawing tools above Shape operations | left-side toolbar | Own | Now (the `tools` side panel). |
| Drawing tools | toolbar | Select, line, shape tools, edit nodes, trim, place tabs, text, move head, measure | Creation Toolbar ("Creation Tools" / "Tools") | Own | Current spec name "tool palette" (no visible label); renamed because "palette" means the color strip. Proposed change to ui-shell.md. Now (text buttons); icons later. |
| Node tools | toolbar (contextual) | Narrow strip beside Drawing tools while Edit nodes is active, one button per node action | Edit Nodes Toolbar | Echo | Buttons repeat the D, I, M, L, C, B node keys (nodes.md) plus later actions; see Node tool buttons. Later. |
| Shape operations | toolbar | Lower part of the tool column: offset, weld, booleans, grid and circular array, start point, round corners | Modifiers Toolbar ("Modifiers") | Own | Later. |
| Canvas (no visible label) | region | Central drawing area with the bed, grid and shapes | Workspace / Edit Window | Own | Current spec name (canvas.md). Now. |
| Right dock (no visible label) | region | Right-hand column holding tabbed panel groups (Layers group above, Machine group below) | (right dock column) | Own | README.md (docking), staged. Now (single Layers panel). |
| Drop zone (no visible label) | dock feedback | Highlighted area that shows where a dragged panel or toolbar will dock | drop target highlight (Docking and Undocking) | Own | README.md (docking) stage b (docking crate, owner decision). Look set in visual-language.md (accent blue is reserved for its listed uses, README.md accent). Later. |
| Layers | panel | Layers in output order with mode, speed and power, Output/Show/Air switches, buttons and the layer editor | Cuts / Layers Window | Match | Current heading "Layers". Now. |
| Machine | panel | Connection, machine state, job buttons, framing, home, job placement and device row | Laser (window) | Own | Same word as the Machine menu and keymap.md's "Machine" table. Tooltip and Window → Panels tooltip: "Laser connection and job control", so a search for "laser" finds it. M6. |
| Jog | panel | Live head position, go-to fields, saved positions, jog pad, jog distance and speed, user origin and end position buttons | Move (window) | Own | "Jog" is the plain CNC word users search for. M6. |
| Console | panel | GRBL message log with a line for sending raw commands to the controller | Console (window) | Match | M6. Takes the word over from today's command console (next row). |
| Command line | panel | Line for typing any LaserCraft command, with its log | (no doc equivalent) | Own | Current: bottom panel called "Console" in ui-shell.md, keymap.md:121 and import.md:29 (no visible title). Renamed so "Console" means the controller console switchers expect. Proposed change to ui-shell.md, keymap.md, import.md. Now. |
| Macros | panel | One button per stored set of G-code lines, plus Edit macros… | Macros (window) | Match | Later. |
| Properties | panel | Settings of the one selected shape: run order, power (% of layer), lock, then fields for its kind | Shape Properties (window) | Echo | Later. Text rows: Later; proposed change to text.md. |
| Text data | panel | Counter, date and CSV controls for text that changes from job to job | Variable Text (window) | Own | Draft: "Text variables" (the doc name with its words swapped). Tooltip and Window → Panels tooltip: "Counters, dates and CSV values for text that changes per job". Later; proposed change to text.md (editable text objects, owner decision). |
| Materials | panel | Browse our material library and apply a stored setting to a layer | Material Library (Library Window) | Echo | M8 (our format, our data). |
| Saved artwork | panel | Thumbnails of artwork the user stored for reuse | Art Library | Own | Later; not in the milestone list. |
| Camera | panel | Camera overlay controls and one status block per camera | Cameras (tab) / Camera Control | Match | M9. The dialog that lists cameras is "Camera list". |
| Palette | toolbar (strip) | Bottom strip of 30 numbered layer swatches plus two guide swatches; the current one outlined | Color Palette | Echo | Prose keeps today's "palette strip"; Window-menu label "Palette". Now (guide swatches later). |
| Status bar | region | Thin bottom strip of read-outs, hints and messages | Status Bar | Match | Now. |
| Job bar | region | Always-visible compact strip while connected: machine state, Pause/Resume, Stop | (no doc equivalent) | Own | README.md (safety): never hidden by F12, tabs or dialogs; hidden when disconnected. M6. |
| Title strip (no visible label) | panel chrome | Flat strip on each docked panel with its name and float/close buttons | window top bar (dock title strip) | Own | Look per README.md (title strips) and visual-language.md. Later. |
| Tab strip (no visible label) | panel chrome | Text tabs under a dock group that bring one panel forward | labeled tabs beneath windows | Match | README.md (docking) stage a. Later. |
| Grip (no visible label) | toolbar chrome | Dotted handle at a toolbar's start for dragging it | dotted line (toolbar handle) | Own | Needs drag-to-redock (README.md docking, stage b). Later. |
| More | toolbar chrome | "»" button at the end of a clipped toolbar that pops out the hidden buttons | arrow symbol (collapsed toolbar) | Own | The visible text is the tooltip "More". Later. |
| Flyout button (no visible label) | button variant | Toolbar button holding related tools behind a small corner marker | stacked buttons | Own | Echo look per README.md (tiers). Later. |
| Floating toolbar, floating panel (no visible label) | state | A toolbar or panel dragged out of its dock to float over the canvas | Floating toolbar / Docking and Undocking | Match | Stage b needs a docking crate (owner decision, ARCHITECTURE.md line). Later. |

## Menus (menu bar titles and every top-level item group)

| LaserCraft name (exact UI label) | Kind | What it is (our words, one line) | Public docs term (provenance only) | Tier (Match/Echo/Own) | Notes (why this name; current label if changed; milestone if future) |
|---|---|---|---|---|---|
| File | menu title | Projects, import and export, G-code file, notes, quit | File Menu | Match | Now. |
| Edit | menu title | Undo and history, clipboard, path clean-up, selection helpers, image commands, preferences | Edit Menu | Match | Now. |
| Tools | menu title | Canvas tools first, then shape and image operations | Tools Menu | Match | Now. |
| Arrange | menu title | Grouping, mirror and rotate, align and distribute, slide and move, arrays, outline, order, lock | Arrange Menu | Match | Now. |
| Machine | menu title | Connection, job control, head moves, device profiles, tests and accessories | Laser Tools Menu | Own | README.md (menus) asks for our own name. M6. |
| View | menu title | Job preview, zoom, fills and curve display, snapping, side panels | (no doc menu; these items sit in the Window menu) | Own | Kept per README.md (menus); a deliberate difference. Now. |
| Window | menu title | Reset layout, check lists of toolbars and panels | Window Menu | Match | Later. |
| Help | menu title | About, quick reference, user guide, reporting a problem | Help Menu | Match | M10 (About may come earlier). |
| Language | menu title | Interface language choice | Language Menu | Match | Later; not planned while there are no translations (README.md, menus). |
| New, Open…, Recent projects ▸ | File group | Start, open and reopen projects | New / Open / Recent Projects ▸ | Match | New, Open… now; Recent projects later. |
| Save as template…, Recent templates ▸ | File group | Save and reuse starting projects that keep layer settings | Save As Template / Recent Templates ▸ | Match | Same plain words as the docs: "save as template" and "recent templates" are stock phrases in office and design software, not a coinage. Later. |
| Import…, Export vector art… | File group | Bring in SVG/DXF artwork; write shapes out as vector art | Import / Export | Echo | Import… now (its window is titled "Import vector art", so export says "vector art" too). Draft: "Export vectors…". Export later (Alt+X, keymap.md). |
| Save, Save as… | File group | Save the project (.lcproj) | Save / Save As | Match | Now. |
| Save G-code… | File item | Write the job to a GRBL G-code file | Save GCode (label varies by device) | Echo | Now; also a Machine panel button in M6. |
| Export image as engraved…, Save camera image… | File group | Save an image with its dithering applied; save the camera picture | Save Processed Bitmap / Save Background Capture | Own | Draft: "Export processed image…" (kept the doc's structure). M7 / M9. |
| Project notes… | File item | Notes stored with the project | Show Notes / Project Notes | Match | Now (current label). |
| New window | File item | Open a second LaserCraft window | New Window | Match | Later, optional (multi-viewport). |
| Quit | File item | Close LaserCraft, asking about unsaved changes first | Exit | Own | Now (current label). |
| Undo ‹action›, Redo, History… | Edit group | Step back or forward; list of past actions | Undo / Redo / Undo History | Echo | Current label shows the raw command id ("Undo shape.move"); change to readable words ("Undo move"). History… later. |
| Cut, Copy, Duplicate, Paste, Paste in place, Delete, Select all, Invert selection | Edit group | Clipboard and basic selection | Cut / Copy / Duplicate / Paste / Paste in Place / Delete / Select All / Invert Selection | Match | Now. |
| Convert to path, Close path, Delete duplicates | Edit group | Path clean-up with plain names | Convert to Path / Close Path / Delete Duplicates | Match | Convert to path and Close path exist in Tools today (the menus file decides placement); Delete duplicates later (Alt+D). |
| Close nearly closed paths…, Join paths, Simplify…, Reverse direction | Edit group | Close paths whose ends almost meet (asks for the gap); join touching ends; reduce nodes; flip path direction | Close Selected Paths With Tolerance / Auto-Join Selected Shapes / Optimize Selected Shapes / Reverse Shape Direction | Own | Draft: "Close paths within…". Join paths exists in Tools today (Alt+J); the rest later (Alt+Shift+O). keymap.md says "optimize": proposed change to keymap.md. |
| Select open shapes, Select circles… | Edit group | Select every open path; select circles of one size or a range | Select Open Shapes / Select Circles | Match | "Select open shapes" is a stock phrase in vector and CAD programs (select + the generic term "open shape"), not a coinage. Later. |
| Select open shapes on fill layers, Select the current layer's shapes, Highlight the current layer's shapes, Add enclosed shapes to selection, Add smaller shapes to selection | Edit group | Selection helpers that name their reference (which layer, inside what, smaller than what) | … Set to Fill / Select All Shapes in Current Layer / Flash Shapes in Current Layer / Select Contained Shapes / Select Shapes Smaller Than Selected | Own | Draft: "Select layer shapes", "Highlight layer shapes", "Select shapes inside", "Select smaller shapes". The last two add to the selection, so the labels say "Add". Later (Shift+click on a layer name already selects its shapes). |
| Image ▸ (Reload from file, Replace…, Replace and fit…) | Edit submenu | Swap or refresh a single placed image | Image options ▸ (Refresh Image / Replace Image / Replace Image to Fit) | Own | M7. |
| Convert to image… | Edit item | Turn selected shapes into an image | Convert to Bitmap | Own | M7 (Ctrl+Shift+B). "Image" is our model's word; keymap.md says "convert to bitmap": proposed change to keymap.md. |
| Preferences… | Edit item | App-wide settings dialog | Settings (Preferences on macOS) | Match | Later. |
| Keyboard shortcuts…, Restore preferences… | Edit items | Shortcut editor; pick an automatic preferences backup | Edit Hotkeys / Load Prefs Backup | Own | Later. Kept next to Preferences… so "Preferences" names one thing; the menus file may move them to File. |
| Preferences backup ▸ (Export preferences…, Import preferences…, Open preferences folder) | Edit submenu | Save, load and show the preferences files | Preferences ▸ (Export Prefs / Import Prefs / Open Prefs Folder) | Match | The doc's "Prefs" spelled out; plain words. Submenu title Own. Later. |
| Select, Edit nodes, Text, Measure | Tools group | Canvas tools with plain names | Select / Edit Nodes / Edit Text / Measure | Match | Select, Text, Edit nodes now (current labels, sentence case); Measure later (Alt+M). |
| Line, Shape tools ▸, Trim, Place tabs, Move head, Barcode… | Tools group | Canvas tools with our own names | Draw Lines / Draw Shape ▸ / Trim Shapes / Add Tabs / Position Laser / Create Bar Code | Echo | Line now (current); Trim and Place tabs echo the doc verbs; Move head and Shape tools Own. Today Rectangle and Ellipse are separate items. Trim (Ctrl+K), Place tabs (Ctrl+Tab) later; Move head M6 (Alt+L). |
| Shape tools ▸ (Rectangle, Ellipse, Triangle, Pentagon, Polygon, Octagon, Star, Alternating star) | Tools submenu | Built-in shape tools, listed exactly | Draw Shape ▸ / Primary Shapes (… Dual Radius Star) | Match | Draft: "Shapes ▸ (… Star…)". Geometry names are plain; "Alternating star" (a star whose points alternate between two lengths) is Own. Rectangle and Ellipse now; the rest later; proposed change to shapes.md (new shape kinds). |
| Offset…, Weld, Union, Subtract, Intersect | Tools group | Shape operations | Offset Shapes / Weld Shapes / Boolean Union / Boolean Subtract / Boolean Intersection | Echo | Now. |
| Combine shapes…, Split by shape | Tools group | Pick a boolean result from previews; split shapes along another shape | Boolean Assistant / Cut Shapes | Own | Draft: "Combine…". Tooltip "Pick a union, subtract or intersect result" (other drawing apps use "combine" for compound paths). Later (Ctrl+B, Alt+Shift+C); keymap.md says "boolean helper" and "cut shapes": proposed change to keymap.md. |
| Image adjustments…, Trace to vectors…, Trace several files…, Clip image with shape, Crop image | Tools group | Image operations | Adjust Image / Trace Image / Multi-File Trace Image (Labs) / Apply Mask to Image / Crop Image | Own | "Crop image" alone is Match. M7; Trace several files… later. |
| Put text on path, Fit slots to thickness…, Perspective warp, Mesh warp, Add tangent circle | Tools group | Special shape operations | Apply Path to Text / Resize Slots in Selection / Warp Selection (4 points) / Deform Selection (16 points) / Generate Tangent Circle | Own | Draft: "Tangent circle" (a noun on an action). Later. Put text on path: Later; proposed change to text.md. |
| Group, Ungroup, Group nested shapes | Arrange group | Grouping | Group / Ungroup / Auto-Group | Echo | Group, Ungroup now. |
| Mirror horizontally, Mirror vertically, Rotate 90° clockwise, Rotate 90° counter-clockwise | Arrange group | Mirror and quarter turns | Flip Horizontal / Flip Vertical / Rotate 90° Clockwise / Rotate 90° Counter-Clockwise | Echo | Now (current labels). |
| Mirror across a line | Arrange item | Click two points, mirror the selection over that line | Mirror Across Line | Match | Draft: "Mirror over line". keymap.md's own wording (Ctrl+Shift+M); generic phrase. Later. |
| Rotate and scale by two points | Arrange item | Pick a pivot and a handle point, then drag to rotate (and scale) | Two-Point Rotate / Scale | Own | Later (Ctrl+2). |
| Align ▸ | Arrange submenu | Line up edges or centers with the anchor object | Align ▸ | Match | Now (current items). |
| Distribute ▸ (… Pack horizontally, Pack vertically) | Arrange submenu | Even spacing; Pack pushes objects edge to edge toward the anchor object | Distribute ▸ (incl. Move H Together / Move V Together) | Match | Now; "Pack" is today's label. keymap.md says "Move together": proposed change to keymap.md. |
| Match width, Match height | Arrange items + toolbar buttons | Resize each selected object to the anchor object's width or height | Make Same Width / Make Same Height | Own | Later; new arrange command (not in arrange.md). |
| Nest | Arrange item | Place the selected parts close together on the bed to save material, computed locally | Quick Nest | Echo | Later. |
| Slide ▸ (Left, Right, Up, Down) | Arrange submenu | Move the selection one way until it touches something | Dock ▸ (Dock Left / Right / Up / Down) | Own | Later; proposed change to arrange.md (currently excluded by design). |
| Move on bed ▸ | Arrange submenu | Move the selection to the bed center, a corner, an edge, or the head | Move Selected Objects ▸ | Own | Now (current label; items Center, Top left, Top right, Bottom left, Bottom right); edges later; "To head position" M6. |
| Go to selection ▸ (Center, Top left, Top, Top right, Left, Right, Bottom left, Bottom, Bottom right) | Machine submenu | Send the laser head to a point of the selection | Move Laser to Selection ▸ | Own | Draft: "Move head to selection ▸". Matches "Go to user origin". Tooltip "Send the laser head to a point of the selection". M6; the menus file decides Machine vs Arrange. |
| Grid array…, Circular array… | Arrange group | Rows-and-columns copies; copies around a center | Grid / Array / Circular Array | Echo | Now (current labels). |
| Path array… | Arrange item | Copies spread along a path | Copy Along Path | Own | Draft: "Copies along path…". Generic CAD term beside Grid and Circular array. Later. |
| Outline around selection | Arrange item | Add the tight convex outline around the selection | Create rubber band outline from selection | Own | Later. |
| Break apart | Arrange item | Split shapes into single segments | Break Apart | Match | Now, but in Tools today (current: Tools → Break Apart); moves to Arrange. |
| Bring forward, Send backward, Bring to front, Send to back | Arrange group | Drawing order | Push Forward / Push Backward in Draw Order / Push to Front / Push to Back | Own | Stock phrases from design programs. Current labels: Forward, Backward, To Front, To Back (no verb). Now. |
| Select next, Select previous | Arrange items | Step the selection through objects | (Tab / Shift+Tab gestures) | Own | Now. |
| Lock, Unlock | Arrange group | Stop or allow editing of the selected shapes | Lock Selected Shapes / Unlock Selected Shapes | Echo | Later. "Lock" is only for shapes; the alarm button is "Clear alarm". |
| Connect, Disconnect | Machine group | Open or close the serial connection | (Devices button, right-click reconnect) | Own | No doc label. M6. |
| Start, Pause, Stop, Frame | Machine group | Job control, same commands as the Machine panel | Start / Pause / Stop / Frame | Match | M6 (Alt+S, Pause key, Alt+R). |
| Frame outline | Machine item | Trace the job's tight convex outline, laser off | Frame (Rubber Band) | Echo | M6 (Alt+Shift+R); keymap.md's wording. |
| Frame at low power… | Machine item | Opens the Low-power frame dialog | (laser-on framing option) | Own | Later, optional, owner decision (safety.md rule 2; README.md, safety). |
| Home | Machine item | Run GRBL's homing cycle | Home | Match | M6 (Ctrl+H). |
| Go to user origin | Machine item | Move the head to the stored user origin | Go to Origin | Echo | M6. |
| Device profiles…, Edit device profile…, GRBL settings… | Machine group | Saved machines, the active machine's settings, the controller's $ settings | Manage Devices / Device Settings / Machine Settings | Own | M8 for profiles (M6 needs one profile); GRBL settings… later. |
| Speed and power test…, Line interval test…, Focus height test… | Machine group | Test-pattern generators | Material Test / Interval Test / Focus Test | Own | M8. |
| Camera list…, Rotary attachment…, Taper correction…, Find center… | Machine group | Accessories and helpers | Manage Cameras / Rotary Setup / Taper Warp / Center Finder | Own | Draft: "Cameras…" (one letter from the Camera panel). M9 (Rotary Ctrl+Shift+R); Find center… later. |
| Print registration ▸ (Guided setup…, Set target 1, Set target 2, Fit job to targets, Place job on targets (no scaling), Clear registration) | Machine submenu | Line a job up with a printed sheet using two targets | Print and Cut ▸ (Start Wizard / Set First Target Position / Set Second Target Position / Align Output to Targets / … (no scaling) / Reset Print and Cut) | Own | Draft item "Fit job to targets without scaling" contradicted itself. Tooltip mentions "print-and-cut". M9 (Alt+0 … Alt+4); keymap.md says "print-and-cut steps": proposed change to keymap.md. |
| Zoom in, Zoom out, Fit bed, Zoom to selection | View group | Zoom | Zoom In / Zoom Out / Zoom to Page / Frame Selection | Echo | First three now ("Fit Bed" current). Zoom to selection later; see its row in Major controls. |
| Show fills, Smooth curves | View check items | Draw fill layers solid (off: strokes only); draw curves with more segments | View Style: Wireframe / Filled, Coarse / Smooth; Toggle Wireframe / Filled | Own | Draft: "Shape display ▸ (Outline, Filled, Smooth edges), Switch outline and filled". One switch frees "outline" for the convex outline. Alt+Shift+W toggles Show fills; keymap.md says "outline / filled": proposed change to keymap.md. Later. |
| Side panels | View item (check) | Hide or show the side panels (F12) | Toggle Side Panels | Echo | Now (current label). The job bar never hides. |
| Job preview… | View item | Open the job preview window (Alt+P) | Preview | Echo | Now (current label). |
| Reset layout | Window item | Put toolbars and panels back where they start | Reset to Default Layout | Own | Later. |
| Toolbars (check list) | Window group | Show or hide each toolbar by short name | toolbar check items | Match | Later. |
| Panels (check list) | Window group | Show or hide each panel by title; each entry's tooltip says what the panel does | window check items | Match | Later. |
| About LaserCraft… | Help item | Version, credits, GPL-3.0-or-later notice | About LightBurn | Own | Opens Quick reference on its About tab. |
| Quick reference… | Help item | Shortcuts, tips and about in one window | Quick Help and Notes | Own | F1 is our context help (keymap.md, M10), so this item has no chord. M10. |
| User guide | Help item | Open LaserCraft's documentation | Online Documentation | Own | M10. |
| License… | Help item | Show the GPL text | End User License Agreement | Own | No licensing UI (README.md, scope). |
| Report a problem | Help item | Open the project's issue tracker in the browser | Support Forum / Submit Feature Request | Own | Own design; no vendor links. |
| Copy system info | Help item | Copy version, OS, display and serial details for a bug report | Generate Support Data | Own | Later. |
| Debug log | Help item (check) | Write a debug log file | Enable Debug Log | Own | Later. |
| Canvas menu (no visible label) | context menu | Right-click on the canvas: clipboard, group, convert, job preview, linked-copy items | (workspace right-click menu) | Own | Later. |
| Panels menu (no visible label) | context menu | Right-click on a toolbar or title strip: the Window menu's check lists | Right-Click Context Menu | Own | Later. |
| Layer menu (no visible label): Output only this layer, Hide other layers, Select shapes, Highlight shapes | context menu | Right-click on a layer row | Layer Context Menu | Echo | Later. |
| All layers menu (no visible label): Output all layers, Output no layers, Invert output, Show all layers, Hide all layers, Invert shown, Air on for all layers, Air off for all layers, Invert air, Put line layers last | context menu | Right-click on the layer list header | Column Header Context Menu (… Sort Cuts Last) | Own | Draft: slash shorthand and "Cutting layers last" (there is no "cutting layer" in layers.md). Later; needs column headers. |
| Library ▸ (New library, Open library…, Save library, Save library as…, Merge library…) | popup menu | Material library file actions in the Materials panel | Manage Library menu | Echo | Draft: "Load…, Save, Save as…, New, Merge…". "Open" as in the File menu, and every item names what it acts on. M8. |
| Saved artwork menu (no visible label): Add selection, Rename, Delete | context menu | Right-click in the Saved artwork panel | Art Library context menu | Match | Later. |
| Camera menu (no visible label): Edit…, Remove, Export…, Add camera…, Import camera… | context menu | Opened by right-clicking the Camera panel | Camera context menus | Match | M9. |
| Image menu (no visible label): Image adjustments…, Trace to vectors…, Export image as engraved… | context menu | Extra canvas items for a selected image | Image context menu | Own | M7. |
| Edit linked copies…, Duplicate with linked copies, Unlink copies, Take out of linked copies, Add to linked copies, Remove linked copies | context menu section | Extra canvas items for an original with linked copies: reopen its settings, copy the whole set, make the copies independent, take originals out, add shapes in, delete the copies and keep the original | Virtual Array context menu items (Edit Array / Duplicate Array / Flatten Array / Remove from Array / Add to Array / Exit Array) | Own | Draft kept four doc captions word for word and said "array". Later; proposed change to arrange.md (virtual arrays are currently excluded by design). |
| Edit barcode… | context menu item | Reopen the Barcode dialog for one selected barcode | Edit Bar Code | Match | Later. |
| Device menu (no visible label): Duplicate, Rename, Sort by name | context menu | Right-click on a row in Device profiles | Devices context menu | Echo | "Sort by type" dropped: every profile is GRBL (README.md, scope). The icon item is omitted. M8. |

## Dialogs and windows

| LaserCraft name (exact UI label) | Kind | What it is (our words, one line) | Public docs term (provenance only) | Tier (Match/Echo/Own) | Notes (why this name; current label if changed; milestone if future) |
|---|---|---|---|---|---|
| Project notes | dialog | Multi-line notes saved in the project | Show Notes / Project Notes | Match | Now (button "Save notes"); switch "Show when the project opens" later. |
| Open project, Save project as, Import vector art, Save G-code | dialogs | Typed-path windows for files | (OS file dialogs) | Own | Now (current titles); native file dialogs deferred (ARCHITECTURE.md). |
| Unsaved changes | modal | Asks before discarding unsaved changes; buttons "Save", "Discard", "Cancel" | Save Project? prompt | Own | Current buttons "Discard", "Cancel". Adding "Save" is an owner decision; proposed change to ui-shell.md. |
| Switch device profile? | prompt | On open, offers the profile the project was saved with | Switch devices? prompt | Echo | M8. |
| History | dialog | Past actions; pick one to preview it, OK goes back there | Undo History | Echo | Later. |
| Offset | dialog | Fields "Distance (mm)" (hint "Negative moves inward"), "Corners" (Round, Miter, Bevel) | Offset (dialog) | Match | Now. Current: "Distance (negative: inward)" and lowercase corner names. |
| Grid array | dialog | Fields "Columns", "Rows", "Column gap (mm)", "Row gap (mm)"; later switch "Make linked copies" | Create Grid Array (… Create Virtual Array) | Echo | Now (current title). Current fields "Gap between columns", "Gap between rows" (units outside the label). Make linked copies: Later; proposed change to arrange.md. |
| Circular array | dialog | Fields "Total count" (hint "Includes the original"), "Center X (mm)", "Center Y (mm)", "Sweep (°)"; switch "Rotate copies" | Circular Array | Match | Now (current title). Current fields "Count (with the original)", "Center x", "Center y", "Sweep", "Turn the copies with the circle". |
| Path array | dialog | "Fixed count" or "Fixed spacing (mm)"; switches "Follow path direction", "Last copy size (%)" | Copy Along Path (Number of Copies / Space Between Copies / Rotate Copies / Scale Copies) | Own | Draft: "Copies along path". Later. |
| Add text | dialog | Text, font, capital height for a new text shape | (text tool click) | Own | Now (current title; it repeats the command). |
| Barcode | dialog | Code type, tabs "Text", "Wi-Fi", "Contact"; "Text source" and "Counter offset" as for text | Bar Code Properties (Raw Content / WIFI / Contact tabs; Text Evaluation; Variable Offset) | Echo | Later; proposed change to shapes.md (barcode shape kind). |
| Fit slots to thickness | dialog | Old and new material thickness, tolerance, what to adjust | Resize Slots | Own | Later. |
| Combine shapes | dialog | Buttons "Union", "Intersect", "First minus second", "Second minus first", "Reset", each previewed on the canvas | Boolean Assistant (Union / Intersection / Subtract A − B / Subtract B − A / Reset) | Own | Later (Ctrl+B). "First"/"second" follow selection order (arrange.md). |
| Align shapes | dialog | Large align buttons with a live preview, for the simple layout | Align Tools Helper | Own | Later, with Simple layout. |
| Measurements | dialog | Size, length, area and segment details of the shape under the pointer | Measurement | Match | Later (Alt+M). |
| Select circles | dialog | "Diameter (mm)", "One size" or "Size range" with "Up to (mm)" | Select Circles (Diameter / Max Diameter / Exact Size / Range) | Match | Later. Field and choice names Own. |
| Font browser | dialog | Search and preview system fonts, mark favorites | Font Manager | Own | Later. |
| Quick reference | window | Tabs "Shortcuts", "Tips", "About" | Help and Notes | Own | M10. |
| Preferences | dialog | App-wide settings | Settings (Preferences on macOS) | Match | Later. |
| General, Grid and snapping, Display, Import and export, Camera | Preferences tabs | Tab names | Editor Settings / Units and Grids / Display / Import / Export / Camera | Echo | Draft tab "Files" read like the File menu. "Import and export", "Display", "Camera" are plain words. Camera tab M9. Later. |
| Restore preferences | dialog | Pick an automatic backup of preferences | Load Prefs Backup / Load Preferences Backup | Own | Later. |
| Keyboard shortcuts | dialog | Browse and change chords; saves a user keymap file | Hotkey Editor / Edit Hotkeys | Own | Later (keymap.md: user keymap). Controls in Major controls. |
| Job preview | window | Simulated job: moves, time slider, summary, bounds check | Preview (window) | Echo | Now; tests look for "Job preview". |
| Cut order | dialog | Rules and switches for the order shapes are cut in | Cut Optimization Settings / Optimization Settings | Own | Later; planning.md ("Cut order") fixes the order today. Controls in Major controls. |
| Layer settings | window | Full editor for one layer, opened by double-clicking a layer; a column of the project's layer swatches on its left switches layers; tabs "Basics", "Advanced" | Cut Settings Editor (Common / Advanced tabs, editor layer list) | Own | Later; today all layer settings live in the Layers panel's layer editor. "Advanced" is a plain word. |
| No selection | message | "Selected shapes only" is on but nothing is selected; buttons "OK", "Use all shapes" | Nothing Selected (Ok / Turn off Cut Selected) | Own | M6. "Use all shapes" turns the switch off; nothing runs until the user starts again. Draft button: "Turn off selected only". |
| Before you start | dialog | The user's own checklist, confirmed before Start when "Confirm before start" is on | Job checklist (Enable Job Checklist) | Own | M8. Nothing starts until the list is confirmed; framing stays laser-off as always. |
| Device profiles | dialog | Saved machines: buttons "Scan for devices…", "New profile…", "Import…", "Export…", "Set as default", "Edit…", "Remove" | Devices (Find My Laser / Create Manually / Import / Export / Make Default / Edit / Remove) | Own | M8. The bridge button is omitted (README.md, scope). |
| Device scan | wizard | Scan serial ports for a GRBL controller and make a profile; results button "Create profile" | Device Discovery Wizard (Find My Laser; Add Device) | Own | Draft: "Find device" (echoed the branded name, verb as title). Opened by "Scan for devices…". M6/M8. |
| New device profile | wizard | Manual setup: name, bed size, origin corner, homing | New Device Wizard (Create Manually) | Own | M8; controller choice is GRBL only. |
| Device profile | dialog | Settings of one machine; title "Device profile: ‹name›" | Device Settings | Own | M8 (M6 needs work area, port, baud). |
| Work area, Options, G-code, Motion | Device profile tabs | Tab names | Dimensions / Units, Basic Settings, GCode, Additional Settings | Own | M8. |
| Axis orientation | dialog | Choose how X and Y map onto the bed | Choose Orientation (Swap X/Y) | Own | M8. |
| GRBL settings | dialog | Read and write the controller's $ settings | Machine Settings | Own | Later. Precise because scope is GRBL only. Buttons in Major controls. |
| Restore GRBL settings | dialog | Pick a saved copy of the $ settings | Load from Backup (Machine Settings) | Own | Later. |
| Calibrate steps per mm | dialog | Requested vs measured distance to correct an axis's steps per mm ($100–$102) | Calibrate Axis | Own | Draft: "Calibrate steps". "Step" here is the motor-step meaning only. Later. |
| Saved head positions | dialog | Named head positions (X, Y, optional Z) | Saved Positions (dialog) | Echo | Draft: "Stored positions" ("saved" and "stored" both used for one idea). Later. |
| Edit macros | dialog | List of macros with add, edit, delete, move up/down | Manage Macros | Echo | Later. |
| New macro, Edit macro | dialogs | Button label and command lines | New Macro | Match | Later. |
| Speed and power test | dialog | Generate a grid of speed × power squares | Material Test Generator | Own | M8 ("test grids (speed × power)"). |
| Line interval test | dialog | Generate patches at different fill intervals | Interval Test | Echo | M8. |
| Focus height test | dialog | Generate lines at different Z heights | Focus Test | Echo | M8; needs a Z axis. |
| No Z axis | message | The focus test needs "Motorized Z axis" on in the device profile | Z axis is not enabled | Own | M8. |
| Image adjustments | window | Before/after panes, tone sliders, presets | Adjust Image | Own | M7. |
| Save adjustment preset | dialog | Name and which settings a preset keeps | Save Image Preset | Own | M7. |
| Trace to vectors | window | Turn an image's edges into shapes | Trace Image | Own | M7. Controls in Major controls. |
| Convert to image | dialog | "Draw as" (Strokes only, Filled, By layer mode) and "Resolution (dpi)" | Convert to Bitmap (Render Type / DPI) | Own | Draft value: "Outlines" ("outline" is the convex outline only). M7. |
| Camera list | dialog | Cameras of this device: add, edit, remove | Device Cameras window | Own | Draft: "Cameras" (one letter from the Camera panel). M9. |
| Add camera | wizard | Pick kind, source, mounting and calibration | New Camera wizard | Echo | M9. |
| Camera setup | window | One camera's source, lens and bed alignment with a live view | Edit Camera | Own | M9. |
| Camera lens calibration | wizard | Capture pictures of a pattern to correct lens distortion; buttons "Capture", "Discard" | Camera Calibration Wizard | Echo | M9. |
| Camera alignment | wizard | Burn and pick targets to line the camera up with the bed | Camera Alignment Wizard | Echo | M9; the burn step is a job and follows safety.md. |
| Rotary attachment | dialog | Rotary type and size settings | Rotary Setup | Echo | M9. |
| Taper correction | dialog | Compensate for tapered cups on a rotary | Taper Warp | Own | M9. |
| Find center | wizard | Find a round part's center from three jogged points | Center Finder | Own | Later. |
| Print registration | wizard | Three steps: target 1, target 2, fit the job | Print and Cut Wizard | Own | M9. |
| Registration failed | message | Why a target could not be set | Print and Cut / Target Set Failed | Own | M9. |
| Nesting failed | message | Why the parts could not be nested | Quick Nest Unsuccessful | Own | Later. |
| Low-power frame | dialog | Opt-in per use: asks for and shows the power before framing with the laser on | (laser-on framing option) | Own | Opened by "Frame at low power…". Later, optional, owner decision (safety.md rule 2; README.md, safety). |

## Major controls (job buttons, toggles, modes, status fields, canvas features)

| LaserCraft name (exact UI label) | Kind | What it is (our words, one line) | Public docs term (provenance only) | Tier (Match/Echo/Own) | Notes (why this name; current label if changed; milestone if future) |
|---|---|---|---|---|---|
| Not connected, Connecting…, Ready, Running, Framing, Paused, Jogging, Homing, Alarm, Door open, Checking, Sleeping, No response | machine state text | State line in the Machine panel and job bar, from GRBL status reports and the app's own job state | Device Status (Disconnected / Ready / Busy) | Own | M6; maps device.md's GRBL states. "Connecting…" covers Connect until the first report; "Framing" comes from the app (GRBL reports Run or Jog); "No response" is the stalled state; "Checking" is GRBL's check state ($C), worded so "mode" stays a layer word (draft: "Check mode"). Proposed change to device.md (state words). |
| Job progress | progress bar | Percent of the job sent, elapsed and remaining time | Busy progress bar | Own | M6; says "sent", not "done". |
| Start | button | Run the job after the bounds check | Start | Match | M6 (Alt+S). |
| Pause / Resume | button | Feed hold; the same button resumes | Pause / Resume | Match | M6 (Pause key); in the job bar too. |
| Pause (motion only) | button variant | Pause for a constant-power (M3) job, with the note "The laser may stay on" | (none) | Own | M6, safety.md rule 9; Stop becomes the highlighted action. |
| Stop | button | Stop streaming and soft-reset the controller | Stop | Match | M6 (Ctrl+Break, keymap.md legacy chord); the strongest control (`danger`); in the job bar. |
| Frame | button | Trace the job's bounding rectangle with the laser off | Frame (Bounding Box) | Match | M6 (Alt+R). |
| Frame outline | button | Trace the tight convex outline around the job, laser off | Frame (Rubber Band) | Echo | M6 (Alt+Shift+R); keymap.md's wording. Tooltip explains "tight outline". |
| Save G-code… | button | Same command as File → Save G-code… | Save GCode | Echo | Button M6 (menu item now). |
| Run G-code file… | button | Stream a saved G-code file after the same checks | Run GCode | Echo | Later. |
| Home | button | Run GRBL's homing cycle | Home | Match | M6 (Ctrl+H). |
| Go to user origin | button | Move the head to the stored user origin | Go to Origin | Echo | M6. |
| Place job at | dropdown | Where the job goes: "Absolute coordinates", "Current position", "User origin" | Start From (Absolute Coords / Current Position / User Origin) | Own | Draft label "Job origin", which the reference uses for the grid (tutorials would send switchers to the wrong control). Values are plain CNC terms (Match). The user guide may say "job origin mode" (CLAUDE.md). M6. |
| Job reference point | 3×3 point grid | Which point of the job's bounds sits on the job origin; greyed with Absolute coordinates | Job Origin (9-dot selector) | Own | Draft: "Anchor" (arrange.md's anchor object keeps that word). Tooltip "Which point of the job lands on the job origin". Separate label from the selection grid because it is a separate setting. M6. |
| Selected shapes only | switch | Start, frame, preview and G-code use only the selection | Cut Selected Graphics | Own | M6. |
| Reference point from selection | switch | The job reference point is taken from the selected shapes, not the whole design | Use Selection Origin | Own | Draft: "Use selection bounds" (one word from the doc caption). Tooltip as in column 3. M6; greyed unless Selected shapes only is on and Place job at is not Absolute coordinates. |
| Use rotary | switch | Use the rotary attachment for jobs | Enable Rotary | Echo | Draft: "Rotary" (a bare noun). M9. |
| Show head marker | switch | Show the head marker on the canvas, following status reports | Show Last Position | Own | Draft: "Show head position" (one word from the doc caption). Ours is a live switch, not a one-shot button. Tooltip "Show where the laser head is on the canvas". M6. |
| Reorder cuts | switch | Let the planner order shapes by the Cut order rules; off runs shapes in drawing order | Optimize Cut Path / Enable Optimization and output re-ordering | Own | Lives at the top of the Cut order dialog. Later; proposed change to planning.md (the order is fixed today). |
| Cut order… | button | Open the Cut order dialog | Optimization Settings | Own | Later. |
| Connect / Disconnect | button | Open or close the serial connection | (Devices button, right-click reconnect) | Own | M6. |
| Port | dropdown | Serial port: "Automatic", "Choose a port", or a port name | Serial Port Dropdown ((Auto) / (Choose)) | Own | M6. |
| Baud rate | dropdown | Serial speed | (none) | Own | M6 (device.md Connect { port, baud }). |
| Refresh ports | button | Look for serial ports again | Re-scan | Own | M6. |
| Device | dropdown | Active device profile | Device Selection Dropdown | Own | M8 (one default profile in M6). |
| Device profiles… | button | Open the Device profiles dialog | Devices button | Own | M8. |
| Serial port access | help box | When a port is refused: the serial group (uucp or dialout), the command, the Flatpak note | (none) | Own | M6 (device.md, linux-serial.md). |
| Alarm | banner | GRBL alarm or error with its meaning; streaming stays stopped | (red error lines in the console) | Own | M6, safety.md rules 10–11. |
| Clear alarm | button | Clear GRBL's alarm lock after an alarm | (console $X) | Own | Draft: "Unlock" (also the shape command). Tooltip "Clear the alarm lock (GRBL $X) once the machine is safe". M6; placement is an owner decision. |
| Test fire | button (placeholder) | Placeholder only | Fire / Power | Own | Beyond a placeholder, out of the guide's scope (README.md, safety); owner decision. |
| Head position | read-out | Live X, Y (and Z) of the head | Get Position and X:/Y:/Z: read-outs | Own | M6; status polling makes a query button unnecessary. |
| Move head to: X (mm), Y (mm), Go | fields + button | Send the head to typed coordinates | Move to Position / Go | Echo | M6. |
| Jog pad | button grid | X−, X+, Y+, Y−, Z up, Z down, Home in the middle; rotary arrows in M9 | Manual Movement (directional arrows) | Own | M6 (Ctrl+Shift+[ ], Ctrl+Alt+[ ]); tooltips "Jog X+" etc. |
| Jog distance (mm) | field | Distance per jog | Distance | Own | Draft: "Step (mm)" ("step" means motor steps only). M6. |
| Jog speed (mm/s) | field | Speed of jog moves (and framing) | Speed | Echo | M6. |
| Z jog distance (mm), Z jog speed (mm/s) | fields | Same for the Z axis | Z-Distance / Z-Speed | Echo | Draft: "Z step (mm)" (almost the layer's "Z step per pass"). M6, only with a motorized Z axis. |
| Continuous jog | switch | Keep moving while a jog button or key is held | Continuous Jog | Match | Draft: "Hold to jog". The standard CNC term users search for. Later (GRBL $J jogging). |
| Absolute jog | switch | Jog arrows go to the jog distance measured from the machine origin | Move From Machine Zero | Own | Draft: "Jog to coordinates". Tooltip "Jog arrows move to the jog distance measured from the machine origin". Later. |
| Set user origin, Clear user origin | buttons | Store or clear the head position as the user origin | Set Origin / Clear Origin | Echo | M6. |
| Set end position | button | Store where the head goes after a job | Set Finish Position | Own | Draft: "Set park position" (GRBL's own parking means the safety-door motion). Pairs with "After the job" in the device profile. Later. |
| Auto focus | button | Run the controller's focus routine | Focus Z | Own | Later, owner decision (not standard GRBL). |
| Saved position + Edit… | dropdown + button | Jump to a named head position; edit the list | Saved Positions / Manage | Echo | Later. |
| Add, Delete, Sort by name; Name, X (mm), Y (mm), Use Z, Z (mm) | Saved head positions controls | List buttons and the fields of the chosen entry | Add New / Delete / Sort by Name; Edit Info (Name / X / Y / Enable Z axis / Z) | Match | Plain words; "Use Z" Own. Later. |
| Speed override, Power override (+10%, +1%, 100%, −1%, −10%) | button columns | Change the running job's speed or power live | Adjust Speed / Adjust Power | Own | Later; GRBL feed and spindle overrides. Middle button "100%" instead of a reset word. Power stays clamped (safety). |
| Show all messages | switch | Console also shows status reports, jogs and other bulk traffic | Show all | Echo | Draft: "All messages". M6. |
| Type a GRBL command | console field hint | Placeholder of the console's input line | (type commands here) | Own | M6. |
| Color; Name; Mode; Speed, power; Output; Show; Air | layer list columns | Column headers of the layer list | # / Layer / Mode / Spd/Pwr / Output / Show / Air | Echo | Draft header "Summary" was vague. No headers today; Air column later. Output and Show are today's tooltips. |
| Line, Fill | layer modes | Trace outlines / scan inside closed shapes | Line / Fill | Match | Now; the combo and summary show raw "line"/"fill" today → "Line"/"Fill". |
| Concentric fill | layer mode | Fill by repeated inward offsets | Offset Fill | Own | Later; proposed change to layers.md (modes are line and fill only). |
| Image | layer mode | The separate image entry for a color | Image | Match | M7. |
| Mixed | layer mode | A layer with more than one operation | Multi | Own | Later, owner decision; proposed change to layers.md. |
| Operations | tab strip | Several settings sets on one layer, run left to right; each tab shows the operation name (default "‹mode› ‹n›", e.g. "Fill 2") | Sub-layer tabs ("1 Line" …) | Own | Draft: "Steps" ("step" already meant jog distance, Z per pass, motor steps, history). Generic CAM word. Later, owner decision; proposed change to layers.md. |
| Add operation, Duplicate operation, Delete operation, Operation name, Output this operation | operation controls | Buttons beside the Operations tabs; name field and output switch of the open operation | Adding / Duplicating / Deleting; Sub-Layer Name; Output Sub-Layer | Own | Later, owner decision; proposed change to layers.md. |
| Output | switch | Include the layer in preview, G-code and jobs | Output | Match | Now. |
| Show | switch | Draw the layer on the canvas | Show | Match | Now. |
| Air | switch | Air assist for the layer | Air | Match | List column later; the editor has "Air assist" now. |
| Move up, Move down | buttons | Change the edited layer's place in output order | Rearrange (up/down) buttons | Own | Now. |
| Delete shapes | button | Delete every shape on the layer; its settings stay | Delete button | Own | Now; clearer than a bare delete. |
| Copy settings, Paste settings | buttons | Copy one layer's settings onto another | Copy / Paste buttons | Echo | Later (layer.copy_settings exists). |
| Layer editor (no visible label) | form | Settings of the edited layer under the list | (quick-edit fields under the list) | Own | Now ("the editor" in ui-shell.md). |
| Name, Mode | layer fields | Layer label; Line or Fill | Name / Mode dropdown | Match | Now. |
| Speed (mm/s), Max power (%), Min power (%), Passes, Air assist | layer fields | Core settings | Speed / Power Max / Power Min / Pass Count / Air Assist | Match | Now; units move from field suffixes (" mm/s", " %") into the labels. |
| Constant power (M3) | switch | Use M3 instead of dynamic M4; marks the job for the Pause warning | Constant Power Mode | Echo | In the model since M5, missing from the editor; add it (safety.md rule 9). Wording from GRBL's laser-mode docs. |
| Kerf offset (mm), Tabs per shape, Tab width (mm) | layer fields | Line-mode extras | Kerf Offset / Tabs Per Shape / Tab Size | Match | Now (current labels). |
| Tabs, Tab placement (Automatic, By hand), Tab spacing (mm), Max tabs per shape, Power across tabs (%), Clear hand-placed tabs, No tabs on inner shapes | tab settings group | Tabs on or off; placed by the planner or by hand; evenly spaced or a count per shape; a cap; power used across a tab; remove hand-placed tabs; put no tabs on shapes inside other shapes | Tabs / Bridges group (Tab Generation Automatic/Manual, Even Spacing, Limit Max Tabs, Tab Cut Power, Clear Tabs, Skip Inner Shapes) | Own | Today 0 tabs per shape means off. Later; proposed change to layers.md (new tab settings). Power across tabs is clamped like every power value. |
| Interval (mm), Scan angle (°), Both directions, Overscan, Crosshatch | layer fields | Fill-mode extras | Interval / Scan Angle / Bi-directional / Overscan / Cross-Hatch | Match | Now; "Both directions" is today's own label. |
| Z offset (mm), Lower Z each pass (mm), Overcut (mm) | layer fields | Advanced line settings | Z Offset / Z step per pass / Overcut (Common / Advanced tabs, Line mode) | Echo | Draft: "Z step per pass (mm)", the doc caption word for word and a second "step". Later. |
| Dashed cut, Dash on (mm), Dash off (mm) | layer fields | Alternate burned and unburned lengths along a line | Perforation Mode (Cut / Skip) | Own | Later. |
| Dwell at start (ms), Dwell at end (ms), Laser on during dwell, Dwell power (%) | layer fields | Pause at the start or end of each cut, optionally with the laser on | Start / End Pause Time, Cut Through, Power | Own | Later; laser-on dwell is part of the job (safety.md rule 1). |
| Lead-in length (mm), Lead-out length (mm), Lead angle (°), Lead shape (Straight, Arc) | layer fields | Short entry and exit moves around the start point | Add Lead-In / Add Lead-Out / Lead In/Out style (Line / Arc) | Own | Draft: "Lead-in, Lead-out" without units. Later. |
| Fill order: All at once, By group, One shape at a time | radio group | How fills are grouped | fill-grouping radios | Own | Later. |
| Ramp length (mm), Skip blank areas | fill fields | Fade power at fill edges; jump over empty stretches of a fill | Ramp Length / Flood Fill | Match | "Ramp length" is the doc's words (plain engraving term); "Skip blank areas" Own. Later. |
| Images on this layer: Method, Interval (mm), Negative | layer section | Collapsed image settings | Image Settings | Own | Now (current heading and labels). |
| Use image as is, Beam compensation (mm), Halftone frequency (lpi), Halftone angle (°) | image fields | Engrave a pre-processed image unchanged; shorten each burned dash so neighbouring dots do not merge; halftone density and angle | Pass-Through / Dot Width Correction / Cells per Inch / Halftone Angle | Own | Draft: "Dot size correction (mm)" (one synonym from the doc caption), "Halftone cells per inch" (inch unit, no parentheses). lpi matches layers.md's suffix. Tooltip for Beam compensation as in column 3. M7. |
| Grayscale, Threshold, Floyd–Steinberg, Jarvis, Stucki, Ordered (Bayer), Halftone | Method values | Image methods by their public algorithm names | image-processing dropdown | Match | Today the combo shows raw ids ("floyd_steinberg") → these names. M7. |
| Brightness, Contrast, Gamma, Sharpen amount, Sharpen radius, Denoise | image sliders | Tone, sharpening and noise reduction for one image (Properties panel and Image adjustments) | Gamma / Contrast / Brightness / Enhance Radius / Enhance Amount / Enhance denoise | Echo | Denoise added. M7. |
| Restore default, Save as default, Save as default for all layers | buttons | Per-device layer defaults | Reset to Default / Make Default / Make Default for All | Own | M8. |
| 00 … 29 | palette swatches | One numbered swatch per layer; current color outlined | Color Palette entry (00-29) | Echo | Now (unnumbered today; README.md (palette) adds two-digit numbers). Tooltip "‹nn›: ‹layer name›" (two digits, as layers-panel.md draws it; today "‹n›: ‹layer name›"), second line "Click to put the selected shapes on this layer." or, with nothing selected, "Click to draw new shapes on this layer." |
| (guide pictogram) 1, (guide pictogram) 2 | palette swatches | Guide layers for helper shapes that are never output | Tool layers T1 / T2 | Own | Draft: "G1", "G2", which read as G-code motion words. Tooltip "Guide layer 1: ‹name›". Later, owner decision; proposed change to layers.md (30 colors only). |
| Project | status field | Name, unsaved dot, path or "not saved" | (none) | Own | Now. |
| Work area | status field | Size and origin corner (front/rear words) | (none) | Own | Now. |
| Pointer position | status field | Pointer X and Y in mm | Cursor Coordinates | Own | Now. |
| Selection size | status field | Count and size of the selection | Tool Status (selection part) | Own | Now. |
| Tool hint | status field | What the active tool does next; line length; rotation angle | Tool Status | Own | Later. |
| Messages | status field | Last notice (device found, file saved) | Notification Area | Own | Later. |
| Active setups | status field | Job setups that are on (rotary, print registration) | Modes Indicator / Enabled Mode(s) | Own | Draft: "Active modes" ("mode" is for layers). M9. |
| Move, Scale, Rotate, Skew | status bar switches (prose: handle switches) | Turn kinds of canvas handles on or off for the project | Transform Control Toggles (Move / Size / Rotate / Shear) | Own | Draft "Resize" broke the handle-name pattern (Scale handles). Later. |
| Bed (no visible label) | canvas feature | Light sheet the size of the work area | Working Size (border) | Own | Now. |
| Grid | canvas feature | Minor and major lines on the bed | Grid | Match | Now. |
| Rulers | canvas feature | Scale numbers along the view edges | ruler | Match | Later (not specced). |
| Machine origin marker (no visible label) | canvas feature | Marks the machine-origin corner of the bed | (origin marker with X/Y letters) | Own | Now. |
| Job origin marker (no visible label) | canvas feature | Marks the job origin, where the job's reference point will land | job origin canvas marker | Own | M6. |
| Head marker (no visible label) | canvas feature | Marks the laser head's reported position | Show Last Position marker | Own | Draft: "Head position marker"; same words as the Show head marker switch. M6. |
| Frame path (no visible label) | canvas feature | Outline the head will trace when framing | frame preview outline | Own | M6. |
| Selection bounds (no visible label) | canvas feature | Dashed combined bounds around the selection, carrying the handles | Selection Indicators | Own | Draft called this "Selection box", reversing canvas.md ("combined bounds"). Now. |
| Selection box (no visible label) | canvas feature | The rectangle dragged from empty space to select | Enclosing / Crossing Selection | Own | canvas.md's term. Now. |
| Inside box, Touching box (no visible label) | canvas feature | Left-to-right box selects shapes inside it; right-to-left selects shapes it touches | Enclosing Selection / Crossing Selection | Own | Draft: "Contain box", "Touch box". canvas.md wording "contained"/"touched". Now. |
| Scale handles | canvas handles | Corner and edge squares for resizing | Size handles | Own | Now (canvas.md term). |
| Rotate handle | canvas handle | Handle above the top edge for rotating | Rotate handles | Match | Now. |
| Move handle | canvas handle | Center handle for dragging the selection | Move handle | Match | Later. |
| Skew handles | canvas handles | Handles for slanting the selection | Shear handles | Own | Later. |
| Arc handle | canvas handle | Bends a text shape along an arc | Bend handle | Own | Later; proposed change to text.md (editable text objects, owner decision). |
| Corner radius handle | canvas handle | Rounds a rectangle's corners by dragging | Rectangle corner radius handle | Echo | Later; proposed change to shapes.md (rect has no corner radius). |
| Nodes, Curve handles, Start node | canvas features | Path points, the handles that shape a curve segment, a subpath's first node | nodes / handles | Match | Now (nodes.md). Draft: "Control handles"; canvas.md says "curve handles" in prose, so the one word is used everywhere (proposed change to canvas.md for its two "Control handle" table rows). |
| Start point | tool | Click a node to choose where a shape's cut begins | Set Start Point | Echo | Later. |
| Start arrow | canvas marker | Shows a shape's start and direction | Set Start Point marker | Own | Later. |
| Round corners, Radius (mm) | tool + field | Click corners to round them | Radius (Radius / Fillet) | Own | Later. |
| Snap points, Snap cursors (no visible label) | canvas features | Node, midpoint, center, intersection, line snaps and their cursors | Object Snapping Points | Echo | Later; canvas.md snapping exists now without cursors. |
| Snap to grid, Snap to shapes | switches | Turn grid or shape snapping on or off | Snap to Grid / Snap to Objects | Echo | Later (snapping is on by default; Ctrl turns it off). |
| Alignment lines (no visible label) | canvas feature | Temporary lines when edges or centers line up | Alignment Guides | Own | Later. |
| Guide lines | canvas feature | Lines dragged out of the rulers, on a guide layer | Draggable Guidelines | Own | Later. |
| Tool cursors (no visible label) | canvas feature | Crosshair cursors with a small badge for the active tool, including the round-corner add/remove/no-room badges | tool cursors / Radius cursors | Match | Later; no per-tool cursors today. |
| Tab markers (no visible label) | canvas feature | Marks where hand-placed tabs sit (a short bar across the path; canvas.md) | Add Tabs marker | Own | Later. |
| Linked copies | canvas feature | Copies that follow edits to their original, drawn paler with dashed outlines | Virtual array clones | Own | Later; proposed change to arrange.md (virtual arrays are currently excluded by design). |
| Guide circle | canvas feature | Circle added by Find center, on a guide layer | Center Finder guide circle | Echo | Later. |
| Registration targets | canvas feature | The two print-registration targets and their state | Print and Cut canvas feedback | Own | M9. |
| Camera overlay | canvas feature | Camera picture under the bed | camera overlay | Match | M9. |
| Select | tool | Default pointer tool; Esc returns to it | Select | Match | Now. |
| Line | tool | Click points to draw lines and polylines | Draw Lines | Own | Now (current label). |
| Rectangle, Ellipse | tools | Drag to draw the shape | Draw Shapes | Match | Now. |
| Shape tools | flyout | Shape tools behind one button | Draw Shapes (Primary Shapes) | Own | Draft: "Shapes" (the model's word for every drawn thing). Later. |
| Triangle, Pentagon, Polygon, Octagon, Star, Alternating star | tools | Built-in shapes; Polygon and Star have adjustable sides or points | Primary Shapes (Triangle / Pentagon / Polygon / Octagon / Star / Dual Radius Star) | Match | "Alternating star" Own (points alternate between two lengths). Later; proposed change to shapes.md. |
| Edit nodes | tool | Edit nodes and segments of paths | Edit Nodes | Match | Now (current label, sentence case). |
| Delete node (D), Add node here (I), Add node at middle (M), Make straight (L), Make curve (C), Break at node (B), Make node smooth, Make node sharp, Level by segment, Trim segment, Extend to crossing | Node tool buttons (tooltips) | One button per node action; the first six repeat nodes.md's keys; the rest rotate the selection so a segment lies at 0°, 45° or 90°, cut a segment back to a crossing, or lengthen an open end to the next path | Edit Nodes actions (Delete node / Insert / Insert node at midpoint / Convert curve to line / Convert line to smooth curve / Break shape at point / Smooth corner node / Convert smooth node to corner / Align Selected / Trim line / Extend line) | Own | First six now as keys (nodes.md), buttons later; the last five later. |
| Trim | tool | Click a segment to cut it back to the next crossing | Trim Shapes | Echo | Later (Ctrl+K). |
| Place tabs | tool | Click a line-mode shape to place a tab by hand | Add Tabs | Echo | Later (Ctrl+Tab). |
| Text | tool | Click to add text | Create / Edit Text | Match | Now. |
| Move head | tool | Click the bed to send the laser head there | Position Laser | Own | M6 (Alt+L). |
| Measure | tool | Hover or drag to measure | Measure | Match | Later (Alt+M). |
| Booleans | flyout | Union, Subtract, Intersect and Combine shapes… | Boolean Tools | Echo | Draft: "Combine" (same name as one of its own items, and "combine" means compound paths elsewhere). Later. |
| Pan | tool | Drag the view (also middle button or Space) | Panning mode / Pan / Drag View | Match | Now (gestures). |
| X (mm), Y (mm), Width (mm), Height (mm), Width (%), Height (%) | fields | Typed position, size and scale of the selection | XPos / YPos / Width / Height / percent fields | Echo | Later (Position and size toolbar). |
| Keep proportions | lock toggle | Width and height change together | Lock (aspect ratio) | Own | Later. |
| Reference point | 3×3 point grid | Point of the selection's bounds used for position, resize and rotation | 9-Dot Control | Own | Draft: "Anchor" (arrange.md's anchor object keeps that word). Later. |
| Rotation (°) | field | Rotate the selection by a typed angle | Rotate | Echo | Later. |
| mm / in | units toggle | Design units | Units Toggle | Match | Later (README.md, units). |
| Font, Capital height (mm) | text fields | Font family and capital-letter height | Font / Height | Echo | Now (Add text dialog); later also on the Text style toolbar for the next text. |
| Bold, Italic, Right to left | text switches | Style and direction | Bold / Italic / Right-to-Left | Match | Later; proposed change to text.md (editable text objects, owner decision). |
| All caps, Weld letters, Bend letters along arc | text switches | Force capitals; weld touching letters into one outline; let letters bend with the arc | Upper Case / Welded / Distort | Own | Draft: "Merge letters" (avoided our own Weld), "Warp letters on curve" (unclear; "warp" names other tools). Later; proposed change to text.md. |
| Letter spacing (%), Line spacing (%), Horizontal align, Vertical align | text fields | Spacing and anchoring of text | HSpace / VSpace / Align X / Align Y | Own | Later; proposed change to text.md. |
| Text source: Plain, Date and time, From CSV, Counter, Layer speed and power | dropdown | Where a text shape's content comes from | Text Mode dropdown (Normal / Date/Time / Merge/CSV / Serial Number / Cut Setting) | Own | Draft options "Serial number" (the same counter is "Counter" elsewhere) and "Layer settings" (the title of the Layer settings window). Later; proposed change to text.md. |
| Counter offset | field | Added to the Text data counter for this shape | Offset (integer spin field) / Variable Offset | Own | Later; proposed change to text.md. |
| Current value, First value, Last value, Increment | Text data fields | The value used next and the range it counts through | Current / Start / End / Advance by | Own | Later; proposed change to text.md. |
| Previous, Next, Show values, Back to first, Freeze values, Count up after each job | Text data buttons + switch | Step the counter; show substituted text while held; reset to the first value; replace codes by their current text for good; count up after each job is started or saved as G-code | Previous / Next / Test / Reset / Bake / Auto-Advance | Own | "Previous" and "Next" are plain words. Later; proposed change to text.md. |
| CSV file, Choose file…, Remove file | Text data section | The loaded CSV and its buttons | CSV File / Browse / Clear | Own | Later; proposed change to text.md. |
| Max width (mm), Shrink width only, Skip empty fields | text properties | Width limit; shrink horizontally only to fit; drop line breaks before empty CSV fields | Max Width / Squeeze / Ignore Empty Vars | Own | Draft: "Narrow only" (not plain English). Later; proposed change to text.md. |
| Run order, Power (% of layer), Locked | shape properties | Per-shape number for the Cut order run-order rule (lower runs first); the shape's power as a share of its layer's; edit lock | Cut Order Priority / Power Scale / Locked | Own | Draft: "Order priority" (the doc caption minus one word), "Power scale (%)" (the doc caption word for word). "Locked" is a plain word (Match). Power stays clamped to layer and device maximum. Later. |
| Corner radius (mm), Sides, Points, Inner radius (%) | shape properties | Fields of built-in rectangles, polygons and stars | Ellipse / Polygon / Rectangle properties (Corner Radius / Sides / points / ratio) | Echo | Later; proposed change to shapes.md. |
| Slide as one block, Inner shapes follow outer, Keep rotation, Gap (mm), Edge margin (mm) | Slide and nest options | Slide the whole selection together; shapes inside another keep their place relative to it; nesting does not turn parts; space between parts; space kept from the bed edge | move as group / lock inner objects / lock-for-rotation / Padding / Edge Buffer | Own | Draft: "Move together" (keymap.md's pack wording) and "Keep inner shapes" (read as "do not delete"). Later; proposed change to arrange.md. |
| Library, Rename library…, Close library | Materials row | Pick the active library; rename it; take it off the list without deleting the file | Select Library / Rename / Unload | Own | M8. |
| Apply to layer, Follow this entry, Save layer as material…, Edit settings…, Update from layer, Edit description…, Duplicate, Delete | Materials buttons | Copy an entry onto the active layer; keep the layer tied to the entry; store the layer as an entry; edit an entry's settings, refresh it from the layer, edit its text; copy or remove it | Assign / Link / Create new from layer / Edit Cut / Update / Edit Desc / Duplicate / Delete | Own | "Duplicate", "Delete" are plain words. M8. |
| Material, Thickness (mm), Thickness does not apply, Group name, Description, OK, Cancel | material entry form | Fields shown in place of the buttons when saving or describing an entry | Material Name / Thickness / No Thickness / Title / Description / Ok / Cancel | Own | "Group name" is used instead of a thickness when Thickness does not apply. M8. |
| Capture camera image, Trace camera image… | Camera buttons | Refresh the overlay; open Trace to vectors on the camera picture | Update Overlay / Trace Overlay | Own | Draft "Trace camera image" lacked the ellipsis. M9 (Alt+C). |
| Overlay fit: Overlay opacity (%), Overlay width (%), Overlay height (%), Overlay offset X (mm), Overlay offset Y (mm) | Camera overlay controls | Fade, stretch and nudge the camera overlay | Adjust Overlay / Overlay Opacity / Width (%) / Height (%) / X Shift / Y Shift | Echo | "Overlay opacity" is a plain phrase. M9. |
| Camera header (no visible label), Picture settings, Advanced | Camera panel blocks | Per-camera status row; exposure, focus and color sliders; capture options | Camera status area / Camera Settings / Advanced | Own | M9; "Advanced" is a plain word. |
| Width (mm), Height (mm), Origin corner (Front left, Front right, Rear left, Rear right), Motorized Z axis, Red-dot pointer offset, Pointer offset X (mm), Pointer offset Y (mm) | device profile fields (Work area tab) | Work area size, which corner is the machine origin, whether Z moves, and the aiming dot's offset from the beam | Dimensions / Units (Working Size, Origin, Enable Z Axis, Enable pointer offset) | Echo | M8; corner words are today's status-bar words (machine words). Motorized Z axis gates the Z jog fields and the focus test. |
| Home on connect, Full-power S value ($30), Max power (%), Confirm before start + Edit list…, Repeat framing, Air assist code (M7, M8) | device profile options (Options tab) | Per-machine behavior; the S value that means full power must equal GRBL's $30; the device's power ceiling; a user checklist before Start; framing repeats until stopped | Basic Settings (Auto-Home on Startup, S-value Max, Enable Job Checklist, Frame Continuously, Air Assist M7/M8) | Own | Draft: "Max S value" (doc words reordered), "Job checklist" (doc feature name). Max power (%) is the device clamp of safety.md rule 6 and gcode.md's max_power. Tooltip for Full-power S value mentions "S max". Repeat framing stays laser-off. M6/M8. |
| After the job (Stay, Go to origin, Go to end position), End position X (mm), End position Y (mm) | device profile options (Options tab) | Where the head goes when a job ends | Return to Finish Position (X / Y) | Own | Proposed change to gcode.md (today the finish option is on/off for G0 X0 Y0). Set end position on the Jog panel stores the X/Y. Later. |
| Streaming (Character counting, Send and wait), Status poll interval (ms), Stall timeout (s) | device profile options (Options tab) | How G-code is streamed; how often `?` is sent; how long without a reply counts as stalled | Transfer mode (Buffered / synchronous) / Query regularly for laser position | Own | Streaming names follow GRBL's published protocols. M6 (device.md plans counting with a send-and-wait fallback; whether the user can force it is an owner decision). |
| Scan shift correction, Speed (mm/s), Shift (mm), Add row, Delete row, Import…, Export… | device profile switch, table columns and buttons (Options tab) | Shift fill and image lines by a speed-dependent amount so both scan directions line up | Scanning Offset Adjust (Enable Scanning Offset Adjustment, Speed / Line Shift table, Add / Delete / Import / Export) | Own | M7/M8. The table has no caption of its own: the switch label heads the group and the two column headers say what each row holds (machine-panels.md). Draft caption "Scan shift by speed" dropped, since no file draws it. |
| Start G-code, End G-code | device profile fields (G-code tab) | Lines sent before and after every job | GCode tab (start and end G-code) | Match | M8; plain G-code terms. |
| Overscan as rapid moves (G0), Repeat S on every G1, Read GRBL settings on connect, Check work offsets on connect | device profile switches (G-code tab) | Output and connection options for GRBL | Use G0 Moves for Overscan / Emit S Value With Every G1 Command / Fetch Settings on Connect / Check for workpiece offsets on Connect | Own | Later; output-changing switches are a proposed change to gcode.md. |
| Max speed X (mm/s), Max speed Y (mm/s), Acceleration (mm/s²), Read from controller | device profile fields (Motion tab) | Motion values used for time estimates; fill them from GRBL's $110/$111 and $120/$121 | Additional Settings (Simulation Settings, Read From Controller) | Own | "Read from controller" is a stock phrase (Match). M8. |
| Scan for devices…, New profile…, Import…, Export…, Set as default, Edit…, Remove | Device profiles buttons | Profile manager actions | Find My Laser / Create Manually / Import / Export / Make Default / Edit / Remove | Own | Draft button: "Find device". M8. |
| Create profile | Device scan button | Make a profile from the found controller | Add Device | Own | M6/M8. |
| Read from controller, Write to controller, Save to file…, Load from file…, Restore backup…, Calibrate steps per mm… | GRBL settings buttons | Read or write the $ settings; save or load them as a file; pick an automatic backup; open the step calibration | Read / Write / Save to File / Load from File / Load from Backup / Calibrate Axis | Match | Stock phrases; Restore backup… and Calibrate steps per mm… Own. Later. |
| Add layer rule, Add group rule, Add run-order rule, Remove rule | Cut order rule controls | Build the nested list of ordering rules shown between them | Order by Layer / Order by Group / Order by Priority / Remove | Own | Later; proposed change to planning.md. |
| Inner shapes first, Shortest travel, Sweep in one direction (From top, From bottom, From left, From right), Backlash-friendly order, Keep head direction, Start anywhere on closed shapes, Prefer starting at corners, Allow reversing paths, Skip duplicate lines + Within (mm), Save as default, Restore default | Cut order switches and buttons | Ordering options and their defaults | Cut Inner Shapes First / Reduce Travel Moves / Cut in Direction Order (From Top …) / Hide Backlash / Reduce Direction Changes / Choose Best Starting Point / Choose Corners, if Possible / Choose Best Direction / Remove Overlapping Lines (Distance) / Set as Defaults / Reset to Defaults | Own | Named exception to the one-word-dropped rule: "Inner shapes first" is existing-spec wording (planning.md:80–81, "inner shapes first"; CLAUDE.md M4 "inner first"), and existing specs win (README.md), so the label is kept; it is Match tier for this label. "Shortest travel" is CLAUDE.md M4's wording; sweep directions are screen words. Later; proposed change to planning.md beyond the first two. |
| Auto-save every (min), Keep a job log, Block Start while the screen is off, Start button skips keyboard focus | General preferences | Backup interval (0 = off); a CSV log of each job; two guards against starting a job by accident | Auto-save Interval / Save Job Log / Ignore 'Start' Button if Monitor is Off / Asleep / Don't Let 'Start' Button Hold Keyboard Focus | Own | Owner decision whether the two Start guards are fixed behaviour instead of switches (safety.md). Later. |
| Default font, Default capital height (mm), Right-to-left text, Single-line font folder (SHX)…, Clear folder, Double-click edits text | Text preferences (General tab) | Defaults for new text; where single-line fonts live; whether double-click reopens a text shape | Text Settings (Default Font / Default Height / Set SHX Font Path / Enable Right-To-Left / Selection Double-Click Shows Text Edit Dialog) | Own | Later. Double-click edits text: proposed change to text.md. |
| Grid spacing (mm), Snap spacing (mm), Grid strength (Off, Faint, Medium, Strong), Click tolerance (px), Snap distance (px) | Grid and snapping preferences | Grid and snapping settings | Visual Grid Spacing / Grid Snap Distance / Grid Contrast / Click-Selection Tolerance / Object Snap Distance | Own | Draft click and snap distances had no units; canvas.md gives them in px. Later. |
| Nudge distances (no visible label: Small (mm), Normal (mm), Large (mm)) | settings group | Arrow-key move steps | Shape Move Increments | Own | Values fixed now (keymap.md: 0.1, 1, 10 mm); setting later. |
| Reduce motion, Show swatch numbers, Handle size, Icon size, Text size | Display preferences | Reduce motion stops the moving selection dashes and slide-in animations; the rest set how the interface draws | Display tab (Reduce Motion / Show Palette Button Labels / Cursors / Nodes / Toolbar Icon Size / Font Size) | Own | Reduce motion is the OS accessibility term. The docs describe the reference's setting only vaguely, as trying to reduce motion and flicker in rendering; LaserCraft defines its own concrete effect (above). Draft: "Number the swatches". Later. |
| Theme: Light, Dark, Follow system | dropdown | Interface theme; the bed stays light | application theme dropdown | Match | Later (README.md, themes). |
| DXF units (Detect, Millimetres, Centimetres, Metres, Inches, Feet), DXF close tolerance (mm), SVG resolution (px per inch: 96, 72, Detect), Output curve tolerance (mm) | Import and export preferences | Import scaling and tolerances; how closely output follows curves | Import / Export tab (DXF Import units / Auto-Close Tolerance / SVG Import Settings (96 / 72 DPI) / Curve Tolerance) | Own | Options name numbers, not other programs. Later; proposed change to import.md (units are fixed today). |
| Search commands, Press keys, Clear shortcut, Reset all, Export keymap…, Import keymap… | Keyboard shortcuts controls | Filter the command list; capture a chord; remove it; restore defaults; write or read a user keymap file | Search / Press shortcut / Clear / Reset All / Export / Import | Echo | "Search commands" and "Press keys" are field hints. Later. |
| Preset, Save preset…, Delete preset, Import presets…, Export presets…, Reset all, Show inverted; groups "For the whole layer", "For this image only" | Image adjustments controls | Presets; undo every change; preview light-on-dark; which settings change the layer and which only this image | Presets / Reset All / Invert Display / Layer Settings / Image Settings | Own | M7. |
| Brightness from, Brightness to, Use transparency, Follow edges (sketches), Drop specks under (px), Smoothness, Node reduction, Dim image, Show nodes, Clear trace area, Delete image when done | Trace to vectors controls | Brightness range to trace; trace by transparency or by local edges; ignore tiny bits; curve smoothing; fewer nodes; view helpers; remove the image after OK | Cutoff / Threshold / Trace Transparency / Sketch Trace / Ignore Less Than / Smoothness / Optimize / Fade Image / Show Points / Clear Boundary / Delete Image After Trace | Own | "Smoothness" is a plain word (Match). Draft: "Edge mode (sketches)" ("mode" is a layer word only). M7. |
| Top of design, Top diameter (mm), Bottom diameter (mm), Length (mm) | Taper correction fields | Which end of the design is the wide end of the cup, and the cup's measurements | Taper Warp (object top, top diameter, length, bottom diameter) | Echo | M9. |
| Network name, Password, Hidden network, Security; First name, Last name, Address, Phone, Note, Email, Web address | Barcode Wi-Fi and Contact tab fields | Fields that fill a QR code's text | WIFI tab (Network name / SSID, Password, Hidden network, Authentication type); Contact tab (First name … URL) | Match | Plain field names; "Security" and "Web address" Own. Later. |
| Time (s) | preview slider | Scrub through the job | Time Slider | Echo | Now; current label "time" with suffix " s". |
| Show whole job | preview button | Jump the slider to the end | (none) | Own | Current label "Whole job" (a noun on a button). Now. |
| Show travel | preview switch | Draw laser-off travel moves | Show Traversal Moves | Own | Now. |
| Play, Playback speed | preview button + slider | Animate the job | Play / Playback Speed | Match | Later. |
| Lighter for low power, Invert colors | preview switches | Draw low-power output lighter; light on dark | Shade According to Power / Invert | Own | Draft: "Shade by power" (the doc caption squeezed). Later. |
| Show legend | preview switch | Show the key to the move styles | Show Legend | Match | Later. |
| Legend: Burn, Unlit, Travel | preview key | Key to the three move styles | Legend (Cut / Move) | Own | Draft "Laser on / Laser off / Travel" did not match preview.md (travel is laser-off too). Later. |
| Burn moves, Unlit moves, Travel moves, Head marker, Bed edge (no visible label) | preview drawing | How the preview draws the job | Preview rendering (cut / travel / fill / image, work area border, head marker) | Own | Now (preview.md); the legend uses the same words. Draft "Bed outline": "outline" is the convex outline only. |
| Job summary (no visible label) | preview text | Burn, unlit and travel distances, time, bounds | Job Time Statistics | Own | Now. Current text says "laser-off run-on" for unlit moves → "unlit". |
| Fits the work area. / Cannot run: ‹reason› | preview status text | Result of the bounds check | (none) | Own | Now; tests look for "Cannot run:". |
| Run from here | preview button | Start the real job from the slider position | Start Here | Own | Later; fires the laser, so safety rules apply. |
| Save image…, Close | preview buttons | Save the preview picture; close the window | Save Image / Ok | Echo | Later. |
| Show fills, Smooth curves | view switches | Fill layers drawn solid (off: strokes only); smoother curve drawing | View Style (Wireframe / Filled, Coarse / Smooth) | Own | Draft: "Outline, Filled, Smooth edges". View menu check items; Alt+Shift+W toggles Show fills. Later. |
| Simple layout | layout (setting) | A reduced interface for new users | Beginner Mode | Own | Later, optional (README.md, simple mode). Title suffix "(simple layout)". |
| OK, Cancel, Apply, Close | dialog buttons | Confirm, discard, apply without closing, close a non-editing window | OK / Cancel / Apply / Ok | Match | Plain words; the preview's closing button says "Close". |
| Back, Next, Skip, Finish, Cancel | wizard buttons | Move through Device scan, New device profile, camera and calibration wizards | Go Back / Next / Skip / Finish / Cancel | Match | Plain words. M6–M9. |
| Capture, Discard | wizard buttons | Take or drop one calibration picture | capture / discard buttons | Match | M9. |
| Tooltip | popup | "‹what it does› (‹chord›)", chord shown with symbols | Tooltips | Match | Now (e.g. "Rectangle tool (Ctrl+R)"). |
| Context help | interaction | F1 opens the user guide page for the item under the pointer | Topic-Aware Help (F1) | Own | M10. |
| Zoom to selection | view command | Zoom so the selection fills the view | Frame Selection | Echo | Later. keymap.md binds Ctrl+Shift+A to laser framing of the selection (legacy page) while the 2.1 Window-menu page shows that chord on zoom to selection: open question for the owner; proposed change to keymap.md if zoom gets it. |

## Labels adopted from the other style files

The other style files introduced the labels below while specifying their
parts and listed them as proposals for this file. They are reviewed here
against the Naming rules and are now binding like every other row; the
files that use them quote them from here. A proposal that was changed or
rejected is listed under Naming decisions with its replacement. Rows are
grouped by the file that places them on screen; that file owns placement,
look and behavior.

### Arrange, Align and distribute, Node tools, Slide and nest (toolbars.md)

| LaserCraft name (exact UI label) | Kind | What it is (our words, one line) | Public docs term (provenance only) | Tier (Match/Echo/Own) | Notes (why this name; current label if changed; milestone if future) |
|---|---|---|---|---|---|
| Align centers | toolbar button | Put every object's center on the anchor object's center | Align Centers | Match | Plain words. Direct button on the Arrange toolbar and first align button of Align and distribute. Now (Alt+Home is today's chord). |
| Align on a vertical line | flyout button | Holds the three alignments along an upright line: left edges, vertical centers, right edges | (vertical-line family; the notes give no family name) | Own | Name used by the More popup and the flyout's first tooltip line; the face shows the chosen member. Now. |
| Align on a horizontal line | flyout button | Holds top edges, horizontal centers, bottom edges | (horizontal-line family) | Own | As above. Now. |
| Left edges, Vertical centers, Right edges, Top edges, Horizontal centers, Bottom edges | flyout members | The six alignments, named by what lines up | Align Left / Align Vertical Centers / Align Right / Align Top / Align H-Center / Align Bottom | Match | The same words as today's Align ▸ items in sentence case. Now. |
| Distribute vertically, Distribute horizontally | flyout buttons | Hold the even-spacing actions for one direction | (distribute families) | Own | Now. |
| Vertical spacing, Vertical centers, Horizontal spacing, Horizontal centers | flyout members | Equal gaps between objects, or equal steps between their centers | Distribute V-Spaced / V-Centered / H-Spaced / H-Centered | Own | "Vertical centers" also names an align member; the flyout it sits in tells them apart, and the long forms below are used where both could meet. Now. |
| Align left edges, Align vertical centers, Align right edges, Align top edges, Align horizontal centers, Align bottom edges | toolbar buttons (Align and distribute) | One button per alignment, each a verb phrase | Align Left … Align Bottom | Echo | Long form of the flyout members, used where a button stands alone. Now. |
| Distribute vertical spacing, Distribute vertical centers, Distribute horizontal spacing, Distribute horizontal centers | toolbar buttons (Align and distribute) | One button per distribute action | Distribute V-Spaced … H-Centered | Own | Now. |
| Move to top left, Move to top right, Move to bottom left, Move to bottom right, Move to bed center, Move to head position | toolbar buttons (Align and distribute) | Move the selection to a bed corner, the bed's middle, or under the laser head | Move Selected Objects items | Own | Verb form of the Move on bed ▸ items (Top left … To head position). Move to head position: M6; the rest Now. |
| Node pointer | toolbar button (Node tools) | The plain node tool with no action armed: drag nodes and curve handles | (none in the notes) | Own | First Node tools button; Esc or a click on it disarms an action. Later. |
| Slide left, Slide right, Slide up, Slide down | toolbar buttons (Slide and nest) | Move the selection one way until it touches a shape or the bed edge | Dock Left / Right / Up / Down | Own | Button form of Slide ▸ (Left, Right, Up, Down). Later; proposed change to arrange.md. |

### Machine panel, Jog, Console, Macros, device dialogs (machine-panels.md)

| LaserCraft name (exact UI label) | Kind | What it is (our words, one line) | Public docs term (provenance only) | Tier (Match/Echo/Own) | Notes (why this name; current label if changed; milestone if future) |
|---|---|---|---|---|---|
| Job refused | band | Notes band shown when Start, Frame, Frame outline or Run G-code file… was refused before anything was sent; carries the "Cannot run: …" message | (none) | Own | `danger-soft` band with a `danger` mark (visual-language.md). Prose name; the band's visible text is the message itself. M6, safety.md rule 5. |
| End framing | button state | What the Frame button that started a repeating frame reads; ends the loop after the current pass | (a small dialog whose button stops continuous framing; Frame Continuously setting) | Own | Repeat framing stays laser-off. Stop remains the immediate way out. M6/M8. |
| Show in job preview | link button | Opens the job preview at the problem named in the Job refused band | (none) | Own | M6. |
| Show in console | button | Brings the Console forward at the line that caused an alarm, error or stall | (none) | Own | M6. |
| Jump to latest | button | Appears when the Console log is scrolled up; scrolls back to the newest line | (none) | Own | M6. |
| Clear log | context menu item | Empties the Console view; the saved history is kept | (none) | Own | M6. |
| Scan again | button | Repeats the Device scan after nothing was found | (none) | Own | M6/M8. |
| Try again | button | In the Serial port access help box: connect again after fixing permissions; also the last page of Camera alignment (later-windows.md) | (none) | Own | Same words for "repeat the step that failed" everywhere. M6 / M9. |
| Hide | button | Closes the Serial port access help box | (none) | Match | Plain word. M6. |
| No serial ports found | list item (greyed) | Shown in Port when no port exists; tooltip "Check the USB cable and that the laser is powered; then use Refresh ports." | (none) | Own | M6. |
| Reset GRBL settings?, Reset settings | confirmation + button | Asked before a Console reset command; the button names the result | (none) | Own | Confirmations rules. M6. |
| Write GRBL settings?, Discard ‹n› unwritten changes? | confirmations | Asked before writing $ settings, and before reading over unwritten edits | (none) | Own | Buttons "Write to controller" / "Discard", each with Cancel. Later. |
| Signals and steppers, Reports, Limits and homing, Laser, Axes | GRBL settings group rows | Our own grouping of GRBL's $ settings by number range ($0–$6, $10–$13, $20–$27, $30–$32, $100–$132) | (collapsible category bands) | Own | Group words follow the GRBL configuration wiki's topics in our own words. Columns "Setting", "Value", "Unit". Later. |
| Restore | button (Restore GRBL settings) | Loads the chosen backup into the table, still unwritten | Load from Backup | Own | Later. |
| Axis, Current steps per mm, Commanded distance (mm), Measured distance (mm), New steps per mm | Calibrate steps per mm fields | Pick the axis; read the current value; type what was asked for and what moved; read the new value | (axis, requested and actual distance, new step value) | Own | "Step" keeps its motor-step meaning. Later. |
| Delete macro?, Delete macro | confirmation + button | Asked before a macro is removed | (none) | Own | Later. |
| Add…, Edit…, Delete, Move up, Move down | Edit macros buttons | List actions for macros | (manage macros buttons) | Match | Plain words. Later. |
| Button label, Command lines | New macro / Edit macro fields | The macro button's text; the GRBL lines it sends (hint "One GRBL command per line") | (none) | Own | Later. |
| Start job | button | The primary button of Before you start, enabled once every item is ticked; never pressed by Enter | (none) | Own | Distinct from Start so the checklist cannot be skipped with one key. M8. |
| Edit list | settings window title | Edits the Before you start checklist; opened by "Edit list…" in the device profile | (job checklist editor) | Own | Title repeats its button without the ellipsis (Parts of speech). M8. |
| Jog refused: it would leave the work area. | message | Messages field text when a jog target lies outside the bed; nothing is sent | (none) | Own | safety.md rule 5 applied to jogs. M6. |
| — Not sent: this line could turn the laser on. Use a job or framing instead. | console note | Appended to a typed Console line that could fire the laser outside a job | (none) | Own | safety.md rule 1. Owner decision (machine-panels.md question). M6. |
| Line ‹k› could turn the laser on; it will not be sent. | field warning | Under a macro's Command lines | (none) | Own | Later. |
| Finished in ‹h:mm:ss› | status line | Stays under the state line after a job until the next action | (none) | Own | M6. |
| Planning… | progress text | Replaces the progress bar while the job length is still unknown | (none) | Own | No indeterminate animation (visual-language.md). Later. |

### Dialogs (dialogs.md)

| LaserCraft name (exact UI label) | Kind | What it is (our words, one line) | Public docs term (provenance only) | Tier (Match/Echo/Own) | Notes (why this name; current label if changed; milestone if future) |
|---|---|---|---|---|---|
| Replace file?, Replace | message + button | Asked when Save project as or Save G-code would overwrite an existing file; Replace is the destructive button, Cancel has focus | (none) | Own | Confirmations rules. With native file dialogs or earlier. |
| Shape, Segment | Measurements headings | The two groups of the Measurements dialog | (none in the notes) | Own | Generic geometry words. Later. |
| Nodes, Straight segments, Curved segments, Kind, Width (mm), Height (mm), Perimeter (mm), Area (mm²) | Measurements read-outs (Shape) | Counts, closed or open, bounds and size of the shape under the pointer | (measurement read-outs) | Own | Generic geometry terms checked against clean-room.md. Kind values "Closed path", "Open path", "‹n› paths". Later. |
| Length (mm), From (mm), To (mm), Change (mm), Angle (°), Circle radius (mm), Circle center (mm) | Measurements read-outs (Segment) | The hovered or measured segment: length, end points, offset between them, direction, and the arc's radius and center | (measurement read-outs) | Own | Spelling "center" as in every other label (Center X (mm)). "—" when a value does not apply. Later. |
| Copy values | button | Copies every Measurements label and value as text lines | (none) | Own | Later. |
| Point at a shape to measure it. | empty state | Measurements with nothing under the pointer | (none) | Own | Later. |
| Up to (mm) | field hint / label | Upper bound of a size range (Select circles); "Up to ‹cap›%" as the Low-power frame's hint | (Max Diameter) | Own | Later. |
| Rules, Options | Cut order headings | The rule builder and the switches below it | (order rules, options) | Own | Later. |
| By layer order, Groups kept together, By run order (lowest first) | Cut order rule rows | How each rule reads in the rule list | (the three ordering rules; doc terms in the Cut order rule controls row) | Own | Each row says what the rule does; the buttons that add them keep "Add … rule". Later; proposed change to planning.md. |
| Saving, Starting jobs, Text, Layout, Grid, Snapping, Nudge distances, Interface, Canvas, DXF, SVG, Output | Preferences group headings | Group headings inside the Preferences tabs | (settings groups) | Own | "Text" here is a heading, not the Text tool or menu item; the tab tells them apart. Later. |
| Smaller, Normal, Larger, Largest | dropdown values | Text size and Icon size steps (Icon size stops at Larger) | (font size, toolbar icon size) | Own | Later. |
| Restore default sizes | button (Preferences, Display) | Puts text, icon and handle sizes back to their defaults | (none) | Own | The way back from an unusable size; Reset layout does not touch sizes (visual-language.md). Later. |
| All commands, Canvas keys, Machine keys, Command, Shortcut | Keyboard shortcuts list rows and columns | Category rows beside the per-menu rows; the two columns of the command list | (hotkey editor list) | Own | Later. |
| Restore backup | button (Restore preferences) | Primary button: loads the chosen backup after saving the current preferences as a new one | Load Prefs Backup | Own | The menu item keeps "Restore preferences…". Later. |
| Current thickness (mm), New thickness (mm), Tolerance (mm), Change: Slot depth, Slot width, Tab height | Fit slots to thickness fields | The material thickness the slots were drawn for, the new one, how far off a slot may be, and what to adjust | (old and new thickness, tolerance, what to adjust) | Own | Later. |
| Code type, Encoded text, Convert control codes, Show digits, Keep square, Error correction (Low, Medium, Quartile, High) | Barcode fields | Symbology; the text that will be encoded; turn named control codes into characters; print digits under linear codes; keep Data Matrix square; QR error correction level | (bar code properties) | Own | Error correction values are the QR code standard's level names. Later. |
| Search fonts, Preview size, All fonts, Favorites, Recently used, Single-line, Use font | Font browser controls | Search hint, preview size, the four lists, and the primary button | (font manager) | Own | Later. |
| Trace: Bounding box, Outline; Frame power (%); Frame at ‹p›% | Low-power frame controls | Which path to trace, the power typed fresh each time, and the button that names the power | (laser-on framing option) | Own | "Outline" here is the tight convex outline, its one meaning. The button always shows the power (safety.md rule 2). Later, optional, owner decision. |
| Place the parts that fit | button (Nesting failed) | Places the parts that fit and leaves the others where they are | (none) | Own | Later. |
| Start | History row | First row of History: the project as opened or created | (first undo history entry) | Own | Not a button; reads as a point in time. Later. |

### Layers panel and palette (layers-panel.md)

| LaserCraft name (exact UI label) | Kind | What it is (our words, one line) | Public docs term (provenance only) | Tier (Match/Echo/Own) | Notes (why this name; current label if changed; milestone if future) |
|---|---|---|---|---|---|
| Swatch menu (no visible label): Use for new shapes, Select shapes, Highlight shapes, Layer settings… | context menu | Right-click on a palette swatch | (none in the notes) | Own | Use for new shapes makes the swatch the current color without moving the selection. Later. |
| Layer settings — ‹nn› ‹name› | window title | Title form of the Layer settings window, following the edited layer | Cut Settings Editor | Own | Later. |
| M3 | tag | Small tag after the Speed, power cell when Constant power (M3) is on | (none) | Own | Tooltip second line "Constant power (M3): the laser may stay on while paused." Now (restyle). |
| Double-click a layer to open its settings. | hint | Tooltip on empty space under the layer rows | (none) | Own | Later, with Layer settings. |

### Property bar (property-bar.md)

| LaserCraft name (exact UI label) | Kind | What it is (our words, one line) | Public docs term (provenance only) | Tier (Match/Echo/Own) | Notes (why this name; current label if changed; milestone if future) |
|---|---|---|---|---|---|
| varies | field placeholder | Shown in a field whose selected objects hold different values | (none) | Own | Lowercase, in `text-muted`; not "mixed", which would echo the Mixed layer mode. Later. |
| Not a number: ‹text› | field message | A typed value that cannot be read; the field keeps its old value | (none) | Own | Later. |
| ‹n› shapes are locked | message | An edit refused because some selected shapes are locked | (none) | Own | Later. |
| Dragging to move is off. Turn on Move in the status bar to drag shapes. | hint | Shown when a drag meets a handle switch that is off; the same pattern for Scale ("Dragging to scale is off…"), Rotate and Skew | (none) | Own | Later. |
| Reference point: ‹point› | tooltip | Tooltip on each dot of a 3×3 point grid ("Reference point: top left"); the job grid reads "Job reference point: ‹point›" | (none) | Own | Point words are screen words (top left … bottom right, center). M6 (job grid), later (selection grid). |

### Later windows (later-windows.md)

| LaserCraft name (exact UI label) | Kind | What it is (our words, one line) | Public docs term (provenance only) | Tier (Match/Echo/Own) | Notes (why this name; current label if changed; milestone if future) |
|---|---|---|---|---|---|
| Rows, Columns; Count, Setting (Speed, Power, Interval, Passes), From, To, Box height (mm), Box width (mm), Center Y (mm), Center X (mm) | Speed and power test groups and fields | The two varied settings, their ranges, box sizes and the grid's center for absolute placement | (rows and columns groups) | Own | From and To carry the chosen setting's unit ("From (mm/s)"). M8. |
| Box settings…, Label settings…, Border settings…, Draw labels, Draw border | Speed and power test parts | Open Layer settings for the boxes, labels and border; switch labels and border on or off | (cut settings buttons, label and border switches) | Own | M8. |
| Pattern size ‹w› × ‹h› mm | read-out | Size of the generated test pattern | (output size) | Own | M8. |
| Preset; (none), Built-in, Yours; Diode engraving, Diode cutting | preset dropdown rows | No preset; headings for shipped and user presets; two shipped preset names (own data) | (presets list) | Own | Preset names are our data and may change with M8. "Photo, general" and "Photo, light marks on dark" are Image adjustments' shipped presets. M8 / M7. |
| Save preset, Delete preset? | dialog title + confirmation | Name a preset; confirm removing one ("Delete preset", Cancel) | (none) | Own | M7/M8. |
| Patches, Smallest interval (mm), Largest interval (mm), Patch size (mm); Patch fill: Solid, Gradient | Line interval test fields | Number and size of the patches, the interval range, and how each patch is filled | (samples, interval range, sample size, solid or dithered) | Own | M8. |
| Z from (mm), Z to (mm), Lines, Dashed labels | Focus height test fields | Z range as offsets from the current head height, number of lines, cut labels as dashes | (start and end height, steps, perforated labels) | Own | Read-out "‹d› mm of Z between lines" (not "Line spacing", which is a text field). M8. |
| Original, As engraved | pane captions (Image adjustments) | The source image and the processed result | (before and after panes) | Own | M7. |
| Switch on: All, None, Invert | quick choice (Save adjustment preset) | Turn every switch on, off, or flip them | (none) | Own | M7. |
| Picture taken ‹time›, No picture yet, Picture taken before the head last moved | Camera panel line | Age of the overlay picture | (none) | Own | M9. |
| Use this camera in the overlay | tooltip (Camera header toggle) | Include this camera's picture in the overlay | (none) | Own | M9. |
| Exposure, Focus, White balance, Brightness, Contrast, Hue, Backlight compensation, Auto, Restore defaults | Picture settings rows | Camera controls the camera reports, three with an Auto switch | (camera settings) | Match | Standard USB Video Class control names. M9. |
| Preview rate, Insert picture: Corrected, As captured | Advanced rows (Camera panel) | Live picture rate; put the current picture on the canvas with or without lens correction | (camera advanced settings) | Own | M9. |
| Cameras for ‹device profile› | caption (Camera list) | Heading above the camera list | (none) | Own | M9. |
| What kind of camera?, USB camera, Network camera; Choose the picture source; Where is the camera mounted?, Above the bed, On the laser head; Lens correction: Calibrate now, Load from file…, No lens correction; Camera added | Add camera pages and cards | Page headings and option cards of the Add camera wizard | (new camera wizard pages) | Own | M9. |
| Source, Device, Format, Address, Pictures per second, Status, Live view; Resolution, Rate, Camera controls | Add camera and Camera setup fields | Picture source box, its facts list and the live pane | (camera source settings) | Own | Status values "Connected", "Connecting…", "No picture source". M9. |
| Camera, Source, Lens correction, Bed alignment; Calibrate lens…, Align to bed…; Source details, Correction details, Alignment details; Kind: ‹USB camera / Network camera›, Mounted: ‹above the bed / on the laser head›; Identify device by (Port, Name), Skip first pictures | Camera setup groups and fields | The four groups, their buttons, collapsing sections and fact lines, and the Advanced options | (edit camera window) | Own | Title "Camera setup: ‹name›". M9. |
| Correct the lens, Take pictures of the pattern, Pick a lens model, Lens corrected; Print the pattern, Take pictures, Pick a model; Open the pattern file…; Pictures taken: ‹n› of ‹m›; Standard, Wide angle, Fish-eye, Keep previous; Fit ‹n›%; Tips | Camera lens calibration pages | Page headings, step cards, counter, lens model cards, fit read-out and tips box | (camera calibration wizard) | Own | Pictures are never called frames. M9. |
| How should the marks be found? (Automatically, By clicking); Line up the camera with the bed; Place the pattern, Burn the pattern, Click the marks, Check the result; Pattern on the bed, Pattern size, Mark size (mm), Size (Default, Custom), Pattern width (mm), Pattern height (mm); Find the marks, Next mark; Check the alignment; From materials… | Camera alignment pages | Page headings, step cards, pattern group and buttons | (camera alignment wizard) | Own | The burn page is a job and follows safety.md. "Try again" as in the machine table. M9. |
| Rollers, Chuck; Rotary axis (Instead of Y, A axis); Travel per turn (mm), Roller diameter (mm), Object diameter (mm); Test rotation | Rotary attachment fields | Rotary kind, which axis drives it, its sizes, and a laser-off test turn | (rotary setup) | Own | Test rotation is a jog (safety.md rule 3). M9. |
| Top of design (Top edge, Left edge, Right edge); Correction ‹n›%; Select the art to correct | Taper correction values and lines | Which edge goes on the cup's wide end; how much narrower the narrow end is drawn; hint with nothing selected | (taper warp settings) | Own | M9. |
| Go to expected target 2 | button (Print registration) | Move the head to where the second target should be | (none) | Own | Laser off. M9. |
| Set point 1, Set point 2, Set point 3, Go to center, Add guide circle, Clear points; Center X (mm), Center Y (mm), Diameter ‹d› mm | Find center controls | Store three edge points from the head position, move to the center, add a guide circle | (center finder) | Own | Later. |
| Target layer, No layer selected | Materials target line | Which layer the Materials buttons act on | (none) | Own | M8. |
| No materials yet. Select a layer, then use Save layer as material…. | empty state | A library with no entries | (none) | Own | M8. |
| Follows material ‹description› (‹library›), Stop following | layer editor band + button | A layer tied to a material entry; untie it | (linked material) | Own | M8; proposed change to layers-panel.md and layers.md (owned by later-windows.md). |
| collection; New collection…, Open collection…, Close collection, Collection; Search saved artwork, Thumbnail size; Place on canvas, Import files… | Saved artwork words and controls | A Saved artwork file is a *collection*; its file buttons, the narrow-panel dropdown, the search hint and size slider, and the artwork buttons | Art Library (library file, load, unload, import) | Own | "Library" keeps meaning the material library (one meaning per word). Later. |
| Add to saved artwork | canvas menu item | Store the canvas selection in the current collection | (art library rows of the right-click menu) | Own | Later; answers menus.md question 8 for the label. |

### Menus (menus.md)

| LaserCraft name (exact UI label) | Kind | What it is (our words, one line) | Public docs term (provenance only) | Tier (Match/Echo/Own) | Notes (why this name; current label if changed; milestone if future) |
|---|---|---|---|---|---|
| Clear recent projects | menu item | Last row of Recent projects ▸, under a separator, empties the list | (none in the notes) | Own | Only if the owner wants the row (menus.md question 9). Later. |

## Omitted (out of scope)

DSP and proprietary controllers (README.md, scope):

- Send (button) — transfers a job to a DSP controller without running it; GRBL streams instead. keymap.md binds Alt+Shift+S "send it without starting" for M6: proposed change to keymap.md (mark it not bound).
- Save RD file / Run RD file / Save OUT File / Save UD5 File — proprietary controller file formats.
- File List (window) and its Refresh / Start / Upload / Delete / Calc Time / Delete All Files buttons — files stored on DSP controllers.
- Controller Status Info (dialog) — Ruida-only.
- Feeder Setup — DSP-only. keymap.md lists Ctrl+Shift+F "feeder setup" for M9: proposed change to keymap.md (not planned).
- Min Power "Default" switches beside Speed and Power (DSP layout) — DSP-only.
- U axis (U: read-out, U axis settings, U Offset) — not a GRBL axis in our scope.
- Laser 1 / Laser 2 choice in the scanning offset table, Laser 2 controls — dual-source DSP machines.
- Override PWM Frequency, Dot Mode — DSP-only layer settings.
- Order by Type (device context menu) — every profile is GRBL, so the sort has nothing to sort by.

Galvo (README.md, scope):

- Framing (galvo status text), galvo live-framing window — galvo-only.
- Enable Cylinder Correction, Cylinder Correction Setup — galvo-only.
- Repeat Marking — galvo-only.
- Calibrate Galvo Lens ▸ / 9 Point Correction (Easy) — galvo lens calibration.
- Enable Red Light — galvo-only.
- Split marking (Modes Toolbar button) — galvo-only.
- Galvo device-settings tabs, Calibrate Camera Lens / Calibrate Camera Alignment (galvo menu variant) — galvo-only.

Vendor, network, web and licensing (README.md, scope):

- Vendor Menus — vendor bundle menus.
- Bundles ▸ (Import Bundle(s), Export Bundle) and the bundle dialog — vendor bundle archives.
- Device logo badges, Set Icon (devices context menu) — vendor logos.
- Bridge button (Devices dialog), IP address wizard page, network connection types, Network Port — network bridge and network controllers; LaserCraft connects over USB serial (device.md).
- Web Tools menu, Cuttle, built-in browser window — third-party web tools and embedded browser.
- Nest Selected / Waiting for nesting... — sends parts to an external web service; local nesting is "Nest".
- License Management — licensing UI.
- Check for Updates — tied to the licence period; distribution packages and Flatpak handle updates (revisit at M10).
- Support Forum, Submit Feature Request, PDF Documentation Link, Online Video Tutorials — the vendor's own services; replaced by our "Report a problem" and "User guide".
- CorelDraw macro setup help — third-party design-program integration.
- Camera Selection Help — vendor product chooser that links to product pages.
- Real-Time Statistics — network camera bandwidth.
- Debug Drawing, Convert to Cut (Debug) — the reference's developer tools; ours would be our own design if ever needed.

Clean-room, specs and safety:

- Project-file import warning (Open / Import / Cancel) — no reading of `.lbrn`/`.lbrn2` (clean-room.md).
- Older Camera Settings File (message) — about the reference's own camera file format.
- Print (Black Only) / Print (Keep Colors), Print Settings line width — not planned (keymap.md).
- Use Dark Background (canvas) — conflicts with canvas.md and README.md (themes): the bed stays a light sheet in every theme.
- Shift+click Frame (frame with the laser on) — README.md (safety): no modifier turns the laser on during framing; laser-on framing is only the opt-in "Low-power frame" dialog.
- Laser On When Framing (device setting) — README.md (safety): framing is laser-off by default; low power is chosen per use through "Frame at low power…".
- Enable 'Out of Bounds' Warning (device setting) — the bounds check is always on (CLAUDE.md safety rules), so there is no switch.
- Ignore Out-of-Bounds Shapes if Possible (preference) — a move outside the bed is refused with a clear error, never silently dropped (CLAUDE.md safety rules).
- Accessibility Statement — a policy page, not a UI surface.

## Writing UI text

Everything in this section is own design (tier Own). The reference's wording is
not used as a model; the research notes record only that its toolbar buttons
and fields show tooltips. The rules apply to every string the UI draws,
including text built by the app crate and handed to the shell (job preview
summary, command line log, status messages).

### General voice

- Plain, short, present tense. Say what the thing does, not how it is built.
- Address the user only when an instruction needs it ("Connect to see GRBL
  messages."), never with praise, blame or jokes. No exclamation marks, no
  "Oops", no "simply", no "please" in labels.
- Verbs for things that act, nouns for things that are (Naming rules, Parts of
  speech).
- "Click" for the mouse, "press" for keys, "choose" for menus and dropdowns,
  "select" only for shapes and objects on the canvas, "drag" for pointer moves,
  "type" for fields and the command line.
- Keys are written as they appear in menus: Ctrl+=, Alt+Shift+R, Esc, Del,
  PgUp; on macOS Cmd and Option replace Ctrl and Alt. A chord is only quoted if
  `crates/lasercraft-ui/keymap/default.keymap` binds it; a chord keymap.md lists
  for a later milestone is not shown until that milestone binds it.
- Field labels have no trailing colon. A sentence prompt above a field may end
  in a colon (today's "Path to a .lcproj file:").
- LaserCraft's own vocabulary only (Naming rules, Our model's vocabulary). A
  word in the "Words to avoid" table below never appears in UI text.
- Text is drawn in `type-body`; units, hints and secondary lines in
  `type-small` with `text-muted`; G-code, console lines and coordinate read-outs
  in `type-mono`.

### Tooltips

Tooltips appear on every toolbar button, palette swatch, switch, icon-only
control, status field and field whose label is shortened. Menu items do not get
tooltips except the Window → Panels entries (Naming rules, Toolbars).

| Part | Rule | Example |
|---|---|---|
| First line | What it does, as a verb phrase in sentence case, no final period; for tools "‹Tool name› tool". Then the chord in parentheses if one is bound | "Rectangle tool (Ctrl+R)", "Open the job preview (Alt+P)", "Zoom in (Ctrl+=)" |
| Second line (optional) | One short sentence in `text-muted`, ending in a period: a detail, a limit, or a generic search term for switchers | "Also known as continuous jog.", "Never above the device's Max power (%)." |
| Disabled control | First line as usual; the second line says what is missing and how to get it | "Needs a connected machine.", "Select two or more objects." |
| Field | What the value controls, then its range or rule | "Lowest power used while the head slows in corners. 0–100." |
| Switch | What happens while it is on | "Draw laser-off travel moves." |
| Swatch | "‹nn›: ‹layer name›" (two digits; today one or two), then what a click does | "07: Engrave logo" / "Click to draw new shapes on this layer." |
| Status field | What the read-out means | "Pointer position on the bed" |

Rules:

- At most two lines; never a paragraph. Longer help belongs in the user guide
  (Context help, F1, M10).
- Never quote doc text; describe the behavior in our own words.
- The chord comes from the keymap at run time, never from a string in code
  (keymap.md, Behavior).
- Safety wording is fixed (Naming rules, Safety wording): a low-power frame's
  tooltip always says the laser is on and names its power; laser-off framing
  never says "burn", "fire" or "power".

### Messages and errors

Every message answers three questions, in this order, each as its own short
sentence or clause:

1. **What happened.** Past tense, plain words, naming the object involved.
2. **Why.** The cause if it is known (the OS error, the GRBL code meaning, the
   rule that refused it). Omit if unknown; never guess.
3. **What to do next.** One imperative step the user can take, naming the
   control by its LaserCraft label.

| Severity | Lead-in | Band and text tokens (visual-language.md) | Where it appears |
|---|---|---|---|
| Error that refuses an action | "Cannot ‹verb›: " (today's "Cannot run:" is the model; tests look for it) | `danger-soft` band, `text` with a `danger` mark | where the action was started: job preview status, dialog inline band, Messages field |
| GRBL alarm | "Alarm ‹n›: " | Alarm banner, `danger-soft`, stays until Clear alarm or disconnect | Machine panel and Job bar (machine-panels.md); raw line also in Console |
| GRBL error reply | "GRBL error ‹n›: " | as for an alarm | as for an alarm |
| Warning | "Warning: " (tests look for it) | `warning-soft` band | job preview, dialogs, Messages field |
| Constant-power pause | fixed text "The laser may stay on" (safety.md rule 9) | `text` on `warning-soft` with a `warning` mark (never `warning`-colored text; visual-language.md) | beside "Pause (motion only)" |
| Success or info | no lead-in | `success-soft` or `info-soft`, fades from the Messages field | Messages field |
| Command line result | today's "> ‹invocation›" line, then the message, or "error: ‹message›" | `type-mono` | Command line log |

GRBL alarm and error codes:

- Shown as the number and its meaning, then what LaserCraft did, then the next
  step: "Alarm 1: a hard limit switch was hit during a move. Streaming stopped.
  The position may be lost; home the machine before the next job, then click
  Clear alarm." / "GRBL error 9 on line 412: G-code is locked while the machine
  is in alarm. Streaming stopped. Click Clear alarm once the machine is safe."
- The meaning is our own short wording of GRBL's published alarm and error
  code tables (GRBL v1.1 interface wiki). The lookup table lives with the
  device crate, not in the UI.
- An unknown code reads "Alarm ‹n›: no description is known for this code."
- Streaming never resumes by itself (safety.md rule 10); no message offers
  "Resume" after an alarm or error.
- The Console shows the raw GRBL line unchanged (in `type-mono`) beside our
  wording, so users can search for the exact code.

Fixed message shapes:

| Situation | Message (template) | Rule it follows |
|---|---|---|
| Job outside the work area | "Cannot run: shape "‹name›" goes ‹d› mm past the ‹left/right/top/bottom› edge of the bed (‹X/Y›, screen words as on the canvas). Move it onto the bed or change the work area in the device profile." | safety.md rule 5 (axis, distance outside, shape) |
| Nothing to output | "Nothing to cut: no output layer has shapes." (today's text) | — |
| Serial port refused | "Cannot connect to ‹port›: permission denied. Your user is not in the serial group (‹uucp or dialout›). Run the command shown below, then log out and in again." plus the Serial port access help box | linux-serial.md |
| Stalled controller | "No response from the controller for ‹s› s. Streaming stopped. Check the cable and power, then connect again." | safety.md rule 10 |
| Disconnect during a job | "The connection to ‹port› was lost during the job. Streaming stopped. Check the machine before connecting again." | safety.md rule 10 |
| File error | "Cannot open ‹path›: ‹OS reason›. Check the path and try again." | ui-shell.md (Files) |
| Power clamped | "Warning: layer "‹name›" asks for ‹p›% but the device maximum is ‹m›%. ‹m›% is used." | safety.md rule 6 |
| No selection for Selected shapes only | the No selection message (Dialogs table) | — |
| Jog outside the bed | "Jog refused: it would leave the work area." | safety.md rule 5 |
| Typed value unreadable | "Not a number: ‹text›" | — |
| Edit meets locked shapes | "‹n› shapes are locked" | — |
| Console line that could fire the laser | "— Not sent: this line could turn the laser on. Use a job or framing instead." (appended to the logged line) | safety.md rule 1 |
| Job refused before sending | the job-outside-the-work-area text above, in the Job refused band | safety.md rule 5 |

Rules:

- Never show a Rust error type, a command id or a raw setting value; wrap the
  underlying text as the "why".
- Numbers in messages use the field formats below, with units.
- A message names controls exactly as their label reads, including the
  ellipsis when it opens a dialog ("Use Device profiles… to add one").

### Confirmations

- Ask only before something that cannot be undone or that loses work:
  discarding unsaved changes, writing GRBL settings to the controller, removing
  a device profile, deleting a material library entry or a macro. An undoable
  command never asks (commands.md: undoable commands return their inverse).
- Title: a short question naming the action ("Remove device profile?") or a
  noun for a state ("Unsaved changes", today's title).
- Body: one sentence on the consequence, naming the object ("The profile
  "‹name›" and its settings are removed from this computer.").
- Buttons are verbs that name the result: "Save", "Discard", "Remove profile",
  "Write to controller", with "Cancel" as the way out. Never "Yes", "No" or
  "OK" in a confirmation.
- The destructive button uses the `danger` style; Enter triggers the safe
  choice, Esc is Cancel. Button order and placement: dialogs.md.
- While connected, no confirmation covers or blocks the Job bar (README.md,
  safety; safety.md rule 7).
- "Switch device profile?" offers "Switch to ‹name›" and "Keep ‹current
  name›".
- The confirmations defined so far (Labels adopted from the other style
  files): Replace file? (Replace), Reset GRBL settings? (Reset settings),
  Write GRBL settings? (Write to controller), Discard ‹n› unwritten changes?
  (Discard), Delete macro? (Delete macro), Delete preset? (Delete preset),
  Remove device profile? (Remove profile), Remove camera? (Remove camera),
  each with Cancel.

### Empty states

An empty panel, list or window shows one line in `text-muted` saying what is
missing and how to add it, using the control's exact label. No pictures.

| Place | Text |
|---|---|
| Layers | "Draw a shape to start a layer." (today's text) |
| Console (not connected) | "Not connected. Messages from GRBL appear here once you connect." |
| Machine (not connected) | state "Not connected" and the Connect button; no extra text |
| Jog (not connected) | controls greyed; tooltip "Needs a connected machine." |
| Macros | "No macros yet. Use Edit macros… to add one." |
| Properties | "Select one shape to see its settings." |
| Text data | "Select a text shape whose Text source is not Plain." |
| Materials | "No library is open. Use the Library menu to open or start one." |
| Saved artwork | "Nothing saved yet. Select shapes, then right-click and choose Add selection." |
| Camera | "No camera set up. Use Add camera… to add one." |
| Device profiles | "No device profiles yet. Use Scan for devices… or New profile…." |
| History | "No actions yet." |
| Recent projects ▸ | one greyed item "No recent projects" |
| Keyboard shortcuts search | "No command matches "‹text›"." |
| Job preview | "Nothing to cut: no output layer has shapes." (today's text) |
| Port dropdown | one greyed item "No serial ports found" |
| Measurements | "Point at a shape to measure it." |
| Materials, open library with no entries | "No materials yet. Select a layer, then use Save layer as material…." |
| Camera panel, no picture | "No picture yet" |
| Fields whose selected objects differ | placeholder "varies" |

### Numbers in fields and read-outs

Millimetres and mm/s are the defaults (README.md, units); inches are a later
preference. The unit is in the label, never typed after the number in a field
(Naming rules, Units). Read-outs and messages write the number, a space and
the unit ("12.50 mm"), except "%" and "°", which follow the number without a
space ("20%", "90°").

| Quantity | Unit in label | Decimal places shown | Notes |
|---|---|---|---|
| Position, size, distance, gap, offset | (mm) | 2 | read-outs keep both decimals so the text does not jump |
| Fill interval, kerf, beam compensation, dash lengths | (mm) | 3 | small values; trailing zeros trimmed in fields |
| Speed | (mm/s) | 1 | |
| Acceleration | (mm/s²) | 0 | |
| Power, scale, opacity, spacing | (%) | 1 | power is clamped and shown clamped (safety.md rule 6) |
| Angle | (°) | 2 | |
| Time | (s), (ms), (min) | s: 1, ms and min: 0 | job times in summaries as h:mm:ss |
| Counts (passes, copies, sides, tabs) | none | 0 | |
| Resolution and frequency | (dpi), (lpi) | 0 | |
| Screen distances | (px) | 0 | preferences only (canvas.md) |
| Inches (later) | (in) | 3 | |

- The decimal separator is a full stop; thousands are not grouped in fields
  (open question: locale formats).
- A field accepts typed values with more decimals than it shows and keeps them;
  it only rounds what it draws.
- Negative values show a leading minus; a field that only takes positive values
  refuses a minus.
- Out-of-range input is clamped to the field's range and the field's tooltip
  states the range; power fields clamp to the layer's and the device's maximum.

### Sentence case

Sentence case is a style fix, not a rename (Naming rules). Examples from
today's menus (current-ui.md 1.4):

| Today | Target |
|---|---|
| Save As… | Save as… |
| Project Notes… | Project notes… |
| Invert Selection, Paste in Place, Select All | Invert selection, Paste in place, Select all |
| Edit Nodes, Convert to Path, Close Path, Join Paths | Edit nodes, Convert to path, Close path, Join paths |
| Break Apart (Tools) | Break apart (moves to Arrange) |
| Rotate 90° Clockwise, Mirror Horizontally | Rotate 90° clockwise, Mirror horizontally |
| Forward, Backward, To Front, To Back | Bring forward, Send backward, Bring to front, Send to back (also a rename) |
| Align ▸ Left Edges … Vertical Centers | Align ▸ Left edges … Vertical centers |
| Move on Bed ▸ Top Left | Move on bed ▸ Top left |
| Grid Array…, Circular Array… | Grid array…, Circular array… |
| Zoom In, Zoom Out, Fit Bed | Zoom in, Zoom out, Fit bed |
| Side Panels, Job Preview… | Side panels, Job preview… |

Proper names keep their case: LaserCraft, GRBL, G-code, SVG, DXF, CSV, SHX,
Wi-Fi, Floyd–Steinberg, Bayer. Menu bar titles are single capitalised words.

### Words to avoid

Each row follows from the one-meaning-per-word rules above.

| Avoid | When it means | Use instead |
|---|---|---|
| cut, color | a layer | layer |
| cut | splitting shapes with another shape | Split by shape |
| frame | zooming to the selection | Zoom to selection |
| dock | pushing shapes until they touch | Slide |
| palette | the left tool buttons | Drawing tools |
| console | the LaserCraft command line | Command line |
| outline | the strokes-only view | Show fills (off) |
| lock, unlock | GRBL's alarm lock | Clear alarm |
| step | jog distance, Z per pass, operations, history entries | Jog distance, Lower Z each pass, operation, action |
| mode | job placement, rotary or registration, the reduced interface, GRBL check state | Place job at, setup, layout, Checking |
| gap | the distance at which nearly closed paths are closed | tolerance (Close nearly closed paths…) |
| anchor | the 3×3 point grids | Reference point, Job reference point |
| bitmap | an image object | image |
| optimize | reducing nodes / reordering cuts | Simplify… / Reorder cuts |
| object | a single drawn path inside a group | shape |
| laser-off run-on | unlit moves in the preview | unlit |
| park position | where the head goes after a job | end position |
| Yes, No | confirmation buttons | a verb naming the result, and Cancel |
| Ok | the confirm button | OK |
| abort, kill | stopping a job | Stop |
| any command id or raw value ("shape.move", "floyd_steinberg") | a label | the readable name (Undo move, Floyd–Steinberg) |
| vendor or program names | any option | the number or behavior it stands for |
| library | a Saved artwork file | collection |
| mixed | a field whose selected objects differ | varies |
| frame | a camera picture | picture |

## Naming decisions

Two review passes (clean-room and familiarity) were applied to the draft
names. Every must-fix from both reviews is applied. Should-fix and nit items are applied except as listed
here. Where the two lenses suggested different names, the reason for the choice is given.

Rejected or replaced suggestions:

- Familiarity nit "Variable data" for the Text data panel: rejected. The clean-room must-fix wins
  ("Text data"); "variable" is the distinctive word of the doc's two-word feature name.
- Familiarity nit "Cut priority" for the shape's order field: rejected. It keeps two of the three
  words of the doc caption; the clean-room must-fix "Run order" is used.
- Familiarity nit "one Arrange toolbar with a Show all buttons item": rejected. The clean-room
  should-fix ("Align and distribute", named after its contents) wins on a conflict, and two toolbars
  keep the reference's arrangement (Match tier).
- Familiarity nit "S max ($30)": rejected for the clean-room nit "Full-power S value ($30)"
  (clean-room wins on a conflict). The tooltip mentions "S max" so GRBL users still find it.
- Familiarity should-fix "Close gaps…": rejected for the clean-room's "Close nearly closed paths…",
  because "gap" already names spacing between shapes and copies (Column gap, Gap (mm)).
- Clean-room should-fix wording "Move as one group" and "Keep inner shapes in place": replaced by the
  familiarity wording "Slide as one block" and "Inner shapes follow outer". "Move as one group" is the
  doc's "move as group" with one word added and keeps "move together" ambiguity; "in place" reads as
  if inner shapes stay still while the outer shape moves.
- Familiarity should-fix "H1" and "H2" for the guide swatches: rejected. H is also a G-code word (tool
  length offset, G43 H1), so it has the same clash as G1 and G2. The clean-room option (guide
  pictogram plus 1 and 2) is used.
- Familiarity "consider Lock shapes / Unlock shapes": not adopted. "Clear alarm" removes the clash,
  and plain "Lock"/"Unlock" match the shape property "Locked". The tier is corrected to Echo.
- Clean-room "Anchor to selection": adapted to "Reference point from selection". The familiarity
  should-fix renames the 3×3 grids ("Reference point", "Job reference point") so "anchor" keeps
  arrange.md's meaning; the switch follows the grid's word.
- Familiarity alternative "keep Anchor and rename arrange.md's anchor object to key object": not
  chosen; renaming the grids changes no existing spec.
- Clean-room exact label "Head marker" for the switch: adapted to "Show head marker", so the switch
  names what is on like "Show travel" and "Show legend"; the canvas feature itself is "Head marker".
- Clean-room option "Boolean helper…": rejected. It is the doc's "Boolean Assistant" with one synonym
  swapped. keymap.md's description changes to "combine shapes" instead.
- Familiarity nit "Jog from machine origin": rejected. It keeps the structure of the doc's "Move From
  Machine Zero" with two words swapped. "Absolute jog" is used, and its tooltip says "machine origin"
  as the familiarity review asked.
- Clean-room nit "keep Hold to jog with continuous jog in the tooltip": the other offered option,
  "Continuous jog", is used because it is the generic CNC term people search for.
- Familiarity nit "Bend letters with arc": the clean-room should-fix "Bend letters along arc" is used
  (should-fix over nit; same meaning).
- Familiarity nit "Halftone cell size (mm)": the other offered option, "Halftone frequency (lpi)", is
  used because layers.md already accepts lpi.
- Familiarity nit "Command box": not adopted; the review made it conditional on confusion showing up
  in the user guide. The spec changes it listed are applied.
- Familiarity should-fix "fold add, edit and remove into the Camera panel": the other offered option
  (rename the dialog "Camera list") is used, which keeps the reference's arrangement.
- Familiarity should-fix wording "Select shapes on current layer", "Select shapes inside selection",
  "Select shapes smaller than selection": partly replaced. The last keeps the doc caption "Select
  Shapes Smaller Than Selected" with one word changed (a clean-room copy), and both inside and smaller
  items add to the selection, so they read "Add enclosed shapes to selection" and "Add smaller shapes
  to selection"; the layer item reads "Select the current layer's shapes". Each now names its
  reference, which was the point of the issue.
- Clean-room option "Sort: fills first, lines last": the shared idea is kept, worded as the
  familiarity verb form "Put line layers last" because menu items that act are verbs.
- Clean-room option "Window box / Crossing box": "Inside box" and "Touching box" are used, which
  match canvas.md's "contained" and "touched".
- Clean-room option "rename verb-phrase dialogs to nouns": the other option is used. The rule is
  restated as the familiarity review proposed (a dialog title repeats its command), so no titles
  change for that reason.
- Clean-room example "Duplicate with links": worded "Duplicate with linked copies" so it says what is
  copied (the original together with its copies) in our own vocabulary.

Decisions made while turning the draft name list into this spec (same rules):

- "Check mode" (machine state) → "Checking": "mode" only means a layer mode,
  and the other state words are participles or adjectives (Running, Homing).
- "Edge mode (sketches)" (Trace to vectors) → "Follow edges (sketches)", for the
  same reason.
- "Outlines" (Convert to image, Draw as) → "Strokes only", because "outline"
  only means the tight convex outline; "Layer mode" → "By layer mode" so the
  value reads as a choice.
- Camera calibration descriptions say "picture", not "frame", because "frame"
  only means tracing the job with the head.
- Simple layout's kind is "layout (setting)", not "mode".
- Stop lists its legacy chord Ctrl+Break from keymap.md, and its look names the
  `danger` token instead of a color word.
- References to the binding style decisions now point to README.md by topic
  (scope, safety, menus, docking, palette, themes, units, …).

Decisions made while adopting the other style files' proposals (same
rules; the adopted labels are in "Labels adopted from the other style files"):

- dialogs.md's "Circle centre (mm)" → "Circle center (mm)", the spelling of
  every other label (Center X (mm), Go to center).
- dialogs.md's screen-distance unit "(screen points)" for Snap distance and
  Click tolerance: not adopted. The labels keep "(px)" as in the Grid and
  snapping row, because canvas.md (an existing spec, which wins) gives those
  distances in px. egui's logical points are what the code measures; the
  label does not need to say so. Proposed change to dialogs.md.
- dialogs.md's "Keep the switch on" in place of No selection's "OK": not
  adopted. No selection is a notice, not a confirmation: neither button
  loses work or runs anything, so the Confirmations rule against "OK" does
  not apply, and machine-panels.md already draws "OK" and "Use all shapes".
  Proposed change to dialogs.md; open question 19 if the owner prefers the
  verb.
- visual-language.md's placeholder "mixed": replaced by property-bar.md's
  "varies", because "Mixed" is a layer mode (one meaning per word).
  Proposed change to visual-language.md.
- later-windows.md's Focus height test read-out "Line spacing ‹d› mm of Z"
  → "‹d› mm of Z between lines": "Line spacing (%)" is a text field.
  Proposed change to later-windows.md.
- canvas.md's "curve handles" adopted over the draft "Control handles", so
  one word names them on screen and in prose.
- The draft caption "Scan shift by speed" is dropped instead of being added
  to machine-panels.md: the switch label and the two column headers already
  say what the table holds.
- later-windows.md's word "collection" for a Saved artwork file is adopted
  (one meaning per word: "library" is the material library); it is added to
  Words to avoid below.
- "Job refused" is a prose name for the band (its visible text is the
  message), like the other "(no visible label)" rows.
- "Try again" is used for both the Serial port access help box and the
  Camera alignment wizard: both repeat the step that failed.

## Current state → target

From current-ui.md (audit at commit 151eb26). Paths are relative to the repo.

- **Title Case everywhere** in the menus (`crates/lasercraft-ui/src/shell.rs:432-519`):
  becomes sentence case (Sentence case table above).
- **Undo shows the raw command id** ("Undo shape.move",
  `shell.rs:447-472`): becomes "Undo ‹readable action›" ("Undo move"). Needs a
  readable name per command in the registry (lasercraft-commands), not a string
  table in the UI.
- **Chord display** uses egui key names ("Ctrl+Equals", "Ctrl+Backtick",
  "Period", "PageUp") and "Ctrl+" on macOS (`Chord`'s Display, keymap.rs
  ~70-80): becomes symbols (Ctrl+=, Ctrl+`, `.`, PgUp) and Cmd/Option on macOS.
- **Drawing order items** "Forward", "Backward", "To Front", "To Back"
  (`shell.rs:40-117`): become Bring forward, Send backward, Bring to front,
  Send to back.
- **Break Apart** sits in Tools (`shell.rs:17-37`): moves to Arrange as
  "Break apart".
- **Tool column** labels (`shell.rs:537-554`): "Edit Nodes" → "Edit nodes";
  tooltip form "‹label› tool (‹chord›)" stays.
- **Layer list summary** shows raw lowercase mode and units in one string
  ("line 10 mm/s 20%", `layers_panel.rs:118-148`): becomes the Speed, power
  column with "Line"/"Fill" in the Mode column; the list gains column headers
  Color, Name, Mode, Speed, power, Output, Show (Air later).
- **Mode combo** shows "line"/"fill" (`layers_panel.rs:162-271`): shows Line,
  Fill.
- **Unit suffixes** inside fields (" mm/s", " %", " mm",
  `layers_panel.rs:358-388`): units move into the labels: Speed (mm/s), Max
  power (%), Min power (%), Kerf offset (mm), Tab width (mm), Interval (mm),
  Scan angle (°).
- **Image method combo** shows raw ids ("floyd_steinberg",
  `layers_panel.rs:272-314`): shows Grayscale, Threshold, Floyd–Steinberg,
  Jarvis, Stucki, Ordered (Bayer), Halftone.
- **Constant power** is in the model but missing from the editor
  (`crates/lasercraft-project/src/layer.rs`, Layer::new): add the switch
  "Constant power (M3)" (safety.md rule 9).
- **Offset dialog** (`crates/lasercraft-ui/src/dialogs.rs:147-156`): "Distance
  (negative: inward)" → "Distance (mm)" with hint "Negative moves inward";
  corner names "round / miter / bevel" → Round, Miter, Bevel.
- **Grid array dialog** (`dialogs.rs:157-162`): "Gap between columns", "Gap
  between rows" → "Column gap (mm)", "Row gap (mm)".
- **Circular array dialog** (`dialogs.rs:163-176`): "Count (with the
  original)" → "Total count" (hint "Includes the original"); "Center x",
  "Center y" → "Center X (mm)", "Center Y (mm)"; "Sweep" → "Sweep (°)"; "Turn
  the copies with the circle" → "Rotate copies".
- **Job preview** (`crates/lasercraft-ui/src/preview.rs:134-233`, text from
  `crates/lasercraft-app/src/preview.rs:41-69`): "laser-off run-on" → "unlit";
  slider "time" with suffix " s" → "Time (s)"; button "Whole job" → "Show whole
  job". "Job preview", "Cannot run:" and "Warning:" stay (tests in
  `crates/lasercraft-ui/tests/shell.rs`).
- **Command line** is called "Console" in ui-shell.md, keymap.md:121 and
  import.md:29 (it has no visible title, `shell.rs:556-579`): it becomes the
  Command line; "Console" is kept for the M6 GRBL console.
- **Unsaved changes** modal has "Discard" and "Cancel"
  (`crates/lasercraft-app/src/main.rs:185-210`): gains "Save" if the owner
  agrees; body text follows the Confirmations rules.
- **Window title** "• ‹name› — LaserCraft" (`main.rs:212-215`): unchanged until
  M6/M8 add the device profile.
- **Not present yet**: Window, Machine and Help menus; toolbar band; every M6+
  label. They use the names in the tables when built.

## Proposed changes to other specs

Each spec is updated before the code that follows it (README.md, existing specs
win).

- **Proposed change to ui-shell.md**: "tool palette" → Drawing tools; the
  bottom "Console" → Command line (collapsed to its input line by default, per
  layout.md); sentence case for every label it spells in Title Case (Save As…,
  Paste in Place, Fit Bed, Edit Nodes, Side Panels, Job Preview…, Project
  Notes…, Grid Array…, Circular Array…, Move on Bed); Unsaved changes gains a
  Save button (owner decision); drawing-order items become verb phrases.
- **Proposed change to keymap.md** (action descriptions only; chords
  unchanged): Alt+Shift+C "cut shapes" → split by shape; Alt+Shift+O
  "optimize" → simplify; Ctrl+B "boolean helper" → combine shapes;
  Ctrl+Shift+B "convert to bitmap" → convert to image; Alt+Shift+W "toggle
  outline / filled display" → show or hide fills; Alt+Shift+H/V "move
  together" → pack horizontally / vertically; Alt+0 … Alt+4 "print-and-cut
  steps" → print registration steps; keymap.md:121 "the console" → the command
  line; Alt+Shift+S ("send it without starting") and Ctrl+Shift+F ("feeder
  setup") marked not bound (out of scope); a "Chord display" rule (symbols,
  Cmd/Option on macOS).
- **Proposed change to import.md**: import.md:29 "the console" → the command
  line; DXF units, SVG resolution and tolerances become preferences.
- **Proposed change to preview.md**: sentence case "Job preview…"; summary
  wording "unlit" instead of "laser-off run-on"; control labels Time (s), Show
  whole job, Show travel.
- **Proposed change to canvas.md and nodes.md**: sentence case "Edit nodes".
- **Proposed change to text.md**: editable text objects (Text style toolbar
  beyond next-text defaults, Text data, text properties, Arc handle, Put text
  on path, Double-click edits text); sentence case. Owner decision.
- **Proposed change to arrange.md**: sliding (Slide ▸, Slide and nest options)
  and linked copies, both excluded by design today; Match width / Match height.
- **Proposed change to layers.md**: Concentric fill mode, Operations and the
  Mixed mode, guide layers (guide swatches), extra tab settings.
- **Proposed change to shapes.md**: polygon, star and alternating-star shapes,
  rectangle corner radius, barcodes.
- **Proposed change to planning.md**: the Cut order dialog's rules and switches
  beyond today's fixed order (Reorder cuts).
- **Proposed change to gcode.md**: "After the job" (Stay, Go to origin, Go to
  end position) replaces the on/off finish option; G-code tab switches that
  change output.
- **Proposed change to dialogs.md**: keep "OK" and "Use all shapes" on No
  selection (instead of "Keep the switch on"); write Snap distance and Click
  tolerance as "(px)" (instead of "(screen points)"); spell "Circle center
  (mm)"; its other proposed labels are adopted as worded.
- **Proposed change to visual-language.md**: the placeholder for a field
  whose selected objects differ is "varies", not "mixed"; the constant-power
  pause note is now worded here as `text` on `warning-soft` with a `warning`
  mark, as visual-language.md asked. Its other proposed labels
  (Restore default sizes, Planning…, Reference point: ‹point›, Job refused)
  are adopted.
- **Proposed change to later-windows.md**: the Focus height test read-out
  reads "‹d› mm of Z between lines"; its other proposed labels and the word
  "collection" are adopted.
- **Proposed change to canvas.md**: the node-editing table rows "Control
  handle line" and "Control handle end" become "Curve handle line" and
  "Curve handle end" (prose names). The Tab markers meaning it proposed is
  adopted.
- **Proposed change to toolbars.md, machine-panels.md, layers-panel.md,
  property-bar.md and menus.md**: their proposed labels are adopted as
  worded; each file can drop the "until naming.md lists them" note.
- **Proposed change to device.md**: the machine state words of the Major
  controls table (including "Checking" for GRBL's check state and "No
  response" for the stalled state) and the GRBL alarm and error message form
  of Writing UI text.

## Open questions

1. Unsaved changes: add a "Save" button beside Discard and Cancel?
2. Ctrl+Shift+A: keymap.md binds it to laser framing of the selection (legacy
   page, M6) while the 2.1 Window-menu page shows it on zoom to selection.
   Which does LaserCraft keep?
3. The laser framing of the selection (keymap.md, Ctrl+Shift+A, M6) has no
   label in the tables. Proposed own label "Frame selected shapes" in the
   Machine menu, or is it covered by Frame with Selected shapes only on?
4. CLAUDE.md's M9 says "print-and-cut"; align it with "Print registration"?
5. Editable text objects (Text data, text switches, Arc handle, Put text on
   path): build them, and when?
6. Operations and the Mixed layer mode: adopt them?
7. Guide layers and their two guide swatches: adopt them?
8. The two Start guards in General preferences: switches, or fixed behavior?
9. Streaming: may the user force Send and wait, or is it only a fallback?
10. Where does Clear alarm sit: Alarm banner only, or also the Machine menu and
    Console?
11. Auto focus: in scope at all (not standard GRBL)?
12. Test fire: anything beyond a placeholder (README.md, safety)?
13. Frame at low power…: offer it at all (safety.md rule 2)?
14. Locale number formats (decimal comma, grouping) once translations exist,
    and whether a Language menu follows.
15. "Command line" vs "Command box": revisit if the user guide shows confusion
    with the Console.
16. Material libraries: where the files live and whether "Close library"
    keeps them listed for the next run.
17. Should Help get "Check for updates" at M10 for builds outside distribution
    packages?
18. Note for the lead author: README.md (decision 15 on doc terms) allows doc
    terms only in this file's mapping column and in Sources; the Omitted list
    is covered by decision 1. This file also quotes doc captions in the Naming
    decisions (to show why a draft was a copy) and states that wider rule in
    "Why doc terms appear here" and the Naming rules. Keep this exception, or
    should the Naming decisions point to the mapping-column row instead of
    quoting the caption?

19. No selection: keep "OK" beside "Use all shapes" (this file), or use a
    verb such as "Keep the switch on" (dialogs.md's proposal)?
20. Recent projects ▸: add "Clear recent projects" (menus.md question 9)?
21. Built-in preset names (Diode engraving, Diode cutting, Photo, general,
    Photo, light marks on dark) are data: confirm them with the M7/M8 specs.

## Sources

All LightBurn pages are part of the public 2.1 documentation and were read
2026-10-21 through the style guide's research notes. They support the "Public
docs term" column and the Omitted list only; no wording is taken from them.

1. [R1] https://docs.lightburnsoftware.com/2.1/Reference/UI/ — window map: names of the main regions
2. [R2] https://docs.lightburnsoftware.com/2.1/GetStarted/UITour/ — main window, toolbar and panel names
3. [R3] https://docs.lightburnsoftware.com/2.1/Guides/CustomizingTheLightBurnWindow/ — docking, floating, reset layout, title strips and tab strips
4. [R4] https://docs.lightburnsoftware.com/2.1/Reference/UI/MenuToolbar/ — menu bar
5. [R5] https://docs.lightburnsoftware.com/2.1/Reference/UI/FileMenu/ — File menu items
6. [R6] https://docs.lightburnsoftware.com/2.1/Reference/UI/EditMenu/ — Edit menu items
7. [R7] https://docs.lightburnsoftware.com/2.1/Reference/UI/ToolsMenu/ — Tools menu items
8. [R8] https://docs.lightburnsoftware.com/2.1/Reference/UI/ArrangeMenu/ — Arrange menu items
9. [R9] https://docs.lightburnsoftware.com/2.1/Reference/UI/LaserToolsMenu/ — machine-tools menu items
10. [R10] https://docs.lightburnsoftware.com/2.1/Reference/UI/WindowMenu/ — Window menu, view style and zoom items
11. [R11] https://docs.lightburnsoftware.com/2.1/Reference/UI/HelpMenu/ — Help menu items
12. [R12] https://docs.lightburnsoftware.com/2.1/Reference/UI/LanguageMenu/ — Language menu
13. [R13] https://docs.lightburnsoftware.com/2.1/Reference/UI/MainToolbar/ — main toolbar contents
14. [R14] https://docs.lightburnsoftware.com/2.1/Reference/UI/ArrangeToolbar/ — arrange toolbar and its long variant
15. [R15] https://docs.lightburnsoftware.com/2.1/Reference/UI/CreationToolbar/ — left tool toolbar
16. [R16] https://docs.lightburnsoftware.com/2.1/Reference/UI/ModifiersToolbar/ — shape-operation toolbar
17. [R17] https://docs.lightburnsoftware.com/2.1/Reference/UI/ModesToolbar/ — special-setups toolbar
18. [R18] https://docs.lightburnsoftware.com/2.1/Reference/NumericEditsToolbar/ — position and size fields, 3×3 grid
19. [R19] https://docs.lightburnsoftware.com/2.1/Reference/Text/ — text toolbar, text modes and text properties
20. [R20] https://docs.lightburnsoftware.com/2.1/Reference/Dock/ — docking (sliding) toolbar, its direction buttons and options
21. [R21] https://docs.lightburnsoftware.com/2.1/Reference/UI/ColorPalette/ — palette strip and tool layers
22. [R22] https://docs.lightburnsoftware.com/2.1/Reference/UI/StatusBar/ — status bar fields and transform toggles
23. [R23] https://docs.lightburnsoftware.com/2.1/Reference/UI/Tooltips/ — tooltips
24. [R24] https://docs.lightburnsoftware.com/2.1/Reference/UI/Workspace/ — canvas, rulers, selection indicators
25. [R25] https://docs.lightburnsoftware.com/2.1/Reference/CutsLayersWindow/ — layer list, columns, context menus
26. [R26] https://docs.lightburnsoftware.com/2.1/Reference/CutSettingsEditor/ — layer settings window, tabs; with its LineMode, FillMode, OffsetFillMode, ImageMode, SharedSettings, SubLayers and WorkingWithDefaults pages
27. [R27] https://docs.lightburnsoftware.com/2.1/Reference/LaserWindow/ — machine panel controls
28. [R28] https://docs.lightburnsoftware.com/2.1/Reference/MoveWindow/ — jog panel controls, saved positions
29. [R29] https://docs.lightburnsoftware.com/2.1/Reference/ConsoleWindow/ — console panel
30. [R30] https://docs.lightburnsoftware.com/2.1/Reference/MacrosWindow/ — macros panel
31. [R31] https://docs.lightburnsoftware.com/2.1/Reference/FileListWindow/ — controller file list (omitted)
32. [R32] https://docs.lightburnsoftware.com/2.1/Reference/MachineSettings/ — controller settings dialog and backups
33. [R33] https://docs.lightburnsoftware.com/2.1/Reference/GetControllerInfo/ — controller status info (omitted)
34. [R34] https://docs.lightburnsoftware.com/2.1/Reference/Devices/ — device manager buttons and context menu
35. [R35] https://docs.lightburnsoftware.com/2.1/Reference/DeviceSettings/ — device settings tabs and fields (with its subpages)
36. [R36] https://docs.lightburnsoftware.com/2.1/GetStarted/FindMyLaser/ — device discovery wizard
37. [R37] https://docs.lightburnsoftware.com/2.1/GetStarted/CreateManually/ — manual device wizard
38. [R38] https://docs.lightburnsoftware.com/2.1/Reference/CoordinatesOrigin/ — job placement choices and job origin grid
39. [R39] https://docs.lightburnsoftware.com/2.1/Reference/CutSelectedGraphics/ — selected-only output, selection origin, nothing-selected message
40. [R40] https://docs.lightburnsoftware.com/2.1/GetStarted/JobControl/ — start, pause, stop, frame
41. [R41] https://docs.lightburnsoftware.com/2.1/Reference/Preview/ — preview window controls
42. [R42] https://docs.lightburnsoftware.com/2.1/Reference/OptimizationSettings/ — cut ordering settings
43. [R43] https://docs.lightburnsoftware.com/2.1/Reference/MaterialLibrary/ — material library panel
44. [R44] https://docs.lightburnsoftware.com/2.1/Reference/MaterialTest/ — material test generator
45. [R45] https://docs.lightburnsoftware.com/2.1/Reference/IntervalTest/ — interval test
46. [R46] https://docs.lightburnsoftware.com/2.1/Reference/FocusTest/ — focus test and its Z-axis message
47. [R47] https://docs.lightburnsoftware.com/2.1/Reference/ArtLibrary/ — art library panel
48. [R48] https://docs.lightburnsoftware.com/2.1/Reference/Cameras/ — camera panel, camera list and wizards (with its subpages)
49. [R49] https://docs.lightburnsoftware.com/2.1/Reference/AdjustImage/ — image adjustment window
50. [R50] https://docs.lightburnsoftware.com/2.1/Reference/TraceImage/ — trace window
51. [R51] https://docs.lightburnsoftware.com/2.1/Reference/ConvertToBitmap/ — convert-to-image dialog
52. [R52] https://docs.lightburnsoftware.com/2.1/Reference/PrintAndCut/ — print registration wizard and items
53. [R53] https://docs.lightburnsoftware.com/2.1/Reference/GridArray/ — grid array dialog, virtual array switch
54. [R54] https://docs.lightburnsoftware.com/2.1/Reference/CircularArray/ — circular array dialog
55. [R55] https://docs.lightburnsoftware.com/2.1/Reference/OffsetShapes/ — offset dialog
56. [R56] https://docs.lightburnsoftware.com/2.1/Reference/CopyAlongPath/ — copies along a path
57. [R57] https://docs.lightburnsoftware.com/2.1/Reference/BarCode/ — barcode dialog and tabs
58. [R58] https://docs.lightburnsoftware.com/2.1/Reference/ResizeSlots/ — slot resizing dialog
59. [R59] https://docs.lightburnsoftware.com/2.1/Reference/QuickNest/ — local nesting and its failure message
60. [R60] https://docs.lightburnsoftware.com/2.1/Reference/NestSelected/ — web nesting (omitted)
61. [R61] https://docs.lightburnsoftware.com/2.1/Reference/TransformControls/ — transform handles and toggles
62. [R62] https://docs.lightburnsoftware.com/2.1/Reference/Snapping/ — snapping points and switches
63. [R63] https://docs.lightburnsoftware.com/2.1/Reference/ViewStyle/ — wireframe/filled and smooth view styles
64. [R64] https://docs.lightburnsoftware.com/2.1/Reference/ZoomPan/ — zoom and pan
65. [R65] https://docs.lightburnsoftware.com/2.1/Reference/BeginnerMode/ — simplified interface
66. [R66] https://docs.lightburnsoftware.com/2.1/Reference/UndoRedo/ — undo, redo and history
67. [R67] https://docs.lightburnsoftware.com/2.1/Reference/SettingsPreferences/ — preferences tabs and settings
68. [R68] https://docs.lightburnsoftware.com/2.1/Reference/ManagingPreferences/ — preferences export, import, backups
69. [R69] https://docs.lightburnsoftware.com/2.1/Reference/EditHotkeys/ — shortcut editor
70. [R70] https://docs.lightburnsoftware.com/2.1/Reference/HelpAndNotes/ — help and notes window
71. [R71] https://docs.lightburnsoftware.com/2.1/Reference/ShowNotes/ — project notes
72. [R72] https://docs.lightburnsoftware.com/2.1/Reference/UserBundles/ — vendor bundles (omitted)
73. [R73] https://docs.lightburnsoftware.com/2.1/Reference/FeederSetup/ — feeder setup (omitted)
74. [R74] https://docs.lightburnsoftware.com/2.1/Reference/CalibrateGalvoLens/ — galvo lens calibration (omitted)
75. [R75] https://docs.lightburnsoftware.com/2.1/NewFeatures/NewIn2.1/ — features new in 2.1 (multi-file trace, tool layers and others)
76. [R76] https://docs.lightburnsoftware.com/2.1/Reference/FileManagement/ — new, open and save prompts, the device-switch prompt, the project-file import warning
77. [R77] https://docs.lightburnsoftware.com/2.1/Reference/VariableText/ — text that changes per job: counter fields, baking, auto-advance
78. [R78] https://docs.lightburnsoftware.com/2.1/Reference/TaperWarp/ — taper correction terms
79. [R79] https://docs.lightburnsoftware.com/2.1/Reference/AddTabs/ — tab placement and tab settings
80. [R80] https://docs.lightburnsoftware.com/2.1/Reference/PrimaryShapes/ — built-in shape list, including the two-radius star
81. [R81] https://docs.lightburnsoftware.com/2.1/Reference/EditNodes/ — node editing actions and their toolbar
82. [R82] https://docs.lightburnsoftware.com/2.1/Reference/ShapeProperties/ — per-shape properties panel fields
83. [R83] https://docs.lightburnsoftware.com/2.1/Reference/AutomaticGuidelines/ — automatic and draggable guidelines
84. [R84] https://docs.lightburnsoftware.com/2.1/Reference/Selection/ — selection kinds and selection helpers
85. [R85] https://docs.lightburnsoftware.com/2.1/GetStarted/SelectingMovingSizing/ — enclosing and crossing selection boxes
86. [R86] https://docs.lightburnsoftware.com/2.1/Reference/Distribute/ — distribute items, their toolbar buttons and the move-together pair
87. [R87] https://docs.lightburnsoftware.com/2.1/Licensing/AccessibilityStatement/ — accessibility statement (omitted)
88. [R88] https://docs.lightburnsoftware.com/2.1/Reference/CenterFinder/ — center finding helper
89. [R89] https://docs.lightburnsoftware.com/2.1/Reference/Measure/ — measure tool
90. [R90] https://docs.lightburnsoftware.com/2.1/Reference/LockShapes/ — locking and unlocking shapes
91. [R91] https://docs.lightburnsoftware.com/2.1/Reference/MakeSameWidthHeight/ — same width and same height actions
92. [R92] https://docs.lightburnsoftware.com/2.1/Reference/TrimShapes/ — trim tool
93. [R93] https://docs.lightburnsoftware.com/2.1/Reference/TwoPointRotateScale/ — two-point rotate and scale
94. [R94] https://docs.lightburnsoftware.com/2.1/Reference/WarpDeform/ — four-point and sixteen-point warps
95. [R95] https://docs.lightburnsoftware.com/2.1/Reference/TangentCircleGenerator/ — tangent circle action
96. [R96] https://docs.lightburnsoftware.com/2.1/Reference/RadiusFillet/ — rounded corners
97. [R97] https://docs.lightburnsoftware.com/2.1/Reference/SetStartPoint/ — start point tool
98. [R98] https://docs.lightburnsoftware.com/2.1/Reference/SaveProcessedBitmap/ — saving a dithered image
99. [R99] https://docs.lightburnsoftware.com/2.1/Reference/ApplyMaskToImage/ — clipping an image with a shape
100. [R100] https://docs.lightburnsoftware.com/2.1/Reference/MultiFileTraceImage/ — tracing several files
101. [R101] https://docs.lightburnsoftware.com/2.1/Reference/BooleanTools/ — union, subtract, intersect, weld and the boolean helper window
102. [R102] https://docs.lightburnsoftware.com/2.1/Reference/Print/ — printing items (omitted)
103. [R103] https://docs.lightburnsoftware.com/2.1/Reference/DrawOrder/ — drawing order items
104. [R104] https://docs.lightburnsoftware.com/2.1/Reference/ImageOptions/ — image refresh and replace items
105. [R105] https://docs.lightburnsoftware.com/2.1/Reference/NewWindow/ — second window
106. [R106] https://docs.lightburnsoftware.com/2.1/Reference/ResetToDefaultLayout/ — resetting the window layout
107. [R107] https://docs.lightburnsoftware.com/2.1/Reference/CreateRubberBandOutline/ — outline around the selection
108. [R108] https://docs.lightburnsoftware.com/2.1/Reference/MoveLaserToSelection/ — sending the head to a point of the selection
109. [R109] https://docs.lightburnsoftware.com/2.1/Reference/MoveSelectedObjects/ — moving the selection to bed positions
110. [R110] https://docs.lightburnsoftware.com/2.1/Reference/PositionLaser/ — head positioning tool
111. [R111] https://docs.lightburnsoftware.com/2.1/Reference/Align/ — align items, the align toolbar buttons and their families
112. [R112] https://docs.lightburnsoftware.com/2.1/Reference/Grouping/ — group, ungroup and auto-group
113. [R113] https://docs.lightburnsoftware.com/2.1/Reference/FlipMirror/ — flip and mirror items
114. [R114] https://docs.lightburnsoftware.com/2.1/Reference/Clipboard/ — clipboard items
115. [R115] https://docs.lightburnsoftware.com/2.1/Reference/DrawLines/ — line tool
116. [R116] https://docs.lightburnsoftware.com/2.1/Reference/ApplyPathToText/ — text on a path
117. [R117] https://docs.lightburnsoftware.com/2.1/GetStarted/FramingBeginner/ — framing, including the small dialog that ends continuous framing (End framing row)

Open standards:

- GRBL v1.1 wiki: machine states, alarm and error code tables, alarm unlock
  (`$X`), check mode (`$C`), real-time commands, feed and spindle overrides,
  streaming protocols (character counting, send-response), `$30` maximum S
  value, `$100`–`$102` steps per mm, `$110`–`$121` speeds and accelerations,
  constant (`M3`) vs dynamic (`M4`) power —
  https://github.com/gnea/grbl/wiki/Grbl-v1.1-Interface,
  https://github.com/gnea/grbl/wiki/Grbl-v1.1-Commands,
  https://github.com/gnea/grbl/wiki/Grbl-v1.1-Configuration,
  https://github.com/gnea/grbl/wiki/Grbl-v1.1-Laser-Mode.
- Image method names: the published dithering algorithms (Floyd–Steinberg,
  Jarvis–Judice–Ninke, Stucki, Bayer ordered dithering) as listed in CLAUDE.md
  M7; halftone frequency in lines per inch (lpi) as in `docs/spec/layers.md`.

Existing specs and project files used: `docs/clean-room.md`,
`docs/spec/safety.md`, `docs/spec/ui-shell.md`, `docs/spec/keymap.md`,
`crates/lasercraft-ui/keymap/default.keymap`, `docs/spec/layers.md`,
`docs/spec/arrange.md`, `docs/spec/text.md`, `docs/spec/canvas.md`,
`docs/spec/preview.md`, `docs/spec/planning.md`, `docs/spec/gcode.md`,
`docs/spec/device.md`, `docs/spec/import.md`, `docs/spec/shapes.md`,
`docs/spec/nodes.md`, `docs/linux-serial.md`, CLAUDE.md, and the audit of
today's UI in the style guide's research notes.

Own design: every LaserCraft name in the tables, the naming rules and the
naming tiers, the Writing UI text section (tooltip, message, confirmation and
empty-state patterns, number formats, sentence-case and words-to-avoid tables)
and the wording of every example message.
