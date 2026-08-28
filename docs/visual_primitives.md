# Transforms, gradients, and images

Visual primitives stay renderer-independent. `argui-ui` exposes ergonomic
builders, `argui-paint` records immutable commands, and `argui-render` uploads
only visible frame data to WGPU. Native and WASM consume the same display list.

## Transforms

`Transform2D` provides translation, scale, rotation, skew, and a normalized
`TransformOrigin`. Transforms compose through the retained tree and apply to
quads, text, images, exact nested clips, effect layers, and hit testing. They
run after Taffy layout, so animating a transform never recomputes layout.

The type implements `Interpolate` and `MotionValue`. It therefore works with
typed keyframes, implicit transitions, springs, decay, and velocity-preserving
spring retargeting without a transform-specific timing engine.

## Gradients

`Fill` is shared by every painted element. A linear or radial gradient can
therefore fill a container, button, card, or any composed widget:

```rust
let stops = GradientStops::from_vec(generated_stops)?;
let fill = Fill::Linear(LinearGradient::with_stops(
    Point::new(0.0, 0.0),
    Point::new(1.0, 0.0),
    stops,
))
```

There is no fixed per-gradient stop count. Stops are stored in shared immutable
memory on the CPU and packed into one storage buffer per frame. The total is
bounded by `RendererConfig::gradient_stop_capacity` (65,536 by default), so an
application can raise or lower the explicit GPU-memory/work budget. Gradients
with matching topology interpolate as gradients during implicit transitions.

## Images

The renderer contract is a validated `ImageAsset`: stable `ImageId`, dimensions,
and RGBA8 pixels. The optional `argui-image` crate decodes PNG and JPEG with only
those two codec features enabled. Applications may instead supply pixels from
another decoder, an asset pack, or generated content.

`argui_image::ImageLibrary::insert` generates an opaque handle and stores the
decoded asset, so application code never invents numeric IDs. Applications
return the library assets once from `Render::image_assets`; both native and WASM
launchers register them automatically. `Element::image` supports
`Fill`, `Contain`, and `Cover`, linear or nearest sampling, rounded corners,
nested clipping, opacity, transforms, overlays, and effect layers.

GPU texture residency is bounded by `RendererConfig::image_cache_bytes` (64 MiB
by default). Registration fails explicitly if an asset cannot fit; normal
non-image scenes allocate no image texture.

## Showcase

Run `cargo run -p argui --example state` or `./scripts/serve-web.sh`. Both launch
the same Rust `StateShowcase`, including a generated 12-stop gradient, transformed
content, the supplied transparent PNG, and the supplied JPEG.
