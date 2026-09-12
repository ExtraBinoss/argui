use super::{WidgetGallery, text};
use crate::navigation::Page;
use argui::{
    core::{CaretAffinity, Key, KeyState, TextPosition},
    paint::{Border, BorderWidths},
    runtime::Context,
    ui::{
        Axes, Element, ElementKind, JustifyContent, Overflow, Role, ScrollConfig, Sides,
        TextSelection, UiEvent, UiEventKind, length, percent,
    },
    widgets::{Button, Input, InputKind, TablerIcon, WidgetAssets, WidgetTheme},
};

impl WidgetGallery {
    pub(super) fn select_page(&mut self, page: Page) {
        if self.page != page {
            if self.page == Page::Tooltip {
                self.tooltip.update(|demo, cx| {
                    demo.reset();
                    cx.notify();
                });
            }
            if self.page == Page::Popover {
                self.popover.update(|demo, cx| {
                    demo.close();
                    cx.notify();
                });
            }
            self.page = page;
        }
    }

    pub(super) fn sidebar(&self, theme: &WidgetTheme, assets: &WidgetAssets) -> Element {
        let search = Input::new(
            "gallery-search",
            self.search.clone(),
            "Search components…",
            theme.input(),
        )
        .kind(InputKind::Search)
        .label("Search components")
        .description("Type anywhere outside an editor, or press Ctrl or Command K")
        .leading(assets.icon(TablerIcon::Search, 16.0), 38.0)
        .build();
        let mut children = vec![search];
        for category in ["Widgets", "Effects", "Examples"] {
            let pages = Page::ALL
                .into_iter()
                .filter(|page| page.category() == category && page.matches(&self.search))
                .collect::<Vec<_>>();
            if pages.is_empty() {
                continue;
            }
            children.push(text(category, 11.0, theme.muted_foreground, 700));
            children.extend(pages.into_iter().map(|page| {
                let style = if page == self.page {
                    theme.button()
                } else {
                    theme.ghost_button()
                };
                Button::new(format!("nav::{}", page.slug()), page.label(), style)
                    .build()
                    .width(percent(1.0))
                    .justify_content(JustifyContent::START)
            }));
        }
        let mut sidebar = Element::column(children)
            .keyed("gallery-sidebar")
            .width(length(260.0))
            .shrink(0.0)
            .height(percent(1.0))
            .padding(Sides::length(16.0))
            .gap(9.0)
            .background(theme.card)
            .border(Border {
                widths: BorderWidths {
                    right: 1.0,
                    ..BorderWidths::default()
                },
                color: theme.border,
            })
            .overflow(Axes {
                x: Overflow::Hidden,
                y: Overflow::Auto,
            })
            .scroll_config(ScrollConfig::default().scrollbar(theme.scrollbar.clone()));
        if self.backdrop.has_background() {
            sidebar = sidebar.desktop_backdrop(self.backdrop.paint(theme));
        }
        sidebar
    }

    fn filtered_pages(&self) -> Vec<Page> {
        Page::ALL
            .into_iter()
            .filter(|page| page.matches(&self.search))
            .collect()
    }

    pub(super) fn type_to_search(&mut self, event: &UiEvent, cx: &mut Context<Self>) -> bool {
        let UiEventKind::KeyInput(input) = &event.kind else {
            return false;
        };
        if input.state != KeyState::Pressed || input.modifiers.command() || input.modifiers.alt {
            return false;
        }
        if event.target_key() == Some("gallery-search") && input.key == Key::Escape {
            self.search.clear();
            self.search_highlight = 0;
            cx.request_focus("gallery-root");
            cx.notify();
            let _ = event.prevent_default();
            event.stop_propagation();
            return true;
        }
        let Key::Character(key) = &input.key else {
            return false;
        };
        let text = input.text.as_deref().unwrap_or(key);
        if text.trim().is_empty() || text.chars().any(char::is_control) {
            return false;
        }
        if !event
            .target_key()
            .and_then(|key| searchable_target(&self.navigation_root, key))
            .unwrap_or(false)
        {
            return false;
        }
        self.search = text.into();
        self.search_highlight = 0;
        cx.request_focus("gallery-search");
        cx.select_text(
            "gallery-search",
            TextSelection::Caret(TextPosition::new(self.search.len(), CaretAffinity::After)),
        );
        cx.notify();
        let _ = event.prevent_default();
        event.stop_propagation();
        true
    }

    pub(super) fn update_search_keys(&mut self, event: &UiEvent) -> Option<Page> {
        if event.target_key() != Some("gallery-search") {
            return None;
        }
        let UiEventKind::KeyInput(input) = &event.kind else {
            return None;
        };
        if input.state != KeyState::Pressed {
            return None;
        }
        let pages = self.filtered_pages();
        if pages.is_empty() {
            self.search_highlight = 0;
            return None;
        }
        match input.key {
            Key::ArrowDown => self.search_highlight = (self.search_highlight + 1) % pages.len(),
            Key::ArrowUp => {
                self.search_highlight = (self.search_highlight + pages.len() - 1) % pages.len();
            }
            Key::Home => self.search_highlight = 0,
            Key::End => self.search_highlight = pages.len() - 1,
            Key::Enter => return pages.get(self.search_highlight).copied(),
            _ => {}
        }
        None
    }
}

fn searchable_target(element: &Element, key: &str) -> Option<bool> {
    let target = if element.key.as_deref() == Some(key) {
        Some(true)
    } else {
        element
            .children
            .iter()
            .find_map(|child| searchable_target(child, key))
    }?;
    let owns_typing = matches!(
        element.kind,
        ElementKind::TextEditor { .. } | ElementKind::Custom(_)
    ) || element.native_content.is_some()
        || (element.portal.is_some() && element.focus_scope.is_some())
        || element.semantics.as_ref().is_some_and(|semantics| {
            semantics.state.expanded.is_some()
                || matches!(
                    semantics.role,
                    Role::TextInput
                        | Role::TextArea
                        | Role::SearchInput
                        | Role::ComboBox
                        | Role::Menu
                        | Role::MenuBar
                        | Role::ListBox
                        | Role::List
                        | Role::Tree
                        | Role::Grid
                )
        });
    Some(target && !owns_typing)
}
