use argui_core::{Size, TextPosition};
use cosmic_text::{Buffer, Metrics};

use super::{CaretScroll, TextInputScroll, TextInputWindow, resolved_scroll};
use crate::{TextContent, TextStyle, engine};

pub(super) const VIRTUAL_INPUT_MIN_BYTES: usize = 16 * 1024;
const INPUT_WINDOW_OVERSCAN: usize = 8;

pub(crate) struct InputBuffer {
    pub(super) content: TextContent,
    pub(super) style: TextStyle,
    pub(super) viewport: Size,
    pub(super) line_offsets: Vec<usize>,
    pub(super) unwrapped_width: f32,
    max_columns: usize,
    last_columns: usize,
    pub(super) window: Option<TextInputWindow>,
    pub(super) buffer: Buffer,
}

/// Selects hard lines around a non-wrapping editor's vertical viewport.
///
/// * `offsets` — byte offset of each source line start.
/// * `text_len` — full source length in UTF-8 bytes.
/// * `line_height` — logical height of one unwrapped line.
/// * `viewport_height` — visible editor height.
/// * `scroll_y` — resolved vertical content offset.
///
/// Returns `None` when the document is already small enough to shape whole.
pub(super) fn visible_window(
    offsets: &[usize],
    text_len: usize,
    line_height: f32,
    viewport_height: f32,
    scroll_y: f32,
) -> Option<TextInputWindow> {
    if line_height <= 0.0 {
        return None;
    }
    let visible_lines = (viewport_height / line_height).ceil().max(1.0) as usize;
    if offsets.len() <= visible_lines + 4 {
        return None;
    }
    let visible_start = (scroll_y / line_height).floor() as usize;
    let anchor = visible_start / visible_lines * visible_lines;
    let first = anchor.saturating_sub(INPUT_WINDOW_OVERSCAN);
    let end_line = anchor
        .saturating_add(visible_lines.saturating_mul(2))
        .saturating_add(INPUT_WINDOW_OVERSCAN)
        .min(offsets.len());
    Some(TextInputWindow {
        byte_range: offsets[first]..offsets.get(end_line).copied().unwrap_or(text_len),
        x: 0.0,
        y: first as f32 * line_height,
        height: (end_line - first) as f32 * line_height,
    })
}

/// Selects a bounded source range for a very long single-line editor.
///
/// * `text` — full UTF-8 editor value.
/// * `style` — font size used for the content-space width estimate.
/// * `viewport` — visible editor dimensions.
/// * `cursor` — caret byte position.
/// * `scroll` — current offset and caret visibility policy.
///
/// Returns a source range and estimated content-space origin around the viewport.
fn horizontal_window(
    text: &str,
    style: &TextStyle,
    viewport: Size,
    cursor: TextPosition,
    scroll: TextInputScroll,
) -> TextInputWindow {
    let advance = (style.font_size * 0.62).max(1.0);
    let visible = (viewport.width / advance).ceil().max(1.0) as usize;
    let ascii = text.is_ascii();
    let first = if ascii {
        match scroll.caret {
            CaretScroll::Reveal => cursor.index.saturating_sub(visible.saturating_mul(2)),
            CaretScroll::Preserve => {
                ((scroll.offset.x / advance).floor() as usize).saturating_sub(visible)
            }
        }
    } else {
        match scroll.caret {
            CaretScroll::Reveal => text[..cursor.index]
                .char_indices()
                .rev()
                .nth(visible.saturating_mul(2))
                .map_or(0, |(index, _)| index),
            CaretScroll::Preserve => text
                .char_indices()
                .nth(((scroll.offset.x / advance).floor() as usize).saturating_sub(visible))
                .map_or(text.len(), |(index, _)| index),
        }
    };
    let end = if ascii {
        first
            .saturating_add(visible.saturating_mul(4))
            .min(text.len())
    } else {
        text[first..]
            .char_indices()
            .nth(visible.saturating_mul(4))
            .map_or(text.len(), |(index, _)| first + index)
    };
    let columns_before = if ascii {
        first
    } else {
        column_counts(&text[..first]).0
    };
    TextInputWindow {
        byte_range: first..end,
        x: columns_before as f32 * advance,
        y: 0.0,
        height: style.line_height,
    }
}

