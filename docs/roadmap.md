# Build roadmap

Each step ends with tests, the quality gate, native profiling, and a WASM compile
check before the next feature starts.

1. **Window shell — implemented.** Open one `winit` window/canvas. Configure title, size,
   decorations, resizing, and transparency; translate close, resize, scale, and
   redraw events. Verify zero continuous redraw while idle.
2. **WGPU surface — implemented.** Select an adapter, configure/reconfigure the surface, clear,
   and present on native and web. Recover cleanly from lost/outdated surfaces.
3. **Text — rendering and single-line editing implemented.** Shape bidi/fallback text
   with `cosmic-text`; use a bounded reusable glyph atlas, one instanced batch,
   and per-block clipping. Caret geometry, grapheme-safe selection/editing, IME,
   and native/web clipboard behavior share that shaped representation. Add
   multiline editing and richer composition decoration next. See
   [text input](text_input.md).
4. **Retained state and interaction — foundation implemented.** Persistent keyed elements,
   stable reconciled `NodeId`s, revision tracking, clipped reverse-order hit
   testing, hover/press/focus state, pointer capture, UI events, and paint-only
   invalidation are present. `UiApp` now updates plain Rust state and rebuilds a
   view classified as no work, repaint, or relayout. Keyboard focus traversal is
   present. Add reusable local component state, touch, and accessibility integration.
5. **Layout — responsive foundation implemented.** A small renderer-independent
   style model maps to `taffy`; Cosmic Text performs constrained intrinsic
   measurement and native/web viewport changes share one reflow path. Add
   per-subtree invalidation and intrinsic measurement caching as the tree grows.
6. **Paint primitives — implemented.** Ordered quads and text, solid fills,
   per-side borders, per-corner radii, primitive opacity, nested rectangular
   clipping, analytic GPU antialiasing, and compatible-command batching work on
   native and web. Gradients, images, transforms, and shadows extend this layer.
7. **Widgets — interaction, scroll, and overlays implemented.** `Button`
   composes layout, paint, text, and interaction. Retained nested scroll regions,
   configurable wheel polarity, fast geometry translation, stable z-index,
   absolute overlays, draggable frame-coalesced scrollbars, and fixed-row virtual
   lists and a composed single-line `TextInput` share native and web code. Add
   variable-row virtualization, reusable state ownership, focus traps, controlled
   inputs, and accessibility nodes. See the [scroll model](scroll.md).
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
