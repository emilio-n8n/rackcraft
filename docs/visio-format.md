# Visio stencil format notes

This document records what RackCraft's parser actually consumes. It was written
after inspecting real vendor `.vssx` packages (Allied Telesis, StorSimple, QNAP
and small public stencils) and the ArtCraft reference repositories
`storytold/vectorcraft`, `storytold/craft-libs` and `UCS-Care-Group/mailcraft`.
No vendor stencil is redistributed in this repository.

## Container

A `.vssx` is an OPC package (ZIP); vendor `.zip` downloads containing `.vssx`
or `.vss` are opened directly and `.vssx` is preferred:

```
[Content_Types].xml
_rels/.rels
docProps/app.xml
visio/document.xml
visio/masters/masters.xml
visio/masters/_rels/masters.xml.rels
visio/masters/master1.xml
...
```

`masters.xml` lists `<Master ID Name NameU ...>` entries with a `<PageSheet>`
holding `PageWidth` and `PageHeight` in **inches**, an optional base64 `<Icon>`
(ICO), and a `<Rel r:id="...">` that resolves through the masters relationship
part to the master contents XML.

RackCraft is namespace-agnostic: it compares local tag names, so it accepts
`Visio 2013+` packages produced by Visio, LibreOffice, draw.io and vendor
exporters.

## Units and coordinates

- Visio cell values are inches for lengths and **radians** for angles.
- Visio page space has Y growing upwards; RackCraft converts to millimetres with
  Y growing downwards at the final step:
  `x_mm = x_in * 25.4`, `y_mm = (page_height_in - y_in) * 25.4`.
- The parser flattens the shape tree. Each shape applies
  `Translate(Pin) · Rotate(Angle) · Scale(FlipX, FlipY) · Translate(-LocPin)`
  in its parent's coordinate space; child shapes inherit the parent matrix.
- Normalized artwork uses the master page as its box (`0..=1`), so scaling an
  item stays crisp at any zoom.

## Geometry rows

Supported rows: `MoveTo`, `LineTo`, `RelMoveTo`, `RelLineTo`, `CubBezTo`,
`RelCubBezTo`, `QuadBezTo`, `RelQuadBezTo`, `Ellipse`, `EllipticalArcTo`,
`RelEllipticalArcTo`, `NURBSTo` (endpoint approximation), plus `SplineStart`/
`SplineKnot` as straight segments. `Rel*` coordinates are fractions of the
shape's `Width`/`Height`; absolute rows are in inches.

Geometry cells:

- `NoFill`, `NoLine`, `NoShow` control rendering.
- `NoSnap` is currently informational; RackCraft uses explicit connection
  points for glue.
- Ellipses are approximated by four cubic Béziers; other unsupported rows are
  reduced to their endpoint so a shape never disappears completely.

Real vendor stencils often contain a `ForeignData` bitmap or metafile with a
simple geometry rectangle. For `.vssx`, RackCraft records EMF/WMF entries and
converts them to normalized vector paths lazily when the master is placed (Cisco
Catalyst 9000 is the reference case). When no geometry exists, RackCraft draws a
visible fallback box with the master's fill/line colours and keeps the
connection points and shape data.

## Shape data

Visio 2013+ stores shape data in `Section N="Property"` (older files may use
`ShapeData`). Rows carry:

- `Label` — user-facing field name;
- `Value` — cached string/number;
- `Prompt` — help text.

RackCraft merges rows from nested shapes by label (first non-empty value wins),
then maps `Installed Date`, `Warranty Exp`, `Location`, `Building` and
`Notes`/`Comments` into project metadata. Manufacturer, Part Number and Product
Description stay in the BOM identity.

Rack metadata (`Rack Units`, plus optional `Mount Left/Right/Top/Bottom` in
millimetres) is read from shape data, falling back to a number followed by `U`
in the master name (e.g. `25U Rack`).

## Connection points

`Section N="Connection"` rows have `X`/`Y` in shape-local inches. They are
transformed through the shape matrix, normalized to the page and clamped to the
item box. The optional `Prompt` cell becomes the tooltip name. This is what
makes keystones glue to modular panel apertures and equipment align to rack U
slots.

## Icons

`<Icon>` is base64 ICO. RackCraft decodes uncompressed 1-, 4-, 8-, 24- and
32-bpp DIB icons (including the paletted icons common in vendor stencils);
PNG-compressed ICO entries are skipped and the UI falls back to a vector
thumbnail generated from the geometry.

## Legacy `.vss`

Binary `.vss` files are OLE Compound Files, detected by their magic
`D0 CF 11 E0 A1 B1 1A E1`. Import tries, in order:

1. LibreOffice `soffice --headless --convert-to vssx` (full metadata).
2. `vss2xhtml` from libvisio-tools, whose SVG/XHTML output is parsed into
   vector masters with four edge connection points and a clear metadata warning.
   The AppImage bundles `vss2xhtml` plus its private library directory.
3. A native `cfb` reader that lists OLE streams, extracts printable UTF-16/ASCII
   metadata and decodes PNG/ICO previews, then creates one clearly labelled
   placeholder master. It never claims to have recovered Visio geometry.

Package hints shown to users: `pacman -S libvisio` (Arch/CachyOS),
`dnf install libvisio-tools` (Fedora), `apt install libvisio-tools`
(Debian/Ubuntu).

## Out of scope (for 0.1.x)

- Master inheritance / `BaseID` chains and style-sheet inheritance beyond
  explicit shape cells.
- Text layout and rich character runs (item names are drawn by the UI).
- Embedded bitmap rendering beyond the ICO preview and fallback box.
- Theme colours (`THEMEVAL`) and indexed palette colours; defaults are used.
- Visio `.vsdx` documents as full drawings (stencils only).
