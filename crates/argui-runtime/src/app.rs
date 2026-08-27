use crate::translate::{
    button_state, ime_input, key_input, modifiers_state, pointer_button, scroll_delta,
};
use crate::{
    LayoutBounds, LayoutSnapshot, RuntimeError, RuntimeEvent, UiApp, ViewUpdate,
    animation::RuntimeAnimations, event::UserEvent,
};
use argui_core::{Point, Size};
use argui_inspect::InspectorHandle;
use argui_layout::{LayoutEngine, LayoutOutput};
use argui_paint::{ImageAsset, VectorAsset};
use argui_platform::{
    ButtonState, Modifiers, PlatformError, PlatformEvent, PointerButton, ScrollDelta, WindowConfig,
};
use argui_render::{EffectShader, RendererConfig, SurfaceRenderer};
use argui_text::{PreparedText, TextEngine, TextScene};
use argui_ui::{InteractionUpdate, UiTree};
use std::{cell::RefCell, rc::Rc, sync::Arc};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

mod frame;
mod inspect;
mod renderer;
mod scroll;

enum RendererState {
    Loading,
    Ready(Box<SurfaceRenderer>),
    #[cfg(target_arch = "wasm32")]
    Failed(String),
}

pub(crate) struct Application {
    window_config: WindowConfig,
    pub(super) renderer_config: RendererConfig,
    window: Option<Arc<Window>>,
    renderer: Rc<RefCell<RendererState>>,
    renderer_announced: bool,
    effect_shaders: Vec<EffectShader>,
    image_assets: Vec<ImageAsset>,
    vector_assets: Vec<VectorAsset>,
    pub(super) inspector: Option<InspectorHandle>,
    pub(super) text_engine: TextEngine,
    text_scene: Option<TextScene>,
    pub(super) ui_tree: Option<UiTree>,
    pub(super) animations: RuntimeAnimations,
    pub(super) model: Option<Box<dyn UiApp>>,
    pub(super) ui_layout: Option<LayoutOutput>,
    pub(super) layout_engine: LayoutEngine,
    pub(super) prepared_text: Option<PreparedText>,
    viewport: Size,
    scale_factor: f32,
    pointer: Option<Point>,
    modifiers: Modifiers,
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) clipboard: argui_platform::Clipboard,
    #[cfg(target_arch = "wasm32")]
    pub(super) event_proxy: Option<winit::event_loop::EventLoopProxy<UserEvent>>,
    pending_scrollbar_drag: Option<Point>,
    pending_pointer_scroll: Option<ScrollDelta>,
    scroll_inertia: scroll::ScrollInertia,
    pending_ui_frame: frame::PendingUiFrame,
    pub(crate) fatal_error: Option<RuntimeError>,
    pub(super) on_event: Box<dyn FnMut(RuntimeEvent)>,
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
        let inspector = model.as_deref().and_then(UiApp::inspector);
        let renderer_config = RendererConfig {
            profiling: renderer_config.profiling || inspector.is_some(),
            ..renderer_config
        };
        let effect_shaders = model
            .as_deref()
            .map(UiApp::effect_shaders)
            .unwrap_or_default()
            .to_vec();
        let image_assets = model
            .as_deref()
            .map(UiApp::image_assets)
            .unwrap_or_default();
        let vector_assets = model
            .as_deref()
            .map(UiApp::vector_assets)
            .unwrap_or_default();
        Self {
            window_config,
            renderer_config,
            window: None,
            renderer: Rc::new(RefCell::new(RendererState::Loading)),
            renderer_announced: false,
            effect_shaders,
            image_assets,
            vector_assets,
            inspector,
            text_engine,
            text_scene,
            ui_tree,
            animations: RuntimeAnimations::new(model.as_deref()),
            model,
            ui_layout: None,
            layout_engine: LayoutEngine::new(),
            prepared_text: None,
            viewport: Size::default(),
            scale_factor: 1.0,
            pointer: None,
            modifiers: Modifiers::default(),
            #[cfg(not(target_arch = "wasm32"))]
            clipboard: argui_platform::Clipboard::new(),
            #[cfg(target_arch = "wasm32")]
            event_proxy: None,
            pending_scrollbar_drag: None,
            pending_pointer_scroll: None,
            scroll_inertia: scroll::ScrollInertia::default(),
            pending_ui_frame: frame::PendingUiFrame::default(),
            fatal_error: None,
            on_event: Box::new(on_event),
        }
    }

    fn prepare_text(&mut self) -> Result<(), RuntimeError> {
        if let Some(layout) = self.compute_ui_layout()? {
            let snapshot = LayoutSnapshot {
                viewport: layout.viewport,
                nodes: layout
                    .nodes
                    .iter()
                    .map(|node| LayoutBounds {
                        node: node.node,
                        key: self
                            .ui_tree
                            .as_ref()
                            .and_then(|ui| ui.key(node.node))
                            .map(ToOwned::to_owned),
                        bounds: node.bounds,
                    })
                    .collect(),
            };
            self.ui_layout = Some(layout);
            let rebuild = self
                .model
                .as_mut()
                .is_some_and(|model| model.layout_changed(&snapshot) == ViewUpdate::Rebuild);
            if rebuild && let Some(root) = self.inspected_view() {
                if let Some(ui) = &mut self.ui_tree {
                    ui.update(root);
                }
                self.ui_layout = self.compute_ui_layout()?;
            }
            self.publish_inspection();
        }
        let scene = self
            .ui_layout
            .as_ref()
            .map(|layout| &layout.text)
            .or(self.text_scene.as_ref());
        self.prepared_text = scene.map(|scene| self.text_engine.prepare(scene, self.scale_factor));
        Ok(())
    }

    fn compute_ui_layout(&mut self) -> Result<Option<LayoutOutput>, RuntimeError> {
        let Some(ui) = &mut self.ui_tree else {
            return Ok(None);
        };
        self.layout_engine
            .compute(ui, &mut self.text_engine, self.viewport)
            .map(Some)
            .map_err(Into::into)
    }

    fn update_viewport(&mut self, width: u32, height: u32) {
        self.viewport = Size::new(
            width as f32 / self.scale_factor,
            height as f32 / self.scale_factor,
        );
    }

    pub(super) fn prepare_or_exit(&mut self, event_loop: &ActiveEventLoop) -> bool {
        if let Err(error) = self.prepare_text() {
            (self.on_event)(RuntimeEvent::LayoutFailed(error.to_string()));
            self.fatal_error = Some(error);
            event_loop.exit();
            return false;
        }
        true
    }

    pub(super) fn apply_ui_update(
        &mut self,
        update: InteractionUpdate,
        window: &Window,
        event_loop: &ActiveEventLoop,
    ) {
        let mut clipboard = update.clipboard.clone();
        let mut rebuild = false;
        for event in &update.events {
            if let Some(model) = &mut self.model {
                rebuild |= model.update(event) == ViewUpdate::Rebuild;
            }
            (self.on_event)(RuntimeEvent::Ui(event.clone()));
        }
        if clipboard.is_none() {
            clipboard = self
                .model
                .as_mut()
                .and_then(|model| model.take_clipboard_request());
        }
        let animation_changed = self.sync_animations();
        self.pending_ui_frame.merge(&update, rebuild);
        let scroll_request = self
            .model
            .as_mut()
            .and_then(|model| model.take_scroll_request());
        self.pending_ui_frame.request_scroll(scroll_request);
        if self.pending_ui_frame.needs_frame() || animation_changed {
            window.request_redraw();
        }
        if let Some(request) = clipboard {
            self.clipboard_request(request, window, event_loop);
        }
    }

    pub(super) fn repaint(&mut self) {
        if let (Some(ui), Some(layout)) = (&self.ui_tree, &mut self.ui_layout) {
            self.layout_engine.repaint(ui, layout);
        }
        self.publish_inspection();
    }

    fn pointer_moved(&mut self, point: Point, window: &Window, event_loop: &ActiveEventLoop) {
        self.pointer = Some(point);
        let Some(layout) = &self.ui_layout else {
            return;
        };
        let Some(ui) = &mut self.ui_tree else {
            return;
        };
        if ui.scrollbar_dragging() {
            self.pending_scrollbar_drag = Some(point);
            window.request_redraw();
            return;
        }
        if ui.text_cursor_dragging()
            && let Some(node) = ui.focused_node()
            && let Some(region) = layout.text_inputs.iter().find(|region| region.node == node)
        {
            let local = local_point(layout, node, point).unwrap_or(point);
            let update = ui.drag_text_position(node, region.closest_position(local));
            self.apply_ui_update(update, window, event_loop);
            return;
        }
        let blocked = layout
            .scroll_regions
            .iter()
            .rev()
            .any(|region| region.scrollbar_contains(point));
        let hit_regions = if blocked {
            &[]
        } else {
            layout.hit_regions.as_slice()
        };
        let update = ui.pointer_moved(point, hit_regions);
        self.apply_ui_update(update, window, event_loop);
    }

    fn flush_scrollbar_drag(&mut self, window: &Window, event_loop: &ActiveEventLoop) {
        let Some(point) = self.pending_scrollbar_drag.take() else {
            return;
        };
        let (Some(layout), Some(ui)) = (&self.ui_layout, &mut self.ui_tree) else {
            return;
        };
        if let Some(update) = ui.scrollbar_dragged(point, &layout.scroll_regions) {
            self.apply_ui_update(update, window, event_loop);
        }
    }

    fn pointer_left(&mut self, window: &Window, event_loop: &ActiveEventLoop) {
        self.scroll_inertia.cancel();
        self.flush_pointer_scroll(window, event_loop);
        self.flush_scrollbar_drag(window, event_loop);
        self.pointer = None;
        if let Some(ui) = &mut self.ui_tree {
            ui.scrollbar_released();
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
        self.scroll_inertia.cancel();
        self.flush_pointer_scroll(window, event_loop);
        if state == ButtonState::Released {
            self.flush_scrollbar_drag(window, event_loop);
        }
        let Some(layout) = &self.ui_layout else {
            return;
        };
        let placement = self.pointer.and_then(|point| {
            layout.text_inputs.iter().rev().find_map(|region| {
                let point = local_point(layout, region.node, point).unwrap_or(point);
                region
                    .hit_position(point)
                    .map(|position| (region.node, position))
            })
        });
        let Some(ui) = &mut self.ui_tree else {
            return;
        };
        if state == ButtonState::Pressed
            && let Some(point) = self.pointer
            && let Some(update) = ui.scrollbar_pressed(point, &layout.scroll_regions)
        {
            self.apply_ui_update(update, window, event_loop);
            return;
        }
        if state == ButtonState::Released && ui.scrollbar_released() {
            return;
        }
        let update = match state {
            ButtonState::Pressed => ui.primary_pressed(&layout.hit_regions),
            ButtonState::Released => {
                ui.release_text_cursor();
                ui.primary_released()
            }
        };
        self.apply_ui_update(update, window, event_loop);
        if state == ButtonState::Pressed
            && let Some((node, position)) = placement
            && let Some(ui) = &mut self.ui_tree
        {
            let update = ui.place_text_position(node, position, self.modifiers.shift);
            self.apply_ui_update(update, window, event_loop);
        }
        self.update_ime(window);
    }

    fn window_focus(&mut self, focused: bool, window: &Window, event_loop: &ActiveEventLoop) {
        if !focused {
            self.scroll_inertia.cancel();
            self.flush_pointer_scroll(window, event_loop);
            self.flush_scrollbar_drag(window, event_loop);
        }
        if !focused && let Some(ui) = &mut self.ui_tree {
            ui.scrollbar_released();
            let update = ui.window_blurred();
            self.apply_ui_update(update, window, event_loop);
        }
    }
}

