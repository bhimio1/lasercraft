# Spec: job preview

Milestone: M4; restyled to `docs/spec/style/job-preview.md`. Shows the planned job (`docs/spec/planning.md`) without any
hardware.

## Opening

- Alt+P (`ui.preview`) or View → Job preview… opens the preview window, a
  movable, resizable, non-modal window titled "Job preview" (not
  collapsible). It works with no device connected.
- The app plans the job when the window opens and again whenever the project
  changes while it is open. The UI never plans: it draws the moves and text
  the app hands it (`ARCHITECTURE.md`: the UI does not depend on
  `lasercraft-plan`).

## Drawing

- The drawing is at the top of the window, edge to edge: the `canvas`
  surround, the `bed` sheet with the canvas grid, the work area edge
  (`bed-edge`) and the machine-origin marker as on the canvas, then the
  job's moves on top:
  - burn moves as solid lines in their layer's palette color (`move-burn`);
  - unlit moves (overscan, tab and fill gaps) as hairline dotted light grey
    lines (`move-unlit`);
  - travel moves as hairline dashed mid grey lines (`move-travel`), hidden
    when "Show travel" is off. Travel is never red.
- The view fits the work area and the job; the mouse wheel zooms about the
  pointer; middle-drag or Space+drag pans; resizing the window keeps the
  zoom.
- A time slider scrubs through the job: only moves that have started by the
  chosen time are drawn (the current one up to where the head is). The
  simulated head is always shown once there is a job: an `accent` dot with
  a white halo.

## Text

Below the drawing, top to bottom:

- Time row: "Time (s)", the slider, the time value (one decimal, no
  suffix) and "Show whole job" (disabled at the end).
- Summary: "Burn ‹d› mm (‹t›), unlit ‹d› mm (‹t›), travel ‹d› mm (‹t›)",
  "Estimated time ‹t› for ‹n› moves", and the job bounds (overscan
  included) in small muted text. Times are h:mm:ss.
- Check and notes: "Fits the work area." with a success mark, or the bounds
  error (`Plan::check_bounds`) as a "Cannot run: …" band (`danger-soft` with
  a `danger` mark); the plan's warnings as "Warning: …" bands
  (`warning-soft`).
- Playback row: the "Show travel" switch and, at the right, "‹time› of
  ‹total› · move ‹n› of ‹N›" in small muted text.
- "Close" at the bottom right.

Wording and layout: `docs/spec/style/job-preview.md`.

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
