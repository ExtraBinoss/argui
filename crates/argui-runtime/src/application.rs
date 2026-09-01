use argui_animation::Frame;
use argui_inspect::InspectorHandle;
use argui_paint::{ImageAsset, VectorAsset};
use argui_platform::{PlatformEvent, TrayConfig, TrayEvent, WindowKey, WindowLevel, WindowSpec};
use argui_ui::{ClipboardRequest, Element, FocusRequest, TextSelectionRequest, UiEvent};

use crate::{
    Entity, LayoutSnapshot, Render, ScrollRequest, ThemeRequest, ViewUpdate, WindowEnvironment,
};

#[derive(Clone, Debug, PartialEq)]
pub enum AppEvent {
    Ui {
        window: WindowKey,
        event: UiEvent,
    },
    Window {
        window: WindowKey,
        event: PlatformEvent,
    },
    Tray(TrayEvent),
}

#[derive(Clone, Debug, PartialEq)]
pub enum AppCommand {
    OpenWindow(WindowSpec),
    CloseWindow(WindowKey),
    ShowWindow(WindowKey),
    HideWindow(WindowKey),
    ToggleWindow(WindowKey),
    FocusWindow(WindowKey),
    SetWindowTitle {
        window: WindowKey,
        title: String,
    },
    MinimizeWindow(WindowKey),
    SetWindowMaximized {
        window: WindowKey,
        maximized: bool,
    },
    ToggleWindowMaximized(WindowKey),
    SetWindowLevel {
        window: WindowKey,
        level: WindowLevel,
    },
    SetWindowMousePassthrough {
        window: WindowKey,
        passthrough: bool,
    },
    Quit,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WindowInvalidation {
    pub window: WindowKey,
    pub update: ViewUpdate,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AppUpdate {
    pub windows: Vec<WindowInvalidation>,
    pub commands: Vec<AppCommand>,
    pub tray_changed: bool,
}

impl AppUpdate {
    #[must_use]
    pub const fn none() -> Self {
        Self {
            windows: Vec::new(),
            commands: Vec::new(),
            tray_changed: false,
        }
    }

    #[must_use]
    pub fn window(mut self, window: WindowKey, update: ViewUpdate) -> Self {
        self.invalidate(window, update);
        self
    }

    #[must_use]
    pub fn command(mut self, command: AppCommand) -> Self {
        self.commands.push(command);
        self
    }

    #[must_use]
    pub fn tray_changed(mut self) -> Self {
        self.tray_changed = true;
        self
    }

    pub fn invalidate(&mut self, window: WindowKey, update: ViewUpdate) {
        if update == ViewUpdate::None {
            return;
        }
        if let Some(existing) = self
            .windows
            .iter_mut()
            .find(|candidate| candidate.window == window)
        {
            existing.update = strongest(existing.update, update);
        } else {
            self.windows.push(WindowInvalidation { window, update });
        }
    }
}

fn strongest(left: ViewUpdate, right: ViewUpdate) -> ViewUpdate {
    match (left, right) {
        (ViewUpdate::Rebuild, _) | (_, ViewUpdate::Rebuild) => ViewUpdate::Rebuild,
        (ViewUpdate::Paint, _) | (_, ViewUpdate::Paint) => ViewUpdate::Paint,
        _ => ViewUpdate::None,
    }
}

pub trait AppModel: 'static {
    fn view(&self, window: &WindowKey, environment: WindowEnvironment) -> Option<Element>;

    fn update(&mut self, event: &AppEvent) -> AppUpdate;

    fn animation_frame(&mut self, window: &WindowKey, frame: Frame) -> AppUpdate {
        let _ = (window, frame);
        AppUpdate::none()
    }

    fn wants_animation_frame(&self, _window: &WindowKey) -> bool {
        false
    }

    fn layout_changed(&mut self, _window: &WindowKey, _layout: &LayoutSnapshot) -> AppUpdate {
        AppUpdate::none()
    }

    fn tray(&self) -> Option<TrayConfig> {
        None
    }

    fn image_assets(&self) -> Vec<ImageAsset> {
        Vec::new()
    }

    fn vector_assets(&self) -> Vec<VectorAsset> {
        Vec::new()
    }

    fn inspector(&self, _window: &WindowKey) -> Option<InspectorHandle> {
        None
    }

    fn take_clipboard_request(&mut self, _window: &WindowKey) -> Option<ClipboardRequest> {
        None
    }

    fn take_scroll_request(&mut self, _window: &WindowKey) -> Option<ScrollRequest> {
        None
    }

    fn take_focus_request(&mut self, _window: &WindowKey) -> Option<FocusRequest> {
        None
    }

    fn take_text_selection_request(&mut self, _window: &WindowKey) -> Option<TextSelectionRequest> {
        None
    }

    fn take_theme_request(&mut self, _window: &WindowKey) -> Option<ThemeRequest> {
        None
    }
}

pub(crate) struct SingleWindowModel<A: Render> {
    app: Entity<A>,
    clipboard: std::cell::RefCell<Option<ClipboardRequest>>,
    scroll: std::cell::RefCell<Option<ScrollRequest>>,
    focus: std::cell::RefCell<Option<FocusRequest>>,
    text_selection: std::cell::RefCell<Option<TextSelectionRequest>>,
    theme: std::cell::RefCell<Option<ThemeRequest>>,
    animation_requested: std::cell::Cell<bool>,
}

impl<A: Render> SingleWindowModel<A> {
    pub(crate) fn new(app: A) -> Self {
        Self {
            app: Entity::new(app),
            clipboard: std::cell::RefCell::new(None),
            scroll: std::cell::RefCell::new(None),
            focus: std::cell::RefCell::new(None),
            text_selection: std::cell::RefCell::new(None),
            theme: std::cell::RefCell::new(None),
            animation_requested: std::cell::Cell::new(false),
        }
    }

