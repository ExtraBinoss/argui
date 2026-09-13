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
index. At this stage, all indices were rebuilt for the current revision, including
inherited policy changes. The incremental work below later removes that rebuild
when topology is unchanged. The `u32` representation explicitly rejects trees with four
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

At this first stage, an `ElementNode` description occupies 2,872 bytes, and its
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

The committed [raw measurements](data/optimizations-data.json) retain individual
samples and executable hashes. [`profile-trees.py`](../../scripts/profile-trees.py)
runs one warm-up pair followed by seven pairs for the synthetic trees and three
pairs for the gallery, alternating execution order and
pinning each process to logical CPU 0. Timings are medians from normal release
executables, with no compilation, coverage, browser automation or Valgrind run
in parallel. CPU frequency and other desktop activity are not controlled.

[`tree_profile`](../../crates/argui-ui/examples/tree_profile.rs) creates text leaves
with branching factors 100 and 10. It measures index construction, parent
queries, inherited selection reads, event-path preparation and paint updates.
These event measurements have no user listeners attached.
[`layout_profile`](../../crates/argui-layout/examples/layout_profile.rs) lays out
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
exclusions are described in [code quality](../contributing/code-quality.md).

## Incremental updates, optional storage and layout boundaries

Measured on 2026-09-12 against `52c9f32`, which already contains the dense-tree
work above. These results are **additional comparisons against that baseline**;
percentages from the two studies must not be added together. The machine,
release profile and feature flags are unchanged.

### Decisions, in implementation order

1. **Move large optional descriptions out of every element and remove a duplicate
   kind.** `semantics`, `layer` and `scroll` now use `Option<Box<T>>`. An absent
   feature occupies one pointer, while a present feature pays for its own
   allocation. `ElementNode` shrinks from 2,872 to 1,904 bytes on this build
   (-33.7%). `NodeMap` no longer stores another 360-byte `ElementKind` beside the
   retained element that already owns it. Public builders keep their arguments;
   direct assignments to those three public fields now use boxed values.
2. **Stop dirty propagation at an already-invalid ancestor.** Clearing a Taffy
   cache returns whether it was already empty. `mark_dirty` stops at that point,
   rather than repeating the full ancestor walk for every modified sibling.
   Updating 200 leaves below 64 ancestors measures this case directly.
3. **Refresh existing indices and skip shared subtrees.** A preorder end column
   lets identity reconciliation copy a shared subtree's IDs without allocating
   a temporary mirror of the old tree. With unchanged identities/topology,
   index refresh preserves its storage and skips shared descendants unless an
   inherited selection policy, owner or direction changed. Animation bindings
   are rebuilt only when needed; trees without state transitions avoid the
   global transition-context pass. Structural changes still rebuild the index.
4. **Compact layout links without penalizing indexed child access.** Parent and
   measurement-context fields use `u32` positions with a sentinel. Children use
   `Box<[NodeId]>`, replacing a 24-byte vector header with a 16-byte slice header
   on this platform. Equal-length updates reuse the allocation. Reconciliation
   attaches the final child list before dropping removed subtrees, avoiding
   temporary allocation on each removal. Swap removal updates moved parents'
   child links and preserves stable identities.
5. **Add explicit layout boundaries and register exceptional roots.**
   `Element::layout_boundary(content)` keeps its child's intrinsic size and
   baseline out of the outer sizing calculation. A private layout root contains
   descendant invalidations. Portals and boundaries have a sparse preorder
   registry instead of being discovered by recursive full-tree scans. Each
   boundary checks Taffy's actual layout inputs and cache; an unchanged inner
   root is skipped, or only rounded again if its global position moved.
   `ScrollArea` uses this contract, including `MessageScroller` built on it.

### What was already present, and what was not added

