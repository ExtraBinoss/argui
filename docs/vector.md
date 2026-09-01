# Vector rendering

`argui-vector` validates immutable SVG resources with `usvg` and retains their
resolution-independent source. Applications register the resulting
`VectorAsset` values through `Render::vector_assets`. `Element::vector(id)` then
emits a normal display-list primitive with `ImageFit`, paint-time color,
opacity, transforms, clips, z-order, layers and effects.

```rust
let mut vectors = VectorLibrary::new();
let id = vectors.insert_svg(include_bytes!("icon.svg"))?;

Element::vector(id)
    .vector_fit(ImageFit::Contain)
    .vector_color(theme.foreground)
    .width(Length::Px(24.0))
    .height(Length::Px(24.0));
```

## Raster and GPU boundary

`argui-render` parses each registered source into a retained `resvg` tree. The
first request for a physical-size variant rasterizes it with tiny-skia's
antialiased SVG renderer. That RGBA result is uploaded into one 2048×2048 sRGB
WGPU atlas. Later frames draw instanced quads from the atlas; they neither parse
nor rasterize the SVG again.

The atlas has a fixed 16 MiB GPU budget and transparent two-pixel gutters so
linear sampling cannot leak adjacent icons. Nearby requested sizes reuse the
closest cached variant within explicit quality hysteresis. A full atlas is
recycled as a unit and rebuilt only from the current visible display list, so
memory stays bounded on native WGPU and WebGPU.

Adjacent vector commands share one render batch regardless of asset identity.
The common vector WGSL pipeline applies instance transforms, rounded clip-chain
coverage, opacity and tint without creating per-icon pipelines or bind groups.

## Color and static SVG features

SVGs authored with `currentColor` become alpha masks. Their RGB color is supplied
by `Element::vector_color`, so light/dark themes and interaction states update a
small instance record without duplicating or rerasterizing the asset. SVGs
without `currentColor` preserve their authored colors.

The retained static path accepts the shapes, curves, strokes, transforms,
gradients, opacity, clipping, masks and filters supported by the configured
`usvg`/`resvg` build. Font resolution, system fonts and external resource lookup
are intentionally not part of the icon renderer.

## Animation

Vector elements use the same paint-time transform and opacity bindings as other
primitives, so rotation, scale, translation and fades stay on the retained GPU
path. The former affine-only `morph_svg` API was removed: it rejected legitimate
SVG pairs and did not provide a general path-morph contract.

## Profiling

`RenderProfile::vector_atlas` reports entry count, cache hits, rasterizations and
allocated bytes for the current frame. DevTools records the same counters in
strict `argui-gpu-trace-v2` traces, making resize thrashing visible on Linux,
Windows, macOS and WebGPU.
