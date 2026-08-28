# Vector rendering

`argui-vector` is the SVG ingestion boundary. It parses SVG paths with `usvg`
and tessellates their fills and strokes with Lyon once, when the application
creates its immutable assets. It never rasterizes SVG into a bitmap.

The resulting `VectorAsset` is registered through `Render::vector_assets` and
uploaded once. `Element::vector(id)` then emits a renderer-independent vector
command. `argui-render` draws its vertex and index buffers directly with WGPU,
using the same transforms, clipping, z-order, layers and effects as every other
display-list primitive. Native and WASM therefore execute the same UI tree and
the same WGSL pipeline.

```rust
let mut vectors = VectorLibrary::new();
let id = vectors.insert_svg(include_bytes!("icon.svg"))?;

Element::vector(id)
    .width(Length::Px(24.0))
    .height(Length::Px(24.0));
```

## Morphing

`morph_svg` accepts two SVGs with corresponding path commands and verifies the
affine mapping between their control points. It tessellates only the source, so
the two states always share one stable topology. Each GPU vertex stores `from`
and `to`; the WGPU vertex shader interpolates them from
`Element::vector_progress(0.0..=1.0)`. Updating progress uploads only the small
instance record: it does not parse or retessellate the SVG and does not allocate
a texture.

Complex morphs with different path structures need an explicit normalization
step before registration. Silent raster fallback is intentionally forbidden.

## Current SVG boundary

The first implementation supports visible path fills and strokes, Bézier
curves, nested transforms, paint/group opacity, fill rules, paint order, joins and caps. Embedded raster images,
text, patterns and non-solid SVG paints return a typed error. Those features
must be added as native vector/display-list capabilities instead of secretly
falling back to CPU pixels.

The WGPU vector pipeline expands indexed meshes once during registration and
adds analytic coverage only to their exterior edges. Small stroked icons remain
smooth without MSAA, while internal tessellation edges stay invisible.
