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
    pub scroll_x: f32,
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
        previous_scroll_x: f32,
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
        let scroll_x = visible_scroll(previous_scroll_x, caret_x, viewport.width, content_width);
        let caret = Rect::new(
            Point::new(caret_x - scroll_x, caret_y),
            Size::new(1.5, style.line_height),
        );
        let selection = selection
            .and_then(|(anchor, focus)| {
                visual_selection_rect(buffer, &raw_stops, anchor, focus, scroll_x)
            })
            .into_iter()
            .collect();
        let stops = raw_stops
            .into_iter()
            .map(|mut stop| {
                stop.point.x -= scroll_x;
                stop
            })
            .collect();
        TextInputLayout {
            caret,
            selection,
            stops,
            scroll_x,
        }
    }
}

fn caret_stops(buffer: &Buffer, text: &str) -> Vec<CaretStop> {
    let mut boundaries = text
        .unicode_word_indices()
        .flat_map(|(index, word)| [index, index + word.len()])
        .collect::<Vec<_>>();
    boundaries.extend([0, text.len()]);
    let mut stops = Vec::new();
    for run in buffer.layout_runs() {
        for visual in visual_clusters(&run) {
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
                    TextPosition::new(visual.start + offset, CaretAffinity::After),
                    x,
                    run.line_top,
                    &boundaries,
                );
            }
            let (position, x) = if visual.rtl {
                (
                    TextPosition::new(visual.end, CaretAffinity::Before),
                    visual.left,
                )
            } else {
                (
                    TextPosition::new(visual.end, CaretAffinity::Before),
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
    stops.sort_by(|a, b| a.point.x.total_cmp(&b.point.x));
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
        buffer.set_size(Some(viewport.width), Some(viewport.height));
        buffer.set_wrap(cosmic_text::Wrap::None);
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

fn visible_scroll(previous: f32, caret: f32, viewport: f32, content: f32) -> f32 {
    let maximum = (content - viewport).max(0.0);
    let previous = previous.clamp(0.0, maximum);
    if caret < previous {
        caret.clamp(0.0, maximum)
    } else if caret + 2.0 > previous + viewport {
        (caret - viewport + 2.0).clamp(0.0, maximum)
    } else {
        previous
    }
}

fn visual_selection_rect(
    buffer: &Buffer,
    stops: &[CaretStop],
    anchor: TextPosition,
    focus: TextPosition,
    scroll_x: f32,
) -> Option<Rect> {
    let anchor = find_stop(stops, anchor)?;
    let focus = find_stop(stops, focus)?;
    let left = anchor.point.x.min(focus.point.x);
    let right = anchor.point.x.max(focus.point.x);
    let run = buffer.layout_runs().next()?;
    (right > left).then(|| {
        Rect::new(
            Point::new(left - scroll_x, run.line_top),
            Size::new(right - left, run.line_height),
        )
    })
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
