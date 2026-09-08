use argui_animation::{Frame, Spring, SpringConfig};
use argui_core::{Point, Rect};
use argui_inspect::{FrameRecord, InspectNodeId, InspectorHandle};
use argui_paint::{ImageAsset, VectorAsset};
use argui_runtime::{Context, Entity, LayoutSnapshot, Render, ScrollRequest, ViewUpdate};
use argui_ui::{ClipboardRequest, Element, EventType, UiEvent};

use crate::{icons::DevtoolsIcons, view};

mod interaction;
mod tree;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum Tab {
    #[default]
    Elements,
    Profiling,
}

pub(crate) struct ProfileExtents {
    pub frames: f32,
    pub frames_header: f32,
    pub gpu: f32,
    pub gpu_header: f32,
}

pub struct DevtoolsHost<A> {
    pub(crate) app: Entity<A>,
    pub(crate) inspector: InspectorHandle,
    pub(crate) open: bool,
    pub(crate) scroll_effect: Option<argui_ui::ScrollEffect>,
    pub(crate) tab: Tab,
    pub(crate) dock_height: f32,
    pub(crate) viewport: Rect,
    pub(crate) app_viewport: Rect,
    pub(crate) picking: bool,
    pub(crate) picker_hovered: Option<InspectNodeId>,
    picker_point: Option<Point>,
    last_pick_point: Option<Point>,
    pub(crate) tree_offset: f32,
    pub(crate) profiling_offset: f32,
    pub(crate) profile_frames: Vec<FrameRecord>,
    pub(crate) frame_cursor: argui_inspect::FrameCursor,
    profile_refresh: argui_animation::Duration,
    detail_refresh: argui_animation::Duration,
    pub(crate) profile_details: FrameRecord,
    pub(crate) detail_cache: std::cell::RefCell<Option<view::DetailCache>>,
    pub(crate) search: String,
    pub(crate) tree_cache: std::cell::RefCell<tree::TreeCache>,
    pub(crate) tree_rows: std::cell::RefCell<argui_widgets::TreeViewCache>,
    pub(crate) collapsed: std::collections::BTreeSet<String>,
    pending_focus: Option<argui_ui::FocusRequest>,
    pub(crate) sections: [bool; 3],
    pub(crate) section_progress: [f32; 3],
    pub(crate) sheet_progress: f32,
    sheet_motion: Spring<f32>,
    section_motion: [Spring<f32>; 3],
    pub(crate) icons: DevtoolsIcons,
    clipboard: Option<ClipboardRequest>,
    pending_scroll: Option<ScrollRequest>,
    pub(crate) splitter: argui_widgets::SplitPane,
    pub(crate) dock_mode: crate::DockMode,
    pub(crate) dock_width: f32,
    pub(crate) tree_height: f32,
    pub(crate) show_properties: bool,
    pub(crate) selected_frame: Option<usize>,
    pub(crate) profile_panel: usize,
    pub(crate) gpu_offset: f32,
    pub(crate) profile_extents: ProfileExtents,
    pub(crate) dock_presence: argui_widgets::Presence,
    pub(crate) reduced_motion: bool,
    pub(crate) dock_highlight: usize,
    pub(crate) detach_available: bool,
    pub(crate) requested_mode: Option<crate::DockMode>,
}

