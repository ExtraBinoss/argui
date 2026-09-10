# Application footprint

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
[Individual samples](widget-gallery-startup.json) are preserved for review.

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
cargo nextest run -p argui-render --all-features --test surface \
  --run-ignored ignored-only
```
