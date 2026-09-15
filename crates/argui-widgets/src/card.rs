use argui_paint::{Border, CornerRadii};
use argui_text::TextStyle;
use argui_ui::{AlignItems, Element, Role, Semantics, Sides, length, percent};

use crate::WidgetTheme;

/// A themed surface with optional header, action and footer slots.
#[derive(Clone, Debug)]
pub struct Card {
    key: String,
    title: Option<String>,
    description: Option<String>,
    action: Option<Element>,
    content: Element,
    footer: Option<Element>,
    heading_level: u32,
}

impl Card {
    /// Creates a card around `content`.
    /// `key` identifies the card in the UI tree.
    #[must_use]
    pub fn new(key: impl Into<String>, content: Element) -> Self {
        Self {
            key: key.into(),
            title: None,
            description: None,
            action: None,
            content,
            footer: None,
            heading_level: 3,
        }
    }

    #[must_use]
    /// Adds a heading to the card.
    /// `title` is the heading text.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    #[must_use]
    /// Adds supporting text beneath the heading.
    /// `description` supplies the secondary card text.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    #[must_use]
    /// Adds an action element to the card heading.
    pub fn action(mut self, action: Element) -> Self {
        self.action = Some(action);
        self
    }

    #[must_use]
    /// Adds a footer element beneath the card content.
    pub fn footer(mut self, footer: Element) -> Self {
        self.footer = Some(footer);
        self
    }

    #[must_use]
    /// Sets the semantic heading level used for the title.
    pub fn heading_level(mut self, level: u32) -> Self {
        self.heading_level = level.clamp(1, 6);
        self
    }

    #[must_use]
    /// Builds the card using `theme` for its surface and text styling.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let title_key = format!("{}::title", self.key);
        let description_key = format!("{}::description", self.key);
        let mut root = Element::column([])
            .keyed(self.key)
            .semantics(Semantics::new(Role::Group))
            .width(percent(1.0))
            .min_width(length(0.0))
            .padding(Sides::length(24.0))
            .gap(24.0)
            .background(theme.card)
            .border(Border::all(1.0, theme.border))
            .radius(CornerRadii::all(12.0));
        let mut heading = Vec::with_capacity(2);
        if let Some(title) = self.title {
            root = root.labelled_by([title_key.clone()]);
            heading.push(
                Element::text(title.clone())
                    .keyed(title_key)
                    .text_style(TextStyle {
                        font_size: 18.0,
                        line_height: 24.0,
                        weight: 600,
                        color: theme.foreground,
                        ..TextStyle::default()
                    })
                    .semantics(
                        Semantics::new(Role::Heading)
                            .label(title)
                            .level(self.heading_level),
                    ),
            );
        }
        if let Some(description) = self.description {
            root = root.described_by([description_key.clone()]);
            heading.push(
                Element::text(description)
                    .keyed(description_key)
                    .text_style(TextStyle {
                        font_size: 14.0,
                        line_height: 21.0,
                        color: theme.muted_foreground,
                        ..TextStyle::default()
                    }),
            );
        }
        if !heading.is_empty() || self.action.is_some() {
            let mut header = Vec::with_capacity(2);
            if !heading.is_empty() {
                header.push(
                    Element::column(heading)
                        .gap(6.0)
                        .grow(1.0)
                        .min_width(length(0.0)),
                );
            }
            header.extend(self.action);
            root.children.push(
                Element::row(header)
                    .gap(16.0)
                    .align_items(AlignItems::START),
            );
        }
        root.children.push(self.content);
        root.children.extend(self.footer);
        root
    }
}
