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

The controlled element changes its painted layout bounds instead of using a
presentation `transform`. Argui promotes transforms to retained compositor
layers; those exercise compositor reuse rather than root-surface damage, so
mixing them into this workload would measure a different optimization.

## Correctness boundaries

Damage bounds include both the old and new locations of changed primitives,
prepared glyph geometry, transforms, clipping, layers, and compositor regions.
They receive a small antialiasing margin before tile alignment. Image or vector
registration, resize, surface recreation, and incompatible render paths
invalidate the retained root.

Effect graphs that require an offscreen root keep the existing full effect
composition path. Their retained layer cache still avoids regenerating
unchanged layer contents, while the final surface composition remains complete.
This prevents blur, shadow, refraction, and custom shader sampling from reading
stale pixels outside a local rectangle.

## Verification checklist

- [x] Automatic old/new scene comparison with no widget annotations.
- [x] Quad, image, vector, text, GPU-canvas, transform, clip, and layer bounds.
- [x] Region clipping, tile alignment, transitive merging, and adaptive fallback.
- [x] Persistent WGPU target, regional clear, scissored replay, and presentation.
- [x] Safe invalidation on resize, surface recreation, and resource registration.
- [x] Native and WebAssembly compilation through the shared renderer path.
- [x] Public configuration, decision types, profiling counters, and opt-out.
- [x] Renderer-neutral policy and scene-diff tests plus native GPU lifecycle coverage.
