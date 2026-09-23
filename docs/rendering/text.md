# Text fidelity

Argui shapes logical text with `cosmic-text 0.19` and renders physical glyphs
with `wgpu 30`. Widgets and native primitives use the same renderer. Text layout,
CPU raster images, and GPU atlas residency have independent caches.

## Layout and positioning

`TextEngine` caches logical glyph geometry independently of text color,
opacity, screen position, and monitor scale. Recoloring or moving a label
reuses shaping. Font, content, wrapping, alignment, and metrics changes
select a new logical layout. Editor buffers remain persistent. Mutating the
font system invalidates affected caches, including editor buffers.

Glyph preparation preserves the full physical `cosmic_text::CacheKey`,
including weight, flags, size, and subpixel bins. Fractional scrolling
recomputes the physical key from logical geometry. It never adds a rounded
offset to a raster variant selected for a different origin.

Positive uniform scale and translation transforms are resolved before GPU
atlas lookup. Glyphs are rasterized at the final physical size and position;
the GPU draws their bitmaps without a second scale. Rotation, shear,
reflection, and nonuniform scaling retain the general affine rendering path.
Those transformations can still interpolate a bitmap. Decorations and clips
keep their existing affine geometry.

Retained transform animations reuse layer pixels while moving and request a
final paint when they settle, even if another animation is still active.
Authored transform changes on text subtrees repaint as well. This resolves
the resting text at its final physical size instead of keeping a scaled layer.

The renderer uses Cosmic's horizontal subpixel bins and vertical hinting
policy. It does not introduce a second subpixel grid. It also keeps Argui's
backdrop-aware coverage correction for consistent stroke weight over opaque
light, dark, and colored surfaces. Paint colors retain their floating-point
precision rather than passing through an 8-bit shaping color.

## Raster and atlas residency

The CPU raster cache keeps at most 1,024 results and 8 MiB of bitmap data.
Missing glyph images are cached too. Swash runs on cache misses; images can
be uploaded again after GPU eviction without another rasterization.

Each renderer owns two bounded texture arrays:

| Content | Format | Pages | Maximum texture memory |
| --- | --- | --- | --- |
| Monochrome glyph coverage | `R8Unorm` | 4 × 1024² | 4 MiB |
| Color emoji and color glyphs | `Rgba8UnormSrgb` | 2 × 1024² | 8 MiB |

Mask coverage is linear data. Swash color-outline images are unpremultiplied
before caching; color bitmap images already use straight alpha. Color glyphs are decoded from sRGB when sampled
and blended in the renderer's linear color space. Every uploaded rectangle
contains a transparent one-pixel border, including after page reuse. There
are no mipmaps.

Pages referenced by the current prepared frame are pinned before inserting
new glyphs. Allocation recycles the least recently used unpinned page and
removes its cache entries. A visible working set that exceeds the fixed
budget returns `RendererError::GlyphAtlasFull`; it cannot silently corrupt
glyphs already referenced by the frame. CPU and GPU residency stay bounded.

Instances select their atlas layer without sorting the scene by glyph type
or page, preserving text, decorations, and surrounding UI paint order.
Unchanged instance data is not uploaded again.

## Diagnostics

`TextEngine::stats()` exposes logical layout hits/misses, CPU raster hits/misses,
and CPU bitmap residency. These counters are cumulative; compare snapshots
before and after an operation.

`RenderProfile::text_atlas` exposes current GPU entries and allocated bytes,
plus per-frame hits, raster requests, uploaded bytes, and page evictions.
DevTools shows these in Rendering & memory and includes glyph texture bytes
in its tracked GPU total. Trace export uses `argui-gpu-trace-v5` to preserve
the additional counters.

After warmup, an unchanged scene should require no shaping, rasterization,
or glyph texture uploads. Recoloring should also reuse its glyphs. A
fractional move can need another subpixel variant; a DPI or scale change can
need a new physical size. These misses are expected and do not require new
logical shaping.

## Visual checks

The widget gallery's Typography page includes identical small text on light
and dark surfaces, with ligatures, diacritics, mixed scripts, and muted text.
GPU regression captures exercise scales 1, 1.25, 1.5, and 2, fractional
positions, and repeated rendering. Run graphical checks through the private
display described in [Linux testing](../contributing/linux-testing.md).
Inspect the saved PNGs; successful GPU submission alone is insufficient.

## Design references

The implementation adapts the supplied cosmic-text/WGPU renderer plan to
Argui's existing boundaries. The comparison with Warp used revision
`f4f9b8838f65b106cfafadc0f622f0c72d82beef`:

- [Warp glyph cache](https://github.com/warpdotdev/warp/blob/f4f9b8838f65b106cfafadc0f622f0c72d82beef/crates/warpui/src/rendering/glyph_cache.rs)
  for lazy rasterization and physical/subpixel cache identities.
- [Warp glyph shader](https://github.com/warpdotdev/warp/blob/f4f9b8838f65b106cfafadc0f622f0c72d82beef/crates/warpui/src/rendering/wgpu/shaders/glyph_shader.wgsl)
  for vertical alignment and brightness-dependent coverage handling.
- [Warp Swash rasterizer](https://github.com/warpdotdev/warp/blob/f4f9b8838f65b106cfafadc0f622f0c72d82beef/crates/warpui/src/windowing/winit/fonts/swash_rasterizer.rs)
  for the physical rasterization boundary.

The separate R8 atlas, explicit memory limits, and bounded CPU raster tier
come from the supplied plan; they are not claims about features in Warp's
referenced implementation. Argui retains its own shaping, clipping,
compositing, and backdrop correction rather than replacing them wholesale.
