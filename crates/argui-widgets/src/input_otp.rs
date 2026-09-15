use crate::{Input, WidgetTheme};
use argui_text::{FontFamily, LetterSpacing};
use argui_ui::{Element, ElementKind, TextInputFilter, UiEvent, UiEventKind, length};

/// One accessible editor for a numeric verification code. Native editing handles selection,
/// arrows, deletion, clipboard and undo; the engine filter enforces length before mutation.
#[derive(Clone, Debug)]
pub struct InputOtp {
    pub key: String,
    pub label: String,
    pub value: String,
    pub length: usize,
    pub enabled: bool,
}

impl InputOtp {
    /// Creates a one-time-password input with a fixed number of slots.
    ///
    /// `key` identifies the control, `label` names it accessibly, `value` is the current
    /// digit string, and `length` is the desired slot count (clamped to 1 through 16).
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        value: impl Into<String>,
        length: usize,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            value: value.into(),
            length: length.clamp(1, 16),
            enabled: true,
        }
    }

    #[must_use]
    /// Returns the updated value when `event` changes or completes the input.
    pub fn action(&self, event: &UiEvent) -> Option<String> {
        if !self.enabled || event.target_key() != Some(&self.key) {
            return None;
        }
        let UiEventKind::TextChanged(value) = &event.kind else {
            return None;
        };
        (value.len() <= self.length.clamp(1, 16) && value.bytes().all(|byte| byte.is_ascii_digit()))
            .then(|| value.clone())
    }

    #[must_use]
    /// Returns whether every configured slot contains a character.
    pub fn complete(&self) -> bool {
        self.value.len() == self.length.clamp(1, 16)
            && self.value.bytes().all(|byte| byte.is_ascii_digit())
    }

    #[must_use]
    /// Builds the input slots using `theme` for their appearance.
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        let count = self.length.clamp(1, 16);
        let value: String = self
            .value
            .chars()
            .filter(char::is_ascii_digit)
            .take(count)
            .collect();
        let mut style = theme.input();
        style.text.family = FontFamily::Monospace;
        style.text.font_size = 20.0;
        style.text.letter_spacing = LetterSpacing::Px(12.0);
        style.placeholder = style.text.clone();
        style.placeholder.color = theme.muted_foreground;
        let mut input = Input::new(&self.key, value, "·".repeat(count), style)
            .label(&self.label)
            .enabled(self.enabled)
            .build()
            .width(length(count as f32 * 26.0 + 26.0));
        if let ElementKind::TextEditor { filter, .. } = &mut input.kind {
            *filter = TextInputFilter::Digits {
                max_length: count as u16,
            };
        }
        input
    }
}