    fn drain_effects(&self, window: &WindowKey) -> AppUpdate {
        let effects = self.app.take_effects();
        if effects.clipboard.is_some() {
            *self.clipboard.borrow_mut() = effects.clipboard;
        }
        if effects.scroll.is_some() {
            *self.scroll.borrow_mut() = effects.scroll;
        }
        if effects.focus.is_some() {
            *self.focus.borrow_mut() = effects.focus;
        }
        if effects.text_selection.is_some() {
            *self.text_selection.borrow_mut() = effects.text_selection;
        }
        if effects.theme.is_some() {
            *self.theme.borrow_mut() = effects.theme;
        }
        self.animation_requested
            .set(self.animation_requested.get() || effects.animation_frame);
        AppUpdate {
            windows: (effects.update != ViewUpdate::None)
                .then(|| WindowInvalidation {
                    window: window.clone(),
                    update: effects.update,
                })
                .into_iter()
                .collect(),
            commands: effects.commands,
            tray_changed: false,
        }
    }
}

impl<A: Render> AppModel for SingleWindowModel<A> {
    fn view(&self, window: &WindowKey, environment: WindowEnvironment) -> Option<Element> {
        (window.as_str() == WindowKey::MAIN_VALUE).then(|| self.app.render_in(environment))
    }

    fn update(&mut self, event: &AppEvent) -> AppUpdate {
        match event {
            AppEvent::Ui { window, event } if window.as_str() == WindowKey::MAIN_VALUE => {
                self.app.event(event);
                self.drain_effects(window)
            }
            _ => AppUpdate::none(),
        }
    }

    fn animation_frame(&mut self, window: &WindowKey, frame: Frame) -> AppUpdate {
        self.animation_requested.set(false);
        self.app.animation_frame(frame);
        self.drain_effects(window)
    }

