use argui_core::{CaretAffinity, ImeInput, Key, KeyInput, KeyState, TextPosition};
use unicode_segmentation::UnicodeSegmentation;

use crate::FocusTarget;

mod filter;
pub use filter::TextInputFilter;
mod states;
pub(crate) use states::TextInputStates;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct TextInputState {
    value: String,
    cursor: usize,
    affinity: CaretAffinity,
    anchor: Option<TextPosition>,
    preedit: Option<Preedit>,
    multiline: bool,
    read_only: bool,
    filter: TextInputFilter,
    reveal_cursor: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Preedit {
    text: String,
    cursor: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClipboardRequest {
    Read,
    Write(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextSelection {
    All,
    Caret(TextPosition),
    Range {
        anchor: TextPosition,
        cursor: TextPosition,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextSelectionRequest {
    pub target: FocusTarget,
    pub selection: TextSelection,
}

impl TextSelectionRequest {
    #[must_use]
    pub fn new(target: impl Into<FocusTarget>, selection: TextSelection) -> Self {
        Self {
            target: target.into(),
            selection,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct EditResult {
    pub changed: bool,
    pub submitted: bool,
    pub layout: bool,
    pub reshape: bool,
    pub clipboard: Option<ClipboardRequest>,
}

impl TextInputState {
    pub fn new(value: String, multiline: bool, read_only: bool, filter: TextInputFilter) -> Self {
        let cursor = value.len();
        Self {
            value,
            cursor,
            multiline,
            read_only,
            filter,
            reveal_cursor: true,
            ..Self::default()
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn sync(&mut self, value: &str, multiline: bool, read_only: bool, filter: TextInputFilter) {
        self.multiline = multiline;
        self.read_only = read_only;
        self.filter = filter;
        if self.value == value {
            return;
        }
        self.value.clear();
        self.value.push_str(value);
        self.cursor = grapheme_boundary(&self.value, self.cursor.min(self.value.len()));
        self.anchor = self.anchor.and_then(|anchor| {
            (anchor.index <= self.value.len()).then(|| {
                TextPosition::new(
                    grapheme_boundary(&self.value, anchor.index),
                    anchor.affinity,
                )
            })
        });
        self.preedit = None;
        self.reveal_cursor = true;
    }

    pub fn display_cursor(&self) -> usize {
        self.display_position().index
    }

    pub fn display_position(&self) -> TextPosition {
        self.preedit
            .as_ref()
            .map_or(TextPosition::new(self.cursor, self.affinity), |preedit| {
                TextPosition::new(self.cursor + preedit.cursor, CaretAffinity::Before)
            })
    }

    pub fn selection(&self) -> Option<(usize, usize)> {
        let anchor = self.anchor?;
        (anchor.index != self.cursor).then(|| ordered(anchor.index, self.cursor))
    }

    pub fn selection_positions(&self) -> Option<(TextPosition, TextPosition)> {
        let anchor = self.anchor?;
        (anchor != self.display_position()).then(|| (anchor, self.display_position()))
    }

    pub fn display(&self) -> String {
        let Some(preedit) = &self.preedit else {
            return self.value.clone();
        };
        let mut display = self.value.clone();
        display.insert_str(self.cursor, &preedit.text);
        display
    }

    pub fn key(&mut self, input: &KeyInput) -> EditResult {
        if input.state != KeyState::Pressed {
            return EditResult::default();
        }
        let command = input.modifiers.command();
        if command && let Key::Character(key) = &input.key {
            return self.shortcut(key);
        }
        let before = (
            self.cursor,
            self.affinity,
            self.anchor,
            self.preedit.clone(),
        );
        let old_cursor = TextPosition::new(self.cursor, self.affinity);
        let mut result = EditResult::default();
        match input.key {
            Key::ArrowLeft if command || input.modifiers.alt => {
                self.move_to(previous_word(&self.value, self.cursor));
            }
            Key::ArrowRight if command || input.modifiers.alt => {
                self.move_to(next_word(&self.value, self.cursor));
            }
            Key::ArrowLeft => self.move_to(previous_grapheme(&self.value, self.cursor)),
            Key::ArrowRight => self.move_to(next_grapheme(&self.value, self.cursor)),
            Key::Home => self.move_to(if self.multiline && !command {
                line_start(&self.value, self.cursor)
            } else {
                0
            }),
            Key::End => self.move_to(if self.multiline && !command {
                line_end(&self.value, self.cursor)
            } else {
                self.value.len()
            }),
            Key::Backspace if !self.read_only => result.changed = self.backspace(),
            Key::Delete if !self.read_only => result.changed = self.delete(),
            Key::Enter if self.multiline && !command && !self.read_only => {
                self.insert("\n");
                result.changed = true;
            }
            Key::Enter if self.multiline && self.read_only => {}
            Key::Enter => result.submitted = true,
            Key::Escape => self.anchor = None,
            _ if !self.read_only && !input.modifiers.alt => {
                if let Some(text) = input.text.as_deref().filter(|text| !text.is_empty()) {
                    result.changed = self.insert_input(text);
                }
            }
            _ => {}
        }
        if matches!(
            input.key,
            Key::ArrowLeft | Key::ArrowRight | Key::Home | Key::End
        ) {
            self.extend_selection(old_cursor, input.modifiers.shift);
        }
        result.layout = before
            != (
                self.cursor,
                self.affinity,
                self.anchor,
                self.preedit.clone(),
            )
            || result.changed;
        result.reshape = result.changed;
        result
    }

    pub fn ime(&mut self, input: ImeInput) -> EditResult {
        if self.read_only {
            return EditResult::default();
        }
        let before = self.preedit.clone();
        match input {
            ImeInput::Preedit { text, cursor } => {
                self.preedit = (!text.is_empty()).then(|| {
                    let cursor = cursor.map_or(text.len(), |(_, end)| end.min(text.len()));
                    Preedit {
                        cursor: char_boundary(&text, cursor),
                        text,
                    }
                });
            }
            ImeInput::Commit(text) => {
                self.preedit = None;
                let changed = self.insert_input(&text);
                return EditResult {
                    changed,
                    layout: changed,
                    reshape: changed,
                    ..EditResult::default()
                };
            }
            ImeInput::Disabled => self.preedit = None,
            ImeInput::Enabled => {}
        }
        EditResult {
            layout: before != self.preedit,
            reshape: before != self.preedit,
            ..EditResult::default()
        }
    }

    pub fn paste(&mut self, text: &str) -> EditResult {
        if self.read_only {
            return EditResult::default();
        }
        let changed = self.insert_input(text);
        EditResult {
            changed,
            layout: changed,
            reshape: changed,
            ..EditResult::default()
        }
    }

    pub fn place_position(&mut self, position: TextPosition, extend: bool) -> EditResult {
        let old = TextPosition::new(self.cursor, self.affinity);
        self.move_to(position.index.min(self.value.len()));
        self.affinity = position.affinity;
        self.extend_selection(old, extend);
        EditResult {
            layout: true,
            ..EditResult::default()
        }
    }

    pub fn select(&mut self, selection: TextSelection) -> EditResult {
        let boundary = |position: TextPosition| {
            TextPosition::new(
                grapheme_boundary(&self.value, position.index.min(self.value.len())),
                position.affinity,
            )
        };
        match selection {
            TextSelection::All => {
                self.anchor = Some(TextPosition::new(0, CaretAffinity::After));
                self.cursor = self.value.len();
                self.affinity = CaretAffinity::Before;
            }
            TextSelection::Caret(position) => {
                let position = boundary(position);
                self.anchor = None;
                self.cursor = position.index;
                self.affinity = position.affinity;
            }
            TextSelection::Range { anchor, cursor } => {
                let anchor = boundary(anchor);
                let cursor = boundary(cursor);
                self.anchor = (anchor != cursor).then_some(anchor);
                self.cursor = cursor.index;
                self.affinity = cursor.affinity;
            }
        }
        self.preedit = None;
        self.reveal_cursor = true;
        EditResult {
            layout: true,
            ..EditResult::default()
        }
    }

    fn shortcut(&mut self, key: &str) -> EditResult {
        match key.to_lowercase().as_str() {
            "a" => {
                self.anchor = Some(TextPosition::new(0, CaretAffinity::After));
                self.cursor = self.value.len();
                self.affinity = CaretAffinity::Before;
                EditResult {
                    layout: true,
                    ..EditResult::default()
                }
            }
            "c" => EditResult {
                clipboard: self.selected_text().map(ClipboardRequest::Write),
                ..EditResult::default()
            },
            "x" => {
                let clipboard = self.selected_text().map(ClipboardRequest::Write);
                if self.read_only {
                    return EditResult {
                        clipboard,
                        ..EditResult::default()
                    };
                }
                let changed = clipboard.is_some() && self.delete_selection();
                EditResult {
                    changed,
                    layout: changed,
                    reshape: changed,
                    clipboard,
                    ..EditResult::default()
                }
            }
            "v" if !self.read_only => EditResult {
                clipboard: Some(ClipboardRequest::Read),
                ..EditResult::default()
            },
            _ => EditResult::default(),
        }
    }

    fn selected_text(&self) -> Option<String> {
        let (start, end) = self.selection()?;
        Some(self.value[start..end].to_owned())
    }

    fn insert(&mut self, text: &str) {
        self.delete_selection();
        self.value.insert_str(self.cursor, text);
        self.cursor += text.len();
        self.affinity = CaretAffinity::Before;
        self.anchor = None;
    }

    fn insert_input(&mut self, text: &str) -> bool {
        let single_line;
        let text = if self.multiline {
            text
        } else {
            single_line = text.replace(['\r', '\n'], "");
            &single_line
        };
        if text.is_empty() {
            return false;
        }
        let (start, end) = self.selection().unwrap_or((self.cursor, self.cursor));
        let mut candidate = String::with_capacity(self.value.len() + text.len());
        candidate.push_str(&self.value[..start]);
        candidate.push_str(text);
        candidate.push_str(&self.value[end..]);
        if !self.filter.accepts(&candidate) {
            return false;
        }
        self.insert(text);
        true
    }

    fn backspace(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        let start = previous_grapheme(&self.value, self.cursor);
        if start == self.cursor {
            return false;
        }
        self.value.replace_range(start..self.cursor, "");
        self.cursor = start;
        self.affinity = CaretAffinity::After;
        true
    }

    fn delete(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        let end = next_grapheme(&self.value, self.cursor);
        if end == self.cursor {
            return false;
        }
        self.value.replace_range(self.cursor..end, "");
        true
    }

    fn delete_selection(&mut self) -> bool {
        let Some((start, end)) = self.selection() else {
            return false;
        };
        self.value.replace_range(start..end, "");
        self.cursor = start;
        self.affinity = CaretAffinity::After;
        self.anchor = None;
        true
    }

    fn move_to(&mut self, cursor: usize) {
        self.cursor = cursor;
        self.affinity = CaretAffinity::Before;
        self.preedit = None;
    }

    fn extend_selection(&mut self, old_cursor: TextPosition, extend: bool) {
        if extend {
            if self.anchor.is_none() {
                self.anchor = Some(old_cursor);
            }
        } else {
            self.anchor = None;
        }
    }
}

const fn ordered(a: usize, b: usize) -> (usize, usize) {
    if a <= b { (a, b) } else { (b, a) }
}

fn previous_grapheme(value: &str, cursor: usize) -> usize {
    value[..cursor]
        .grapheme_indices(true)
        .next_back()
        .map_or(0, |(index, _)| index)
}

fn next_grapheme(value: &str, cursor: usize) -> usize {
    value[cursor..]
        .grapheme_indices(true)
        .nth(1)
        .map_or(value.len(), |(index, _)| cursor + index)
}

fn previous_word(value: &str, cursor: usize) -> usize {
    value[..cursor]
        .unicode_word_indices()
        .next_back()
        .map_or(0, |(index, _)| index)
}

fn next_word(value: &str, cursor: usize) -> usize {
    value[cursor..]
        .unicode_word_indices()
        .map(|(index, word)| cursor + index + word.len())
        .next()
        .unwrap_or(value.len())
}

fn char_boundary(value: &str, index: usize) -> usize {
    (0..=index)
        .rev()
        .find(|candidate| value.is_char_boundary(*candidate))
        .unwrap_or(0)
}

fn grapheme_boundary(value: &str, index: usize) -> usize {
    value[..char_boundary(value, index)]
        .grapheme_indices(true)
        .next_back()
        .map_or(0, |(boundary, grapheme)| boundary + grapheme.len())
        .min(index)
}

fn line_start(value: &str, cursor: usize) -> usize {
    value[..cursor].rfind('\n').map_or(0, |index| index + 1)
}

fn line_end(value: &str, cursor: usize) -> usize {
    value[cursor..]
        .find('\n')
        .map_or(value.len(), |index| cursor + index)
}
