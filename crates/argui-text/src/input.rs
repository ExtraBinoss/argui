use std::collections::{HashMap, HashSet};

use argui_core::{CaretAffinity, Point, Rect, Size, TextPosition};
use cosmic_text::{Buffer, Scroll};
use unicode_segmentation::UnicodeSegmentation;

use crate::{TextContent, TextEngine, TextStyle, engine};

mod buffer;
pub(crate) use buffer::InputBuffer;
use buffer::{VIRTUAL_INPUT_MIN_BYTES, unwrapped_width_estimate, visible_window};

const INPUT_BUFFER_CACHE_CAPACITY: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaretStop {
    pub position: TextPosition,
    pub point: Point,
    pub word_boundary: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextInputLayout {
    pub caret: Rect,
    pub selection: Vec<Rect>,
    pub stops: Vec<CaretStop>,
    pub content_size: Size,
    pub scroll_x: f32,
    pub scroll_y: f32,
}

/// Source and vertical geometry retained for a virtualized text-input window.
#[derive(Clone, Debug, PartialEq)]
pub struct TextInputWindow {
    /// UTF-8 byte range shaped for the current viewport and its overscan.
    pub byte_range: std::ops::Range<usize>,
    /// Content-space horizontal origin of the shaped source range.
    pub x: f32,
    /// Content-space vertical origin of the shaped source range.
    pub y: f32,
    /// Content-space height covered by the shaped source range.
    pub height: f32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CaretScroll {
    Preserve,
    #[default]
    Reveal,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextInputScroll {
    pub offset: Point,
    pub caret: CaretScroll,
}

impl TextInputScroll {
    /// Creates a text-input scroll policy with an offset and caret visibility mode.
    ///
    /// * `offset` — current horizontal and vertical content offset.
    /// * `caret` — whether to reveal the caret or preserve the offset.
    #[must_use]
    pub const fn new(offset: Point, caret: CaretScroll) -> Self {
        Self { offset, caret }
    }
}

#[derive(Clone, Copy)]
struct VisualCluster {
    start: usize,
    end: usize,
    left: f32,
    right: f32,
    rtl: bool,
}

impl TextEngine {
    /// Measures a large non-wrapping editor without shaping its offscreen document.
    ///
    /// * `content` — complete editor content.
    /// * `style` — editor text style.
    /// * `width` — optional containing width used for ordinary wrapped measurement.
    ///
    /// Returns an approximate intrinsic width for large non-wrapping content, its
    /// exact logical line count, and a measured baseline. Smaller or wrapped
    /// content uses ordinary text measurement.
    pub fn measure_editor_content(
        &mut self,
        content: &TextContent,
        style: &TextStyle,
        width: Option<f32>,
    ) -> crate::TextMeasurement {
        if style.wrap != crate::TextWrap::None || content.as_str().len() < VIRTUAL_INPUT_MIN_BYTES {
            return self.measure_content(content, style, width);
        }
        let baseline = self
            .measure_layout("M", style, None)
            .first_baseline
            .unwrap_or(style.line_height * 0.8);
        let lines = content
            .as_str()
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1;
        crate::TextMeasurement {
            size: Size::new(
                unwrapped_width_estimate(content.as_str(), style.font_size),
                lines as f32 * style.line_height,
            ),
            first_baseline: Some(baseline),
            last_baseline: Some(baseline + (lines - 1) as f32 * style.line_height),
        }
    }

    /// Shapes text for editing and computes caret, selection, and scrolling geometry.
    ///
    /// * `text` — displayed editor text.
    /// * `style` — editor text style.
    /// * `viewport` — available input size in logical pixels.
    /// * `cursor` — caret position and affinity in the displayed text.
    /// * `selection` — optional anchor and focus positions.
    /// * `scroll` — current offset and caret reveal policy.
    ///
    /// Returns caret stops, selection rectangles, content size, and resolved offsets.
    pub fn input_layout(
        &mut self,
        text: &str,
        style: &TextStyle,
        viewport: Size,
        cursor: TextPosition,
        selection: Option<(TextPosition, TextPosition)>,
        scroll: TextInputScroll,
    ) -> TextInputLayout {
        self.input_layout_content(
            &TextContent::plain(text),
            style,
            viewport,
            cursor,
            selection,
            scroll,
        )
    }

    /// Shapes rich editor content and computes caret, selection, and scrolling geometry.
    ///
    /// * `content` — displayed editor content with optional per-span styling.
    /// * `style` — base editor text style inherited by spans.
    /// * `viewport` — available input size in logical pixels.
    /// * `cursor` — caret position and affinity in the displayed text.
    /// * `selection` — optional anchor and focus positions.
    /// * `scroll` — current offset and caret reveal policy.
    ///
    /// Returns caret stops, selection rectangles, content size, and resolved offsets.
    pub fn input_layout_content(
        &mut self,
        content: &TextContent,
        style: &TextStyle,
        viewport: Size,
        cursor: TextPosition,
        selection: Option<(TextPosition, TextPosition)>,
        scroll: TextInputScroll,
    ) -> TextInputLayout {
        let text = content.as_str();
        let cursor = TextPosition::new(boundary(text, cursor.index), cursor.affinity);
        let cache_index = self
            .input_buffers
            .iter()
            .position(|entry| entry.matches(content, style, viewport))
            .inspect(|_| self.stats.layout_hits += 1)
            .unwrap_or_else(|| {
                self.stats.layout_misses += 1;
                let entry = InputBuffer::new(
                    &mut self.fonts,
                    content,
                    style,
                    viewport,
                    cursor,
                    scroll,
                    self.input_buffers.last(),
                );
                if text.len() >= VIRTUAL_INPUT_MIN_BYTES {
                    self.input_buffers.clear();
                }
                if self.input_buffers.len() == INPUT_BUFFER_CACHE_CAPACITY {
                    self.input_buffers.remove(0);
                }
                self.input_buffers.push(entry);
                self.input_buffers.len() - 1
            });
        let no_wrap = style.wrap == crate::TextWrap::None;
        let entry = &mut self.input_buffers[cache_index];
        let content_height = if no_wrap {
            entry.line_offsets.len().max(1) as f32 * style.line_height
        } else {
            0.0
        };
        let cursor_line = entry
            .line_offsets
            .partition_point(|offset| *offset <= cursor.index)
            .saturating_sub(1);
        let cursor_y = cursor_line as f32 * style.line_height;
        let scroll_y = no_wrap.then(|| {
            resolved_scroll(
                scroll.caret,
                scroll.offset.y,
                cursor_y,
                style.line_height,
                viewport.height,
                content_height,
            )
        });
        if let Some(scroll_y) = scroll_y {
            entry.ensure_window(&mut self.fonts, scroll_y, cursor, scroll);
        }
        let line_offsets = &entry.line_offsets;
        let buffer = &mut entry.buffer;
        if let Some(scroll_y) = scroll_y {
            let line = (scroll_y / style.line_height.max(1.0)).floor() as usize;
            let first_line = entry
                .window
                .as_ref()
                .map_or(0, |window| (window.y / style.line_height) as usize);
            let line = line
                .min(line_offsets.len().saturating_sub(1))
                .saturating_sub(first_line);
            let vertical = scroll_y - (line + first_line) as f32 * style.line_height;
            buffer.set_scroll(Scroll::new(line, vertical, 0.0));
            buffer.shape_until_scroll(&mut self.fonts, false);
        }
        let window_y = entry.window.as_ref().map_or(0.0, |window| window.y);
        let y_offset = scroll_y.map_or(0.0, |_| {
            let shaped = buffer.scroll();
            window_y + shaped.line as f32 * style.line_height + shaped.vertical
        });
        let first_byte = entry
            .window
            .as_ref()
            .map_or(0, |window| window.byte_range.start);
        let mut window_offsets = engine::source_line_offsets(
            &text[first_byte
                ..entry
                    .window
                    .as_ref()
                    .map_or(text.len(), |window| window.byte_range.end)],
        );
        for offset in &mut window_offsets {
            *offset += first_byte;
        }
        let mut raw_stops = caret_stops(buffer, &window_offsets);
        if no_wrap {
            for stop in &mut raw_stops {
                stop.point.x += entry.window.as_ref().map_or(0.0, |window| window.x);
                stop.point.y += y_offset;
            }
        }
        let caret_stop = raw_stops
            .iter()
            .find(|stop| stop.position == cursor)
            .or_else(|| {
                raw_stops
                    .iter()
                    .find(|stop| stop.position.index == cursor.index)
            });
        let (caret_x, caret_y) =
            caret_stop.map_or((0.0, cursor_y), |stop| (stop.point.x, stop.point.y));
        let shaped_width = buffer
            .layout_runs()
            .map(|run| run.line_w)
            .fold(0.0_f32, f32::max)
            + entry.window.as_ref().map_or(0.0, |window| window.x);
        let content_width = if no_wrap
            && (style.family == crate::FontFamily::Monospace
                || (entry.line_offsets.len() == 1 && entry.window.is_some()))
        {
            shaped_width.max(entry.unwrapped_width)
        } else {
            shaped_width
        };
        let content_height = if no_wrap {
            content_height
        } else {
            buffer
                .layout_runs()
                .map(|run| run.line_top + run.line_height)
                .fold(style.line_height, f32::max)
        };
        let scroll_x = resolved_scroll(
            scroll.caret,
            scroll.offset.x,
            caret_x,
            2.0,
            viewport.width,
            content_width,
        );
        let scroll_y = scroll_y.unwrap_or_else(|| {
            resolved_scroll(
                scroll.caret,
                scroll.offset.y,
                caret_y,
                style.line_height,
                viewport.height,
                content_height,
            )
        });
        let caret = Rect::new(
            Point::new(caret_x - scroll_x, caret_y - scroll_y),
            Size::new(1.5, style.line_height),
        );
        let selection = selection.map_or_else(Vec::new, |(anchor, focus)| {
            visual_selection_rects(
                buffer,
                &raw_stops,
                anchor,
                focus,
                Point::new(scroll_x, scroll_y),
                y_offset,
            )
        });
        let stops = raw_stops
            .into_iter()
            .map(|mut stop| {
                stop.point.x -= scroll_x;
                stop.point.y -= scroll_y;
                stop
            })
            .collect();
        TextInputLayout {
            caret,
            selection,
            stops,
            content_size: Size::new(content_width, content_height),
            scroll_x,
            scroll_y,
        }
    }

    /// Returns the cached source window covering a non-wrapping input viewport.
    ///
    /// * `content` — editor content previously passed to [`Self::input_layout_content`].
    /// * `style` — matching editor text style.
    /// * `viewport` — matching input viewport size.
    /// * `scroll_y` — resolved vertical content offset.
    ///
    /// Returns `None` for wrapped or short content and when no matching shaped buffer exists.
    #[must_use]
    pub fn input_window(
        &self,
        content: &TextContent,
        style: &TextStyle,
        viewport: Size,
        scroll_y: f32,
    ) -> Option<TextInputWindow> {
        if style.wrap != crate::TextWrap::None {
            return None;
        }
        let entry = self
            .input_buffers
            .iter()
            .find(|entry| entry.matches(content, style, viewport))?;
        if entry.line_offsets.len() == 1 && entry.window.is_some() {
            entry.window.clone()
        } else {
            (content.as_str().len() >= VIRTUAL_INPUT_MIN_BYTES)
                .then(|| {
                    visible_window(
                        &entry.line_offsets,
                        content.as_str().len(),
                        style.line_height,
                        viewport.height,
                        scroll_y,
                    )
                })
                .flatten()
        }
    }
}

pub(crate) fn caret_stops(buffer: &Buffer, line_offsets: &[usize]) -> Vec<CaretStop> {
    let mut stops = Vec::new();
    let mut seen = HashSet::new();
    for run in buffer.layout_runs() {
        let base = line_offsets.get(run.line_i).copied().unwrap_or_default();
        let boundaries = run
            .text
            .unicode_word_indices()
            .flat_map(|(index, word)| [index, index + word.len()])
            .collect::<Vec<_>>();
        let visuals = visual_clusters(&run);
        if visuals.is_empty() {
            push_stop(
                &mut stops,
                &mut seen,
                TextPosition::new(base, CaretAffinity::Before),
                0.0,
                run.line_top,
                true,
            );
        }
        for visual in visuals {
            let cluster = &run.text[visual.start..visual.end];
            let graphemes = cluster.grapheme_indices(true).collect::<Vec<_>>();
            let count = graphemes.len().max(1) as f32;
            for (part, (offset, _)) in graphemes.iter().enumerate() {
                let progress = part as f32 / count;
                let x = if visual.rtl {
                    visual.right - (visual.right - visual.left) * progress
                } else {
                    visual.left + (visual.right - visual.left) * progress
                };
                push_stop(
                    &mut stops,
                    &mut seen,
                    TextPosition::new(base + visual.start + offset, CaretAffinity::After),
                    x,
                    run.line_top,
                    is_word_boundary(&boundaries, visual.start + offset, run.text.len()),
                );
            }
            let (position, x) = if visual.rtl {
                (
                    TextPosition::new(base + visual.end, CaretAffinity::Before),
                    visual.left,
                )
            } else {
                (
                    TextPosition::new(base + visual.end, CaretAffinity::Before),
                    visual.right,
                )
            };
            push_stop(
                &mut stops,
                &mut seen,
                position,
                x,
                run.line_top,
                is_word_boundary(&boundaries, visual.end, run.text.len()),
            );
        }
    }
    if stops.is_empty() {
        stops.push(CaretStop {
            position: TextPosition::default(),
            point: Point::default(),
            word_boundary: true,
        });
    }
    stops.sort_by(|a, b| {
        a.point
            .y
            .total_cmp(&b.point.y)
            .then_with(|| a.point.x.total_cmp(&b.point.x))
    });
    stops
}

fn is_word_boundary(boundaries: &[usize], index: usize, line_len: usize) -> bool {
    index == 0 || index == line_len || boundaries.binary_search(&index).is_ok()
}

fn visual_clusters(run: &cosmic_text::LayoutRun<'_>) -> Vec<VisualCluster> {
    let mut clusters: HashMap<(usize, usize), VisualCluster> = HashMap::new();
    for glyph in run.glyphs {
        if let Some(cluster) = clusters.get_mut(&(glyph.start, glyph.end)) {
            cluster.left = cluster.left.min(glyph.x);
            cluster.right = cluster.right.max(glyph.x + glyph.w);
        } else {
            clusters.insert(
                (glyph.start, glyph.end),
                VisualCluster {
                    start: glyph.start,
                    end: glyph.end,
                    left: glyph.x,
                    right: glyph.x + glyph.w,
                    rtl: glyph.level.is_rtl(),
                },
            );
        }
    }
    let mut clusters = clusters.into_values().collect::<Vec<_>>();
    clusters.sort_by(|a, b| a.left.total_cmp(&b.left));
    clusters
}

fn push_stop(
    stops: &mut Vec<CaretStop>,
    seen: &mut HashSet<(usize, bool)>,
    position: TextPosition,
    x: f32,
    y: f32,
    word_boundary: bool,
) {
    let key = (position.index, position.affinity == CaretAffinity::After);
    if !seen.insert(key) {
        return;
    }
    stops.push(CaretStop {
        position,
        point: Point::new(x, y),
        word_boundary,
    });
}

fn visible_scroll(previous: f32, caret: f32, extent: f32, viewport: f32, content: f32) -> f32 {
    let maximum = (content - viewport).max(0.0);
    let previous = previous.clamp(0.0, maximum);
    if caret < previous {
        caret.clamp(0.0, maximum)
    } else if caret + extent > previous + viewport {
        (caret - viewport + extent).clamp(0.0, maximum)
    } else {
        previous
    }
}

fn resolved_scroll(
    policy: CaretScroll,
    previous: f32,
    caret: f32,
    extent: f32,
    viewport: f32,
    content: f32,
) -> f32 {
    let maximum = (content - viewport).max(0.0);
    match policy {
        CaretScroll::Preserve => previous.clamp(0.0, maximum),
        CaretScroll::Reveal => visible_scroll(previous, caret, extent, viewport, content),
    }
}

fn visual_selection_rects(
    buffer: &Buffer,
    stops: &[CaretStop],
    anchor: TextPosition,
    focus: TextPosition,
    scroll: Point,
    y_offset: f32,
) -> Vec<Rect> {
    let (start, end) = if anchor.index <= focus.index {
        (anchor.index, focus.index)
    } else {
        (focus.index, anchor.index)
    };
    buffer
        .layout_runs()
        .filter_map(|run| {
            let line_top = run.line_top + y_offset;
            let selected = stops
                .iter()
                .filter(|stop| {
                    (stop.point.y - line_top).abs() < 0.01
                        && stop.position.index >= start
                        && stop.position.index <= end
                })
                .collect::<Vec<_>>();
            let left = selected
                .iter()
                .map(|stop| stop.point.x)
                .min_by(f32::total_cmp)?;
            let right = selected
                .iter()
                .map(|stop| stop.point.x)
                .max_by(f32::total_cmp)?;
            (right > left).then(|| {
                Rect::new(
                    Point::new(left - scroll.x, line_top - scroll.y),
                    Size::new(right - left, run.line_height),
                )
            })
        })
        .collect()
}

fn boundary(text: &str, index: usize) -> usize {
    let index = index.min(text.len());
    if text.is_char_boundary(index) {
        index
    } else {
        (0..index)
            .rev()
            .find(|candidate| text.is_char_boundary(*candidate))
            .unwrap_or(0)
    }
}
