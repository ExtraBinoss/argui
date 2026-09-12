use crate::{Label, WidgetTheme};
use argui_text::TextStyle;
use argui_ui::{Element, LiveRegion, Role, Semantics, UiEvent};

/// Label, help and validation for the control whose key is `control_key`.
/// The target can be nested (for example an Input with a leading icon).
#[derive(Clone, Debug)]
pub struct Field {
    pub key: String,
    pub label: String,
    pub control_key: String,
    pub control: Element,
    pub description: Option<String>,
    pub error: Option<String>,
    pub required: bool,
    pub enabled: bool,
}

impl Field {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        control_key: impl Into<String>,
        control: Element,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            control_key: control_key.into(),
            control,
            description: None,
            error: None,
            required: false,
            enabled: true,
        }
    }

    fn label_widget(&self) -> Label {
        Label::new(
            format!("{}::label", self.key),
            &self.label,
            &self.control_key,
        )
        .enabled(self.enabled)
    }

    #[must_use]
    pub fn focus_target(&self, event: &UiEvent) -> Option<&str> {
        self.label_widget()
            .focus_target(event)
            .map(|_| self.control_key.as_str())
    }

    #[must_use]
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        let label = self.label_widget();
        let mut control = self.control.clone();
        let mut notes = Vec::new();
        let mut descriptions = Vec::new();
        for (part, value, color, live) in [
            (
                "help",
                &self.description,
                theme.muted_foreground,
                LiveRegion::Off,
            ),
            ("error", &self.error, theme.destructive, LiveRegion::Polite),
        ] {
            if let Some(value) = value {
                let key = format!("{}::{part}", self.key);
                notes.push(
                    Element::text(value.as_str())
                        .keyed(&key)
                        .text_style(TextStyle {
                            color,
                            font_size: 13.0,
                            ..TextStyle::default()
                        })
                        .semantics(Semantics::new(Role::Text).label(value).live(live)),
                );
                descriptions.push(key);
            }
        }
        self.associate(&mut control, &label, &descriptions);
        Element::column([label.build(theme), control].into_iter().chain(notes))
            .keyed(&self.key)
            .gap(6.0)
    }

    fn associate(&self, element: &mut Element, label: &Label, descriptions: &[String]) {
        if element.key.as_deref() == Some(&self.control_key) {
            *element = label
                .associate(element.clone())
                .described_by(descriptions.iter().cloned());
            if let Some(semantics) = &mut element.semantics {
                semantics.state.required = self.required;
                semantics.state.invalid = self.error.is_some();
                semantics.state.disabled |= !self.enabled;
            }
            if !self.enabled
                && let Some(interaction) = &mut element.interaction
            {
                interaction.enabled = false;
                interaction.focus_policy = argui_ui::FocusPolicy::None;
            }
        } else {
            for child in &mut element.children {
                self.associate(child, label, descriptions);
            }
        }
    }
}
