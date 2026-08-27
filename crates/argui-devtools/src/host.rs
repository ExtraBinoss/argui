use argui_animation::{Frame, Spring, SpringConfig};
use argui_core::{Point, Rect};
use argui_inspect::{FrameRecord, InspectNodeId, InspectorHandle, StyleProperty};
use argui_paint::{ImageAsset, VectorAsset};
use argui_render::EffectShader;
use argui_runtime::{LayoutSnapshot, ScrollRequest, UiApp, ViewUpdate};
use argui_ui::{ClipboardRequest, Element, UiEvent, UiEventKind};

use crate::{icons::DevtoolsIcons, view};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum Tab {
    #[default]
    Elements,
    Profiling,
}

pub struct DevtoolsHost<A> {
    pub(crate) app: A,
    pub(crate) inspector: InspectorHandle,
    pub(crate) open: bool,
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
    pub(crate) search: String,
    pub(crate) sections: [bool; 3],
    pub(crate) section_progress: [f32; 3],
    pub(crate) morph_progress: f32,
    pub(crate) sheet_progress: f32,
    sheet_motion: Spring<f32>,
    section_motion: [Spring<f32>; 3],
    morph_motion: Spring<f32>,
    pub(crate) icons: DevtoolsIcons,
    clipboard: Option<ClipboardRequest>,
    pending_scroll: Option<ScrollRequest>,
    dragging_splitter: bool,
}

impl<A> DevtoolsHost<A> {
    #[must_use]
    pub fn new(app: A) -> Self {
        Self {
            app,
            inspector: InspectorHandle::default(),
            open: false,
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
            search: String::new(),
            sections: [true, true, true],
            section_progress: [1.0; 3],
            morph_progress: 0.0,
            sheet_progress: 0.0,
            sheet_motion: sheet_spring(0.0),
            section_motion: std::array::from_fn(|_| section_spring(1.0)),
            morph_motion: section_spring(0.0),
            icons: DevtoolsIcons::embedded(),
            clipboard: None,
            pending_scroll: None,
            dragging_splitter: false,
        }
    }

