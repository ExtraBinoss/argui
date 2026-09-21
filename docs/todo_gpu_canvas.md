# Custom WGPU canvas for Argui 0.3

Status: planned

Target release: `0.3.0`

This document is an implementation plan, not a list of optional ideas. Its goal
is to let an Argui application place a retained UI element whose pixels are
produced by application-owned WGPU render, compute and copy commands, while
Argui continues to own the window surface, frame submission and presentation.

An agent implementing this plan must continue through every phase and satisfy
the complete definition of done. A triangle on screen, a native-only prototype,
or a public `Device` getter is not completion. If an external dependency or
platform is unavailable, finish every unblocked item, record the exact blocker
and the command or external action required, and leave this plan in place.

When every checkbox is genuinely complete, migrate all enduring information to
the public documentation and changelog, then delete this file. The final commit
for this feature must not contain `docs/todo_gpu_canvas.md`.

## Required reading

Read these files completely before changing code:

- [`contributing/code-quality.md`](contributing/code-quality.md)
- [`contributing/linux-testing.md`](contributing/linux-testing.md)
- [`architecture.md`](architecture.md)
- [`rendering/primitives.md`](rendering/primitives.md)
- [`rendering/effects.md`](rendering/effects.md)
- [`ui/custom-elements.md`](ui/custom-elements.md)
- [`runtime/models.md`](runtime/models.md)
- [`contributing/devtools.md`](contributing/devtools.md)
- [`contributing/releases.md`](contributing/releases.md)

Also inspect the current implementations rather than relying on line numbers in
this plan. At minimum, inspect:

- `crates/argui-paint/src/display_list.rs` and the image/vector primitives;
- `crates/argui-ui/src/custom.rs` and `crates/argui-ui/src/element/`;
- `crates/argui-layout/src/paint.rs`, `custom.rs` and `surface.rs`;
- `crates/argui-render/src/config.rs`, `batch.rs`, `effect_graph.rs`,
  `offscreen.rs`, `surface.rs` and `surface/effects.rs`;
- `crates/argui-runtime/src/app/renderer.rs`, `app/popups.rs` and `event.rs`;
- `app_examples/Cargo.toml` and `app_examples/README.md`.

The repository may contain unrelated or concurrent 0.3 work. Preserve it. Do
not reset, replace or silently reformat files merely because this plan names
them.

## Problem statement

`CustomElement` currently supports retained layout, paint-command caching,
normal hit testing and accessibility. Its `CustomPaintContext` can emit
renderer-neutral primitives such as quads, but it intentionally has no active
GPU frame and therefore cannot expose:

- `wgpu::Device`;
- `wgpu::Queue`;
- `wgpu::CommandEncoder`;
- a target `wgpu::TextureView`;
- application-owned render or compute pipelines.

This prevents an application from building an editor-shaped interface with
Argui toolbars, panels, menus and overlays around a high-performance WGPU
document canvas. It also prevents scientific views, games, maps, node editors,
3D previews and data visualizations from sharing Argui's renderer without
creating a second native window or surface.

Putting raw WGPU handles directly in `CustomPaintContext` would be the wrong
fix. That context runs while the retained display list is being built and
cached; there is no acquired surface texture or command encoder at that point.
It also belongs to `argui-ui`, which must remain independent from WGPU.

## Scope and non-goals

### Required in 0.3.0

- A renderer-neutral GPU-canvas element and display command.
- A public registration API for application-owned canvas renderers.
- Read-only access to Argui's selected `wgpu::Device` and `wgpu::Queue` at the
  appropriate renderer lifecycle points.
- A borrowed `wgpu::CommandEncoder` and Argui-owned target
  `wgpu::TextureView` while a dirty canvas is rendered.
- Application-owned render, compute, copy, buffer, texture, bind-group and
  pipeline resources.
- Retained canvas textures that rerender only when their content revision or
  physical extent changes.
- Correct composition with normal Argui ordering, transforms, opacity, rounded
  corners, clips, effect layers and backdrop effects.
- Clear capability, registration, creation and render diagnostics.
- Correct multi-window, native-popup, renderer-fallback and WebAssembly
  behavior.
- A complete product-shaped example under `app_examples/gpu-canvas`.

### Explicitly out of scope

- Accepting an externally created `wgpu::Instance`, `Adapter`, `Device`,
  `Queue`, `Surface` or `SurfaceTexture` in the high-level runtime.
- Letting application code acquire, reconfigure, submit or present Argui's
  window surface.
- Replacing the renderer's normal quad, text, image or vector paths.
- A scene graph, shader DSL, material system, ECS or game engine.
- Automatically deriving content revisions by observing arbitrary user GPU
  resources.
- A permanent animation loop for every canvas.
- Direct rendering into the swapchain as the primary integration path.

External WGPU ownership can be planned separately. It requires a distinct
contract for surface lifetime, device-feature negotiation, fallback backends,
desktop backdrop compatibility and presentation. It must not be smuggled into
this feature through a public raw-surface escape hatch.

## Locked architectural decisions

These decisions are part of the required behavior. Private names and file
splits may change when the current code demands it, but an implementation must
not change these contracts without first updating this plan and explaining why.

### 1. Keep UI and paint renderer-independent

`argui-ui`, `argui-layout` and `argui-paint` must not depend on `wgpu` or
`argui-render`. They carry only opaque identities, bounds, revisions, sampling
options, transforms and clips.

