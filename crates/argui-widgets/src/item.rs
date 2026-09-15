use crate::WidgetTheme;
use argui_paint::{Border, CornerRadii};
use argui_text::TextStyle;
use argui_ui::{AlignItems, Element, Role, Semantics, Sides, length, percent};

/// Reusable row with media, title, description and independently interactive actions.
#[derive(Clone, Debug)]
pub struct Item {
    pub key: String,
    pub title: String,
    pub description: Option<String>,
    pub media: Option<Element>,
    pub actions: Option<Element>,
    pub bordered: bool,
}

impl Item {
    /// Creates an item presentation with the supplied identity and title; `key` scopes its element identity.
    #[must_use]
    pub fn new(key: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            description: None,
            media: None,
            actions: None,
            bordered: true,
        }
    }

    #[must_use]
    /// Builds the item using `theme` for its text and surface styling.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let title_key = format!("{}::title", self.key);
        let title = Element::text(self.title)
            .keyed(&title_key)
            .text_style(TextStyle {
                color: theme.foreground,
                weight: 600,
                font_size: 14.0,
                ..TextStyle::default()
            });
        let mut content = vec![title];
        if let Some(description) = self.description {
            content.push(Element::text(description).text_style(TextStyle {
                color: theme.muted_foreground,
                font_size: 13.0,
                ..TextStyle::default()
            }));
        }
        let mut children: Vec<_> = self
            .media
            .into_iter()
            .map(|media| media.semantic_hidden(true).shrink(0.0))
            .collect();
        children.push(
            Element::column(content)
                .gap(3.0)
                .grow(1.0)
                .min_width(length(0.0)),
        );
        children.extend(self.actions);
        let root = Element::row(children)
            .keyed(self.key)
            .width(percent(1.0))
            .padding(Sides::length(12.0))
            .gap(12.0)
            .align_items(AlignItems::CENTER)
            .semantics(Semantics::new(Role::Group))
            .labelled_by([title_key]);
        if self.bordered {
            root.border(Border::all(1.0, theme.border))
                .radius(CornerRadii::all(8.0))
        } else {
            root
        }
    }
}