    #[must_use]
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self.sheet_progress = if open { 1.0 } else { 0.0 };
        self.sheet_motion = sheet_spring(self.sheet_progress);
        self
    }

    #[must_use]
    pub fn inspector(&self) -> InspectorHandle {
        self.inspector.clone()
    }

    fn update_tools(&mut self, event: &UiEvent) -> Option<ViewUpdate> {
        let key = event.key.as_deref()?;
        if key == "__devtools-splitter" {
            match event.kind {
                UiEventKind::Pressed => {
                    self.dragging_splitter = true;
                    return Some(ViewUpdate::None);
                }
                UiEventKind::Released => {
                    self.dragging_splitter = false;
                    return Some(ViewUpdate::None);
                }
                UiEventKind::PointerMoved(Point { y, .. }) if self.dragging_splitter => {
                    let height = (self.viewport.size.height - y).clamp(180.0, 620.0);
                    if self.dock_height != height {
                        self.dock_height = height;
                        return Some(ViewUpdate::Rebuild);
                    }
                    return Some(ViewUpdate::None);
                }
                _ => {}
            }
            return Some(ViewUpdate::None);
        }
        if key == "__devtools-picker-surface" {
            return Some(match event.kind {
                UiEventKind::PointerMoved(point) => {
                    let stack = self.inspector.hit_stack(point, self.app_viewport);
                    let cycle = self.last_pick_point.is_some_and(|last| {
                        (last.x - point.x).abs() <= 2.0 && (last.y - point.y).abs() <= 2.0
                    });
                    let index = if cycle {
                        self.inspector
                            .selected()
                            .and_then(|selected| stack.iter().position(|node| *node == selected))
                            .map_or(0, |index| (index + 1) % stack.len().max(1))
                    } else {
                        0
                    };
                    let hovered = stack.get(index).copied();
                    self.picker_point = Some(point);
                    if self.picker_hovered == hovered {
                        ViewUpdate::None
                    } else {
                        self.picker_hovered = hovered;
                        self.inspector.set_hovered(hovered);
                        ViewUpdate::Paint
                    }
                }
                UiEventKind::PointerLeft => {
                    self.picker_hovered = None;
                    self.picker_point = None;
                    self.inspector.set_hovered(None);
                    ViewUpdate::Paint
                }
                UiEventKind::Clicked => {
                    let selected = self.picker_hovered;
                    self.inspector.select(selected);
                    self.last_pick_point = self.picker_point;
                    if let Some(node) = selected {
                        self.reveal_tree_node(node);
                    }
                    self.picking = false;
                    self.picker_hovered = None;
                    self.picker_point = None;
                    self.inspector.set_hovered(None);
                    self.tab = Tab::Elements;
                    ViewUpdate::Rebuild
                }
                _ => ViewUpdate::None,
            });
        }
        if let UiEventKind::Scrolled { offset, .. } = event.kind
            && key == "__devtools-tree"
        {
            let old = self.tree_window(self.tree_offset);
            self.tree_offset = offset.y;
            let new = self.tree_window(self.tree_offset);
            return Some(if old == new {
                ViewUpdate::None
            } else {
                ViewUpdate::Rebuild
            });
        }
        if let UiEventKind::Scrolled { offset, .. } = event.kind
            && key == "__devtools-frames"
        {
            let old = view::profiling_list_config(self.profile_frames.len())
                .window(self.profiling_offset)
                .range;
            self.profiling_offset = offset.y;
            let new = view::profiling_list_config(self.profile_frames.len())
                .window(self.profiling_offset)
                .range;
            return Some(if old == new {
                ViewUpdate::None
            } else {
                ViewUpdate::Rebuild
            });
        }
        if let UiEventKind::TextChanged(value) = &event.kind
            && key == "__devtools-search"
        {
            self.search.clone_from(value);
            self.tree_offset = 0.0;
            return Some(ViewUpdate::Rebuild);
        }
        if let UiEventKind::TextChanged(value) = &event.kind
            && let Some((property, field)) = style_value_key(key)
        {
            let Some(node) = self.inspector.selected() else {
                return Some(ViewUpdate::None);
            };
            let authored = self
                .inspector
                .node(node)
                .and_then(|candidate| {
                    candidate
                        .properties
                        .into_iter()
                        .find(|candidate| candidate.property == property)
                })
                .map(|property| property.value);
            let mut style = self.inspector.property_value(node, property).or(authored);
            if let (Some(style), Ok(value)) = (&mut style, value.parse::<f32>())
                && style.set_field(field, value)
            {
                self.inspector
                    .set_property_value(node, property, style.clone());
                return Some(ViewUpdate::Rebuild);
            }
            return Some(ViewUpdate::None);
        }
        if event.kind != UiEventKind::Clicked {
            return key.starts_with("__devtools").then_some(ViewUpdate::None);
        }
        match key {
            "__devtools-toggle" => {
                self.open = !self.open;
                self.picking = false;
                self.picker_hovered = None;
                self.picker_point = None;
                self.inspector.set_hovered(None);
                self.sheet_motion
                    .retarget(if self.open { 1.0 } else { 0.0 });
            }
            "__devtools-picker" => {
                self.picking = !self.picking;
                self.picker_hovered = None;
                self.picker_point = None;
                self.inspector.set_hovered(None);
                self.tab = Tab::Elements;
            }
            "__devtools-elements" => self.tab = Tab::Elements,
            "__devtools-profiling" => {
                self.tab = Tab::Profiling;
                self.profile_frames = self.inspector.frames();
            }
            "__devtools-pause" => self.inspector.set_paused(!self.inspector.paused()),
            "__devtools-clear" => {
                self.inspector.clear_frames();
                self.profile_frames.clear();
            }
            "__devtools-refresh" => self.profile_frames = self.inspector.frames(),
            "__devtools-reset" => self.inspector.clear_overrides(),
            "__devtools-copy" => {
                self.clipboard = Some(ClipboardRequest::Write(self.inspector.trace_text()));
            }
            "__devtools-morph" => {
                let target = f32::from(self.morph_motion.target() < 0.5);
                self.morph_motion.retarget(target);
            }
            _ if key.starts_with("__devtools-section-") => {
                if let Ok(index) = key
                    .trim_start_matches("__devtools-section-")
                    .parse::<usize>()
                    && let Some(section) = self.sections.get_mut(index)
                {
                    *section = !*section;
                    self.section_motion[index].retarget(f32::from(*section));
                }
            }
            _ if key.starts_with("__devtools-node-") => {
                let id = key
                    .trim_start_matches("__devtools-node-")
                    .parse::<u64>()
                    .ok()
                    .map(InspectNodeId);
                self.inspector.select(id);
            }
            _ if key.starts_with("__devtools-style-") => {
                let Some(node) = self.inspector.selected() else {
                    return Some(ViewUpdate::None);
                };
                let label = key.trim_start_matches("__devtools-style-");
                if let Some(property) = StyleProperty::ALL
                    .into_iter()
                    .find(|property| property.label() == label)
                {
                    self.inspector.toggle(node, property);
                }
            }
            _ => return key.starts_with("__devtools").then_some(ViewUpdate::None),
        }
        Some(ViewUpdate::Rebuild)
    }

    fn reveal_tree_node(&mut self, node: InspectNodeId) {
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
        let viewport = (self.dock_height - 84.0).max(80.0);
        let total = node_count as f32 * 28.0;
        let centered = index as f32 * 28.0 - (viewport - 28.0) * 0.5;
        self.tree_offset = centered.clamp(0.0, (total - viewport).max(0.0));
        self.pending_scroll = Some(ScrollRequest {
            key: "__devtools-tree".into(),
            offset: Point::new(0.0, self.tree_offset),
        });
    }

    fn tree_window(&self, offset: f32) -> std::ops::Range<usize> {
        let query = self.search.to_lowercase();
        let item_count = self.inspector.with_tree(|snapshot| {
            if query.is_empty() {
                snapshot.nodes.len()
            } else {
                snapshot
                    .nodes
                    .iter()
                    .filter(|node| view::matches_query(node, &query))
                    .count()
            }
        });
        let viewport = (self.dock_height - 84.0).max(80.0);
        view::tree_list_config(item_count, viewport)
            .window(offset)
            .range
    }
}