fn local_point(layout: &LayoutOutput, node: argui_ui::NodeId, point: Point) -> Option<Point> {
    layout
        .hit_regions
        .iter()
        .find(|region| region.node == node)
        .and_then(|region| region.transform.inverse())
        .map(|inverse| inverse.transform_point(point))
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl ApplicationHandler<UserEvent> for Application {
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

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = event_loop;
            match event {}
        }
        #[cfg(target_arch = "wasm32")]
        {
            let Some(window) = self.window.as_ref().map(Arc::clone) else {
                return;
            };
            match event {
                UserEvent::ClipboardText(text) => {
                    if let Some(ui) = &mut self.ui_tree {
                        let update = ui.paste_text(&text);
                        self.apply_ui_update(update, &window, event_loop);
                    }
                }
            }
        }
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
            WindowEvent::MouseWheel { delta, phase, .. } => {
                let delta = scroll_delta(delta, self.scale_factor);
                self.queue_pointer_scroll(delta, phase, &window, event_loop);
                PlatformEvent::PointerScrolled(delta)
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers_state(modifiers.state());
                return;
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let input = key_input(event, self.modifiers);
                self.keyboard_input(&input, &window, event_loop);
                PlatformEvent::Keyboard(input)
            }
            WindowEvent::Ime(ime) => {
                let input = ime_input(ime);
                self.ime_input(input.clone(), &window, event_loop);
                PlatformEvent::Ime(input)
            }
            WindowEvent::Focused(focused) => {
                self.window_focus(focused, &window, event_loop);
                PlatformEvent::Focused(focused)
            }
            WindowEvent::RedrawRequested => {
                self.advance_pointer_inertia(&window);
                self.flush_pointer_scroll(&window, event_loop);
                self.flush_scrollbar_drag(&window, event_loop);
                self.flush_ui_frame(event_loop);
                self.animate(&window, event_loop);
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
