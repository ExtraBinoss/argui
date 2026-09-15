use super::TextInputState;
use argui_core::TextPosition;
use unicode_segmentation::UnicodeSegmentation;

/// Password policies suppress clipboard export and history even while revealed.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextPrivacy {
    #[default]
    Public,
    Password,
    RevealedPassword,
}

impl TextPrivacy {
    /// Returns whether this mode protects text from clipboard and history access.
    #[must_use]
    pub const fn protected(self) -> bool {
        !matches!(self, Self::Public)
    }
}

impl crate::Element {
    /// Sets how this element's text is displayed and retained.
    ///
    /// * `privacy` — public, password, or revealed-password policy.
    #[must_use]
    pub fn text_privacy(mut self, privacy: TextPrivacy) -> Self {
        self.text_privacy = privacy;
        for child in &mut self.children {
            *child = child.clone().text_privacy(privacy);
        }
        if privacy.protected()
            && let Some(semantics) = &mut self.semantics
        {
            semantics.value = None;
            semantics.state.protected = true;
        }
        self
    }
}

impl TextInputState {
    pub fn protected(&self) -> bool {
        self.privacy.protected()
    }
    pub fn set_privacy(&mut self, privacy: TextPrivacy) {
        if self.privacy != privacy {
            self.history.clear();
            self.preedit = None;
            self.privacy = privacy;
        }
    }
    pub(super) fn mask_position(&self, position: TextPosition) -> TextPosition {
        if self.privacy != TextPrivacy::Password {
            return position;
        }
        let display = self.unmasked_display();
        let count = display
            .grapheme_indices(true)
            .take_while(|(index, _)| *index < position.index)
            .count();
        TextPosition::new(count * "•".len(), position.affinity)
    }
    pub(super) fn unmask_index(&self, index: usize) -> usize {
        if self.privacy != TextPrivacy::Password {
            return super::grapheme_boundary(&self.value, index.min(self.value.len()));
        }
        self.value
            .grapheme_indices(true)
            .nth(index / "•".len())
            .map_or(self.value.len(), |(index, _)| index)
    }
}

impl std::fmt::Debug for TextInputState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TextInputState")
            .field(
                "value",
                &if self.protected() {
                    "[protected]"
                } else {
                    &self.value
                },
            )
            .field("cursor", &self.cursor)
            .field("privacy", &self.privacy)
            .finish()
    }
}