`CustomPaintContext` may gain a helper that emits a GPU-canvas display command,
but it must not receive a `Device`, `Queue`, encoder, render pass, texture or
surface. Raw WGPU access belongs to new public contexts in `argui-render`.

### 2. Render offscreen, then compose

Each visible canvas renders into a retained Argui-owned offscreen texture. Argui
then samples that texture at the canvas command's location just as it composes
another textured primitive.

This is required because direct swapchain rendering cannot reliably preserve:

- drawing before and after the canvas;
- ancestor transforms and clip chains;
- rounded corners and opacity;
- nested filters, shadows, blend modes and backdrop filters;
- native popup localization;
- cached effect layers;
- a single owner for surface acquisition and presentation.

The target texture uses the active renderer target format and at least
`RENDER_ATTACHMENT | TEXTURE_BINDING | COPY_SRC | COPY_DST`. It is not promised
to be a storage texture. Compute work may write application-owned buffers or
textures and then render or copy its result into the supplied target.

### 3. Argui owns submission and presentation

The canvas receives a mutable borrowed command encoder. Successful canvas
encoders are finished by Argui and submitted, in dependency order, in the same
queue submission as the compositor frame. The application never receives the
surface or `SurfaceTexture` and never calls `present`.

The raw queue is exposed so an application can use `write_buffer`,
`write_texture` and ordinary WGPU resource APIs. The public safety contract must
state that calling `Queue::submit`, blocking `Device::poll`, or retaining the
borrowed encoder/target view is unsupported. All render, compute and copy work
for the canvas frame must be encoded through the supplied encoder.

Use a dedicated encoder per dirty canvas. If a canvas explicitly returns an
error, discard that encoder so partially recorded commands do not contaminate
the main Argui encoder.

### 4. Registration is explicit and known before device creation

Canvas factories are registered through an immutable `GpuCanvasRegistry` on
`RendererConfig`, following the existing custom-effect registry pattern. The
runtime does not discover WGPU callbacks from an `Element` or store closures in
the retained UI tree.

Knowing registrations before `request_device` lets Argui aggregate required
features and limits, gives every Windows fallback attempt the same requirements,
and lets `new_with_device` reject an incompatible shared device with a clear
error.

The registry is fixed for a renderer device in 0.3. Applications may change
scene data and content revisions at runtime, but adding a factory that requires
new device capabilities requires creating a new renderer. This also avoids
keeping a dynamically replaced trait-object vtable alive past the code that created it.

### 5. Retained revisions control GPU work

Every canvas display primitive carries an application-authored `content_revision`.
The renderer reruns the application callback only when at least one of these
inputs changes:

- registration or retained instance identity;
- content revision;
- exact physical viewport extent;
- output format or device generation;
- an option that changes the rendered texture itself.

Moving the element, changing an ancestor clip, opacity or rounded corners must
update composition without rerendering identical canvas pixels. A resize or DPI
change must recreate the target and rerender. A zero-sized or fully invalid
extent must safely skip work.

Application code is responsible for incrementing the revision after changing
shared scene data. Animation uses the existing `request_animation_frame` /
`wants_animation_frame` mechanisms and stops requesting frames when paused.
There must be no hidden polling thread or permanent redraw loop.

### 6. Canvas instances follow retained node identity

A registration identity selects the factory. A separate instance key, derived
from the retained UI node plus an optional local slot, selects the cached output
texture. This permits the same registered renderer to appear more than once
without cache collisions and lets a custom element emit more than one canvas.

Do not allocate a fresh instance identity every time `Render::render` rebuilds
an `Element`; that would defeat retained caching. Remounting a node may create a
new identity and must release the old cache entry through normal LRU cleanup.

### 7. Fail locally when possible

Unsupported required device capabilities are renderer-initialization errors,
because the requested device cannot satisfy the declared contract. Missing
registrations, factory creation errors and explicit per-frame callback errors
must instead produce:

- a deterministic visible placeholder for the affected canvas;
- a structured renderer diagnostic;
- a window-scoped runtime event with the canvas label, instance and message;
- one console-readable English message, deduplicated until the error changes or
  the canvas recovers.

The rest of the Argui UI must continue rendering when safe. Actual device loss,
surface validation failure or an uncaptured WGPU error may still be fatal under
the renderer's existing policy. Do not promise that arbitrary invalid WGPU API
usage can be recovered.

### 8. WebAssembly is a first-class target

The API must exist without `cfg` differences on native and
`wasm32-unknown-unknown`. Use WGPU/WebGPU capabilities and WGSL; do not expose
native handles. The example must compile and run in a WebGPU browser.

If a particular optional compute feature is unavailable, the registry's
requirements must report that clearly or the example must take a documented
baseline render-only path. A failed local browser run is not justification for
making the public feature native-only.

## Target architecture

```text
Render::render
    |
    v
Element::gpu_canvas(GpuCanvasSpec)
    |
    v
renderer-neutral DisplayCommand::GpuCanvas
    |
    +--> unchanged texture? ------> reuse retained texture
    |
    +--> dirty texture
            |
            v
      GpuCanvasRenderer::render
      Device + Queue + CommandEncoder + TextureView
            |
            v
      Argui finishes encoder
    |
    v
textured-quad composition in normal display-list order
    |
    v
clips / rounded corners / effects / backdrop / surface present
```

The registry/factory boundary is separate from the display list:

