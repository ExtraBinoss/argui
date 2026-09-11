use argui_text::{TextStyle, TextWrap};
use argui_ui::{
    CursorIcon, Element, GestureSet, Interaction, Role, Semantics, TapGesture, UiEvent,
    UiEventKind, UserSelect,
};

use crate::WidgetTheme;

/// Visible label for a separately mounted control in the same semantic scope.
/// Apply `focus_target(event)` with the consumer's context to focus the control.
#[derive(Clone, Debug)]
pub struct Label {
    key: String,
    text: String,
    target: String,
    enabled: bool,
}

impl Label {
    #[must_use]
    pub fn new(key: impl Into<String>, text: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            text: text.into(),
            target: target.into(),
            enabled: true,
        }
    }

    /// Keep this flag in sync with the associated control's enabled state.
    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Assigns the target key and accessible label, preserving other control properties.
    #[must_use]
    pub fn associate(&self, mut control: Element) -> Element {
        if let Some(semantics) = &mut control.semantics {
            semantics.label = None;
        }
        control
            .keyed(self.target.clone())
            .labelled_by([self.key.clone()])
    }

    #[must_use]
    pub fn focus_target(&self, event: &UiEvent) -> Option<&str> {
        (self.enabled
            && event.target_key() == Some(self.key.as_str())
            && matches!(event.kind, UiEventKind::Click(_)))
        .then_some(self.target.as_str())
    }

    #[must_use]
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        Element::text(self.text.clone())
            .keyed(self.key.clone())
            .text_style(TextStyle {
                font_size: 14.0,
                line_height: 20.0,
                weight: 500,
                color: if self.enabled {
                    theme.foreground
                } else {
                    theme.muted_foreground
                },
                wrap: TextWrap::WordOrGlyph,
                ..TextStyle::default()
            })
            .semantics(Semantics::new(Role::Text).label(self.text.clone()))
            .user_select(UserSelect::None)
            .interaction(
                Interaction::default()
                    .enabled(self.enabled)
                    .cursor(if self.enabled {
                        CursorIcon::Pointer
                    } else {
                        CursorIcon::NotAllowed
                    })
                    .gestures(GestureSet::EMPTY.tap(TapGesture::default())),
            )
    }
}
