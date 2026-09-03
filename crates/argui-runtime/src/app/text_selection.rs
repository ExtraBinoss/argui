use std::time::Duration;

use argui_core::{Point, PointerEvent, PointerId, PointerKind, PointerPhase};
use argui_layout::{LayoutOutput, TextRegion};
use argui_ui::{DocumentTextPoint, SelectionGranularity};
use web_time::Instant;
use winit::{event_loop::ActiveEventLoop, window::Window};

use super::Application;

const MULTI_CLICK_INTERVAL: Duration = Duration::from_millis(500);
const MULTI_CLICK_RADIUS_SQUARED: f32 = 25.0;
const LONG_PRESS_INTERVAL: Duration = Duration::from_millis(500);
const TOUCH_SLOP_SQUARED: f32 = 100.0;

#[derive(Clone, Copy, Debug)]
pub(super) struct TouchSelection {
    id: PointerId,
    origin: Point,
    started: Instant,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct SelectionClick {
    at: Instant,
    point: Point,
    count: u8,
}

impl Default for SelectionClick {
    fn default() -> Self {
        Self {
            at: Instant::now() - MULTI_CLICK_INTERVAL,
            point: Point::default(),
            count: 0,
        }
    }
}

impl SelectionClick {
    pub(super) fn next(&mut self, point: Point) -> SelectionGranularity {
        let delta = Point::new(point.x - self.point.x, point.y - self.point.y);
        if self.at.elapsed() <= MULTI_CLICK_INTERVAL
            && delta.x * delta.x + delta.y * delta.y <= MULTI_CLICK_RADIUS_SQUARED
        {
            self.count = self.count % 3 + 1;
        } else {
            self.count = 1;
        }
        self.at = Instant::now();
        self.point = point;
        match self.count {
            2 => SelectionGranularity::Word,
            3 => SelectionGranularity::Line,
            _ => SelectionGranularity::Character,
        }
    }
}

impl Application {
    pub(super) fn static_text_at(layout: &LayoutOutput, point: Point) -> Option<DocumentTextPoint> {
        layout
            .text_regions
            .iter()
            .filter_map(|region| {
                region
                    .hit_position(point)
                    .map(|position| (region.interaction_order, position))
            })
            .max_by_key(|(order, _)| *order)
            .map(|(_, point)| point)
    }

    pub(super) fn closest_static_text(
        layout: &LayoutOutput,
        point: Point,
    ) -> Option<DocumentTextPoint> {
        layout
            .text_regions
            .iter()
            .min_by(|left, right| {
                left.distance_squared(point)
                    .total_cmp(&right.distance_squared(point))
                    .then_with(|| {
                        left.interaction_order
                            .cmp(&right.interaction_order)
                            .reverse()
                    })
            })
            .map(|region| region.closest_position(point))
    }

    pub(super) fn begin_touch_selection(&mut self, id: PointerId, point: Point, window: &Window) {
        if self
            .ui_layout
            .as_ref()
            .and_then(|layout| Self::static_text_at(layout, point))
            .is_some()
        {
            self.touch_selection = Some(TouchSelection {
                id,
                origin: point,
                started: Instant::now(),
            });
            window.request_redraw();
        }
    }

    pub(super) fn move_touch_selection_candidate(&mut self, id: PointerId, point: Point) {
        let Some(candidate) = self.touch_selection.filter(|candidate| candidate.id == id) else {
            return;
        };
        let delta = Point::new(point.x - candidate.origin.x, point.y - candidate.origin.y);
        if delta.x * delta.x + delta.y * delta.y > TOUCH_SLOP_SQUARED {
            self.touch_selection = None;
        }
    }

    pub(super) fn cancel_touch_selection(&mut self, id: PointerId) {
        if self
            .touch_selection
            .is_some_and(|candidate| candidate.id == id)
        {
            self.touch_selection = None;
        }
    }