impl<A: Render> DevtoolsHost<A> {
    #[must_use]
    pub fn new(app: A) -> Self {
        let inspector = InspectorHandle::default();
        inspector.set_recording(false);
        inspector.set_gpu_profiling(false);
        Self {
            app: Entity::new(app),
            inspector,
            open: false,
            scroll_effect: Some(argui_effects::EdgeFade::default().scroll()),
            tab: Tab::Elements,
            dock_height: 320.0,
            viewport: Rect::default(),
            app_viewport: Rect::default(),
            picking: false,
            picker_hovered: None,
            picker_point: None,
            last_pick_point: None,
            tree_offset: 0.0,
            profiling_offset: 0.0,
            profile_frames: Vec::new(),
            frame_cursor: argui_inspect::FrameCursor::default(),
            profile_refresh: argui_animation::Duration::ZERO,
            detail_refresh: argui_animation::Duration::ZERO,
            profile_details: FrameRecord::default(),
            detail_cache: std::cell::RefCell::default(),
            search: String::new(),
            tree_cache: tree::cache(),
            tree_rows: std::cell::RefCell::default(),
            collapsed: tree::collapsed(),
            pending_focus: None,
            sections: [true, true, true],
            section_progress: [1.0; 3],
            sheet_progress: 0.0,
            sheet_motion: sheet_spring(0.0),
            section_motion: std::array::from_fn(|_| section_spring(1.0)),
            icons: DevtoolsIcons::embedded(),
            clipboard: None,
            pending_scroll: None,
            splitter: argui_widgets::SplitPane::new(
                "__devtools-splitter",
                argui_widgets::SplitAxis::Vertical,
                320.0,
                180.0,
                5000.0,
            )
            .trailing(true),
            dock_mode: crate::DockMode::Bottom,
            dock_width: 420.0,
            tree_height: 236.0,
            show_properties: false,
            selected_frame: None,
            profile_panel: 0,
            gpu_offset: 0.0,
            profile_extents: ProfileExtents {
                frames: 200.0,
                frames_header: 260.0,
                gpu: 160.0,
                gpu_header: 70.0,
            },
            dock_presence: argui_widgets::Presence::default(),
            reduced_motion: false,
            dock_highlight: 0,
            detach_available: false,
            requested_mode: None,
        }
    }

    #[must_use]
    pub fn open(mut self, open: bool) -> Self {
        self.set_open_immediate(open);
        self
    }

    /// Configures all DevTools list edges; `None` removes their offscreen passes.
    #[must_use]
    pub fn scroll_effect(mut self, effect: Option<argui_ui::ScrollEffect>) -> Self {
        self.scroll_effect = effect;
        self.detail_cache.borrow_mut().take();
        self
    }

    pub(crate) fn set_open_immediate(&mut self, open: bool) {
        self.open = open;
        self.inspector.set_recording(open);
        self.sheet_progress = if open { 1.0 } else { 0.0 };
        self.sheet_motion = sheet_spring(self.sheet_progress);
    }

    #[must_use]
    pub fn inspector(&self) -> InspectorHandle {
        self.inspector.clone()
    }

    fn reveal_tree_node(&mut self, node: InspectNodeId) {
        self.collapsed.clear();
        let location = self.inspector.with_tree(|snapshot| {
            snapshot
                .nodes
                .iter()
                .position(|candidate| candidate.id == node)
                .map(|index| (index, snapshot.nodes.len()))
        });
        let Some((index, node_count)) = location else {
            return;
        };
        self.search.clear();
        let viewport = self.tree_height;
        self.tree_offset = view::tree_list_config(node_count, viewport).scroll_to(
            index,
            argui_ui::VirtualAlignment::Center,
            self.tree_offset,
        );
        self.pending_scroll = Some(ScrollRequest::offset(
            "__devtools-tree",
            Point::new(0.0, self.tree_offset),
        ));
    }

    fn tree_window(&self, offset: f32) -> std::ops::Range<usize> {
        let nodes = self.tree_nodes();
        let tree = self.tree_view(&nodes, None);
        tree.list
            .config(tree.visible_indices().len())
            .window(offset)
            .range
    }
}

impl<A: Render> DevtoolsHost<A> {
    pub fn update(&mut self, event: &UiEvent) -> ViewUpdate {
        self.update_tools(event).unwrap_or(ViewUpdate::None)
    }

    pub fn animation_frame(&mut self, frame: Frame) -> ViewUpdate {
        self.advance_animations(frame, ViewUpdate::None)
    }

