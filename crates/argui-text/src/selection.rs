use argui_core::{Point, Rect, Size, TextPosition};
use cosmic_text::Buffer;
use unicode_segmentation::UnicodeSegmentation;

use crate::{CaretStop, TextContent, TextEngine, TextStyle, cache::TextLayoutKey, engine};

#[derive(Clone, Debug, PartialEq)]
pub struct TextLineLayout {
    pub source: std::ops::Range<usize>,
    pub bounds: Rect,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextLayout {
    pub stops: Vec<CaretStop>,
    pub lines: Vec<TextLineLayout>,
    pub content_size: Size,
}

impl TextLayout {
    #[must_use]
    pub fn hit_position(&self, point: Point) -> Option<TextPosition> {
        self.lines
            .iter()
            .find(|line| line.bounds.contains(point))
            .and_then(|line| closest_on_line(&self.stops, line, point.x))
    }

    #[must_use]
    pub fn closest_position(&self, point: Point) -> TextPosition {
        let Some(line) = self.lines.iter().min_by(|left, right| {
            distance_to_axis(left.bounds, point.y)
                .total_cmp(&distance_to_axis(right.bounds, point.y))
        }) else {
            return TextPosition::default();
        };
        closest_on_line(&self.stops, line, point.x).unwrap_or_default()
    }

    #[must_use]
    pub fn selection_rects(&self, anchor: TextPosition, focus: TextPosition) -> Vec<Rect> {
        let (start, end) = ordered(anchor.index, focus.index);
        if start == end {
            return Vec::new();
        }
        self.lines
            .iter()
            .filter_map(|line| {
                let selected = self
                    .stops
                    .iter()
                    .filter(|stop| {
                        line.source.contains(&stop.position.index)
                            || stop.position.index == line.source.end
                    })
                    .filter(|stop| (stop.point.y - line.bounds.origin.y).abs() < 0.01)
                    .filter(|stop| stop.position.index >= start && stop.position.index <= end);
                let (left, right) = selected
                    .fold((f32::INFINITY, f32::NEG_INFINITY), |range, stop| {
                        (range.0.min(stop.point.x), range.1.max(stop.point.x))
                    });
                left.is_finite()
                    .then(|| {
                        Rect::new(
                            Point::new(left, line.bounds.origin.y),
                            Size::new((right - left).max(0.0), line.bounds.size.height),
                        )
                    })
                    .filter(|rect| rect.size.width > 0.0)
            })
            .collect()
    }
}

impl TextEngine {
    #[must_use]
    pub fn layout_text(
        &mut self,
        content: &TextContent,
        style: &TextStyle,
        viewport: Size,
    ) -> std::sync::Arc<TextLayout> {
        let key = TextLayoutKey::new(content, style, viewport);
        if let Some(layout) = self.cache.layout(&key) {
            return layout;
        }
        let mut buffer = Buffer::new(
            &mut self.fonts,
            cosmic_text::Metrics::new(style.font_size, style.line_height),
        );
        engine::configure_buffer(
            &mut buffer,
            style,
            Some(viewport.width),
            Some(viewport.height),
        );
        engine::set_content(&mut buffer, content, style);
        buffer.shape_until_scroll(&mut self.fonts, false);
        let layout = std::sync::Arc::new(from_buffer(&buffer, content.as_str(), style.line_height));
        self.cache.insert_layout(key, layout)
    }
}

#[must_use]
pub fn word_range(text: &str, index: usize) -> std::ops::Range<usize> {
    let index = boundary(text, index);
    text.split_word_bound_indices()
        .find_map(|(start, word)| {
            let end = start + word.len();
            (start <= index && index < end && word.chars().any(char::is_alphanumeric))
                .then_some(start..end)
        })
        .unwrap_or(index..index)
}

#[must_use]
pub fn line_range(text: &str, index: usize) -> std::ops::Range<usize> {
    let index = boundary(text, index);
    let start = text[..index].rfind('\n').map_or(0, |offset| offset + 1);
    let end = text[index..]
        .find('\n')
        .map_or(text.len(), |offset| index + offset);
    start..end
}

fn from_buffer(buffer: &Buffer, text: &str, fallback_height: f32) -> TextLayout {
    let offsets = engine::source_line_offsets(text);
    let stops = crate::input::caret_stops(buffer, text);
    let mut lines = Vec::new();
    let mut content_size = Size::new(0.0, fallback_height);
    for run in buffer.layout_runs() {
        let base = offsets.get(run.line_i).copied().unwrap_or_default();
        let start = run
            .glyphs
            .iter()
            .map(|glyph| base + glyph.start)
            .min()
            .unwrap_or(base);
        let end = run
            .glyphs
            .iter()
            .map(|glyph| base + glyph.end)
            .max()
            .unwrap_or(base)
            .min(text.len());
        let left = run
            .glyphs
            .iter()
            .map(|glyph| glyph.x)
            .min_by(f32::total_cmp)
            .unwrap_or_default();
        let right = run
            .glyphs
            .iter()
            .map(|glyph| glyph.x + glyph.w)
            .max_by(f32::total_cmp)
            .unwrap_or(left);
        let bounds = Rect::new(
            Point::new(left, run.line_top),
            Size::new((right - left).max(0.0), run.line_height.max(0.0)),
        );
        lines.push(TextLineLayout {
            source: start..end,
            bounds,
        });
        content_size.width = content_size.width.max(run.line_w);
        content_size.height = content_size.height.max(run.line_top + run.line_height);
    }
    TextLayout {
        stops,
        lines,
        content_size,
    }
}

fn closest_on_line(stops: &[CaretStop], line: &TextLineLayout, x: f32) -> Option<TextPosition> {
    stops
        .iter()
        .filter(|stop| {
            line.source.contains(&stop.position.index) || stop.position.index == line.source.end
        })
        .filter(|stop| (stop.point.y - line.bounds.origin.y).abs() < 0.01)
        .min_by(|left, right| {
            (left.point.x - x)
                .abs()
                .total_cmp(&(right.point.x - x).abs())
        })
        .map(|stop| stop.position)
}

fn distance_to_axis(rect: Rect, value: f32) -> f32 {
    if value < rect.origin.y {
        rect.origin.y - value
    } else if value > rect.origin.y + rect.size.height {
        value - rect.origin.y - rect.size.height
    } else {
        0.0
    }
}

fn ordered(left: usize, right: usize) -> (usize, usize) {
    if left <= right {
        (left, right)
    } else {
        (right, left)
    }
}

fn boundary(text: &str, index: usize) -> usize {
    let index = index.min(text.len());
    (0..=index)
        .rev()
        .find(|candidate| text.is_char_boundary(*candidate))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_ranges_obey_unicode_and_lines() {
        let text = "hello café\nsecond";
        assert_eq!(word_range(text, 8), 6..11);
        assert_eq!(line_range(text, 8), 0..11);
        assert_eq!(line_range(text, text.len()), 12..18);
    }
}
