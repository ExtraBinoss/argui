# GPU canvases

A GPU canvas embeds application-authored WGPU compute and render commands in
the normal retained Argui scene. It is intended for editor viewports, maps,
games, scientific visualization and other bounded views whose pixels are more
naturally produced on the GPU than as ordinary UI primitives.

Use the smallest rendering boundary that fits the content:

| Need | API |
| --- | --- |
| Text, fills, borders, images or vectors | normal `Element` primitives |
| New renderer-independent layout or paint behavior | `CustomElement` |
| Post-process an Argui layer | a registered custom effect |
| Encode application compute/render work into a retained texture | `Element::gpu_canvas` |

A canvas does not give the application a second window surface. Argui still
selects the backend and adapter, creates the device, acquires the surface,
submits command buffers in order and presents. The application receives a
borrowed encoder and an offscreen target view only while a dirty canvas is
rendered. Externally owned WGPU instances, devices, queues and surfaces are not
accepted by the high-level runtime.

## Mental model

The UI and paint layers stay renderer-independent:

```text
GpuCanvasSpec + retained NodeId
              |
              v
GpuCanvasPrimitive in the ordered display list
              |
              v
bounded retained sRGB texture ---- normal textured-quad composition
              |                    (transform, clip, radii, opacity, effects)
              v
application callback encodes only when revision or physical extent changes
```

The texture covers the visible local viewport. Ancestor transforms affect its
composition but do not silently increase its allocation. `resolution_scale` is
the explicit opt-in for lower-resolution or supersampled content.

## Register a renderer before launch

Every canvas is backed by an immutable startup registration. Cloning a
`GpuCanvasRegistration` preserves its process-local identity and factory.
Registry labels are stable diagnostic names; they are not cache keys.

```rust,ignore
use std::sync::{Arc, RwLock};
use argui::render::{
    GpuCanvasDeviceContext, GpuCanvasError, GpuCanvasFactory,
    GpuCanvasRegistration, GpuCanvasRegistry, GpuCanvasRenderContext,
    GpuCanvasRenderer, RendererConfig, wgpu,
};

#[derive(Default)]
struct Scene { revision: u64 }

struct Factory { scene: Arc<RwLock<Scene>> }

impl GpuCanvasFactory for Factory {
    fn create(
        &self,
        context: &GpuCanvasDeviceContext<'_>,
    ) -> Result<Box<dyn GpuCanvasRenderer>, GpuCanvasError> {
        let pipeline = create_pipeline(context.device(), context.target_format());
        Ok(Box::new(CanvasRenderer {
            scene: Arc::clone(&self.scene),
            pipeline,
        }))
    }
}

struct CanvasRenderer {
    scene: Arc<RwLock<Scene>>,
    pipeline: wgpu::RenderPipeline,
}

impl GpuCanvasRenderer for CanvasRenderer {
    fn render(
        &mut self,
        context: &mut GpuCanvasRenderContext<'_>,
    ) -> Result<(), GpuCanvasError> {
        // Copy application data, then release the lock before submission/present.
        let _revision = self.scene.read().map_err(|_| {
            GpuCanvasError::new("scene state is unavailable")
        })?.revision;

        let pipeline = &self.pipeline;
        let (encoder, target) = context.encoder_and_target();
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("document-canvas"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                // Argui already cleared dirty targets to transparent.
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        pass.set_pipeline(pipeline);
        pass.draw(0..3, 0..1);
        Ok(())
    }
}

let scene = Arc::new(RwLock::new(Scene::default()));
let registration = GpuCanvasRegistration::new(
    "editor.document",
    Factory { scene: Arc::clone(&scene) },
);
let canvas_id = registration.id();
let registry = GpuCanvasRegistry::new([registration])?;
let renderer = RendererConfig::default()
    .gpu_canvas_cache_bytes(128 * 1024 * 1024)
    .gpu_canvases(registry);
```

Use `argui::render::wgpu`, not a separately versioned direct WGPU dependency.
The re-export is the exact version used by the public contexts and descriptor
types.

Factories are lazy and surface-local. One registration can be used in several
windows, but every `SurfaceRenderer` creates its own `GpuCanvasRenderer` and
retained textures. A recreated surface keeps resources when the device and
format are unchanged; a new device creates factories and textures again.

## Place the retained element

Pass the registration ID and the scene's current pixel-content revision:

