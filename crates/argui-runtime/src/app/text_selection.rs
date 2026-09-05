use argui_core::{Point, PointerEvent, PointerId, PointerKind, PointerPhase};
use argui_layout::{LayoutOutput, TextRegion};
use argui_ui::{DocumentTextPoint, SelectionGranularity};
use web_time::Instant;
use winit::{event_loop::ActiveEventLoop, window::Window};

use super::Application;

#[derive(Clone, Copy, Debug)]
pub(super) struct TouchSelection {
    id: PointerId,
    origin: Point,
    started: Instant,
}

impl TouchSelection {
    fn tracks(self, id: PointerId) -> bool {
        self.id == id
    }

    fn moved_beyond(self, point: Point, slop: f32) -> bool {
        let delta = Point::new(point.x - self.origin.x, point.y - self.origin.y);
        delta.x * delta.x + delta.y * delta.y > slop * slop
    }

    fn ready(self, interval: std::time::Duration) -> bool {
        self.started.elapsed() >= interval
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct SelectionClick {
    at: Option<Instant>,
    point: Point,
    count: u8,
}

impl SelectionClick {
    pub(super) fn next(
        &mut self,
        point: Point,
        settings: argui_core::PointerSettings,
    ) -> SelectionGranularity {
        let delta = Point::new(point.x - self.point.x, point.y - self.point.y);
        let radius = settings.multi_click_distance();
        if self
            .at
            .is_some_and(|at| at.elapsed() <= settings.multi_click_interval())
            && delta.x * delta.x + delta.y * delta.y <= radius * radius
        {
            self.count = self.count % 3 + 1;
        } else {
            self.count = 1;
        }
        self.at = Some(Instant::now());
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
        self.touch_selection = touch_candidate(self.ui_layout.as_ref(), id, point);
        if self.touch_selection.is_some() {
            window.request_redraw();
        }
    }

    pub(super) fn move_touch_selection_candidate(&mut self, id: PointerId, point: Point) {
        move_touch_candidate(
            &mut self.touch_selection,
            id,
            point,
            self.pointer_settings.touch_slop(),
        );
    }

    pub(super) fn cancel_touch_selection(&mut self, id: PointerId) {
        cancel_touch_candidate(&mut self.touch_selection, id);
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn advance_touch_selection(
        &mut self,
        window: &Window,
        event_loop: &ActiveEventLoop,
    ) {
        let Some(candidate) = self.touch_selection else {
            return;
        };
        if !candidate.ready(self.pointer_settings.long_press_interval()) {
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
                    modifiers: self.modifiers,
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

fn touch_candidate(
    layout: Option<&LayoutOutput>,
    id: PointerId,
    point: Point,
) -> Option<TouchSelection> {
    layout
        .and_then(|layout| Application::static_text_at(layout, point))
        .map(|_| TouchSelection {
            id,
            origin: point,
            started: Instant::now(),
        })
}

fn move_touch_candidate(
    candidate: &mut Option<TouchSelection>,
    id: PointerId,
    point: Point,
    slop: f32,
) {
    if candidate
        .filter(|candidate| candidate.tracks(id))
        .is_some_and(|candidate| candidate.moved_beyond(point, slop))
    {
        *candidate = None;
    }
}

fn cancel_touch_candidate(candidate: &mut Option<TouchSelection>, id: PointerId) {
    if candidate.is_some_and(|candidate| candidate.tracks(id)) {
        *candidate = None;
    }
}

pub(super) fn text_region_at(regions: &[TextRegion], point: Point) -> bool {
    regions
        .iter()
        .any(|region| region.hit_position(point).is_some())
}