    fn wants_animation_frame(&self, window: &WindowKey) -> bool {
        window.as_str() == WindowKey::MAIN_VALUE
            && (self.animation_requested.get() || self.app.wants_frame())
    }

    fn layout_changed(&mut self, window: &WindowKey, layout: &LayoutSnapshot) -> AppUpdate {
        self.app.layout_changed(layout);
        self.drain_effects(window)
    }

    fn image_assets(&self) -> Vec<ImageAsset> {
        self.app.read(Render::image_assets)
    }

    fn vector_assets(&self) -> Vec<VectorAsset> {
        self.app.read(Render::vector_assets)
    }

    fn inspector(&self, window: &WindowKey) -> Option<InspectorHandle> {
        (window.as_str() == WindowKey::MAIN_VALUE)
            .then(|| self.app.read(Render::inspector))
            .flatten()
    }

    fn take_clipboard_request(&mut self, window: &WindowKey) -> Option<ClipboardRequest> {
        (window.as_str() == WindowKey::MAIN_VALUE)
            .then(|| self.clipboard.borrow_mut().take())
            .flatten()
    }

    fn take_scroll_request(&mut self, window: &WindowKey) -> Option<ScrollRequest> {
        (window.as_str() == WindowKey::MAIN_VALUE)
            .then(|| self.scroll.borrow_mut().take())
            .flatten()
    }

    fn take_focus_request(&mut self, window: &WindowKey) -> Option<FocusRequest> {
        (window.as_str() == WindowKey::MAIN_VALUE)
            .then(|| self.focus.borrow_mut().take())
            .flatten()
    }

    fn take_text_selection_request(&mut self, window: &WindowKey) -> Option<TextSelectionRequest> {
        (window.as_str() == WindowKey::MAIN_VALUE)
            .then(|| self.text_selection.borrow_mut().take())
            .flatten()
    }

    fn take_theme_request(&mut self, window: &WindowKey) -> Option<ThemeRequest> {
        (window.as_str() == WindowKey::MAIN_VALUE)
            .then(|| self.theme.borrow_mut().take())
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use argui_core::{Color, ColorScheme, Point};
    use argui_ui::{ClipboardRequest, Element, TextSelection, UiEvent, UiEventKind, UiTree};

    use super::{AppEvent, AppModel, SingleWindowModel};
    use crate::{Context, Render, ThemeRequest};

    struct ThemedApp;

    impl Render for ThemedApp {
        fn render(&mut self, _cx: &mut Context<Self>) -> Element {
            Element::container([]).keyed("theme")
        }

        fn event(&mut self, _event: &UiEvent, cx: &mut Context<Self>) {
            cx.write_clipboard(ClipboardRequest::Write("theme".into()));
            cx.scroll_to("theme", Point::new(1.0, 2.0));
            cx.request_focus("theme");
            cx.select_text("theme", TextSelection::All);
            cx.request_animation_frame();
            cx.set_theme(ThemeRequest {
                color_scheme: Some(ColorScheme::Dark),
                primary: Some(Color::rgb(0.8, 0.2, 0.4)),
            });
        }
    }

    #[test]
    fn single_window_models_forward_theme_requests_to_the_runtime() {
        let window = argui_platform::WindowKey::main();
        let mut model = SingleWindowModel::new(ThemedApp);
        let tree = UiTree::new(Element::container([]));
        model.update(&AppEvent::Ui {
            window: window.clone(),
            event: UiEvent {
                target: tree.node_ids()[0],
                key: Some("theme".into()),
                kind: UiEventKind::Clicked,
            },
        });
        let _ = model.take_theme_request(&argui_platform::WindowKey::new("secondary"));
        let _ = model.take_clipboard_request(&window);
        let _ = model.take_scroll_request(&window);
        let _ = model.take_focus_request(&window);
        let _ = model.take_text_selection_request(&window);
        let _ = model.wants_animation_frame(&window);
        let _ = model.take_theme_request(&window);
        let _ = model.take_theme_request(&window);
    }
}
