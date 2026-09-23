use argui_core::Color;

use crate::{
    PreparedDecoration, PreparedGlyph, PreparedText, TextBlock, TextSpanStyle,
    cache::{CachedDecoration, CachedShape},
    layout::{physical_clip, physical_decoration},
};

/// Appends cached logical `shape` for `block` at `block_index`, resolving paint at `scale`.
pub(crate) fn append_shape(
    prepared: &mut PreparedText,
    block_index: usize,
    block: &TextBlock,
    scale: f32,
    shape: &CachedShape,
) {
    let clip = physical_clip(block.clip, scale);
    prepared.glyphs.extend(shape.glyphs.iter().map(|glyph| {
        let mut value = PreparedGlyph {
            key: glyph.key,
            block: block_index,
            start: glyph.start,
            end: glyph.end,
            rtl: glyph.rtl,
            x: 0,
            y: 0,
            color: text_color(block, glyph.span).to_linear_rgba(),
            clip,
            local: glyph.local,
            physical: glyph.local.physical(block.bounds.origin, scale),
        };
        value.resolve_physical();
        value
    }));
    prepared
        .decorations
        .extend(shape.decorations.iter().map(|decoration| {
            let style = span(block, decoration.span)
                .and_then(|style| style.decoration)
                .unwrap_or(block.style.decoration);
            let color = if decoration.strikethrough {
                style.strikethrough_color
            } else {
                style.underline_color
            }
            .unwrap_or_else(|| text_color(block, decoration.span));
            PreparedDecoration {
                block: block_index,
                rect: physical_decoration(decoration.local, block.bounds.origin, scale),
                color: color.to_linear_rgba(),
                local: decoration.local,
            }
        }));
}

/// Returns the rich span selected by `index` for `block`, or `None` for base styling.
fn span(block: &TextBlock, index: usize) -> Option<&TextSpanStyle> {
    index
        .checked_sub(1)
        .and_then(|index| block.content.runs().get(index).map(|(_, style)| style))
}

/// Resolves full-precision text color for `block`'s span `index`.
fn text_color(block: &TextBlock, index: usize) -> Color {
    span(block, index)
        .and_then(|style| style.color)
        .unwrap_or(block.style.color)
}

/// Retains logical decoration geometry from `run`, split at rich-span paint boundaries.
pub(crate) fn collect_decorations(
    items: &mut Vec<CachedDecoration>,
    run: &cosmic_text::LayoutRun<'_>,
) {
    for decoration in run.decorations {
        let glyphs = &run.glyphs[decoration.glyph_range.clone()];
        let mut start = 0;
        while let Some(first) = glyphs.get(start) {
            let count = glyphs[start..]
                .iter()
                .take_while(|glyph| glyph.metadata == first.metadata)
                .count();
            let group = &glyphs[start..start + count];
            let left = group
                .iter()
                .map(|glyph| glyph.x)
                .fold(f32::INFINITY, f32::min);
            let right = group
                .iter()
                .map(|glyph| glyph.x + glyph.w)
                .fold(f32::NEG_INFINITY, f32::max);
            if right > left {
                decoration_rects(
                    items,
                    decoration,
                    run.line_y,
                    first.metadata,
                    left,
                    right - left,
                );
            }
            start += count;
        }
    }
}

/// Adds underline/strike rectangles from `decoration` at `baseline` for paint `span` and extent.
fn decoration_rects(
    items: &mut Vec<CachedDecoration>,
    decoration: &cosmic_text::DecorationSpan,
    baseline: f32,
    span: usize,
    x: f32,
    width: f32,
) {
    let data = &decoration.data;
    let size = decoration.font_size;
    let thickness = (data.underline_metrics.thickness * size).max(f32::EPSILON);
    let underline = match data.text_decoration.underline {
        cosmic_text::UnderlineStyle::None => 0,
        cosmic_text::UnderlineStyle::Single => 1,
        cosmic_text::UnderlineStyle::Double => 2,
    };
    for line in 0..underline {
        items.push(CachedDecoration {
            local: [
                x,
                baseline - data.underline_metrics.offset * size
                    + line as f32 * 2.0 * thickness.max(1.0),
                width,
                thickness,
            ],
            span,
            strikethrough: false,
        });
    }
    if data.text_decoration.strikethrough {
        items.push(CachedDecoration {
            local: [
                x,
                baseline - data.strikethrough_metrics.offset * size,
                width,
                data.strikethrough_metrics.thickness * size,
            ],
            span,
            strikethrough: true,
        });
    }
}
