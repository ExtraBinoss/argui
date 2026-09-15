use argui_core::{Key, KeyState, Point};
use argui_ui::{GestureKind, GesturePhase, SemanticValue, UiEvent, UiEventKind};

use super::{ColorFormat, ColorPickerState};

impl ColorPickerState {
    /// Handle events for this picker. `key` scopes events to this picker and
    /// `event` is the UI input being handled. Consumed keys and gestures suppress their
    /// default action so arrow keys do not also scroll the surrounding panel.
    pub fn update(&mut self, key: &str, event: &UiEvent) -> bool {
        if !self.enabled {
            return false;
        }
        let Some(target) = event
            .target_key()
            .and_then(|target| target.strip_prefix(key))
            .and_then(|target| target.strip_prefix("::"))
        else {
            return false;
        };
        let changed = if let Some(format) = target.strip_prefix("format::") {
            if !matches!(event.kind, UiEventKind::Click(_)) {
                return false;
            }
            let Some(format) = ColorFormat::ALL
                .into_iter()
                .find(|candidate| candidate.label() == format)
            else {
                return false;
            };
            self.set_format(format);
            true
        } else if let Some(index) = target
            .strip_prefix("field::")
            .and_then(|index| index.parse::<usize>().ok())
        {
            if index
                >= if self.format == ColorFormat::Hex {
                    1
                } else {
                    4
                }
            {
                return false;
            }
            self.edit_field(index, event)
        } else if target == "pad" {
            self.update_pad(event)
        } else if target == "hue" || target == "alpha" {
            if matches!(&event.kind, UiEventKind::SemanticAction {
                value: Some(SemanticValue::Number { value, .. }), ..
            } if !value.is_finite())
            {
                return false;
            }
            let alpha = target == "alpha";
            let behavior = self.range(key, alpha);
            let range = if alpha {
                &mut self.alpha_range
            } else {
                &mut self.hue_range
            };
            let Some(action) = range.update(event, &behavior) else {
                return false;
            };
            let value = action.value();
            if !value.is_finite() {
                return false;
            }
            if alpha {
                self.alpha = value / 100.0;
            } else {
                self.hsv[0] = value;
            }
            self.draft = None;
            true
        } else {
            false
        };
        if changed
            && matches!(
                event.kind,
                UiEventKind::KeyInput(_) | UiEventKind::Gesture(_)
            )
        {
            let _ = event.prevent_default();
        }
        changed
    }

    fn edit_field(&mut self, index: usize, event: &UiEvent) -> bool {
        match &event.kind {
            UiEventKind::TextChanged(text) | UiEventKind::Submitted(text) => {
                let valid = self.parse_field(index, text);
                self.draft = Some((index, text.clone(), !valid));
                true
            }
            UiEventKind::Blurred => {
                if self.draft.as_ref().is_some_and(|draft| draft.0 == index) {
                    self.draft = None;
                    true
                } else {
                    false
                }
            }
            UiEventKind::KeyInput(input)
                if input.key == Key::Escape && input.state == KeyState::Pressed =>
            {
                self.draft.take().is_some()
            }
            _ => false,
        }
    }

    fn update_pad(&mut self, event: &UiEvent) -> bool {
        match &event.kind {
            UiEventKind::Gesture(gesture) => match gesture.kind {
                GestureKind::Pan { position, .. } => {
                    if gesture.phase == GesturePhase::Cancelled {
                        let Some(start) = self.drag_start.take() else {
                            return false;
                        };
                        self.hsv = start;
                        return true;
                    }
                    if gesture.phase == GesturePhase::Started {
                        self.drag_start = Some(self.hsv);
                    }
                    if gesture.phase == GesturePhase::Ended {
                        self.drag_start = None;
                    }
                    self.pad_position(position)
                }
                GestureKind::Tap { position } => self.pad_position(position),
                _ => false,
            },
            UiEventKind::KeyInput(input)
                if input.state == KeyState::Pressed
                    && !input.modifiers.command()
                    && !input.modifiers.alt =>
            {
                let step = if input.modifiers.shift { 0.001 } else { 0.01 };
                match input.key {
                    Key::ArrowLeft => self.hsv[1] = (self.hsv[1] - step).max(0.0),
                    Key::ArrowRight => self.hsv[1] = (self.hsv[1] + step).min(1.0),
                    Key::ArrowDown => self.hsv[2] = (self.hsv[2] - step).max(0.0),
                    Key::ArrowUp => self.hsv[2] = (self.hsv[2] + step).min(1.0),
                    Key::Home => {
                        self.hsv[1] = 0.0;
                        self.hsv[2] = 1.0;
                    }
                    Key::End => {
                        self.hsv[1] = 1.0;
                        self.hsv[2] = 0.0;
                    }
                    _ => return false,
                }
                self.draft = None;
                true
            }
            _ => false,
        }
    }

    fn pad_position(&mut self, position: Point) -> bool {
        let Some(bounds) = self.pad else {
            return false;
        };
        if bounds.size.width <= 0.0
            || bounds.size.height <= 0.0
            || !position.x.is_finite()
            || !position.y.is_finite()
        {
            return false;
        }
        self.hsv[1] = ((position.x - bounds.origin.x) / bounds.size.width).clamp(0.0, 1.0);
        self.hsv[2] = (1.0 - (position.y - bounds.origin.y) / bounds.size.height).clamp(0.0, 1.0);
        self.draft = None;
        true
    }
}
