# Architecture

Argui is a retained UI engine. Application models rebuild element descriptions;
the runtime reconciles them with a persistent tree and recomputes only the data
invalidated by that change. WGPU receives renderer-neutral paint commands.

```text
application model
      |
      v
argui-runtime ---- platform events and windows
      |
      v
  argui-ui ---- argui-layout ---- argui-text
      |
      v
 argui-paint
      |
      v
 argui-render ---- WGPU surface
```

`argui-core` supplies dependency-light types to these layers. Applications
depend on the engine and runtime crates directly. See [repository
structure](repo/structure.md) for the complete crate graph.

## One update

1. A platform event enters through `argui-platform`.
2. The runtime converts coordinates to logical pixels and dispatches the event
   through `argui-ui`.
3. A listener mutates an `Entity<T>` and calls `Context::notify`.
4. The affected presentation rebuilds its `Element` description.
5. Reconciliation preserves keyed node identity and classifies the change.
6. Layout, text, paint, and accessibility update only when their inputs changed.
7. The renderer submits the ordered display list and presents the surface.

The event loop schedules no frame while the application is idle. Animation,
scroll momentum, async completion, window changes, or explicit invalidation wake
it.

## Invalidation

| Change | Reused | Recomputed |
| --- | --- | --- |
| Identical description | tree, layout, text, paint | nothing |
| Semantics only | layout, text, paint | accessibility diff |
| Color, primitive opacity, or effects | layout, shaped text, node identity | paint data |
| Active transform animation or group opacity | layout, shaped text, paint primitives, GPU uploads | compositor properties and presentation geometry |
| Settled transform animation or authored text transform | layout and shaped text | paint data and glyphs at their final physical size |
| Scroll offset | layout and shaped text | translated paint and hit geometry |
| Text, size, structure, or layout style | stable keyed nodes where possible | affected layout, text, and paint |

See [text fidelity](rendering/text.md) for subpixel positioning, atlas budgets,
and the independent logical-layout and physical-raster caches.

`Element::layout_boundary` stops a child's intrinsic size from invalidating an
ancestor. Use it only when the parent supplies the child's size and clips or
scrolls both axes. Virtualized lists rely on the same ownership rule: the model
can contain many rows while the UI mounts only the visible window.

Painting preserves tree order. Compatible adjacent commands may batch, but
batching never moves later content behind earlier content. Hit testing follows
the same paint and clip order in reverse.

Retained GPU canvases preserve the same boundary. UI and paint store only an
opaque registration ID, retained object/slot, revision and composition data.
`argui-render` owns each bounded target texture, invokes application WGPU code
only when that target is dirty, then samples it in ordinary display/effect
order. Argui remains the sole owner of command submission and presentation.
See the [GPU canvas guide](rendering/gpu-canvas.md).

## State and identity

`Entity<T>` owns shared model data. A renderable entity can have several
`Mount<T>` presentations, each with its own environment, handlers, render
cache, and view-owned tasks. A stable `NodeId` belongs to the retained UI tree.
Keys preserve node identity when siblings move.

Model notifications invalidate presentations that observed the model. Typed
events and services coordinate models without global registries. Resource scopes
own tasks, subscriptions, and cleanup. The full lifetime contract is in
[models](runtime/models.md).

## Layer ownership

| Concern | Owner |
| --- | --- |
| Geometry, color, identifiers | `argui-core` |
| Elements, styles, focus, hit testing, semantics | `argui-ui` |
| Flexbox and Grid adaptation | `argui-layout` |
| Shaping, bidi, fallback, glyph preparation | `argui-text` |
| Renderer-neutral draw commands | `argui-paint` |
| GPU resources, batching, effects, presentation | `argui-render` |
| Windows, native input, clipboard and OS adapters | `argui-platform` |
| Models, scheduling, composition, multi-window lifecycle | `argui-runtime` |
| TSX host transactions and schema | `argui-host`, `argui-schema` |
| Reusable Solid and React components | `packages/widgets`, `packages/solid`, `packages/react` |

Text shaping does not own editing. Selection, IME, clipboard commands, and undo
cross UI and platform layers. The renderer does not own timelines. Animation
state lives in `argui-animation` and the retained UI; the renderer receives the
resolved value for the current frame.

Accessibility follows the same stable node IDs. Semantic-only changes bypass
layout and paint. Native AccessKit and the browser semantic DOM consume
incremental semantic patches after event dispatch.

The [retained compositor](rendering/compositor.md) is a distinct invalidation
phase between paint and presentation. The UI thread still owns event ordering,
tree state, and animation sampling; WGPU owns cached layer surfaces and applies
their transform and group opacity. This boundary is renderer-neutral and shared
by native windows, native popups, and WebAssembly.

## Platform boundary

Rust applications implement `Render` or `AppModel` once. Native and WebAssembly
launchers differ at the executable entry point. Android has a checked-in
`NativeActivity` shell for the TSX gallery; `argui-runtime` provides optional
Android and iOS entry modules for native integrations. The repository does not include
an iOS app shell or Xcode project.

Target-specific code belongs at the platform edge. Do not put Winit handles in
UI state, WGPU resources in paint descriptions, or OS policy in reusable
components. Optional integrations such as localization, WebView, and updater
remain feature-gated. JavaScript framework adapters communicate with the native
UI through the shared `argui-host` transaction contract and `argui-schema`.

For the Solid and React gallery, QuickJS is the selected embedded JavaScript
runtime on desktop and Android. Bun builds and tests the TSX bundles; it is not
embedded in the app. The same Rust transaction host and `UiTree` serve both
framework adapters. Reusable components live in `@argui/widgets`. The
[native gallery architecture and measurements](solid-react-native.md) record
the runtime decision, mobile limits, and performance data.

## Where to change code

| Goal | Start in |
| --- | --- |
| Add or change a reusable TSX component | `packages/widgets/src/` |
| Change Solid or React host rendering | `packages/solid/` or `packages/react/` |
| Change the JS/Rust transaction contract | `packages/host/`, `crates/argui-host`, and `crates/argui-schema` |
| Change style resolution, focus, input, or reconciliation | `crates/argui-ui` |
| Change Flexbox/Grid integration | `crates/argui-layout` |
| Change shaping or glyph preparation | `crates/argui-text` |
| Add a paint primitive | `argui-paint`, then `argui-render` |
| Add an OS capability | `crates/argui-platform`, then expose it through the runtime |
| Change model lifetime or scheduling | `crates/argui-runtime` |
| Add a TSX application example | `apps/gallery/` |

Follow the [development guide](contributing/development.md) before crossing a
crate boundary.
