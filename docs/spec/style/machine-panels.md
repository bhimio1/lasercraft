# Spec: style — machine panel, job bar, jog, console, macros and device dialogs

Milestones: M6 (Machine panel, job bar buttons, machine states, alarm banner,
Serial port access help, Jog panel core, Console panel, Edit device
profile… with work area, port and baud, the scan routine behind the
"Automatic" port), M8 (Device profiles dialog, Device scan and New device
profile wizards, all Device profile tabs, the Device dropdown with several
profiles, Before you start), M9 (Use rotary, rotary jog arrows); Macros,
Continuous jog, Absolute jog, Set end position, saved head positions, speed
and power overrides, Run G-code file…, Cut order…, GRBL settings and its
sub-dialogs are later (some are owner decisions, marked as such).

This file fixes what the machine-facing parts of LaserCraft contain, in which
order, how each part looks (through tokens only), when each control is
enabled, what it says and what it does: the Machine panel, the buttons of the
job bar, the Jog, Console and Macros panels, and the device dialogs and
wizards. Where these panels sit by default, the job bar's place in toolbar
row 1 and the rule that it stays visible over everything are `layout.md`'s;
every label comes from `naming.md`; colors, type and sizes are the tokens
defined in `visual-language.md`; icon artwork is `icons.md`'s (this file only
names each icon's concept); the Machine menu rows are in `menus.md`; the
Move head tool is in `toolbars.md`; the head marker, job origin marker and
frame path drawn on the canvas are `canvas.md`'s; dialog frames, button
order, wizard chrome and the "no modal over the job bar" mechanics are
`dialogs.md`'s; the Materials panel (the second tab of the lower dock group)
is in `later-windows.md`; the job preview window is in `job-preview.md`.
`README.md` (the style guide's index, pending: it is not yet in the repo)
will explain the tiers (Match, Echo, Own); until it lands, the "How to read
this file" section below and `visual-language.md` use them as defined there.
The behavior rules stay in
`docs/spec/safety.md`, `docs/spec/device.md` and `docs/spec/gcode.md`, which
win wherever they already decide something.

## How to read this file

- Every element has a **Reference** part (what the public docs show, each
  fact tagged with a source key such as [R1], listed under Sources), a
  **LaserCraft** part (the binding spec) and a **Tier** line.
- Milestone tags: *Now* = exists today; *M6*–*M10* as in `CLAUDE.md`;
  *Later* = no milestone yet; *owner decision* = needs the owner before it is
  built.
- Every button, switch and field invokes a registry command or a `ui.*`
  action (CLAUDE.md, architecture rules). Machine actions are commands that
  are not undoable (they change the machine, not the project); settings that
  are saved in the project or a device profile are undoable commands.
- "Connected" means from the moment Connect is pressed until the connection
  is closed or lost (the time the job bar is shown, `layout.md`).
- "The job" means what Start would send now: every output layer's shapes, or
  only the selected shapes while Selected shapes only is on.
- Shortcuts are quoted from `docs/spec/keymap.md`. A chord that keymap.md
  lists for M6 is not yet in `crates/lasercraft-ui/keymap/default.keymap`;
  this file says "(M6)" after it. A chord keymap.md does not list is "not
  bound".

---

## Shared look of this file's panels

These rules apply to the Machine, Jog, Console and Macros panels; the parts
below only state exceptions.

- Body: `panel` fill inside the dock group; title strip and tab strip as in
  `layout.md` (flat `panel-title` strip, decision 2; tabs along the group's
  bottom edge).
- Every panel body is a vertical scroll area (`layout.md`, height
  shrinking), with one exception: the Machine panel's state line and its two
  big button rows never scroll away (see Machine panel).
- Three button sizes (Echo of the reference's three tiers [R1][R2]):
  - **big**: `row-big` tall, `icon-big` left of the label, label in
    `type-body`; icon and label centred together; equal widths across a
    row; used only for the two job rows of the Machine panel;
  - **ordinary**: `row` tall, optional `icon-small` left of the label;
  - **pad**: square, one `row` on a side, icon only (`icon`), used only for
    the jog pad.
- Button face: `field` fill with a thin `border`, slightly rounded corners;
  hover `hover`, pressed `pressed`; keyboard focus shows an `accent` ring.
  Disabled: icon and label in `text-disabled`, the face loses its border
  contrast; nothing else changes. A disabled control always has a tooltip
  whose second line says what is missing (naming.md, Tooltips).
- Stop (big, job bar, menu) is the only panel control drawn with a
  `danger` fill; its label and icon use `text-on-strong` so the pair meets
  WCAG 2.x 4.5:1. Destructive confirmation buttons in this file's dialogs
  (for example "Remove profile") also take a `danger` fill, as `dialogs.md`
  defines for every dialog.
- Switches follow decision 4 (`visual-language.md` draws them): pill left of
  its label, `accent` fill with the knob right when on, neutral grey with the
  knob left when off, faded when disabled (label in `text-disabled`).
- Forms: labels right-aligned directly left of their field, units in the
  label (naming.md, Units), two columns of label and field pairs where the
  panel is wide enough, one column otherwise. Read-outs of coordinates use
  `type-mono` so digits do not jump.
- Dropdowns: `field` with a small chevron at the right; the open list uses
  `accent-soft` for the row under the pointer and the chosen row (not a
  solid blue band with white text as in the reference [R1]).
- Spacing: `gap` between controls, `gap-tight` inside a group (jog pad,
  label and field), `gap-wide` between blocks; thin `border` separators
  between blocks only where this file says so.

Tier: **Echo** (button tiers, right-aligned labels, switches instead of
check boxes), **Own** (every look).

---

## Machine states

### Reference

- The reference's job-control window shows its device status as one line of
  small plain text at the top: disconnected, ready, or busy [R1].
- While busy, the line carries the elapsed streaming time, the estimated
  remaining time in parentheses, a long progress bar that fills from the
  left, and a percentage at the far right; the percentage counts commands
  already sent, not work physically done [R1].
- A galvo-only framing state also exists [R1]. The docs show no separate
  paused or alarm wording in that line, and they never state which buttons
  are enabled in which state; screenshots disagree [R1][R6][R8][R16].
- On connecting, a GRBL-type controller's greeting with its firmware
  version shows up in the console panel [R1][R3].

### LaserCraft

The state words are `naming.md`'s machine state text. The app derives the
state from GRBL status reports (`device.md`, `MachineState`) and from its own
job state; the UI only draws what the app hands it (CLAUDE.md, UI has no
business logic).

| State text | Comes from | State dot | Word | Pause / Resume button | Start, Frame, Frame outline | Home | Jog pad, Move head to, Go to user origin, Set user origin | Connect / Disconnect | Progress line |
|---|---|---|---|---|---|---|---|---|---|
| Not connected | no port open | none | `text-muted` | disabled, reads Pause (job bar hidden) | disabled | disabled | disabled | Connect, enabled when a port is chosen | hidden |
| Connecting… | Connect pressed, no welcome banner or report yet | `text-muted` | `text` | enabled, reads Pause | disabled | disabled | disabled | Disconnect (cancels the attempt) | hidden |
| Ready | GRBL Idle, no job or framing of ours running | `success` | `text` | enabled, reads Pause | enabled when the job has output | enabled | enabled | Disconnect | hidden, or the last job's result (below) |
| Running | our job streaming; GRBL Run | `accent` | `text` | enabled, reads Pause, or "Pause (motion only)" for a constant-power job | disabled | disabled | disabled | disabled (Stop first) | shown, bar fill `accent` |
| Framing | our framing in progress; GRBL Run or Jog | `accent` | `text` | enabled, reads Pause | disabled; the Frame button that started a repeating frame reads "End framing" | disabled | disabled | disabled | shown as "Framing" text only, no bar |
| Paused | GRBL Hold (feed hold) | `warning` | `text` | enabled, reads Resume | disabled | disabled | disabled | disabled | shown, frozen, bar fill `warning` |
| Jogging | GRBL Jog from our jog command | `accent` | `text` | enabled, reads Pause | disabled | disabled | jog pad enabled (to add moves); the rest disabled | disabled | hidden |
| Homing | GRBL Home | `accent` | `text` | enabled, reads Pause | disabled | disabled | disabled | disabled | hidden |
| Alarm | GRBL Alarm, or an `ALARM:` line | `danger` | `text` | enabled, reads Pause (never Resume, safety rule 10) | disabled | enabled (homing clears GRBL's lock) | disabled | Disconnect | frozen, bar fill `danger`, if a job was streaming |
| Door open | GRBL Door | `warning` | `text` | enabled, reads Resume while a job is held | disabled | disabled | disabled | disabled while a job is held, else Disconnect | frozen, bar fill `warning`, if a job was streaming |
| Checking | GRBL check state (`$C` on) | `text-muted` | `text` | enabled, reads Pause | Start enabled (runs the check), Frame disabled | disabled | disabled | Disconnect | shown while a check streams, bar fill `accent` |
| Sleeping | GRBL Sleep | `text-muted` | `text` | enabled, reads Pause | disabled | disabled | disabled | Disconnect | hidden |
| No response | no status reply within the stall timeout | `warning` | `text` | enabled, reads Pause (never Resume) | disabled | disabled | disabled | Disconnect | frozen, bar fill `danger`, if a job was streaming |

Rules that apply to every row:

- Stop is enabled in every connected state (safety rule 7) and disabled
  only while Not connected. Pause is likewise enabled in every connected
  state; the same button reads Resume while GRBL holds a job (Paused, or Door
  open with a held job) and Pause otherwise. Pressing it while nothing moves
  still sends feed hold; the next status report decides what the state line
  shows.
- The dot colors are `layout.md`'s job-bar dots and `visual-language.md`'s
  Machine state dot, repeated here so the Machine panel and the job bar
  always agree. The state word itself is always `text` (or `text-muted`
  while Not connected): the dot carries the hue, never the word, because
  `visual-language.md` allows no colored body text in chrome and `warning`
  text misses the text contrast target in the light theme.
- "Enabled" for Move head to's Go and for Go to user origin also needs a
  homing cycle since connecting (Jog panel, Homing requirement); jogs are
  the stated exemption (Jog panel, Jog block).
- Disabled states in the table also apply to the Machine menu rows
  (`menus.md`) and to the chords bound to the same commands: a chord whose
  command is disabled does nothing and the Messages field says why
  (naming.md, Tooltips, disabled control wording).
- The state word's tooltip names the GRBL state behind it ("GRBL reports
  Hold"), and for Alarm the alarm's meaning.
- Changes of state are never animated beyond the progress bar moving
  (Reduce motion, `naming.md` Display preferences and `dialogs.md`
  Preferences → Display, changes nothing here).

### Progress line

- Shown while a job streams or is held, and after a job ends until the next
  Start, Frame, jog or home.
- Contents left to right: elapsed time, "·", remaining time with "about"
  ("12:04 · about 31:40 left"), the bar (stretches), the sent share as
  "‹n›% sent". Times use h:mm:ss (naming.md, Numbers) in `type-mono`; the
  sent share in `type-small` `text-muted`. The bar is `visual-language.md`'s
  Progress bar: a thin `field-alt` track with a *thin* `border` and *pill*
  ends, filled from the left with the fill token from the table.
- After a job, all in `text` (no colored text):
  - Finished: the bar disappears; a line with a `success` tick mark
    (`icon-small`) and "Finished in ‹h:mm:ss›" remains.
  - Stopped by the user: "Stopped at ‹n›% sent", with the frozen bar in
    `text-muted`.
  - Stopped by an alarm, error, stall or lost connection: "Stopped by alarm
    at ‹n›% sent" (or the matching cause), with the frozen bar in `danger`
    and a `danger` `mark-error` (`icons.md`) before the line.
  These lines are information only; nothing offers to continue a stopped
  job (safety rule 10).
- Narrow panel: the bar moves to its own line under the times; the words
  "sent" and "left" stay (they say what the numbers mean).

Tier: **Echo** (one state line at the top with a progress bar and a sent
share [R1]), **Own** (the state set, words, dots and every enabling rule).

---

## Machine panel

### Reference

- A dock panel in the lower group of the right column, sharing that group
  with the material library; tabs along the group's bottom [R1][R4][R5][R22].
- Top to bottom for GRBL-type devices: the status line; a row of three big
  buttons Pause, Stop, Start, about twice the height of ordinary buttons,
  each with a bold glyph left of its label; a second big row with two frame
  buttons (bounding box and tight outline), a save-machine-file button and a
  run-machine-file button [R1][R6][R23].
- Then an ordinary row: Home and go-to-origin buttons on the left, a
  start-from label with a dropdown of three placements on the right
  (absolute coordinates, current position, user origin) [R1][R7].
- Then a job origin label with a 3×3 grid of round radio buttons in a
  slightly recessed box, greyed while placement is absolute; left of it,
  optional switches such as a rotary enable that appears only when a
  preference asks for it [R1][R7][R30].
- Then a switch that limits output to the selection, with a last-position
  button on the right whose crosshair is placed once on the canvas and not
  updated during a job; then a switch that takes the origin from the
  selection (greyed unless the selection switch is on and placement is not
  absolute) and a button that opens the cut optimization settings
  [R1][R11][R21].
- Bottom row: a devices button (right-click reconnects), a serial port
  dropdown (automatic, choose, or a port name; an empty list means no port
  was found; the choice is remembered), and a device profile dropdown with
  a small badge per entry [R1][R6][R13].
- The reduced beginner mode hides the machine-file buttons, the
  optimization button and the selection switch [R16]. The 2.1 changelog
  shows a rotary icon on Start while rotary is on, shows laser source errors
  in this panel, and blocks the fire control while framing or busy [R24].

### LaserCraft

Panel title "Machine"; Window → Panels tooltip "Laser connection and job
control" (naming.md). Default place: right dock, lower group, first tab, in
front (`layout.md`). M6.

The panel is a fixed grid: blocks 1–3 below are pinned at the top and never
scroll; blocks 4–9 scroll if the group is short.

| # | Block | Contents left to right | Milestone |
|---|---|---|---|
| 1 | State line | state dot, state word; progress line under it when shown | M6 |
| 2 | Job row (big) | Pause / Resume, Stop, Start | M6 |
| 3 | Framing row (big) | Frame, Frame outline, Save G-code…, Run G-code file… | M6 (Run G-code file… later) |
| 4 | Notes band | constant-power note, Job refused band, Alarm banner (each only when it applies) | M6 |
| 5 | Positioning row | Home, Go to user origin; Place job at with its dropdown | M6 |
| 6 | Placement block | left column: Use rotary (M9), Selected shapes only, Reference point from selection; right column: Job reference point grid | M6 (Use rotary M9) |
| 7 | View row | left: (empty); right: Show head marker, Cut order… | M6 (Cut order… later) |
| 8 | Connection row | Connect / Disconnect, Port, Baud rate, Refresh ports | M6 |
| 9 | Device row | Device, Device profiles… (M6: Edit device profile…) | M6 / M8 |
| 10 | Serial port access help box | only after a refused port | M6 |

A thin `border` separator sits between block 3 and block 4, and between
block 7 and block 8. Blocks 5–7 form a two-column grid: the left column
holds the buttons and switches, the right column the dropdown, grid and the
view controls, as in the reference's left-half/right-half split [R1].

#### 1. State line

- Directly under the title strip, left-aligned: the state dot (a small
  filled circle in the dot token from Machine states), then the state word in
  `type-body`, then, in `text-muted`, "on ‹port› at ‹baud› baud" while
  connected ("on /dev/ttyUSB0 at 115200 baud"). The port text is cut with "…"
  first when space is short.
- Under it, the progress line (Machine states, Progress line).
- No frame or band of its own while all is well; it sits on `panel` (Match
  [R1]). In Alarm the whole line sits on a `danger-soft` band.
- Click on the state word: no action; tooltip as in Machine states.

#### 2. Job row (big buttons)

Order is the reference's (Match [R1]): Pause / Resume, Stop, Start, three
equal columns.

| Button | Label | Icon concept | Look | Tooltip (first line; chord from the keymap) | Enabled | Command |
|---|---|---|---|---|---|---|
| Pause / Resume | "Pause"; "Resume" while held; "Pause (motion only)" during a constant-power job | two upright bars; a right-pointing triangle for Resume; bars with a small `warning` mark for motion-only | big, normal face | "Pause the job (Pause)" / "Resume the held job (Pause)" / "Stop motion; the laser may stay on (Pause)" | every connected state (Machine states) | `machine.pause` / `machine.resume` (new, M6) |
| Stop | "Stop" | filled square | big, `danger` fill, `text-on-strong` label and icon; an `accent` ring as well during a constant-power job | "Stop at once and reset the controller (Ctrl+Break)"; second line "Streaming ends; the job cannot be resumed." | every connected state | `machine.stop` (new, M6) |
| Start | "Start" | right-pointing triangle; with Use rotary on, a small cylinder mark joins it (M9; idea from [R24]) | big, normal face | "Check the job against the work area, then run it (Alt+S)" | Ready or Checking, and the job has output | `machine.start` (new, M6) |

- None of the three keeps keyboard focus after a click, so a later Space or
  Enter can never start or resume a job by accident (own design; the
  reference makes this a preference [R30]; whether it stays fixed is
  naming.md question 8).
- Start runs, in order: the No selection message if Selected shapes only is
  on with nothing selected (naming.md: buttons "OK" and "Use all shapes";
  "Use all shapes" turns the switch off and starts nothing); the Before you
  start checklist if Confirm before start is on (M8; Dialogs below); the
  bounds check (safety rule 5). A refusal shows the Job refused band and
  sends nothing.
- Keyboard: Alt+S (M6) starts; the Pause key (M6) pauses and resumes;
  Ctrl+Break (M6) stops. Alt+Shift+S ("send it without starting") is not
  bound (proposed change to keymap.md, which lists it for M6): there is no
  separate send step for GRBL (naming.md, Omitted).

#### 3. Framing row (big buttons)

Four equal columns once Run G-code file… exists; three until then.

| Button | Label | Icon concept | Tooltip | Enabled | Command |
|---|---|---|---|---|---|
| Frame | "Frame" | dashed rectangle with a corner arrow | "Trace the job's bounding box with the laser off (Alt+R)" | Ready and the job has output | `machine.frame box` (new, M6) |
| Frame outline | "Frame outline" | dashed hull with a corner arrow | "Trace the tight outline around the job with the laser off (Alt+Shift+R)" | as Frame | `machine.frame outline` (new, M6) |
| Save G-code… | "Save G-code…" | page with a toolpath zigzag | "Write the job to a G-code file (Alt+Shift+L)" | always (no machine needed); the bounds check reports problems when it runs | `ui.export_gcode` (today's action) |
| Run G-code file… | "Run G-code file…" | page with a toolpath zigzag and a triangle | "Stream a saved G-code file after the same checks as Start" | Ready | new front-end action (later); the file is scanned for `M3` before streaming and, if it holds one, the run is a constant-power job (Constant-power (M3) variant) |

- The reference labels both frame buttons with one word and tells them
  apart by glyph only [R1]; LaserCraft writes the second one out ("Frame
  outline") so the difference is readable (keymap.md's wording).
- Framing is always laser-off (safety rule 2). There is no modifier on a
  Frame button and no device setting that turns the laser on; the
  reference's modifier-click and laser-on-framing setting are omitted
  [R1][R12] (Deliberately omitted). Frame at low power… lives only in the
  Machine menu and only if the owner approves it (`menus.md`,
  `dialogs.md`).
- Before framing, the frame path is drawn on the canvas in `frame-preview`
  while the pointer is over a Frame button and while framing runs
  (`canvas.md`); the reference shows its frame outline on the canvas in a
  single highlight color [R1][R5].
- Frame speed is the Jog panel's Jog speed (mm/s) (Match [R1][R5]); the
  tooltip's second line says so.
- With Repeat framing on (device profile, Options tab) the frame repeats
  until ended. The reference opens a small dialog whose button ends it [R5];
  LaserCraft opens no dialog: the Frame button that started it reads "End
  framing" (icon unchanged) and ends the loop at the end of the current
  pass, while Stop stays the immediate way out. The state line reads
  "Framing" with "repeating" in `text-muted`.
- The frame path is checked against the work area like a job (safety
  rule 5); a refusal shows the Job refused band.
- Ctrl+Shift+A (keymap.md, M6, "frame the selection") has no button here;
  whether it stays a framing chord is naming.md question 2.

#### 4. Notes band

Up to three bands may stack here, in this order; each spans the panel width
with `gap` padding, an `icon-small` mark at the left and text in
`type-body`. They sit below the big rows on purpose: the big buttons never
move when a band appears, so a user reaching for Stop never hits something
else (own design).

**Constant-power note** (safety rule 9; M6)

- Shown whenever the job includes an output layer with Constant power (M3)
  on, before and during the job; the app supplies the layer names.
- `warning-soft` band, `warning` mark (`mark-warning`), text in `text`. Text: "Constant power (M3) on
  ‹layer names›. The laser may stay on while paused; Stop turns it off."
  Layer names are cut with "… and ‹n› more" after two names.
- While such a job runs, Pause reads "Pause (motion only)" and Stop gets
  its `accent` ring (job row); the job bar does the same (`layout.md`).

**Job refused band** (safety rule 5; M6)

- Shown when Start, Frame, Frame outline or Run G-code file… was refused
  before anything was sent: `danger-soft` band, `danger` mark, the message
  from naming.md ("Cannot run: shape "‹name›" goes ‹d› mm past the
  ‹edge› edge of the bed …"), and a "Show in job preview" link-style button
  that opens the job preview at the problem (`job-preview.md`).
- A close button (×, `icon-small`) at the band's right end dismisses it; it
  also goes away at the next successful Start or Frame.

**Alarm banner** (safety rules 10–11; M6) — see "Alarm banner" below.

#### 5. Positioning row

| Control | Label | Icon concept | Tooltip | Enabled | Command |
|---|---|---|---|---|---|
| Home | "Home" | house | "Run the homing cycle; the laser stays off (Ctrl+H)" | Ready or Alarm | `machine.home` (new, M6) |
| Go to user origin | "Go to user origin" | the flag motif on a small cross (`go-user-origin`, `icons.md`; not a pin) | "Move the head to the stored user origin, laser off" | Ready, a user origin is set, and the machine has homed since it connected (Homing requirement, below) | `machine.go_to_user_origin` (new, M6) |
| Place job at | label "Place job at", dropdown | none | "Where the job goes on the bed" | always except while a job or framing runs | `job.set_placement <absolute\|current\|user-origin>` (new, M6, undoable, saved in the project) |

Place job at values, in this order (Match [R1][R7]; values are plain CNC
terms, naming.md):

| Value | Meaning shown as the item's tooltip |
|---|---|
| Absolute coordinates | "The job lands where it sits on the bed." |
| Current position | "The job's reference point lands where the head is when you press Start." |
| User origin | "The job's reference point lands on the user origin." Disabled in the list while no user origin is set; its tooltip then adds "Set one in the Jog panel." |

- Home and Go to user origin are ordinary buttons, each about a quarter of
  the panel wide (Match [R1]); Place job at's label is right-aligned and the
  dropdown fills the right half.
- macOS: Ctrl+H is Cmd+H, the system "hide" chord (keymap.md); the tooltip
  shows the chord as the keymap resolves it.

#### 6. Placement block

Left column, switches stacked top to bottom:

| Switch | Label | Tooltip | Enabled | Default | Command |
|---|---|---|---|---|---|
| Use rotary | "Use rotary" | "Run jobs on the rotary attachment" | always; shown only once a rotary is set up (M9) | off | `job.set_rotary` (M9) |
| Selected shapes only | "Selected shapes only" | "Start, frame, preview and G-code use only the selection" | always except while a job runs | off, and reset to off when a project opens | `job.set_selected_only` (new, M6, not saved) |
| Reference point from selection | "Reference point from selection" | "The job reference point is taken from the selected shapes, not the whole design" | only when Selected shapes only is on and Place job at is not Absolute coordinates (Match [R1][R11]) | off, reset when a project opens | `job.set_reference_from_selection` (new, M6, not saved) |

- The two selection switches are not saved in the project because the
  selection is not saved either (own design; the reference offers
  preferences for both [R30]).
- The reference's rotary switch appears only when a preference asks for it
  [R1][R30]; LaserCraft shows Use rotary as soon as a rotary attachment is
  set up (own design, M9), and its state shows in the status bar's Active
  setups field (`layout.md`).

Right column: **Job reference point**

- Label "Job reference point", right-aligned, then `visual-language.md`'s
  3×3 point grid (Echo [R1][R7]): nine small choice dots joined by
  *hairline* `border` lines that outline the bounding box they stand for,
  sitting on a `field` square with a *thin* `border`. No recessed box and no
  shading.
- Each dot is a radio choice; the chosen dot is a filled `accent` dot, a
  little larger than the others; the others are `field` circles with a
  `border` rim; the dot under the pointer gets a `hover` disc. The dots are spaced like the reference point grid of the
  Position and size toolbar (`property-bar.md`), so the two grids read as
  siblings; they stay separate settings (naming.md).
- Disabled while Place job at is Absolute coordinates: dots in
  `text-disabled`, the chosen one shown as a filled `text-disabled` dot
  (Match [R1][R7]).
- Tooltip "Which point of the job lands on the job origin"; each dot's
  tooltip names its point in screen words ("Bottom left", "Center").
- Default: the point that matches the device's origin corner (front-left
  origin → bottom left), own design. Saved in the project
  (`job.set_reference_point <point>`, new, M6, undoable).
- The job origin marker on the canvas moves when the grid, Place job at or
  either selection switch changes (Match [R7][R11]; `canvas.md` draws it).
- The grid keeps a fixed, compact size; extra panel width goes to the gap
  between the columns, never to the grid (the reference's grid grows and
  leaves empty space [R4]).

#### 7. View row

Right column (the left column stays empty, reserved for later switches):

- **Show head marker** (switch): label "Show head marker"; tooltip "Show
  where the laser head is on the canvas". Default on. While on, the canvas
  draws the head marker at the last reported position (`canvas.md`,
  `head-marker`). Unlike the reference's one-shot button that places a
  crosshair and does not follow the job [R1], ours is a live switch that
  follows status reports, including during a job (own design; naming.md).
  Enabled always; with no connection the marker shows the last known
  position in `text-disabled` (canvas.md decides the look).
- **Cut order…** (ordinary button, text only): opens the Cut order dialog
  (`dialogs.md`; planning.md fixes the order today). Later; until the dialog
  exists the cell is empty rather than holding a disabled button.

#### 8. Connection row

| Control | Label | Icon concept | Width | Tooltip | Enabled | Command |
|---|---|---|---|---|---|---|
| Connect / Disconnect | "Connect" / "Disconnect" | plug / unplugged plug | about a fifth of the row | "Open the serial connection to the laser" / "Close the connection; Stop first if a job runs" | Connect: a port is chosen (or Automatic). Disconnect: connected and no job or framing running (`menus.md`) | `machine.connect` / `machine.disconnect` (new, M6) |
| Port | "Port" (label before the dropdown when the panel is wide; otherwise the dropdown's tooltip carries it) | none | stretches | "Serial port the laser is on" | while not connected | `device.set_port <name\|auto>` (new, M6, saved in the device profile) |
| Baud rate | "Baud rate" (same rule) | none | about a fifth | "Serial speed; GRBL uses 115200 unless its firmware was built otherwise" | while not connected | `device.set_baud <n>` (new, M6, saved in the device profile) |
| Refresh ports | none (icon only) | circular arrow | one `row` square | "Refresh ports: look for serial ports again" | while not connected | `ui.refresh_ports` (new) |

Port list contents, top to bottom (own design; Echo of the reference's
automatic and choose entries [R1][R6]):

| Entry | Shown as | Meaning |
|---|---|---|
| Automatic | "Automatic" | On Connect, LaserCraft tries each USB serial port in turn and keeps the first that answers with a GRBL welcome banner (the scan routine of Device scan). Default for a new profile. |
| Choose a port | "Choose a port" (`text-muted`) | Shown only when no port is chosen and Automatic is off; not selectable as a value. |
| each found port | the OS's port name in `type-mono`, then in `text-muted` a description when the OS gives one (for example the USB adapter's name) | Use exactly this port. |
| no ports | one greyed row "No serial ports found" | Tooltip "Check the USB cable and that the laser is powered; then use Refresh ports." |

- On Linux, port names look like `/dev/ttyUSB0` or `/dev/ttyACM0`
  (`docs/linux-serial.md`); on Windows like `COM3`. A port another
  LaserCraft window holds is listed with "(in use)" and cannot be chosen
  (`layout.md`, Multiple windows).
- The chosen port and baud are remembered in the device profile, so they
  come back after a restart (Match [R1]).
- Baud rate list: 9600, 19200, 38400, 57600, 115200 (default), 230400,
  250000; the dropdown also accepts a typed value. GRBL's interface wiki
  documents the default.
- The reference reconnects by right-clicking its devices button [R1][R6];
  LaserCraft has a visible Connect / Disconnect button instead and no hidden
  right-click action (own design).
- While Connecting…, the button reads "Disconnect" and cancels the attempt.

#### 9. Device row

| Control | Label | Tooltip | Enabled | Milestone |
|---|---|---|---|---|
| Device | "Device" dropdown | "Device profile in use: work area, origin, port and limits" | while not connected | M6 shows the single default profile (the dropdown is then read-only, `text-muted` chevron); M8 lists all profiles |
| Edit device profile… | "Edit device profile…" | "Change this machine's work area, port, speed and limits" | always; while connected, fields that need a reconnect are read-only in the dialog | M6 (until M8) |
| Device profiles… | "Device profiles…" | "Add, remove and choose saved machines" | always | M8 (replaces Edit device profile… here; that command stays in the Machine menu) |

- Device list items show the profile name, then in `text-muted` "‹w› ×
  ‹h› mm"; the default profile carries a small "default" tag in
  `text-muted`. There are no device-type badges (vendor logos are omitted,
  naming.md) — every profile is GRBL.
- Switching the device while a project is open asks nothing; opening a
  project saved with another profile shows the "Switch device profile?"
  prompt (naming.md, M8).

#### 10. Serial port access help box

See "Serial port access help" below.

#### Machine panel: disconnected, narrow and simple

- Not connected: the state line reads "Not connected" with no dot; Pause,
  Stop, Start, Frame, Frame outline, Home and Go to user origin are disabled
  (tooltips "Needs a connected machine."); Save G-code…, Place job at, the
  Job reference point grid (when not absolute), the switches, Show head
  marker and the connection and device rows work. No extra empty-state text
  (naming.md, Empty states).
- Narrow: the big rows keep equal columns; labels shorten to their icon plus
  first word only where the label would clip ("Frame outline" keeps both
  words; "Save G-code…" may become an icon with the label in its tooltip).
  Blocks 5–7 drop to one column (left column first, then right). Block 8
  wraps Baud rate and Refresh ports onto a second line.
- Short: blocks 1–3 stay pinned; the rest scrolls (`layout.md`, never hidden
  by size).
- Simple layout (later): hides Save G-code…, Run G-code file…, Cut order…,
  Selected shapes only, Reference point from selection and Use rotary
  (`layout.md`); the big rows keep their size. The Framing row then has two
  columns.

Tier: **Match** (place, block order, the big-row order, the left/right
split, the dropdown values, the 3×3 grid and its greying), **Echo** (state
line with progress, the selection switches, the device row), **Own**
(labels, icons, tooltips, the pinned top, the notes band, Connect /
Disconnect, Baud rate, Refresh ports, the live head marker switch, every
look).

---

## Constant-power (M3) variant

### Reference

- The docs read for this file show no special labelling of Pause for a
  constant-power layer (the reference's laser and console pages say nothing
  about it [R1][R3]). The device's G-code settings describe an option that
  repeats the power word on every move [R18]; none of the pages read for
  this file shows job-control wording for constant power.

### LaserCraft

Safety rule 9 is binding: GRBL's feed hold stops motion but not the
spindle output, so a constant-power (`M3`) job may keep the laser on while
paused. Everything below is our own design.

| Where | Normal job (`M4`) | Constant-power job (any output layer with Constant power (M3) on, or a Run G-code file… file that contains `M3`) |
|---|---|---|
| Machine panel, job row | "Pause" | "Pause (motion only)" with a `warning` mark on the bars icon; Stop gets an `accent` ring around its `danger` fill |
| Machine panel, notes band | — | constant-power note in `warning-soft`, before and during the job |
| Job bar | "Pause"; normal background | "Pause (motion only)", background `warning-soft`, note "The laser may stay on" in `type-small` `text` with a `warning` mark (`mark-warning`) immediately left of Pause; the note and the full Pause label never shrink, move to a tooltip or leave the bar at any width (`layout.md`, Job bar); Stop ringed |
| Machine menu | "Pause" | "Pause (motion only)" with a `warning` icon and the tooltip "The laser may stay on"; the Stop row carries a `danger-soft` band (`menus.md`) |
| Pause tooltip | "Pause the job (Pause)" | "Stop motion; the laser may stay on (Pause)", second line "Use Stop to turn the laser off." |
| While held | "Resume" | "Resume", and the state line adds "laser may be on" in `text` with a `warning` mark (`mark-warning`) beside Paused |
| Hardware test | H4 | new check (proposed change to hardware-testing.md): M3 job, Pause, confirm the label and that Stop turns the beam off |

- The app decides whether the job is a constant-power job: from the
  layers for a project job, and for Run G-code file… (later) by scanning the
  file for an `M3` word before streaming. Either way the UI receives one
  flag (plus the layer names, or the file name, for the note) and shows the
  same note, labels and Stop ring (no business logic in the UI).
- `layers-panel.md` adds the Constant power (M3) switch to the layer editor
  with a note pointing at this behaviour.

Tier: **Own**.

---

## Alarm banner and Clear alarm

### Reference

- The console page shows GRBL error lines and a readable explanation below
  each in a dark red text color, while the echoed command stays dark
  [R3]. The docs read here show no alarm display in the job-control window;
  the 2.1 changelog says laser source errors now show in that panel [R24].
- The console page lists GRBL's unlock command among the standard commands
  [R3]. A troubleshooting page phrases a failed-transfer message as the
  machine possibly being busy or paused [R15].

### LaserCraft

Safety rules 10 and 11: a disconnect, a stall, an `ALARM:` line or an
`error:` reply stops streaming and shows what happened with its GRBL
meaning; streaming never resumes by itself.

- Place: the Machine panel's notes band (below the big rows), and the job
  bar turns `danger-soft` (`layout.md`). If the Machine panel is hidden or
  behind another tab, the job bar's state word "Alarm" is the visible cue;
  clicking the state word brings the Machine panel to the front (and shows
  the side panels if F12 hid them).
- Look: `danger-soft` band, `danger` `mark-error` (a circle holding an
  exclamation mark, `icon-small`, `icons.md`; the triangle is the warning
  mark and is not used here), up to three lines in `type-body`, then a row
  of ordinary buttons. All text in `text`, the lead-in included (no colored
  body text, `visual-language.md`); the raw GRBL line, where quoted, in
  `type-mono`.
- Wording follows naming.md, Messages and errors: what happened, why, what to
  do. Templates:

| Kind | Lead-in and text | Buttons | Stays until |
|---|---|---|---|
| GRBL alarm | "Alarm ‹n›: ‹meaning›. Streaming stopped. ‹next step›" | Clear alarm, Home, Show in console | Clear alarm succeeds or the connection closes |
| GRBL error reply during a job | "GRBL error ‹n› on line ‹k›: ‹meaning›. Streaming stopped; feed hold and reset were sent." | Show in console, × | dismissed or next Start |
| No response (stall) | "No response from the controller for ‹s› s. Streaming stopped. Check the cable and power, then connect again." | Disconnect, Show in console, × | dismissed or a report arrives |
| Connection lost during a job | "The connection to ‹port› was lost during the job. Streaming stopped. Check the machine before connecting again." | Connect, × | dismissed |
| Unknown alarm code | "Alarm ‹n›: no description is known for this code." then the same steps | as for an alarm | as for an alarm |

- The meanings are our own short wording of GRBL's alarm and error tables,
  kept with the device crate (naming.md). The raw GRBL line always appears
  unchanged in the Console.
- Rule 11: on an `error:` reply while streaming, LaserCraft also sends feed
  hold and soft reset; the banner says so ("feed hold and reset were sent")
  so the user knows why GRBL may report an alarm next.
- **Clear alarm** (ordinary button, label "Clear alarm", no icon): tooltip
  "Clear the alarm lock (GRBL $X) once the machine is safe"; enabled only in
  Alarm; sends `$X` (`machine.clear_alarm`, new, M6). It asks no question,
  fires nothing (rule 1: nothing fires after an alarm is cleared), and does
  not restart streaming. After an alarm that may have lost position (limit
  hit, reset during motion) the banner's next step says to home first. Its
  placement beyond the banner (Machine menu, Console) is naming.md question
  10; `menus.md` lists a Machine menu row.
- **Home** in the banner is the same command as the positioning row's
  Home, repeated because homing is GRBL's other way out of an alarm.
- **Show in console** brings the Console to the front of its group and
  scrolls to the alarm line (`layout.md`, tab strip rules).
- Nothing in the banner or anywhere else offers "Resume" or "Continue the
  job" after an alarm, error, stall or disconnect (rule 10).
- The status bar's Messages field repeats the first line (`layout.md`); the
  reference also reports device messages in its status strip, though only
  connection news for other controller kinds is described there [R26].

Tier: **Own** (the reference shows only console error lines [R3]).

---

## Serial port access help

### Reference

- The connection pages explain that an empty port list points to cable,
  power or driver problems [R1][R6]. The docs read here show no handling of
  a refused port or of Linux group membership (no Linux screenshots at all)
  [R6][R15].

### LaserCraft

Required by `CLAUDE.md` (Linux: detect "permission denied" and explain the
fix in-app), `docs/linux-serial.md` and `device.md` (`PermissionDenied {
port, help }`, the family read from `/etc/os-release`). Own design. M6.

- Appears as block 10 of the Machine panel, right under the device row
  (next to the Port control it is about), when Connect fails with
  permission denied. The status bar's Messages field shows the first
  sentence (`layout.md`). It is never only a dialog.
- Look: a `warning-soft` box (not `danger`: nothing is wrong with the
  machine) with a `warning` mark, heading "Serial port access" in
  `type-heading`, then text in `type-body`, commands in `type-mono` on a
  `field` strip with a "Copy" button (copy icon) at its right.
- Contents, top to bottom:

| # | Part | Text (template) |
|---|---|---|
| 1 | Message | "Cannot connect to ‹port›: permission denied. Your user is not in the serial group (‹group›)." (naming.md) |
| 2 | Command | `sudo usermod -aG ‹group› $USER` with Copy |
| 3 | Next step | "Run the command in a terminal, then log out and back in (or restart). Then click Connect." |
| 4 | Check (`text-muted`) | "To see which group owns the port: `ls -l ‹port›`." |
| 5 | Flatpak note (only inside Flatpak) | "This Flatpak also needs device access. If it was turned off, run:" then `flatpak override --user --device=all io.github.bhimio1.LaserCraft` with Copy. "Group membership still applies inside the sandbox." |
| 6 | Hint (`text-muted`) | "If connecting fails right after plugging in, wait a few seconds and try again." (ModemManager, linux-serial.md) |
| 7 | Buttons | "Try again" (= Connect), "Hide" |

- ‹group› is `uucp` on Arch and its derivatives (CachyOS, Manjaro,
  EndeavourOS) and `dialout` on Debian, Ubuntu and their derivatives and on
  Fedora, as `linux-serial.md` tabulates. When the family is unknown, the box
  names both and says to check with `ls -l` (row 4 moves up).
- The box stays until Hide, a successful connection, or another port is
  chosen.
- Windows and macOS: a refused or busy port shows the same box shape with
  only rows 1 and 7 and a "why" that names the OS reason ("the port is in use
  by another program"), never a Linux command.

Tier: **Own**.

---

## Job bar

### Reference

- None: the reference keeps Pause, Stop and Start only in its job-control
  panel, which can be hidden or covered by another tab [R1][R22][R25]; its
  panels can also be undocked, moved and tabbed with others [R27].

### LaserCraft

`layout.md` fixes the job bar's place (right end of toolbar row 1), when it
shows (while connected), its contents (state dot and word, sent share,
Pause / Resume, Stop), its backgrounds (normal, `warning-soft` for a
constant-power job, `danger-soft` in Alarm) and the rule that nothing hides
or covers it. This section fixes the buttons and interactions. M6.

| # | Item | Look | Tooltip | Behaviour |
|---|---|---|---|---|
| 1 | State dot and word | as the Machine panel's state line, `type-body` | GRBL state, alarm meaning | click: brings the Machine panel to the front (and shows side panels if hidden) |
| 2 | Sent share | `type-small`, `text-muted`, "‹n›% sent" | "Share of the job sent to the controller; the machine may still be working on it" | read-only |
| 3 | Constant-power note | `type-small` `text` with a `warning` mark (`mark-warning`) before it, on the bar's `warning-soft` background, "The laser may stay on"; sits immediately left of Pause and never shrinks, moves to a tooltip or leaves the bar at any width (`layout.md`, Job bar) | "This job uses constant power (M3): Pause stops motion only. Use Stop to turn the laser off." | read-only; only during a constant-power job |
| 4 | Pause / Resume | compact: one `row` tall, `icon-small` plus label; labels and icons as the big button | as the big button | same command as the big button |
| 5 | Stop | compact, `danger` fill, `text-on-strong` label and icon, a little wider than Pause; `accent` ring during a constant-power job | as the big button | same command as the big button |

- Pause and Stop in the bar are enabled for the whole time the bar is shown
  (safety rule 7; Machine states).
- Narrow window (`layout.md`, Job bar): the constant-power note and the full
  Pause label never give way. What yields, in order: the other toolbars of
  row 1 fold into More, the sent share goes, then the state word shrinks to
  its dot (the word moves to the dot's tooltip).
- Start, Frame and Clear alarm are deliberately absent: the always-visible
  strip only offers actions that make the machine safer (`layout.md`).
- Keyboard focus never rests on the bar's buttons by default; Tab order
  reaches them last in the main window.
- The Pause key and Ctrl+Break act on the same commands from anywhere in the
  window, including while a text field has keyboard focus and while a
  dialog is open (proposed change to keymap.md, already raised in
  `layout.md`).
- Above dialogs: while connected the bar is drawn on the top-most layer so
  no window covers it, and no dialog uses an input-blocking backdrop over
  it (`layout.md`, `dialogs.md`). Every dialog in this file (Device profile,
  Device profiles, the wizards, GRBL settings, Edit macros, confirmations)
  follows that rule; several of them refuse to open while a job runs anyway
  (their sections say which).
- A first click on the bar while a menu or dropdown is open acts on the bar
  at once (it does not merely close the popup), as `menus.md` requires.
- Not connected: the bar is gone (hardware test H2, which reads
  "Disconnected" today; proposed change to hardware-testing.md); the Machine
  panel's
  disabled Pause and Stop remain as placeholders.

Tier: **Own**.

---

## Jog panel

### Reference

- A dock panel in the upper right group, behind the layer list, reached by
  its tab; tabs for GRBL-type devices run layers, move, console, macros
  [R2][R3][R22]. Not shown for galvo devices [R2].
- Upper block: a get-position button with X, Y, Z (and U) read-outs, blank
  until queried; a move-to-position row with X and Y fields and a go
  button; a saved-positions dropdown with a manage button; then a thin
  separator [R2][R7].
- Lower block: a 3×3 pad of small square glyph buttons — rotary arrows in the
  top corners, Y up and down in the middle column, X left and right in the
  middle row with home in the centre, Z down and up in the bottom corners;
  each click moves by the distance at the speed; a continuous-jog switch
  under the pad that appears only once a jog-command setting is on
  [R2][R12].
- Right of the pad: distance, speed (also used for framing and the canvas
  position-laser tool), Z distance and Z speed fields with units [R2][R5].
- A focus button for machines with a movable Z; three origin buttons (set,
  clear, set finish position), taller with two-line labels, hidden for one
  proprietary controller family; a switch that makes the arrows go to the
  distance measured from machine zero; a power field with a fire button,
  shown only when a device setting allows it, meant for diode lasers
  [R2][R7][R12].
- While a job streams, the body is replaced by two equal framed groups to
  adjust speed and power in percent steps around a reset button [R2].
- Keyboard jogging uses Ctrl/Cmd chords with the bracket keys; the numeric
  keypad also jogs (8, 2, 4, 6 and diagonals) with canvas focus [R2].

### LaserCraft

Panel title "Jog"; Window → Panels tooltip "Move the laser head by hand and
store origins". Default place: right dock, upper group, second tab
(`layout.md`). M6.

Contents top to bottom:

| # | Block | Contents | Milestone |
|---|---|---|---|
| 1 | Head position | label "Head position", read-outs X, Y (and Z with a motorized Z axis) | M6 |
| 2 | Move head to | label "Move head to:", X (mm) field, Y (mm) field, Go button | M6 |
| 3 | Saved positions | Saved position dropdown, Edit… button | later |
| — | separator | thin `border` line | |
| 4 | Jog block | left: jog pad with Continuous jog under it; right: Jog distance (mm), Jog speed (mm/s), Z jog distance (mm), Z jog speed (mm/s) | M6 (Continuous jog later; Z fields only with a motorized Z axis) |
| 5 | Origin row | Set user origin, Clear user origin, Set end position | M6 (Set end position later) |
| 6 | Bottom row | Absolute jog switch (left) | later |

#### 1. Head position

- Read-outs in `type-mono`: "X 123.45  Y 67.80" (and "Z 5.00"), two decimals
  (naming.md, Numbers), `text`; the label in `type-body`, right-aligned like
  other labels.
- Live: updated from every status report (the device thread polls `?` at the
  profile's Status poll interval, `device.md`). No query button is needed,
  so the reference's get-position button is dropped (`naming.md`).
- Coordinates are machine coordinates: distances from the machine origin
  corner, as `gcode.md` "Coordinates" defines them. How they relate to the
  status bar's Pointer position is an open question.
- Not connected: the read-outs show "—" in `text-disabled`; after a
  disconnect they keep the last values in `text-disabled` with the tooltip
  "Last known position".
- Alarm or No response: values turn `text-muted` with the tooltip "Position
  may be wrong until the machine is homed" (alarm) or "No recent report"
  (stall).

#### 2. Move head to

| Control | Label | Tooltip | Enabled | Command |
|---|---|---|---|---|
| X field | "X (mm)" | "Target X, measured from the machine origin" | Ready | — (value only) |
| Y field | "Y (mm)" | "Target Y" | Ready | — |
| Go | "Go" (ordinary, text only) | "Move the head to these coordinates with the laser off" | Ready, the machine has homed since it connected, and the target is inside the work area | `machine.move_to <x> <y>` (new, M6) |

- Enter in either field is the same as Go.
- Homing requirement (the same rule as `toolbars.md`'s Move head, To head
  position and Go to selection, own design): until the machine has
  completed a homing cycle in the current connection, Go (and Go to user
  origin in the Machine panel) stays disabled with the tooltip's second
  line "Home the machine first." Without homing, the position GRBL
  reports need not match the bed, so a work-area check of an absolute
  target could pass a move that really leaves the bed. How machines
  without homing switches are handled is the open question shared with
  `toolbars.md` (Open questions).
- A target outside the work area turns the field's border `danger` and Go's
  tooltip says which edge it passes (spirit of safety rule 5); nothing is
  sent.
- The fields start empty; clicking the head read-out copies the current
  position into them (own design).

#### 3. Saved positions (later)

- Label "Saved position", right-aligned; a wide dropdown listing names
  ("Corner jig — X 10.00, Y 20.00"); choosing one moves the head there with
  the laser off (Ready only); "Edit…" opens the Saved head positions dialog
  (Dialogs below). Empty: the dropdown shows "No saved positions" in
  `text-muted`.

#### 4. Jog block

Jog pad (left), a 3×3 grid of pad buttons with `gap-tight` between them,
arranged like the reference's pad (Match [R2]):

| | left column | middle column | right column |
|---|---|---|---|
| top row | rotary turn counter-clockwise (M9; empty cell before) | Y+ (toward the rear, screen up) | rotary turn clockwise (M9) |
| middle row | X− (screen left) | Home (house) | X+ (screen right) |
| bottom row | Z down (only with a motorized Z axis) | Y− (toward the front, screen down) | Z up (only with a motorized Z axis) |

- Icons: chevrons for X and Y, a house for Home, an arrow pointing at a short
  bar for Z down / up, and for rotary `jog-rotary-ccw` / `jog-rotary-cw`:
  two small rollers seen end-on with a short straight motion stroke on their
  upper edge pointing the way of the turn (no curved arrow around a dot;
  `icons.md` draws them all). An empty cell is blank `panel`, not a disabled
  button.
- Arrow directions are screen words: up always moves the head toward the
  rear of the bed (top of the canvas), whatever the origin corner
  (naming.md, screen words and machine words).
- Tooltips: "Jog X+ by the jog distance" … with the chord where one is bound:
  Y− "Jog Y− (Ctrl+Shift+[)", Y+ "Jog Y+ (Ctrl+Shift+])", X− "Jog X−
  (Ctrl+Alt+[)", X+ "Jog X+ (Ctrl+Alt+])" (keymap.md, M6); Home "Run the
  homing cycle (Ctrl+H)"; Z and rotary arrows not bound.
- Each click sends one jog move of Jog distance (mm) at Jog speed (mm/s),
  laser off (safety rule 3), as a GRBL jog command (`machine.jog <dx> <dy>`,
  new, M6). A jog whose end point would leave the work area is refused:
  nothing is sent, and the Messages field says "Jog refused: it would leave
  the work area" (own design; safety rule 5 applied to jogs, the same as Go;
  nothing partial is sent).
- Jogs before homing (explicit exemption from the homing requirement, own
  design): the pad and jog chords stay enabled before the machine has homed,
  because jogging is how a user backs the head off a limit switch, brings it
  near the switches before homing, and positions a machine that has no
  homing switches at all. A jog is a short laser-off move relative to where
  the head is, made while the user watches the machine; it is not a job and
  places nothing on the bed. Until homing, though, the reported position is
  not trusted, so the bounds check above cannot be applied: jogs are sent
  unchecked, each click moves at most one Jog distance (mm), and Continuous
  jog (later) stays disabled. The Jog panel shows a `warning-soft` band
  with a `warning` mark above the pad: "Not homed: jogs are not checked
  against the bed. Home the machine to turn the check on." After homing the
  band goes and every jog is bounds-checked. Whether machines without
  homing get another way to trust the position is the open question shared
  with `toolbars.md`.
- Enabled: Ready and Jogging (clicks while jogging queue further moves).
  Everything else disables the pad (Machine states).
- The numeric keypad is not bound (keymap.md lists no keypad chords); this is
  an open question.
- **Continuous jog** (switch, under the pad, later): label "Continuous jog";
  tooltip "Keep moving while a jog button or key is held"; when on, pressing
  and holding a pad button or jog chord moves until release, and release
  cancels the jog at once. Its enabling idea follows the reference [R2][R12],
  but in LaserCraft the switch is always shown once built (GRBL 1.1 jog
  commands are the only jog method we use).

Jog fields (right of the pad), right-aligned labels, one column:

| Field | Label | Tooltip | Default | Range |
|---|---|---|---|---|
| Jog distance | "Jog distance (mm)" | "How far one jog click moves" | 10 | 0.01–1000 |
| Jog speed | "Jog speed (mm/s)" | "Speed of jog moves, framing and Move head; never above the device's maximum speed" | 50 | 1 to the device profile's Max speed |
| Z jog distance | "Z jog distance (mm)" | "How far one Z click moves" | 1 | 0.01–100 |
| Z jog speed | "Z jog speed (mm/s)" | "Speed of Z moves" | 5 | 0.1–100 |

- Jog distance has a small preset button at its right (chevron) offering 0.1,
  1, 10 and 50 (own design).
- These values are stored in the device profile (they belong to the machine,
  not the project); changes are undoable profile commands.
- Narrow panel: the fields move below the pad.

#### 5. Origin row

Three equal ordinary buttons, text only; a label that does not fit wraps to
a second line and the row grows to fit it (Echo of the reference's two-line
buttons [R2]).

| Button | Label | Tooltip | Enabled | Command |
|---|---|---|---|---|
| Set user origin | "Set user origin" | "Store the head position as the user origin" | Ready | `machine.set_user_origin` (new, M6; saved in the device profile) |
| Clear user origin | "Clear user origin" | "Forget the stored user origin" | a user origin is set | `machine.clear_user_origin` (new, M6) |
| Set end position | "Set end position" | "Store the head position as where the head goes after a job" | Ready | later; pairs with After the job (Options tab) |

- Setting the user origin while Place job at is User origin moves the job
  origin marker at once (`canvas.md`).

#### 6. Bottom row (later)

- **Absolute jog** (switch): label "Absolute jog"; tooltip "Jog arrows move
  to the jog distance measured from the machine origin". Echo of the
  reference's move-from-zero switch [R2]. Later. Its moves are absolute
  targets, so they follow the Homing requirement (Move head to), not the
  jog exemption.
- The reference's power field and fire button are not built; the space stays
  empty. A Test fire control is a placeholder only, owner decision
  (decision 7, naming.md).
- Auto focus is not shown (later, owner decision: not standard GRBL).

#### During a job

- The pad, Move head to, Go, the saved position dropdown and the origin row
  are disabled; the head read-out keeps updating.
- Later, with GRBL feed and spindle overrides: the body below the head
  read-out is replaced by two equal columns, "Speed override" and "Power
  override", each with the current percentage centred above five full-width
  ordinary buttons "+10%", "+1%", "100%", "−1%", "−10%" (up and down chevron
  icons; "100%" with a circular-arrow icon) (Match arrangement [R2]; labels
  naming.md). Power override never lifts any burn above the layer's or the
  device's maximum (safety rule 6); the app clamps, the UI shows the clamped
  percentage.

Tier: **Match** (place, block order, pad arrangement, fields beside the pad,
origin buttons row, the during-job override columns), **Echo** (Continuous
jog, Absolute jog), **Own** (live read-out, labels, tooltips, bounds on
jogs, presets, every look).

---

## Console panel

### Reference

- A dock panel in the upper right group, only for GRBL-type devices; not
  usable for proprietary or galvo controllers [R3][R22].
- Almost all of it is a large white bordered read-only log with a vertical
  scrollbar; newest messages at the bottom; the docs' screenshots use the
  ordinary proportional font [R3].
- Content seen: a waiting-for-connection line, `ok`, bracketed version and
  option lines, angle-bracket status replies, echoed commands, multi-line
  bracketed answers; an `error:` line and an explanation under it in dark red
  [R3].
- Bottom row: a single-line input with a grey placeholder asking for
  commands (Enter sends), and a show-all switch at its right [R3]. With the
  switch off the log keeps startup messages, simple commands, alarms, errors
  and status replies but leaves out bulky traffic such as jogs; with it on,
  everything both ways [R3].
- The page lists standard GRBL commands (version, settings, offsets, status,
  unlock, setting changes, resets with a strong warning) [R3]. Macro buttons
  moved out of the console into their own panel in 2.0 [R3][R9]. The 2.1
  changelog says console colors follow the theme and the console colors its
  syntax [R24].

### LaserCraft

Panel title "Console" (the word means only the GRBL console; LaserCraft's
own commands go in the Command line, naming.md). Window → Panels tooltip
"Messages from GRBL and a line for raw commands". Default place: right dock,
upper group, third tab (`layout.md`). M6.

Contents top to bottom: the log (stretches), then the input row.

#### Log

- A `field` area with a thin `border`, read-only, vertical scroll bar at the
  right, lines in `type-mono` (own choice: G-code and status reports line up),
  newest at the bottom.
- Auto-scroll: sticks to the bottom while the view is at the bottom; once the
  user scrolls up it stays put and a small "Jump to latest" button appears at
  the log's lower right (own design).
- Text can be selected and copied (Ctrl+C); a right-click menu offers "Copy",
  "Select all" and "Clear log" (clearing the view only, not the saved
  history) (own design).
- Kept: the last few thousand lines of this connection, in memory only.
- Line kinds:

| Kind | Example | Shown with Show all messages off | Token | Prefix |
|---|---|---|---|---|
| Line you typed | `$$` | yes | `text` | "> " |
| GRBL welcome and bracketed messages | `Grbl 1.1h ['$' for help]`, `[MSG:'$H'|'$X' to unlock]`, `[VER:…]` | yes | `text` | none |
| Settings and other replies to your lines | `$30=1000` | yes | `text` | none |
| `ok` after your lines | `ok` | yes | `text-muted` | none |
| Status reports | `<Idle|MPos:0.000,0.000,0.000|FS:0,0>` | only the reply to a typed `?` | `text-muted` | none |
| Lines LaserCraft sends (job stream, jogs, framing, polls) and their `ok` | `G1 X10 Y5 S400` | no | `text-muted` | "→ " |
| GRBL error reply | `error:9` | yes | `text`, with a `danger` `mark-error` at the line's start | none; our meaning follows on the next line, indented, in `text` |
| GRBL alarm | `ALARM:1` | yes | `text` on a `danger-soft` line band, with a `danger` `mark-error` | none; meaning on the next line, on the same band |
| LaserCraft notes | "Connected to /dev/ttyUSB0 at 115200 baud", "Streaming stopped" | yes | `text-muted`, upright (no italics in chrome) | "— " |
| LaserCraft warnings | "No response for 5 s" | yes | `text` on a `warning-soft` line band, with a `warning` `mark-warning` | "— " |

- Hovering a line shows its time of arrival as a tooltip (no timestamp
  column; own design).
- Errors and alarms are always shown, whatever the switch (Match [R3]).
- No colouring of individual G-code words (own choice, unlike [R24]). Text
  is only ever `text` or `text-muted`; hue appears only as a mark or a line
  band on the kinds above, so color always means one thing and every line
  meets the text contrast target (`visual-language.md`, no colored body
  text, no italics in chrome).

#### Input row

| Control | Label / hint | Tooltip | Enabled | Behaviour |
|---|---|---|---|---|
| Input line | hint "Type a GRBL command" (`text-muted`) | "Send a line to the controller; Up and Down recall earlier lines" | connected, and no job or framing streaming (hint then reads "Wait for the job to end") | Enter sends; Esc clears; Up/Down step through this session's sent lines |
| Show all messages | switch, label "Show all messages", at the row's right end | "Also show status reports and every line LaserCraft sends" | always | off by default; filters the whole log, past lines included |

- Not connected: the log shows "Not connected. Messages from GRBL appear
  here once you connect." in `text-muted` (naming.md, Empty states); the
  input is disabled.
- Focus: the input line takes keyboard focus only when clicked; global
  shortcuts stay ignored while it has focus, except the safety chords (Pause
  key, Ctrl+Break; Job bar).
- Lines that could turn the laser on are refused (safety rule 1: the laser
  fires only in a job, framing or test fire the user started): a typed line
  with `M3` or `M4` together with an `S` above zero, or an `S` above zero
  while a laser mode is active, is not sent, and the log shows "— Not sent:
  this line could turn the laser on. Use a job or framing instead." as a
  LaserCraft warning line (`text` on a `warning-soft` band with a `warning`
  mark). Proposed change to safety.md; owner decision (Open questions).
- GRBL reset commands (`$RST=*`, `$RST=$`, `$RST=#`) first ask a
  confirmation "Reset GRBL settings?" with the buttons "Reset settings"
  (`danger`) and "Cancel" (naming.md, Confirmations; Echo of the reference's
  warning [R3]).
- `$X` typed here is the same as Clear alarm; `$H` the same as Home; `?`,
  `!` and `~` typed alone are sent as GRBL's real-time bytes.
- `$C` toggles GRBL check state (`$C`); the state line then reads Checking
  (Machine states).

Tier: **Match** (place, log plus input plus show-all arrangement, errors
always shown), **Echo** (what the filter hides: the reference keeps status
replies visible, we hide polled status reports and keep only the reply to a
typed `?`, our own choice), **Own** (monospace, line kinds and
tokens, history, refusal of laser-on lines, confirmations, every look).

---

## Macros panel (later)

### Reference

- A dock panel in the upper right group for GRBL-type devices, working only
  while connected [R9].
- A framed, slightly inset area holds one text-only push button per macro,
  sized to its label, from the top left; clicking sends the macro's stored
  lines; a manage button sits at the bottom right outside the frame [R9].
- A manage dialog lists macros with add, edit, delete, move up and move down
  buttons in a column at the right (move buttons greyed when impossible),
  drag to reorder, and OK and Cancel [R9]. A new-macro dialog asks for a
  button label and the macro's lines, one command per line [R9].
- An ampersand in a macro name showed up altered on the panel button [R9].

### LaserCraft

Panel title "Macros"; Window → Panels tooltip "Buttons that send your own
G-code lines". Default place: upper group, fourth tab (`layout.md`). Later.

- Body: a `field-alt` area with a thin `border` and `gap` padding, holding
  one ordinary button per macro, text only, sized to its label, flowing left
  to right and wrapping to new lines (a flow layout; the reference's
  behaviour with many macros is not shown). The area scrolls when full.
- Labels are drawn literally, with no mnemonic underline processing, so an
  ampersand stays an ampersand (own design, avoiding what [R9] shows).
- Bottom right, outside the area: "Edit macros…" (ordinary button).
- Each macro button: tooltip shows its first two lines in `type-mono`, then
  "…" if longer. Enabled while Ready; disabled (with the tooltip "Needs a
  connected, idle machine.") otherwise.
- Clicking sends the lines in order through the same path as the Console's
  input, so the same refusal of laser-on lines applies (safety rule 1) and
  each line appears in the Console as a typed line.
- Empty: "No macros yet. Use Edit macros… to add one." (naming.md).
- Macros are stored in the device profile (they belong to a machine), not in
  the project (own design).

Edit macros and New macro / Edit macro dialogs: see Dialogs below.

Tier: **Match** (button area plus manage button), **Own** (flow layout,
literal labels, storage, safety path, look).

---

## Dialogs

Frames, title bars, footer button order, default and destructive button
styles, and wizard chrome are `dialogs.md`'s. While connected none of these
dialogs covers or blocks the job bar (`layout.md`). This section fixes their
contents.

### Device profiles (dialog, M8)

#### Reference

- Opened by the panel's devices button and automatically on first start
  [R8][R13]. A caption over a large white list; each entry two lines tall
  with a vendor logo at the left, the name on the first line (the default
  one marked with a leading asterisk) and controller and kind on the second;
  the selected entry is a solid blue band [R8].
- Two summary lines for the selected entry (controller and connection;
  work-area size and origin corner) [R8].
- Two framed rows of four equal text buttons: find over USB, create by hand,
  a vendor bridge, import; then make default, edit (re-runs the setup
  wizard), remove, export, greyed with no selection [R8][R14].
- OK saves, Cancel throws away everything changed in the dialog including
  new devices; double-click edits; a right-click menu offers duplicate, set
  icon, rename, then sort by name and by type [R8][R14]. Closing with no
  devices asks whether to create one, make a placeholder or quit [R8][R13].

#### LaserCraft

Title "Device profiles". Opened by Device profiles… (Machine panel, Machine
menu). M8.

| # | Part | Contents |
|---|---|---|
| 1 | List | `field` list with `border`, one entry per profile, two lines each: line 1 the name in `type-body` plus a "default" tag in `text-muted` for the default profile; line 2 in `text-muted`: "GRBL · ‹port or Automatic› · ‹w› × ‹h› mm · origin ‹corner›". Selected entry `accent-soft`. No icons or logos. |
| 2 | Summary | under the list, two lines in `type-body` for the selected profile: "‹port› at ‹baud› baud" and "‹w› × ‹h› mm, origin ‹corner›, max power ‹p›%" |
| 3 | Buttons, row 1 (always enabled) | "Scan for devices…", "New profile…", "Import…" |
| 4 | Buttons, row 2 (need a selection) | "Set as default", "Edit…", "Remove", "Export…" |
| 5 | Footer | OK, Cancel (`dialogs.md` order) |

- Rows 3 and 4 are four equal columns (row 3 leaves its last cell empty),
  each row in a frame with a *thin* `border` on `panel` (never `field-alt`,
  `visual-language.md` group frames) (Echo [R8]).
- Scan for devices… opens Device scan; New profile… opens New device
  profile; Edit… (or double-click) opens the Device profile dialog for that
  profile — not the wizard (own design: a tabbed dialog shows every setting
  at once; the wizard is only for creating). Remove asks "Remove device
  profile?" (naming.md, Confirmations) with "Remove profile" (`danger`) and
  "Cancel". Import… and Export… read and write LaserCraft's own profile file
  (format set by the M8 spec).
- Right-click on an entry: the Device menu (`menus.md`): Duplicate, Rename,
  separator, Sort by name.
- OK commits every change made in the dialog; Cancel discards them all,
  including profiles created through the wizards (Match [R8][R14]).
- Empty list: "No device profiles yet. Use Scan for devices… or New
  profile…." (naming.md). Closing with no profiles is allowed: LaserCraft
  works as a design tool without a device; the Machine panel's Connect is
  then disabled with the tooltip "Add a device profile first." (own design,
  differs from the reference's prompt [R8]).
- Disabled while a job or framing runs (the button that opens it is
  disabled then).

Tier: **Echo** (list, summary, two button rows, OK/Cancel semantics),
**Own** (labels, no logos, Edit opening the tabbed dialog, empty handling).

### Device scan (wizard, M8; scan routine M6)

#### Reference

- A wide wizard with a large faded brand watermark at the left and an inset
  page; footer buttons for back, next or done or add, and cancel [R13].
- Pages: an intro asking to connect over USB; a scan with a small floating
  progress box over an empty results table (device info, type, connection);
  results rows such as a controller with its size, port and baud; the add
  button greyed until a row is chosen [R13].
- Then a name page, a units page with per-second and per-minute speed units
  in two columns, a work-area page with X and Y lengths, and an origin page
  with four corner choices in a 2×2 arrangement and, for GRBL-type devices,
  a switch to home on startup [R13]. Network devices and some firmware
  cannot be found this way [R13].

#### LaserCraft

Title "Device scan". Opened by Scan for devices…. No watermark or artwork
(the reference's is vendor branding). Wizard buttons: Back, Next, Finish,
Cancel (naming.md), placed per `dialogs.md`.

| Page | Contents | Buttons |
|---|---|---|
| 1. Connect | heading "Connect your laser" (`type-heading`), one paragraph: "Plug the laser's USB cable into this computer and switch the laser on. LaserCraft then opens each serial port and listens for GRBL. Nothing moves and the laser stays off." | Back (disabled), Next, Cancel |
| 2. Scan | heading "Looking for GRBL"; an inline progress bar (`accent` fill) with "Checking ‹port› (‹i› of ‹n›)"; under it a results table filling as ports answer: columns "Port", "Controller", "Details" ("/dev/ttyUSB0" · "GRBL 1.1h" · "115200 baud; work area 400 × 400 mm from $130/$131"); a port that refused shows "Permission denied" in `text` with a `warning` mark (`mark-warning`) before it and a "How to fix" link that opens the Serial port access text inline; "Scan again" button under the table | Back, "Create profile" (enabled when a row is chosen), Cancel |
| 3. Name and size | "Name" text field (prefilled "GRBL laser"), "Width (mm)", "Height (mm)" (prefilled from the controller when it reported its travel; hint "Read from the controller" in `text-muted`) | Back, Next, Cancel |
| 4. Origin and homing | heading "Where is the machine origin?"; "Origin corner": four choice dots in a 2×2 square with their labels outside: Rear left, Rear right above, Front left, Front right below (Front left chosen by default); switch "Home on connect" (default off; tooltip "Run the homing cycle each time you connect; needs homing switches") | Back, Next, Cancel |
| 5. Summary | name, port, baud, size, origin, homing, each as "label: value" lines | Back, Finish, Cancel |

- The scan only opens ports, waits for the welcome banner and sends GRBL's
  read-only queries for its build info and settings (`$I`, `$$`); it never
  sends motion or power (safety rule 1). A scan that touches a port another
  window holds skips it with "(in use)".
- The results table selection is `accent-soft`. "Nothing found": the table
  shows one `text-muted` row "No GRBL controller answered. Check the cable
  and power, then click Scan again." and Create profile stays disabled.
- No units page: LaserCraft uses mm and mm/s (decision 14); inch display is a
  later preference.
- Finish returns to Device profiles with the new entry selected; it is saved
  only by that dialog's OK (Match [R8][R14]).
- The same scan routine serves the Port list's "Automatic" entry (M6),
  without the wizard.

Tier: **Echo** (page sequence, results table, 2×2 origin choice, home on
connect), **Own** (title, wording, inline progress, no artwork, no units
page, permission row).

### New device profile (wizard, M8)

#### Reference

- A wizard reached by creating a device by hand or editing one: a device
  type list (a placeholder no-machine type, a custom G-code type, then
  vendors), a connection type list (network, serial, other), a network
  address page for network choices, a name and work-area page, an origin page
  with a home-on-startup switch for GRBL-type devices, and a review page
  [R14].

#### LaserCraft

Title "New device profile". Opened by New profile…. GRBL over USB serial
only (decision 1), so the type and connection-type pages are dropped.

| Page | Contents |
|---|---|
| 1. Connection | one line in `text-muted`: "LaserCraft drives GRBL controllers over USB serial."; then Port (the Machine panel's list, with Refresh ports) and Baud rate |
| 2. Name and size | as Device scan page 3 (no prefill from a controller) |
| 3. Origin and homing | as Device scan page 4 |
| 4. Summary | as Device scan page 5 |

Buttons as in Device scan; Finish returns to Device profiles. Duplicate (the
Device menu) opens this wizard prefilled with a copy, the name ending in
" (copy)".

Tier: **Echo** (wizard idea and page content), **Own** (fewer pages, labels).

### Device profile (dialog, M6 subset, M8 full)

#### Reference

- A tabbed modal titled with the device's name; tabs for dimensions and
  units, basic settings, G-code, and additional settings for GRBL-type
  devices; OK and Cancel; pages made of headed group boxes [R17][R12].
- Dimensions: working width and height, a machine-coordinates switch for
  machines that use an offset coordinate system, a Z group (enable Z,
  Z moves relative to the start height, skipping redundant Z moves, and
  elsewhere on the page a reversed Z direction), a pointer offset with X
  and Y, control units, and an origin corner chosen from four radio buttons
  in a small square [R19].
- Basic settings for GRBL-type devices: a left column of general options (a
  job checklist with an edit button, framing that repeats, an X/Y swap that
  opens an orientation picker, rotary settings saved with projects) and a
  scanning offset table with add, delete, import and export; a right column
  with home on startup, jog commands, a fire button and laser-on framing
  (greyed unless the fire button is on), an out-of-bounds warning, a finish
  position with X and Y, regular position queries, air assist on `M7` or
  `M8`, the S value that means full power, and a transfer mode [R12].
- G-code: start and end scripts in two large text boxes with a note that
  nothing checks them, and switches for overscan as rapid moves, the power
  word on every cut move, reading settings on connect and checking work
  offsets on connect [R18].
- Additional settings: simulation values for time estimates only under a
  red note: maximum X and Y speeds, separate X/Y accelerations for line
  work and for fill work, a rapid (travel) speed, a corner accuracy and a
  minimum corner speed, then a cut speed scale and a global speed scale in
  percent; a full-width button reads the speed and acceleration values from
  the controller [R20].
- An orientation picker shows three bed drawings with arrows meeting at the
  zero corner, separated by rotate arrows [R12].

#### LaserCraft

Title "Device profile: ‹name›" (naming.md). Opened by Edit device profile…
(Machine panel in M6, Machine menu) and Edit… in Device profiles (M8).
Tabs, left to right: "Work area", "Options", "G-code", "Motion" (Match
order, Own names). Each tab is made of headed groups: a `type-heading`
line above a frame with a *thin* `border`, *slightly rounded*, on `panel`
(never `field-alt`; `visual-language.md` group frames) (Echo [R12][R19]).
Footer OK, Cancel. In M6 only the fields marked M6 exist; the others appear
with their milestone, and a tab with nothing yet is not shown.

While connected, Port and Baud rate are read-only (tooltip "Disconnect to
change"); the dialog cannot be opened while a job or framing runs.

**Work area tab**

| Group | Field / control | Kind | Milestone | Notes |
|---|---|---|---|---|
| Size | Width (mm), Height (mm) | number fields side by side | M6 | the bed the canvas draws (open question: project vs profile) |
| Origin | Origin corner | four choice dots in a 2×2 square, labels outside: Rear left, Rear right / Front left, Front right | M6 | Match [R19]; today's status-bar words |
| Axes | Motorized Z axis | switch | M8 | gates the Z jog fields, Z pad buttons, the Z axis group below and the focus height test |
| Axes | Axis orientation… | ordinary button | M8 | opens Axis orientation (below) |
| Z axis | Relative Z moves only | switch, default on; greyed unless Motorized Z axis is on | later | tooltip "Z moves are measured from the height the head starts at, which counts as zero"; Echo of the reference's relative Z switch [R19] |
| Z axis | Skip repeated Z moves | switch, default on; greyed unless Motorized Z axis is on | later | tooltip "Leave out a Z move that would only return to the height already set"; Echo of the reference's Z move optimisation [R19] |
| Z axis | Reverse Z direction | switch, default off; greyed unless Motorized Z axis is on | later | tooltip "Swap which way Z up and Z down move, for machines wired the other way"; Echo [R19] |
| Pointer | Red-dot pointer offset | switch | M8 | |
| Pointer | Pointer offset X (mm), Pointer offset Y (mm) | number fields, greyed unless the switch is on | M8 | Match [R19] |

- The Z axis group is a headed group of its own under Axes (the reference
  keeps its Z switches in a separate Z group below the working size [R19]).
  Whether Z output exists at all is later (no milestone covers Z moves
  yet); until then the group is not shown. Its three labels are proposals
  for `naming.md` (Proposed changes).
- No machine-coordinates (work-offset) switch. The reference's dimensions
  page has a switch for machines that work in an offset coordinate system
  [R19]. Decision for now (own design): LaserCraft writes coordinates as
  `gcode.md` "Coordinates" defines them and assumes GRBL's active work
  coordinate system equals machine coordinates (no `G54` offset). The
  later "Check work offsets on connect" switch (G-code tab) warns when the
  controller reports an offset. Whether a work-offset mode is added is an
  open question; it is listed under Deliberately omitted until then.

**Options tab** — two columns of groups (Match arrangement [R12]).

| Column | Group | Field / control | Kind | Milestone |
|---|---|---|---|---|
| left | Connection | Port, Baud rate | dropdowns (as on the Machine panel) | M6 |
| left | Connection | Home on connect | switch, default off | M6 |
| left | Connection | Streaming: Character counting, Send and wait | two choice dots | M6 (whether the user may force Send and wait is naming.md question 9) |
| left | Connection | Status poll interval (ms), Stall timeout (s) | number fields | M6 |
| left | Power and safety | Full-power S value ($30) | number field; tooltip "Must equal GRBL's $30 (S max)" | M6 (today an export option) |
| left | Power and safety | Max power (%) | number field; tooltip "The device's power ceiling; no layer goes above it" | M6 (safety rule 6) |
| left | Power and safety | Confirm before start, Edit list… | switch with a button on the same row | M8 |
| right | Framing and end of job | Repeat framing | switch; tooltip "Framing repeats until you end it; the laser stays off" | M8 |
| right | Framing and end of job | After the job: Stay, Go to origin, Go to end position | three choice dots | later (proposed change to gcode.md) |
| right | Framing and end of job | End position X (mm), End position Y (mm) | number fields, greyed unless Go to end position | later |
| right | Air assist | Air assist code: M7, M8 | two choice dots | M6 (today an export option) |
| right | Scan shift correction | Scan shift correction (switch); table Speed (mm/s) · Shift (mm); Add row, Delete row, Import…, Export… | switch, table, four buttons in a 2×2 block under the table | M7/M8 |

- There is no switch for the bounds check (always on), no laser-on framing
  setting and no fire button setting (Deliberately omitted).
- A mismatch between Full-power S value ($30) and the controller's `$30`
  (read on connect when known) shows a `warning-soft` band at the top of the
  tab: "GRBL reports $30=‹n›. Power is scaled wrongly until both match."

**G-code tab** (M8)

- Left half: group "Start and end" with "Start G-code" and "End G-code", each
  a large multi-line field in `type-mono` (one line per command). Above them
  an `info-soft` note: "Sent before and after every job. Lines that could
  turn the laser on are refused." (own wording; the reference's note says the
  user is responsible [R18]; ours checks, proposed change to gcode.md and
  safety.md).
- Right half: group "Output and connection" with switches "Overscan as rapid
  moves (G0)", "Repeat S on every G1", "Read GRBL settings on connect",
  "Check work offsets on connect" (later; output-changing ones are a
  proposed change to gcode.md).

**Motion tab** (M8)

One group "Time estimates", one column of right-aligned labels with the
fields after them (the reference aligns its labels in a column ending at
the first field [R20]). Contents in order:

| # | Field / control | Kind | Default | Tooltip | Read from controller |
|---|---|---|---|---|---|
| 1 | note | `info-soft` band, no mark (hints are not alerts, `visual-language.md`): "Used only to estimate job time; the controller keeps its own limits." | — | — | — |
| 2 | Max speed X (mm/s), Max speed Y (mm/s) | two number fields on one row | planning.md's `max_speed` | "Top speed of each axis" | `$110`, `$111` (converted from mm/min) |
| 3 | Travel speed (mm/s) | number field | the lower of the two max speeds | "Speed of moves with the laser off between shapes" | the lower of `$110`/`$111` (GRBL runs `G0` at the axis maximum) |
| 4 | Cut acceleration (mm/s²) | number field | planning.md's `acceleration` | "Acceleration used to time line moves and to size overscan" | the lower of `$120`/`$121` |
| 5 | Fill acceleration (mm/s²) | number field | same as Cut acceleration | "Acceleration used to time fill and image scan lines" | the lower of `$120`/`$121` |
| — | separator | thin `border` line | | | |
| 6 | Time estimate scale (%) | number field, 10–1000 | 100 | "Multiply every estimate by this; raise it if real jobs take longer than the preview says" | not read (no GRBL setting) |
| 7 | Read from controller | ordinary button spanning the group width | — | "Fill the fields above from the connected controller's settings" | enabled only while connected and Ready |

- Rows 2–5 and the full-width button follow the reference's simulation
  group (separate cut and fill accelerations, a travel speed, a scale
  factor and a read button, Match [R20]); the labels, the per-kind single
  acceleration and the one scale factor are ours.
- The reference's note is red text [R20]; ours is not an error, so it is
  an info band. Jog speed's upper limit also uses Max speed (Jog panel).
- Deliberately left out (own design, see Deliberately omitted): the
  reference's corner accuracy and minimum corner speed, because the time
  estimate in `planning.md` already derives the speed through a corner from
  the turn angle and the acceleration and needs no extra cornering inputs;
  separate X and Y accelerations, because the planner times each move with
  one acceleration (the lower axis value is the safe choice); and a second,
  cut-only speed scale, because one scale for the whole estimate is enough
  to correct a machine that runs slower than predicted.
- Rows 3, 5 and 6 are new inputs for the time estimate: proposed change to
  planning.md (Proposed changes). Until planning.md adopts them the job
  preview's estimate uses Max speed and Cut acceleration only, and rows 3,
  5 and 6 are not shown.

**Axis orientation** (dialog, M8)

- Title "Axis orientation". A one-line prompt "How do X and Y run on your
  bed?", then a plain text list of four radio choices ("X right, Y to the
  rear" …) on the left and one live preview on the right: a single bed
  outline whose axis labels and zero corner (in `origin-marker`) follow the
  chosen row. OK, Cancel. Showing a picture of the bed with axes and a zero
  corner is the same idea as the reference's picture picker with rotate
  arrows [R12] (Echo); the text list with one live preview, and no rotate
  buttons, is ours. Whether this dialog is needed at all is an open
  question.

Tier: **Match** (tab order and what each tab holds, the 2×2 origin corner,
two-column options), **Own** (names, groups, the checks, every look).

### GRBL settings (dialog, later)

#### Reference

- A tall dialog opened from the machine-tools menu: a two-column property
  grid with collapsible category bands, each GRBL row showing its `$` number
  in its name, booleans as a switch plus a word; a status line; two rows of
  three buttons for saving to and loading from a file, an axis calibration,
  reading, loading an automatic backup and writing; OK and Cancel [R10].
- Reading drops unwritten edits; writing sends changed values and makes a
  backup each time; a backup picker shows timestamps and a detail pane; an
  axis calibration dialog takes an axis, a requested and an actual distance
  and shows the new step value, with its write button greyed until there is
  a change [R10].

#### LaserCraft

Title "GRBL settings". Opened by Machine → GRBL settings…. Later. Enabled
only while Ready.

- A table with three columns "Setting", "Value", "Unit", grouped under
  full-width group rows (own grouping by GRBL's numbering: "Signals and
  steppers" `$0`–`$6`, "Reports" `$10`–`$13`, "Limits and homing"
  `$20`–`$27`, "Laser" `$30`–`$32`, "Axes" `$100`–`$132`), each group row in
  `panel-title` with a chevron to collapse it.
- Each row: "$‹n› ‹our short name›" (for example "$30 Full-power S value"),
  the value as a number field or, for on/off settings, a switch; the unit in
  `text-muted`. Names are our own short wording of the GRBL configuration
  wiki.
- A changed, unwritten value shows its row in `accent-soft` with a small
  `accent` dot before the name.
- `$32` (laser mode) off shows a `warning-soft` band at the top: "Laser mode
  is off ($32=0). LaserCraft expects laser mode; see GRBL's laser mode
  documentation." A `$30` differing from the profile's Full-power S value
  shows the band from the Options tab.
- Status line under the table in `text-muted`: "Read ‹n› settings from the
  controller at ‹time›." or "‹n› changes not written."
- Buttons, two rows of three equal ordinary buttons: "Read from
  controller", "Write to controller", "Save to file…" / "Load from file…",
  "Restore backup…", "Calibrate steps per mm…". Footer: Close (writing is
  always explicit, so there is no OK/Cancel pair; own design).
- Write to controller asks "Write GRBL settings?" (body: "‹n› settings change
  on the controller. A backup of the current values is kept.") with "Write to
  controller" and "Cancel"; a backup is saved before every write (Match
  [R10]).
- Read from controller with unwritten changes asks first ("Discard ‹n›
  unwritten changes?" — "Discard", "Cancel").
- **Restore GRBL settings** (dialog): a list of backups by date and time on
  the left, the chosen backup's values on the right in `type-mono`; buttons
  "Restore" (loads into the table, still unwritten) and "Cancel".
- **Calibrate steps per mm** (dialog): "Axis" dropdown (X, Y, Z when
  motorized); "Current steps per mm" read-out; "Commanded distance (mm)";
  "Measured distance (mm)"; "New steps per mm" read-out; an `info-soft` hint
  "Jog the commanded distance with the Jog panel, measure what the head
  really moved, then enter it."; buttons "Write to controller" (disabled
  until the new value differs) and "Cancel".

Tier: **Echo** (grid with groups, the button set, backups, calibration
flow), **Own** (groups, names, change marking, warnings, Close).

### Saved head positions (dialog, later)

#### Reference

- A small dialog: a two-column table of names and positions with add and
  delete buttons under it; at the right an edit group with name, X, Y, a Z
  switch and Z (greyed with nothing selected) and a sort-by-name button;
  OK and Cancel [R2].

#### LaserCraft

Title "Saved head positions". Opened by Edit… in the Jog panel. Left: a
`field` table with "Name" and "Position" ("X 10.00, Y 20.00") columns,
selection `accent-soft`; under it "Add" (adds the current head position when
connected, else 0, 0) and "Delete". Right: fields "Name", "X (mm)", "Y (mm)",
switch "Use Z", "Z (mm)" (greyed unless Use Z), all greyed with nothing
selected; under them "Sort by name". Footer OK, Cancel. Stored in the device
profile.

Tier: **Echo** (layout), **Own** (labels, storage).

### Edit macros, New macro, Edit macro (dialogs, later)

#### Reference

- See Macros panel [R9].

#### LaserCraft

- **Edit macros**: title "Edit macros". Left: a `field` list of macro names,
  single selection in `accent-soft`, drag to reorder. Right: a column of
  equal ordinary buttons "Add…", "Edit…", "Delete", a `gap-wide`, "Move up",
  "Move down" (each disabled when it cannot act). Footer OK, Cancel. Delete
  asks "Delete macro?" with "Delete macro" (`danger`) and "Cancel".
- **New macro** / **Edit macro**: title as named. "Button label" text field;
  "Command lines" multi-line field in `type-mono` with the hint "One GRBL
  command per line"; a `warning-soft` line under it when a line could turn
  the laser on ("Line ‹k› could turn the laser on; it will not be sent.");
  footer OK, Cancel.

Tier: **Match** (structure [R9]), **Own** (labels, warning, look).

### Messages and small dialogs used by the Machine panel

| Name | Opened by | Contents | Milestone |
|---|---|---|---|
| No selection | Start, Frame, Frame outline, Save G-code… or job preview with Selected shapes only on and nothing selected | message "Selected shapes only is on, but nothing is selected."; buttons "OK" (keeps the switch on, runs nothing) and "Use all shapes" (turns the switch off, runs nothing); Echo of [R11] | M6 |
| Before you start | Start while Confirm before start is on | title "Before you start"; the user's checklist, one switch per item (all off when it opens); buttons "Start job" (primary, enabled when every item is on) and "Cancel"; nothing moves or fires until Start job (safety rule 1) | M8 |
| Low-power frame | Machine → Frame at low power… | owner decision; `dialogs.md` and `menus.md`; always names its power and says the laser will be on (safety rule 2) | Later, optional |
| Reset GRBL settings? | Console reset commands | see Console panel | M6 |

---

## Safety rules → UI behaviour

| Rule (`docs/spec/safety.md`) | What the UI does | Where in this file |
|---|---|---|
| 1. The laser fires only in a job, framing or test fire the user started | Connect, Clear alarm, Home, jogs, Move head, Device scan and every Console or macro line never turn the laser on; laser-on lines are refused; no button fires anything as a side effect | Alarm banner; Jog panel; Console panel; Macros panel; Device scan |
| 2. Framing is laser-off by default; low power is opt-in per use and shows its power | Frame and Frame outline are laser-off with tooltips saying so; no modifier click; no laser-on framing setting; Frame at low power… only by owner decision, in the menu, through a dialog that names its power | Framing row; Deliberately omitted |
| 3. Jogging and homing never turn the laser on | Jog pad, chords, Move head to, Go to user origin, Home, Saved positions all send laser-off moves; tooltips say "laser off" | Jog panel; Positioning row |
| 4. Generated G-code ends with the laser off and uses `G0` for travel | Nothing in the UI changes the end sequence; start and end G-code lines that could turn the laser on are refused | Device profile, G-code tab |
| 5. Every move is checked against the work area; a failure refuses the whole job with a clear error | Start, Frame, Frame outline and Run G-code file… show the Job refused band and send nothing; Go and Go to user origin need homing since connect and are disabled for targets off the bed; after homing a jog that would leave the bed is refused and nothing is sent; before homing jogs are the stated exemption (single relative laser-off steps, a "Not homed" band) | Notes band; Jog panel; Positioning row |
| 6. Power is clamped to the layer's and the device's maximum | Max power (%) in the device profile; Power override shows the clamped value; power fields clamp (naming.md) | Device profile, Options tab; Jog panel, during a job |
| 7. Stop and Pause always visible and enabled while connected | Job bar (never hidden or covered); Machine panel's big buttons pinned at the top; Pause and Stop enabled in every connected state; safety chords work in text fields and over dialogs | Machine states; Job bar; Machine panel |
| 8. Pause `!`, Resume `~`, Stop soft reset `0x18`, handled before queued work | Pause / Resume and Stop send these commands; the UI never queues them behind other actions | Job row; Job bar |
| 9. A constant-power job labels Pause as motion only and highlights Stop | "Pause (motion only)", "The laser may stay on", `warning-soft` bar, Stop ringed; applies to project jobs with an `M3` layer and to Run G-code file… files that contain `M3` | Constant-power (M3) variant |
| 10. Disconnect, stall, alarm or error stops streaming and is shown with its meaning; no automatic resume | Alarm banner kinds, state words Alarm and No response, Console lines with meanings, no Resume offered | Alarm banner; Machine states; Console panel |
| 11. Lines in GRBL's buffer run after an error, so check first and send hold and reset | Banner says feed hold and reset were sent; Checking state and `$C` | Alarm banner; Console panel |
| 12. No `unsafe`; the device thread never blocks the UI | The UI only reads `Event`s and sends `Request`s; no control waits for a reply (buttons change state when the next event arrives) | egui implementation notes |
| 13. Tests use the simulated GRBL device | UI tests drive the panels against the simulator (egui implementation notes) | egui implementation notes |
| 14. The simulator asserts rule 1 in every test | UI tests for Connect, Clear alarm, jog, Console and macros run with the simulator's assertion on | egui implementation notes |

---

## Deliberately omitted (decision 1 and safety)

- Send (to a controller without running), save and run for proprietary
  controller files, the controller file list panel and its buttons, the
  controller status information dialog [R1][R28][R29].
- Galvo-only items: the galvo framing state, the red-light switch, cylinder
  correction, the galvo variants of the panels [R1][R2].
- The fourth axis read-out and its settings (U axis), dual-source laser
  controls, network devices (address page, network port, bridge button),
  device logos and the set-icon menu item, sort by type [R2][R8][R12][R14].
- The modifier-click that frames with the laser on, the laser-on framing
  device setting, the fire button and its power field (a Test fire
  placeholder stays an owner decision), the out-of-bounds warning switch
  (the check is always on) [R1][R2][R12].
- The devices button's right-click reconnect (Connect / Disconnect is
  visible instead) [R1].
- The speed-unit choice in the setup wizard (mm/s only, decision 14) [R13].
- Device settings with no GRBL meaning in our scope: the DTR switch, fast
  whitespace scanning, G-code clustering, tab pulse width, the rotary
  settings switch (M9 decides rotary storage) [R12] — open question whether
  any returns.
- Auto focus (owner decision; not standard GRBL) [R2].
- The machine-coordinates (work-offset) mode switch: LaserCraft assumes no
  work offset for now (Device profile, Work area tab; open question) [R19].
- Simulation inputs the time estimate does not need: corner accuracy,
  minimum corner speed, separate X and Y accelerations and a cut-only speed
  scale (Device profile, Motion tab gives the reasons) [R20].
- The reference's vendor-specific Z switch on its dimensions page has no
  GRBL meaning and is not built [R19].

---

## egui implementation notes

No numbers; sizes come from the size tokens and `Style` spacing.

- **State in, actions out.** The app hands the shell a read-only
  `MachineView` each frame (state, port and baud, progress, the constant-power
  flag and layer names, the current alarm or refusal, head position, user
  origin set, port list, profile list). The shell draws from it and emits
  `Action`s (`machine.*`, `device.*`, `job.*`, `ui.*`). Enabling follows the
  Machine states table, written as one pure function from `MachineView` to a
  set of enabled controls, unit-tested per state.
- **No waiting.** A click emits an action and returns; the button reflects
  the new state only when the app's next `MachineView` arrives from a device
  `Event` (safety rule 12). A pressed Stop shows `pressed` until then.
- **Pinned top.** The Machine panel draws blocks 1–3 above its
  `ScrollArea`, so they never scroll.
- **Job bar.** Drawn in an `egui::Area` on the foreground order after every
  window while connected (`layout.md`); its buttons call the same action
  helpers as the panel.
- **Switches.** One shared switch widget (`visual-language.md`) for every
  switch here; no `Checkbox`.
- **Console log.** Lines kept in a ring buffer in the app (not the UI);
  the view filters by kind for Show all messages; `ScrollArea::stick_to_bottom`
  while at the bottom.
- **Flow layout for macros.** `horizontal_wrapped` is fine here (it is
  neither a toolbar nor the palette strip, which `layout.md` keeps
  unwrapped).
- **Tests** (`crates/lasercraft-ui/tests/`): for each machine state, the
  enabled set matches the table; Stop is clickable with each dialog of this
  file open; the constant-power labels appear for an M3 job; the Serial port
  access box shows the right group for an Arch and a Debian `os-release`;
  Console refuses a laser-on line; all run against the simulated GRBL device
  with its rule-1 assertion (safety rules 13–14).

---

## Current state → target

| Today (current-ui.md) | Target | Where |
|---|---|---|
| No Machine, Jog, Console or Macros panel; the right dock holds only Layers (`shell.rs:262-270`) | Machine panel in the lower dock group (M6), Jog and Console as tabs of the upper group (M6), Macros later | this file; `layout.md` |
| No job bar; no Pause or Stop anywhere | Job bar at the right end of toolbar row 1 while connected | Job bar; `layout.md` |
| No machine state, port, baud or connection UI; `lasercraft_device` already defines `Request`, `Event` and `MachineState` (`device.md`) | State line, Machine states table, Connection row | Machine states; Machine panel |
| Save G-code… exists only in the File menu, as a typed-path window titled "Save G-code" (`shell.rs:581-621`) | Also a big button in the Framing row (same `ui.export_gcode` action) | Framing row |
| The bottom panel called "Console" is LaserCraft's command line with a monospace log (`shell.rs:556-579`) | Renamed Command line (proposed in naming.md/ui-shell.md); "Console" becomes the GRBL panel here | Console panel |
| "Unsaved changes" uses `egui::Modal`, which blocks all input (`main.rs:185-210`) | While connected no dialog may block the job bar (`dialogs.md`); must change before M6 ships | Job bar |
| All shortcuts are ignored while a text field has focus (keymap.md) | Pause key and Ctrl+Break act everywhere | Job bar; Proposed changes |
| The work area lives in the project (`project.set_work_area`, status bar "Work area … origin …", `shell.rs:624-652`); there are no device profiles | Device profile holds work area, origin, port, baud, limits (M6 one default profile, M8 many); open question on project vs profile | Device profile |
| Export options (`s_max`, `air`, `max_power`, `finish`) are command arguments only (`gcode.md`) | Full-power S value ($30), Air assist code, Max power (%), After the job in the device profile | Options tab |
| `constant_power` exists in the layer model but not in the editor (current-ui.md 1.6) | Constant power (M3) switch in the layer editor (`layers-panel.md`) feeding the constant-power variant here | Constant-power (M3) variant |
| No `Visuals`, no switches (check boxes everywhere), no icons (current-ui.md 2, 3) | Tokens, the shared switch, own icons (`visual-language.md`, `icons.md`) | Shared look |
| `device.md` has `PermissionDenied { port, help }` text; nothing shows it | Serial port access help box | Serial port access help |

---

## Proposed changes to other specs

Each is labelled per decision 15; the named spec is updated before code.

- **Proposed change to safety.md**: rule 1 gains "Lines typed in the Console,
  sent by a macro, or set as start or end G-code that could turn the laser on
  are refused" (owner decision); rule 7 names the job bar (as `layout.md`
  already proposes) and states that Pause and Stop are enabled in every
  connected state, including Connecting… and No response.
- **Proposed change to device.md**: `Request` gains `Jog { dx, dy, dz, speed }`,
  `MoveTo { x, y }`, `Home`, `Frame { path }`, `Start { job }`, `ClearAlarm`,
  `RawLine { text }`, `SetCheckMode`; `Event` gains `Progress { sent, total,
  elapsed, remaining }`, `ErrorReply { code, line }`, `Line { kind, text }` for
  the Console, `Banner { version }`; the app-level states Connecting…,
  Framing and No response are named alongside `MachineState`.
- **Proposed change to keymap.md**: the Pause key and Ctrl+Break work while a
  text field has focus (also proposed by `layout.md`); Alt+Shift+S is marked
  not bound (no separate send step; naming.md); numeric-keypad jogging is
  listed as not bound pending the open question.
- **Proposed change to gcode.md**: Save G-code… applies Place job at
  (Absolute coordinates always; User origin when one is set; Current
  position only while connected with a known head position); "After the job"
  replaces the finish option (already in naming.md); start and end G-code
  lines are checked by the verifier's laser rules.
- **Proposed change to project-file.md**: the project stores Place job at
  and Job reference point; Selected shapes only and Reference point from
  selection are not stored.
- **Proposed change to planning.md**: the job origin offset uses Place job
  at, Job reference point, Selected shapes only and Reference point from
  selection; the frame path (box and outline) is planned and bounds-checked
  like a job.
- **Proposed change to planning.md** (time estimate inputs): `Machine`
  gains `travel_speed` (mm/s, default the lower axis maximum; travel moves
  are timed at it instead of `max_speed`), `fill_acceleration` (mm/s²,
  default `acceleration`; fill and image scan lines, including overscan,
  use it) and `estimate_scale` (%, default 100; multiplies the total). They
  come from the device profile's Motion tab (M8).
- **Proposed change to ui-shell.md**: a "Machine (M6)" section listing the
  Machine, Jog and Console panels, the job bar and the Serial port access
  help, pointing to this file.
- **Proposed change to hardware-testing.md**: new checks — a constant-power
  (`M3`) job shows "Pause (motion only)" and Stop turns the beam off; Stop
  works with each dialog open; the Console refuses a laser-on line and
  nothing fires; Clear alarm fires nothing and does not resume.
- **Proposed change to linux-serial.md**: note that the in-app box shows the
  same commands with Copy buttons and the ModemManager hint.
- **Proposed change to hardware-testing.md** (H2): unplugging USB while idle
  shows the state word "Not connected" (not "Disconnected") and the job bar
  is hidden; the Machine panel keeps disabled Pause and Stop placeholders.
- **Proposed change to naming.md**: add the labels and messages this file
  introduces, so that they come from naming.md like every other label:
  "End framing", "Show in console", "Show in job preview", "Jump to latest",
  "Scan again", "Try again", "Clear log", "Reset settings", "Delete macro",
  "Start job", "Command lines", "No serial ports found", "Jog refused: it
  would leave the work area", "Home the machine first.", "Not homed: jogs
  are not checked against the bed. Home the machine to turn the check on.",
  the Z axis switches ("Relative Z moves only", "Skip repeated Z moves",
  "Reverse Z direction"), the Motion tab fields ("Travel speed (mm/s)",
  "Cut acceleration (mm/s²)", "Fill acceleration (mm/s²)", "Time estimate
  scale (%)"), the GRBL settings group names (see GRBL
  settings), and the Console refusal note ("— Not sent: this line could turn
  the laser on. Use a job or framing instead."). Until naming.md lists
  them they are proposals, not binding labels (open question 16).

---

## Open questions

1. Work area: does it belong to the project (today) or the device profile
   (this file's M6 proposal), or does the profile set it and the project keep
   a copy?
2. Head position and Move head to use machine coordinates; should the
   status bar's Pointer position use the same frame (so the user can compare)
   or keep the canvas frame (`canvas.md`)?
3. Should the Console, macros and start/end G-code refuse lines that could
   turn the laser on (proposed), only warn, or trust the user?
4. Clear alarm: banner and Machine menu only (proposed), or also a Console
   button (naming.md question 10)?
5. Port "Automatic": acceptable to open every USB serial port in turn on
   Connect (some boards reset when opened, `linux-serial.md`), or only the
   last port that worked?
6. Show head marker: is following the head live during a job wanted, or
   should it freeze during a job as in the reference, so the user watches the
   machine rather than the screen?
7. Repeat framing: end at the end of the current pass ("End framing") as
   proposed, or stop at once?
8. Numeric keypad jogging (the reference uses 8, 2, 4, 6 and diagonals with
   canvas focus): bind it, and how to avoid clashes with typing?
9. Should Machine jump to the front of its group on connect (`layout.md`
   question 3)?
10. The job bar's sent share: keep it (layout.md question 4)?
11. Axis orientation dialog: needed at all for GRBL lasers, or covered by
    Origin corner?
12. Device settings omitted for now (DTR, fast whitespace scan, clustering):
    does any of them matter for the owner's machines?
13. Saved head positions and macros stored per device profile (proposed) or
    per user?
14. Answered by `visual-language.md`: no `job-origin-marker` token; the job
    origin marker uses `origin-marker` in its hollow form (`canvas.md`).
15. Test fire and Frame at low power…: anything beyond the placeholder
    (naming.md questions 12–13)?
16. The labels and messages listed under "Proposed change to naming.md"
    are not in naming.md yet: does the owner accept them as worded?
17. Machines without homing switches (shared with `toolbars.md` question
    15): keep Go, Go to user origin and bounds-checked jogs unavailable on
    them, with jogs always in the "Not homed" exemption (proposed, safest),
    or let the user confirm the head's position once (for example after
    setting an origin) through a device-profile flag saying the machine has
    no homing? The same answer applies to Place job at Current position
    before homing.
18. Work-offset mode: should LaserCraft ever support machines that work in
    an offset coordinate system (the reference's machine-coordinates switch
    [R19]), or keep assuming no offset and only warn?
19. Z control: is Z output (relative Z, skipping repeated Z moves, reversed
    Z) wanted at all, and in which milestone?

---

## Sources

All LightBurn pages read 2026-10-21, through the research notes for this
file (laser, move, console, macros and devices notes; preview, optimization
and device settings notes; main window layout notes; status bar and
preferences notes; the current UI audit).

1. [R1] https://docs.lightburnsoftware.com/2.1/Reference/LaserWindow/ —
   job-control panel place, status line and progress, big button rows,
   framing buttons and their glyph-only difference, machine-file buttons,
   home and origin buttons, placement dropdown, 3×3 job origin grid and its
   greying, selection switches, last-position button, optimization button,
   device row, laser-on framing modifier, reconnect by right-click, list
   highlight look, keyboard pause and stop.
2. [R2] https://docs.lightburnsoftware.com/2.1/Reference/MoveWindow/ — jog
   panel place and tabs, position read-outs, go-to fields, saved positions
   and their dialog, jog pad arrangement, distance and speed fields,
   continuous jog, focus button, origin buttons, move-from-zero switch, fire
   control, during-job speed and power adjustments, keyboard and keypad
   jogging.
3. [R3] https://docs.lightburnsoftware.com/2.1/Reference/ConsoleWindow/ —
   console place, log, error lines with explanations, input line, show-all
   switch and what it hides, startup message, standard GRBL commands, macro
   buttons moving out in 2.0.
4. [R4] https://docs.lightburnsoftware.com/2.1/GetStarted/JobControl/ — job
   buttons and resuming, lower dock tabs, roomy grid layouts.
5. [R5] https://docs.lightburnsoftware.com/2.1/GetStarted/FramingBeginner/ —
   frame outline on the canvas, frame speed from the jog speed, repeating
   frame ended by a dialog.
6. [R6] https://docs.lightburnsoftware.com/2.1/GetStarted/ConnectingToLaser/
   — GRBL variant of the button rows, port list states and empty list,
   disabled buttons while disconnected, reconnect.
7. [R7] https://docs.lightburnsoftware.com/2.1/Reference/CoordinatesOrigin/
   — placement modes, job origin grid and canvas marker, user origin
   buttons.
8. [R8] https://docs.lightburnsoftware.com/2.1/Reference/Devices/ — device
   list, summary, button rows, OK/Cancel meaning, right-click menu, empty
   guard.
9. [R9] https://docs.lightburnsoftware.com/2.1/Reference/MacrosWindow/ —
   macros panel, manage and new-macro dialogs, ampersand quirk.
10. [R10] https://docs.lightburnsoftware.com/2.1/Reference/MachineSettings/
    — controller settings grid, buttons, backups, axis calibration.
11. [R11] https://docs.lightburnsoftware.com/2.1/Reference/CutSelectedGraphics/
    — selection-only output, origin from the selection, nothing-selected
    message and its two buttons.
12. [R12] https://docs.lightburnsoftware.com/2.1/Reference/DeviceSettings/BasicSettings/
    — basic settings for GRBL-type devices (checklist, repeat framing, axis
    swap and its picker, scanning offsets, home on startup, jog commands,
    fire button, laser-on framing, bounds warning, finish position, position
    queries, air assist, S value, transfer mode).
13. [R13] https://docs.lightburnsoftware.com/2.1/GetStarted/FindMyLaser/ —
    discovery wizard pages, scanning progress, results table, name, units,
    work area and origin pages, first-start opening.
14. [R14] https://docs.lightburnsoftware.com/2.1/GetStarted/CreateManually/ —
    manual wizard pages, editing through the wizard, greyed second button
    row.
15. [R15] https://docs.lightburnsoftware.com/2.1/Troubleshooting/ConnectionTroubleshooting/
    — failed-transfer message idea; no permission handling shown.
16. [R16] https://docs.lightburnsoftware.com/2.1/Reference/BeginnerMode/ —
    what the reduced mode hides in the job-control panel.
17. [R17] https://docs.lightburnsoftware.com/2.1/Reference/DeviceSettings/ —
    device settings dialog, tabs, title with the device name.
18. [R18] https://docs.lightburnsoftware.com/2.1/Reference/DeviceSettings/GCode/
    — start and end scripts and their note, G-code switches.
19. [R19] https://docs.lightburnsoftware.com/2.1/Reference/DeviceSettings/DimensionsUnits/
    — working size, machine-coordinates (offset) switch, Z group (relative
    Z, skipping redundant Z moves, reversed Z), pointer offset, units,
    origin corner choice.
20. [R20] https://docs.lightburnsoftware.com/2.1/Reference/DeviceSettings/AdditionalSettings/
    — estimate-only motion values (axis speeds, cut and fill
    accelerations, rapid speed, corner values, two speed scales), red note,
    read from controller.
21. [R21] https://docs.lightburnsoftware.com/2.1/Reference/OptimizationSettings/
    — the optimization button's place in the job-control panel.
22. [R22] https://docs.lightburnsoftware.com/2.1/Reference/UI/ — default
    window map: lower group with job control and library, upper group tabs.
23. [R23] https://docs.lightburnsoftware.com/2.1/GetStarted/UITour/ —
    job-control panel contents in the tour.
24. [R24] https://docs.lightburnsoftware.com/2.1/NewFeatures/NewIn2.1/ —
    rotary icon on Start, source errors in the panel, fire blocked while
    busy, console colors and syntax coloring.
25. [R25] https://docs.lightburnsoftware.com/2.1/Reference/UI/WindowMenu/ —
    the job-control, move, console and macros panels as separate show/hide
    entries.
26. [R26] https://docs.lightburnsoftware.com/2.1/Reference/UI/StatusBar/ —
    device messages in the notification area (background for the Messages
    field, `layout.md`).
27. [R27] https://docs.lightburnsoftware.com/2.1/Guides/CustomizingTheLightBurnWindow/
    — panels can be hidden, moved and tabbed (why the job bar is needed).
28. [R28] https://docs.lightburnsoftware.com/2.1/Reference/FileListWindow/ —
    controller file list (omitted).
29. [R29] https://docs.lightburnsoftware.com/2.1/Reference/GetControllerInfo/
    — controller status information (omitted).
30. [R30] https://docs.lightburnsoftware.com/2.1/Reference/SettingsPreferences/
    — preferences for the rotary switch, saving and defaulting the selection
    switches, and keeping Start from holding keyboard focus.

Existing specs and documents used: `docs/spec/safety.md` (rules 1–14),
`docs/spec/device.md` (threading, `Request`/`Event`, `MachineState`,
permission help), `docs/spec/gcode.md` (coordinates, `$30`, `M3`/`M4`, air
assist, finish option), `docs/spec/keymap.md` and
`crates/lasercraft-ui/keymap/default.keymap` (Alt+S, Alt+R, Alt+Shift+R,
Ctrl+Shift+A, Ctrl+H, Ctrl+Shift+[ ], Ctrl+Alt+[ ], Alt+L, Pause,
Ctrl+Break, all M6; Alt+Shift+L and Alt+P bound today), `docs/spec/ui-shell.md`,
`docs/spec/canvas.md`, `docs/spec/preview.md`, `docs/spec/planning.md`,
`docs/linux-serial.md` (groups, commands, Flatpak override, ModemManager),
`docs/spec/project-file.md`, `docs/spec/layers.md`,
`docs/hardware-testing.md` (H2, H4), `docs/clean-room.md`, and the style
files `layout.md`, `menus.md`, `naming.md`, `toolbars.md`, `layers-panel.md`,
`dialogs.md`, `visual-language.md`, `icons.md`, `canvas.md` (style),
`later-windows.md`, `job-preview.md`, `property-bar.md`.
Open standards: GRBL v1.1 interface (status reports, `ok`/`error:`/`ALARM:`
lines, welcome banner, real-time bytes, streaming protocols, alarm and error
codes): https://github.com/gnea/grbl/wiki/Grbl-v1.1-Interface; GRBL v1.1
commands (`$X`, `$H`, `$C`, `$I`, `$$`, `$RST`, jogging, feed hold, soft
reset, overrides): https://github.com/gnea/grbl/wiki/Grbl-v1.1-Commands; GRBL
v1.1 configuration (`$0`–`$132`): https://github.com/gnea/grbl/wiki/Grbl-v1.1-Configuration;
GRBL v1.1 laser mode (`$32`, `M3`/`M4`): https://github.com/gnea/grbl/wiki/Grbl-v1.1-Laser-Mode;
os-release (`ID`, `ID_LIKE`); WCAG 2.x (contrast 4.5:1 for text).

Own design: every label, tooltip and message; the machine state set, its
dots and every enabling rule; the pinned top of the Machine panel and the
notes band below the big rows; Connect / Disconnect, Baud rate, Refresh ports
and the Automatic port; the live Show head marker switch; the constant-power
variant; the Alarm banner and Clear alarm; the Serial port access help box;
the job bar's buttons and interactions; the live head read-out, jog bounds
and presets; the homing requirement for Go and Go to user origin and the
"Not homed" jog exemption; the no-work-offset assumption; the Z axis switch
wording; the Motion tab's single acceleration per kind and single estimate
scale; the Console's monospace log, line kinds, history and refusal of
laser-on lines; the macro flow layout and storage; the Device profiles,
Device scan, New device profile, Device profile, Axis orientation, GRBL
settings and Saved head positions contents and labels; the safety mapping;
all looks through tokens.
