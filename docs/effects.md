# Effects implementation plan

Effects keep the existing dependency direction:

```text
argui-ui              ergonomic style and builders
    |
argui-paint           declarative effects and layer commands
    |
argui-render          render graph, textures, pipelines, and WGPU execution
```

`argui-paint` never imports WGPU, compiles shaders, or allocates textures. An UI
without effects keeps the current direct surface-rendering path and pays no
off-screen texture or extra-pass cost.

## Public model

The paint model will grow only when its first renderer implementation lands:

```rust
enum Filter {
    Blur(f32),
    Brightness(f32),
    Contrast(f32),
    Saturation(f32),
    ColorMatrix([f32; 20]),
    Custom(CustomEffect),
}

enum BlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Difference,
}

struct CustomEffect {
    shader: ShaderEffectId,
    parameters: ParameterRange,
}
```

The display list will use balanced `BeginLayer(LayerStyle)` and `EndLayer`
commands. A layer may describe group opacity, a blend mode, foreground filters,
backdrop filters, and a mask or rounded clip. Shader source is registered once
with the renderer; display commands contain only a stable ID and parameters in
a reusable per-frame data arena.

## Implementation order

1. **Layer semantics.** Add renderer-independent layer/filter/blend descriptors
   and validate balanced display-list commands. Compute conservative layer
   bounds, including filter expansion, without touching the GPU.
2. **Render graph.** Lower flat commands and nested layers into explicit passes.
   Keep ordinary content on the surface path; allocate an intermediate target
   only when the layer semantics require one.
3. **Bounded texture pool.** Reuse off-screen textures by format and power-of-two
   size class, enforce a configurable memory budget, evict deterministically,
   and release oversized targets after the frame.
4. **Compositing foundation.** Implement group opacity, transforms, rectangular
   and rounded masks, then the blend modes supported safely by fixed-function
   WGPU blending. Fall back to an explicit composition pass when destination
   sampling is required.
5. **Blur and shadows.** Implement separable GPU blur with downsampling for large
   radii. Build box/text shadows from the same mask and blur machinery instead
   of maintaining a second effect path.
6. **Color filters.** Combine brightness, contrast, saturation, and color matrix
   operations into one pass when possible. Fuse adjacent compatible filters to
   avoid temporary textures.
7. **Backdrop filters.** Snapshot only the affected background region, filter
   it, clip it to the layer shape, and composite its foreground. Nested backdrop
   layers must preserve paint order and never sample later content.
8. **Custom WGSL effects.** Define a small, versioned shader ABI for input
   texture, sampler, geometry, frame data, and parameters. Validate and compile
   registration once, cache pipelines by shader ID and target format, and
   return structured errors on native and WebGPU.
9. **Optimization and diagnostics.** Merge compatible passes, skip identity
   effects, expose pass/texture-byte counters, and profile representative native
   and browser scenes before adding special cases.

## Acceptance checks for every step

- The same scene behaves on native WGPU and browser WebGPU.
- Paint-model and render-graph decisions have CPU tests without a GPU.
- Pixel tests use deterministic off-screen targets where supported.
- Nested layers, clipping, resize, DPI changes, and surface loss are covered.
- Idle UI performs no redraw and no effect work.
- A scene without effects uses no intermediate textures.
- Texture-pool memory is bounded and observable.
- All repository quality and coverage gates continue to pass.

This milestone comes after the retained interaction foundation. Its primitives
can land incrementally: group opacity and masks do not need to wait for backdrop
filters or custom shaders.