```text
RendererConfig
    `-- GpuCanvasRegistry
          `-- GpuCanvasRegistration { id, label, factory, requirements }

Display list
    `-- GpuCanvasPrimitive { id, retained instance, revision, geometry }
```

## Proposed public API shape

The final names may be adjusted once all existing public naming conventions are
checked, but the responsibilities and ownership in this section are required.

### Renderer-neutral types

Add a small module in `argui-paint` with types equivalent to:

```rust,ignore
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GpuCanvasId(u64);

#[derive(Clone, Debug, PartialEq)]
pub struct GpuCanvasPrimitive {
    pub canvas: GpuCanvasId,
    pub object: RenderObjectId,
    pub slot: u32,
    pub bounds: Rect,
    pub content_revision: u64,
    pub resolution_scale: f32,
    pub sampling: ImageSampling,
    pub opacity: f32,
    pub radii: CornerRadii,
    pub transform: Affine2D,
    pub clips: ClipChain,
}
```

`GpuCanvasId` is process-local and opaque, like image/vector identities. The
registration constructor normally allocates it. All floating-point options
must reject or normalize non-finite, negative and unreasonable values before
they reach texture allocation.

Add `DisplayCommand::GpuCanvas`, `DisplayList::push_gpu_canvas`, validation,
storage accounting and exhaustive-match handling. Display-list equality must
include every field that affects texture generation or composition.

### UI types

Provide a concise common path equivalent to:

```rust,ignore
Element::gpu_canvas(
    GpuCanvasSpec::new(self.canvas_id)
        .content_revision(self.scene_revision)
        .sampling(ImageSampling::Linear)
        .resolution_scale(1.0),
)
```

`GpuCanvasSpec` is WGPU-independent. The element behaves as a normal leaf in
layout and can use all existing style, transform, effect, semantics, key,
interaction, gesture and event APIs. Its normal paint opacity and corner radii
apply during composition.

Also add a renderer-neutral `CustomPaintContext::gpu_canvas` helper. It takes a
local slot, local bounds and a spec, then emits the same primitive with the
custom node's retained object identity, ancestor transform and clips. This is
the advanced path for custom elements that combine several display primitives.
It still exposes no WGPU handle.

### Registry and callbacks

Expose the exact WGPU version used by Argui as `argui::render::wgpu`. Public
examples and documentation must use this re-export so user code cannot
accidentally compile descriptors against an incompatible WGPU version.

Provide public types equivalent to:

```rust,ignore
pub trait GpuCanvasFactory: Send + Sync + 'static {
    fn requirements(&self) -> GpuCanvasRequirements {
        GpuCanvasRequirements::default()
    }

    fn create(
        &self,
        context: &GpuCanvasDeviceContext<'_>,
    ) -> Result<Box<dyn GpuCanvasRenderer + Send>, GpuCanvasError>;
}

pub trait GpuCanvasRenderer: Send + 'static {
    fn render(
        &mut self,
        context: &mut GpuCanvasRenderContext<'_>,
    ) -> Result<(), GpuCanvasError>;
}
```

Preserve the useful `Send`/`Sync` auto-traits of `RendererConfig` and
`SurfaceRenderer`. If current platform constraints require different bounds,
prove that with native and Wasm compile assertions before weakening them; do not
add or remove thread bounds accidentally.

`GpuCanvasDeviceContext` provides getters for:

- `&wgpu::Device`;
- `&wgpu::Queue`;
- the enabled feature set and effective limits;
- the target texture format;
- a stable opaque device generation/identity for diagnostics.

`GpuCanvasRenderContext` provides getters for:

- `&wgpu::Device` and `&wgpu::Queue`;
- `&mut wgpu::CommandEncoder`;
- `&wgpu::TextureView` for the canvas target;
- target format and exact physical extent;
- logical bounds, scale factor and requested resolution scale;
- canvas registration ID, retained object/slot and frame number.

The contexts have private fields so application code cannot construct an
invalid one. Their Rustdoc must say which references may be retained (device
resources) and which must not be retained (encoder and target view).

`GpuCanvasRegistration::new(label, factory)` allocates an ID and stores the
factory behind an `Arc`. Cloning a registration preserves its ID and factory.
`GpuCanvasRegistry::new` validates registrations, deduplicates repeated clones
and reports inconsistent duplicates. Labels are stable, non-empty diagnostic
names but are not cache identities.

The intended launch shape is:

```rust,ignore
let scene = Arc::new(RwLock::new(Scene::default()));
let canvas = GpuCanvasRegistration::new(
    "example.document-canvas",
    DocumentCanvasFactory::new(Arc::clone(&scene)),
);
let canvas_id = canvas.id();

let renderer = RendererConfig::default().gpu_canvases(
    GpuCanvasRegistry::new([canvas])?,
);
let model = Editor::new(canvas_id, scene);

