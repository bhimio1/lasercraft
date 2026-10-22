# Parity checklist

LaserCraft aims to feel familiar to people who know the reference
application. The parity checklist tracks, page by page of the reference's
public documentation, what LaserCraft does, in our own words, and lets a
tool check those claims against the code.

## The checklist

`docs/parity/checklist.json`:

```json
{
  "format": "lasercraft-parity",
  "version": 1,
  "pages": [
    {
      "page": "2.1/Reference/TraceImage",
      "feature": "Trace an image into vector outlines",
      "status": "done",
      "spec": "docs/spec/trace.md",
      "commands": ["image.trace"],
      "actions": ["ui.trace_image"],
      "keys": [],
      "note": ""
    }
  ]
}
```

- `page`: the documentation page, as a path under
  `https://docs.lightburnsoftware.com/` (no trailing slash). Each page
  appears once.
- `feature`: what the page is about, in our own words (never copied).
- `status`: `done` (LaserCraft does what the page describes, as our spec
  says), `partial` (some of it; `note` says what is missing), `missing`,
  `out_of_scope` (not planned: hardware or services LaserCraft does not
  support; `note` says why) or `guide` (an explanation or how-to, not a
  feature).
- `spec`: our spec that covers it (required for `done` and `partial`).
- `commands`: registry commands that implement it; `actions`: `ui.*`
  actions; `keys`: default chords that must be bound (as written in
  `default.keymap`). All optional.

## The tool

`cargo xtask parity` (an `xtask` crate in the workspace; `.cargo/config.toml`
aliases it):

1. Reads the checklist and checks it: known status, each page once, a spec
   file that exists for `done` and `partial`, every command in the
   built-in registry, every action in the UI's action list or the
   registry, every key bound in `crates/lasercraft-ui/keymap/default.keymap`.
2. Collects every documentation link in `docs/spec/` (recursively) and
   checks that each linked page is in the checklist, so a spec cannot cite
   a page without it being tracked.
3. Prints the problems, then a summary: pages per status and the share
   done (`done` + half of `partial`, over all pages except `guide` and
   `out_of_scope`).
4. `--markdown <path>` writes the report (summary and one table row per
   page: feature, linked page, status, note) to a file; `--check <path>`
   fails when that file differs from the report it would write.

The exit status is non-zero when any check fails. CI runs `cargo xtask
parity --check docs/parity.md`.

## Sources

- Own design. The page list is the reference's public documentation as
  our specs cite it: https://docs.lightburnsoftware.com/2.1/ (pages listed
  in the checklist).
