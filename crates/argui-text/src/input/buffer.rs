mod line_metrics;

pub(super) use line_metrics::LineMetrics;
use std::ops::Range;

use argui_core::{Size, TextPosition};
use cosmic_text::{Buffer, Metrics};

use super::{CaretScroll, TextInputScroll, TextInputWindow, resolved_scroll};
use crate::{TextContent, TextStyle, engine};

pub(super) const VIRTUAL_INPUT_MIN_BYTES: usize = 16 * 1024;
const INPUT_WINDOW_OVERSCAN: usize = 8;

/// Caret state needed to choose a virtualized input window.
struct InputCursorWindow {
    cursor: TextPosition,
    scroll: TextInputScroll,
    ascii: bool,
    columns: usize,
}

pub(crate) struct InputBuffer {
    pub(super) content: TextContent,
    pub(super) style: TextStyle,
    pub(super) viewport: Size,
    pub(super) line_offsets: LineMetrics,
    pub(super) unwrapped_width: f32,
    ascii: bool,
    cursor: usize,
    cursor_columns: usize,
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
    offsets: &LineMetrics,
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
        byte_range: offsets.get(first).unwrap_or(0)..offsets.get(end_line).unwrap_or(text_len),
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
    ascii: bool,
    cursor_columns: usize,
) -> TextInputWindow {
    let advance = (style.font_size * 0.62).max(1.0);
    let visible = (viewport.width / advance).ceil().max(1.0) as usize;
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
    } else if scroll.caret == CaretScroll::Reveal {
        cursor_columns.saturating_sub(column_counts(&text[first..cursor.index]).1)
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

/// Counts the widest unwrapped line without shaping the document.
///
/// * `text` — complete editor text.
///
/// Returns approximate logical columns on its widest hard line.
pub(super) fn unwrapped_columns(text: &str) -> usize {
    column_counts(text).0
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

/// Recognizes an insertion or deletion at the previous caret without scanning
/// the document as Unicode text. Both sides are compared as byte slices.
///
/// * `previous` — cached content and caret before the edit.
/// * `text` — candidate content after the edit.
/// * `cursor` — candidate byte caret after the edit.
///
/// Returns the replaced old range and inserted slice when the change is local.
fn direct_cursor_change<'a>(
    previous: &InputBuffer,
    text: &'a str,
    cursor: usize,
) -> Option<(Range<usize>, &'a str)> {
    let old = previous.content.as_str();
    let old_cursor = previous.cursor;
    if cursor >= old_cursor {
        let added = cursor - old_cursor;
        if text.len() == old.len() + added
            && old[..old_cursor] == text[..old_cursor]
            && old[old_cursor..] == text[cursor..]
        {
            return Some((old_cursor..old_cursor, &text[old_cursor..cursor]));
        }
    } else if old.len() >= text.len() {
        let removed = old.len() - text.len();
        if old_cursor == cursor + removed
            && old[..cursor] == text[..cursor]
            && old[old_cursor..] == text[cursor..]
        {
            return Some((cursor..old_cursor, ""));
        }
    }
    None
}

impl InputBuffer {
    /// Estimates line count and maximum columns after one edit at the prior caret.
    ///
    /// * `content` — newly authored plain text.
    /// * `style` — layout style that must match the retained buffer.
    ///
    /// Returns `None` when a full scan is needed for a complex edit or style change.
    pub(super) fn estimate_edit(
        &self,
        content: &TextContent,
        style: &TextStyle,
    ) -> Option<(usize, usize)> {
        if content.is_rich()
            || self.content.is_rich()
            || !crate::cache::same_style_layout(&self.style, style)
        {
            return None;
        }
        let text = content.as_str();
        let old = self.content.as_str();
        let cursor = if text.len() >= old.len() {
            self.cursor.checked_add(text.len() - old.len())?
        } else {
            self.cursor.checked_sub(old.len() - text.len())?
        };
        let (range, _) = direct_cursor_change(self, text, cursor)?;
        let metrics = self.line_offsets.edited(old, text, range);
        Some((metrics.max_width(), metrics.len()))
    }

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
        let compatible = previous.filter(|previous| {
            previous.viewport == viewport && crate::cache::same_style_layout(&previous.style, style)
        });
        let direct = compatible.and_then(|previous| {
            direct_cursor_change(previous, text, cursor.index).map(|change| (previous, change))
        });
        let appended = compatible
            .filter(|previous| direct.is_none() && text.starts_with(previous.content.as_str()));
        let ascii = appended.map_or_else(
            || {
                direct.as_ref().map_or_else(
                    || text.is_ascii(),
                    |(previous, (_, inserted))| previous.ascii && inserted.is_ascii(),
                )
            },
            |previous| previous.ascii && text[previous.content.as_str().len()..].is_ascii(),
        );
        let line_offsets = if let Some((previous, (range, _))) = &direct {
            previous
                .line_offsets
                .edited(previous.content.as_str(), text, range.clone())
        } else if let Some(previous) = appended {
            let base = previous.content.as_str().len();
            previous
                .line_offsets
                .edited(previous.content.as_str(), text, base..base)
        } else {
            LineMetrics::new(text)
        };
        let cursor_columns = if line_offsets.len() != 1 {
            0
        } else if let Some((previous, (range, inserted))) = &direct {
            if previous.line_offsets.len() == 1 {
                let removed = &previous.content.as_str()[range.start..range.end];
                previous.cursor_columns + column_counts(inserted).1 - column_counts(removed).1
            } else {
                column_counts(&text[..cursor.index]).1
            }
        } else if let Some(previous) = appended {
            if previous.line_offsets.len() == 1 && cursor.index >= previous.cursor {
                previous.cursor_columns + column_counts(&text[previous.cursor..cursor.index]).1
            } else {
                column_counts(&text[..cursor.index]).1
            }
        } else {
            if cursor.index == text.len() {
                line_offsets.width(0)
            } else {
                column_counts(&text[..cursor.index]).1
            }
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
            InputCursorWindow {
                cursor,
                scroll,
                ascii,
                columns: cursor_columns,
            },
        );
        let buffer = make_input_buffer(fonts, content, style, viewport, window.as_ref());
        let max_columns = line_offsets.max_width();
        let unwrapped_width = max_columns as f32 * style.font_size * 0.62;
        Self {
            content: content.clone(),
            style: style.clone(),
            viewport,
            line_offsets,
            unwrapped_width,
            ascii,
            cursor: cursor.index,
            cursor_columns,
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
        self.update_cursor_columns(cursor.index);
        let next = input_window_for(
            self.content.as_str(),
            &self.style,
            self.viewport,
            &self.line_offsets,
            scroll_y,
            InputCursorWindow {
                cursor,
                scroll,
                ascii: self.ascii,
                columns: self.cursor_columns,
            },
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

    /// Updates the logical column before `cursor` using only the traversed slice.
    ///
    /// * `cursor` — byte offset of the next caret position in the unchanged content.
    fn update_cursor_columns(&mut self, cursor: usize) {
        if self.line_offsets.len() != 1 || self.cursor == cursor {
            return;
        }
        let text = self.content.as_str();
        if cursor > self.cursor {
            self.cursor_columns += column_counts(&text[self.cursor..cursor]).1;
        } else {
            self.cursor_columns = self
                .cursor_columns
                .saturating_sub(column_counts(&text[cursor..self.cursor]).1);
        }
        self.cursor = cursor;
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
/// * `caret` — current caret, horizontal offset, and source metrics.
///
/// Returns a bounded source window when the large editor can be virtualized.
fn input_window_for(
    text: &str,
    style: &TextStyle,
    viewport: Size,
    offsets: &LineMetrics,
    scroll_y: f32,
    caret: InputCursorWindow,
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
        (offsets.len() == 1).then(|| {
            horizontal_window(
                text,
                style,
                viewport,
                caret.cursor,
                caret.scroll,
                caret.ascii,
                caret.columns,
            )
        })
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
