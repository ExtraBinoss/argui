use crate::WidgetTheme;
use argui_paint::{Border, CornerRadii};
use argui_text::{TextAlign, TextStyle};
use argui_ui::{AlignItems, Element, Role, Semantics, Sides, length, percent};

/// A centered empty state with optional media, description and arbitrary actions.
#[derive(Clone, Debug)]
pub struct Empty {
    key: String,
    title: String,
    description: Option<String>,
    media: Option<Element>,
    content: Option<Element>,
    bordered: bool,
}

impl Empty {
    /// Creates an empty-state presentation with a title; `key` scopes the elements and `title` is the primary message.
    #[must_use]
    pub fn new(key: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            description: None,
            media: None,
            content: None,
            bordered: true,
        }
    }

    #[must_use]
    /// Adds explanatory text beneath the title; `description` supplies the supporting copy.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Decorative media. Put meaningful or interactive content in `content` instead.
    #[must_use]
    /// Adds media above the empty-state text.
    pub fn media(mut self, media: Element) -> Self {
        self.media = Some(media);
        self
    }

    #[must_use]
    /// Adds supplementary content below the text.
    pub fn content(mut self, content: Element) -> Self {
        self.content = Some(content);
        self
    }

    #[must_use]
    /// Sets whether the empty state has a visible border; `bordered` controls the surface outline.
    pub const fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    #[must_use]
    /// Builds the empty state using `theme` for its text and surface colors.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let title_key = format!("{}::title", self.key);
        let description_key = format!("{}::description", self.key);
        let mut root = Element::column([])
            .keyed(self.key)
            .semantics(Semantics::new(Role::Group))
            .labelled_by([title_key.clone()])
            .width(percent(1.0))
            .min_width(length(0.0))
            .padding(Sides::length(32.0))
            .align_items(AlignItems::CENTER)
            .gap(12.0)
            .radius(CornerRadii::all(12.0));
        if self.bordered {
            root = root.border(Border::all(1.0, theme.border));
        }
        root.children
            .extend(self.media.map(|media| media.semantic_hidden(true)));
        root.children.push(
            Element::text(self.title)
                .keyed(title_key)
                .text_style(TextStyle {
                    font_size: 18.0,
                    line_height: 25.0,
                    weight: 600,
                    color: theme.foreground,
                    ..TextStyle::default()
                })
                .width(percent(1.0))
                .text_align(TextAlign::Center),
        );
        if let Some(description) = self.description {
            root = root.described_by([description_key.clone()]);
            root.children.push(
                Element::text(description)
                    .keyed(description_key)
                    .text_style(TextStyle {
                        font_size: 14.0,
                        line_height: 21.0,
                        color: theme.muted_foreground,
                        ..TextStyle::default()
                    })
                    .width(percent(1.0))
                    .max_width(length(360.0))
                    .text_align(TextAlign::Center),
            );
        }
        root.children.extend(self.content);
        root
    }
}
