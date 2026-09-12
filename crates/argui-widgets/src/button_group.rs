use argui_ui::{AlignItems, Element, Orientation, Role, Semantics};

/// A named group of independently tabbable buttons and other controls.
#[derive(Clone, Debug)]
pub struct ButtonGroup {
    pub key: String,
    pub label: String,
    pub orientation: Orientation,
    pub children: Vec<Element>,
}

impl ButtonGroup {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        children: impl IntoIterator<Item = Element>,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            orientation: Orientation::Horizontal,
            children: children.into_iter().collect(),
        }
    }

    #[must_use]
    pub fn build(self) -> Element {
        let root = match self.orientation {
            Orientation::Horizontal => Element::row(self.children),
            Orientation::Vertical => Element::column(self.children),
        };
        root.keyed(self.key)
            .gap(1.0)
            .align_items(AlignItems::STRETCH)
            .semantics(Semantics::new(Role::Group).label(self.label))
    }
}
