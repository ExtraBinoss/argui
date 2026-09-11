use argui_core::{Key, KeyInput, KeyState, TextPosition};
use argui_text::CaretStop;
use argui_ui::{InteractionUpdate, UiTree};

use super::TextInputRegion;

impl TextInputRegion {
    /// Resolve visual movement against shaped lines; word/document commands use editor text.
    pub fn navigate(&self, ui: &mut UiTree, input: &KeyInput) -> Option<InteractionUpdate> {
        if input.state != KeyState::Pressed
            || ui.focused_node() != Some(self.node)
            || ui.text_input_composing(self.node)
        {
            return None;
        }
        let position = ui.text_input_position(self.node)?;
        let extend = input.modifiers.shift;
        let horizontal = matches!(input.key, Key::ArrowLeft | Key::ArrowRight);
        if horizontal && (input.modifiers.command() || input.modifiers.alt)
            || matches!(input.key, Key::Home | Key::End) && input.modifiers.command()
        {
            return Some(ui.edit_text_input(input));
        }
        let next = match input.key {
            Key::ArrowLeft | Key::ArrowRight => {
                let left = input.key == Key::ArrowLeft;
                if !extend
                    && let Some((anchor, cursor)) = ui.text_input_selection_positions(self.node)
                {
                    let a = self.stop_for(anchor)?;
                    let b = self.stop_for(cursor)?;
                    let anchor_first = (a.point.y, a.point.x) <= (b.point.y, b.point.x);
                    if left == anchor_first { anchor } else { cursor }
                } else {
                    self.visual_neighbor(position, left, false)
                }
            }
            Key::Home | Key::End => self.line_edge(position, input.key == Key::End),
            Key::ArrowUp | Key::ArrowDown | Key::PageUp | Key::PageDown => {
                let current = self.stop_for(position)?;
                let up = matches!(input.key, Key::ArrowUp | Key::PageUp);
                let goal = ui
                    .text_input_goal_x(self.node)
                    .unwrap_or(current.point.x - self.viewport.origin.x + self.scroll_x);
                let page = matches!(input.key, Key::PageUp | Key::PageDown);
                let next = self.vertical_target(position, goal, up, page);
                return Some(ui.move_text_vertically(self.node, next, goal, extend));
            }
            _ => return None,
        };
        Some(ui.move_text_position(self.node, next, extend))
    }

    fn stop_for(&self, position: TextPosition) -> Option<&CaretStop> {
        self.stops
            .iter()
            .find(|stop| stop.position == position)
            .or_else(|| {
                self.stops
                    .iter()
                    .find(|stop| stop.position.index == position.index)
            })
    }

    fn line_edge(&self, position: TextPosition, end: bool) -> TextPosition {
        let Some(current) = self.stop_for(position) else {
            return position;
        };
        self.stops
            .iter()
            .filter(|stop| (stop.point.y - current.point.y).abs() < 0.01)
            .min_by(|a, b| {
                if end {
                    b.point.x.total_cmp(&a.point.x)
                } else {
                    a.point.x.total_cmp(&b.point.x)
                }
            })
            .map_or(position, |stop| stop.position)
    }

    fn vertical_target(
        &self,
        position: TextPosition,
        goal: f32,
        up: bool,
        page: bool,
    ) -> TextPosition {
        let Some(current) = self.stop_for(position) else {
            return position;
        };
        let target_y = current.point.y
            + if page {
                if up {
                    -self.viewport.size.height
                } else {
                    self.viewport.size.height
                }
            } else {
                0.0
            };
        let next_y = self
            .stops
            .iter()
            .filter(|stop| {
                if up {
                    stop.point.y < current.point.y - 0.01
                } else {
                    stop.point.y > current.point.y + 0.01
                }
            })
            .min_by(|a, b| {
                (a.point.y - target_y)
                    .abs()
                    .total_cmp(&(b.point.y - target_y).abs())
            })
            .map(|stop| stop.point.y);
        let Some(next_y) = next_y else {
            return self.line_edge(position, !up);
        };
        let x = goal + self.viewport.origin.x - self.scroll_x;
        self.stops
            .iter()
            .filter(|stop| (stop.point.y - next_y).abs() < 0.01)
            .min_by(|a, b| (a.point.x - x).abs().total_cmp(&(b.point.x - x).abs()))
            .map_or(position, |stop| stop.position)
    }

    pub(super) fn adjacent_line_edge(
        &self,
        position: TextPosition,
        previous: bool,
    ) -> TextPosition {
        let Some(current) = self.stop_for(position) else {
            return position;
        };
        let next = self
            .stops
            .iter()
            .filter(|stop| {
                if previous {
                    stop.point.y < current.point.y - 0.01
                } else {
                    stop.point.y > current.point.y + 0.01
                }
            })
            .min_by(|a, b| {
                (a.point.y - current.point.y)
                    .abs()
                    .total_cmp(&(b.point.y - current.point.y).abs())
            });
        next.map_or(position, |stop| self.line_edge(stop.position, previous))
    }
}
