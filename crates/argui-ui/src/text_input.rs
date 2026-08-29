use argui_core::{CaretAffinity, ImeInput, Key, KeyInput, KeyState, TextPosition};
use argui_paint::{Color, PaintStyle, QuadStyle};
use argui_text::{TextColor, TextStyle, TextWrap};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    Align, CursorIcon, Edges, Element, ElementKind, GestureSet, Interaction, LayoutStyle, Length,
    NodeId, Role, SemanticAction, SemanticValue, Semantics,
};

#[derive(Clone, Debug, PartialEq)]
pub struct TextInputStyle {
    pub layout: LayoutStyle,
    pub paint: PaintStyle,
    pub hovered: QuadStyle,
    pub focused: QuadStyle,
    pub text: TextStyle,
    pub placeholder: TextStyle,
    pub selection: Color,
    pub caret: Color,
}

impl TextInputStyle {
    #[must_use]
    pub fn new(paint: PaintStyle, mut text: TextStyle) -> Self {
        text.wrap = TextWrap::None;
        let mut placeholder = text.clone();
        placeholder.color = TextColor::rgba(0.55, 0.60, 0.68, 1.0);
        Self {
            layout: LayoutStyle {
                width: Length::Percent(1.0),
                padding: Edges::symmetric(13.0, 10.0),
                align: Align::Center,
                shrink: 0.0,
                ..LayoutStyle::default()
            },
            hovered: paint.quad.clone(),
            focused: paint.quad.clone(),
            paint,
            text,
            placeholder,
            selection: Color::rgba(0.20, 0.68, 0.94, 0.38),
            caret: Color::WHITE,
        }
    }

    #[must_use]
    pub fn hovered(mut self, style: QuadStyle) -> Self {
        self.hovered = style;
        self
    }

    #[must_use]
    pub fn focused(mut self, style: QuadStyle) -> Self {
        self.focused = style;
        self
    }

    #[must_use]
    pub const fn selection(mut self, color: Color) -> Self {
        self.selection = color;
        self
    }

    #[must_use]
    pub const fn caret(mut self, color: Color) -> Self {
        self.caret = color;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextInput {
    key: String,
    initial_value: String,
    placeholder: String,
    style: TextInputStyle,
}

impl TextInput {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        initial_value: impl Into<String>,
        placeholder: impl Into<String>,
        style: TextInputStyle,
    ) -> Self {
        Self {
            key: key.into(),
            initial_value: initial_value.into(),
            placeholder: placeholder.into(),
            style,
        }
    }

    #[must_use]
    pub fn build(self) -> Element {
        let interaction = Interaction::default()
            .focusable(true)
            .cursor(CursorIcon::Text)
            .gestures(GestureSet::NONE.tap().pan())
            .hovered(self.style.hovered)
            .focused(self.style.focused);
        let semantics = Semantics::new(Role::TextInput)
            .label(self.placeholder.clone())
            .value(SemanticValue::Text(self.initial_value.clone()))
            .action(SemanticAction::Focus)
            .action(SemanticAction::SetValue);
        let mut element = Element::container([]).semantics(semantics);
        element.key = Some(self.key);
        element.kind = ElementKind::TextInput {
            initial_value: self.initial_value,
            placeholder: self.placeholder,
            text: self.style.text,
            placeholder_text: self.style.placeholder,
            selection: self.style.selection,
            caret: self.style.caret,
        };
        element.style = self.style.layout;
        element.paint = self.style.paint;
        element.interaction = Some(interaction);
        element
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct TextInputState {
    value: String,
    cursor: usize,
    affinity: CaretAffinity,
    anchor: Option<TextPosition>,
    preedit: Option<Preedit>,
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct EditResult {
    pub changed: bool,
    pub submitted: bool,
    pub layout: bool,
    pub reshape: bool,
    pub clipboard: Option<ClipboardRequest>,
}

impl TextInputState {
    pub fn new(value: String) -> Self {
        let cursor = value.len();
        Self {
            value,
            cursor,
            ..Self::default()
        }
    }

    pub fn value(&self) -> &str {
        &self.value
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
            Key::Home => self.move_to(0),
            Key::End => self.move_to(self.value.len()),
            Key::Backspace => result.changed = self.backspace(),
            Key::Delete => result.changed = self.delete(),
            Key::Enter => result.submitted = true,
            Key::Escape => self.anchor = None,
            _ if !input.modifiers.alt => {
                if let Some(text) = input.text.as_deref().filter(|text| !text.is_empty()) {
                    self.insert(text);
                    result.changed = true;
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
                self.insert(&text);
                return EditResult {
                    changed: !text.is_empty(),
                    layout: !text.is_empty(),
                    reshape: !text.is_empty(),
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
        self.insert(text);
        EditResult {
            changed: !text.is_empty(),
            layout: !text.is_empty(),
            reshape: !text.is_empty(),
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
                let changed = clipboard.is_some() && self.delete_selection();
                EditResult {
                    changed,
                    layout: changed,
                    reshape: changed,
                    clipboard,
                    ..EditResult::default()
                }
            }
            "v" => EditResult {
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

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TextInputStates {
    states: Vec<(NodeId, TextInputState)>,
    drag: Option<NodeId>,
}

impl TextInputStates {
    pub fn get(&self, node: NodeId) -> Option<&TextInputState> {
        self.states
            .iter()
            .find_map(|(id, state)| (*id == node).then_some(state))
    }

    pub fn get_mut(&mut self, node: NodeId) -> Option<&mut TextInputState> {
        self.states
            .iter_mut()
            .find_map(|(id, state)| (*id == node).then_some(state))
    }

    pub fn sync(&mut self, inputs: impl IntoIterator<Item = (NodeId, String)>) {
        let inputs = inputs.into_iter().collect::<Vec<_>>();
        self.states
            .retain(|(node, _)| inputs.iter().any(|(id, _)| id == node));
        if self.drag.is_some_and(|node| self.get(node).is_none()) {
            self.drag = None;
        }
        for (node, value) in inputs {
            if self.get(node).is_none() {
                self.states.push((node, TextInputState::new(value)));
            }
        }
    }

    pub fn place(&mut self, node: NodeId, index: usize, extend: bool) -> Option<EditResult> {
        self.place_position(
            node,
            TextPosition::new(index, CaretAffinity::Before),
            extend,
        )
    }

    pub fn place_position(
        &mut self,
        node: NodeId,
        position: TextPosition,
        extend: bool,
    ) -> Option<EditResult> {
        let result = self.get_mut(node)?.place_position(position, extend);
        self.drag = Some(node);
        Some(result)
    }

    pub fn drag(&mut self, node: NodeId, index: usize) -> Option<EditResult> {
        self.drag_position(node, TextPosition::new(index, CaretAffinity::Before))
    }

    pub fn drag_position(&mut self, node: NodeId, position: TextPosition) -> Option<EditResult> {
        if self.drag != Some(node) {
            return None;
        }
        self.get_mut(node)
            .map(|state| state.place_position(position, true))
    }

    pub fn move_to(&mut self, node: NodeId, index: usize, extend: bool) -> Option<EditResult> {
        self.move_to_position(
            node,
            TextPosition::new(index, CaretAffinity::Before),
            extend,
        )
    }

    pub fn move_to_position(
        &mut self,
        node: NodeId,
        position: TextPosition,
        extend: bool,
    ) -> Option<EditResult> {
        self.get_mut(node)
            .map(|state| state.place_position(position, extend))
    }

    pub const fn dragging(&self) -> bool {
        self.drag.is_some()
    }

    pub fn release(&mut self) -> bool {
        self.drag.take().is_some()
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
