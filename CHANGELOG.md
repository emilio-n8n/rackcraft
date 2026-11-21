# Changelog

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
