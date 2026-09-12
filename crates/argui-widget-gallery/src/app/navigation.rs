use super::{WidgetGallery, text};
use crate::navigation::Page;
use argui::{
    core::{Key, KeyState},
    paint::{Border, BorderWidths},
    ui::{
        Axes, Element, JustifyContent, Overflow, ScrollConfig, Sides, UiEvent, UiEventKind, length,
        percent,
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
        .description("Ctrl or Command K")
        .leading(assets.icon(TablerIcon::Search, 16.0), 38.0)
        .build();
        let mut children = vec![search];
        for category in ["Widgets", "Examples"] {
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
        Element::column(children)
            .width(length(260.0))
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
            .scroll_config(ScrollConfig::default().scrollbar(theme.scrollbar.clone()))
    }

    fn filtered_pages(&self) -> Vec<Page> {
        Page::ALL
            .into_iter()
            .filter(|page| page.matches(&self.search))
            .collect()
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