run_application(config, renderer, model, on_event)?;
```

Do not add a Cargo feature for this API: Argui already has an unconditional
renderer/WGPU dependency, and a feature flag would only multiply build variants
without removing that dependency.

### Requirements and device negotiation

`GpuCanvasRequirements` must support:

- required WGPU features;
- optional WGPU features, enabled only when the chosen adapter supports them;
- required WGPU limits;
- a human-readable reason attached to non-baseline requirements.

Aggregate all registrations before requesting the device. Required features
are unioned, optional features are intersected with adapter support, and limits
are merged using WGPU's own resolution/alignment helpers rather than assuming
that every field uses `max`. Validate requirements against the adapter first so
errors name the canvas registration, missing feature or insufficient limit.

For `SurfaceRenderer::new_with_device`, compare the registry requirements with
the already enabled device features and limits. Never silently drop a required
capability. Optional features must not by themselves force a Windows backend
fallback; required features may, and every failed attempt must retain its clear
canvas-specific reason in `RendererAttemptFailure`.

The example must require only WebGPU-baseline capabilities. Its compute pass
should use storage buffers and ordinary compute dispatch rather than an exotic
native-only feature.

## Texture, cache and color contract

Add a dedicated canvas cache/manager in `argui-render`; do not overload the
transient effect `TexturePool` with persistent canvas ownership.

Each cache entry records at least:

- registration and retained instance/slot key;
- target texture, view and compositor bind groups;
- exact physical extent and byte size;
- last rendered content revision;
- output format/device generation;
- last-used frame for LRU eviction;
- creation/render failure state and diagnostic deduplication state.

Required rules:

1. The texture covers the visible local canvas viewport, not the logical size
   of an infinite document.
2. Physical extent is `ceil(logical_size * scale_factor * resolution_scale)`,
   checked for finite values, overflow, the adapter's
   `max_texture_dimension_2d` and the configured byte budget.
3. Default `resolution_scale` is `1.0`. Ancestor transforms do not silently
   allocate giant textures; an app that needs supersampling opts in explicitly.
4. A dirty target is cleared deterministically before the callback. The
   initial 0.3 contract is a transparent clear followed by user commands; do
   not expose an uninitialized `Load` path.
5. Canvas output uses the renderer target's sRGB view and straight-alpha
   composition, matching Argui's current image pipeline. Document this with a
   shader example and an alpha-edge regression test.
6. Cache entries are exact-sized or correctly limit UVs to the requested
   extent. Never display uninitialized padding.
7. The budget is configurable through `RendererConfig` and has a documented
   bounded default. A single oversized request fails before allocation.
8. Pre-scan canvases needed by the frame. Never evict a texture that is still
   required later in that same frame. If all visible canvases cannot fit, choose
   a deterministic subset and show placeholders for the rest.
9. Entries not referenced by the current frame become LRU candidates. Dropped
   registrations, remounted nodes and resized targets release GPU resources.
10. A surface resize does not flush unrelated canvas pipelines. A new device or
    output format does recreate the relevant renderer and textures.

Add public profile data for canvas entries, bytes, dirty renders, cache hits,
failed canvases and encoded CPU time. Include it in runtime inspector records
and DevTools without pretending to measure custom GPU pass duration. Application
code can create its own timestamp queries when it needs pass-level GPU timings.

## Frame integration details

The implementation order inside one UI frame must be:

1. Acquire a presentable surface frame. If it is occluded, timed out or skipped,
   do not run custom canvas work.
2. Prepare normal Argui primitives and scan canvas commands.
3. Validate registrations, extents and visible-frame memory demand.
4. Create factories lazily for canvases first used by this `SurfaceRenderer`.
5. For each dirty canvas, create a dedicated labeled encoder, clear its target,
   invoke the callback and keep its command buffer only on success.
6. Prepare the canvas compositor instances/bind groups.
7. Build ordinary draw batches and the effect graph, including canvas draws.
8. Encode the main/effect compositor after all canvas producers.
9. Submit successful canvas command buffers followed by the Argui command
   buffer in one ordered queue submission.
10. Poll and present using the existing renderer policy.

`DrawKind` must gain a canvas variant that preserves display-list order. Do not
merge batches that sample different retained textures. Both the direct surface
path and `draw_offscreen` effect path must support the new kind.

Canvas texture and compositor changes must advance the renderer's content
revision so cached effect layers invalidate correctly. A canvas nested inside a
blur, opacity layer, blend mode or backdrop filter must never leave the parent
layer stale after its revision changes.

The compositor must apply:

- display-list bounds and affine transform;
- clip chains;
- element corner radii;
- element opacity;
- nearest or linear sampling;
- normal alpha blending;
- the correct target-region offset when drawing inside an offscreen effect
  layer.

Prefer extracting a small shared textured-quad compositor from the image path
if that removes meaningful duplication. Do not couple canvas texture ownership
to CPU `ImageAsset` upload or regress existing image caching.

## Errors and diagnostics

Add structured errors for at least:

- duplicate or invalid registration;
- missing canvas registration referenced by a display list;
- unsupported required feature or limit, including the registration label;
- incompatible shared `RendererDevice`;
- invalid/non-finite resolution scale or extent;
- extent beyond adapter limits;
- canvas cache budget exhaustion;
- factory creation failure;
- explicit canvas render failure;
- conflicting descriptors for the same retained key in one frame.

Expose renderer diagnostics through a draining method such as
`SurfaceRenderer::take_gpu_canvas_diagnostics`. The runtime drains it after the
main surface and native popup render paths and emits a window-scoped event such
as `RuntimeEvent::GpuCanvasFailed` / `WindowRuntimeEvent::GpuCanvasFailed`.
Recovery after a later successful revision should emit one corresponding
recovery event, not a failure every frame.

All user-facing messages are concise English and include the canvas label,
stage and actionable cause, for example:

```text
GPU canvas 'example.document-canvas' could not be created: required feature
TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES is unavailable on adapter '...'.
```

The default placeholder must be deterministic and visible in release builds,
without requiring text shaping. Document how an application can place its own
Argui error overlay using the runtime event. Do not print the same diagnostic
on every animation frame.

## Multi-window, popups and fallback behavior

- Every `SurfaceRenderer` gets its own canvas renderer instances and retained
  target textures, even when windows share a `RendererDevice`.
- The immutable registry and requirements are cloned with `RendererConfig` for
  secondary windows and native popups.
- `NativeSurfacePaint::localize` must translate canvas transforms and clips in
  the same way as images/vectors/text.
- A canvas inside a native popup must render or produce a scoped popup
  diagnostic; it must not silently disappear.
- `recreate_surface` retains canvas resources when the device/format remains
  valid. Creating a renderer with a new device invokes factories again.
- Canvas support must not disable Windows DirectComposition or desktop
  backdrops. Baseline registrations should work on the preferred DX12 path.
- Renderer fallback remains controlled by the existing default-on
  `renderer_fallback` setting. Required canvas capabilities are evaluated on
  every attempted backend and included in the existing clear fallback report.
- Do not expose the selected backend as behavior that application rendering
  must branch on. Branch only on declared WGPU capabilities.

## Interaction and accessibility

A GPU canvas does not introduce a second input system. It uses normal Argui:

- hit testing, gestures and pointer capture;
- direct handlers and advanced event bubbling;
- focus and keyboard shortcuts;
- semantics, labels and accessible actions;
- overlays rendered before or after it in the normal tree.

Canvas pixels have no automatic semantic meaning. The public guide and example
must show a labelled/focusable canvas plus accessible Argui controls or a
semantic overlay for important actions. Keyboard alternatives must exist for
pan/zoom/reset and animation controls shown in the example.

`argui-testing` remains GPU-free. It should be able to lay out, hit-test,
interact with and inspect a canvas element as an opaque leaf, but it does not
execute WGPU callbacks. Pixel correctness belongs in `argui-render` tests and
the native/browser integration checks.

## Required example: GPU Canvas Lab

Add a new non-publishable crate at `app_examples/gpu-canvas` and register it in
the separate `app_examples` workspace. It must be a product-shaped example,
not a single full-window triangle.

The application contains:

- an Argui top toolbar with play/pause, reset and zoom controls;
- an Argui side inspector showing canvas state and renderer capability/status;
- a central `Element::gpu_canvas` that fills the remaining space;
- an Argui overlay above the canvas proving ordering and clipping;
- pointer drag to pan, wheel/pinch or controls to zoom, and keyboard equivalents;
- a WGPU compute pass that updates a bounded set of particles or scene objects;
- a WGPU render pass that draws a grid/background and the computed objects;
- resize and HiDPI correctness;
- pause behavior that stops animation requests and custom GPU rerenders;
- an in-app diagnostic panel driven by GPU-canvas runtime events.

Keep the workload bounded and laptop-friendly. The example should visibly prove
that Argui remains responsive around the WGPU view, not serve as a benchmark
that pins the GPU.

Use shared application data with explicit synchronization between the model and
factory, and explain the pattern in the example README. Every scene mutation
increments a revision that is passed to `GpuCanvasSpec`. The renderer reads a
short-lived snapshot; it must not hold a lock while WGPU submits or presents.

The crate must support:

```sh
cargo run --release --manifest-path app_examples/Cargo.toml \
  -p argui-example-gpu-canvas

