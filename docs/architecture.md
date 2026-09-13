# Architecture

For a complete workspace inventory, direct dependency table and crates.io
order, see [workspace crates and dependencies](repo/structure.md).

Argui is retained-mode with selective immediate GPU recording: the UI tree and
widget state persist between frames. Paint passes reuse cached subtree fragments
and assemble an ordered display list; some assembly work still spans the tree.
Animation schedules frames; an idle UI does not.

```text
Rust builders or future DSL
            |
         argui-ui
       /           \
layout            text
       \           /
          paint
            |
        argui-runtime
        /           \
  platform       render
    winit          wgpu
```

The actual dependency graph stays acyclic even where the diagram groups runtime
collaboration. `argui-core` contains dependency-light shared primitives, and
`argui` is only the public facade.

## Crate boundaries

- `argui-core`: shared geometry, colors and input primitives.
- `argui-accessibility`: renderer-independent semantics, incremental tree
  patches, AccessKit lowering, and the browser semantic DOM adapter.
- `argui-platform`: `winit` window lifecycle, input, IME, clipboard hooks, and
  native/web surface handles, application identity, and optional native tray.
- `argui-layout`: the narrow adapter from UI style/tree data to `taffy`.
- `argui-paint`: renderer-independent fills, borders, corner radii, clips, and
  the ordered display list shared by native and web.
- `argui-text`: shaping, bidi, fallback, line breaking, cursor geometry, and
  glyph preparation through `cosmic-text`.
- `argui-i18n`: renderer-independent Fluent catalogs, locale negotiation,
  message fallback, formatting, and writing-direction metadata.
- `argui-render`: WGPU resources, batching, atlases, clips, layers, filters, and
  surface presentation.
- `argui-runtime`: the composition root connecting the window, retained UI,
  layout, text, and renderer. It owns scheduling but none of their algorithms.
- `argui-ui`: retained elements, reconciliation, widget state, focus, hit testing,
  and animation scheduling.
- `argui-inspect`: renderer-independent tree snapshots, reversible typed style
  overrides, and bounded frame records.
- `argui-devtools`: an optional application wrapper whose dock, virtualized tree,
  controls, and highlights are ordinary Argui elements. Engine crates never
  depend on this frontend.
- `argui-animation`: typed timelines, motion, interpolation and physics.
- `argui-theme` and `argui-widgets`: theme values and optional controlled widgets.
- `argui-image` and `argui-vector`: optional asset decoding and validation.
- `argui-effects`: optional effect definitions; the renderer executes their passes.
- `argui-webview`: optional retained native WebViews and browser frames.
- `argui-updater`: optional update transactions, signed downloads and desktop
  installation adapters, independent of the renderer, runtime and UI. The
  controlled update dialog belongs to the separate `argui-widgets/updater` feature.
- `argui`: deliberate re-exports; application code should start here.
- `argui-showcase`: non-published example application shared unchanged by the
  native and WASM launchers; it is not part of the framework dependency graph.

Text shaping is not text editing. Selection, copy/paste commands, IME composition,
and undo belong to UI/platform code around the text engine. This avoids rejecting
`cosmic-text` for a responsibility it does not claim to own.

Fonts are application resources. Native applications may use the system-backed
`TextEngine::new()`. Sandboxed targets can create an isolated engine with
`TextEngine::from_embedded_fonts(...)` and pass it to the runtime. The library
does not force a font, a download, or an application bundle-size cost.

## DSL seam

`argui-ui::Element` is the lowering target. A future `argui-dsl` crate can parse
CSS-like or custom syntax and produce the same elements/styles as Rust builders.
The DSL may depend on the public UI description API; the UI runtime, layout,
text, and renderer must never depend on the DSL or parser.

The DSL remains optional and testable without a GPU. Its syntax does not become
part of the runtime representation.

## Retained layout flow

`argui-ui::UiTree` owns the persistent element description, a revision, and an
explicit dirty state. `argui-layout::LayoutEngine` reconciles retained Taffy nodes
when the description changes, reusing unchanged branches and measurement caches.
Viewport and DPI changes also reuse the retained layout tree. Text leaves
are measured by Cosmic Text under Taffy's width constraint, then lowered to a
`TextScene`. No layout or shaping work runs while the event loop is idle.

