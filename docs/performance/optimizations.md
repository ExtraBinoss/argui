# Performance

Argui's performance contract is structural:

- retained UI, layout, text, and paint data survive between frames;
- clean subtrees reuse their cached element descriptions;
- paint-only changes do not run layout or shape text;
- transforms and scroll offsets reuse layout geometry;
- virtual lists mount a bounded visible window;
- an idle application schedules no frame.

These rules are tested. Timing and memory values depend on hardware, drivers,
features, viewport, and workload; treat the values below as repository snapshots,
not guarantees for every application.

## Current snapshots

The native Widget Gallery release sample was measured on 2026-09-10 on Linux
with an Intel Core Ultra 5 125H integrated GPU. Values are means of three runs,
sampled three seconds after launch on the Buttons page.

| Build | CPU used in first 3 s | RSS | Private memory | GPU resident |
| --- | ---: | ---: | ---: | ---: |
| Default features | 0.387 s | 100.16 MiB | 56.34 MiB | 81.94 MiB |
| All features | 0.723 s | 152.96 MiB | 71.14 MiB | 81.23 MiB |

[Raw native samples](data/widget-gallery-startup.json) include every run and the
measurement fields. RSS includes shared libraries and mapped graphics memory;
private memory does not include all GPU allocations.

Release WebAssembly startup was measured on 2026-09-14 with Chrome, a local HTTP
server, disabled HTTP cache, and five loads per application:

| Application | Optimized Wasm | Median renderer-ready time |
| --- | ---: | ---: |
| AI Harness | 6.15 MiB | 296.9 ms |
| Widget Gallery | 9.84 MiB | 789.2 ms |

[Raw WebAssembly samples](data/wasm-startup.json) define the viewport, browser,
interval, and spread. Network transfer, compression, and cache policy change
remote startup.

Historical optimization data remains machine-readable:

- [tree, layout, heap, and gallery workloads](data/optimizations-data.json);
- [incremental update workloads](data/incremental-optimizations-data.json);
- [color-picker release samples](data/color-picker.json);
- [color-picker development samples](data/color-picker-transitions.json);
- [DevTools resource samples](data/devtools-resources.json).

The main guide does not preserve each intermediate implementation comparison.
Use the raw revision and environment fields when investigating one.

## Measure a native application

Build once, then sample the saved release binary without compiling during the
measurement:

```sh
cargo build -p argui-widget-gallery --release --all-features
./scripts/linux-hidden-display.sh \
  python3 scripts/profile-widget-gallery.py --app --warmup 3
```

For a scrolling workload:

```sh
./scripts/linux-hidden-display.sh \
  python3 scripts/profile-widget-gallery.py \
    --page data-table --scroll --seconds 20
```

Use `--binary` to measure a preserved executable. Keep the viewport, feature
set, display backend, warm-up, and input replay identical between runs.

## Compare engine revisions

Build the reference and changed revisions in separate target directories. The
profilers never compile:

```sh
python3 scripts/profile-trees.py target/reference target/changed \
  --gallery --runs 7 --gallery-runs 3 \
  --output target/tree-comparison.json

python3 scripts/profile-incremental.py target/reference target/changed \
  --cases paint batch panel vlist table data-table \
  --frames 300 --runs 3 \
  --output target/incremental-comparison.json
```

`profile-trees.py` consumes the `tree_profile`, `layout_profile`, and
optional `gallery-pages` executables. `profile-incremental.py` consumes
`incremental_profile`, `boundary_profile`, and `update_profile`.

Use DHAT for retained heap and `perf stat` for instruction counts:

```sh
valgrind --tool=dhat --dhat-out-file=target/layout-heap.json \
  target/changed/layout_profile 10000 heap

taskset -c 0 perf stat -e instructions:u -- \
  target/changed/update_profile message-scroller 40
```

## Measurement rules

- Alternate reference and changed runs after the same warm-up.
- Pin the same CPU when comparing short CPU workloads.
- Do not build, run coverage, or launch unrelated tests while sampling.
- Verify that graphical windows continue presenting. A suspended compositor can
  report unrealistically low CPU.
- Compare geometry checksums, retained-node counts, and layout counts before
  comparing time.
- Report regressions as well as improvements. Do not combine release and
  development profiles.
- Distinguish CPU time, frame interval, input-to-display latency, retained heap,
  RSS, private memory, and GPU memory.

Performance changes still pass the behavioral [quality gate](../contributing/code-quality.md).
Browser profiles and captures use the
[private Linux display](../contributing/linux-testing.md).
