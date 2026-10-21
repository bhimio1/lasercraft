# Clean-room policy

LaserCraft is an independent implementation of a laser design-and-control
workflow. It is not affiliated with, endorsed by, or derived from LightBurn
Software or any other vendor. "LightBurn" is named in our docs only to say where
public information came from and which workflow we aim to make familiar; it
never appears in the app's name, branding or UI.

## Why this document exists

Claiming "clean room" only means something if the rules are written down and
followed. A classic clean room uses two separate teams: one studies the original
and writes a specification, the other implements from that specification
without ever seeing the original. LaserCraft is built by one developer working
with an AI coding agent, so we cannot claim that separation. Instead we commit
to the rules below and record provenance for every behavior we implement. Nobody
has independently verified this process.

## Allowed sources

1. LightBurn's publicly published documentation at
   `https://docs.lightburnsoftware.com/`, cited by page URL.
2. Behavior the project owner describes from using software they are licensed
   for (what happens when you press a key, what a dialog offers), recorded in
   the spec as "owner's description" with a date.
3. Open standards and public specifications:
   - GRBL's published documentation (the `gnea/grbl` wiki: commands, interface,
     settings, alarm and error codes, laser mode).
   - SVG (W3C), DXF (Autodesk's published DXF reference), G-code (RS-274 and
     the GRBL dialect).
   - PNG and JPEG specifications.
   - USB CDC-ACM and serial port documentation.
4. Our own design decisions, marked "own design".
5. Open-source code under a license compatible with LaserCraft's
   (GPL-3.0-or-later), copied or ported with its copyright notice and license
   kept, and named in the spec's Sources. Third-party crates used as ordinary
   dependencies are listed in `ARCHITECTURE.md`.

## Forbidden

- Decompiling, disassembling, debugging, tracing or otherwise inspecting
  LightBurn binaries, installers, bundled resources or network traffic.
- Any leaked, stolen or otherwise unauthorized source code, documentation or assets.
- Copying LightBurn's icons, artwork, fonts, screenshots, names of its own
  features or products, material library data, or UI text. Paraphrase behavior
  in our own words; never paste sentences from the docs into the UI.
- Reading, parsing or writing LightBurn's `.lbrn` or `.lbrn2` project formats,
  or its material library and preference files, unless the owner explicitly
  approves a specific source and approach later. That decision is recorded here
  with its date.
- Proprietary controller protocols (Ruida, Trocen, TopWisdom and similar). They
  stay out of scope until the owner decides which public sources, if any, are
  acceptable. GRBL (open source, publicly documented) is in scope.
- Using another product's trademarks in LaserCraft's name, branding or marketing.

## Look and feel

Familiar layout and default keyboard shortcuts help people switch. Shortcut
assignments and general workflow are taken from the public docs, never from the
program itself. Even so:

- Draw every icon and widget ourselves.
- Keep shortcuts in a keymap data file (`crates/lasercraft-ui/keymap/`), citing
  the doc page in `docs/spec/keymap.md`.
- Avoid pixel-for-pixel copies of another product's screens. Close imitation of a
  distinctive interface can raise trade dress concerns; when in doubt, differentiate.
- Our own terms come first. Where a familiar term helps ("cut layer"), the spec
  says so and uses it as a plain description, not as a copied label.

## Process

1. Before implementing a behavior, write or update a spec in `docs/spec/`.
2. Each spec ends with a **Sources** section naming a public LightBurn doc page by
   URL, an open standard, the owner's description, or "own design".
3. Code follows the spec. If the code needs behavior the spec does not describe,
   update the spec first.
4. Reviewers (human or agent) reject work whose provenance is unclear.
5. When a doc page cannot be reached, the spec says so, and nothing is filled in
   from memory or guesswork.

## Decisions log

- 2026-10-21: project started. License GPL-3.0-or-later, matching the owner's
  MusicCraft, whose command registry and keymap design (same owner) are reused.

## When in doubt

Stop and ask. Dropping a feature costs less than contaminating the project.
