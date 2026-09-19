use super::{WidgetGallery, text};
use crate::navigation::Page;
use argui::{
    core::{CaretAffinity, Key, KeyState, TextPosition},
    paint::{Border, BorderWidths},
    platform::WindowKey,
    render::DamageTracking,
    runtime::{AppCommand, Context},
    ui::{
        Axes, Element, ElementKind, EventHandler, JustifyContent, Overflow, Role, ScrollAxes,
        ScrollConfig, Semantics, Sides, TextSelection, UiEvent, UiEventKind, length, percent,
    },
    widgets::{Button, Input, InputKind, TablerIcon, WidgetAssets, WidgetTheme},
};

impl WidgetGallery {
    pub(super) fn select_page(&mut self, page: Page, cx: &mut Context<Self>) {
        if self.page != page {
            if self.page == Page::DamageControl {
                self.damage_control.update(|demo, child_cx| {
                    demo.deactivate();
                    child_cx.notify();
                });
                cx.command(AppCommand::SetDamageTracking {
                    window: WindowKey::main(),
                    tracking: DamageTracking::enabled(),
                });
                cx.command(AppCommand::SetRendererProfiling {
                    window: WindowKey::main(),
                    enabled: false,
                });
            }
            if let Some(previous) = self.catalogue.get(&self.page) {
                previous.update(|demo, cx| {
                    demo.reset_transient();
                    cx.notify();
                });
            }
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
            if page == Page::DamageControl {
                let tracking = self.damage_control.read(|demo| demo.tracking());
                self.damage_control.update(|demo, child_cx| {
                    demo.activate();
                    child_cx.notify();
                });
                cx.command(AppCommand::SetDamageTracking {
                    window: WindowKey::main(),
                    tracking,
                });
                cx.command(AppCommand::SetRendererProfiling {
                    window: WindowKey::main(),
                    enabled: true,
                });
            }
            cx.notify();
        }
    }

    pub(super) fn sidebar(
        &self,
        theme: &WidgetTheme,
        assets: &WidgetAssets,
        cx: &mut Context<Self>,
    ) -> Element {
        let mut children = vec![self.search_input(theme, assets, cx)];
        for category in ["Widgets", "Effects", "Examples"] {
            let pages = self
                .filtered_pages()
                .into_iter()
                .filter(|page| page.category() == category)
                .collect::<Vec<_>>();
            if pages.is_empty() {
                continue;
            }
            children.push(text(category, 11.0, theme.muted_foreground, 700));
            for page in pages {
                let select = cx.event_handler(move |gallery, _, cx| gallery.select_page(page, cx));
                children.push(self.navigation_button(page, theme, true, select));
            }
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

    pub(super) fn mobile_navigation(
        &self,
        theme: &WidgetTheme,
        assets: &WidgetAssets,
        cx: &mut Context<Self>,
    ) -> Element {
        let mut items = vec![self.theme_button(theme, assets, cx)];
        for category in ["Widgets", "Effects", "Examples"] {
            let pages = self
                .filtered_pages()
                .into_iter()
                .filter(|page| page.category() == category)
                .collect::<Vec<_>>();
            if pages.is_empty() {
                continue;
            }
            items.push(
                text(category, 11.0, theme.muted_foreground, 700)
                    .padding(Sides::length(8.0))
                    .shrink(0.0),
            );
            for page in pages {
                let select = cx.event_handler(move |gallery, _, cx| gallery.select_page(page, cx));
                items.push(self.navigation_button(page, theme, false, select));
            }
        }
        let strip = Element::row(items)
            .keyed("gallery-mobile-navigation-strip")
            .padding(Sides {
                left: length(12.0),
                right: length(12.0),
                top: length(4.0),
                bottom: length(8.0),
            })
            .gap(6.0)
            .align_items(argui::ui::AlignItems::CENTER)
            .shrink(0.0);
        let navigation = Element::layout_boundary(strip)
            .keyed("gallery-mobile-navigation")
            .width(percent(1.0))
            .height(length(52.0))
            .overflow(Axes {
                x: Overflow::Auto,
                y: Overflow::Hidden,
            })
            .scroll_config(
                ScrollConfig::default()
                    .axes(ScrollAxes::Horizontal)
                    .scrollbar(theme.scrollbar.clone()),
            )
            .semantics(Semantics::new(Role::Navigation).label("Component navigation"));
        Element::column([
            Element::container([self.search_input(theme, assets, cx)])
                .padding(Sides {
                    left: length(12.0),
                    right: length(12.0),
                    top: length(10.0),
                    bottom: length(4.0),
                })
                .width(percent(1.0)),
            navigation,
        ])
        .keyed("gallery-mobile-header")
        .width(percent(1.0))
        .background(theme.card)
        .border(Border {
            widths: BorderWidths {
                bottom: 1.0,
                ..BorderWidths::default()
            },
            color: theme.border,
        })
    }

    fn search_input(
        &self,
        theme: &WidgetTheme,
        assets: &WidgetAssets,
        cx: &mut Context<Self>,
    ) -> Element {
        Input::new(
            "gallery-search",
            self.search.clone(),
            "Search components…",
            theme.input(),
        )
        .kind(InputKind::Search)
        .label("Search components")
        .description("Type anywhere outside an editor, or press Ctrl or Command K")
        .leading(assets.icon(TablerIcon::Search, 16.0), 38.0)
        .on_input(cx.input_callback(|gallery, value| {
            gallery.search = value;
            gallery.search_highlight = 0;
        }))
        .build()
    }

    fn navigation_button(
        &self,
        page: Page,
        theme: &WidgetTheme,
        full_width: bool,
        select: EventHandler,
    ) -> Element {
        let style = if page == self.page {
            theme.button()
        } else {
            theme.ghost_button()
        };
        let mut button = Button::new(format!("nav::{}", page.slug()), page.label(), style)
            .on_click(select)
            .build()
            .shrink(0.0)
            .justify_content(JustifyContent::START);
        if full_width {
            button = button.width(percent(1.0));
        }
        button
    }

    fn filtered_pages(&self) -> Vec<Page> {
        let mut pages: Vec<_> = Page::ALL
            .into_iter()
            .filter(|page| page.matches(&self.search))
            .collect();
        let query = self.search.trim();
        if !query.is_empty() {
            pages.sort_by_key(|page| !page.label().eq_ignore_ascii_case(query));
        }
        pages
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
