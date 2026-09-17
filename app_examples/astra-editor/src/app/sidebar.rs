use argui::{
    paint::{Border, BorderWidths},
    runtime::Context,
    ui::{AlignItems, Element, JustifyContent, Sides, length, percent},
    widgets::{Button, Kbd, TablerIcon, TreeView, VList, WidgetTheme},
};

use super::AstraEditor;
use crate::ui::{label, scroll_shadow};

impl AstraEditor {
    /// Returns whether the project tree crossed into a new virtual row window.
    pub(super) fn project_tree_window_changed(&self, previous: f32, next: f32) -> bool {
        let viewport =
            (self.viewport.height - if self.compact { 142.0 } else { 168.0 }).clamp(120.0, 1_200.0);
        VList::new("project-tree", 30.0, viewport, next).window_changed(
            self.tree_cache.borrow().visible_count(),
            previous,
            next,
        )
    }

    /// Builds the virtualized project explorer for desktop and compact layouts.
    pub(super) fn project_sidebar(&self, theme: &WidgetTheme, cx: &mut Context<Self>) -> Element {
        let viewport =
            (self.viewport.height - if self.compact { 142.0 } else { 168.0 }).clamp(120.0, 1_200.0);
        let selected =
            (!self.selected_tree_key.is_empty()).then_some(self.selected_tree_key.as_str());
        let select = cx.value_event_handler(|editor, key: String, _, cx| {
            editor.selected_tree_key.clone_from(&key);
            if let Some(document) = editor.project.document_for_key(&key) {
                editor.open_document(document, cx);
                return;
            }
            if editor.project.is_directory(&key) {
                if editor.collapsed.remove(&key) {
                    editor.tree_reveal_root = Some(key);
                    editor.tree_reveal_started_at = web_time::Instant::now();
                    editor.tree_reveal_complete = false;
                } else {
                    editor.collapsed.insert(key);
                    editor.tree_reveal_root = None;
                    editor.tree_reveal_complete = true;
                }
                cx.notify();
            }
        });
        let mut tree = TreeView::new(
            &self.tree_nodes,
            selected,
            &self.collapsed,
            VList::new("project-tree", 30.0, viewport, self.explorer_offset)
                .effect(scroll_shadow(theme)),
        )
        .disclosure(Some(self.assets.vector_id(TablerIcon::ChevronRight)))
        .on_select(select);
        if let Some(parent) = self.tree_reveal_root.as_deref() {
            tree = tree.reveal_descendants(parent, self.tree_open_progress());
        }
        let tree = tree
            .build_cached(theme, &mut self.tree_cache.borrow_mut())
            .grow(1.0)
            .min_height(length(0.0));
        let header = self.explorer_header(theme, cx);
        let shortcuts = (!self.compact).then(|| {
            Element::column([
                shortcut("Quick open", ["Ctrl", "P"], "explorer-quick-open", theme),
                shortcut("Toggle explorer", ["Ctrl", "B"], "explorer-toggle", theme),
            ])
            .padding(Sides {
                left: length(12.0),
                right: length(12.0),
                top: length(8.0),
                bottom: length(10.0),
            })
            .gap(7.0)
            .border(Border {
                widths: BorderWidths {
                    top: 1.0,
                    ..BorderWidths::default()
                },
                color: theme.border,
            })
        });
        Element::column(
            std::iter::once(header)
                .chain(std::iter::once(tree))
                .chain(shortcuts),
        )
        .keyed("project-explorer")
        .width(percent(1.0))
        .height(percent(1.0))
        .min_width(length(0.0))
        .min_height(length(0.0))
        .background(theme.card)
    }

    /// Builds the explorer title, indexed-file count, and compact close action.
    fn explorer_header(&self, theme: &WidgetTheme, cx: &mut Context<Self>) -> Element {
        let title = Element::column([
            label("EXPLORER", 10.5, theme.muted_foreground, 720),
            label(
                format!(
                    "{} · {} files",
                    self.project.name,
                    self.project.documents.len()
                ),
                12.0,
                theme.foreground,
                600,
            ),
        ])
        .gap(3.0)
        .min_width(length(0.0));
        let action = if self.compact {
            Button::icon(
                "close-explorer",
                "Close explorer",
                self.assets
                    .icon(TablerIcon::Close, 16.0)
                    .vector_color(theme.foreground),
                theme.ghost_button(),
            )
            .on_click(cx.event_handler(|editor, _, cx| {
                editor.explorer_revealed = false;
                cx.notify();
            }))
            .build()
            .width(length(32.0))
            .height(length(32.0))
            .padding(Sides::length(0.0))
        } else {
            Button::icon(
                "refresh-project",
                "Open another project",
                self.assets
                    .icon(TablerIcon::ChevronRight, 16.0)
                    .vector_color(theme.muted_foreground),
                theme.ghost_button(),
            )
            .on_click(cx.event_handler(|editor, _, cx| editor.choose_project(cx)))
            .build()
            .width(length(32.0))
            .height(length(32.0))
            .padding(Sides::length(0.0))
        };
        Element::row([title, action])
            .height(length(66.0))
            .padding(Sides::length(12.0))
            .gap(8.0)
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::SPACE_BETWEEN)
            .border(Border {
                widths: BorderWidths {
                    bottom: 1.0,
                    ..BorderWidths::default()
                },
                color: theme.border,
            })
    }
}

/// Builds one explorer shortcut row with a real Kbd widget.
fn shortcut(title: &str, keys: [&str; 2], key: &str, theme: &WidgetTheme) -> Element {
    Element::row([
        label(title, 10.5, theme.muted_foreground, 500),
        Kbd::new(key, keys).label(title).size(18.0).build(theme),
    ])
    .align_items(AlignItems::CENTER)
    .justify_content(JustifyContent::SPACE_BETWEEN)
}