```rust,ignore
use argui::ui::{Element, GpuCanvasSpec};

let viewport = Element::gpu_canvas(
    GpuCanvasSpec::new(canvas_id)
        .content_revision(scene_revision)
        .resolution_scale(1.0),
)
.keyed("document-viewport")
.width(argui::ui::percent(1.0))
.height(argui::ui::percent(1.0));
```

It is an ordinary leaf. Width, height, transforms, opacity, rounded corners,
overflow clips, effect layers, hit testing, gestures, focus and semantics use
the same APIs as other elements. Later siblings and higher `z_index` content
compose above it. Canvas pixels do not generate semantics: give the leaf a
label and focus policy, and provide accessible Argui controls or semantic
overlays for important actions.

`CustomPaintContext::gpu_canvas(slot, bounds, spec)` is the advanced path for a
custom element that emits several primitives. The local `slot` distinguishes
canvases owned by the same retained node. Reusing a slot in one frame is a
recoverable descriptor conflict. The helper deliberately remains WGPU-free so
`argui-ui`, custom layout, tests and paint caching do not depend on renderer
handles or device lifetime.

## Context ownership and submission

`GpuCanvasDeviceContext` exposes the selected device, queue, enabled features,
effective limits, sRGB target format and opaque device generation. A renderer
may keep owned buffers, textures, bind groups and pipelines created from the
device.

`GpuCanvasRenderContext` additionally exposes the dedicated command encoder,
offscreen target view, exact physical extent, logical bounds, DPI scale,
requested resolution scale, registration ID, retained object/slot and frame
number.

The lifetime boundary is strict:

- Do not retain the encoder or target view after `render` returns.
- Do not call `Queue::submit`, acquire a surface, present, or block in device
  polling. Argui submits successful canvas command buffers before the
  compositor in one ordered submission.
- Queue `write_buffer` and `write_texture` calls are supported. Release model
  locks before returning so submission and presentation never wait on user
  synchronization.
- Use the supplied encoder for compute, render and copies. If `render` returns
  `GpuCanvasError`, that entire encoder is discarded and other canvases and UI
  still render.

Each dirty canvas gets a separately labelled encoder. Argui clears a newly
allocated or dirty target to transparent before calling the renderer.

On native targets a canvas renderer is `Send + Sync`, preserving the native
thread traits of `SurfaceRenderer`. Browser WGPU handles are thread-local, so
the same renderer trait has no thread bound on `wasm32`; application code and
method names are otherwise identical.

## Revisions, resize and animation

`content_revision` is application-owned. Increment it whenever data read by the
callback changes. Keep it fixed for an unchanged scene. Argui invokes the
callback when any pixel-producing input changes:

- no successful texture exists yet;
- `content_revision` changes;
- logical size, DPI scale or `resolution_scale` changes;
- output format or device changes.

Composition-only changes such as transform, clip, rounded corners and opacity
update the textured quad without rerunning the callback. Cached effect layers
are invalidated when canvas content or composition changes.

Animation remains model-driven. While running, request animation frames,
advance the scene and revision, then notify the model. While paused, request no
frames and keep the revision fixed. Unrelated Argui redraws then count as cache
hits and do not invoke custom GPU work. The [GPU Canvas Lab](../../app_examples/gpu-canvas/)
demonstrates this pattern.

## Texture size and memory

Physical size is:

```text
ceil(logical width  × window scale × resolution scale)
ceil(logical height × window scale × resolution scale)
```

Zero-sized canvases allocate and draw nothing. Non-finite or negative geometry,
unrepresentable sizes, dimensions above `max_texture_dimension_2d` and a single
texture above the budget produce a placeholder instead of an allocation.

The per-`SurfaceRenderer` cache has a 128 MiB default budget, configurable with
`RendererConfig::gpu_canvas_cache_bytes`. It pre-scans a frame, reserves a
deterministic display-order subset and never evicts a texture still needed in
that frame. Unreferenced entries become least-recently-used candidates. Cache
keys include registration, retained node identity and custom slot, so remounts
do not accidentally reuse another node's pixels.

`RenderProfile::gpu_canvases` reports retained entries and bytes, dirty renders,
cache hits, failures and callback CPU encode time. Inspector records, strict GPU
trace JSON and DevTools expose the same values. This is CPU timing, not a claim
about the application's custom GPU pass duration; use WGPU timestamp queries
when that measurement is required.

## Color, alpha and sampling

The target uses the renderer surface's sRGB view format and is sampled with
`ImageSampling::Linear` by default. `Nearest` is available for pixel art and
discrete data. Shader outputs follow Argui's primitive contract: straight
linear RGBA, with normal alpha blending into premultiplied linear intermediate
targets. Return transparent colors with RGB already describing their straight
color; do not pre-darken edges by premultiplying them in the shader.

