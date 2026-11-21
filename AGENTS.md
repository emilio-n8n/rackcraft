# RackCraft — agent & contributor guide

RackCraft is a native Rust desktop application for designing technical racks and
structured-cabling layouts from Visio stencils (`.vssx`), with magnetic snapping,
shape data and Excel/CSV bill-of-materials export.

RackCraft is an **independent, community open-source project**. It is compatible
with the conventions of the ArtCraft ecosystem, but it is **not** an official
Storytold / ArtCraft product and must never be presented as one.

## ⚠ Critical environment constraint (Google Cloud Shell)

This checkout runs in a small Cloud Shell workspace. **Never build inside
`$HOME`.**

- Always export `CARGO_TARGET_DIR=/tmp/cargo-target`.
- The Rust toolchain lives outside the home directory in this environment:
  ```sh
  export CARGO_HOME=/tmp/cargo-home
  export RUSTUP_HOME=/tmp/rustup-home
  export PATH=/tmp/cargo-home/bin:$PATH
  export CARGO_TARGET_DIR=/tmp/cargo-target
  ```
- Only run **light** checks locally, on the pure-logic crates:
  ```sh
  CARGO_TARGET_DIR=/tmp/cargo-target cargo check -p rackcraft-engine -p rackcraft-io
  CARGO_TARGET_DIR=/tmp/cargo-target cargo test  -p rackcraft-engine -p rackcraft-io
  ```
- A full `cargo build --release` is **forbidden locally** (disk/RAM). Full
  Linux/Windows/macOS release builds are delegated to GitHub Actions.
- Reference repositories are shallow-cloned only under `/tmp/refs/`; they are
  never copied or committed into this repository.
- Keep the repository free of build outputs. `/tmp/cargo-target` is the only
  target directory used here.

## Workspace layout

```
apps/rackcraft        Native binary (eframe/egui), supports --sample
crates/engine         UI-free document model, units, snapping, undo, BOM, JSON
crates/io             .vssx parser, .vss detection, BOM .xlsx/.csv export
crates/ui-egui        Dark native UI: stencil browser, canvas, shape data
samples/              Offline .vssx fixtures and documented sample project
.github/workflows/    CI and cross-platform release automation
```

Dependency direction is strictly one-way:

```
engine  <-  io  <-  ui-egui  <-  apps/rackcraft
```

`engine` must not depend on UI, zip, XML, egui or the filesystem. `io` may use
`engine`, `zip`, `roxmltree`, `kurbo`, `cfb`, `rust_xlsxwriter`, `csv`, but no
UI crate. The UI never parses `.vssx` itself.

## Core model

- All document geometry is in **millimetres** (f64), origin at the canvas
  top-left, Y grows downwards.
- One rack unit `U` = **44.45 mm**. Racks are 19-inch (482.6 mm) wide by
  convention; sample documents use a 600 mm cabinet.
- An `Item` is a placed stencil instance. It owns:
  - position and size in mm;
  - normalized vector graphics (0..1 bounding box) so it scales crisply at any
    zoom without depending on the master file at render time;
  - normalized connection points;
  - stencil shape data + editable project fields.
- Documents embed everything needed to render and compute a BOM, so a saved
  `.json` opens without the original stencil library.
- A `Connector` glues two `ConnectorEnd`s (item connection point or free mm
  point) and stores label, colour, straight/orthogonal routing, line style and
  arrows. Endpoints resolve dynamically, so moved equipment updates cables.
  Deleting equipment converts its attached ends to free anchors. Connectors use
  their own id sequence and are schema-versioned with the document.
- Shape data fields are case-insensitive on import. Manufacturer, Part Number
  and Product Description are the BOM identity; project fields are Installed
  Date, Warranty Exp, Location, Building and Notes.
- Undo/redo uses bounded document snapshots (`History`), not fragile inverse
  commands. Every mutating user action records one snapshot.
- BOM grouping key: `(manufacturer, part_number, product_description)`.
  Quantities count placed items only.

## VSSX parsing rules

- `.vssx` is an OPC ZIP package; use `zip` + `roxmltree` (namespace-agnostic
  local names).
- Read `visio/masters/masters.xml`, then resolve each master's `<Rel r:id>` via
  `visio/masters/_rels/masters.xml.rels`; fall back to `masterN.xml`.
- Parse `PageSheet/PageWidth` and `PageHeight` (inches), all `Cell` values and
  `Section N='Connection'` / `Section N='Property'` (Visio 2013+ name; some
  producers use `ShapeData`).
- Geometry: `MoveTo`, `LineTo`, `RelMoveTo`, `RelLineTo`, `RelCubBezTo`,
  `RelQuadBezTo`, ellipse/arc approximations and NURBS endpoints. Values in
  cells are inches; apply the shape transform (Pin, LocPin, Angle, Flip) through
  nested groups, then normalize to the master page.
- Legacy binary `.vss` is detected by the CFB magic, never parsed as ZIP.
  Conversion order: LibreOffice (`soffice`) → `.vssx`; `vss2xhtml`
  (libvisio-tools) → SVG/XHTML; native CFB inspection fallback for streams,
  PNG/ICO previews, printable metadata and a clearly labelled placeholder.
  The Linux AppImage bundles `vss2xhtml` and its private libraries. Never present
  a placeholder as a fully converted master.
- Keep parsing defensive: malformed masters are skipped with diagnostics rather
  than panicking. Library must not `unwrap()` on external input.

## Rust conventions

- Edition 2024, MSRV 1.95 (imposed by eframe 0.37 in the ArtCraft ecosystem).
- `cargo fmt --all` before committing.
- `cargo clippy --workspace --all-targets -- -D warnings` must pass. Prefer
  small explicit types, `thiserror` for library errors, `#[must_use]` where it
  prevents mistakes, and `f64` for millimetres.
- No `unsafe`. No `unwrap`/`expect`/`panic!` on external data paths. Tests may
  use `unwrap`.
- Public API gets doc comments. Keep functions focused and testable.
- Serialize with `serde`; schema changes need a migration note and a version
  bump in `Document::schema_version`.
- New features require tests: parser fixtures, snap math, BOM grouping, undo
  bounds, CSV escaping, XLSX generation (open the generated ZIP and assert
  sheet contents), connector resolution and PDF validity/page content.
- UI wording is English (the ArtCraft ecosystem convention); user-facing docs
  may be French or English. Keep code comments in English.

## Commits & releases

- Conventional, imperative commit messages (`feat:`, `fix:`, `docs:`, `ci:`).
- CI: fmt, clippy, workspace tests on Ubuntu.
- Release: tags `v*` or manual dispatch builds Linux x86_64, Windows x86_64,
  macOS arm64 and macOS x86_64, packages archives, writes `SHA256SUMS.txt` and
  publishes the GitHub Release. Never create release binaries locally.
