use super::*;
use argui_inspect::StyleProperty;
use argui_ui::UiEventKind;

pub(super) fn style_value_key(key: &str) -> Option<(StyleProperty, usize)> {
    let suffix = key.strip_prefix("__devtools-value-")?;
    let (_, suffix) = suffix.split_once('-')?;
    let (label, field) = suffix.rsplit_once('-')?;
    let property = StyleProperty::ALL
        .into_iter()
        .find(|property| property.label() == label)?;
    Some((property, field.parse().ok()?))
}

impl<A: Render> DevtoolsHost<A> {
    pub(super) fn update_tools(&mut self, event: &UiEvent) -> Option<ViewUpdate> {
        let key = event.target_key()?;
        if let UiEventKind::Scrolled { offset, .. } = event.kind
            && key == "__devtools-gpu-passes"
        {
            let count = self
                .profile_details
                .gpu
                .as_ref()
                .map_or(0, |gpu| gpu.passes.len());
            let list =
                argui_widgets::VList::new(key, 28.0, self.profile_extents.gpu, 0.0).config(count);
            let header = self.profile_extents.gpu_header;
            let changed = list.window((self.gpu_offset - header).max(0.0)).range
                != list.window((offset.y - header).max(0.0)).range;
            self.gpu_offset = offset.y;
            return Some(if changed {
                ViewUpdate::Rebuild
            } else {
                ViewUpdate::None
            });
        }
        if key.starts_with("__devtools-node-") {
            let action = {
                let nodes = self.tree_nodes();
                self.tree_view(&nodes, None).action(event)
            };
            if let Some(action) = action {
                match action {
                    argui_widgets::TreeAction::Select(key) => {
                        let id = key
                            .trim_start_matches("__devtools-node-")
                            .parse()
                            .ok()
                            .map(InspectNodeId);
                        self.inspector.select(id);
                        self.show_properties = id.is_some();
                        if let Some(id) = id {
                            self.reveal_tree_node(id);
                            self.pending_focus = Some(argui_ui::FocusRequest::Focus(key.into()));
                        }
                    }
                    argui_widgets::TreeAction::Expand(key) => {
                        self.collapsed.remove(&key);
                    }
                    argui_widgets::TreeAction::Collapse(key) => {
                        self.collapsed.insert(key);
                    }
                }
                return Some(ViewUpdate::Rebuild);
            }
        }
        if self.splitter.update(event) {
            if self.dock_mode == crate::DockMode::Right {
                self.dock_width = self.splitter.size;
            } else {
                self.dock_height = self.splitter.size;
            }
            return Some(ViewUpdate::Rebuild);
        }
        if let UiEventKind::KeyInput(input) = &event.kind
            && input.key == argui_core::Key::Escape
            && self.picking
        {
            self.picking = false;
            self.inspector.set_hovered(None);
            return Some(ViewUpdate::Rebuild);
        }
        let options = crate::presentation::dock_options(self.detach_available);
        let behavior = argui_widgets::SelectBehavior::new(
            "__devtools-dock",
            "Dock position",
            options,
            Some(self.dock_mode as usize),
        )
        .open(self.dock_menu)
        .highlighted(self.dock_highlight);
        if let Some(action) = behavior.action(event) {
            match action {
                argui_widgets::SelectAction::Toggle => self.dock_menu = !self.dock_menu,
                argui_widgets::SelectAction::Close => self.dock_menu = false,
                argui_widgets::SelectAction::Highlight(index) => self.dock_highlight = index,
                argui_widgets::SelectAction::Select(index) => {
                    self.dock_menu = false;
                    let mode = [
                        crate::DockMode::Bottom,
                        crate::DockMode::Right,
                        crate::DockMode::Detached,
                    ][index];
                    if self.detach_available {
                        self.requested_mode = Some(mode);
                    } else {
                        self.set_dock_mode(mode);
                    }
                }
            }
            return Some(ViewUpdate::Rebuild);
        }
        if key == "__devtools-picker-surface" {
            return Some(match event.kind {
                UiEventKind::Pointer(argui_core::PointerEvent {
                    phase: argui_core::PointerPhase::Moved,
                    position: point,
                    ..
                }) => {
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
                UiEventKind::Pointer(argui_core::PointerEvent {
                    phase: argui_core::PointerPhase::Left,
                    ..
                }) => {
                    self.picker_hovered = None;
                    self.picker_point = None;
                    self.inspector.set_hovered(None);
                    ViewUpdate::Paint
                }
                UiEventKind::Click(_) => {
                    let selected = self.picker_hovered;
                    self.inspector.select(selected);
                    self.last_pick_point = self.picker_point;
                    if let Some(node) = selected {
                        self.reveal_tree_node(node);
                    }
                    self.show_properties = selected.is_some();
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
            let old =
                view::profiling_list_config(self.profile_frames.len(), self.profile_extents.frames)
                    .window((self.profiling_offset - self.profile_extents.frames_header).max(0.0))
                    .range;
            self.profiling_offset = offset.y;
            let new =
                view::profiling_list_config(self.profile_frames.len(), self.profile_extents.frames)
                    .window((self.profiling_offset - self.profile_extents.frames_header).max(0.0))
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
        if !matches!(event.kind, UiEventKind::Click(_)) {
            return key.starts_with("__devtools").then_some(ViewUpdate::None);
        }
        match key {
            "__devtools-toggle" => {
                self.open = !self.open;
                self.inspector.set_recording(self.open);
                self.picking = false;
                self.picker_hovered = None;
                self.picker_point = None;
                self.inspector.set_hovered(None);
                self.sheet_motion
                    .retarget(if self.open { 1.0 } else { 0.0 });
            }
            "__devtools-picker" => {
                self.inspector.set_gpu_profiling(false);
                self.picking = !self.picking;
                self.picker_hovered = None;
                self.picker_point = None;
                self.inspector.set_hovered(None);
                self.tab = Tab::Elements;
            }
            "__devtools-elements" => {
                self.tab = Tab::Elements;
                self.inspector.set_gpu_profiling(false);
            }
            "__devtools-profile-overview" => self.profile_panel = 0,
            "__devtools-profile-gpu" => self.profile_panel = 1,
            "__devtools-profile-details" => self.profile_panel = 2,
            "__devtools-profiling" => {
                self.inspector.set_gpu_profiling(true);
                self.tab = Tab::Profiling;
                self.inspector
                    .sync_frames(&mut self.profile_frames, &mut self.frame_cursor);
                self.refresh_profile_details();
            }
            "__devtools-pause" => {
                self.inspector.set_paused(!self.inspector.paused());
                self.refresh_profile_details();
            }
            "__devtools-clear" => {
                self.inspector.clear_frames();
                self.profile_frames.clear();
                self.selected_frame = None;
                self.refresh_profile_details();
            }
            "__devtools-refresh" => {
                self.selected_frame = None;
                self.inspector.set_paused(false);
                self.inspector
                    .sync_frames(&mut self.profile_frames, &mut self.frame_cursor);
                self.refresh_profile_details();
            }
            "__devtools-back" => self.show_properties = false,
            "__devtools-reset" => self.inspector.clear_overrides(),
            "__devtools-copy" => {
                if let Ok(trace) = self.inspector.trace_json() {
                    self.clipboard = Some(ClipboardRequest::Write(trace));
                }
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
                self.show_properties = id.is_some();
            }
            _ if key.starts_with("__devtools-frame-") => {
                self.selected_frame = key
                    .trim_start_matches("__devtools-frame-")
                    .parse::<usize>()
                    .ok()
                    .filter(|index| *index < self.profile_frames.len());
                self.refresh_profile_details();
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
}
