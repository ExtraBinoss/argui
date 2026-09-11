use argui_text::{TextStyle, TextWrap};
use argui_ui::{AlignItems, Element, FlexWrap, JustifyContent, Role, Semantics, UiEvent, length};

use crate::{Button, ButtonBehavior, WidgetTheme};

#[derive(Clone, Debug)]
pub struct PaginationLabels {
    pub navigation: String,
    pub previous: String,
    pub next: String,
    pub page: String,
    pub current: String,
}

impl Default for PaginationLabels {
    fn default() -> Self {
        Self {
            navigation: "Pagination".into(),
            previous: "Previous".into(),
            next: "Next".into(),
            page: "Page".into(),
            current: "Current page".into(),
        }
    }
}

/// Controlled, one-based pagination. Zero total pages produces disabled navigation.
/// The visible range stays bounded to seven entries, even with very large totals.
#[derive(Clone, Debug)]
pub struct Pagination {
    key: String,
    page: usize,
    total: usize,
    enabled: bool,
    labels: PaginationLabels,
}

impl Pagination {
    #[must_use]
    pub fn new(key: impl Into<String>, page: usize, total: usize) -> Self {
        Self {
            key: key.into(),
            page: page.max(1).min(total),
            total,
            enabled: true,
            labels: PaginationLabels::default(),
        }
    }

    #[must_use]
    pub const fn page(&self) -> usize {
        self.page
    }

    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    pub fn labels(mut self, labels: PaginationLabels) -> Self {
        self.labels = labels;
        self
    }

    #[must_use]
    pub fn page_key(&self, page: usize) -> String {
        format!("{}::page::{page}", self.key)
    }

    #[must_use]
    pub fn previous_key(&self) -> String {
        format!("{}::previous", self.key)
    }

    #[must_use]
    pub fn next_key(&self) -> String {
        format!("{}::next", self.key)
    }

    #[must_use]
    pub fn action(&self, event: &UiEvent) -> Option<usize> {
        if !self.enabled {
            return None;
        }
        if self.page > 1
            && ButtonBehavior::new(self.previous_key(), &self.labels.previous)
                .action(event)
                .is_some()
        {
            return Some(self.page - 1);
        }
        if self.page < self.total
            && ButtonBehavior::new(self.next_key(), &self.labels.next)
                .action(event)
                .is_some()
        {
            return Some(self.page + 1);
        }
        self.entries().into_iter().flatten().find(|page| {
            *page != self.page
                && ButtonBehavior::new(self.page_key(*page), "")
                    .action(event)
                    .is_some()
        })
    }

    fn entries(&self) -> Vec<Option<usize>> {
        if self.total <= 7 {
            return (1..=self.total).map(Some).collect();
        }
        if self.page <= 4 {
            return (1..=5).map(Some).chain([None, Some(self.total)]).collect();
        }
        if self.page >= self.total - 3 {
            return [Some(1), None]
                .into_iter()
                .chain((self.total - 4..=self.total).map(Some))
                .collect();
        }
        vec![
            Some(1),
            None,
            Some(self.page - 1),
            Some(self.page),
            Some(self.page + 1),
            None,
            Some(self.total),
        ]
    }

    #[must_use]
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        let mut children = vec![self.navigation_button(
            self.previous_key(),
            &self.labels.previous,
            self.enabled && self.page > 1,
            theme,
        )];
        for entry in self.entries() {
            if let Some(page) = entry {
                let current = page == self.page;
                let style = if current {
                    theme.outline_button()
                } else {
                    theme.ghost_button()
                };
                let mut button = Button::new(
                    self.page_key(page),
                    format!("{} {page}", self.labels.page),
                    style,
                )
                .content(Element::text(page.to_string()).text_style(TextStyle {
                    font_size: 14.0,
                    line_height: 20.0,
                    color: if self.enabled {
                        theme.foreground
                    } else {
                        theme.muted_foreground
                    },
                    wrap: TextWrap::None,
                    ..TextStyle::default()
                }))
                .enabled(self.enabled)
                .build()
                .paint_opacity(if self.enabled { 1.0 } else { 0.5 })
                .min_width(length(36.0))
                .padding(argui_ui::sides(10.0, 8.0))
                .justify_content(JustifyContent::CENTER);
                if current && let Some(semantics) = &mut button.semantics {
                    semantics.description = Some(self.labels.current.clone());
                }
                children.push(button);
            } else {
                children.push(
                    Element::text("…")
                        .text_style(TextStyle {
                            font_size: 14.0,
                            line_height: 20.0,
                            color: theme.muted_foreground,
                            ..TextStyle::default()
                        })
                        .padding(argui_ui::sides(8.0, 8.0))
                        .semantic_hidden(true),
                );
            }
        }
        children.push(self.navigation_button(
            self.next_key(),
            &self.labels.next,
            self.enabled && self.page < self.total,
            theme,
        ));
        Element::row(children)
            .keyed(self.key.clone())
            .semantics(Semantics::new(Role::Group).label(self.labels.navigation.clone()))
            .align_items(AlignItems::CENTER)
            .flex_wrap(FlexWrap::Wrap)
            .gap(4.0)
            .min_width(length(0.0))
    }
    fn navigation_button(
        &self,
        key: String,
        label: &str,
        enabled: bool,
        theme: &WidgetTheme,
    ) -> Element {
        let mut style = theme.ghost_button();
        if !enabled {
            style.label.color = theme.muted_foreground;
        }
        Button::new(key, label, style).enabled(enabled).build()
    }
}
