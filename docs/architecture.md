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

`argui-core` supplies dependency-light types to these layers. `argui` is the
public facade; it contains no second implementation. See
[repository structure](repo/structure.md) for the complete crate graph.

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
| Transform or group opacity | layout, shaped text, paint primitives, GPU uploads | compositor properties and presentation geometry |
| Scroll offset | layout and shaped text | translated paint and hit geometry |
| Text, size, structure, or layout style | stable keyed nodes where possible | affected layout, text, and paint |

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
| Reusable controlled components | `argui-widgets` |

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

Applications implement `Render` or `AppModel` once. Native and WebAssembly
launchers differ only at the executable entry point. Android and iOS add small
ABI shells, while the model, layout, text, paint, and renderer remain shared.

Target-specific code belongs at the platform edge. Do not put Winit handles in
UI state, WGPU resources in paint descriptions, or OS policy in reusable
widgets. Optional integrations such as localization, WebView, updater and
DevTools remain feature-gated. DSL compilation and transactional development
reload live outside the engine and never become release runtime dependencies.

## Where to change code

| Goal | Start in |
| --- | --- |
| Add a widget or change widget behavior | `crates/argui-widgets` |
| Change style resolution, focus, input, or reconciliation | `crates/argui-ui` |
| Change Flexbox/Grid integration | `crates/argui-layout` |
| Change shaping or glyph preparation | `crates/argui-text` |
| Add a paint primitive | `argui-paint`, then `argui-render` |
| Add an OS capability | `crates/argui-platform`, then expose it through the runtime |
| Change model lifetime or scheduling | `crates/argui-runtime` |
| Change DSL syntax, semantics, AOT or live development | `argui-dsl` |
| Add an application example | `app_examples` or `crates/argui-widget-gallery` |

Follow the [development guide](contributing/development.md) before crossing a
crate boundary.
