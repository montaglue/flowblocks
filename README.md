# flowblocks

`flowblocks` is a Rust library for building control-flow graphs, laying them out
with the [Triskel](https://crates.io/crates/triskel) CFG layout engine, and
computing VEIL-style layout metrics.

The crate owns a small CFG model and uses:

- [`triskel`](https://crates.io/crates/triskel) for C++-backed CFG layout.
- [`petgraph`](https://crates.io/crates/petgraph) for internal graph storage.
- [`egui`](https://crates.io/crates/egui) for feature-gated immediate-mode CFG
  display widgets.

## Requirements

`flowblocks` depends on `triskel`, which builds vendored C++ sources by default.
You need:

- A C++23 compiler.
- `fmt` available through `pkg-config` or installed under `/opt/homebrew`.

## Example

```rust
use flowblocks::{Cfg, EdgeKind, Result};

fn main() -> Result<()> {
    let mut cfg = Cfg::new();

    let entry = cfg.add_node(96.0, 44.0)?;
    let branch = cfg.add_node(112.0, 48.0)?;
    let true_exit = cfg.add_node(96.0, 44.0)?;
    let false_exit = cfg.add_node(120.0, 48.0)?;

    cfg.add_edge(entry, branch, EdgeKind::Default)?;
    cfg.add_edge(branch, true_exit, EdgeKind::True)?;
    cfg.add_edge(branch, false_exit, EdgeKind::False)?;

    let layout = cfg.layout()?;
    let metrics = layout.metrics();

    println!("blocks: {}", layout.blocks.len());
    println!("edges: {}", layout.edges.len());
    println!("graph area: {}", metrics.graph_area);
    println!("consistent flow: {}", metrics.consistent_flow);

    Ok(())
}
```

## CFG API

The main entry point is `Cfg`:

- `add_node(width, height)` adds a basic block with explicit layout dimensions.
- `add_edge(from, to, kind)` adds a directed control-flow edge.
- `validate()` requires exactly one entry block and at least one exit block.
- `layout()` validates the CFG, runs Triskel, and returns `CfgLayout`.

`CfgLayout` stores graph dimensions, block sizes and positions, edge polylines,
inferred ranks/columns, entry, and exits. Ranks and columns are inferred from
Triskel coordinates because Triskel exposes coordinates and waypoints, not
explicit rank metadata.

## VEIL Metrics

`VeilMetrics::compute(&layout)` and `layout.metrics()` compute:

- C1 node orthogonality.
- C2 edge orthogonality.
- C3 edge crossings.
- C4 edge bends.
- C5 edge uniformity as median absolute deviation of log edge lengths.
- C6 short-edge summary: total, max, and median routed length.
- C7 graph area.
- C8 symmetry tension.
- C9 consistent flow.
- C10 happens-before exit placement.
- C11 edge-direction grouping.

Some VEIL definitions leave implementation details open. This crate documents
those choices in code by using inferred layout ranks, routed edge polylines, and
median-based summaries for robust first-pass metrics.

## egui Display

With the `ui` feature enabled, `flowblocks::ui` exposes a reusable egui widget
for displaying any `CfgLayout`:

```rust
# use flowblocks::{Result, cfg_viewer, examples};
#
# fn render(ui: &mut egui::Ui) -> Result<()> {
let cfg = examples::branch_with_join()?;
let layout = cfg.layout()?;

cfg_viewer(&layout).show(ui);
# Ok(())
# }
```

The viewer draws routed edges, arrowheads, blocks, and entry/exit emphasis. Use
`CfgViewer::with_options` to customize colors, strokes, and padding.

The crate also includes a native viewer binary:

```sh
cargo run --features viewer --bin flowblocks-viewer
```

The binary lets you switch between the built-in examples and inspect their CFG
shape, rendered layout, and VEIL metrics.

## Built-in Examples

`flowblocks::examples` contains reusable CFG builders:

- `branch_with_join()`
- `counted_loop()`
- `nested_conditionals()`
- `switch_dispatch()`
- `retry_with_cleanup()`
- `single_block_loop()`

Use `examples::all()` to iterate over all named examples.

## Development

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```
