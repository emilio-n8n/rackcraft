# Third-party notices

RackCraft itself is licensed under **MIT OR Apache-2.0** (see `LICENSE-MIT` and
`LICENSE-APACHE`).

The Linux AppImage bundles the following tools/libraries so that legacy `.vss`
stencils can be converted without installing LibreOffice:

| Component | Purpose | License |
| --- | --- | --- |
| `vss2xhtml` / libvisio | legacy Visio `.vss` to SVG conversion | MPL-2.0 |
| librevenge, librevenge-stream | libvisio support library | MPL-2.0 |
| libxml2 | XML parsing used by libvisio | MIT |
| ICU (`libicuuc`, `libicudata`) | Unicode support used by libxml2/libvisio | Unicode License / MIT-style |
| zlib, liblzma | compression support used by libxml2 | zlib / public domain |
| GNU libstdc++ / libgcc | C++ runtime for the bundled converter | GPL-3.0 with GCC Runtime Library Exception |
| `metafile` (Rust) | EMF/WMF vector rendering for vendor stencils | Apache-2.0 |

The bundled libraries are used only by the isolated `vss2xhtml` subprocess. The
AppImage remains extractable, so users can replace those libraries if desired.

The application also uses the following Rust crates under their respective
licenses (all MIT/Apache-2.0-compatible): `eframe`, `egui`, `kurbo`, `zip`,
`roxmltree`, `pdf-writer`, `rust_xlsxwriter`, `cfb`, `svgtypes`, `png`, `rfd`,
`serde`, and their transitive dependencies. Full license texts are available in
the source repository and from each crate's upstream project.

Upstream project pages:

- libvisio / librevenge: <https://wiki.documentfoundation.org/Development/Libvisio>
- libxml2: <https://gitlab.gnome.org/GNOME/libxml2>
- ICU: <https://icu.unicode.org/>
- GCC runtime: <https://gcc.gnu.org/onlinedocs/libstdc++/manual/license.html>
