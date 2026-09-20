# Adaptive damage rendering

Argui compares each retained paint scene with the scene that was last presented.
Small changes update only conservative physical-pixel rectangles in a persistent
GPU texture. Large or fragmented changes continue to render directly to the
swapchain, avoiding a permanent full-screen copy cost.

Damage rendering is enabled by default on native and WebAssembly renderers. It
does not change layout, hit testing, animation timing, paint order, or widget
APIs.

## Frame decisions

`RenderProfile::damage.mode` reports one of four decisions:

| Mode | Work performed |
| --- | --- |
| `Full` | Clear and draw the complete scene directly to the surface. |
| `Seed` | Draw the complete scene once into the retained root, then present it. |
| `Partial` | Clear and redraw only merged damage rectangles, then present the retained root. |
| `Reused` | Present an unchanged retained root without repainting it. |

The default policy merges touching rectangles on 32-pixel tile boundaries. It
chooses a full frame above eight merged rectangles or 45% of the viewport. A
direct full frame drops the retained root, so scrolling and full-window
animation do not pay an extra texture copy or memory cost indefinitely.

The retained root uses approximately `width * height * 4` bytes for the usual
RGBA8 surface format. For example, 1920×1080 needs about 7.9 MiB while partial
rendering is active.

## Configuration

Use `RendererConfig::damage_tracking` to tune the policy or opt out:

```rust
use argui_render::{DamageTracking, RendererConfig};

let renderer = RendererConfig::default().damage_tracking(
    DamageTracking::enabled()
        .max_regions(6)
        .max_area_ratio(0.35),
);

let always_full = RendererConfig::default()
    .damage_tracking(DamageTracking::disabled());
```

The same policy can be changed later for an individual window:

```rust
use argui::{
    platform::WindowKey,
    render::DamageTracking,
    runtime::{AppCommand, Context},
};

fn use_full_frames<App: 'static>(cx: &mut Context<App>) {
    cx.command(AppCommand::SetDamageTracking {
        window: WindowKey::main(),
        tracking: DamageTracking::disabled(),
    });
}
```

Pass `DamageTracking::enabled()` to return to the default adaptive policy, or
chain `max_regions` and `max_area_ratio` to select custom fallback thresholds.
Applications choose the policy, not a `DamageMode`: `Seed`, `Partial`, and
`Reused` are safety-dependent decisions made by the renderer, while disabling
tracking consistently produces full-frame rendering.

`DamageSnapshot` and `DamagePlan` expose the renderer-neutral comparison and
policy for custom renderer integrations. `RenderProfile::damage` includes the
chosen mode, region count, repainted pixel count, and retained texture bytes.
The runtime inspector forwards the repainted count as `damaged_pixels`.

## Live comparison

Open **Examples → Damage control** in the Widget Gallery to compare the real
renderer paths without changing the workload. The **Auto** tab uses the default
adaptive policy; **Off** forces full-surface redraws. Switching tabs invalidates
the retained root once, clears the rolling sample window, and then reports:

- the latest damage mode and merged-region count;
- repainted pixels as a count and viewport percentage;
- retained root memory;
- rolling renderer CPU encoding time;
- rolling GPU time when the adapter supports timestamp queries.

The Auto baseline can include a `Full` frame followed by `Seed`; judge the
steady state after several samples. GPU milliseconds are not GPU utilization,
and an unavailable value means the backend did not expose timestamp queries.
Repaint percentage is the portable comparison across native and WebAssembly
backends.

The controlled element uses a presentation `transform`, so the demo exercises
the compositor reuse path without rerunning Taffy. Composition-only frames
reuse prepared primitives and text, then feed the old and new visual bounds
through the same adaptive damage policy. The **Off** tab disables that policy
while keeping the workload identical.

Open **Open blur** or **Open glass** to put a bounded backdrop effect over the
moving element. Auto includes the filter's sampling halo when deciding whether
the change reaches that layer, repaints the complete effect output when needed,
and preserves the rest of the persistent effect root. The glass variant runs
three custom shader passes through the same path. Off still rebuilds the entire
effect root on every frame, which makes the comparison directly observable.

## Correctness boundaries

Damage bounds include both the old and new locations of changed primitives,
prepared glyph geometry, transforms, clipping, layers, and compositor regions.
They receive a small antialiasing margin before tile alignment. Image or vector
registration, resize, surface recreation, and incompatible render paths
invalidate the retained root.

Effect graphs use a persistent offscreen root. Built-in filters and bounded
custom effects propagate changed pixels through their layout-derived output and
sampling bounds before the adaptive threshold is applied. Regional clears and
scissored graph replay then update that root; unchanged frames only present it.
Custom shaders remain `Unbounded` unless their definition opts into bounded
damage, so unknown sampling behavior still selects a safe full composition.

## Verification checklist

- [x] Automatic old/new scene comparison with no widget annotations.
- [x] Quad, image, vector, text, GPU-canvas, transform, clip, and layer bounds.
- [x] Region clipping, tile alignment, transitive merging, and adaptive fallback.
- [x] Persistent WGPU target, regional clear, scissored replay, and presentation.
- [x] Effect-aware propagation, retained effect roots, and bounded custom shader API.
- [x] Safe invalidation on resize, surface recreation, and resource registration.
- [x] Native and WebAssembly compilation through the shared renderer path.
- [x] Public configuration, decision types, profiling counters, and opt-out.
- [x] Renderer-neutral policy and scene-diff tests plus native GPU lifecycle coverage.
