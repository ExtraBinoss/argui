use crate::{Select, SelectAction, SelectBehavior, SelectOption, Typeahead, WidgetTheme};
use argui_ui::{Element, UiEvent};

/// Compact form select drawn by Argui on every platform; no system control or WebView.
/// Uses the same controlled selection, keyboard and typeahead contract as Select.
#[derive(Clone, Debug)]
pub struct NativeSelect {
    pub key: String,
    pub label: String,
    pub options: Vec<SelectOption>,
    pub selected: Option<usize>,
    pub highlighted: usize,
    pub open: bool,
    pub enabled: bool,
    pub required: bool,
}

impl NativeSelect {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        options: impl IntoIterator<Item = SelectOption>,
        selected: Option<usize>,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            options: options.into_iter().collect(),
            selected,
            highlighted: selected.unwrap_or(0),
            open: false,
            enabled: true,
            required: false,
        }
    }

    fn behavior(&self) -> SelectBehavior {
        SelectBehavior::new(&self.key, &self.label, self.options.clone(), self.selected)
            .open(self.open)
            .highlighted(self.highlighted)
    }

    #[must_use]
    pub fn action(&self, event: &UiEvent) -> Option<SelectAction> {
        self.enabled
            .then(|| self.behavior().action(event))
            .flatten()
    }

    pub fn search(
        &self,
        event: &UiEvent,
        state: &mut Typeahead,
        now: std::time::Duration,
    ) -> Option<SelectAction> {
        self.enabled
            .then(|| self.behavior().search(event, state, now))
            .flatten()
    }

    #[must_use]
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        let mut root =
            Select::new(&self.key, &self.label, self.options.clone(), self.selected)
                .open(self.open && self.enabled)
                .highlighted(self.highlighted)
                .trailing(Element::text("▾").semantic_hidden(true).text_style(
                    argui_text::TextStyle {
                        color: theme.foreground,
                        ..Default::default()
                    },
                ))
                .build(theme);
        let trigger = &mut root.children[0];
        let semantics = trigger.semantics.as_mut().expect("select trigger");
        semantics.role = argui_ui::Role::ComboBox;
        semantics.popup = Some(argui_ui::PopupKind::ListBox);
        semantics.state.required = self.required;
        semantics.state.disabled = !self.enabled;
        semantics.value = self
            .selected
            .and_then(|index| self.options.get(index))
            .map(|option| argui_ui::SemanticValue::Text(option.label.clone()));
        let interaction = trigger.interaction.as_mut().expect("select interaction");
        interaction.enabled = self.enabled;
        if !self.enabled {
            interaction.focus_policy = argui_ui::FocusPolicy::None;
        }
        root
    }
}
