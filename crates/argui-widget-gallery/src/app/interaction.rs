use argui::{
    core::{ColorScheme, Size},
    paint::{ImageAsset, VectorAsset},
    runtime::{Context, LayoutSnapshot, Render},
    ui::{Element, EventType, GestureKind, GesturePhase, UiEventKind},
    widgets::shadcn,
};

use super::{EDITOR_DEFAULT_SIZE, WidgetGallery};
use crate::pages::ResizeListeners;

impl WidgetGallery {
    pub(super) fn render_element(&mut self, cx: &mut Context<Self>) -> Element {
        let environment = cx.environment();
        let themes = shadcn(environment.primary);
        let theme = themes.resolve(environment.color_scheme);
        let assets = match environment.color_scheme {
            ColorScheme::Light => &self.light_assets,
            ColorScheme::Dark => &self.dark_assets,
        };
        let spinner = cx.entity(&self.spinner);
        let resize_listener = cx.listener(EventType::Gesture, |gallery, event, cx| {
            let UiEventKind::Gesture(gesture) = event.kind else {
                return;
            };
            gallery.resize_editor(gesture, cx);
        });
        let resize_reset_listener = cx.listener(EventType::Click, |gallery, event, cx| {
            let UiEventKind::Click(click) = &event.kind else {
                return;
            };
            if click.count >= 2 {
                gallery.editor_size = EDITOR_DEFAULT_SIZE;
                gallery.editor_resize_start = EDITOR_DEFAULT_SIZE;
                cx.notify();
            }
        });
        let shell = cx.entity(&self.shell);
        let scroll_demo = cx.entity(&self.scroll_demo);
        self.view(
            environment,
            theme,
            assets,
            spinner,
            ResizeListeners {
                textarea: resize_listener,
                textarea_reset: resize_reset_listener,
                shell,
                scroll_demo,
            },
        )
    }

    fn resize_editor(&mut self, gesture: argui::ui::GestureEvent, cx: &mut Context<Self>) {
        let GestureKind::Pan { total, .. } = gesture.kind else {
            return;
        };
        if gesture.phase == GesturePhase::Started {
            self.editor_resize_start = self.editor_size;
        }
        if matches!(gesture.phase, GesturePhase::Started | GesturePhase::Changed) {
            self.editor_size = Size::new(
                (self.editor_resize_start.width + total.x).clamp(280.0, 760.0),
                (self.editor_resize_start.height + total.y).clamp(120.0, 480.0),
            );
            cx.notify();
        }
    }
}

impl Render for WidgetGallery {
    fn animation_frame(&mut self, frame: argui::animation::Frame, cx: &mut Context<Self>) {
        if self.select_presence.advance(frame.elapsed) {
            cx.notify();
        } else {
            cx.request_paint();
        }
    }

    fn wants_animation_frame(&self) -> bool {
        self.select_presence.animating()
    }

    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        if cx.environment().reduced_motion {
            self.select_presence
                .set_open(self.select_presence.is_open(), true);
        }
        let mut root = self.render_element(cx);
        for event_type in EventType::ALL {
            root = root.on(cx
                .listener(event_type, WidgetGallery::handle_event)
                .capture(true));
        }
        root
    }

    fn layout_changed(&mut self, layout: &LayoutSnapshot, _cx: &mut Context<Self>) {
        self.handle_layout(layout);
    }

    fn image_assets(&self) -> Vec<ImageAsset> {
        self.images.assets().to_vec()
    }

    fn vector_assets(&self) -> Vec<VectorAsset> {
        self.light_assets
            .assets()
            .iter()
            .chain(self.dark_assets.assets())
            .chain(self.accent_assets.assets())
            .cloned()
            .collect()
    }
}