cargo check --manifest-path app_examples/Cargo.toml \
  -p argui-example-gpu-canvas --target wasm32-unknown-unknown
```

Provide a `cdylib` entry point and minimal Web page/build instructions so the
same example can be exercised in a WebGPU browser. Use the repository's pinned
Wasm tooling and do not invent a second JavaScript framework.

## Documentation deliverables

Create a permanent `docs/rendering/gpu-canvas.md` guide containing:

1. when to use normal Argui primitives, a custom element, a custom effect or a
   GPU canvas;
2. the offscreen/composition mental model;
3. complete registration, factory and element code;
4. device/queue/encoder/target lifetimes and the no-submit rule;
5. revisions, caching, resizing, DPI and animation scheduling;
6. color/alpha, texture format and compute-intermediate rules;
7. requirements, limits, renderer fallback and Wasm capability handling;
8. input/accessibility integration;
9. errors and runtime diagnostics;
10. multi-window and native-popup behavior;
11. the distinction from externally owned WGPU instances/surfaces.

Update `docs/rendering/primitives.md`, `docs/architecture.md`, `docs/README.md`,
the repository README feature list, the roadmap and `CHANGELOG.md`. Add any
0.3-specific API note to `docs/migrations/0.3.md` only if users must change
existing code; do not manufacture a migration for an additive feature.

Do not add this temporary plan to the published documentation navigation.

### Required Unreleased changelog entry

The changelog is not a one-line checkbox. Before this plan is deleted, add a
reviewed entry under `## [Unreleased]` that tells a 0.3 user exactly what was
added. It must cover, in plain release-note language:

- the new retained GPU-canvas element and its intended editor, visualization,
  game/map and scientific-view use cases;
- the new public registration/factory/render-context API and access to Argui's
  selected WGPU device, queue, command encoder and offscreen target view;
- the ownership boundary: Argui still owns backend selection, surface
  acquisition, command submission and presentation;
- retained revision/resize caching, bounded texture memory and how paused or
  unchanged canvases avoid custom GPU work;
- correct composition with Argui primitives, transforms, clipping, rounded
  corners, opacity, effects, overlays, multi-window surfaces and native popups;
