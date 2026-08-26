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
- `argui-text`: shaping, bidi, fallback, line breaking, cursor geometry, and
  glyph preparation through `cosmic-text`.
- `argui-render`: WGPU resources, batching, atlases, clips, layers, filters, and
  surface presentation.
- `argui-runtime`: the composition root connecting the window, retained UI,
  layout, text, and renderer. It owns scheduling but none of their algorithms.
- `argui-ui`: retained elements, reconciliation, widget state, focus, hit testing,
  and animation scheduling.
- `argui`: deliberate re-exports; application code should start here.

Text shaping is not text editing. Selection, copy/paste commands, IME composition,
and undo belong to UI/platform code around the text engine. This avoids rejecting
`cosmic-text` for a responsibility it does not claim to own.

## DSL seam

`argui-ui::Element` is the lowering target. A future `argui-dsl` crate can parse
CSS-like or custom syntax and produce the same elements/styles as Rust builders.
The DSL may depend on the public UI description API; the UI runtime, layout,
text, and renderer must never depend on the DSL or parser.

That makes the DSL optional, replaceable, testable without a GPU, and unable to
leak syntax concerns into rendering. The initial `Element` is intentionally tiny;
styles, events, and components are added only when their runtime representation
is understood.
