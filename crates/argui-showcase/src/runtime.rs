use argui_animation::Frame;
use argui_core::Size;
use argui_paint::{ImageAsset, VectorAsset};
use argui_runtime::{Context, Render, ViewUpdate};
use argui_ui::{Element, EventType, GestureKind, GesturePhase, UiEventKind};

use crate::StateShowcase;

impl Render for StateShowcase {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let resize = cx.listener(EventType::Gesture, |showcase, event, cx| {
            let UiEventKind::Gesture(gesture) = event.kind else {
                return;
            };
            let GestureKind::Pan { total, .. } = gesture.kind else {
                return;
            };
            if gesture.phase == GesturePhase::Started {
                showcase.editor_resize_start = showcase.editor_size;
            }
            if matches!(gesture.phase, GesturePhase::Started | GesturePhase::Changed) {
                showcase.editor_size = Size::new(
                    (showcase.editor_resize_start.width + total.x).clamp(280.0, 900.0),
                    (showcase.editor_resize_start.height + total.y).clamp(120.0, 520.0),
                );
                cx.notify();
            }
        });
        let mut root = self.view_with_resize(cx.environment(), Some(resize));
        for event_type in EventType::ALL {
            root = root.on(cx
                .listener(event_type, |showcase, event, cx| {
                    let update = showcase.update(event);
                    if matches!(event.target_key(), Some("theme" | "primary"))
                        && matches!(event.kind, UiEventKind::Click(_))
                    {
                        cx.set_theme(showcase.theme_request());
                    }
                    request_update(cx, update);
                })
                .capture(true));
        }
        root
    }

    fn animation_frame(&mut self, frame: Frame, cx: &mut Context<Self>) {
        request_update(cx, StateShowcase::animation_frame(self, frame));
    }

    fn wants_animation_frame(&self) -> bool {
        StateShowcase::wants_animation_frame(self)
    }

    fn image_assets(&self) -> Vec<ImageAsset> {
        StateShowcase::image_assets(self)
    }

    fn vector_assets(&self) -> Vec<VectorAsset> {
        StateShowcase::vector_assets(self)
    }
}

fn request_update<T: Render>(cx: &mut Context<T>, update: ViewUpdate) {
    match update {
        ViewUpdate::None => {}
        ViewUpdate::Paint => cx.request_paint(),
        ViewUpdate::Rebuild => cx.notify(),
    }
}
