# Visual primitives

`argui-ui` exposes builders, `argui-paint` records immutable
renderer-independent commands, and `argui-render` uploads visible frame data to
WGPU. Native and WebAssembly use the same display list.

## Transforms

`Transform2D` supports translation, scale, rotation, skew, and normalized
`TransformOrigin`. Transforms apply after layout to quads, text, images, vectors,
clips, effects, and hit testing. Retained transform motion uses the
[compositor path](compositor.md), so animation reruns neither Taffy nor primitive
preparation.

The type implements `Interpolate` and `MotionValue` and works with keyframes,
springs, decay, and velocity-preserving retargeting.

## Fills and gradients

`Fill` can be solid, linear, radial, or bilinear. Every painted element uses the
same type.

```rust,ignore
let stops = GradientStops::from_vec(generated_stops)?;
let fill = Fill::Linear(LinearGradient::with_stops(
    Point::new(0.0, 0.0),
    Point::new(1.0, 0.0),
    stops,
));
```

Gradient stops use shared immutable CPU storage and one frame storage buffer.
`RendererConfig::gradient_stop_capacity` bounds the total; its default is
65,536. Geometry and stops can bind to typed motions.

`BilinearGradient` interpolates four premultiplied corners. The ColorPicker's
HSV pad uses one bilinear quad: white/hue at the top and black at the bottom.

## Color

Visual colors are authored in sRGB and converted once to extended linear sRGB.
Painting and effects remain linear; an sRGB surface performs final display
encoding.

- Use `Color::srgb`, `srgba`, 8-bit constructors, or `from_hex` for authored
  colors.
- Use `linear_rgb` or `linear_rgba` only for renderer math and shader data.
- Use `to_srgba` for editing and serialization.
- Use `to_linear_rgba` for GPU upload.

Gradients choose `Oklab`, `LinearSrgb`, or `Srgb` interpolation and use
premultiplied alpha to avoid dark transparent fringes. Widget state colors and
typed color motions default to OKLab.

Primitive shaders output straight linear RGBA. WGPU blending stores premultiplied
linear intermediate textures. Custom effects receive straight linear RGBA
through an ABI that converts to and from those intermediates.

Image and rasterized SVG textures use sRGB sampling. Glyph atlases store coverage;
the text shader applies the renderer's text-coverage transfer and linear text
color before blending. Display-P3, HDR, and ICC color management are outside the
current contract.

## Images

`ImageAsset` contains a stable `ImageId`, dimensions, and RGBA8 pixels.
`argui-image` optionally decodes PNG and JPEG; applications can register pixels
from another decoder.

```rust,ignore
let id = image_library.insert(include_bytes!("photo.jpg"))?;
let image = Element::image(id).image_fit(ImageFit::Cover);
```

Return registered assets from `Render::image_assets`. `Element::image`
supports Fill, Contain, Cover, sampling choice, rounded clips, opacity,
transforms, overlays, and effects.

`RendererConfig::image_cache_bytes` bounds GPU residency at 64 MiB by default.
A scene without images allocates no image texture.

## Vectors

`argui-vector` parses SVG into immutable `VectorAsset` values:

```rust,ignore
let mut vectors = VectorLibrary::new();
let id = vectors.insert_svg(include_bytes!("icon.svg"))?;

let icon = Element::vector(id)
    .vector_fit(ImageFit::Contain)
    .vector_color(theme.foreground)
    .width(Length::Px(24.0))
    .height(Length::Px(24.0));
```

Return assets from `Render::vector_assets`. The renderer retains the parsed SVG,
rasterizes requested physical sizes on first use, and packs them into one bounded
sRGB atlas. Nearby sizes reuse a cached variant. At capacity, visible content
can repack the atlas; a frame that still cannot fit fails explicitly.

`currentColor` SVGs become alpha masks and take
`Element::vector_color`, so theme changes update instance data without
rerasterizing. Other SVGs preserve authored colors. External resources and
system-font lookup are disabled.

Vector elements share transforms, opacity, clips, layers, and effects with other
primitives. General SVG path morphing is not part of the vector asset API; use
crossfade or transform animation between assets.

`RenderProfile::vector_atlas` reports entries, hits, rasterizations, and
allocated bytes. DevTools records the same values in GPU traces.

## GPU canvases

`Element::gpu_canvas` composes application-authored WGPU output as a retained
offscreen texture. It participates in normal display order, transforms, clips,
rounded corners, opacity and effects while Argui continues to own the device,
submission, surface and presentation. An explicit content revision prevents
unchanged or paused canvases from rerunning their callback.

Use it for bounded editor, visualization, map, game or scientific viewports,
not for text and controls that ordinary Argui primitives already render and
make accessible. Registration, memory, alpha, capability, diagnostics and
native/WebAssembly contracts are in the [GPU canvas guide](gpu-canvas.md).
