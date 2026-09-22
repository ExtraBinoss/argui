use super::pixels::padded_pixels;
use argui_text::{GlyphContent, GlyphImage};

/// Constructs one row of `data` with `content` and `width` physical pixels.
fn image(content: GlyphContent, width: u32, data: &[u8]) -> GlyphImage {
    GlyphImage {
        left: 0,
        top: 0,
        width,
        height: 1,
        content,
        data: data.to_vec(),
    }
}

/// Checks mask coverage and the freshly cleared border on every side.
#[test]
fn masks_have_fresh_transparent_padding_on_all_four_sides() {
    let pixels = padded_pixels(&image(GlyphContent::Mask, 2, &[255, 128]));
    assert_eq!(pixels, [0, 0, 0, 0, 0, 255, 128, 0, 0, 0, 0, 0]);
}

/// Checks color bytes remain unchanged and transparent padding is initialized.
#[test]
fn color_uploads_preserve_srgb_and_alpha_inside_zeroed_padding() {
    let source = [20, 100, 220, 128, 200, 40, 60, 255];
    let pixels = padded_pixels(&image(GlyphContent::Color, 2, &source));
    assert_eq!(&pixels[20..28], &source);
    assert!(
        pixels[..20]
            .iter()
            .chain(pixels[28..].iter())
            .all(|channel| *channel == 0)
    );
}

/// Checks Swash RGBA subpixel pixels reduce independently to grayscale coverage.
#[test]
fn subpixel_masks_read_rgba_stride_without_mixing_adjacent_pixels() {
    let pixels = padded_pixels(&image(
        GlyphContent::SubpixelMask,
        2,
        &[10, 30, 20, 255, 70, 40, 50, 255],
    ));
    assert_eq!(pixels, [0, 0, 0, 0, 0, 30, 70, 0, 0, 0, 0, 0]);
}
