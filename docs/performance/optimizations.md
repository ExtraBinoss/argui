# Performance

Argui retains its UI tree, layout caches and paint data, reuses unchanged
subtrees and schedules frames only when needed. Virtualized controls mount a
bounded visible window. Explicit layout boundaries isolate content whose
intrinsic size does not affect its parent.

These are measured workloads, not a promise about every Argui application.
The historical snapshots below retain their dates, baselines and raw samples.
Do not add percentages from different studies or treat a cached computation
as the cost of an idle event loop.

## What the numbers mean

- **RSS** includes resident shared libraries and mapped graphics resources.
- **Private memory** identifies process-private pages; it is not total RSS.
- **Retained heap** from DHAT excludes stacks, GPU allocations, allocator metadata
  and unused allocator pages.
- **GPU resident memory** can overlap system-memory mappings on an integrated GPU.
- **CPU percent** uses 100% for one occupied logical CPU. Process CPU seconds,
  frame time and input-to-display latency are different measurements.

Use matching feature flags, viewport, workload and display conditions. Alternate
reference/candidate order, warm up first and save the actual executables. Do not
compile, run coverage or launch other tests while timing. Keep GPU windows
actively presenting; a suspended private window can produce misleadingly low CPU.

## Widget gallery release footprint

The startup sample on 2026-09-10 used Linux, an Intel Core Ultra 5 125H and its
integrated GPU. Three alternating pairs were sampled three seconds after launch
on the initial Buttons page. Normal and all-feature builds use different native
hosts and are compared only within their own configuration.

| Measurement | Normal release, before → after | All features, before → after |
| --- | ---: | ---: |
| CPU consumed during the first three seconds | 0.487 → 0.387 s | 0.860 → 0.723 s |
| GPU resident memory | 97.44 → 81.94 MiB | 97.48 → 81.23 MiB |
| RSS | 99.69 → 100.16 MiB | 153.92 → 152.96 MiB |
| Private memory | 55.74 → 56.34 MiB | 72.01 → 71.14 MiB |

The main change was GPU storage: a smaller initial vector atlas and lazy effect
compilation. Process RAM was broadly unchanged. The initial atlas grows when
needed; lazy compilation pays the cost on first use. No rendering-quality option
was disabled. [Raw startup samples](data/widget-gallery-startup.json).

A separate release scrolling workload used a 1220 × 780 native window, embedded
fonts, assets and DevTools, with four startup seconds excluded and 20 seconds
of scrolling. Two alternating pairs gave these final-sample means:

| Page | RSS, before → after | Private memory, before → after |
| --- | ---: | ---: |
| Table, idle | 171.71 → 166.91 MiB | 79.25 → 74.46 MiB |
| VList, scrolling | 179.68 → 171.62 MiB | 86.89 → 79.11 MiB |
| DataTable, scrolling | 187.03 → 176.29 MiB | 94.10 → 83.22 MiB |

Paint-cache cleanup released removed rows, reduced intermediate copies and
created the gallery's large DataTable only on first use. Idle CPU was 0% in
these sampled idle runs. Scrolling CPU varied substantially and did not establish
a general CPU improvement. Later samples in which the private compositor stopped
presenting were discarded.

```sh
cargo build -p argui-widget-gallery --release --all-features --example footprint
./scripts/linux-hidden-display.sh \
  python3 scripts/profile-widget-gallery.py --page data-table --scroll --seconds 20
```

For initial-page sampling, build the application and pass `--app --warmup 3`.
The sampler accepts `--binary` for a saved executable and reports CPU, RSS, PSS,
private memory and available GPU counters. Its scrolling fixture includes an
extra UI tree used to inject controlled scrolling in both compared builds.

## Retained tree and layout

The 2026-09-12 study used Rust 1.98.0, release builds with all features, Linux
x86-64 and the same Core Ultra 5 125H. Dense storage against `fb88ea7` separated
layout geometry/caches from node metadata, compacted removed nodes and replaced
linear parent lookups. Subsequent work against `52c9f32` moved large optional
properties out of every element, stopped repeated dirty propagation, reused
unchanged indices and introduced explicit layout boundaries.

