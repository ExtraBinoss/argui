use std::time::Duration;

use argui_core::Point;
use argui_ui::{NodeId, WindowDragBehavior};
use winit::window::Window;

use crate::{RuntimeEvent, app::Application};

const DOUBLE_CLICK_TIME: Duration = Duration::from_millis(500);
const DOUBLE_CLICK_DISTANCE_SQUARED: f32 = 16.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WindowDragAction {
    Move,
    ToggleMaximize,
}

#[derive(Clone, Copy, Debug)]
struct DragPress {
    node: NodeId,
    point: Point,
    timestamp: Duration,
}

#[derive(Debug, Default)]
pub(super) struct WindowDragState {
    last: Option<DragPress>,
}

impl WindowDragState {
    fn press(
        &mut self,
        behavior: WindowDragBehavior,
        node: NodeId,
        point: Point,
        timestamp: Duration,
    ) -> WindowDragAction {
        if behavior == WindowDragBehavior::Move {
            self.last = None;
            return WindowDragAction::Move;
        }
        let double_click = self.last.is_some_and(|last| {
            last.node == node
                && timestamp.saturating_sub(last.timestamp) <= DOUBLE_CLICK_TIME
                && squared_distance(last.point, point) <= DOUBLE_CLICK_DISTANCE_SQUARED
        });
        if double_click {
            self.last = None;
            WindowDragAction::ToggleMaximize
        } else {
            self.last = Some(DragPress {
                node,
                point,
                timestamp,
            });
            WindowDragAction::Move
        }
    }
}

impl Application {
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn handle_window_drag(&mut self, window: &Window) -> bool {
        let Some(point) = self.pointer else {
            return false;
        };
        let Some((node, behavior)) = self.ui_layout.as_ref().and_then(|layout| {
            let region = layout
                .hit_regions
                .iter()
                .rev()
                .find(|region| region.contains(point))?;
            region.window_drag.map(|behavior| (region.node, behavior))
        }) else {
            return false;
        };
        let capabilities = argui_platform::window_capabilities(window);
        match self
            .window_drag
            .press(behavior, node, point, self.input_epoch.elapsed())
        {
            WindowDragAction::Move if capabilities.native_drag => {
                if let Err(error) = window.drag_window() {
                    (self.on_event)(RuntimeEvent::CommandFailed(format!(
                        "window drag failed: {error}"
                    )));
                }
            }
            WindowDragAction::Move => (self.on_event)(RuntimeEvent::CommandFailed(
                "window drag is unavailable on this window backend".into(),
            )),
            WindowDragAction::ToggleMaximize if capabilities.maximize => {
                window.set_maximized(!window.is_maximized());
            }
            WindowDragAction::ToggleMaximize => (self.on_event)(RuntimeEvent::CommandFailed(
                "window maximization is unavailable on this window backend".into(),
            )),
        }
        true
    }
}

fn squared_distance(left: Point, right: Point) -> f32 {
    let x = left.x - right.x;
    let y = left.y - right.y;
    x.mul_add(x, y * y)
}
