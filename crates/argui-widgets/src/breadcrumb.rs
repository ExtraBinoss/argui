use argui_text::{TextStyle, TextWrap};
use argui_ui::{
    AlignItems, Element, EventType, FlexWrap, KeyboardActivation, Role, Semantics, Sides, UiEvent,
    ValueHandler, auto, length, percent,
};

use crate::{Button, ButtonBehavior, WidgetTheme};

/// An ancestor destination. The current page is supplied separately to `Breadcrumb`.
#[derive(Clone, Debug)]
pub struct BreadcrumbLink {
    pub id: String,
    pub label: String,
}

impl BreadcrumbLink {
    /// Creates a navigation link with a stable identity and display label.
    /// `id` is the destination identity returned on activation; `label` is displayed to users.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

/// Controlled application navigation. Activation returns an ancestor ID; no URL is opened.
#[derive(Clone, Debug)]
pub struct Breadcrumb {
    key: String,
    label: String,
    links: Vec<BreadcrumbLink>,
    current: String,
    current_description: String,
    separator: String,
    activate_handlers: Vec<ValueHandler<String>>,
}

impl Breadcrumb {
    /// Creates a breadcrumb trail from the ordered links and current page.
    /// Ancestor IDs must be unique within this breadcrumb.
    /// `key` identifies the trail, `links` lists ancestors in order, and `current` names the current page.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        links: impl IntoIterator<Item = BreadcrumbLink>,
        current: impl Into<String>,
    ) -> Self {
        let links: Vec<_> = links.into_iter().collect();
        let mut ids = std::collections::HashSet::new();
        assert!(
            links.iter().all(|link| ids.insert(&link.id)),
            "breadcrumb IDs must be unique"
        );
        Self {
            key: key.into(),
            label: "Breadcrumb".into(),
            links,
            current: current.into(),
            current_description: "Current page".into(),
            separator: "/".into(),
            activate_handlers: Vec::new(),
        }
    }

    #[must_use]
    /// Sets the accessible label for the breadcrumb navigation.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    #[must_use]
    /// Sets the accessible description for the current-page item.
    pub fn current_description(mut self, description: impl Into<String>) -> Self {
        self.current_description = description.into();
        self
    }

    #[must_use]
    /// Sets the text displayed between breadcrumb links.
    /// `separator` is the text inserted between adjacent items.
    pub fn separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = separator.into();
        self
    }

    /// Adds a callback receiving the activated ancestor's stable ID.
    #[must_use]
    pub fn on_activate(mut self, handler: ValueHandler<String>) -> Self {
        self.activate_handlers.push(handler);
        self
    }

    #[must_use]
    /// Returns the element key for the link identified by `id`.
    pub fn link_key(&self, id: &str) -> String {
        format!("{}::link::{id}", self.key)
    }

    #[must_use]
    /// Returns the activated link identity when `event` targets a breadcrumb link.
    pub fn action(&self, event: &UiEvent) -> Option<&str> {
        self.links
            .iter()
            .find(|link| {
                ButtonBehavior::new(self.link_key(&link.id), &link.label)
                    .action(event)
                    .is_some()
            })
            .map(|link| link.id.as_str())
    }

    #[must_use]
    /// Builds the breadcrumb navigation using `theme` for styling.
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        let style = TextStyle {
            font_size: 14.0,
            line_height: 20.0,
            color: theme.muted_foreground,
            wrap: TextWrap::WordOrGlyph,
            ..TextStyle::default()
        };
        let mut children = Vec::with_capacity(self.links.len() + 1);
        for link in &self.links {
            let mut button_style = theme.ghost_button();
            button_style.label = style.clone();
            let mut button = Button::new(self.link_key(&link.id), &link.label, button_style)
                .content(
                    Element::column([Element::text(link.label.clone()).text_style(style.clone())])
                        .grow(1.0)
                        .min_width(length(0.0)),
                )
                .build()
                .height(auto())
                .min_height(length(28.0))
                .min_width(length(0.0))
                .shrink(1.0)
                .padding(Sides::length(3.0));
            if let Some(semantics) = &mut button.semantics {
                semantics.role = Role::Link;
            }
            if let Some(interaction) = &mut button.interaction {
                interaction.keyboard_activation = KeyboardActivation::Enter;
            }
            for handler in &self.activate_handlers {
                button =
                    button.on(handler.direct_listener_value(EventType::Click, link.id.clone()));
            }
            children.push(
                Element::row([
                    button,
                    Element::text(self.separator.clone())
                        .text_style(style.clone())
                        .semantic_hidden(true)
                        .shrink(0.0),
                ])
                .gap(8.0)
                .min_width(length(0.0))
                .max_width(percent(1.0))
                .align_items(AlignItems::CENTER),
            );
        }
        children.push(
            Element::text(self.current.clone())
                .keyed(format!("{}::current", self.key))
                .text_style(TextStyle {
                    color: theme.foreground,
                    ..style
                })
                .semantics(
                    Semantics::new(Role::Text)
                        .label(self.current.clone())
                        .description(self.current_description.clone()),
                )
                .min_width(length(0.0))
                .padding(argui_ui::sides(3.0, 4.0)),
        );
        Element::row(children)
            .keyed(self.key.clone())
            .width(percent(1.0))
            .semantics(Semantics::new(Role::Group).label(self.label.clone()))
            .flex_wrap(FlexWrap::Wrap)
            .align_items(AlignItems::CENTER)
            .gap(8.0)
            .min_width(length(0.0))
    }
}
