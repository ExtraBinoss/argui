# Build roadmap

Each step ends with tests, the quality gate, native profiling, and a WASM compile
check before the next feature starts.

1. **Window shell — implemented.** Open one `winit` window/canvas. Configure title, size,
   decorations, resizing, and transparency; translate close, resize, scale, and
   redraw events. Verify zero continuous redraw while idle.
2. **WGPU surface — implemented.** Select an adapter, configure/reconfigure the surface, clear,
   and present on native and web. Recover cleanly from lost/outdated surfaces.
3. **Text — rendering implemented, editing pending.** Shape bidi/fallback text
   with `cosmic-text`; use a bounded reusable glyph atlas, one instanced batch,
   and per-block clipping. Add cursor geometry, selection, IME, clipboard, and
   copy/paste behavior around it next.
4. **Retained tree — foundation implemented.** Persistent keyed elements,
   revision tracking, and explicit layout invalidation are present. Add keyed
   reconciliation, per-node state, hit testing, focus, pointer capture, and a
   compact display list with the first interactive widgets.
5. **Layout — responsive foundation implemented.** A small renderer-independent
   style model maps to `taffy`; Cosmic Text performs constrained intrinsic
   measurement and native/web viewport changes share one reflow path. Add
   per-subtree invalidation and intrinsic measurement caching as the tree grows.
6. **Paint primitives — implemented.** Ordered quads and text, solid fills,
   per-side borders, per-corner radii, primitive opacity, nested rectangular
   clipping, analytic GPU antialiasing, and compatible-command batching work on
   native and web. Gradients, images, transforms, and shadows extend this layer.
7. **Widgets.** Compose primitives into text, containers, buttons, input, scroll,
   overlays, and accessibility nodes without renderer-specific widget code.
8. **Layers and effects.** Follow the staged [effects plan](effects.md): declarative
   layer commands in `argui-paint`, a render graph and bounded texture pool in
   `argui-render`, then group opacity, masks, shadows, filters, destination-aware
   blend modes, backdrop filters, and a stable custom-WGSL ABI. Allocate layers
   only when their semantics require them.
9. **Animation.** Typed transitions, keyframes, spring/timeline scheduling, and
   separation of paint-only animation from layout-invalidating animation.
10. **Optional DSL.** A separate parser/compiler lowering into the same public UI
   tree used by Rust builders. No runtime or renderer dependency on the DSL.

The first useful milestone is steps 1–3: a configurable window rendering correct,
selectable text on native and web. It validates the riskiest seams before growing
the widget system.
