# Optimizations and measurements

Argui uses measurements and behavioral checks to decide which storage and
execution costs to remove. This document records the workload, comparison point
and limits of each result. Percentages here describe those workloads, not every
application built with Argui.

## Dense tree storage and selective SoA

Measured on 2026-09-12 against `fb88ea7`, after the shadcn catalogue additions.
Both versions use the same release profile and all feature flags, on Linux
x86-64, Intel Core Ultra 5 125H, Rust 1.98.0 (`88d9e12ae`, LLVM 22.1.8).

### Decision

A structure of arrays is useful in Argui's retained indices and layout engine.
The implementation separates geometry and measurement caches from node metadata,
and keeps large payloads out of unused hash-table buckets. It preserves the
public `Element` builders, shared immutable descriptions, stable node identities
and the renderer's `LayoutOutput` contract.

The previous layout tree stored `HashMap<NodeId, Node>`, where each hash bucket
contained styles, child lists, a measurement cache, two layouts, and space for
optional custom behavior. The current tree stores:

```text
positions: HashMap<NodeId, usize>    stable identity → dense position
ids:       Vec<NodeId>             reverse mapping for swap removal
nodes:     Vec<Node>               style, children, parent, text context
caches:    Vec<Cache>              Taffy measurement caches
unrounded: Vec<Layout>             logical geometry before rounding
layouts:   Vec<Layout>             final geometry
custom:    HashMap<NodeId, Custom>  only nodes with custom behavior
```

Removing a node swaps the last entry into its position in every column and
updates the reverse mapping. Within a retained layout tree, identity allocation
remains monotonic: a removed
identity cannot silently address the replacement. Storage follows the live
graph rather than the highest identity ever allocated. Initial allocation uses
the known node count; subsequent growth keeps up to 25% planned headroom.
Compaction after reconciliation releases excess capacity left by replacement or
large removals. Small maps keep some bucket slack, bounded separately.

The UI preorder index also owns the parent column. Parents use `u32` positions
with a sentinel, replacing the event registry's separate `Option<usize>` vector.
Selection policy and writing direction remain compact typed columns. Policy
queries read only their column, without fetching selection colors. A selection
style stores the position of its nearest explicit owner instead of copying two
RGBA colors into every descendant. The default style is computed once per
index. All indices are rebuilt for the current revision, including inherited
policy changes; the `u32` representation explicitly rejects trees with four
billion retained nodes instead of truncating positions.

Input, action, portal and parent queries use the existing ID lookup. The old
linear search was especially expensive for repeated queries near the end of a
large tree. The CPU gains of this combined change must therefore not all be
attributed to cache locality or to SoA alone.

### Why not split every coordinate into a separate vector?

Four `f32` coordinates occupy 16 bytes whether stored together or in four
columns. The relevant Argui passes consume whole rectangles, clips, sizes and
other layout data; there is no measured scalar-only coordinate pass that
justifies splitting these further. A bare `parent/first_child/next_sibling/flags`
tree would also omit styles, text, accessibility, retained caches and custom
layout state that real widgets need.

On this build, an `ElementNode` description itself occupies 2,872 bytes, and its
`ElementKind` enum occupies 360 bytes even for a container. Those descriptions,
text data and painting output remain substantial memory costs. This change is
an internal storage optimization, not a claim that total application RAM is now
minimal. A further separation of rarely used description properties would need
its own measurements of allocation, cloning and builder costs.

### Memory results

DHAT bytes, with 10,101 retained nodes in the initial large graph:

| Workload / counter | Before | After | Change |
| --- | ---: | ---: | ---: |
| UI tree, live heap | 30,538,725 | 30,003,854 | −1.8% |
| UI + layout + paint output, live heap | 70,495,834 | 62,390,819 | −11.5% |
| Initial layout, cumulative allocated bytes | 143,118,269 | 114,947,559 | −19.7% |
| Five grow/shrink cycles, peak live heap | 71,216,862 | 63,318,692 | −11.1% |
| After final shrink to 102 nodes, live heap | 20,816,230 | 866,020 | −95.8% |
| Five grow/shrink cycles, cumulative allocated bytes | 587,553,977 | 634,879,508 | **+8.1%** |

Compaction trades buffer reuse for releasing memory: repeated large growth
allocates the buffers again. This explains the increased allocation volume in
the last row despite the much smaller retained heap. Initial construction also
retains six more allocation blocks, since the dense columns are separate
allocations; their total size is smaller. The UI-only saving is modest because
large element descriptions dominate that workload.

### CPU results

Seven paired release runs, medians. The following two tables use 10,101 nodes.

| Operation | Before | After | Duration change |
| --- | ---: | ---: | ---: |
| First layout | 171.12 ms | 127.52 ms | -25.5% |
| Cached computation | 18.96 ms | 17.01 ms | -10.3% |
| Viewport resize | 38.22 ms | 37.79 ms | -1.1% |
| Keyed row reorder | 73.93 ms | 72.48 ms | -2.0% |

| Operation | Before | After | Duration change |
| --- | ---: | ---: | ---: |
| UI index construction | 3.99 ms | 4.84 ms | +21.5% |
| Parent lookup, per node | 6,403 ns | 45 ns | -99.3% |
| Selection policy + style read, per node | 80 ns | 122 ns | +52.5% |
| Event-path preparation, per event | 4,834 ns | 570 ns | -88.2% |
| UI paint update | 3.23 ms | 3.11 ms | -3.8% |

