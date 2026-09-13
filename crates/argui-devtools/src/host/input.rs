use argui_core::{CaretAffinity, TextPosition};
use argui_core::{Key, KeyState, PointerPhase};
use argui_inspect::InspectNodeId;
use argui_runtime::{Render, ViewUpdate};
use argui_ui::{FocusRequest, TextSelection, TextSelectionRequest, UiEvent, UiEventKind};

use super::{DevtoolsHost, Tab};

impl<A: Render> DevtoolsHost<A> {
    pub(super) fn navigation_input(&mut self, event: &UiEvent) -> Option<ViewUpdate> {
        let key = event.target_key()?;
        if !self.open || !key.starts_with("__devtools-") {
            return None;
        }
        if let UiEventKind::Pointer(pointer) = &event.kind {
            let row = key
                .strip_prefix("__devtools-node-")
                .and_then(|value| value.parse::<u64>().ok())
                .map(InspectNodeId);
            let hovered = match pointer.phase {
                PointerPhase::Entered | PointerPhase::Moved => row,
                PointerPhase::Left if row == self.tree_hovered => None,
                _ => return None,
            };
            if key == "__devtools-picker-surface" || hovered == self.tree_hovered {
                return None;
            }
            self.tree_hovered = hovered;
            self.inspector.set_hovered(hovered);
            return Some(ViewUpdate::Paint);
        }
        if self.properties_splitter.update(event) {
            let _ = event.prevent_default();
            return Some(ViewUpdate::Rebuild);
        }
        if key == "__devtools-panel" && matches!(event.kind, UiEventKind::Click(_)) {
            self.pending_focus = Some(FocusRequest::Focus(key.into()));
            return Some(ViewUpdate::None);
        }
        let UiEventKind::KeyInput(input) = &event.kind else {
            return None;
        };
        if input.state != KeyState::Pressed || input.modifiers.alt {
            return None;
        }
        let shortcut = input.modifiers.command()
            && matches!(&input.key, Key::Character(value) if value.eq_ignore_ascii_case("f"));
        if !shortcut {
            if self.tab != Tab::Elements
                || input.modifiers.command()
                || key == "__devtools-search"
                || key.starts_with("__devtools-value-")
                || key.starts_with("__devtools-color-")
                || key.starts_with("__devtools-dock")
            {
                return None;
            }
            let Key::Character(character) = &input.key else {
                return None;
            };
            let text = input.text.as_deref().unwrap_or(character);
            if text.trim().is_empty() || text.chars().any(char::is_control) {
                return None;
            }
            self.search.push_str(text);
        }
        self.tab = Tab::Elements;
        self.inspector.set_gpu_profiling(false);
        self.show_properties = false;
        self.tree_offset = 0.0;
        self.pending_focus = Some(FocusRequest::Focus("__devtools-search".into()));
        self.pending_selection = Some(TextSelectionRequest::new(
            "__devtools-search",
            if shortcut {
                TextSelection::All
            } else {
                TextSelection::Caret(TextPosition::new(self.search.len(), CaretAffinity::After))
            },
        ));
        let _ = event.prevent_default();
        event.stop_propagation();
        Some(ViewUpdate::Rebuild)
    }
}
