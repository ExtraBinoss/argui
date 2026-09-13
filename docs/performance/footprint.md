# Application and gallery footprint

Measured on 2026-09-10 after the earlier scroll/cache optimizations, using the
real `argui-widget-gallery` executable on its initial Buttons page. No synthetic
scrolling or input was injected. Hardware: Intel Core Ultra 5 125H, integrated
Intel GPU, Linux. Windows were placed in a separate headless Mutter session to
leave the user's desktop uninterrupted.

## General renderer changes

- The vector atlas starts at 256 × 256 instead of 2048 × 2048: 256 KiB instead
  of 16 MiB. It doubles when necessary up to the same 2048 × 2048 capacity.
  Growth rebuilds the texture binding and current frame's coordinates; capacity
  exhaustion still supports eviction and an explicit error if a frame cannot fit.
- Registered custom effects are compiled for the GPU when first used and then
  retained. Shader validation still happens when the registry is constructed,
  including shaders not yet used by a frame.
- Extending an effect registry validates only the added definition. Gallery and
  devtools configuration preserve the existing validated registry instead of
  rebuilding and validating it several times.

The changes apply to the renderer and effect configuration, independently of
which gallery page is open. No animation, feature or rendering quality setting
was disabled. First use of an effect now pays its compilation cost once.

## Before and after

Three alternating pairs per feature configuration, sampled three seconds after
launch; means below. No compilation ran during these final measurements.
The baseline includes the previous scroll and paint-cache fixes.
[Individual samples](data/widget-gallery-startup.json) are preserved for review.

| Measurement | Normal release before → after | Release with all features before → after |
| --- | ---: | ---: |
| Process CPU consumed in first 3 seconds | 0.487 → 0.387 s (−20.5%) | 0.860 → 0.723 s (−15.9%) |
| GPU resident memory | 97.44 → 81.94 MiB (−15.50 MiB) | 97.48 → 81.23 MiB (−16.25 MiB) |
| Process RSS | 99.69 → 100.16 MiB | 153.92 → 152.96 MiB |
| Process private memory | 55.74 → 56.34 MiB | 72.01 → 71.14 MiB |

CPU is process user + system time, not startup latency or steady-state CPU
utilization. The normal executable and the all-features executable use different
native hosts; each is compared only with the same feature configuration.

Process RAM remains broadly unchanged within launch-to-launch variation. The
clear memory saving is GPU-resident storage, reported by Linux DRM after
deduplicating file descriptors for each DRM client. On this integrated GPU it
uses shared system memory; do not add it to RSS because some mappings can overlap.
The isolated session can stop presenting animations after startup, so these
results do not establish a reduction in continuous-animation or idle CPU usage.

CPU sampling identified repeated shader parsing/validation during startup.
Massif also identified Vulkan texture and command-buffer allocations as the
dominant allocation costs. Renderer integration checks exercise atlas growth,
maximum capacity, recovery, repeated frames, first use of a multipass effect and
missing-effect errors through a real Wayland surface.
On the first-presentation workload, Massif's peak heap including driver
allocations decreased from 98.59 to 81.35 MiB. Instrumented execution times are
excluded from the CPU comparison above.

## Reproduce

Build the normal executable and run the sampler inside a dedicated test display:

```sh
cargo build -p argui-widget-gallery --release
./scripts/linux-hidden-display.sh \
  python3 scripts/profile-widget-gallery.py --app --warmup 3 --seconds 10
```

For the all-features configuration, add `--all-features` to the build command.
Save each executable before changing source so paired measurements use the
correct binaries. The sampler accepts `--binary /path/to/saved/executable`.
Its startup record reports process CPU seconds, RSS, PSS, private memory and,
where supported, GPU-resident memory. Subsequent records report CPU percent,
where 100% is one logical CPU, and memory at five-second intervals.

Run the optional native surface check on the dedicated Wayland display:

```sh
./scripts/linux-hidden-display.sh \
  cargo nextest run -p argui-render --all-features --test surface \
  --run-ignored ignored-only
```

## Widget gallery release footprint

Measured on 2026-09-10, Linux/GNOME Wayland, Intel Core Ultra 5 125H
(18 logical CPUs), Intel GPU, release builds with `--all-features`.
The comparison starts after the gallery UI and first-paint text fixes, before
the footprint changes described below. It does not compare against debug builds.

### Changes

- Discard paint fragments when their nodes disappear, move to another index, or
  receive replacement elements. Removed virtual rows previously kept their
  element properties and drawing commands alive through the paint cache.
- Extend retained drawing output directly from cached commands and hit regions,
  avoiding intermediate vector allocations and copies.
- Carry the intersection of ancestor clip bounds through painting instead of
  transforming every ancestor clip again for every accessible node.
- Create the gallery's 10,000-row DataTable model on its first visit. Keep that
  model afterward so edits and selection survive navigation.

### Engine workload

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

### Native window

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

### Reproduce

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
./scripts/linux-hidden-display.sh \
  python3 scripts/profile-widget-gallery.py --page data-table --scroll --seconds 20
```

The wrapper opens the workload on a private display; see
[Linux testing](../contributing/linux-testing.md) for its prerequisites.
Keep the window actively presenting throughout CPU measurements; a hidden or
suspended window is not a scrolling benchmark. The sampler prints CPU, RSS,
PSS and private memory every five seconds and closes its own workload afterward.
Use the same viewport, features, workload and display setup in both builds,
alternate the order, and avoid compiling during timing runs.