Painting preserves tree order across primitive types. Consecutive compatible
commands are batched, but a later quad is never moved behind earlier text merely
to reduce draw calls. The WGPU quad pipeline uses one reusable instance buffer;
rounded corners, asymmetric borders, clipping, and antialiasing stay in WGSL.

## Interaction flow

Winit pointer and focus events are translated by `argui-platform`; the runtime
normalizes coordinates and dispatches them through `argui-ui`. Stable node IDs,
clipped reverse-order hit testing, focus, and pointer capture remain independent
from WGPU. Application events include the stable ID and optional element key.

Semantic nodes reuse those stable IDs. Native AccessKit and the browser DOM are
updated from semantic diffs after UI dispatch; semantic-only mutations never
invalidate Taffy or paint. Pointer contacts use one mouse/touch/pen event schema,
and the UI gesture arena remains independent from both platform and renderer.

Hover, press, focus, named state, and container conditions compose sparse typed
`StylePatch` values. Paint and transform properties reuse Taffy geometry;
layout properties update retained Taffy nodes; text color updates prepared text
without reshaping glyphs. See the [interaction model](ui/interaction.md) and
[responsive styling](ui/styling.md) for the invalidation contract.

## Application state flow

`argui-runtime::Render`, `Context<T>`, and `Entity<T>` connect component-local
Rust state to the retained tree. Each mount owns an independent render cache;
clean presentations return their exact cached COW subtree. Notifications
invalidate dependent presentations and their ancestor composition paths.
`UiTree` uses pointer equality to skip shared descendants, and keyed
reconciliation preserves stable identities in linear sibling work. See the
[application state model](runtime/models.md).

The optional native `hot-reload` feature resolves each `Render` hook and
`AppModel` callback through Subsecond in debug builds. After a patch, the
runtime advances a generation, invalidates each window and rejects render-cache
entries from older generations while retaining the model instances. The bridge
is compiled out of release and WebAssembly code. See the
[hot-reload guide](hot-reload.md).

`DevtoolsHost<A>` decorates the retained application boundary without changing
`A`. It gives the application the remaining docked viewport, delegates assets, shaders,
events and animation lifecycle, and attaches an `InspectorHandle` to the
runtime. `DevtoolsApp<M>` wraps a multi-window application and supports a
detached native tools window. The inspection protocol contains no `Element`,
WGPU resource, or platform handle.

An application implements this boundary once for every target. Native and WASM
need different executable entry symbols, but those launchers contain no view,
style, state, or business logic. A future project generator can own these static
entrypoint files completely.

`AppModel` extends the same retained boundary to several windows. A stable
`WindowKey` routes events and invalidations, each window keeps independent UI,
layout, scroll, and surface state, and all surfaces reuse the same WGPU device.
The optional tray and Web favicon consume the same immutable application icon
set and never add work to a frame.

## Retained performance path

`Element` is a copy-on-write `Rc` node. Taffy `NodeMap`s retain the exact
element and subtree length, allowing layout reconciliation to jump over an
unchanged branch in constant time. Quad uploads compare stable POD ranges and
write only the changed interval through WGPU, identically on Vulkan, Metal,
DX12, and WebGPU.

`VirtualList` handles fixed and variable rows. Variable extents use a Fenwick
prefix tree, find visible rows in `O(log n)`, receive their real measurements
from layout, and correct the scroll offset to preserve the current top anchor.
Run `cargo run -p argui-perf-showcase --example perf` for the native million-row
and component-isolation labs; the crate's `cdylib` entry runs the same labs on
WASM without DevTools instrumentation.

## Scroll and stacking

Scroll input keeps line and pixel units distinct until the target container
applies its policy. Offsets live beside interaction state in `argui-ui`; Taffy
geometry remains unchanged while layout output, prepared glyphs, clips, and hit
regions are translated. Stable sibling `z_index` ordering drives both painting
and reverse hit testing. See [scroll, stacking, and virtual lists](ui/scroll.md).

Storage decisions, baselines and remaining costs are documented in the
[optimization measurements](performance/optimizations.md).
