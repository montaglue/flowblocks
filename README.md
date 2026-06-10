# flowblocks

`flowblocks` is a Rust library for building control-flow graphs, laying them out
with the [Triskel](https://crates.io/crates/triskel) CFG layout engine, and
computing VEIL-style layout metrics.

The crate owns a small CFG model and uses:

- [`triskel`](https://crates.io/crates/triskel) for C++-backed CFG layout.
- [`petgraph`](https://crates.io/crates/petgraph) for internal graph storage.

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

    let entry = cfg.add_block("entry");
    let branch = cfg.add_block("branch");
    let true_exit = cfg.add_block("true_exit");
    let false_exit = cfg.add_block_with_size("false_exit", 120.0, 48.0)?;

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

- `add_block(label)` adds a labeled basic block.
- `add_block_with_size(label, width, height)` adds a block with explicit layout
  dimensions.
- `add_edge(from, to, kind)` adds a directed control-flow edge.
- `validate()` requires exactly one entry block and at least one exit block.
- `layout()` validates the CFG, runs Triskel, and returns `CfgLayout`.

`CfgLayout` stores graph dimensions, block positions, edge polylines, inferred
ranks/columns, entry, and exits. Ranks and columns are inferred from Triskel
coordinates because Triskel exposes coordinates and waypoints, not explicit rank
metadata.

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

## Development

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```
