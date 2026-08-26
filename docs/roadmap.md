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
6. **Widgets.** Compose primitives into text, containers, buttons, input, scroll,
   overlays, and accessibility nodes without renderer-specific widget code.
7. **Layers and effects.** Stacking contexts, z-index, clips, transforms, opacity,
   shadows, filters, and backdrop filters. Reuse off-screen textures with bounded
   pools and allocate layers only when semantics require them.
8. **Animation.** Typed transitions, keyframes, spring/timeline scheduling, and
   separation of paint-only animation from layout-invalidating animation.
9. **Optional DSL.** A separate parser/compiler lowering into the same public UI
   tree used by Rust builders. No runtime or renderer dependency on the DSL.

The first useful milestone is steps 1–3: a configurable window rendering correct,
selectable text on native and web. It validates the riskiest seams before growing
the widget system.