- required/optional GPU capability negotiation, Windows renderer fallback and
  clear recoverable runtime diagnostics/placeholders;
- native and WebAssembly/WebGPU support;
- the new **GPU Canvas Lab** in `app_examples/gpu-canvas`, including that it
  demonstrates an Argui application shell, a custom WGPU compute pass, a custom
  render pass, pan/zoom, pause/resume, overlays, resize/HiDPI and in-app errors;
- the deliberate limitation that externally owned WGPU instances, devices and
  surfaces are still not accepted by the high-level runtime;
- any narrower limitation discovered during implementation, with no vague
  promise that unsupported behavior works.

Use the normal changelog categories accurately. New API/example work belongs
under `### Added`; behavior changed for existing public APIs belongs under
`### Changed`; actual constraints belong under `### Known limitations`. Do not
describe implementation details as user-facing features, and do not claim Wasm,
compute, effects, native popups or fallback support until their acceptance tests
are present.

Before deleting this plan, compare the changelog entry against the final public
API, the GPU Canvas Lab README and `docs/rendering/gpu-canvas.md`. Every public
name and command must match the shipped code. The implementation commit must
contain the changelog update and the example; deleting the plan without them is
explicitly incomplete.

## File and dependency boundaries

The expected implementation areas are:

| Area | Expected responsibility |
| --- | --- |
| `argui-paint` | Opaque ID, primitive and display command only |
| `argui-ui` | `GpuCanvasSpec`, leaf constructor and custom-paint helper |
| `argui-layout` | Retained instance identity, paint lowering and popup localization |
| `argui-render` | WGPU traits/contexts, registry, requirements, cache, callback execution and composition |
| `argui-runtime` | Diagnostic draining and window-scoped events |
| `argui-inspect` / DevTools | Canvas memory, cache and failure metrics |
| `argui-testing` | GPU-free opaque-leaf interaction/inspection coverage |
| `argui` facade | Existing module re-exports; no large prelude expansion |
| `app_examples/gpu-canvas` | Native and Wasm reference application |

Suggested renderer files are `gpu_canvas/mod.rs`, `registry.rs`, `cache.rs` and
`pipeline.rs`, split by responsibility. Follow the repository rule that every
Rust file stays at or below 600 physical lines. Do not create an independent
public crate unless implementation proves a real dependency cycle that cannot
be solved inside these boundaries.

`wgpu` remains a dependency only of `argui-render` in this path. User code uses
the renderer's re-export. Add no third-party synchronization, shader or math
dependency unless the directly affected crate genuinely needs it and existing
workspace facilities cannot solve the problem.

## Test matrix

### Pure and retained tests

- ID allocation and registration clone identity.
- Empty/duplicate/invalid registry labels and duplicate handling.
- Required/optional feature aggregation and limit resolution, including
  alignment limits whose direction differs from capacity limits.
- `new_with_device` compatibility checks.
- Display-list append, clear, extend, equality, validation and storage metrics.
- UI reconciliation: same node/revision retains identity; revision changes are
  paint-only; remount creates a new instance.
- Custom-paint local slot behavior and duplicate-slot diagnostics.
- Layout bounds, transforms, clips, opacity, radii and popup localization.
- Zero, non-finite, overflow, excessive-dimension and over-budget extents.
- Cache hit, dirty revision, resize, DPI change, LRU eviction and same-frame
  retention decision logic without requiring a physical GPU.
- Diagnostic deduplication and recovery.
- `argui-testing` hit testing, focus and handlers on an opaque canvas leaf.

### GPU renderer tests

- A headless offscreen test renders a known solid/gradient through the custom
  callback and reads pixels back.
- A compute-to-buffer then render-to-target test proves arbitrary compute and
  render commands share the supplied encoder correctly.
- Quads before and after a canvas preserve exact ordering.
- Linear and nearest sampling differ as expected.
- Straight-alpha edges composite without dark or light fringes.
- Ancestor clipping, rounded corners, opacity and affine transforms match
  normal image behavior.
- A canvas inside a cached effect layer updates when its content revision
  changes and remains cached when it does not.
- Explicit callback failure discards its command buffer, shows the placeholder
  and leaves surrounding UI valid.
- Multiple canvases, repeated registrations, multiple instances and conflicting
  same-instance descriptors behave deterministically.
- Resize, surface recreation and a second `SurfaceRenderer` sharing a device do
  not reuse invalid target resources.
- Profiling/cache counters report hits, renders, bytes and failures accurately.

GPU-unavailable test environments may skip only tests that truly require an
adapter. Keep cache, negotiation and display-graph logic separately testable so
coverage does not depend on a desktop GPU.

### Native integration

On Linux, run GUI checks only through the private display wrapper and inspect
saved captures; a blank capture is a failure. Validate:

- the GPU Canvas Lab initial frame;
- pan, zoom, pause/resume and resize;
- a clipped/effected canvas and Argui overlay ordering;
- idle behavior after pause, including a stable callback counter and no
  continuous redraw;
- the diagnostic placeholder path;
- a second window or native popup containing the canvas when that feature is
  available.

Windows CI must cover DX12 with fallback enabled and disabled. The canvas must
not change desktop-backdrop behavior unless its declared required capabilities
force another backend, in which case the existing fallback event explains it.

### WebAssembly/browser integration

