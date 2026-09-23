use argui_text::{GlyphContent, GlyphImage};

/// Returns `image` pixels surrounded by one transparent texel on every side.
/// Masks use one channel, including a grayscale reduction of RGBA subpixel masks;
/// color images retain their sRGB RGBA bytes for hardware decoding when sampled.
/// Rewriting the border prevents stale pixels from bleeding after page eviction.
///
/// # Panics
/// Panics if the rasterizer supplied fewer bytes than `image` dimensions require.
pub(super) fn padded_pixels(image: &GlyphImage) -> Vec<u8> {
    let channels = if image.content == GlyphContent::Color {
        4
    } else {
        1
    };
    let width = image.width as usize;
    let height = image.height as usize;
    let stride = (width + 2) * channels;
    let mut pixels = vec![0; stride * (height + 2)];
    for y in 0..height {
        let target = &mut pixels[(y + 1) * stride + channels..][..width * channels];
        match image.content {
            GlyphContent::Color | GlyphContent::Mask => {
                let start = y * width * channels;
                target.copy_from_slice(&image.data[start..start + width * channels]);
            }
            GlyphContent::SubpixelMask => {
                // Swash's subpixel format is RGBA, not packed RGB.
                for (x, value) in target.iter_mut().enumerate() {
                    let start = (y * width + x) * 4;
                    *value = image.data[start..start + 3]
                        .iter()
                        .copied()
                        .max()
                        .unwrap_or(0);
                }
            }
        }
    }
    pixels
}
