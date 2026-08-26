# Architecture

Argui is retained-mode with selective immediate GPU recording: the UI tree and
widget state persist between frames, while a paint pass emits a compact display
list only for dirty regions. Animation schedules frames; an idle UI does not.

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

- `argui-core`: stable geometry and shared IDs/events as they become necessary.
- `argui-platform`: `winit` window lifecycle, input, IME, clipboard hooks, and
  native/web surface handles.
- `argui-layout`: the narrow adapter from UI style/tree data to `taffy`.
- `argui-paint`: renderer-independent fills, borders, corner radii, clips, and
  the ordered display list shared by native and web.
- `argui-text`: shaping, bidi, fallback, line breaking, cursor geometry, and
  glyph preparation through `cosmic-text`.
- `argui-render`: WGPU resources, batching, atlases, clips, layers, filters, and
  surface presentation.
- `argui-runtime`: the composition root connecting the window, retained UI,
  layout, text, and renderer. It owns scheduling but none of their algorithms.
- `argui-ui`: retained elements, reconciliation, widget state, focus, hit testing,
  and animation scheduling.
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

That makes the DSL optional, replaceable, testable without a GPU, and unable to
leak syntax concerns into rendering. The initial `Element` is intentionally tiny;
styles, events, and components are added only when their runtime representation
is understood.

## Retained layout flow

`argui-ui::UiTree` owns the persistent element description, a revision, and an
explicit layout-dirty flag. `argui-layout::LayoutEngine` rebuilds its Taffy tree
only when that revision changes; viewport or DPI changes reuse it. Text leaves
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

Hover, press, and focus select a `QuadStyle`, which excludes layout and clipping.
The layout engine can therefore rebuild only the display list for these state
changes. Taffy geometry and shaped Cosmic Text remain untouched. See the
[interaction model](interaction.md) for the event and invalidation contract.

## Application state flow

`argui-runtime::UiApp` connects plain Rust state to the retained tree. Its
`update` method consumes UI events and its `view` method returns the same
`Element` representation available to Rust builders and the future DSL.
`UiTree` compares rebuilt descriptions and selects no work, paint-only work, or
layout work while keyed reconciliation preserves stable identities. See the
[application state model](state.md).

An application implements this boundary once for every target. Native and WASM
need different executable entry symbols, but those launchers contain no view,
style, state, or business logic. A future project generator can own these static
entrypoint files completely.

## Scroll and stacking

Scroll input keeps line and pixel units distinct until the target container
applies its policy. Offsets live beside interaction state in `argui-ui`; Taffy
geometry remains unchanged while layout output, prepared glyphs, clips, and hit
regions are translated. Stable sibling `z_index` ordering drives both painting
and reverse hit testing. See [scroll, stacking, and virtual lists](scroll.md).