/// Estimates the widest unwrapped line from its logical character columns.
///
/// * `text` — complete editor text.
/// * `font_size` — font size used for the approximate column advance.
///
/// Returns an approximate content width in logical pixels.
pub(super) fn unwrapped_width_estimate(text: &str, font_size: f32) -> f32 {
    column_counts(text).0 as f32 * font_size * 0.62
}

/// Counts the widest logical line and final-line columns for width estimation.
///
/// * `text` — complete editor text.
///
/// Returns the maximum columns on any line and the columns on the final line.
fn column_counts(text: &str) -> (usize, usize) {
    let mut maximum = 0;
    let mut current = 0;
    for character in text.chars() {
        if character == '\n' {
            maximum = maximum.max(current);
            current = 0;
        } else {
            current += if character == '\t' { 4 } else { 1 };
        }
    }
    (maximum.max(current), current)
}

impl InputBuffer {
    /// Builds a cached editor buffer around the requested viewport.
    ///
    /// * `fonts` — font system used to shape the selected source range.
    /// * `content` — complete editor content retained for cache identity.
    /// * `style` — layout style for shaping and line geometry.
    /// * `viewport` — visible editor size.
    /// * `cursor` — requested caret position.
    /// * `scroll` — previous offset and caret reveal policy.
    /// * `previous` — prior buffer whose line and width data may be reused after an append.
    ///
    /// Returns the shaped buffer and full-document metadata for the editor.
    pub(super) fn new(
        fonts: &mut cosmic_text::FontSystem,
        content: &TextContent,
        style: &TextStyle,
        viewport: Size,
        cursor: TextPosition,
        scroll: TextInputScroll,
        previous: Option<&Self>,
    ) -> Self {
        let text = content.as_str();
        let appended = previous.filter(|previous| {
            previous.viewport == viewport
                && crate::cache::same_style_layout(&previous.style, style)
                && text.starts_with(previous.content.as_str())
        });
        let line_offsets = if let Some(previous) = appended {
            let mut offsets = previous.line_offsets.clone();
            let base = previous.content.as_str().len();
            offsets.extend(
                text[base..]
                    .char_indices()
                    .filter_map(|(index, character)| {
                        (character == '\n').then_some(base + index + 1)
                    }),
            );
            offsets
        } else {
            engine::source_line_offsets(text)
        };
        let cursor_line = line_offsets
            .partition_point(|offset| *offset <= cursor.index)
            .saturating_sub(1);
        let content_height = line_offsets.len().max(1) as f32 * style.line_height;
        let scroll_y = resolved_scroll(
            scroll.caret,
            scroll.offset.y,
            cursor_line as f32 * style.line_height,
            style.line_height,
            viewport.height,
            content_height,
        );
        let window = input_window_for(
            content.as_str(),
            style,
            viewport,
            &line_offsets,
            scroll_y,
            cursor,
            scroll,
        );
        let buffer = make_input_buffer(fonts, content, style, viewport, window.as_ref());
        let (max_columns, last_columns) = if let Some(previous) = appended {
            let suffix = &text[previous.content.as_str().len()..];
            let (suffix_max, suffix_last) = column_counts(suffix);
            let first_columns = suffix
                .split('\n')
                .next()
                .map_or(0, |line| column_counts(line).0);
            let joined_max = previous.last_columns + first_columns;
            (
                previous.max_columns.max(joined_max).max(suffix_max),
                if suffix.contains('\n') {
                    suffix_last
                } else {
                    previous.last_columns + suffix_last
                },
            )
        } else {
            column_counts(text)
        };
        let unwrapped_width = max_columns as f32 * style.font_size * 0.62;
        Self {
            content: content.clone(),
            style: style.clone(),
            viewport,
            line_offsets,
            unwrapped_width,
            max_columns,
            last_columns,
            window,
            buffer,
        }
    }

