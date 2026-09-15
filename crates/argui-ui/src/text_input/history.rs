use std::{collections::VecDeque, time::Duration};

use argui_core::{CaretAffinity, ImeInput, Key, KeyInput, KeyState, TextPosition};
use web_time::Instant;

use super::{EditResult, TextInputState};

/// Per-editor limits. Zero disables history; the text itself is not size limited.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HistoryConfig {
    pub transactions: usize,
    pub bytes: usize,
    pub grouping_interval: Duration,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            transactions: 100,
            bytes: 1024 * 1024,
            grouping_interval: Duration::from_millis(750),
        }
    }
}

impl crate::Element {
    /// Sets history limits for this element and its descendants.
    ///
    /// * `config` — per-editor transaction and byte limits.
    #[must_use]
    pub fn text_history(mut self, config: HistoryConfig) -> Self {
        self.text_history = Some(config);
        for child in &mut self.children {
            *child = child.clone().text_history(config);
        }
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum EditKind {
    Typing,
    Backspace,
    Delete,
    Atomic,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Caret {
    cursor: usize,
    affinity: CaretAffinity,
    anchor: Option<TextPosition>,
}

impl Caret {
    fn capture(state: &TextInputState) -> Self {
        Self {
            cursor: state.cursor,
            affinity: state.affinity,
            anchor: state.anchor,
        }
    }
    fn restore(self, state: &mut TextInputState) {
        state.cursor = self.cursor;
        state.affinity = self.affinity;
        state.anchor = self.anchor;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Replacement {
    start: usize,
    removed: String,
    inserted: String,
    before: Caret,
    after: Caret,
}

impl Replacement {
    fn between(before: &str, after: &str, caret: Caret, next: Caret) -> Self {
        let mut start = before
            .bytes()
            .zip(after.bytes())
            .take_while(|(a, b)| a == b)
            .count();
        while !before.is_char_boundary(start) || !after.is_char_boundary(start) {
            start -= 1;
        }
        let mut suffix = before[start..]
            .bytes()
            .rev()
            .zip(after[start..].bytes().rev())
            .take_while(|(a, b)| a == b)
            .count();
        while !before.is_char_boundary(before.len() - suffix)
            || !after.is_char_boundary(after.len() - suffix)
        {
            suffix -= 1;
        }
        Self {
            start,
            removed: before[start..before.len() - suffix].into(),
            inserted: after[start..after.len() - suffix].into(),
            before: caret,
            after: next,
        }
    }
    fn bytes(&self) -> usize {
        self.removed.len() + self.inserted.len()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct History {
    config: HistoryConfig,
    undo: VecDeque<Vec<Replacement>>,
    redo: Vec<Vec<Replacement>>,
    last: Option<(EditKind, Instant, Caret)>,
    bytes: usize,
}

impl History {
    pub(super) fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.last = None;
        self.bytes = 0;
    }
    pub(super) fn break_group(&mut self) {
        self.last = None;
    }
    fn push(&mut self, replacement: Replacement, kind: EditKind, now: Instant) {
        for transaction in self.redo.drain(..) {
            self.bytes -= transaction.iter().map(Replacement::bytes).sum::<usize>();
        }
        if self.config.transactions == 0 || replacement.bytes() > self.config.bytes {
            self.clear();
            return;
        }
        let joins = kind != EditKind::Atomic
            && !self.config.grouping_interval.is_zero()
            && self.last.is_some_and(|(previous, time, caret)| {
                previous == kind
                    && caret == replacement.before
                    && caret.anchor.is_none()
                    && now.saturating_duration_since(time) <= self.config.grouping_interval
            });
        self.last = Some((kind, now, replacement.after));
        self.bytes += replacement.bytes();
        if joins && let Some(transaction) = self.undo.back_mut() {
            transaction.push(replacement);
        } else {
            self.undo.push_back(vec![replacement]);
        }
        while self.undo.len() > self.config.transactions || self.bytes > self.config.bytes {
            if let Some(transaction) = self.undo.pop_front() {
                self.bytes -= transaction.iter().map(Replacement::bytes).sum::<usize>();
            } else {
                break;
            }
        }
        if self.undo.is_empty() {
            self.break_group();
        }
    }
}

impl TextInputState {
    pub(super) fn record_edit(
        &mut self,
        kind: EditKind,
        edit: impl FnOnce(&mut Self) -> EditResult,
    ) -> EditResult {
        self.goal_x = None;
        if self.protected() {
            return edit(self);
        }
        let before = self.value.clone();
        let caret = Caret::capture(self);
        let result = edit(self);
        if result.changed && self.value != before && !self.protected() {
            self.history.push(
                Replacement::between(&before, &self.value, caret, Caret::capture(self)),
                kind,
                Instant::now(),
            );
        }
        result
    }
    pub fn replace_value(&mut self, value: &str) -> EditResult {
        self.record_edit(EditKind::Atomic, |state| state.replace_value_inner(value))
    }
    pub fn paste(&mut self, text: &str) -> EditResult {
        self.record_edit(EditKind::Atomic, |state| state.paste_inner(text))
    }
    pub fn ime(&mut self, input: ImeInput) -> EditResult {
        self.history.break_group();
        self.record_edit(EditKind::Atomic, |state| state.ime_inner(input))
    }
    pub fn key(&mut self, input: &KeyInput) -> EditResult {
        if input.state != KeyState::Pressed {
            return EditResult::default();
        }
        if input.modifiers.command()
            && let Key::Character(key) = &input.key
        {
            if key.eq_ignore_ascii_case("z") {
                return self.history_step(input.modifiers.shift);
            }
            if key.eq_ignore_ascii_case("y") && !cfg!(target_os = "macos") {
                return self.history_step(true);
            }
        }
        if self.preedit.is_some() {
            return EditResult::default();
        }
        let kind = match &input.key {
            Key::Backspace if !input.modifiers.command() && !input.modifiers.alt => {
                EditKind::Backspace
            }
            Key::Delete if !input.modifiers.command() && !input.modifiers.alt => EditKind::Delete,
            Key::Character(_) if !input.modifiers.command() && !input.modifiers.alt => {
                EditKind::Typing
            }
            _ => EditKind::Atomic,
        };
        if kind == EditKind::Atomic {
            self.history.break_group();
        }
        self.record_edit(kind, |state| state.key_inner(input))
    }
    pub fn composing(&self) -> bool {
        self.preedit.is_some()
    }
    pub fn break_edit_group(&mut self) {
        self.history.break_group();
    }
    pub fn configure_history(&mut self, config: HistoryConfig) {
        if self.history.config != config {
            self.history.clear();
            self.history.config = config;
        }
    }
    pub fn can_undo(&self) -> bool {
        !self.read_only && !self.protected() && !self.history.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.read_only && !self.protected() && !self.history.redo.is_empty()
    }
    pub fn history_step(&mut self, redo: bool) -> EditResult {
        self.goal_x = None;
        self.history.break_group();
        let preedit = self.preedit.take().is_some();
        if self.read_only || self.protected() {
            return EditResult {
                layout: preedit,
                reshape: preedit,
                ..Default::default()
            };
        }
        let transaction = if redo {
            self.history.redo.pop()
        } else {
            self.history.undo.pop_back()
        };
        let Some(transaction) = transaction else {
            return EditResult {
                layout: preedit,
                reshape: preedit,
                ..Default::default()
            };
        };
        if redo {
            for edit in &transaction {
                self.value
                    .replace_range(edit.start..edit.start + edit.removed.len(), &edit.inserted);
                edit.after.restore(self);
            }
            self.history.undo.push_back(transaction);
        } else {
            for edit in transaction.iter().rev() {
                self.value
                    .replace_range(edit.start..edit.start + edit.inserted.len(), &edit.removed);
                edit.before.restore(self);
            }
            self.history.redo.push(transaction);
        }
        self.reveal_cursor = true;
        EditResult {
            changed: true,
            layout: true,
            reshape: true,
            ..Default::default()
        }
    }
}
