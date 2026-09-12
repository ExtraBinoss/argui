use crate::{Separator, Typography, TypographyVariant, WidgetTheme};
use argui_ui::{AlignItems, Element, LiveRegion, Role, Semantics, Sides};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MarkerVariant {
    #[default]
    Inline,
    Border,
    Separator,
}

/// A system note in a conversation. Announcements are opt-in.
#[derive(Clone, Debug)]
pub struct Marker {
    pub key: String,
    pub text: String,
    pub icon: Option<Element>,
    pub variant: MarkerVariant,
    pub live: LiveRegion,
}

impl Marker {
    #[must_use]
    pub fn new(key: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            text: text.into(),
            icon: None,
            variant: MarkerVariant::Inline,
            live: LiveRegion::Off,
        }
    }

    #[must_use]
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let root = if self.variant == MarkerVariant::Separator {
            Separator::new(&self.key).label(&self.text).build(theme)
        } else {
            let row = Element::row(
                self.icon
                    .into_iter()
                    .map(|icon| icon.semantic_hidden(true))
                    .chain([Typography::new(&self.text, TypographyVariant::Muted).build(theme)]),
            )
            .gap(8.0)
            .align_items(AlignItems::CENTER)
            .padding(Sides::length(8.0));
            if self.variant == MarkerVariant::Border {
                Element::column([
                    row,
                    Separator::new(format!("{}::rule", self.key)).build(theme),
                ])
            } else {
                row
            }
        };
        let mut root = root.keyed(self.key).semantics(
            Semantics::new(if self.live == LiveRegion::Off {
                Role::Text
            } else {
                Role::Status
            })
            .label(self.text)
            .live(self.live),
        );
        for child in &mut root.children {
            child.semantic_hidden = true;
        }
        root
    }
}
