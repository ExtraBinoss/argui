use argui_core::{CaretAffinity, Point, Rect, Size, TextPosition};
use cosmic_text::{Attrs, Buffer, Metrics, Shaping, Weight};
use unicode_segmentation::UnicodeSegmentation;

use crate::{TextEngine, TextStyle, engine};

const INPUT_BUFFER_CACHE_CAPACITY: usize = 8;

pub(crate) struct InputBuffer {
    text: String,
    style: TextStyle,
    viewport: Size,
    buffer: Buffer,
}

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
    pub fn input_layout(
        &mut self,
        text: &str,
        style: &TextStyle,
        viewport: Size,
        cursor: TextPosition,
        selection: Option<(TextPosition, TextPosition)>,
        scroll: TextInputScroll,
    ) -> TextInputLayout {
        let cache_index = self
            .input_buffers
            .iter()
            .position(|entry| entry.matches(text, style, viewport))
            .unwrap_or_else(|| {
                let entry = InputBuffer::new(&mut self.fonts, text, style, viewport);
                if self.input_buffers.len() == INPUT_BUFFER_CACHE_CAPACITY {
                    self.input_buffers.remove(0);
                }
                self.input_buffers.push(entry);
                self.input_buffers.len() - 1
            });
        let buffer = &self.input_buffers[cache_index].buffer;

        let cursor = TextPosition::new(boundary(text, cursor.index), cursor.affinity);
        let raw_stops = caret_stops(buffer, text);
        let caret_stop = raw_stops
            .iter()
            .find(|stop| stop.position == cursor)
            .or_else(|| {
                raw_stops
                    .iter()
                    .find(|stop| stop.position.index == cursor.index)
            });
        let (caret_x, caret_y) = caret_stop
            .map(|stop| (stop.point.x, stop.point.y))
            .unwrap_or_default();
        let content_width = buffer
            .layout_runs()
            .map(|run| run.line_w)
            .fold(0.0_f32, f32::max);
        let content_height = buffer
            .layout_runs()
            .map(|run| run.line_top + run.line_height)
            .fold(style.line_height, f32::max);
        let scroll_x = resolved_scroll(
            scroll.caret,
            scroll.offset.x,
            caret_x,
            2.0,
            viewport.width,
            content_width,
        );
        let scroll_y = resolved_scroll(
            scroll.caret,
            scroll.offset.y,
            caret_y,
            style.line_height,
            viewport.height,
            content_height,
        );
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
}

fn caret_stops(buffer: &Buffer, text: &str) -> Vec<CaretStop> {
    let mut boundaries = text
        .unicode_word_indices()
        .flat_map(|(index, word)| [index, index + word.len()])
        .collect::<Vec<_>>();
    boundaries.extend([0, text.len()]);
    let mut line_offsets = vec![0];
    for (index, character) in text.char_indices() {
        if character == '\n' {
            line_offsets.push(index + 1);
        }
    }
    let mut stops = Vec::new();
    for run in buffer.layout_runs() {
        let base = line_offsets.get(run.line_i).copied().unwrap_or_default();
        let visuals = visual_clusters(&run);
        if visuals.is_empty() {
            push_stop(
                &mut stops,
                TextPosition::new(base, CaretAffinity::Before),
                0.0,
                run.line_top,
                &boundaries,
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
                    TextPosition::new(base + visual.start + offset, CaretAffinity::After),
                    x,
                    run.line_top,
                    &boundaries,
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
            push_stop(&mut stops, position, x, run.line_top, &boundaries);
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

fn visual_clusters(run: &cosmic_text::LayoutRun<'_>) -> Vec<VisualCluster> {
    let mut clusters: Vec<VisualCluster> = Vec::new();
    for glyph in run.glyphs {
        if let Some(cluster) = clusters
            .iter_mut()
            .find(|cluster| cluster.start == glyph.start && cluster.end == glyph.end)
        {
            cluster.left = cluster.left.min(glyph.x);
            cluster.right = cluster.right.max(glyph.x + glyph.w);
        } else {
            clusters.push(VisualCluster {
                start: glyph.start,
                end: glyph.end,
                left: glyph.x,
                right: glyph.x + glyph.w,
                rtl: glyph.level.is_rtl(),
            });
        }
    }
    clusters.sort_by(|a, b| a.left.total_cmp(&b.left));
    clusters
}

fn push_stop(
    stops: &mut Vec<CaretStop>,
    position: TextPosition,
    x: f32,
    y: f32,
    word_boundaries: &[usize],
) {
    if stops
        .iter()
        .any(|stop| stop.position == position && (stop.point.x - x).abs() < 0.01)
    {
        return;
    }
    stops.push(CaretStop {
        position,
        point: Point::new(x, y),
        word_boundary: word_boundaries.contains(&position.index),
    });
}

impl InputBuffer {
    fn new(
        fonts: &mut cosmic_text::FontSystem,
        text: &str,
        style: &TextStyle,
        viewport: Size,
    ) -> Self {
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
        let attrs = Attrs::new()
            .family(engine::family(&style.family))
            .weight(Weight(style.weight));
        buffer.set_text(text, &attrs, Shaping::Advanced, None);
        buffer.shape_until_scroll(fonts, false);
        Self {
            text: text.to_owned(),
            style: style.clone(),
            viewport,
            buffer,
        }
    }

    fn matches(&self, text: &str, style: &TextStyle, viewport: Size) -> bool {
        self.text == text && self.style == *style && self.viewport == viewport
    }
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
) -> Vec<Rect> {
    let (Some(anchor), Some(focus)) = (find_stop(stops, anchor), find_stop(stops, focus)) else {
        return Vec::new();
    };
    let (start, end) = if anchor.position.index <= focus.position.index {
        (anchor.position.index, focus.position.index)
    } else {
        (focus.position.index, anchor.position.index)
    };
    buffer
        .layout_runs()
        .filter_map(|run| {
            let selected = stops
                .iter()
                .filter(|stop| {
                    (stop.point.y - run.line_top).abs() < 0.01
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
                    Point::new(left - scroll.x, run.line_top - scroll.y),
                    Size::new(right - left, run.line_height),
                )
            })
        })
        .collect()
}

fn find_stop(stops: &[CaretStop], position: TextPosition) -> Option<&CaretStop> {
    stops
        .iter()
        .find(|stop| stop.position == position)
        .or_else(|| {
            stops
                .iter()
                .find(|stop| stop.position.index == position.index)
        })
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
