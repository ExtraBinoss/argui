use argui_core::{Affine2D, Point, Rect};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GlyphKey(pub(crate) cosmic_text::CacheKey, pub(crate) usize);

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LogicalGlyph {
    pub(crate) font_size: f32,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) baseline: f32,
}

impl LogicalGlyph {
    /// Retains unquantized geometry from `glyph` and its logical line `baseline`.
    pub(crate) fn new(glyph: &cosmic_text::LayoutGlyph, baseline: f32) -> Self {
        Self {
            font_size: glyph.font_size,
            x: glyph.x + glyph.font_size * glyph.x_offset,
            y: glyph.y - glyph.font_size * glyph.y_offset,
            baseline,
        }
    }

    /// Converts this geometry at logical `origin` and pixel `scale` without rounding.
    pub(crate) fn physical(self, origin: Point, scale: f32) -> [f32; 3] {
        [
            self.x.mul_add(scale, origin.x * scale),
            self.y
                .mul_add(scale, self.baseline * scale + origin.y * scale),
            self.font_size * scale,
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreparedGlyph {
    pub key: GlyphKey,
    pub block: usize,
    pub start: usize,
    pub end: usize,
    pub rtl: bool,
    pub x: i32,
    pub y: i32,
    pub color: [f32; 4],
    pub clip: [f32; 4],
    pub(crate) local: LogicalGlyph,
    pub(crate) physical: [f32; 3],
}

impl PreparedGlyph {
    /// Applies physical-space `transform` before raster quantization.
    ///
    /// Returns a glyph rasterized at the transformed size and subpixel position for
    /// finite, positive, uniform axis-aligned scaling and translation. Returns `None`
    /// for other transforms or non-finite resulting positions/sizes. Glyph color and
    /// source metadata are preserved. The clip stays unchanged: the renderer must
    /// apply its independently transformed clip chain.
    #[must_use]
    pub fn transform_for_raster(mut self, transform: Affine2D) -> Option<Self> {
        let [sx, skew_y, skew_x, sy] = transform.matrix;
        if !sx.is_finite() || sx <= 0.0 || sx != sy || skew_x != 0.0 || skew_y != 0.0 {
            return None;
        }
        self.physical = [
            self.physical[0] * sx + transform.translation.x,
            self.physical[1] * sx + transform.translation.y,
            self.physical[2] * sx,
        ];
        if !self.physical.iter().all(|value| value.is_finite()) {
            return None;
        }
        self.resolve_physical();
        Some(self)
    }

    /// Rebuilds the raster key and integer placement from retained physical geometry.
    pub(crate) fn resolve_physical(&mut self) {
        let key = self.key.0;
        let (key, x, y) = cosmic_text::CacheKey::new(
            key.font_id,
            key.glyph_id,
            self.physical[2],
            (self.physical[0], self.physical[1].trunc()),
            key.font_weight,
            key.flags,
        );
        self.key = GlyphKey(key, self.key.1);
        self.x = x;
        self.y = y;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreparedDecoration {
    pub block: usize,
    pub rect: [f32; 4],
    pub color: [f32; 4],
    pub(crate) local: [f32; 4],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PreparedText {
    pub blocks: usize,
    pub glyphs: Vec<PreparedGlyph>,
    pub decorations: Vec<PreparedDecoration>,
    pub(crate) scale_factor: f32,
}

impl PreparedText {
    /// Repositions one block and recomputes its physical glyph keys without shaping.
    ///
    /// * `block` — block index used when the text was prepared.
    /// * `origin` — new logical origin, including its fractional position.
    /// * `clip` — new logical clipping rectangle.
    pub fn reposition_block(&mut self, block: usize, origin: Point, clip: Rect) {
        let clip = physical_clip(clip, self.scale_factor);
        for glyph in self.glyphs.iter_mut().filter(|glyph| glyph.block == block) {
            glyph.physical = glyph.local.physical(origin, self.scale_factor);
            glyph.resolve_physical();
            glyph.clip = clip;
        }
        for decoration in self
            .decorations
            .iter_mut()
            .filter(|item| item.block == block)
        {
            decoration.rect = physical_decoration(decoration.local, origin, self.scale_factor);
        }
    }
}

/// Converts logical `clip` to physical left, top, right, bottom using `scale`.
pub(crate) fn physical_clip(clip: Rect, scale: f32) -> [f32; 4] {
    [
        clip.origin.x * scale,
        clip.origin.y * scale,
        (clip.origin.x + clip.size.width) * scale,
        (clip.origin.y + clip.size.height) * scale,
    ]
}

/// Positions logical decoration `rect` at `origin` and snaps its physical edges at `scale`.
pub(crate) fn physical_decoration(rect: [f32; 4], origin: Point, scale: f32) -> [f32; 4] {
    [
        ((origin.x + rect[0]) * scale).round(),
        ((origin.y + rect[1]) * scale).round(),
        (rect[2] * scale).ceil(),
        (rect[3] * scale).ceil().max(1.0),
    ]
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GlyphContent {
    Mask,
    Color,
    SubpixelMask,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlyphImage {
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
    pub content: GlyphContent,
    /// Mask coverage, RGBA subpixel coverage, or straight-alpha sRGB color pixels.
    pub data: Vec<u8>,
}
