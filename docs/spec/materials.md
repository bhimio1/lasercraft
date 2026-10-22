# Spec: material libraries

Milestone: M8. A material library stores layer settings that worked on a
material, so they can be applied to a layer later. LaserCraft has its own
library format and its own starter data; no other program's library files
are read or written (`docs/clean-room.md`).

## Entries

An entry holds:

| Key | Meaning |
| --- | --- |
| `id` | Whole number, unique in its library; assigned when the entry is made |
| `material` | First tree level, e.g. "Plywood" (1–60 characters) |
| `thickness` | mm, above 0, or absent |
| `group` | Used instead of a thickness when `thickness` is absent ("Thickness does not apply"), e.g. "Any" |
| `description` | The entry's name in the tree, e.g. "Cut" (1–60 characters) |
| `settings` | A layer object (`docs/spec/layers.md`) whose `color`, `name`, `output` and `show` are ignored |

- Exactly one of `thickness` and `group` is present.
- Two entries of one library may not share material, thickness (or group)
  and description.
- Sort order: material, then thickness (numerically; groups after
  thicknesses, alphabetically), then description, each case-insensitive.

## Libraries

- A library has a `name`, an optional `device` (a device profile's name,
  `docs/spec/devices.md`) and its entries.
- File format `.lcmat`: `{"format": "lasercraft-materials", "version": 1,
  "name": …, "device": … or null, "next_id": …, "entries": [ … ]}`,
  written with sorted entries. Unknown keys are ignored; a newer version is
  refused with a message.
- Several libraries can be open; one is **active**. Material commands act
  on the active library. A library remembers its file once it has been
  opened from or saved to one; changes are kept in memory until "Save
  library" (the library shows " •" while unsaved).
- **Starter library**: one read-only library named "LaserCraft starter"
  is always open (it cannot be closed, renamed or edited; "Save library
  as…" makes an editable copy). Its entries are our own conservative
  starting points for a typical 5–10 W diode laser (own data, from general
  physics of absorbed energy, not from any other product's library); every
  description says to run a Speed and power test first. It lists: plywood
  3 mm (engrave, score, cut), basswood 3 mm (engrave, cut), MDF 3 mm
  (engrave, cut), cardboard 2 mm (cut), leather 2 mm (engrave), black
  anodized aluminum (engrave), slate (engrave) and paper 0.2 mm (cut).
- The app remembers which library files were open and which was active in
  `workshop.json` in the configuration folder (`docs/spec/devices.md`) and
  reopens them at start. Files that no longer open are dropped from the
  list with a message.
- When the project's device profile changes (`project.set_device`), the
  first open library whose `device` matches becomes active.

## Following an entry

- A layer may **follow** an entry: its settings are the entry's, and the
  layer editor shows them read-only with an `info-soft` band "Follows
  material ‹description› (‹library›)" and a "Stop following" button.
- The project stores, per layer, `follows`: `{"library": …, "material":
  …, "thickness": … | "group": …, "description": …}` (`docs/spec/
  project-file.md`). It survives entry ids changing between files.
- `layer.set`, `layer.set_all` with other settings and
  `layer.copy_settings` onto a following layer are refused: "Layer ‹nn›
  follows material ‹description›. Stop following it first." (The inverse
  of a material command may still restore a following layer exactly.)
- When an entry changes (`material.update`, `material.set`) every layer of
  the open project that follows it gets the new settings in the same
  command, which is undoable for the project's part.
- When an entry is deleted, layers that follow it keep their settings and
  stop following it.
- If the library a layer follows is not open, the band reads "Follows a
  material from ‹library›, which is not open" (`warning-soft`) and the
  layer keeps its last settings.

## Commands

Material commands act on the active library. Library changes are not
undoable (they are saved explicitly); the project part of a command (a
layer's settings or `follows`) is undoable and its inverse restores the
layers exactly. Editing commands refuse the starter library: "The starter
library cannot be changed. Use Save library as… to make your own copy."

| Command | Effect |
| --- | --- |
| `library.new <name>` | Open a new, empty library and make it active. |
| `library.open <path>` | Open a `.lcmat` file and make it active (already open: just make it active). |
| `library.save [path]` | Write the active library to its file, or to `path` (then that is its file). Without a file and a path: error "Choose a file with Save library as…". |
| `library.close [name]` | Close the active (or named) library; refused while it has unsaved changes unless `force` is given as a second argument. |
| `library.use <name>` | Make an open library active. |
| `library.rename <name>` | Rename the active library. |
| `library.set_device <name\|none>` | Tie the active library to a device profile. |
| `library.merge <path>` | Add every entry of another file; entries whose key already exists are skipped and counted. |
| `material.add <material> <thickness\|-> <description> <color> [group=<name>]` | New entry from layer `color`'s settings; `-` means no thickness (then `group=` is required). Returns the new id. |
| `material.apply <id> <color>` | Copy the entry's settings into the layer (keeping its name, output and show). Undoable. |
| `material.follow <id> <color>` | As apply, and the layer follows the entry. Undoable. |
| `material.unfollow <color>` | The layer stops following; its settings stay. Undoable. |
| `material.update <id> <color>` | Replace the entry's settings with the layer's; followers change too. |
| `material.set <id> <key> <value>` | Change one setting of the entry (keys as `layer.set`); followers change too. |
| `material.describe <id> <material> <thickness\|-> <description> [group=<name>]` | Change the entry's material, thickness or group and description; followers' `follows` change with it. |
| `material.duplicate <id>` | Copy the entry as "‹description› (copy)" (then " (copy 2)", …). |
| `material.delete <id>` | Remove the entry; followers stop following (undoable for the project). |
| `material.list` | The active library as an indented tree with ids. |

## Materials panel (app)

As `docs/spec/style/later-windows.md`, "Materials panel", with these
choices:

- The panel is the second tab of the dock's lower group, after Machine.
- Library row: "Library" dropdown, "Rename library…", "Close library".
- Tree (material, thickness or group, entries with a line, fill or image
  mode mark, and a chain mark on entries a layer follows) and the button
  column: Apply to layer, Follow this entry; Save layer as material…,
  Edit settings…, Update from layer, Edit description…; Duplicate,
  Delete. Under them the target line: "Target layer" and the edited
  layer's chip and name (the Layers panel's edited layer).
- Edit settings… opens the entry's settings in a window with the layer
  editor's fields; OK runs one `material.set` per changed key.
- "Library ▸" opens Open library…, Save library, Save library as…, New
  library, Merge library… (file paths asked in the usual path window).
- Confirmations as the style guide: "Replace material settings?" before
  Update from layer, "Delete material entry?" before Delete, "Close
  library?" when closing with unsaved changes.
- Double-click on an entry applies it to the target layer.

## Sources

- Material library panel, entries by material and thickness or a title
  without thickness, applying versus linking a layer, creating an entry
  from a layer, device-linked libraries, library file actions:
  https://docs.lightburnsoftware.com/2.1/Reference/MaterialLibrary/ (read
  2026-10-22).
- Own design: file format, ids, follow storage and refusals, the starter
  library and its data, command set, undo split.
