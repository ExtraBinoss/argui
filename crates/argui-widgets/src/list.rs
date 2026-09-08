use std::collections::BTreeSet;

use argui_core::{Key, KeyState, Modifiers};
use argui_paint::{Border, QuadStyle};
use argui_ui::{
    Element, GestureSet, Interaction, Role, SemanticAction, SemanticState, Semantics, TapGesture,
    UiEvent, UiEventKind, UserSelect, VisualState,
};

use crate::WidgetTheme;

/// Controlled selection shared by lists and tables. Indices refer to the current order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ListState {
    pub selected: BTreeSet<usize>,
    pub active: Option<usize>,
    anchor: Option<usize>,
}

impl ListState {
    /// Remap selection when rows are inserted before `index`.
    pub fn insert(&mut self, index: usize, count: usize) {
        let shift = |value: usize| {
            if value >= index {
                value.saturating_add(count)
            } else {
                value
            }
        };
        self.selected = self.selected.iter().copied().map(shift).collect();
        self.active = self.active.map(shift);
        self.anchor = self.anchor.map(shift);
    }

    /// Remap selection after removal. A removed active row moves to its next neighbour.
    pub fn remove(&mut self, range: std::ops::Range<usize>, remaining: usize) {
        let shift = |value: usize| {
            if range.contains(&value) {
                None
            } else {
                Some(if value >= range.end {
                    value.saturating_sub(range.len())
                } else {
                    value
                })
            }
        };
        self.selected = self
            .selected
            .iter()
            .copied()
            .filter_map(shift)
            .filter(|index| *index < remaining)
            .collect();
        self.active = self
            .active
            .and_then(|value| shift(value).or(Some(range.start)))
            .filter(|_| remaining > 0)
            .map(|value| value.min(remaining - 1));
        self.anchor = self
            .anchor
            .and_then(shift)
            .filter(|index| *index < remaining);
    }

    pub fn select(&mut self, index: usize, count: usize, multiple: bool, modifiers: Modifiers) {
        if index >= count {
            return;
        }
        self.selected.retain(|index| *index < count);
        if multiple && modifiers.shift {
            let anchor = self
                .anchor
                .unwrap_or(self.active.unwrap_or(index))
                .min(count - 1);
            if !modifiers.command() {
                self.selected.clear();
            }
            self.selected.extend(anchor.min(index)..=anchor.max(index));
            self.anchor = Some(anchor);
        } else {
            if !multiple || !modifiers.command() {
                self.selected.clear();
            }
            if !self.selected.insert(index) {
                self.selected.remove(&index);
            }
            self.anchor = Some(index);
        }
        self.active = Some(index);
    }
}

/// Decoration and event interpretation independent from row contents and rendering.
#[derive(Clone, Debug)]
pub struct List {
    key: String,
    label: String,
    count: usize,
    state: ListState,
    multiple: bool,
}

impl List {
    #[must_use]
    pub fn new(key: impl Into<String>, count: usize) -> Self {
        let key = key.into();
        Self {
            label: key.clone(),
            key,
            count,
            state: ListState::default(),
            multiple: false,
        }
    }

    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    #[must_use]
    pub fn selection(mut self, state: &ListState, multiple: bool) -> Self {
        self.state = state.clone();
        self.multiple = multiple;
        self
    }

    #[must_use]
    pub fn row_key(&self, index: usize) -> String {
        format!("{}::row::{index}", self.key)
    }

    #[must_use]
    pub fn root(&self, element: Element) -> Element {
        element
            .keyed(&self.key)
            .interaction(Interaction::default().focusable(true))
            .semantics(
                Semantics::new(Role::ListBox)
                    .label(&self.label)
                    .state(SemanticState {
                        multiselectable: self.multiple,
                        ..SemanticState::default()
                    }),
            )
    }

    #[must_use]
    pub fn row(&self, index: usize, element: Element, theme: &WidgetTheme) -> Element {
        let selected = self.state.selected.contains(&index);
        let mut semantics = content_semantics(Role::Option, &element);
        if self.state.active == Some(index) {
            semantics = semantics.action(SemanticAction::Focus);
        }
        element
            .keyed(self.row_key(index))
            .user_select(UserSelect::None)
            .interaction(
                Interaction::default()
                    .focusable(self.state.active == Some(index))
                    .gestures(GestureSet::default().tap(TapGesture::default())),
            )
            .semantics(
                semantics
                    .state(SemanticState {
                        selected,
                        ..SemanticState::default()
                    })
                    .position_in_set((index + 1) as u32, self.count as u32)
                    .action(SemanticAction::Click),
            )
            .background(if selected {
                theme.muted
            } else {
                theme.background
            })
            .border(Border::all(
                1.0,
                if self.state.active == Some(index) {
                    theme.ring
                } else {
                    theme.border
                },
            ))
            .when(VisualState::Hovered, QuadStyle::solid(theme.muted).into())
    }

    #[must_use]
    pub fn build(&self, theme: &WidgetTheme, mut row: impl FnMut(usize) -> Element) -> Element {
        self.root(Element::column(
            (0..self.count).map(|index| self.row(index, row(index), theme)),
        ))
    }

    /// Returns the next controlled state for a click or navigation key.
    /// The consumer scrolls `active` into view for virtual lists.
    #[must_use]
    pub fn action(&self, event: &UiEvent) -> Option<ListState> {
        let key = event.target_key()?;
        let index = key
            .strip_prefix(&format!("{}::row::", self.key))
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|index| *index < self.count);
        if key != self.key && index.is_none() {
            return None;
        }
        if self.count == 0 {
            return None;
        }
        let mut state = self.state.clone();
        match &event.kind {
            UiEventKind::Click(click) => {
                state.select(index?, self.count, self.multiple, click.modifiers())
            }
            UiEventKind::KeyInput(input) if input.state == KeyState::Pressed => {
                let active = state.active.unwrap_or(0).min(self.count - 1);
                let target = match &input.key {
                    Key::ArrowDown => state.active.map_or(0, |_| (active + 1).min(self.count - 1)),
                    Key::ArrowUp => active.saturating_sub(1),
                    Key::Home => 0,
                    Key::End => self.count - 1,
                    Key::Character(value)
                        if value.eq_ignore_ascii_case("a")
                            && input.modifiers.command()
                            && self.multiple =>
                    {
                        state.selected = (0..self.count).collect();
                        return Some(state);
                    }
                    Key::Enter => {
                        state.select(active, self.count, self.multiple, input.modifiers);
                        return Some(state);
                    }
                    Key::Character(value) if value == " " => {
                        state.select(active, self.count, self.multiple, input.modifiers);
                        return Some(state);
                    }
                    _ => return None,
                };
                if input.modifiers.command() && !input.modifiers.shift {
                    state.active = Some(target);
                } else {
                    state.select(target, self.count, self.multiple, input.modifiers);
                }
            }
            _ => return None,
        }
        Some(state)
    }
}

// Preserve labels supplied by consumers and the accessible name of a plain text row.
pub(crate) fn content_semantics(role: Role, element: &Element) -> Semantics {
    let mut semantics = element.semantics.clone().unwrap_or_default();
    semantics.role = role;
    if semantics.label.is_none()
        && let argui_ui::ElementKind::Text { content, .. } = &element.kind
    {
        semantics.label = Some(content.as_str().to_owned());
    }
    semantics
}
