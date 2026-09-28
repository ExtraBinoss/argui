use super::MultiApplication;
use crate::host::WindowFactory;

impl MultiApplication {
    pub(super) fn collect_app_commands(&mut self, updates: &mut Vec<crate::AppUpdate>) {
        for entry in self.windows.values_mut() {
            let commands = std::mem::take(&mut entry.runtime.pending_app_commands);
            if !commands.is_empty() {
                updates.push(crate::AppUpdate {
                    commands,
                    ..crate::AppUpdate::none()
                });
            }
        }
    }

    pub(super) fn models_ready(&mut self, event_loop: &dyn WindowFactory) {
        let mut visited = self.retired.clone();
        for entry in self.windows.values() {
            entry.runtime.collect_model_runtimes(&mut visited);
        }
        for runtime in visited {
            runtime.dispatch_pending();
        }
        for entry in self.windows.values_mut() {
            entry.runtime.collect_model_effects(event_loop);
        }
        self.retired.retain(|runtime| {
            runtime.pending_events() != 0
                || runtime.pending_invalidations() != 0
                || runtime.pending_mount_events() != 0
        });
        self.process_pending(event_loop);
    }

    #[cfg(feature = "tasks")]
    pub(super) fn tasks_ready(&mut self, event_loop: &dyn WindowFactory) {
        if let Some(tasks) = &self.tasks {
            tasks.drain();
        }
        for entry in self.windows.values_mut() {
            entry.runtime.tasks_ready(event_loop);
        }
        self.process_pending(event_loop);
    }
}

use super::{SharedModel, SharedUpdates};
use crate::{AppEvent, AppUpdate, Context, LayoutSnapshot, Render, ViewUpdate};
use argui_platform::WindowKey;
use argui_ui::Element;

pub(super) struct WindowModel {
    pub(super) key: WindowKey,
    pub(super) model: SharedModel,
    pub(super) pending: SharedUpdates,
}

impl WindowModel {
    fn view(&self, environment: crate::WindowEnvironment) -> Option<Element> {
        self.model.borrow().view(&self.key, environment)
    }

    fn record(&self, mut update: AppUpdate) -> ViewUpdate {
        let own = update
            .windows
            .iter()
            .filter(|candidate| candidate.window == self.key)
            .fold(ViewUpdate::None, |current, candidate| {
                if candidate.update == ViewUpdate::Rebuild || current == ViewUpdate::Rebuild {
                    ViewUpdate::Rebuild
                } else if candidate.update == ViewUpdate::Paint || current == ViewUpdate::Paint {
                    ViewUpdate::Paint
                } else {
                    ViewUpdate::None
                }
            });
        update
            .windows
            .retain(|candidate| candidate.window != self.key);
        if !update.windows.is_empty() || !update.commands.is_empty() || update.tray_changed {
            self.pending.borrow_mut().push(update);
        }
        own
    }

    fn handle_ui_event(&mut self, event: &argui_ui::UiEvent, cx: &mut Context<Self>) {
        let update = self.model.borrow_mut().update(&AppEvent::Ui {
            window: self.key.clone(),
            event: event.clone(),
        });
        self.record_and_forward(update, cx);
    }

    fn record_and_forward(&mut self, update: AppUpdate, cx: &mut Context<Self>) {
        request_update(cx, self.record(update));
        self.forward_requests(cx);
    }

    fn forward_requests(&mut self, cx: &mut Context<Self>) {
        for command in self.model.borrow_mut().take_ui_commands(&self.key) {
            cx.ui_command(command);
        }
        if let Some(request) = self.model.borrow_mut().take_clipboard_request(&self.key) {
            cx.write_clipboard(request);
        }
        if let Some(request) = self.model.borrow_mut().take_scroll_request(&self.key) {
            cx.scroll(request);
        }
        if let Some(request) = self.model.borrow_mut().take_focus_request(&self.key) {
            match request {
                argui_ui::FocusRequest::Focus(target) => cx.request_focus(target),
                argui_ui::FocusRequest::Clear => cx.clear_focus(),
                argui_ui::FocusRequest::Next => cx.focus_next(),
                argui_ui::FocusRequest::Previous => cx.focus_previous(),
            }
        }
        if let Some(request) = self
            .model
            .borrow_mut()
            .take_text_selection_request(&self.key)
        {
            cx.select_text(request.target, request.selection);
        }
        if let Some(request) = self.model.borrow_mut().take_theme_request(&self.key) {
            cx.set_theme(request);
        }
    }
}

impl Render for WindowModel {
    #[cfg(feature = "tasks")]
    fn tasks_ready(&mut self, cx: &mut Context<Self>) {
        let update = self.model.borrow_mut().tasks_ready(&self.key);
        self.record_and_forward(update, cx);
    }

    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let mut router = self.model.borrow().event_router(&self.key);
        if let Some(router) = router.clone() {
            cx.route_events_to(router);
        }
        let mut root = self
            .view(cx.environment())
            .unwrap_or_else(|| Element::container(Vec::<Element>::new()));
        if router.is_none() {
            router = self.model.borrow().event_router(&self.key);
            if let Some(router) = router.clone() {
                cx.route_events_to(router);
            }
        }
        if router.is_some() && !self.model.borrow().captures_ui_events() {
            return root;
        }
        for event in argui_ui::EventType::ALL {
            root = root.on(cx
                .listener(event, |model, event, cx| model.handle_ui_event(event, cx))
                .capture(true));
        }
        root
    }
    fn animation_frame(&mut self, frame: argui_animation::Frame, cx: &mut Context<Self>) {
        let update = self.model.borrow_mut().animation_frame(&self.key, frame);
        self.record_and_forward(update, cx);
    }
    fn wants_animation_frame(&self) -> bool {
        self.model.borrow().wants_animation_frame(&self.key)
    }
    fn layout_changed(&mut self, layout: &LayoutSnapshot, cx: &mut Context<Self>) {
        let update = self.model.borrow_mut().layout_changed(&self.key, layout);
        self.record_and_forward(update, cx);
    }
    fn image_assets(&self) -> Vec<argui_paint::ImageAsset> {
        self.model.borrow().image_assets()
    }

    fn vector_assets(&self) -> Vec<argui_paint::VectorAsset> {
        self.model.borrow().vector_assets()
    }

    fn effect_definitions(&self) -> Vec<argui_render::EffectDefinition> {
        self.model.borrow().effect_definitions()
    }

    #[cfg(feature = "inspect")]
    fn inspector(&self) -> Option<argui_inspect::InspectorHandle> {
        self.model.borrow().inspector(&self.key)
    }
}

fn request_update<T: Render>(cx: &mut Context<T>, update: ViewUpdate) {
    match update {
        ViewUpdate::None => {}
        ViewUpdate::Paint => cx.request_paint(),
        ViewUpdate::Rebuild => cx.notify(),
    }
}
