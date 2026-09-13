use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use argui_animation::Frame;
use argui_inspect::InspectorHandle;
use argui_paint::{ImageAsset, VectorAsset};
use argui_platform::{TrayConfig, WindowKey};
use argui_ui::{ClipboardRequest, Element, FocusRequest, TextSelectionRequest, UiCommand};
use dioxus_devtools::subsecond::{HotFn, register_handler};

use crate::{
    AnyEntity, AppEvent, AppModel, AppUpdate, Context, LayoutSnapshot, Render, RuntimeEvent,
    ScrollRequest, ThemeRequest, WindowEnvironment, event::UserEvent, host::EventProxy,
};

static GENERATION: AtomicU64 = AtomicU64::new(0);

pub(crate) fn generation() -> u64 {
    GENERATION.load(Ordering::Acquire)
}

pub(crate) fn connect(proxy: EventProxy) {
    register_handler(Arc::new(move || {
        let generation = GENERATION.fetch_add(1, Ordering::AcqRel).wrapping_add(1);
        let _ = proxy.send_event(UserEvent::HotReload { generation });
    }));
    dioxus_devtools::connect_subsecond();
}

macro_rules! hot_call {
    ($function:expr, $arguments:expr) => {{
        let mut function = HotFn::current($function);
        function.call($arguments)
    }};
}

pub(crate) fn render<T: Render>(value: &mut T, cx: &mut Context<T>) -> Element {
    hot_call!(<T as Render>::render, (value, cx))
}

#[cfg(feature = "tasks")]
pub(crate) fn tasks_ready<T: Render>(value: &mut T, cx: &mut Context<T>) {
    hot_call!(<T as Render>::tasks_ready, (value, cx))
}

pub(crate) fn animation_frame<T: Render>(value: &mut T, frame: Frame, cx: &mut Context<T>) {
    hot_call!(<T as Render>::animation_frame, (value, frame, cx))
}

pub(crate) fn wants_animation_frame<T: Render>(value: &T) -> bool {
    hot_call!(<T as Render>::wants_animation_frame, (value,))
}

pub(crate) fn layout_changed<T: Render>(
    value: &mut T,
    layout: &LayoutSnapshot,
    cx: &mut Context<T>,
) {
    hot_call!(<T as Render>::layout_changed, (value, layout, cx))
}

pub(crate) fn image_assets<T: Render>(value: &T) -> Vec<ImageAsset> {
    hot_call!(<T as Render>::image_assets, (value,))
}

pub(crate) fn vector_assets<T: Render>(value: &T) -> Vec<VectorAsset> {
    hot_call!(<T as Render>::vector_assets, (value,))
}

pub(crate) fn inspector<T: Render>(value: &T) -> Option<InspectorHandle> {
    hot_call!(<T as Render>::inspector, (value,))
}

pub(crate) fn app_model<T: AppModel>(model: T) -> Box<dyn AppModel> {
    Box::new(HotAppModel { model })
}

struct HotAppModel<T> {
    model: T,
}

impl<T: AppModel> AppModel for HotAppModel<T> {
    fn take_ui_commands(&mut self, window: &WindowKey) -> Vec<UiCommand> {
        hot_call!(<T as AppModel>::take_ui_commands, (&mut self.model, window))
    }

    fn tasks_ready(&mut self, window: &WindowKey) -> AppUpdate {
        hot_call!(<T as AppModel>::tasks_ready, (&mut self.model, window))
    }

    fn view(&self, window: &WindowKey, environment: WindowEnvironment) -> Option<Element> {
        hot_call!(<T as AppModel>::view, (&self.model, window, environment))
    }

    fn event_router(&self, window: &WindowKey) -> Option<AnyEntity> {
        hot_call!(<T as AppModel>::event_router, (&self.model, window))
    }

    fn captures_ui_events(&self) -> bool {
        hot_call!(<T as AppModel>::captures_ui_events, (&self.model,))
    }

    fn update(&mut self, event: &AppEvent) -> AppUpdate {
        hot_call!(<T as AppModel>::update, (&mut self.model, event))
    }

    fn animation_frame(&mut self, window: &WindowKey, frame: Frame) -> AppUpdate {
        hot_call!(
            <T as AppModel>::animation_frame,
            (&mut self.model, window, frame)
        )
    }

    fn wants_animation_frame(&self, window: &WindowKey) -> bool {
        hot_call!(
            <T as AppModel>::wants_animation_frame,
            (&self.model, window)
        )
    }

    fn layout_changed(&mut self, window: &WindowKey, layout: &LayoutSnapshot) -> AppUpdate {
        hot_call!(
            <T as AppModel>::layout_changed,
            (&mut self.model, window, layout)
        )
    }

    fn tray(&self) -> Option<TrayConfig> {
        hot_call!(<T as AppModel>::tray, (&self.model,))
    }

    fn image_assets(&self) -> Vec<ImageAsset> {
        hot_call!(<T as AppModel>::image_assets, (&self.model,))
    }

    fn vector_assets(&self) -> Vec<VectorAsset> {
        hot_call!(<T as AppModel>::vector_assets, (&self.model,))
    }

    fn inspector(&self, window: &WindowKey) -> Option<InspectorHandle> {
        hot_call!(<T as AppModel>::inspector, (&self.model, window))
    }

    fn take_clipboard_request(&mut self, window: &WindowKey) -> Option<ClipboardRequest> {
        hot_call!(
            <T as AppModel>::take_clipboard_request,
            (&mut self.model, window)
        )
    }

    fn take_scroll_request(&mut self, window: &WindowKey) -> Option<ScrollRequest> {
        hot_call!(
            <T as AppModel>::take_scroll_request,
            (&mut self.model, window)
        )
    }

    fn take_focus_request(&mut self, window: &WindowKey) -> Option<FocusRequest> {
        hot_call!(
            <T as AppModel>::take_focus_request,
            (&mut self.model, window)
        )
    }

    fn take_text_selection_request(&mut self, window: &WindowKey) -> Option<TextSelectionRequest> {
        hot_call!(
            <T as AppModel>::take_text_selection_request,
            (&mut self.model, window)
        )
    }

    fn take_theme_request(&mut self, window: &WindowKey) -> Option<ThemeRequest> {
        hot_call!(
            <T as AppModel>::take_theme_request,
            (&mut self.model, window)
        )
    }
}

pub(crate) fn notify(application: &mut crate::app::Application, generation: u64) {
    application.invalidate(crate::ViewUpdate::Rebuild);
    (application.on_event)(RuntimeEvent::HotReloaded { generation });
}