Storage buffers and compute-intermediate textures may use other formats. The
final render pass must write the supplied target view using
`context.target_format()`. Pipelines are created per surface format, so do not
hard-code `Bgra8UnormSrgb` or `Rgba8UnormSrgb`.

## Capabilities and fallback

Factories that need more than the WebGPU baseline declare requirements before
device creation:

```rust,ignore
fn requirements(&self) -> GpuCanvasRequirements {
    GpuCanvasRequirements::default()
        .required_features(wgpu::Features::TEXTURE_COMPRESSION_BC)
        .optional_features(wgpu::Features::TIMESTAMP_QUERY)
        .required_limits(custom_limits)
        .reason("loads BC-compressed document tiles and optionally profiles them")
}
```

Required features are unioned. Optional features are enabled only when the
adapter supports them. Limits are merged using WGPU's direction-aware helpers,
including alignment limits where smaller values are better. Non-baseline
requirements must include a human-readable reason.

Adapter validation names the registration and missing feature or insufficient
limit. `SurfaceRenderer::new_with_device` rejects a shared device that did not
enable required capabilities. On Windows, required capabilities are evaluated
for every configured backend attempt and their errors remain in the existing
fallback report. Optional capabilities alone do not force fallback. The same
baseline API works with native WGPU and browser WebGPU; unsupported browser
capabilities produce an initialization error instead of a blank canvas.

## Errors and recovery

Invalid registrations fail when the registry is built. Frame-local problems —
a missing ID, invalid extent, budget pressure, factory error, callback error or
duplicate retained key — display a deterministic magenta checkerboard while
surrounding UI remains valid.

Drain structured state changes with
`SurfaceRenderer::take_gpu_canvas_diagnostics`. The high-level runtime does this
after main-window and native-popup renders and emits
`RuntimeEvent::GpuCanvasFailed` / `GpuCanvasRecovered`, or the corresponding
window-scoped variants. Identical failures are not repeated every frame. A
later successful revision emits one recovery event. Applications can use that
event to place a normal Argui error panel above the canvas.

Native popups localize canvas transforms and clips like images and vectors.
Each popup owns its surface-local renderer/cache and reports its diagnostic
through the runtime rather than silently dropping the canvas.

## Input, accessibility and testing

A GPU canvas introduces no private input system. Attach normal Argui pointer,
wheel, gesture, focus, keyboard and semantic handlers. Always provide keyboard
alternatives for visible navigation controls. Expose important scene actions
as Argui buttons or semantic overlays; pixels alone have no accessible meaning.

GPU-free UI/layout tests treat the element as an opaque leaf and can verify
layout, hit testing, focus, handlers and semantics without a renderer.
Downstream GPU-free harnesses, including `argui-testing` integrations, should
keep that boundary rather than execute WGPU callbacks. Renderer pixel,
blending, effect and callback-order correctness belongs in `argui-render` GPU
tests. On Linux, run graphical checks through
`scripts/linux-hidden-display.sh` and inspect saved captures.

## Acceptance measurements

The 0.3 implementation was exercised on Linux/Wayland with the Intel `i915`
WGPU adapter. These are single-run acceptance observations, not portable
benchmarks:

| Transition or output | Observed result |
| --- | --- |
| First 48×32 canvas creation and callback, debug GPU test | 7.182 ms CPU prepare/encode |
| Same canvas after a content-revision change, pipelines already warm | 0.517 ms CPU prepare/encode |
| Unchanged Argui redraw | callback count stayed at 1; one cache hit and zero dirty renders |
| Retained 48×32 BGRA target | 6,144 bytes, unchanged across the cache hit |
| Browser pause after pending frames settled | zero new `requestAnimationFrame` callbacks over 700 ms |
| Native release executable | 29,784,472 bytes |
| Optimized WebAssembly release module | 6,343,348 bytes |

The native hidden-display capture and browser captures covered initial render,
pan/zoom, the clipped effect layer and overlay, failure placeholder, recovery,
pause, resize and a 2× device scale. The exact timings and artifact sizes vary
by toolchain and target; the required invariants are the zero unchanged
callback, bounded cache bytes and stopped paused animation loop.

For a native and WebAssembly reference including compute, render, pan, zoom,
pause/resume, clipping, an effect, overlays and in-app recovery diagnostics,
see [GPU Canvas Lab](../../app_examples/gpu-canvas/README.md).
