# Color

Argui has one color contract on native WGPU and WebGPU. Application colors are
authored in sRGB, converted once to extended linear sRGB, and remain linear
through painting and effects. An sRGB render target performs the final display
encoding. Display-P3, HDR, and ICC profiles are not part of this contract.

Use `Color::srgb`, `Color::srgba`, the 8-bit constructors, or `Color::from_hex`
for visual colors. `Color::linear_rgb` and `Color::linear_rgba` are explicit
low-level constructors for renderer math and shader data. `to_srgba` is for
serialization and editing; `to_linear_rgba` is for GPU uploads.

Primitive shaders output straight linear RGBA and normal WGPU blending creates
premultiplied linear intermediate textures. Built-in compositing, blur, shadows,
and blend modes preserve that representation. Custom `argui_effect` functions
receive and return straight linear RGBA; the generated ABI converts to and from
the renderer's premultiplied intermediate representation.

Gradients declare `ColorInterpolation::Oklab`, `LinearSrgb`, or `Srgb` at
construction. Stops are interpolated with premultiplied alpha to avoid dark
fringes, then converted to linear sRGB before rendering. Widget state colors and
typed color keyframes use OKLab interpolation by default.

Decoded image pixels and rasterized SVG atlases use sRGB textures. Glyph atlases
remain linear coverage masks and receive their linear text color in the shader.