- `cargo check --workspace --all-targets --target wasm32-unknown-unknown` passes.
- The `app_examples/gpu-canvas` crate checks for Wasm.
- The release Wasm build completes with the repository's pinned tooling.
- A browser test waits for `RendererReady`, captures a non-blank canvas, pauses
  animation, verifies the render callback count stops increasing, resizes the
  browser and resumes.
- Missing WebGPU or missing required capabilities produces the documented
  English diagnostic rather than a blank page or panic.

### Performance evidence

Record before/after measurements for:

- idle paused example CPU/GPU activity;
- callback count across unchanged Argui redraws;
- first canvas creation time;
- steady animated frame CPU encode time;
- canvas texture bytes and cache hits;
- native and Wasm release artifact size.

The acceptance requirement is behavioral, not a fabricated percentage: an
unchanged canvas callback is not invoked, a paused example does not request
continuous frames, memory stays within its configured budget, and normal UI
interaction remains smooth at the example's bounded workload.

## Implementation checklist

A box is checked only after the implementation, tests and relevant public
documentation exist. Do not check an item because a stub compiles.

### Phase 0 — baseline and contract

- [ ] Read every required document and current implementation named above.
- [ ] Record the current branch, dirty files and protected-branch CI status;
      preserve unrelated 0.3 work.
- [ ] Record targeted baseline test counts and renderer/UI/runtime coverage.
- [ ] Confirm the current WGPU target formats and feature/limit APIs on native
      and Wasm with a throwaway experiment that is not committed.
- [ ] Confirm and document the public auto-traits that `RendererConfig` and
      `SurfaceRenderer` currently provide.
- [ ] Freeze the final public names and add one compile-only API example that
      matches the ownership contract in this plan.

### Phase 1 — renderer-neutral canvas primitive

- [ ] Add and document `GpuCanvasId` and `GpuCanvasPrimitive` in `argui-paint`.
- [ ] Add `DisplayCommand::GpuCanvas` and all `DisplayList` operations/metrics.
- [ ] Add and document `GpuCanvasSpec` and `Element::gpu_canvas` in `argui-ui`.
- [ ] Add the renderer-neutral `CustomPaintContext::gpu_canvas` advanced helper.
- [ ] Derive a stable instance identity from the retained node plus local slot.
- [ ] Lower the element/custom helper through `argui-layout` with normal
      transform, clip, opacity, radii and sampling fields.
- [ ] Localize canvas commands for native popup surfaces.
- [ ] Audit every exhaustive `ElementKind` and `DisplayCommand` match with `rg`.
- [ ] Add pure paint, UI, layout, surface-localization and reconciliation tests.
- [ ] Re-export the neutral types from the expected modules/facade.

### Phase 2 — registry and device negotiation

- [ ] Re-export Argui's exact WGPU version from `argui-render`.
- [ ] Add and document factory/renderer traits and error/result types.
- [ ] Add private-field device and render contexts with every required getter.
- [ ] Add cloneable registrations with stable IDs and diagnostic labels.
- [ ] Add immutable validated `GpuCanvasRegistry`.
- [ ] Add required/optional feature and required-limit declarations.
- [ ] Aggregate and validate requirements before every device request.
- [ ] Validate registry requirements in `new_with_device`.
- [ ] Preserve Windows fallback attempts and clear per-attempt errors.
- [ ] Add `RendererConfig` registry and canvas-cache-budget builders/defaults.
- [ ] Preserve required `Clone`, `Debug`, `Send` and `Sync` behavior with tests.
- [ ] Add registry, requirement, fallback-policy and shared-device tests.
- [ ] Confirm the public API compiles unchanged on Wasm.

### Phase 3 — retained targets and callback execution

- [ ] Add a persistent, bounded canvas texture cache separate from effect pools.
- [ ] Pre-scan visible canvases and reserve a deterministic within-budget set.
- [ ] Implement exact physical extent calculation and every validation guard.
- [ ] Create factories lazily per `SurfaceRenderer` and output format.
- [ ] Clear new/dirty targets to a defined transparent value.
- [ ] Invoke callbacks only for dirty revision/size/format/device inputs.
- [ ] Give each dirty callback a separate labelled command encoder.
- [ ] Discard failed callback encoders and retain successful command buffers.
- [ ] Keep user locks out of queue submission/presentation in the example/API docs.
- [ ] Implement LRU cleanup without evicting textures needed in the same frame.
- [ ] Implement deterministic placeholders and structured diagnostics.
- [ ] Test zero size, invalid size, resize, DPI, revision, errors, recovery,
      eviction and budget exhaustion.

### Phase 4 — composition and effects

- [ ] Add a canvas textured-quad compositor or safely share image compositor code.
- [ ] Add canvas draw kinds/batches without merging different textures.
- [ ] Prepare compositor geometry, clips and bind groups incrementally.
- [ ] Draw canvases in direct surface order with normal primitives.
- [ ] Draw canvases inside offscreen effect nodes.
- [ ] Advance content revisions for texture and composition changes.
- [ ] Preserve cached effect correctness across canvas revision changes.
- [ ] Implement straight-alpha, sRGB, sampling and target-offset contracts.
- [ ] Submit canvas producers before the compositor in one Argui-owned submit.
- [ ] Add pixel/readback, ordering, clipping, transform, alpha and effect tests.
- [ ] Verify existing image/vector/text/quad tests remain unchanged and green.

### Phase 5 — runtime, windows and observability

