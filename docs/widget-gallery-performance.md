# Widget gallery release footprint

Measured on 2026-09-10, Linux/GNOME Wayland, Intel Core Ultra 5 125H
(18 logical CPUs), Intel GPU, release builds with `--all-features`.
The comparison starts after the gallery UI and first-paint text fixes, before
the footprint changes described below. It does not compare against debug builds.

## Changes

- Discard paint fragments when their nodes disappear, move to another index, or
  receive replacement elements. Removed virtual rows previously kept their
  element properties and drawing commands alive through the paint cache.
- Extend retained drawing output directly from cached commands and hit regions,
  avoiding intermediate vector allocations and copies.
- Carry the intersection of ancestor clip bounds through painting instead of
  transforming every ancestor clip again for every accessible node.
- Create the gallery's 10,000-row DataTable model on its first visit. Keep that
  model afterward so edits and selection survive navigation.

## Engine workload

Three alternating before/after pairs, 6,000 scroll steps per page, 4 logical
pixels per step, 1280 × 900 viewport. Medians:

| Measurement | Before | After | Reduction |
| --- | ---: | ---: | ---: |
| VList, 6,000 steps | 1223.41 ms | 1187.61 ms | 2.9% |
| Table, 6,000 steps | 269.32 ms | 244.36 ms | 9.3% |
| DataTable, 6,000 steps | 1544.22 ms | 1450.26 ms | 6.1% |
| Process user CPU time, all three pages | 3.02 s | 2.88 s | 4.6% |
| Process maximum RSS | 31.54 MiB | 18.30 MiB | 42.0% |

The workload exercises models, tree updates, layout, painting data and pointer
hit testing. It excludes glyph preparation and GPU submission. Retained node
counts and full layout counts were unchanged.

A separate Massif run with 1,200 steps per page reduced peak heap allocation,
including allocator overhead, from 10.91 MiB to 8.54 MiB (21.8%).
Massif timings are not used for the CPU comparison.

## Native window

The native `footprint` example opens a 1220 × 780 window with the real renderer,
embedded font, gallery assets, selection host and devtools wrapper. Its timer
requests scrolling at 960 logical pixels per second after a three-second delay.
The driver maintains a second UI tree to deliver controlled scroll events; its
overhead is included identically in both builds.

Two alternating before/after pairs, four seconds of startup excluded, then
20 seconds of scrolling. Values below are the mean of the final samples:

| Page | RSS before → after | Private memory before → after |
| --- | ---: | ---: |
| Table, idle | 171.71 → 166.91 MiB | 79.25 → 74.46 MiB |
| VList, scrolling | 179.68 → 171.62 MiB | 86.89 → 79.11 MiB |
| DataTable, scrolling | 187.03 → 176.29 MiB | 94.10 → 83.22 MiB |

Between the first and last scrolling samples, DataTable RSS grew by 7.85 MiB
before and 0.71 MiB after. VList grew by 2.20 MiB before and 0.04 MiB after.
RSS includes shared libraries and mapped graphics resources; private memory
helps distinguish those from application-owned allocations.

Idle CPU was 0% in both corrected runs. Unrestricted scrolling CPU varied
considerably on this hybrid CPU: DataTable samples were 20.75–35.32% before and
32.92–42.90% after. A follow-up with affinity restricted to performance cores
(logical CPUs 0–7) measured 20.65% before and 20.45% after over ten seconds,
but only one pair completed. These native measurements do not establish a CPU
improvement or a stable regression; the repeatable CPU gain above is limited to
the engine workload. 100% here means one logical CPU fully occupied.

Later graphical checks used a separate headless Mutter display to avoid
interrupting the user's desktop. Wayland tracing showed that image presentation
stopped after startup there, so those low CPU samples were discarded.
The optional native lifecycle test also fails its rendered-theme assertion in
that session, including when run from the saved pre-footprint binary. It is not
counted as a passing native integration check.

## Reproduce

Run the engine benchmark without creating a native window:

```sh
ARGUI_PROFILE_FRAMES=6000 cargo nextest run -p argui-widget-gallery \
  --all-features --cargo-profile release --test pages \
  --run-ignored ignored-only -E 'test(=data::profile_virtual_scrolling)' \
  --success-output immediate
```

The native sampler uses only Python's standard library and Linux `/proc`:

```sh
cargo build -p argui-widget-gallery --release --all-features --example footprint
python3 scripts/profile-widget-gallery.py --page data-table --scroll --seconds 20
```

The latter command opens a window on the display supplied by its environment.
Use a dedicated test session when the main desktop must remain uninterrupted.
Keep the window actively presenting throughout CPU measurements; a hidden or
suspended window is not a scrolling benchmark. The sampler prints CPU, RSS,
PSS and private memory every five seconds and closes its own workload afterward.
Use the same viewport, features, workload and display setup in both builds,
alternate the order, and avoid compiling during timing runs.
