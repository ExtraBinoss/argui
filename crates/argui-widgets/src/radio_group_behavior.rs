use argui_ui::{Element, Orientation, Role, Semantics, UiEvent, UiEventKind};

use crate::{ToggleBehavior, TogglePart};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RadioGroupPart {
    Root,
    Option(usize),
    Indicator,
    Label,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RadioGroupAction {
    Select(usize),
}

#[derive(Clone, Debug)]
pub struct RadioGroupBehavior {
    key: String,
    label: String,
    options: Vec<(String, bool)>,
    selected: Option<usize>,
    orientation: Orientation,
}

impl RadioGroupBehavior {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        options: impl IntoIterator<Item = (String, bool)>,
        selected: Option<usize>,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            options: options.into_iter().collect(),
            selected,
            orientation: Orientation::Vertical,
        }
    }

    #[must_use]
    pub const fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    #[must_use]
    pub fn option_key(&self, index: usize) -> String {
        format!("{}::option::{index}", self.key)
    }

    #[must_use]
    pub fn decorate(&self, part: RadioGroupPart, element: Element) -> Element {
        match part {
            RadioGroupPart::Root => element.semantics(
                Semantics::new(Role::Group)
                    .label(self.label.clone())
                    .orientation(self.orientation),
            ),
            RadioGroupPart::Indicator => element.semantic_hidden(true),
            RadioGroupPart::Label => element.semantic_hidden(true),
            RadioGroupPart::Option(index) => {
                let (label, enabled) = self
                    .options
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| (String::new(), false));
                ToggleBehavior::new(
                    self.option_key(index),
                    label,
                    Role::RadioButton,
                    if self.selected == Some(index) {
                        argui_ui::CheckedState::Checked
                    } else {
                        argui_ui::CheckedState::Unchecked
                    },
                )
                .enabled(enabled)
                .position_in_set((index + 1) as u32, self.options.len() as u32)
                .decorate(TogglePart::Root, element)
            }
        }
    }

    #[must_use]
    pub fn action(&self, event: &UiEvent) -> Option<RadioGroupAction> {
        if !matches!(event.kind, UiEventKind::Click(_)) {
            return None;
        }
        let index = event
            .target_key()?
            .strip_prefix(&format!("{}::option::", self.key))?
            .parse::<usize>()
            .ok()?;
        self.options
            .get(index)
            .is_some_and(|(_, enabled)| *enabled)
            .then_some(RadioGroupAction::Select(index))
    }
}
