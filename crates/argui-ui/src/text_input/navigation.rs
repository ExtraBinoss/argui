use argui_core::{Key, KeyInput};
use unicode_segmentation::UnicodeSegmentation;

use super::{TextInputState, line_end, line_start, next_grapheme, previous_grapheme};

impl TextInputState {
    pub(super) fn navigate(&mut self, input: &KeyInput) {
        let by_word = input.modifiers.command() || input.modifiers.alt;
        let left = input.key == Key::ArrowLeft;
        let cursor = match input.key {
            Key::ArrowLeft | Key::ArrowRight => {
                if !by_word
                    && !input.modifiers.shift
                    && let Some((start, end)) = self.selection()
                {
                    if left { start } else { end }
                } else if by_word {
                    if left {
                        previous_word(&self.value, self.cursor)
                    } else {
                        next_word(&self.value, self.cursor)
                    }
                } else if left {
                    previous_grapheme(&self.value, self.cursor)
                } else {
                    next_grapheme(&self.value, self.cursor)
                }
            }
            Key::Home => {
                if self.multiline && !input.modifiers.command() {
                    line_start(&self.value, self.cursor)
                } else {
                    0
                }
            }
            Key::End => {
                if self.multiline && !input.modifiers.command() {
                    line_end(&self.value, self.cursor)
                } else {
                    self.value.len()
                }
            }
            _ => return,
        };
        self.move_to(cursor);
    }
}

pub(super) fn previous_word(value: &str, cursor: usize) -> usize {
    value[..cursor]
        .split_word_bound_indices()
        .rfind(|(_, part)| part.contains('\n') || !part.chars().all(char::is_whitespace))
        .map_or(0, |(index, _)| index)
}

pub(super) fn next_word(value: &str, cursor: usize) -> usize {
    value[cursor..]
        .split_word_bound_indices()
        .find(|(_, part)| part.contains('\n') || !part.chars().all(char::is_whitespace))
        .map_or(value.len(), |(index, part)| cursor + index + part.len())
}