    /// Rebuilds the shaped source slice when the viewport leaves its retained line window.
    ///
    /// * `fonts` — font system used to shape a new visible slice.
    /// * `scroll_y` — resolved vertical content offset.
    /// * `cursor` — current caret position.
    /// * `scroll` — previous offset and caret visibility policy.
    pub(super) fn ensure_window(
        &mut self,
        fonts: &mut cosmic_text::FontSystem,
        scroll_y: f32,
        cursor: TextPosition,
        scroll: TextInputScroll,
    ) {
        let next = input_window_for(
            self.content.as_str(),
            &self.style,
            self.viewport,
            &self.line_offsets,
            scroll_y,
            cursor,
            scroll,
        );
        if self.window != next {
            self.buffer = make_input_buffer(
                fonts,
                &self.content,
                &self.style,
                self.viewport,
                next.as_ref(),
            );
            self.window = next;
        }
    }

    /// Checks whether an editor can reuse this cached layout, ignoring paint changes.
    ///
    /// * `content` — requested editor content.
    /// * `style` — requested text style.
    /// * `viewport` — requested editor viewport size.
    ///
    /// Returns `true` when the cached layout matches all three inputs.
    pub(super) fn matches(&self, content: &TextContent, style: &TextStyle, viewport: Size) -> bool {
        self.viewport == viewport
            && crate::cache::same_style_layout(&self.style, style)
            && crate::cache::same_content_layout(&self.content, content)
    }
}

/// Chooses a vertical line slice, or a horizontal ASCII slice for a long line.
///
/// * `text` — full editor value.
/// * `style` — editor layout style.
/// * `viewport` — available editor size.
/// * `offsets` — full-document line starts.
/// * `scroll_y` — resolved vertical content offset.
/// * `cursor` — current caret position.
/// * `scroll` — horizontal offset and caret visibility policy.
///
/// Returns a bounded source window when the large editor can be virtualized.
fn input_window_for(
    text: &str,
    style: &TextStyle,
    viewport: Size,
    offsets: &[usize],
    scroll_y: f32,
    cursor: TextPosition,
    scroll: TextInputScroll,
) -> Option<TextInputWindow> {
    if style.wrap != crate::TextWrap::None || text.len() < VIRTUAL_INPUT_MIN_BYTES {
        return None;
    }
    visible_window(
        offsets,
        text.len(),
        style.line_height,
        viewport.height,
        scroll_y,
    )
    .or_else(|| {
        (offsets.len() == 1).then(|| horizontal_window(text, style, viewport, cursor, scroll))
    })
}

/// Shapes the entire editor or only its selected visible line window.
///
/// * `fonts` — font system used by Cosmic Text.
/// * `content` — complete editor content.
/// * `style` — editor text style.
/// * `viewport` — available editor size.
/// * `window` — optional source range for non-wrapping virtualized content.
///
/// Returns a shaped Cosmic Text buffer for the chosen source range.
fn make_input_buffer(
    fonts: &mut cosmic_text::FontSystem,
    content: &TextContent,
    style: &TextStyle,
    viewport: Size,
    window: Option<&TextInputWindow>,
) -> Buffer {
    let metrics = Metrics::new(style.font_size, style.line_height);
    let mut buffer = Buffer::new(fonts, metrics);
    buffer.set_size(
        Some(viewport.width),
        (style.wrap == crate::TextWrap::None).then_some(viewport.height),
    );
    buffer.set_wrap(match style.wrap {
        crate::TextWrap::None => cosmic_text::Wrap::None,
        crate::TextWrap::Glyph => cosmic_text::Wrap::Glyph,
        crate::TextWrap::Word => cosmic_text::Wrap::Word,
        crate::TextWrap::WordOrGlyph => cosmic_text::Wrap::WordOrGlyph,
    });
    if let Some(window) = window {
        let visible = content.slice(window.byte_range.clone());
        engine::set_content(&mut buffer, &visible, style);
    } else {
        engine::set_content(&mut buffer, content, style);
    }
    buffer.shape_until_scroll(fonts, false);
    buffer
}