- [ ] Add renderer diagnostic draining and deduplication/recovery semantics.
- [ ] Add main/window-scoped runtime canvas failure and recovery events.
- [ ] Drain events after both main-window and native-popup renders.
- [ ] Keep registry/factory behavior correct across multi-window shared devices.
- [ ] Verify surface recreation retains valid canvas resources.
- [ ] Verify a new device recreates factory and texture resources.
- [ ] Add canvas statistics to `RenderProfile`, `argui-inspect` and DevTools.
- [ ] Keep runtime failures actionable without exiting for recoverable canvas errors.
- [ ] Test main window, secondary window, popup, fallback and profiling paths.
- [ ] Audit dynamic-registry lifetime safety; keep the 0.3 registry startup-only.

### Phase 6 — native and Wasm GPU Canvas Lab

- [ ] Create `app_examples/gpu-canvas` with native binary and Wasm library entry.
- [ ] Add the crate to `app_examples/Cargo.toml` and its README catalogue.
- [ ] Build the Argui toolbar, inspector, central canvas and overlay.
- [ ] Implement bounded WGSL compute and render pipelines using the WGPU re-export.
- [ ] Implement pan, zoom, reset, pause/resume and keyboard alternatives.
- [ ] Wire shared scene snapshots and explicit content revisions correctly.
- [ ] Stop all animation requests and callback invocations while paused.
- [ ] Display capability and failure events in-app and in clear English logs.
- [ ] Handle resize, HiDPI, clipping and an Argui overlay above the canvas.
- [ ] Document native and browser run commands in the example README.
- [ ] Add behavior tests that do not require pixel execution where possible.
- [ ] Add native hidden-display and browser capture tests for real rendering.

### Phase 7 — public documentation

- [ ] Write `docs/rendering/gpu-canvas.md` with every listed topic.
- [ ] Document why `CustomPaintContext` remains WGPU-independent.
- [ ] Document queue/encoder/view lifetime and submission restrictions.
- [ ] Document revision, cache, memory and animation behavior.
- [ ] Document feature/limit negotiation, fallback and WebAssembly behavior.
- [ ] Document interaction/accessibility and `argui-testing` boundaries.
- [ ] Link the GPU Canvas Lab from the guide and application catalogue.
- [ ] Update rendering primitives, architecture, docs index, README and roadmap.
- [ ] Add the full `CHANGELOG.md` Unreleased entry required above, including
      the API, retained behavior, WGPU ownership boundary, native/Wasm support,
      diagnostics, GPU Canvas Lab and known external-WGPU limitation.
- [ ] Cross-check every changelog name/claim/command against the final code,
      permanent guide, example README and acceptance-test evidence.
- [ ] Compile every Rust snippet intended to compile.

### Phase 8 — verification and 0.3 release readiness

- [ ] Run targeted `cargo nextest run --all-features` for every affected crate,
      keeping feature flags identical between repeated targeted runs.
- [ ] Run targeted Clippy, rustdoc and native/Wasm checks.
- [ ] Run the renderer GPU integration tests on the hidden Linux display.
- [ ] Inspect every saved native/browser capture and reject blank captures.
- [ ] Run the GPU Canvas Lab in native release mode and a WebGPU browser.
- [ ] Verify paused idle behavior, retained callback counts and memory budgets.
- [ ] Verify Windows/macOS/mobile compile jobs and the Windows fallback matrix.
- [ ] Verify package archives contain required shaders/docs and no example-only files.
- [ ] Confirm every affected crate remains above 85% for lines, functions,
      regions and branches.
- [ ] Search for stale exhaustive matches, `TODO`, `FIXME`, `todo!()`,
      `unimplemented!()` and compatibility shims introduced by this work.
- [ ] Confirm every new/changed public function and method has accurate Rustdoc.
- [ ] Confirm every Rust file is at most 600 physical lines and every test is
      under a mirrored `tests/` path rather than `src/`.
- [ ] Confirm all checkboxes in this plan are complete and all lasting decisions
      have moved to public docs/tests.
- [ ] Confirm the reviewed Unreleased changelog entry and GPU Canvas Lab are in
      the final diff before removing any implementation-plan evidence.
- [ ] Delete `docs/todo_gpu_canvas.md` only now, as the last documentation edit
      in the final implementation patch.
- [ ] Run `./scripts/quality.sh` exactly once after implementation and plan
      deletion are complete, immediately before the final commit.
- [ ] Commit the 0.3 feature, public docs, example, changelog and deletion of this
      plan together; confirm the protected branch CI is green.

## Definition of done

The work is complete only when all of the following are true:

1. An application can create custom WGPU pipelines and resources using Argui's
   device/queue, encode compute/render/copy work into a supplied encoder, and
   render into a supplied texture view.
2. Argui alone acquires, submits and presents the window surface.
3. UI/paint/layout crates remain WGPU-independent.
4. The canvas is a normal retained element with correct layout, input,
   accessibility, ordering, transforms, clips, opacity, radii and effects.
5. Unchanged canvas content is reused; resize/revision changes rerender; paused
   applications do not spin.
6. Resource usage is bounded and visible in profiling/DevTools.
7. Recoverable canvas errors leave the surrounding UI alive and produce one
   actionable English diagnostic plus a visible placeholder.
8. Multi-window, native popup, fallback, native desktop and WebAssembly paths
   have direct evidence.
9. The GPU Canvas Lab demonstrates a real Argui application shell around custom
   compute/render work on native and WebAssembly.
10. Permanent docs, release notes, tests, package checks and the final quality
    gate are complete.
11. This temporary plan has been deleted from the final feature commit.
