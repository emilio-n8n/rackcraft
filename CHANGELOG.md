# Changelog

## 0.2.1 — 2026-11-22

- **Cisco/vendor EMF artwork:** Visio `.vssx` masters whose visuals live in
  `ForeignData` EMF/WMF files are now converted to normalized vector paths the
  first time they are placed. Cisco Catalyst 9000 switch faces render instead of
  plain 1U rectangles; conversion is lazy and cached per master.
- **Vendor ZIP import:** `.zip` downloads are opened directly; the contained
  `.vssx` is preferred over `.vss`, with a size-safe fallback chain.
- **Companion `.vssx`:** importing a legacy `.vss` automatically uses a sibling
  `.vssx` with the same stem when present, avoiding huge lossy conversion of
  vendor packages that ship both formats.
- **Open project from CLI:** `rackcraft --open project.json` (or a positional
  `.json` path) starts directly in an existing project.
- Import detection now records ForeignData image entries without loading
  hundreds of megabytes at import time; vector rendering happens on placement.
- Third-party notices include the Apache-2.0 `metafile` renderer.

## 0.2.0 — 2026-11-21

- **Connectors:** new document model (`Connector`, glued/free endpoints, labels,
  colours, straight/orthogonal routing, dashed lines, arrowheads), full undo/redo
  and JSON persistence. Attached endpoints follow moved equipment; deleting
  equipment detaches them at their last position.
- **Connector tool:** `V` / `C` modes, green connection-point targeting,
  click-A/click-B creation, connector inspector and deletion.
- **Vector PDF:** A4 landscape schematic with project cartouche and an inventory
  page (`Part Number | Description | Quantity`); toolbar button and `Ctrl+E`.
- **Legacy `.vss`:** automatic `vss2xhtml` and LibreOffice discovery, SVG/XHTML
  import, plus a native OLE/CFB fallback with streams, printable metadata and
  PNG/ICO previews. The Linux AppImage now bundles `vss2xhtml` and its libraries.
- **Samples:** new `network-topology.vssx` library (Cloud/FAI, Firewall, Router,
  Core Switch, Server, Workstation), four connection points per symbol.
- **Tests:** 75 tests covering connectors, PDF generation, SVG conversion,
  native CFB fallback and existing engine/io behaviour.

## 0.1.1 — 2026-11-21

- Add a self-contained x86_64 Linux **AppImage** to releases, with desktop
  metadata and a 256 px icon.
- Ship release documentation and docs images in the downloadable archives.
- Build Linux releases on Ubuntu 22.04 so the binary requires only glibc 2.35.
- Fix GitHub Release publishing permissions in the release workflow.

## 0.1.0 — 2026-11-21

Initial public MVP.

- Native Rust workspace: `engine`, `io`, `ui-egui`, `rackcraft` app.
- Visio `.vssx` parser: masters, nested geometry, Bézier/arc/NURBS endpoints,
  connection points, shape data, embedded ICO previews, `.vss` detection and
  optional LibreOffice conversion.
- Document model in millimetres with 25U/19-inch conventions, bounded undo/redo,
  self-contained JSON save/load.
- Magnetic rack-U and connection-point snapping, including modular panel
  apertures.
- Dark native egui UI: searchable stencil browser, vector canvas with zoom/pan,
  drag-and-drop placement, shape-data inspector and live BOM.
- Exports: formatted `.xlsx` and UTF-8 `.csv`.
- Offline sample rack (`--sample`) and two hand-authored `.vssx` fixture
  libraries.
- CI on every push/PR; release workflow for Linux x86_64, Windows x86_64 and
  macOS arm64/Intel with `SHA256SUMS.txt`.
