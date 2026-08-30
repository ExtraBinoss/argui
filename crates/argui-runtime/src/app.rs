use crate::{
    AnyEntity, LayoutBounds, LayoutSnapshot, RuntimeError, RuntimeEvent, ViewUpdate,
    animation::RuntimeAnimations,
};
use argui_core::{Point, PointerEvent, PointerId, PointerPhase, Size};
use argui_inspect::InspectorHandle;
use argui_layout::{LayoutEngine, LayoutOutput};
use argui_paint::{ImageAsset, VectorAsset};
use argui_platform::{ApplicationIdentity, ButtonState, Modifiers, ScrollDelta, WindowConfig};
use argui_render::{RendererConfig, RendererDevice, SurfaceRenderer};
use argui_text::{PreparedText, TextEngine, TextScene};
use argui_ui::{InteractionUpdate, UiTree};
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};
use web_time::Instant;
use winit::{
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

mod accessibility;
mod cursor;
mod frame;
mod inspect;
mod lifecycle;
mod preferences;
mod renderer;
mod scroll;
mod window;

enum RendererState {
    Loading,
    Ready(Box<SurfaceRenderer>),
    #[cfg(target_arch = "wasm32")]
    Failed(String),
}

pub(crate) struct Application {
    window_config: WindowConfig,
    identity: Option<ApplicationIdentity>,
    pub(crate) window_key: argui_platform::WindowKey,
    exit_on_close: bool,
    initial_visible: bool,
    preference_overrides: argui_platform::PreferenceOverrides,
    preferences: argui_platform::SystemPreferences,
    theme_request: crate::ThemeRequest,
    environment: crate::WindowEnvironment,
    pub(super) renderer_config: RendererConfig,
    window: Option<Arc<Window>>,
    #[cfg(not(target_arch = "wasm32"))]
    accessibility: Option<accesskit_winit::Adapter>,
    #[cfg(not(target_arch = "wasm32"))]
    semantic_snapshot: Option<std::sync::Arc<std::sync::Mutex<argui_accessibility::SemanticTree>>>,
    #[cfg(target_arch = "wasm32")]
    dom_accessibility: Option<argui_accessibility::DomTree>,
    renderer: Rc<RefCell<RendererState>>,
    renderer_device: Rc<RefCell<Option<RendererDevice>>>,
    renderer_announced: bool,
    image_assets: Vec<ImageAsset>,
    vector_assets: Vec<VectorAsset>,
    pub(super) inspector: Option<InspectorHandle>,
    pub(super) text_engine: TextEngine,
    text_scene: Option<TextScene>,
    pub(super) ui_tree: Option<UiTree>,
    pub(super) animations: RuntimeAnimations,
    pub(super) model: Option<AnyEntity>,
    pub(super) ui_layout: Option<LayoutOutput>,
    pub(super) layout_engine: LayoutEngine,
    pub(super) prepared_text: Option<PreparedText>,
    viewport: Size,
    scale_factor: f32,
    pointer: Option<Point>,
    pointer_buttons: u16,
    touch_points: HashMap<PointerId, Point>,
    primary_touch: Option<PointerId>,
    input_epoch: Instant,
    last_cursor: argui_ui::CursorIcon,
    modifiers: Modifiers,
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) clipboard: argui_platform::Clipboard,
    pub(super) event_proxy: Option<winit::event_loop::EventLoopProxy<crate::event::UserEvent>>,
    pending_scrollbar_drag: Option<Point>,
    pending_pointer_scroll: Option<ScrollDelta>,
    scroll_inertia: scroll::ScrollInertia,
    window_drag: window::WindowDragState,
    pending_window_frame: frame::PendingWindowFrame,
    pub(super) pending_ui_frame: frame::PendingUiFrame,
    pub(super) frame_record: argui_inspect::FrameRecord,
    pub(super) last_redraw: Option<Instant>,
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
        model: Option<AnyEntity>,
        on_event: impl FnMut(RuntimeEvent) + 'static,
    ) -> Self {
        let inspector = model.as_ref().and_then(AnyEntity::inspector);
        let renderer_config = RendererConfig {
            profiling: renderer_config.profiling || inspector.is_some(),
            ..renderer_config
        };
        let image_assets = model
            .as_ref()
            .map(AnyEntity::image_assets)
            .unwrap_or_default();
        let vector_assets = model
            .as_ref()
            .map(AnyEntity::vector_assets)
            .unwrap_or_default();
        let mut layout_engine = LayoutEngine::new();
        layout_engine.set_assets(&image_assets, &vector_assets);
        Self {
            window_config,
            identity: None,
            window_key: argui_platform::WindowKey::main(),
            exit_on_close: true,
            initial_visible: true,
            preference_overrides: argui_platform::PreferenceOverrides::default(),
            preferences: argui_platform::SystemPreferences::default(),
            theme_request: crate::ThemeRequest::default(),
            environment: crate::WindowEnvironment::default(),
            renderer_config,
            window: None,
            #[cfg(not(target_arch = "wasm32"))]
            accessibility: None,
            #[cfg(not(target_arch = "wasm32"))]
            semantic_snapshot: None,
            #[cfg(target_arch = "wasm32")]
            dom_accessibility: None,
            renderer: Rc::new(RefCell::new(RendererState::Loading)),
            renderer_device: Rc::new(RefCell::new(None)),
            renderer_announced: false,
            image_assets,
            vector_assets,
            inspector,
            text_engine,
            text_scene,
            ui_tree,
            animations: RuntimeAnimations::new(model.as_ref()),
            model,
            ui_layout: None,
            layout_engine,
            prepared_text: None,
            viewport: Size::default(),
            scale_factor: 1.0,
            pointer: None,
            pointer_buttons: 0,
            touch_points: HashMap::new(),
            primary_touch: None,
            input_epoch: Instant::now(),
            last_cursor: argui_ui::CursorIcon::Default,
            modifiers: Modifiers::default(),
            #[cfg(not(target_arch = "wasm32"))]
            clipboard: argui_platform::Clipboard::new(),
            event_proxy: None,
            pending_scrollbar_drag: None,
            pending_pointer_scroll: None,
            scroll_inertia: scroll::ScrollInertia::default(),
            window_drag: window::WindowDragState::default(),
            pending_window_frame: frame::PendingWindowFrame::default(),
            pending_ui_frame: frame::PendingUiFrame::default(),
            frame_record: argui_inspect::FrameRecord::default(),
            last_redraw: None,
            fatal_error: None,
            on_event: Box::new(on_event),
        }
    }

    pub(crate) fn identified(
        mut self,
        identity: ApplicationIdentity,
        window_key: argui_platform::WindowKey,
    ) -> Self {
        self.identity = Some(identity);
        self.window_key = window_key;
        self.exit_on_close = false;
        self
    }

    pub(crate) fn shared_renderer_device(
        mut self,
        renderer_device: Rc<RefCell<Option<RendererDevice>>>,
    ) -> Self {
        self.renderer_device = renderer_device;
        self
    }

    pub(crate) const fn initially_visible(mut self, visible: bool) -> Self {
        self.initial_visible = visible;
        self
    }

    pub(crate) fn window_id(&self) -> Option<WindowId> {
        self.window.as_deref().map(Window::id)
    }

    pub(crate) fn window(&self) -> Option<&Window> {
        self.window.as_deref()
    }

    pub(crate) fn invalidate(&mut self, update: ViewUpdate) {
        match update {
            ViewUpdate::None => return,
            ViewUpdate::Paint => self.pending_ui_frame.request_paint(),
            ViewUpdate::Rebuild => self.pending_ui_frame.request_rebuild(),
        }
        if let Some(window) = &self.window {
            window.request_redraw();
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
            let rebuild = self.model.as_ref().is_some_and(|model| {
                model.layout_changed(&snapshot);
                model.take_effects().update == ViewUpdate::Rebuild
            });
            if rebuild && let Some(root) = self.inspected_view() {
                if let Some(ui) = &mut self.ui_tree {
                    ui.update(root);
                }
                self.ui_layout = self.compute_ui_layout()?;
            }
            self.publish_inspection();
            self.paint_inspection_highlight();
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
        mut update: InteractionUpdate,
        window: &Window,
        event_loop: &ActiveEventLoop,
    ) {
        let mut clipboard = update.clipboard.clone();
        let mut scroll_request = None;
        let mut focus_request = None;
        let mut theme_request = None;
        let mut rebuild = false;
        for event in &update.events {
            if let Some(model) = &self.model {
                model.event(event);
                let effects = model.take_effects();
                match effects.update {
                    ViewUpdate::None => {}
                    ViewUpdate::Paint => update.paint_changed = true,
                    ViewUpdate::Rebuild => rebuild = true,
                }
                if effects.clipboard.is_some() {
                    clipboard = effects.clipboard;
                }
                if effects.scroll.is_some() {
                    scroll_request = effects.scroll;
                }
                if effects.focus.is_some() {
                    focus_request = effects.focus;
                }
                if effects.theme.is_some() {
                    theme_request = effects.theme;
                }
            }
            (self.on_event)(RuntimeEvent::Ui(event.clone()));
        }
        let animation_changed = self.sync_animations();
        if let Some(request) = theme_request {
            self.apply_theme_request(request);
            rebuild = true;
        }
        self.pending_ui_frame.merge(&update, rebuild);
        self.pending_ui_frame.request_scroll(scroll_request);
        self.pending_ui_frame.request_focus(focus_request);
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
        self.paint_inspection_highlight();
    }

    fn pointer_moved(&mut self, point: Point, window: &Window, event_loop: &ActiveEventLoop) {
        self.pointer = Some(point);
        self.refresh_cursor(window);
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
        let scrollbar = argui_ui::scrollbar_at(point, &layout.scroll_regions, &layout.hit_regions);
        let mut update =
            ui.scrollbar_pointer_moved(Some(point), scrollbar.map_or(&[], std::slice::from_ref));
        let blocked = scrollbar.is_some();
        let hit_regions = if blocked {
            &[]
        } else {
            layout.hit_regions.as_slice()
        };
        update.merge(ui.pointer_event(
            PointerEvent {
                buttons: self.pointer_buttons,
                timestamp: self.input_epoch.elapsed(),
                ..PointerEvent::mouse(PointerPhase::Moved, point)
            },
            hit_regions,
        ));
        self.apply_ui_update(update, window, event_loop);
    }

    fn touch_pointer(
        &mut self,
        mut event: PointerEvent,
        window: &Window,
        event_loop: &ActiveEventLoop,
    ) -> PointerEvent {
        if event.phase == PointerPhase::Pressed && self.primary_touch.is_none() {
            self.primary_touch = Some(event.id);
        }
        event.primary = self.primary_touch == Some(event.id);
        let finger_delta = match event.phase {
            PointerPhase::Pressed => {
                self.touch_points.insert(event.id, event.position);
                None
            }
            PointerPhase::Moved => self
                .touch_points
                .insert(event.id, event.position)
                .map(|previous| {
                    Point::new(event.position.x - previous.x, event.position.y - previous.y)
                })
                .filter(|_| event.primary),
            PointerPhase::Released | PointerPhase::Cancelled | PointerPhase::Left => {
                self.touch_points.remove(&event.id);
                None
            }
            PointerPhase::Entered => None,
        };
        let update = if let (Some(layout), Some(ui)) = (&self.ui_layout, &mut self.ui_tree) {
            let mut update = ui.pointer_event(event, &layout.hit_regions);
            if let Some(delta) = finger_delta {
                update.merge(ui.scroll(
                    event.position,
                    ScrollDelta::Pixels(delta),
                    &layout.scroll_regions,
                ));
            }
            Some(update)
        } else {
            None
        };
        if let Some(update) = update {
            self.apply_ui_update(update, window, event_loop);
        }
        if matches!(event.phase, PointerPhase::Pressed | PointerPhase::Released) {
            self.update_ime(window);
        }
        if matches!(
            event.phase,
            PointerPhase::Released | PointerPhase::Cancelled | PointerPhase::Left
        ) && event.primary
        {
            self.primary_touch = self.touch_points.keys().min_by_key(|id| id.get()).copied();
        }
        event
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
        let point = self.pointer.unwrap_or_default();
        self.pointer = None;
        self.refresh_cursor(window);
        if let Some(ui) = &mut self.ui_tree {
            let mut update = ui.scrollbar_pointer_moved(None, &[]);
            update.merge(ui.pointer_event(
                PointerEvent {
                    buttons: self.pointer_buttons,
                    timestamp: self.input_epoch.elapsed(),
                    ..PointerEvent::mouse(PointerPhase::Left, point)
                },
                &[],
            ));
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
        if state == ButtonState::Pressed && self.handle_window_drag(window) {
            return;
        }
        let Some(layout) = &self.ui_layout else {
            return;
        };
        let placement = self.pointer.and_then(|point| {
            let target = layout
                .hit_regions
                .iter()
                .rev()
                .find(|region| region.contains(point))?
                .node;
            layout.text_inputs.iter().rev().find_map(|region| {
                if region.node != target {
                    return None;
                }
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
            && let Some(region) =
                argui_ui::scrollbar_at(point, &layout.scroll_regions, &layout.hit_regions)
            && let Some(update) = ui.scrollbar_pressed(point, std::slice::from_ref(region))
        {
            self.apply_ui_update(update, window, event_loop);
            return;
        }
        if state == ButtonState::Released
            && let Some(mut update) = ui.scrollbar_released()
        {
            let scrollbar = self.pointer.and_then(|point| {
                argui_ui::scrollbar_at(point, &layout.scroll_regions, &layout.hit_regions)
            });
            update.merge(ui.scrollbar_pointer_moved(
                self.pointer,
                scrollbar.map_or(&[], std::slice::from_ref),
            ));
            self.apply_ui_update(update, window, event_loop);
            return;
        }
        if state == ButtonState::Released {
            ui.release_text_cursor();
        }
        let phase = match state {
            ButtonState::Pressed => PointerPhase::Pressed,
            ButtonState::Released => PointerPhase::Released,
        };
        let point = self.pointer.unwrap_or_default();
        let update = ui.pointer_event(
            PointerEvent {
                button: Some(argui_core::PointerButton::Primary),
                buttons: self.pointer_buttons,
                timestamp: self.input_epoch.elapsed(),
                ..PointerEvent::mouse(phase, point)
            },
            &layout.hit_regions,
        );
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
            let mut update = ui.scrollbar_released().unwrap_or_default();
            update.merge(ui.scrollbar_pointer_moved(None, &[]));
            update.merge(ui.window_blurred());
            self.apply_ui_update(update, window, event_loop);
        }
        if focused && let (Some(ui), Some(layout)) = (&mut self.ui_tree, &self.ui_layout) {
            let update = ui.window_focused(&layout.hit_regions);
            self.apply_ui_update(update, window, event_loop);
            self.update_ime(window);
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
