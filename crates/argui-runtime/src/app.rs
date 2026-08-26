use std::{cell::RefCell, rc::Rc, sync::Arc};

use argui_core::{Point, Size};
use argui_layout::{LayoutEngine, LayoutOutput};
use argui_platform::{ButtonState, PlatformError, PlatformEvent, PointerButton, WindowConfig};
use argui_render::{RenderStatus, RendererConfig, SurfaceRenderer};
use argui_text::{PreparedText, TextEngine, TextScene};
use argui_ui::{InteractionUpdate, TreeUpdate, UiTree};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::{RuntimeError, RuntimeEvent, UiApp, ViewUpdate};

enum RendererState {
    Loading,
    Ready(Box<SurfaceRenderer>),
    #[cfg(target_arch = "wasm32")]
    Failed(String),
}

pub(crate) struct Application {
    window_config: WindowConfig,
    renderer_config: RendererConfig,
    window: Option<Arc<Window>>,
    renderer: Rc<RefCell<RendererState>>,
    renderer_announced: bool,
    text_engine: TextEngine,
    text_scene: Option<TextScene>,
    ui_tree: Option<UiTree>,
    model: Option<Box<dyn UiApp>>,
    ui_layout: Option<LayoutOutput>,
    layout_engine: LayoutEngine,
    prepared_text: Option<PreparedText>,
    viewport: Size,
    scale_factor: f32,
    pub(crate) fatal_error: Option<RuntimeError>,
    on_event: Box<dyn FnMut(RuntimeEvent)>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Application {
    pub(crate) fn new(
        window_config: WindowConfig,
        renderer_config: RendererConfig,
        text_engine: TextEngine,
        text_scene: Option<TextScene>,
        ui_tree: Option<UiTree>,
        model: Option<Box<dyn UiApp>>,
        on_event: impl FnMut(RuntimeEvent) + 'static,
    ) -> Self {
        Self {
            window_config,
            renderer_config,
            window: None,
            renderer: Rc::new(RefCell::new(RendererState::Loading)),
            renderer_announced: false,
            text_engine,
            text_scene,
            ui_tree,
            model,
            ui_layout: None,
            layout_engine: LayoutEngine::new(),
            prepared_text: None,
            viewport: Size::default(),
            scale_factor: 1.0,
            fatal_error: None,
            on_event: Box::new(on_event),
        }
    }

    fn prepare_text(&mut self) -> Result<(), RuntimeError> {
        if let Some(ui) = &mut self.ui_tree {
            self.ui_layout = Some(self.layout_engine.compute(
                ui,
                &mut self.text_engine,
                self.viewport,
            )?);
        }
        let scene = self
            .ui_layout
            .as_ref()
            .map(|layout| &layout.text)
            .or(self.text_scene.as_ref());
        self.prepared_text = scene.map(|scene| self.text_engine.prepare(scene, self.scale_factor));
        Ok(())
    }

    fn update_viewport(&mut self, width: u32, height: u32) {
        self.viewport = Size::new(
            width as f32 / self.scale_factor,
            height as f32 / self.scale_factor,
        );
    }

    fn prepare_or_exit(&mut self, event_loop: &ActiveEventLoop) -> bool {
        if let Err(error) = self.prepare_text() {
            (self.on_event)(RuntimeEvent::LayoutFailed(error.to_string()));
            self.fatal_error = Some(error);
            event_loop.exit();
            return false;
        }
        true
    }

    fn apply_ui_update(
        &mut self,
        update: InteractionUpdate,
        window: &Window,
        event_loop: &ActiveEventLoop,
    ) {
        let mut rebuild = false;
        for event in update.events {
            if let Some(model) = &mut self.model {
                rebuild |= model.update(&event) == ViewUpdate::Rebuild;
            }
            (self.on_event)(RuntimeEvent::Ui(event));
        }
        let tree_update = if rebuild {
            let root = self.model.as_ref().map(|model| model.view());
            match (root, &mut self.ui_tree) {
                (Some(root), Some(tree)) => tree.update(root),
                _ => TreeUpdate::None,
            }
        } else {
            TreeUpdate::None
        };
        let redraw = match tree_update {
            TreeUpdate::Layout => self.prepare_or_exit(event_loop),
            TreeUpdate::Paint => {
                self.repaint();
                true
            }
            TreeUpdate::None if update.paint_changed => {
                self.repaint();
                true
            }
            TreeUpdate::None => false,
        };
        if tree_update != TreeUpdate::None {
            (self.on_event)(RuntimeEvent::ViewUpdated(tree_update));
        }
        if redraw {
            window.request_redraw();
        }
    }

    fn repaint(&mut self) {
        if let (Some(ui), Some(layout)) = (&self.ui_tree, &mut self.ui_layout) {
            self.layout_engine.repaint(ui, layout);
        }
    }

    fn pointer_moved(&mut self, point: Point, window: &Window, event_loop: &ActiveEventLoop) {
        let Some(layout) = &self.ui_layout else {
            return;
        };
        let Some(ui) = &mut self.ui_tree else {
            return;
        };
        let update = ui.pointer_moved(point, &layout.hit_regions);
        self.apply_ui_update(update, window, event_loop);
    }

    fn pointer_left(&mut self, window: &Window, event_loop: &ActiveEventLoop) {
        if let Some(ui) = &mut self.ui_tree {
            let update = ui.pointer_left();
            self.apply_ui_update(update, window, event_loop);
        }
    }

    fn primary_button(
        &mut self,
        state: ButtonState,
        window: &Window,
        event_loop: &ActiveEventLoop,
    ) {
        let Some(ui) = &mut self.ui_tree else {
            return;
        };
        let update = match state {
            ButtonState::Pressed => {
                let Some(layout) = &self.ui_layout else {
                    return;
                };
                ui.primary_pressed(&layout.hit_regions)
            }
            ButtonState::Released => ui.primary_released(),
        };
        self.apply_ui_update(update, window, event_loop);
    }

    fn window_focus(&mut self, focused: bool, window: &Window, event_loop: &ActiveEventLoop) {
        if !focused && let Some(ui) = &mut self.ui_tree {
            let update = ui.window_blurred();
            self.apply_ui_update(update, window, event_loop);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn initialize_renderer(&mut self, window: &Arc<Window>, event_loop: &ActiveEventLoop) {
        let size = window.inner_size();
        match pollster::block_on(SurfaceRenderer::new(
            Arc::clone(window),
            size.width,
            size.height,
            self.renderer_config,
        )) {
            Ok(renderer) => {
                *self.renderer.borrow_mut() = RendererState::Ready(Box::new(renderer));
                window.request_redraw();
            }
            Err(error) => {
                (self.on_event)(RuntimeEvent::RendererFailed(error.to_string()));
                self.fatal_error = Some(error.into());
                event_loop.exit();
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn initialize_renderer(&mut self, window: &Arc<Window>, _event_loop: &ActiveEventLoop) {
        let size = window.inner_size();
        let window = Arc::clone(window);
        let renderer = Rc::clone(&self.renderer);
        let config = self.renderer_config;

        wasm_bindgen_futures::spawn_local(async move {
            let mut result =
                SurfaceRenderer::new(Arc::clone(&window), size.width, size.height, config).await;
            if let Ok(renderer) = &mut result {
                let current_size = window.inner_size();
                renderer.resize(current_size.width, current_size.height);
            }
            *renderer.borrow_mut() = match result {
                Ok(renderer) => RendererState::Ready(Box::new(renderer)),
                Err(error) => RendererState::Failed(error.to_string()),
            };
            window.request_redraw();
        });
    }

    fn render(&mut self, event_loop: &ActiveEventLoop) {
        let Some(window) = self.window.as_ref().map(Arc::clone) else {
            return;
        };
        let mut state = self.renderer.borrow_mut();

        #[cfg(target_arch = "wasm32")]
        {
            if let RendererState::Failed(message) = &*state {
                (self.on_event)(RuntimeEvent::RendererFailed(message.clone()));
                event_loop.exit();
                return;
            }
        }
        let RendererState::Ready(renderer) = &mut *state else {
            return;
        };
        if !self.renderer_announced {
            (self.on_event)(RuntimeEvent::RendererReady);
            self.renderer_announced = true;
        }

        let rendered = match (self.prepared_text.as_ref(), self.ui_layout.as_ref()) {
            (Some(text), Some(layout)) => renderer.render_ui(
                &mut self.text_engine,
                text,
                &layout.display_list,
                self.scale_factor,
            ),
            (Some(text), None) => renderer.render_text(&mut self.text_engine, text),
            (None, _) => renderer.render(),
        };
        let result = match rendered {
            Ok(RenderStatus::Presented | RenderStatus::Skipped) => Ok(()),
            Ok(RenderStatus::Reconfigure) => {
                let size = window.inner_size();
                renderer.resize(size.width, size.height);
                window.request_redraw();
                Ok(())
            }
            Ok(RenderStatus::RecreateSurface) => renderer
                .recreate_surface(Arc::clone(&window))
                .inspect(|()| window.request_redraw()),
            Err(error) => Err(error),
        };
        if let Err(error) = result {
            (self.on_event)(RuntimeEvent::RendererFailed(error.to_string()));
            self.fatal_error = Some(error.into());
            event_loop.exit();
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        match event_loop.create_window(self.window_config.clone().into_attributes()) {
            Ok(window) => {
                let window = Arc::new(window);
                let size = window.inner_size();
                self.scale_factor = window.scale_factor() as f32;
                self.update_viewport(size.width, size.height);
                if !self.prepare_or_exit(event_loop) {
                    return;
                }
                (self.on_event)(RuntimeEvent::Platform(PlatformEvent::Opened {
                    width: size.width,
                    height: size.height,
                    scale_factor: window.scale_factor(),
                }));
                self.initialize_renderer(&window, event_loop);
                self.window = Some(window);
            }
            Err(error) => {
                (self.on_event)(RuntimeEvent::Platform(PlatformEvent::WindowCreationFailed(
                    error.to_string(),
                )));
                self.fatal_error = Some(PlatformError::from(error).into());
                event_loop.exit();
            }
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        (self.on_event)(RuntimeEvent::Platform(PlatformEvent::Suspended));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref().map(Arc::clone) else {
            return;
        };
        if window.id() != window_id {
            return;
        }

        let platform_event = match event {
            WindowEvent::CloseRequested => PlatformEvent::CloseRequested,
            WindowEvent::Resized(size) => {
                if let RendererState::Ready(renderer) = &mut *self.renderer.borrow_mut() {
                    renderer.resize(size.width, size.height);
                }
                self.update_viewport(size.width, size.height);
                if !self.prepare_or_exit(event_loop) {
                    return;
                }
                PlatformEvent::Resized {
                    width: size.width,
                    height: size.height,
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = scale_factor as f32;
                let size = window.inner_size();
                self.update_viewport(size.width, size.height);
                if !self.prepare_or_exit(event_loop) {
                    return;
                }
                PlatformEvent::ScaleFactorChanged(scale_factor)
            }
            WindowEvent::CursorMoved { position, .. } => {
                let point = Point::new(
                    position.x as f32 / self.scale_factor,
                    position.y as f32 / self.scale_factor,
                );
                self.pointer_moved(point, &window, event_loop);
                PlatformEvent::PointerMoved {
                    x: point.x,
                    y: point.y,
                }
            }
            WindowEvent::CursorEntered { .. } => PlatformEvent::PointerEntered,
            WindowEvent::CursorLeft { .. } => {
                self.pointer_left(&window, event_loop);
                PlatformEvent::PointerLeft
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let state = button_state(state);
                let button = pointer_button(button);
                if button == PointerButton::Primary {
                    self.primary_button(state, &window, event_loop);
                }
                PlatformEvent::PointerButton { button, state }
            }
            WindowEvent::Focused(focused) => {
                self.window_focus(focused, &window, event_loop);
                PlatformEvent::Focused(focused)
            }
            WindowEvent::RedrawRequested => {
                self.render(event_loop);
                PlatformEvent::RedrawRequested
            }
            _ => return,
        };

        if platform_event.requires_redraw() {
            window.request_redraw();
        }
        if platform_event.closes_window() {
            event_loop.exit();
        }
        (self.on_event)(RuntimeEvent::Platform(platform_event));
    }
}

const fn button_state(state: ElementState) -> ButtonState {
    match state {
        ElementState::Pressed => ButtonState::Pressed,
        ElementState::Released => ButtonState::Released,
    }
}

const fn pointer_button(button: MouseButton) -> PointerButton {
    match button {
        MouseButton::Left => PointerButton::Primary,
        MouseButton::Right => PointerButton::Secondary,
        MouseButton::Middle => PointerButton::Middle,
        MouseButton::Back => PointerButton::Back,
        MouseButton::Forward => PointerButton::Forward,
        MouseButton::Other(value) => PointerButton::Other(value),
    }
}