| Proposal | Decision for Argui |
| --- | --- |
| Arena and compact IDs | Retained layout is already a dense arena. Compact **positions** now cover parent/context links; stable public `NodeId(u64)` and Taffy identities remain independent of relocation and do not alias deleted nodes. |
| `first_child` / `next_sibling` everywhere | Not adopted. Taffy's flex/grid/block adapters request both iteration and indexed children. Boxed contiguous child lists preserve both; linked siblings alone would make indexed access linear. The declarative shared `Element` tree still owns vectors of children. |
| Avoid every `Rc` / `Box` | Not adopted as a blanket rule. `Element` uses immutable sharing and copy-on-write, enabling unchanged subtree reuse. Custom state is sparse. Boxing large optional payloads actually lowers measured live heap despite a few extra allocations on feature-rich pages. |
| Bitflags | `VisualStates(u8)` already packs focused, focus-visible, hovered, pressed and disabled. A single tree-level `layout_dirty` boolean is not a per-node array of booleans; replacing it with a wider flags word saves nothing. |
| Dirty roots | The main tree already enters through one root and uses per-node Taffy caches. The new registry handles independent boundary/portal roots. It is a list of exceptional roots, **not** a general minimal dirty-subtree queue. |
| Versions / epochs | Tree revisions and custom layout/paint revisions already exist. Taffy cache invalidation and shared descriptions supply the relevant validity information here. No duplicate style/children/paint epoch columns or recursive frame reset were introduced. |
| Cache by real layout inputs | Already provided by Taffy's `compute_cached_layout`; boundaries also query with the actual `LayoutInput`. A second Argui layout cache would duplicate retained state. |
| Automatic fixed-size boundaries | Not inferred: fixed dimensions alone do not remove all intrinsic-size or baseline dependencies. The explicit API makes containment a deliberate semantic choice. |

