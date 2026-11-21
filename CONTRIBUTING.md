# Contributing to RackCraft

Thanks for helping. RackCraft is an independent community project; by taking
part you agree that your contributions are licensed under `MIT OR Apache-2.0`.

## Set up

Rust 1.95 or newer is required. On Linux you also need the usual egui/winit
development packages (`libxkbcommon-dev`, `libwayland-dev`, `libx11-dev`,
`libxcursor-dev`, `libxrandr-dev`, `libxi-dev`, `libgl1-mesa-dev`,
`libfontconfig1-dev`).

```bash
git clone https://github.com/emilio-n8n/rackcraft
cd rackcraft
cargo run -p rackcraft -- --sample
```

### Constrained / Cloud Shell environments

Never build into `$HOME` when disk is tight. Use:

```bash
export CARGO_TARGET_DIR=/tmp/cargo-target
cargo test -p rackcraft-engine -p rackcraft-io
```

Release builds belong to CI. See [`AGENTS.md`](AGENTS.md) for the full rules.

## Quality gates

Every pull request must pass:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Add tests with each behaviour change:

- `engine`: document model, BOM grouping, snapping, undo depth, JSON schema.
- `io`: stencil parsing, shape data, connection points, CSV/XLSX output.
- `ui-egui`: pure helpers such as view transforms and field mapping; UI tests
  must not open a window.

## Architecture rules

- Keep dependency direction `engine <- io <- ui-egui <- app`.
- `engine` has no UI, ZIP, XML or filesystem dependency.
- All document geometry is millimetres, origin top-left, `1 U = 44.45 mm`.
- Add public documentation comments. Do not `unwrap()` external input.
- New stencils or fixtures need a deterministic test and a short provenance note.

## Visio stencils

Only commit stencils you have the right to redistribute. Vendor `.vssx` files
found online are reference material, not project assets. The files in
`samples/` are hand-authored fixtures; regenerate with:

```bash
python3 tools/generate_samples.py
```

## Reporting issues

Include the RackCraft version, OS, what you did, what happened, and (for parser
issues) a minimal stencil or the master XML structure that fails. Do not attach
proprietary stencil libraries unless their licence allows it.

RackCraft is not an official Storytold / ArtCraft product. Please report issues
to this repository, not to the ArtCraft team.
