use argui_core::{Affine2D, Color, Point, Rect, TextPosition};
use argui_paint::{Border, ClipChain, DisplayList, Fill, Quad, QuadStyle};
use argui_text::{CaretScroll, CaretStop, TextEngine, TextInputScroll};
use argui_ui::{CaretStyle, Element, ElementKind, NodeId, UiTree};

mod navigation;

impl crate::LayoutOutput {
    /// Convert a viewport pointer to layout coordinates for editor hits and captured drags.
    #[must_use]
    pub fn local_point(&self, node: NodeId, point: Point) -> Option<Point> {
        self.hit_regions
            .iter()
            .find(|region| region.node == node)
            .and_then(|region| region.transform.inverse())
            .map(|inverse| inverse.transform_point(point))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextInputRegion {
    pub node: NodeId,
    pub bounds: Rect,
    pub viewport: Rect,
    pub clip: Rect,
    pub stops: Vec<CaretStop>,
    pub selection: Vec<Rect>,
    pub caret: Option<Rect>,
    pub selection_color: Color,
    pub caret_style: CaretStyle,
    pub content_size: argui_core::Size,
    pub scroll_x: f32,
    pub scroll_y: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct InputPlacement {
    pub text: Rect,
    pub hit: Rect,
    pub clip: Rect,
    pub scroll_x: f32,
    pub scroll_y: f32,
}

impl TextInputRegion {
    #[must_use]
    pub fn hit_position(&self, point: Point) -> Option<TextPosition> {
        (self.bounds.contains(point) && self.clip.contains(point))
            .then(|| self.closest_position(point))
    }

    #[must_use]
    pub fn hit_index(&self, point: Point) -> Option<usize> {
        self.hit_position(point).map(|position| position.index)
    }

    #[must_use]
    pub fn closest_position(&self, point: Point) -> TextPosition {
        let Some(line_y) = self
            .stops
            .iter()
            .filter(|stop| stop.point.y <= point.y)
            .map(|stop| stop.point.y)
            .max_by(f32::total_cmp)
            .or_else(|| {
                self.stops
                    .iter()
                    .map(|stop| stop.point.y)
                    .min_by(f32::total_cmp)
            })
        else {
            return TextPosition::default();
        };
        self.stops
            .iter()
            .filter(|stop| (stop.point.y - line_y).abs() < 0.01)
            .min_by(|a, b| {
                (a.point.x - point.x)
                    .abs()
                    .total_cmp(&(b.point.x - point.x).abs())
            })
            .map_or(TextPosition::default(), |stop| stop.position)
    }

    #[must_use]
    pub fn closest_index(&self, point: Point) -> usize {
        self.closest_position(point).index
    }

    #[must_use]
    pub fn visual_neighbor(
        &self,
        position: TextPosition,
        left: bool,
        by_word: bool,
    ) -> TextPosition {
        let current = self
            .stops
            .iter()
            .find(|stop| stop.position == position)
            .or_else(|| {
                let caret_x = self.caret?.origin.x;
                self.stops
                    .iter()
                    .filter(|stop| stop.position.index == position.index)
                    .min_by(|a, b| {
                        (a.point.x - caret_x)
                            .abs()
                            .total_cmp(&(b.point.x - caret_x).abs())
                    })
            });
        let Some(current) = current else {
            return position;
        };
        self.stops
            .iter()
            .filter(|stop| {
                let same_line = (stop.point.y - current.point.y).abs() < 0.01;
                let direction = if left {
                    stop.point.x < current.point.x
                } else {
                    stop.point.x > current.point.x
                };
                same_line && direction && (!by_word || stop.word_boundary)
            })
            .min_by(|a, b| {
                (a.point.x - current.point.x)
                    .abs()
                    .total_cmp(&(b.point.x - current.point.x).abs())
            })
            .map_or_else(
                || self.adjacent_line_edge(position, left),
                |stop| stop.position,
            )
    }

    pub(crate) fn translate(&mut self, delta: Point, clip: Rect) {
        self.bounds.origin = add(self.bounds.origin, delta);
        self.viewport.origin = add(self.viewport.origin, delta);
        self.clip = clip;
        self.translate_content(delta);
    }

    pub(crate) fn scroll_to(&mut self, offset: Point) {
        let delta = Point::new(self.scroll_x - offset.x, self.scroll_y - offset.y);
        self.scroll_x = offset.x;
        self.scroll_y = offset.y;
        self.translate_content(delta);
    }

    fn translate_content(&mut self, delta: Point) {
        for stop in &mut self.stops {
            stop.point = add(stop.point, delta);
        }
        for rect in &mut self.selection {
            rect.origin = add(rect.origin, delta);
        }
        if let Some(caret) = &mut self.caret {
            caret.origin = add(caret.origin, delta);
        }
    }

    pub(crate) fn scroll_content_size(&self) -> argui_core::Size {
        argui_core::Size::new(
            self.content_size.width + self.bounds.size.width - self.viewport.size.width,
            self.content_size.height + self.bounds.size.height - self.viewport.size.height,
        )
    }
}

pub(crate) fn prepare(
    ui: &UiTree,
    node: NodeId,
    element: &Element,
    engine: &mut TextEngine,
    placement: InputPlacement,
) -> Option<(TextInputRegion, Point)> {
    let ElementKind::TextEditor {
        text,
        selection,
        caret,
        ..
    } = &element.kind
    else {
        return None;
    };
    let value = ui.text_input_display(node)?;
    let cursor = ui.text_input_position(node)?;
    let layout = engine.input_layout(
        &value,
        text,
        placement.text.size,
        cursor,
        ui.text_input_selection_positions(node),
        TextInputScroll::new(
            Point::new(placement.scroll_x, placement.scroll_y),
            if ui.text_input_should_reveal_cursor(node) {
                CaretScroll::Reveal
            } else {
                CaretScroll::Preserve
            },
        ),
    );
    let origin = placement.text.origin;
    let stops = layout
        .stops
        .into_iter()
        .map(|stop| CaretStop {
            position: stop.position,
            point: add(stop.point, origin),
            word_boundary: stop.word_boundary,
        })
        .collect();
    let selection_rects = layout
        .selection
        .into_iter()
        .map(|mut rect| {
            rect.origin = add(rect.origin, origin);
            rect
        })
        .collect();
    let mut caret_rect = layout.caret;
    caret_rect.origin = add(caret_rect.origin, origin);
    Some((
        TextInputRegion {
            node,
            bounds: placement.hit,
            viewport: placement.text,
            clip: placement.clip,
            stops,
            selection: selection_rects,
            caret: (ui.focused_node() == Some(node)).then_some(caret_rect),
            selection_color: *selection,
            caret_style: caret.clone(),
            content_size: layout.content_size,
            scroll_x: layout.scroll_x,
            scroll_y: layout.scroll_y,
        },
        Point::new(layout.scroll_x, layout.scroll_y),
    ))
}

pub(crate) fn update(ui: &mut UiTree, engine: &mut TextEngine, output: &mut crate::LayoutOutput) {
    let elements = crate::engine::flattened(ui.root());
    let mut offsets = Vec::new();
    for node in &output.nodes {
        let Some(text_index) = node.text_index else {
            continue;
        };
        let element = elements[node.index];
        if !matches!(element.kind, ElementKind::TextEditor { .. }) {
            continue;
        }
        let Some(region_index) = output
            .text_inputs
            .iter()
            .position(|region| region.node == node.node)
        else {
            continue;
        };
        let previous_scroll_x = output.text_inputs[region_index].scroll_x;
        let previous_scroll_y = output.text_inputs[region_index].scroll_y;
        let viewport_size = output.text_inputs[region_index].viewport.size;
        let block = &mut output.text.blocks_mut()[text_index];
        let text_bounds = Rect::new(
            Point::new(
                block.bounds.origin.x + previous_scroll_x,
                block.bounds.origin.y + previous_scroll_y,
            ),
            viewport_size,
        );
        if let Some((region, scroll)) = prepare(
            ui,
            node.node,
            element,
            engine,
            InputPlacement {
                text: text_bounds,
                hit: node.bounds,
                clip: block.clip,
                scroll_x: ui.scroll_offset(node.node).x,
                scroll_y: ui.scroll_offset(node.node).y,
            },
        ) {
            block.bounds.origin.x = text_bounds.origin.x - scroll.x;
            block.bounds.origin.y = text_bounds.origin.y - scroll.y;
            block.bounds.size.height = text_bounds.size.height.max(region.content_size.height);
            if let Some(config) = output
                .scroll_regions
                .iter()
                .find(|current| current.node == node.node)
                .map(|current| current.config.clone())
            {
                let content = region.scroll_content_size();
                let refreshed = crate::scroll::region(
                    node.node,
                    node.bounds,
                    region.clip,
                    content,
                    ui.resolved_scroll_config(node.node, &config),
                    scroll,
                );
                if let Some(current) = output
                    .scroll_regions
                    .iter_mut()
                    .find(|current| current.node == node.node)
                {
                    current.max_offset = refreshed.max_offset;
                    current.config = refreshed.config;
                    current.scrollbar = refreshed.scrollbar;
                }
            }
            offsets.push((node.node, scroll));
            output.text_inputs[region_index] = region;
        }
    }
    drop(elements);
    for (node, offset) in offsets {
        ui.set_scroll_offset(node, offset);
    }
    ui.mark_text_input_layout_clean();
}

pub(crate) fn paint_selection(
    region: &TextInputRegion,
    output: &mut DisplayList,
    transform: Affine2D,
    clips: &ClipChain,
) {
    for bounds in &region.selection {
        push_quad(output, *bounds, region.selection_color, transform, clips);
    }
}

pub(crate) fn paint_caret(
    ui: &UiTree,
    region: &TextInputRegion,
    output: &mut DisplayList,
    transform: Affine2D,
    clips: &ClipChain,
) {
    let Some(line) = region.caret else {
        return;
    };
    let frame = ui.resolved_caret_frame(region.node, &region.caret_style);
    if frame.opacity <= 0.0 {
        return;
    }
    let primitives = &region.caret_style.visual.primitives;
    let Some(bounds) = visual_bounds(line, primitives) else {
        return;
    };
    let transform = transform
        * frame
            .transform
            .affine(bounds, argui_core::TransformOrigin::CENTER);
    for primitive in primitives {
        push_caret_quad(
            output,
            primitive.bounds(line),
            &primitive.paint,
            frame.tint,
            frame.opacity,
            transform,
            clips,
        );
    }
}

fn visual_bounds(line: Rect, primitives: &[argui_ui::CaretPrimitive]) -> Option<Rect> {
    let first = primitives.first()?.bounds(line);
    let (mut left, mut top) = (first.origin.x, first.origin.y);
    let (mut right, mut bottom) = (
        first.origin.x + first.size.width,
        first.origin.y + first.size.height,
    );
    for primitive in &primitives[1..] {
        let bounds = primitive.bounds(line);
        left = left.min(bounds.origin.x);
        top = top.min(bounds.origin.y);
        right = right.max(bounds.origin.x + bounds.size.width);
        bottom = bottom.max(bounds.origin.y + bounds.size.height);
    }
    Some(Rect::new(
        Point::new(left, top),
        argui_core::Size::new(right - left, bottom - top),
    ))
}

fn push_caret_quad(
    output: &mut DisplayList,
    bounds: Rect,
    paint: &QuadStyle,
    tint: Color,
    opacity: f32,
    transform: Affine2D,
    clips: &ClipChain,
) {
    if clips.regions().is_empty() || bounds.size.width <= 0.0 || bounds.size.height <= 0.0 {
        return;
    }
    let background = paint.background.as_ref().map(|fill| match fill {
        Fill::Solid(color) => Fill::Solid(tinted(*color, tint)),
        fill => fill.clone(),
    });
    let border = paint.border.map_or_else(
        || Border::all(0.0, Color::TRANSPARENT),
        |mut border| {
            border.color = tinted(border.color, tint);
            border
        },
    );
    output.push_quad(Quad {
        bounds,
        background,
        border,
        radii: radius_for(bounds, paint.radii),
        opacity: paint.opacity * opacity.clamp(0.0, 1.0),
        transform,
        clips: clips.clone(),
    });
}

fn tinted(color: Color, tint: Color) -> Color {
    let color = color.to_linear_rgba();
    let tint = tint.to_linear_rgba();
    Color::linear_rgba(
        color[0] * tint[0],
        color[1] * tint[1],
        color[2] * tint[2],
        color[3] * tint[3],
    )
}

fn radius_for(bounds: Rect, radii: argui_paint::CornerRadii) -> argui_paint::CornerRadii {
    let maximum = bounds.size.width.min(bounds.size.height) * 0.5;
    argui_paint::CornerRadii {
        top_left: radii.top_left.min(maximum),
        top_right: radii.top_right.min(maximum),
        bottom_right: radii.bottom_right.min(maximum),
        bottom_left: radii.bottom_left.min(maximum),
    }
}

fn push_quad(
    output: &mut DisplayList,
    bounds: Rect,
    color: Color,
    transform: Affine2D,
    clips: &ClipChain,
) {
    if !clips.regions().is_empty() {
        output.push_quad(Quad {
            bounds,
            background: Some(Fill::Solid(color)),
            border: Border::all(0.0, Color::TRANSPARENT),
            radii: Default::default(),
            opacity: 1.0,
            transform,
            clips: clips.clone(),
        });
    }
}

const fn add(a: Point, b: Point) -> Point {
    Point::new(a.x + b.x, a.y + b.y)
}