The Taffy adapter uses the dependency pinned in `Cargo.toml`; the upstream
[`compute_cached_layout` contract](https://docs.rs/taffy/latest/taffy/fn.compute_cached_layout.html)
describes lookup by node and layout inputs, followed by computation on a miss.

### Boundary contract and correctness

```rust
let panel = Element::layout_boundary(content)
    .width(length(320.0))
    .height(length(240.0));
```

A boundary is a container with exactly one content child. Both axes must clip
or scroll. Its content contributes no intrinsic size to the wrapper; authored
constraints, padding and border still apply. Give it an external size, usually
an explicit height and a width supplied by its parent. It is not a drop-in
replacement for an auto-sized text panel or a content-derived baseline.

The private root applies the wrapper's normal container layout with its final
allocated size and the original parent inputs. This preserves percentage
padding and scrollbar geometry. Rounding uses the outer node's global
unrounded position, including nested boundaries and fractional ancestors.
Content overflow is copied back for scrolling without exporting a dependency
to outer sizing. UI ancestry, inherited styles, event paths and semantics remain
unchanged. Each boundary adds one internal Taffy node and a sparse registry entry;
there is a CPU/memory tradeoff, not a free optimization on every node.

Regression tests compare isolated and ordinary geometry, scroll extents and
paint output across resizing and fractional movement. They exercise nesting,
hidden ancestors, changes of portal target, boundary removal and invalid API
contracts. A custom ancestor's existing layout counter proves that changing a
leaf inside the boundary does **not** remeasure that ancestor; changing the
wrapper width does. Other regressions compare retained and fresh layout after
batched invalidations, reordering, removal and compaction, and preserve inherited
selection and direction when shared subtrees are reused.

The native validation also exposed an activation problem: after suspension,
an unmanaged popup could receive a click while OS focus stayed elsewhere.
Pointer/touch presses now explicitly focus its native surface when necessary.
The native test exercises reactivation after focus loss and verifies that
dismissal does not steal focus back. This event-driven fix does not affect the
headless benchmark paths; their executable snapshots precede this native-only fix.

### Measurements after each step

The complete [incremental measurement data](data/incremental-optimizations-data.json)
contains every retained sample, executable hashes, DHAT totals, hardware counters
and the targeted follow-ups. Intermediate snapshots were built sequentially;
each step below compares against the preceding snapshot.

| Step | Concrete workload | Result and decision |
| --- | --- | --- |
| Optional storage | 10,101 retained layout nodes; gallery DataTable after two scroll actions | Live heap -23.1% / -7.7%. Keep for memory. No general CPU gain claimed. |
| Early dirty stop | 200 changing siblings below 64 ancestors, 200 updates | 1,680.78 → 1,462.14 ms (-13.0%). Keep for grouped invalidation. Gallery timings do not establish a gain from this step alone. |
| Index/subtree reuse | 200 updates on a 1,641-node tree | Partial paint -19.3%; one panel's layout -22.2%; keyed structural changes -13.5%. Keep. Accordion -4.8%; Table's apparent regression did not repeat in the longer targeted run. |
| Compact parent/context/children | 10,101-node retained graph; five 1,000/100-leaf replacement cycles | Another 323,232 live bytes removed, with unchanged allocation counts. The churn case ends at 538,244 bytes rather than 541,532. Keep for memory; no separate CPU gain assigned. |
| Explicit isolation | Same 40 clipped panels / 1,681 UI nodes, ordinary vs isolated, 300 updates | 4,911.32 → 4,240.27 ms (-13.7%); hardware instructions -19.8%. Keep as an explicit capability. |
| ScrollArea integration | Gallery MessageScroller, 40 append actions / 481 final UI nodes | 288.10 → 163.66 ms (-43.2%); hardware instructions -46.1%. Keep. |

Some early time differences were not repeatable. After step 1, Table initially
looked 15% slower, but the longer comparison was 12.9% faster; partial paint's
initial +6.7% became -1.3%. After step 3, Table's initial +25.9% became -1.4%.
These follow-ups resolve whether a regression is consistently present; they
are not a reason to advertise whichever sample was fastest.

### Final retained memory against `52c9f32`

DHAT live heap at immediate process exit, with the benchmark objects retained:

| Workload | Before, bytes | After, bytes | Change |
| --- | ---: | ---: | ---: |
| 10,101 layout nodes | 62,390,819 | 47,676,143 | **-23.6% / -14.71 MB** |
| Gallery DataTable, 318 UI nodes, two scroll actions | 9,029,113 | 8,321,090 | **-7.8% / -708 kB** |

For the large graph, cumulative allocation drops from 114,947,564 to 95,077,784
bytes (-17.3%); live allocation count stays essentially constant (20,536 →
20,537). On DataTable, live blocks increase from 68,355 to 68,467 because optional
features now own small allocations, while retained bytes decrease.

Boundary-specific comparisons use stage 4 as the MessageScroller reference and
the same final binary with isolation off/on for the synthetic panels:

| Workload | Ordinary content, bytes | Isolated content, bytes | Net retained difference |
| --- | ---: | ---: | ---: |
| 40 panels, two updates | 8,130,533 | 8,145,333 | +14,800 (+0.18%) |
| MessageScroller, 20 appends | 4,907,172 | 4,847,012 | -60,160 (-1.2%) |

These are whole-workload totals, including caches; they do not claim that an
internal root itself occupies 370 bytes. MessageScroller also allocates 6.8%
fewer cumulative bytes and 29.6% fewer cumulative blocks over those 20 appends.

### Final CPU observations

Wall times below are medians of three paired runs, 300 update actions per run,
against `52c9f32`. They include UI reconciliation and the appropriate layout,
scroll or paint path. The gallery cases also dispatch real component events and
render their models. They exclude native windows and GPU submission.

| Workload | Before, ms | After, ms | Observed time change |
| --- | ---: | ---: | ---: |
| Partial paint, 1,641 nodes | 1,336.38 | 1,084.03 | -18.9% |
| Grouped invalidation, 265 nodes | 3,109.14 | 2,602.57 | -16.3% |
| One panel's layout, 1,641 nodes | 2,576.09 | 1,923.28 | -25.3% |
| Keyed structural changes, 1,641 nodes | 3,573.02 | 2,840.42 | -20.5% |
| Wide row, 2,001 nodes | 4,284.60 | 4,265.19 | -0.5% |
| Gallery Toggle | 3,331.11 | 1,344.78 | -59.6% |
| Gallery Accordion | 1,241.67 | 1,119.90 | -9.8% |
| Gallery Sidebar | 1,135.40 | 1,105.60 | -2.6% |
| Gallery VList | 333.64 | 299.50 | -10.2% |
| Gallery Table | 132.77 | 141.76 | +6.8% |
| Gallery DataTable | 596.22 | 417.65 | -29.9% |

**The large Toggle and DataTable time differences are not evidence of equally
large reductions in CPU work.** Frequency and desktop activity are uncontrolled.
Toggle ranges from 970–3,598 ms before and 903–3,252 ms after. MessageScroller's
first pair was slower with isolation (678 → 740 ms), despite the lower median.
Table remains slightly slower in the final wall-time comparison; there is no
uniform gallery speedup.

To separate work reduction from those timing variations, a second short series
records retired user-space instructions with `perf stat`. Each process is pinned
before `perf` starts; the P-core counter runs at 100%, with no multiplexing.
These are medians of three pairs and include startup, unlike the internal timers.

| Workload | Frames | Before, million instructions | After, million instructions | Change |
| --- | ---: | ---: | ---: | ---: |
| Partial paint, baseline → final | 150 | 1,143.16 | 965.71 | **-15.5%** |
| Grouped changes, baseline → final | 150 | 3,435.96 | 3,031.73 | **-11.8%** |
| Gallery Toggle, baseline → final | 150 | 1,364.31 | 1,312.14 | -3.8% |
| Gallery DataTable, baseline → final | 150 | 687.90 | 683.34 | -0.7% |
| Panels, isolation off → on | 150 | 2,043.86 | 1,639.76 | **-19.8%** |
| MessageScroller, stage 4 → final | 40 | 1,228.32 | 661.58 | **-46.1%** |

Instructions are a useful check on work performed, not a universal latency or
energy metric. The strongest supported gains here concern partial updates,
grouped invalidation and growing content inside independent scroll viewports.

### Protocol and reproduction

The new workloads are opt-in examples:
[`incremental_profile`](../../crates/argui-layout/examples/incremental_profile.rs),
[`boundary_profile`](../../crates/argui-layout/examples/boundary_profile.rs) and
[`update_profile`](../../crates/argui-widget-gallery/examples/update_profile.rs).
There are no new runtime profiling counters, periodic scans or sampling threads.
Geometry checksums, retained node counts and full-layout counts must agree across
paired runs. Exact geometry and paint comparisons belong to the regression tests.

Copy `incremental_profile.rs` and `update_profile.rs` into a checkout of
`52c9f32` for the reference. Build reference and candidate with separate Cargo
target directories, using the same command, and save the executables before
switching revisions:

```sh
cargo build -p argui-layout -p argui-ui -p argui-widget-gallery \
  --examples --release --all-features
python3 scripts/profile-incremental.py target/reference target/candidate \
  --cases paint batch panel structure wide toggle accordion sidebar vlist table data-table \
  --frames 300 --runs 3 --cpu 0 --output target/incremental-comparison.json
python3 scripts/profile-incremental.py target/candidate target/candidate \
  --cases boundary --frames 300 --runs 3 --output target/boundary-comparison.json
```

The boundary case selects `normal` for the first binary and `isolated` for the
second: both use the same executable and UI graph. MessageScroller uses
`--cases message-scroller --frames 40`; its isolated stage comparison needs the
stage 4 executable with the same benchmark case enabled. Intermediate snapshots
and the longer targeted follow-ups are identified in the raw data.

The driver performs one unrecorded warm-up pair, then alternates reference and
candidate order. It also records process CPU time and maximum RSS, without
confusing either with DHAT live heap. No build, coverage, browser or DHAT job runs
alongside accepted timings. Samples overlapping build work are discarded.
Hardware counters and heap profiles are collected separately, for example:

```sh
taskset -c 0 perf stat -e instructions:u -- target/candidate/update_profile message-scroller 40
valgrind --tool=dhat --dhat-out-file=layout-heap.json \
  target/candidate/layout_profile 10000 heap
valgrind --tool=dhat --dhat-out-file=gallery-heap.json \
  target/candidate/update_profile data-table 2 heap
valgrind --tool=dhat --dhat-out-file=boundary-heap.json \
  target/candidate/boundary_profile isolated 2 heap
```

DHAT excludes allocator metadata, unused allocator pages, stacks and GPU memory.
The examples include font shaping where relevant, but do not measure glyph
preparation, native event-loop idle cost or rendered frame rate. Visual checks use
the WebGPU gallery catalogue at 1220 × 780 and 800 × 720, in both themes, on the
[private Linux display](../contributing/linux-testing.md). Their screenshots verify rendering
and interactions; browser timings are not added to the CPU tables.

The implementation still assembles full layout output, maintains other UI
registries and does painting work. A sparse root registry and shared-subtree
reuse do not make every update proportional only to the changed leaf. Further
optimization should start with a workload exposing one of those remaining costs,
not with additional per-node caches or flags by default.

## Earlier work

The [gallery footprint study](footprint.md#widget-gallery-release-footprint) records the
2026-09-10 paint-cache cleanup, reduced copying and lazy data-model allocation,
including native RSS and the limits of those native CPU observations. Those
numbers use a different baseline and must not be added to this study's percentages.
The [roadmap](../roadmap.md) tracks further validation and performance work.
Benchmark size should always distinguish total model rows from the nodes
actually present in the UI tree.

## DevTools resource collection and editing

The DevTools expansion adds a reusable color editor, scoped live themes, hover
highlighting, resizable property panes and process/device telemetry. It is a
feature expansion, not a demonstrated reduction in whole-process RAM.
[Raw samples and exclusions](data/devtools-resources.json) compare this revision
with `1e78722` on Linux 7.1.12, Core Ultra 5 125H and Intel Arc/Meteor Lake.

Two valid alternating native pairs used the real release gallery, all features,
a 1220 × 1000 private X11 window, and ten seconds per pane after settling.
No builds or other tests ran concurrently. A third pair was excluded because its
baseline click did not activate Profiling; its captures showed Elements instead.
The driver now checks the active pane and rejects a stopped loading spinner.
Numbers below are means across the two valid runs, with 100% CPU representing
one logical CPU. The Buttons page contains a continuously animated spinner.

| Native state | RSS before → after | Process CPU before → after |
| --- | ---: | ---: |
| Tools closed | 148.2 → 159.5 MiB | 60.2 → 66.0% |
| Elements | 170.3 → 171.1 MiB | 87.6 → 92.4% |
| Profiling overview | 186.9 → 183.3 MiB | 92.9 → 98.1% |
| Resources, process sampling enabled | — → 185.5 MiB | — → 100.2% |

These measurements do not establish an overall CPU/RAM improvement. Closed RSS
increased by 11.3 MiB in this small sample; allocation and driver attribution is
not established. Native timing varied substantially between launches. Within
the candidate runs, changing Profiling overview to Resources added about
2.2 MiB RSS and 2.1 percentage points of process CPU on average. That includes
the resource panel, process sampler and changed renderer workload; it does not
isolate the sampler alone or include an external all-smi process.

The existing renderer-free `argui-devtools` profiling example also ran in three
alternating pairs, with 40 warm-up ticks and 200 recorded ticks per state.
Median run p95 CPU was 0.281 → 0.182 ms closed, 0.386 → 0.287 ms in Elements,
and 10.410 → 8.568 ms in Profiling. These synthetic results omit native GPU
submission/presentation and do not explain away the native overhead above.

Costs are constrained structurally and tested: no process collection before
Resources is selected, no new request while closed/paused/on another pane,
at most one worker request in flight, and at most one sample per second.
GPU sensors require a separate explicit opt-in. UI/layout/paint capacity counters
read Vec capacities without traversing nodes, and the color pad uses one GPU
quad with four shared corners; adding `Fill::Bilinear` preserves `size_of::<Fill>()`.
The collector never inserts a heap scan or vendor query into a render pass.
RSS mapping categories sum to measured resident memory; known Argui capacities
and GPU allocations are kept separate to avoid false totals.

Reproduce with the commands in [DevTools](../contributing/devtools.md#cost-and-reproduction)
and the [native driver](../contributing/linux-testing.md#contrôle-final).
Save each release binary before switching revisions and inspect the captures.
The optional all-smi adapter detected this machine's GPU, but its driver readings
were all zero; these are shown as unavailable, with raw fields retained. No
cross-platform sensor completeness or Windows/macOS execution is claimed.