| Workload | Baseline | Before | After |
| --- | --- | ---: | ---: |
| 10,101-node UI + layout + paint, live heap | `fb88ea7` | 70,495,834 B | 62,390,819 B |
| Final shrink to 102 nodes, live heap | `fb88ea7` | 20,816,230 B | 866,020 B |
| Five grow/shrink cycles, cumulative allocation | `fb88ea7` | 587,553,977 B | 634,879,508 B |
| 10,101-node layout, live heap | `52c9f32` | 62,390,819 B | 47,676,143 B |
| Gallery DataTable after two scroll actions, live heap | `52c9f32` | 9,029,113 B | 8,321,090 B |

Compaction releases memory but can increase allocation on repeated regrowth.
The large synthetic graph is distinct from a virtualized list's total model
size: the gallery mounts only hundreds of nodes, even with many thousands of rows.

Final wall-time medians against `52c9f32`, three pairs and 300 updates per run:

| Workload | Before | After |
| --- | ---: | ---: |
| Partial paint, 1,641 nodes | 1,336.38 ms | 1,084.03 ms |
| Grouped invalidation, 265 nodes | 3,109.14 ms | 2,602.57 ms |
| One panel's layout, 1,641 nodes | 2,576.09 ms | 1,923.28 ms |
| Gallery VList | 333.64 ms | 299.50 ms |
| Gallery Table | 132.77 ms | 141.76 ms |
| Gallery DataTable | 596.22 ms | 417.65 ms |

These CPU replays exclude native windows and GPU submission. Frequency and
background activity were uncontrolled: not every case improved, and wall-time
gains do not imply equal reductions in work. Separate retired-instruction
comparisons measured reductions of 15.5% for partial paint and 11.8% for grouped
changes. Full layout output and other UI registries still require work beyond
the changed leaf.

An explicit `Element::layout_boundary(content)` needs an externally determined
size and clipping/scrolling on both axes. Content does not supply the wrapper's
intrinsic size or baseline. It adds an internal root and sparse registry entry,
so use it where containment is semantically appropriate. See
[scroll and layout boundaries](../ui/scroll.md).

[Dense-storage samples](data/optimizations-data.json) and
[incremental samples](data/incremental-optimizations-data.json) include executable
hashes, heap measurements, intermediate comparisons and hardware counters.

## Reproduce engine measurements

Build reference and candidate in separate target directories, preserving each
executable before switching revisions. Use the same benchmark sources and flags.
Older baselines may need the current benchmark examples copied into their checkout.

```sh
cargo build -p argui-layout -p argui-ui -p argui-widget-gallery \
  --examples --release --all-features
python3 scripts/profile-trees.py target/reference target/candidate \
  --gallery --runs 7 --gallery-runs 3 --cpu 0 --output target/tree-comparison.json
python3 scripts/profile-incremental.py target/reference target/candidate \
  --cases paint batch panel structure wide toggle accordion sidebar vlist table data-table \
  --frames 300 --runs 3 --cpu 0 --output target/incremental-comparison.json
```

`profile-trees.py` expects `tree_profile`, `layout_profile` and, with `--gallery`,
the saved Nextest pages binary named `gallery-pages` in each directory.
`profile-incremental.py` uses `incremental_profile`, `boundary_profile` and
`update_profile`. Their sources live in the corresponding crate's `examples/`.
The boundary case compares ordinary and isolated content in the same candidate
binary; use `--cases boundary` with both directory arguments set to the candidate.

Measure heap and instruction counts separately from timings:

```sh
valgrind --tool=dhat --dhat-out-file=layout-heap.json \
  target/candidate/layout_profile 10000 heap
taskset -c 0 perf stat -e instructions:u -- \
  target/candidate/update_profile message-scroller 40
```