    pub(super) fn advance_touch_selection(
        &mut self,
        window: &Window,
        event_loop: &ActiveEventLoop,
    ) {
        let Some(candidate) = self.touch_selection else {
            return;
        };
        if candidate.started.elapsed() < LONG_PRESS_INTERVAL {
            window.request_redraw();
            return;
        }
        self.touch_selection = None;
        let Some(point) = self
            .ui_layout
            .as_ref()
            .and_then(|layout| Self::static_text_at(layout, candidate.origin))
        else {
            return;
        };
        if let Some(ui) = &mut self.ui_tree {
            let mut update = ui.pointer_event(
                PointerEvent {
                    id: candidate.id,
                    kind: PointerKind::Touch,
                    phase: PointerPhase::Cancelled,
                    position: candidate.origin,
                    button: None,
                    buttons: 0,
                    pressure: None,
                    primary: true,
                    timestamp: self.input_epoch.elapsed(),
                },
                self.ui_layout
                    .as_ref()
                    .map_or(&[][..], |layout| layout.hit_regions.as_slice()),
            );
            update.merge(ui.begin_touch_document_selection(point, SelectionGranularity::Word));
            self.apply_ui_update(update, window, event_loop);
        }
    }
}

pub(super) fn text_region_at(regions: &[TextRegion], point: Point) -> bool {
    regions
        .iter()
        .any(|region| region.hit_position(point).is_some())
}

#[cfg(test)]
mod tests {
    use argui_core::Size;
    use argui_layout::LayoutEngine;
    use argui_text::TextEngine;
    use argui_ui::{Element, UiTree, percent};

    use super::*;

    const NOTO_SANS: &[u8] =
        include_bytes!("../../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");

    fn text_layout() -> LayoutOutput {
        let mut ui = UiTree::new(
            Element::column([Element::text("first"), Element::text("second")]).width(percent(1.0)),
        );
        let mut text =
            TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans");
        LayoutEngine::new()
            .compute(&mut ui, &mut text, Size::new(240.0, 120.0))
            .unwrap()
    }

    #[test]
    fn nearby_clicks_cycle_character_word_and_line() {
        let mut clicks = SelectionClick::default();
        let point = Point::new(20.0, 20.0);
        assert_eq!(clicks.next(point), SelectionGranularity::Character);
        assert_eq!(clicks.next(point), SelectionGranularity::Word);
        assert_eq!(clicks.next(point), SelectionGranularity::Line);
        assert_eq!(clicks.next(point), SelectionGranularity::Character);
        assert_eq!(
            clicks.next(Point::new(200.0, 20.0)),
            SelectionGranularity::Character
        );
    }

    #[test]
    fn shaped_static_text_hit_testing_prefers_visual_order_and_nearest_lines() {
        let mut layout = text_layout();
        assert_eq!(layout.text_regions.len(), 2);
        let first = layout.text_regions[0].clone();
        let second = layout.text_regions[1].clone();
        let first_point = Point::new(first.origin.x + 2.0, first.origin.y + 2.0);
        let second_point = Point::new(second.origin.x + 2.0, second.origin.y + 2.0);

        assert_eq!(
            Application::static_text_at(&layout, first_point)
                .unwrap()
                .node,
            first.node
        );
        assert_eq!(
            Application::static_text_at(&layout, second_point)
                .unwrap()
                .node,
            second.node
        );
        assert!(Application::static_text_at(&layout, Point::new(230.0, 110.0)).is_none());
        assert!(text_region_at(&layout.text_regions, first_point));
        assert!(!text_region_at(
            &layout.text_regions,
            Point::new(230.0, 110.0)
        ));

        assert_eq!(
            Application::closest_static_text(&layout, Point::new(-20.0, first.origin.y))
                .unwrap()
                .node,
            first.node
        );
        assert_eq!(
            Application::closest_static_text(&layout, Point::new(260.0, second.origin.y))
                .unwrap()
                .node,
            second.node
        );

        let mut front = first;
        front.interaction_order = usize::MAX;
        layout.text_regions.push(front.clone());
        assert_eq!(
            Application::static_text_at(&layout, first_point)
                .unwrap()
                .node,
            front.node
        );
        assert!(Application::closest_static_text(&LayoutOutput::default(), first_point).is_none());
    }
}
