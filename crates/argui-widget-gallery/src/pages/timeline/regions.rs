use super::{Clip, TimelineView};
use argui::{
    accessibility::{Role, SemanticAction, SemanticValue, Semantics},
    core::{Color, Key, KeyState},
    paint::{Border, QuadStyle},
    runtime::Context,
    ui::{
        CursorIcon, Element, EventType, GestureCapture, GestureKind, GesturePhase, GestureSet,
        Interaction, PanGesture, StylePatch, UiEventKind, VisualState,
    },
};

impl TimelineView {
    fn resize(&self, index: usize, duration: f32) {
        if duration.is_finite() {
            self.change_clip(index, |clip| {
                clip.duration = duration.clamp(1.0, 20.0 - clip.start);
            });
        }
    }

    pub(super) fn resize_region(
        &self,
        cx: &mut Context<Self>,
        index: usize,
        clip: Clip,
        color: Color,
        ring: Color,
    ) -> Element {
        Element::custom_region(
            self.key(&format!("resize-{index}")),
            Interaction::default()
                .focusable(true)
                .cursor(CursorIcon::EwResize)
                .gestures(
                    GestureSet::EMPTY.pan(
                        PanGesture::default()
                            .immediate()
                            .capture(GestureCapture::OnPress),
                    ),
                ),
            Semantics::new(Role::Slider)
                .label(format!("Clip {} duration", index + 1))
                .description("Drag to resize; Left/Right adjusts by half a second")
                .value(SemanticValue::Number {
                    value: f64::from(clip.duration),
                    minimum: Some(1.0),
                    maximum: Some(f64::from(20.0 - clip.start)),
                    step: Some(0.5),
                })
                .action(SemanticAction::Focus)
                .action(SemanticAction::Increment)
                .action(SemanticAction::Decrement)
                .action(SemanticAction::SetValue),
        )
        .background(color)
        .when(
            VisualState::FocusVisible,
            StylePatch::from_quad(QuadStyle::solid(color).border(Border::all(2.0, ring))),
        )
        .on(cx.listener(EventType::Gesture, move |this, event, cx| {
            let UiEventKind::Gesture(gesture) = event.kind else {
                return;
            };
            let GestureKind::Pan { total, .. } = gesture.kind else {
                return;
            };
            if gesture.phase == GesturePhase::Started {
                this.selected = index;
                this.drag_start = this.clips.read(|clips| clips[index]);
            }
            let delta = if gesture.phase == GesturePhase::Cancelled {
                0.0
            } else {
                total.x / (40.0 * this.zoom)
            };
            this.resize(index, this.drag_start.duration + delta);
            event.stop_propagation();
            cx.notify();
        }))
        .on(cx.listener(EventType::Key, move |this, event, cx| {
            let UiEventKind::KeyInput(key) = &event.kind else {
                return;
            };
            if key.state != KeyState::Pressed {
                return;
            }
            let delta = match key.key {
                Key::ArrowLeft => -0.5,
                Key::ArrowRight => 0.5,
                _ => return,
            };
            let duration = this.clips.read(|clips| clips[index].duration);
            this.resize(index, duration + delta);
            event.stop_propagation();
            cx.notify();
        }))
        .on(
            cx.listener(EventType::SemanticAction, move |this, event, cx| {
                let UiEventKind::SemanticAction { action, value } = &event.kind else {
                    return;
                };
                let duration = this.clips.read(|clips| clips[index].duration);
                let next = match (action, value) {
                    (SemanticAction::Increment, _) => duration + 0.5,
                    (SemanticAction::Decrement, _) => duration - 0.5,
                    (SemanticAction::SetValue, Some(SemanticValue::Number { value, .. })) => {
                        *value as f32
                    }
                    _ => return,
                };
                this.resize(index, next);
                event.stop_propagation();
                cx.notify();
            }),
        )
    }
}