Keep geometry checksums, retained node counts and full-layout counts comparable.
Use the [quality gate](../contributing/code-quality.md) for behavioral correctness.

## DevTools overhead

DevTools collects process resources only while Resources is selected, with at
most one request in flight and one sample per second. Closed, paused or other
panes do not schedule collection. GPU sensors are separately opt-in. Capacity
counters and GPU allocations are displayed separately from RSS.

Two native pairs against `1e78722` used release/all-feature builds and ten seconds
per pane. Closed-tools RSS increased from 148.2 to 159.5 MiB; Profiling changed
from 186.9 to 183.3 MiB. These samples do not establish a whole-process improvement.
[Raw samples and exclusions](data/devtools-resources.json) preserve the workload
and variation. Reproduction is in [DevTools](../contributing/devtools.md#cost-and-reproduction).

## Color picker interaction cost

The 2026-09-13 comparison against `d756e88` used three alternating release pairs,
40 warm-up updates and 300 measured updates per case at 1220 × 1000. It covers
event dispatch, model rendering, overrides, reconciliation, layout, text
preparation and inspection, excluding GPU presentation.

| Interaction | Median update, before → after | p95 update, before → after |
| --- | ---: | ---: |
| Gallery saturation/value pad | 4.163 → 3.348 ms | 15.621 → 13.994 ms |
| Gallery hue slider | 3.976 → 3.340 ms | 15.047 → 13.921 ms |
| DevTools background picker | 7.766 → 5.744 ms | 25.218 → 20.536 ms |
| DevTools Theme picker | 8.754 → 8.391 ms | 25.812 → 24.895 ms |

The Theme result varied across pairs and does not establish a dependable speedup.
The live preview no longer waits for an inherited button color transition;
empty overrides skip copying, and custom decoration replaces redundant layout
nodes. All full-drag cases still require layout when displayed color text changes.
Counted UI/layout capacity is smaller, but those counters do not measure total RSS.

```sh
cargo run -p argui-widget-gallery --example color_picker_profile --release --all-features
```

[Release samples](data/color-picker.json) retain the three pairs. A separate
**development-profile** comparison against `93227a3` measured average process CPU
of 12.710 → 10.334 seconds and retired instructions of 30.398 → 25.183 billion.
[Development samples](data/color-picker-transitions.json) contain those two pairs.
Neither replay measures physical input-to-display latency or proves a process RAM
saving. Do not combine release and development timings.

## Web VList accessibility synchronization

The browser adapter preserves unchanged semantic child order, caches CSS bounds
and semantics, and rewrites ARIA only when semantics change. Removed nodes release
caches and handlers. These changes preserve the accessible tree and focus.

Two before/after Chromium profiles on 2026-09-13 replayed 100 wheel inputs over
10,000 variable-height rows with the same development WASM build, Vulkan WebGPU
and 1220 × 900 viewport. Only 23–24 rows were mounted in the final sampled states.

| Sampled self time | Before, two runs | After, two runs |
| --- | --- | --- |
| `appendChild` | 190 / 149 ms | 0 / 0 ms |
| CSS `setProperty` | 62 / 65 ms | 12 / 8 ms |
| `TextDecoder.decode` | 188 / 119 ms | 18 / 17 ms |

Median frame interval remained 16.7 ms. The p95 improved in one pair and regressed
in the other: these profiles establish reduced DOM work, not a stable frame-rate
or end-to-end latency guarantee. The website's release WASM optimization is
separate from this comparison.

Run `crates/argui-widget-gallery/tests/pages/data.mjs` through the
[private display](../contributing/linux-testing.md), with `CHROME_PATH`,
`PUPPETEER_MODULE`, `GALLERY_URL` and `PROFILE_LABEL=before` or `after`.
Profiles and captures are saved in `target/web-vlist/`.
`crates/argui-accessibility/tests/web.mjs` checks row replacement, order, bounds,
accessible actions and DOM focus preservation.