    fn advance_animations(&mut self, frame: Frame, app: ViewUpdate) -> ViewUpdate {
        let dock_animating = self.dock_presence.animating();
        let dock_unmounted = self.dock_presence.advance(frame.elapsed);
        let mut profile = false;
        if self.open && self.tab == Tab::Profiling && !self.inspector.paused() {
            self.profile_refresh += frame.elapsed;
            self.detail_refresh += frame.elapsed;
            if self.profile_refresh >= argui_animation::Duration::from_millis(100) {
                self.profile_refresh = argui_animation::Duration::ZERO;
                if self.selected_frame.is_none() {
                    profile = self
                        .inspector
                        .sync_frames(&mut self.profile_frames, &mut self.frame_cursor);
                    if self.detail_refresh >= argui_animation::Duration::from_millis(500) {
                        self.refresh_profile_details();
                    }
                }
            }
        }
        let sheet = self.sheet_motion.advance(frame.elapsed);
        if sheet {
            self.sheet_progress = self.sheet_motion.value().clamp(0.0, 1.0);
        }
        let mut sections = false;
        for (index, motion) in self.section_motion.iter_mut().enumerate() {
            if motion.advance(frame.elapsed) {
                self.section_progress[index] = motion.value().clamp(0.0, 1.0);
                sections = true;
            }
        }
        if dock_unmounted || sheet || sections || profile || app == ViewUpdate::Rebuild {
            ViewUpdate::Rebuild
        } else if dock_animating || app == ViewUpdate::Paint {
            ViewUpdate::Paint
        } else {
            ViewUpdate::None
        }
    }

    pub fn wants_animation_frame(&self) -> bool {
        self.dock_presence.animating()
            || self.sheet_motion.is_active()
            || self.section_motion.iter().any(Spring::is_active)
            || (self.open
                && self.tab == Tab::Profiling
                && !self.inspector.paused()
                && self.selected_frame.is_none())
            || self.app.read(Render::wants_animation_frame)
    }

    pub(crate) fn refresh_profile_details(&mut self) {
        self.detail_refresh = argui_animation::Duration::ZERO;
        self.profile_details = self
            .selected_frame
            .and_then(|index| self.profile_frames.get(index))
            .or_else(|| self.profile_frames.last())
            .cloned()
            .unwrap_or_default();
    }

    /// Update inspector geometry without delivering layout to the application.
    /// Application layout delivery belongs to the retained parent presentation.
    pub fn inspect_layout(&mut self, layout: &LayoutSnapshot) -> ViewUpdate {
        let changed = self.measure_profile(layout);
        self.viewport = layout.viewport;
        if let Some(bounds) = layout.bounds("__devtools-tree") {
            self.tree_height = bounds.size.height;
        }
        let mut application = layout.clone();
        if let Some(bounds) = layout.bounds("__devtools-app-root") {
            application.viewport = bounds;
        }
        self.app_viewport = application.viewport;
        if changed {
            ViewUpdate::Rebuild
        } else {
            ViewUpdate::None
        }
    }

    pub fn image_assets(&self) -> Vec<ImageAsset> {
        self.app.read(Render::image_assets)
    }

    pub fn vector_assets(&self) -> Vec<VectorAsset> {
        let mut assets = self.app.read(Render::vector_assets);
        assets.extend_from_slice(self.icons.assets());
        assets
    }

    pub fn runtime_inspector(&self) -> Option<InspectorHandle> {
        Some(self.inspector.clone())
    }

    pub fn take_clipboard_request(&mut self) -> Option<ClipboardRequest> {
        self.clipboard.take()
    }

    pub fn take_focus_request(&mut self) -> Option<argui_ui::FocusRequest> {
        self.pending_focus.take()
    }

    pub fn take_scroll_request(&mut self) -> Option<ScrollRequest> {
        self.pending_scroll.take()
    }
}

