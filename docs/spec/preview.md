# Spec: job preview

Milestone: M4. Shows the planned job (`docs/spec/planning.md`) without any
hardware.

## Opening

- Alt+P (`ui.preview`) or View → Job Preview… opens the preview window. It
  works with no device connected.
- The app plans the job when the window opens and again whenever the project
  changes while it is open. The UI never plans: it draws the moves and text
  the app hands it (`ARCHITECTURE.md`: the UI does not depend on
  `lasercraft-plan`).

## Drawing

- The work area outline, the job's moves on top:
  - burn moves as solid lines in their layer's palette color;
  - unlit moves (overscan, tab and fill gaps) as thin dashed orange lines;
  - travel moves as thin dashed red lines, hidden when "Show travel" is off.
- The view fits the work area and the job; the mouse wheel zooms about the
  pointer.
- A time slider scrubs through the job: only moves that have started by the
  chosen time are drawn (the current one up to where the head is), and the
  head position is marked.

## Text

- Burn, unlit and travel distances with their times, the total estimated
  time, the job bounds (overscan included) and the moves count.
- Whether the job fits the work area, or the bounds error
  (`Plan::check_bounds`) in red.
- The plan's warnings.

## Not yet

- Outline/filled canvas display (Alt+Shift+W) is left for later; the preview
  already shows fill scan lines.
- Planning runs on the UI thread; large jobs may move to a worker thread
  (`ARCHITECTURE.md`, threading model).

## Sources

- A preview window opened with Alt+P, cut and travel moves in different
  colors, a time slider through the job, cut and travel distances with time
  estimates and the total, the work area border:
  https://docs.lightburnsoftware.com/2.1/Reference/Preview/ (read 2026-10-21)
- Own design: colors and dash styles, the layout, recomputing on change.