impl<A: UiApp> UiApp for DevtoolsHost<A> {
    fn view(&self) -> Element {
        view::host(self)
    }

    fn update(&mut self, event: &UiEvent) -> ViewUpdate {
        self.update_tools(event)
            .unwrap_or_else(|| self.app.update(event))
    }

    fn animation_frame(&mut self, frame: Frame) -> ViewUpdate {
        let app = self.app.animation_frame(frame);
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
        let morph = self.morph_motion.advance(frame.elapsed);
        if morph {
            self.morph_progress = self.morph_motion.value().clamp(0.0, 1.0);
        }
        if sheet || sections || morph || app == ViewUpdate::Rebuild {
            ViewUpdate::Rebuild
        } else if app == ViewUpdate::Paint {
            ViewUpdate::Paint
        } else {
            ViewUpdate::None
        }
    }

    fn wants_animation_frame(&self) -> bool {
        self.sheet_motion.is_active()
            || self.section_motion.iter().any(Spring::is_active)
            || self.morph_motion.is_active()
            || self.app.wants_animation_frame()
    }

    fn layout_changed(&mut self, layout: &LayoutSnapshot) -> ViewUpdate {
        self.viewport = layout.viewport;
        let mut application = layout.clone();
        if let Some(bounds) = layout.bounds("__devtools-app-root") {
            application.viewport = bounds;
        }
        self.app_viewport = application.viewport;
        self.app.layout_changed(&application)
    }

    fn effect_shaders(&self) -> &'static [EffectShader] {
        self.app.effect_shaders()
    }

    fn image_assets(&self) -> Vec<ImageAsset> {
        self.app.image_assets()
    }

    fn vector_assets(&self) -> Vec<VectorAsset> {
        let mut assets = self.app.vector_assets();
        assets.extend_from_slice(self.icons.assets());
        assets
    }

    fn inspector(&self) -> Option<InspectorHandle> {
        Some(self.inspector.clone())
    }

    fn take_clipboard_request(&mut self) -> Option<ClipboardRequest> {
        self.clipboard
            .take()
            .or_else(|| self.app.take_clipboard_request())
    }

    fn take_scroll_request(&mut self) -> Option<ScrollRequest> {
        self.pending_scroll
            .take()
            .or_else(|| self.app.take_scroll_request())
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

fn style_value_key(key: &str) -> Option<(StyleProperty, usize)> {
    let suffix = key.strip_prefix("__devtools-value-")?;
    let (_, suffix) = suffix.split_once('-')?;
    let (label, field) = suffix.rsplit_once('-')?;
    let property = StyleProperty::ALL
        .into_iter()
        .find(|property| property.label() == label)?;
    Some((property, field.parse().ok()?))
}