impl<A: Render> Render for DevtoolsHost<A> {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let environment = cx.environment();
        self.reduced_motion = environment.reduced_motion;
        if self.reduced_motion {
            self.dock_presence
                .set_open(self.dock_presence.is_open(), true);
        }
        let app = cx.entity(&self.app);
        let themes = argui_widgets::shadcn(environment.primary);
        let mut root = view::host(self, app, themes.resolve(environment.color_scheme), None);
        for event in EventType::ALL {
            root = root.on(cx
                .listener(event, |host, event, cx| {
                    if let Some(update) = host.update_tools(event) {
                        request_update(cx, update);
                    }
                    if let Some(request) = host.clipboard.take() {
                        cx.write_clipboard(request);
                    }
                    if let Some(argui_ui::FocusRequest::Focus(target)) = host.pending_focus.take() {
                        cx.request_focus(target);
                    }
                    if let Some(request) = host.pending_scroll.take() {
                        cx.scroll(request);
                    }
                })
                .capture(true));
        }
        root
    }

    fn animation_frame(&mut self, frame: Frame, cx: &mut Context<Self>) {
        let update = self.advance_animations(frame, ViewUpdate::None);
        request_update(cx, update);
    }

    fn wants_animation_frame(&self) -> bool {
        DevtoolsHost::wants_animation_frame(self)
    }

    fn layout_changed(&mut self, layout: &LayoutSnapshot, cx: &mut Context<Self>) {
        request_update(cx, self.inspect_layout(layout));
        let mut application = layout.clone();
        application.viewport = self.app_viewport;
        cx.layout_entity(&self.app, &application);
    }

    fn image_assets(&self) -> Vec<ImageAsset> {
        DevtoolsHost::image_assets(self)
    }

    fn vector_assets(&self) -> Vec<VectorAsset> {
        DevtoolsHost::vector_assets(self)
    }

    fn inspector(&self) -> Option<InspectorHandle> {
        self.runtime_inspector()
    }
}

fn request_update<T: Render>(cx: &mut Context<T>, update: ViewUpdate) {
    match update {
        ViewUpdate::None => {}
        ViewUpdate::Paint => cx.request_paint(),
        ViewUpdate::Rebuild => cx.notify(),
    }
}

fn sheet_spring(value: f32) -> Spring<f32> {
    Spring::new(
        value,
        value,
        0.0,
        SpringConfig {
            stiffness: 420.0,
            damping: 32.0,
            rest_speed: 0.002,
            rest_delta: 0.002,
            ..SpringConfig::default()
        },
    )
    .expect("the static DevTools spring is valid")
}

fn section_spring(value: f32) -> Spring<f32> {
    Spring::new(
        value,
        value,
        0.0,
        SpringConfig {
            stiffness: 520.0,
            damping: 34.0,
            rest_speed: 0.002,
            rest_delta: 0.002,
            ..SpringConfig::default()
        },
    )
    .expect("the static DevTools section spring is valid")
}

impl<A> DevtoolsHost<A> {
    pub(crate) fn measure_profile(&mut self, layout: &LayoutSnapshot) -> bool {
        let mut changed = false;
        for (key, extent) in [
            ("__devtools-frames", &mut self.profile_extents.frames),
            (
                "__devtools-frames-header",
                &mut self.profile_extents.frames_header,
            ),
            ("__devtools-gpu-passes", &mut self.profile_extents.gpu),
            (
                "__devtools-gpu-header",
                &mut self.profile_extents.gpu_header,
            ),
        ] {
            if let Some(bounds) = layout.bounds(key) {
                changed |= (*extent - bounds.size.height).abs() > 0.5;
                *extent = bounds.size.height;
            }
        }
        changed
    }
    pub(crate) fn panel_width(&self) -> f32 {
        if self.dock_mode == crate::DockMode::Right {
            self.dock_extent()
        } else if self.viewport.size.width > 0.0 {
            self.viewport.size.width
        } else {
            1220.0
        }
    }

    pub(crate) fn dock_extent(&self) -> f32 {
        let available = if self.dock_mode == crate::DockMode::Right {
            self.viewport.size.width
        } else {
            self.viewport.size.height
        };
        let desired = if self.dock_mode == crate::DockMode::Right {
            self.dock_width
        } else {
            self.dock_height
        };
        if available <= 0.0 {
            desired
        } else {
            desired.min((available - 54.0).max(0.0))
        }
    }

    pub(crate) fn set_dock_mode(&mut self, mode: crate::DockMode) {
        self.dock_mode = mode;
        let (axis, size) = if mode == crate::DockMode::Right {
            (argui_widgets::SplitAxis::Horizontal, self.dock_width)
        } else {
            (argui_widgets::SplitAxis::Vertical, self.dock_height)
        };
        self.splitter =
            argui_widgets::SplitPane::new("__devtools-splitter", axis, size, 180.0, 5000.0)
                .trailing(true);
    }
}
