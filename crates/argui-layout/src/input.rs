use argui_core::{Affine2D, Color, Point, Rect, TextPosition};
use argui_paint::{Border, ClipChain, DisplayList, Fill, Quad};
use argui_text::{CaretStop, TextEngine};
use argui_ui::{Element, ElementKind, NodeId, UiTree};

#[derive(Clone, Debug, PartialEq)]
pub struct TextInputRegion {
    pub node: NodeId,
    pub bounds: Rect,
    pub clip: Rect,
    pub stops: Vec<CaretStop>,
    pub selection: Vec<Rect>,
    pub caret: Option<Rect>,
    pub selection_color: Color,
    pub caret_color: Color,
    pub scroll_x: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct InputPlacement {
    pub text: Rect,
    pub hit: Rect,
    pub clip: Rect,
    pub scroll_x: f32,
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
        self.stops
            .iter()
            .min_by(|a, b| distance(a.point, point).total_cmp(&distance(b.point, point)))
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
                let direction = if left {
                    stop.point.x < current.point.x
                } else {
                    stop.point.x > current.point.x
                };
                direction && (!by_word || stop.word_boundary)
            })
            .min_by(|a, b| {
                (a.point.x - current.point.x)
                    .abs()
                    .total_cmp(&(b.point.x - current.point.x).abs())
            })
            .map_or(position, |stop| stop.position)
    }

    pub(crate) fn translate(&mut self, delta: Point, clip: Rect) {
        self.bounds.origin = add(self.bounds.origin, delta);
        self.clip = clip;
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
}

pub(crate) fn prepare(
    ui: &UiTree,
    node: NodeId,
    element: &Element,
    engine: &mut TextEngine,
    placement: InputPlacement,
) -> Option<(TextInputRegion, f32)> {
    let ElementKind::TextInput {
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
        placement.scroll_x,
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
            clip: placement.clip,
            stops,
            selection: selection_rects,
            caret: (ui.focused_node() == Some(node)).then_some(caret_rect),
            selection_color: *selection,
            caret_color: *caret,
            scroll_x: layout.scroll_x,
        },
        layout.scroll_x,
    ))
}

pub(crate) fn update(ui: &UiTree, engine: &mut TextEngine, output: &mut crate::LayoutOutput) {
    let elements = crate::engine::flattened(ui.root());
    for node in &output.nodes {
        let Some(text_index) = node.text_index else {
            continue;
        };
        let element = elements[node.index];
        if !matches!(element.kind, ElementKind::TextInput { .. }) {
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
        let block = &mut output.text.blocks_mut()[text_index];
        let text_bounds = Rect::new(
            Point::new(
                block.bounds.origin.x + previous_scroll_x,
                block.bounds.origin.y,
            ),
            block.bounds.size,
        );
        if let Some((region, scroll_x)) = prepare(
            ui,
            node.node,
            element,
            engine,
            InputPlacement {
                text: text_bounds,
                hit: node.bounds,
                clip: block.clip,
                scroll_x: previous_scroll_x,
            },
        ) {
            block.bounds.origin.x = text_bounds.origin.x - scroll_x;
            output.text_inputs[region_index] = region;
        }
    }
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
    region: &TextInputRegion,
    output: &mut DisplayList,
    transform: Affine2D,
    clips: &ClipChain,
) {
    if let Some(bounds) = region.caret {
        push_quad(output, bounds, region.caret_color, transform, clips);
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

fn distance(a: Point, b: Point) -> f32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

const fn add(a: Point, b: Point) -> Point {
    Point::new(a.x + b.x, a.y + b.y)
}