The combined selection read is slower in this run: resolving a style now reads
an owner column instead of a copied color pair. The policy-only API avoids that
work, but the benchmark intentionally requests both. For the smaller 1,011-node
layout, resize and reorder medians increase by 8.3% and 2.9%; first layout and
cached computation improve. Small timing differences should be read alongside
the raw sample spread and uncontrolled CPU frequency. The memory reduction does
not imply that every individual operation becomes faster.

### Gallery workload

Three paired runs, 6,000 scroll steps per page:

| Measurement | Before | After | Change |
| --- | ---: | ---: | ---: |
| VList scroll | 12.35 s | 10.72 s | -13.2% |
| Table scroll | 3.82 s | 1.15 s | -70.0% |
| DataTable scroll | 16.59 s | 5.17 s | -68.9% |
| Process user CPU, all three pages | 30.15 s | 23.84 s | -20.9% |
| Process maximum RSS | 24.21 MiB | 22.96 MiB | -5.2% |

Retained node counts are identical: VList 214, Table 213, DataTable 345. The
numbers of full layout computations also match: 1,051, zero and 461 respectively.
The million-row/large-model capability therefore does not imply a million-node
retained graph, and the large synthetic tree's memory percentage should not be
applied to these pages.

Timing variability is substantial. Total process user CPU ranges from
12.76–45.81 seconds before and 10.32–28.82 seconds after. It decreases in all
three pairs (19–37%), but the second VList pair is slower after the change
(12.97 → 25.15 seconds). The per-page medians above are not a stable frame-rate
prediction, and summing separate medians does not produce the median total.
RSS varies much less: 24,752–24,804 KiB before and 23,344–23,912 KiB after.
A dedicated machine with fixed frequency would be needed for tighter CPU claims.

### Measurement protocol

The committed [raw measurements](optimizations-data.json) retain individual
samples and executable hashes. [`profile-trees.py`](../scripts/profile-trees.py)
runs one warm-up pair followed by seven pairs for the synthetic trees and three
pairs for the gallery, alternating execution order and
pinning each process to logical CPU 0. Timings are medians from normal release
executables, with no compilation, coverage, browser automation or Valgrind run
in parallel. CPU frequency and other desktop activity are not controlled.

[`tree_profile`](../crates/argui-ui/examples/tree_profile.rs) creates text leaves
with branching factors 100 and 10. It measures index construction, parent
queries, inherited selection reads, event-path preparation and paint updates.
These event measurements have no user listeners attached.
[`layout_profile`](../crates/argui-layout/examples/layout_profile.rs) lays out
fixed-size colored containers, measures the first computation, repeated cached
computations, viewport resizing and keyed row reordering. It does not load fonts
or initialize a GPU. A cached computation still assembles layout and paint output;
it is not the cost of Argui's idle loop.

Separate Valgrind DHAT runs measure cumulative allocations, live heap at the
global peak, and live heap at process exit. The `heap` mode deliberately retains
the graph until immediate process exit so the latter counter represents retained
objects. The `churn` mode alternates 10,000 and 100 leaves five times, then retains
the small graph. These counters exclude allocator metadata, unused allocator
pages, stacks and GPU memory. They are not RSS. Instrumented elapsed times are
not used as CPU performance measurements.

The gallery comparison uses the existing `data::profile_virtual_scrolling` test:
6,000 four-pixel scroll steps per page at 1280 × 900, including models, tree
updates, layout, paint data, events and hit testing. `/usr/bin/time` measures
process user CPU and maximum RSS separately from each page's internal timer.
This workload includes embedded font shaping, but excludes glyph preparation,
native window services and GPU submission.

### Reproduce

Keep reference and candidate build directories separate: sharing Cargo output
between checkouts can reuse artifacts from the other checkout. Copy the two
benchmark examples from this revision into a checkout of `fb88ea7`; also give
its `argui-layout` crate the benchmark's `web-time` dev dependency. Build each
checkout with its own `CARGO_TARGET_DIR`:

```sh
cargo build -p argui-ui -p argui-layout --examples --release --all-features
cargo nextest list -p argui-widget-gallery --all-features \
  --cargo-profile release --test pages --message-format json
```

Save the two example executables as `tree_profile` and `layout_profile` in each
measurement directory. Save the `pages` executable path reported by Nextest as
`gallery-pages`. Record heap profiles separately for each directory:

```sh
valgrind --tool=dhat --dhat-out-file=tree-heap.json ./tree_profile 10000 100 heap
valgrind --tool=dhat --dhat-out-file=layout-heap.json ./layout_profile 10000 heap
valgrind --tool=dhat --dhat-out-file=churn-heap.json ./layout_profile 10000 churn
```

After all compilation and profiling processes have finished:

```sh
python3 scripts/profile-trees.py target/reference target/candidate \
  --gallery --runs 7 --gallery-runs 3 --cpu 0 --output target/tree-comparison.json
```

Correctness checks compare retained geometry against a fresh engine after
repeated growth, compaction, deletion and keyed reordering. UI tests exercise
parent lookup, stale identities and inherited selection overrides. Existing
custom layout, focus, event, text, clipping and portal tests remain part of the
affected crate checks. The repository's final quality gate requires at least
85% on each coverage metric, globally and in every crate; thresholds and
exclusions are described in [code quality](code_quality.md).

## Earlier work

The [gallery footprint study](widget-gallery-performance.md) records the
2026-09-10 paint-cache cleanup, reduced copying and lazy data-model allocation,
including native RSS and the limits of those native CPU observations. Those
numbers use a different baseline and must not be added to this study's percentages.
The [roadmap](roadmap.md) tracks retained subtree reuse, invalidation and
virtualization; benchmark size should always distinguish total model rows from
the nodes actually present in the UI tree.
