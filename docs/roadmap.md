# Build roadmap

Each step ends with tests, the quality gate, native profiling, and a WASM compile
check before the next feature starts.

1. **Window shell — implemented.** Open one `winit` window/canvas. Configure title, size,
   decorations, resizing, and transparency; translate close, resize, scale, and
   redraw events. Verify zero continuous redraw while idle.
2. **WGPU surface — implemented.** Select an adapter, configure/reconfigure the surface, clear,
   and present on native and web. Recover cleanly from lost/outdated surfaces.
3. **Text.** Shape bidi/fallback text with `cosmic-text`; implement a reusable
   glyph atlas, batching, clipping, cursor geometry, selection, IME, clipboard,
   and copy/paste behavior around it.
4. **Retained tree.** Stable identities, reconciliation, explicit dirty flags,
   hit testing, focus, pointer capture, and a compact display list.
5. **Layout.** Map a small public style model to `taffy`; cache intrinsic text
   measurement and invalidate only affected ancestors.
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
