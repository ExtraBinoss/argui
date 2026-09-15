use argui_core::PointerId;
use argui_ui::{ClipboardRequest, FocusRequest, NodeId, TextSelectionRequest};

use crate::{AppCommand, ThemeRequest};

use super::{AnyEntity, ScrollRequest};

#[derive(Default)]
pub(crate) struct ContextEffects {
    pub(crate) update: ViewUpdate,
    pub(crate) animation_frame: bool,
    pub(crate) clipboard: Option<ClipboardRequest>,
    pub(crate) scroll: Option<ScrollRequest>,
    pub(crate) focus: Option<FocusRequest>,
    pub(crate) text_selection: Option<TextSelectionRequest>,
    pub(crate) theme: Option<ThemeRequest>,
    pub(crate) commands: Vec<AppCommand>,
    pub(crate) pointer_capture: Vec<PointerCaptureRequest>,
    pub(crate) ui_commands: Vec<argui_ui::UiCommand>,
    pub(super) children: Vec<AnyEntity>,
    pub(super) event_routes: Vec<AnyEntity>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PointerCaptureRequest {
    Capture { pointer: PointerId, target: NodeId },
    Release { pointer: PointerId, target: NodeId },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// Strongest rendering work requested for a retained view.
pub enum ViewUpdate {
    /// The cached view remains valid.
    #[default]
    None,
    /// Repaint the view using its existing retained tree.
    Paint,
    /// Rebuild the retained tree and repaint it.
    Rebuild,
}

pub(super) fn strongest_update(left: ViewUpdate, right: ViewUpdate) -> ViewUpdate {
    match (left, right) {
        (ViewUpdate::Rebuild, _) | (_, ViewUpdate::Rebuild) => ViewUpdate::Rebuild,
        (ViewUpdate::Paint, _) | (_, ViewUpdate::Paint) => ViewUpdate::Paint,
        _ => ViewUpdate::None,
    }
}

pub(super) fn merge_effects(target: &mut ContextEffects, mut source: ContextEffects) {
    target.update = strongest_update(target.update, source.update);
    target.animation_frame |= source.animation_frame;
    if source.clipboard.is_some() {
        target.clipboard = source.clipboard.take();
    }
    if source.scroll.is_some() {
        target.scroll = source.scroll.take();
    }
    if source.focus.is_some() {
        target.focus = source.focus.take();
    }
    if source.text_selection.is_some() {
        target.text_selection = source.text_selection.take();
    }
    target.ui_commands.append(&mut source.ui_commands);
    if source.theme.is_some() {
        target.theme = source.theme.take();
    }
    target.commands.append(&mut source.commands);
    target.pointer_capture.append(&mut source.pointer_capture);
    target.children.append(&mut source.children);
    target.event_routes.append(&mut source.event_routes);
}
