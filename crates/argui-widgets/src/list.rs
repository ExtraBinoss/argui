use argui_core::{Key, KeyState};
use argui_paint::{Border, QuadStyle};
use argui_ui::{
    Element, GestureSet, Interaction, Role, SemanticAction, SemanticState, Semantics, TapGesture,
    UiEvent, UiEventKind, UserSelect, VisualState,
};

use crate::{Collection, ListState, WidgetTheme};

/// Decoration and event interpretation independent from row contents and rendering.
#[derive(Clone, Debug)]
pub struct List<'a> {
    key: String,
    label: String,
    collection: &'a Collection,
    state: Option<&'a ListState>,
    multiple: bool,
    page_size: Option<usize>,
}

impl<'a> List<'a> {
    /// Creates list behavior for the immutable `collection` snapshot, identified by `key`.
    #[must_use]
    pub fn new(key: impl Into<String>, collection: &'a Collection) -> Self {
        let key = key.into();
        Self {
            label: key.clone(),
            key,
            collection,
            state: None,
            multiple: false,
            page_size: None,
        }
    }

    #[must_use]
    /// Sets the accessible name of the list; `label` is announced to assistive technology.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    #[must_use]
    /// Uses controlled `state`, with `multiple` enabling additive and range selection.
    pub fn selection(mut self, state: &'a ListState, multiple: bool) -> Self {
        self.state = Some(state);
        self.multiple = multiple;
        self
    }

    /// Set to the number of rows in the viewport when interpreting PageUp/PageDown.
    #[must_use]
    pub fn page_size(mut self, rows: usize) -> Self {
        self.page_size = Some(rows.max(1));
        self
    }

    /// Interpret printable input using an explicitly retained search buffer and clock.
    /// Searches enabled items using a retained typeahead buffer and returns updated selection.
    ///
    /// `input` is the typed text, `now` is its monotonic timestamp, and `matches` compares
    /// an item label with the accumulated query.
    pub fn search(
        &self,
        search: &mut crate::Typeahead,
        input: &str,
        now: std::time::Duration,
        matches: impl Fn(&str, &str) -> bool,
    ) -> Option<ListState> {
        let mut state = self.state.cloned().unwrap_or_default();
        let active = state
            .active
            .as_deref()
            .and_then(|id| self.collection.index_of(id));
        let index = search.search(
            input,
            now,
            active,
            self.collection.len(),
            |index| {
                self.collection
                    .get(index)
                    .filter(|item| item.enabled)
                    .map(|item| item.label.as_str())
            },
            matches,
        )?;
        state.select(
            index,
            self.collection,
            self.multiple,
            argui_core::Modifiers::default(),
        );
        Some(state)
    }

    #[must_use]
    /// Returns the stable element key for the row at `index`.
    ///
    /// # Panics
    ///
    /// Panics if `index` is outside the collection. `element` is row content and `theme` supplies its style.
    pub fn row_key(&self, index: usize) -> String {
        format!(
            "{}::row::{}",
            self.key,
            self.collection.get(index).expect("row index").id
        )
    }

    #[must_use]
    /// Decorates `element` with the list's group semantics and interaction scope.
    pub fn root(&self, element: Element) -> Element {
        let mut root = element
            .semantic_scope()
            .keyed(&self.key)
            .interaction(Interaction::default().focus_policy(argui_ui::FocusPolicy::TabStop))
            .semantics(
                Semantics::new(Role::ListBox)
                    .label(&self.label)
                    .state(SemanticState {
                        multiselectable: self.multiple,
                        ..SemanticState::default()
                    }),
            );
        if let Some(index) = self
            .state
            .and_then(|state| state.active.as_deref())
            .and_then(|id| self.collection.index_of(id))
        {
            root = root.active_descendant(self.row_key(index));
        }
        root
    }

    #[must_use]
    /// Decorates a row with selection, focus, identity and themed interaction styling.
    ///
    /// # Panics
    ///
    /// Panics if `index` is outside the collection.
    /// `element` is the row content and `theme` supplies the row interaction styling.
    pub fn row(&self, index: usize, element: Element, theme: &WidgetTheme) -> Element {
        let item = self.collection.get(index).expect("row index");
        let selected = self
            .state
            .is_some_and(|state| state.selected.contains(&item.id));
        let active = self
            .state
            .is_some_and(|state| state.active.as_ref() == Some(&item.id));
        let mut semantics = content_semantics(Role::Option, &element);
        if active {
            semantics = semantics.action(SemanticAction::Focus);
        }
        element
            .keyed(self.row_key(index))
            .user_select(UserSelect::None)
            .interaction(
                Interaction::default()
                    .enabled(item.enabled)
                    .focus_policy(argui_ui::FocusPolicy::None)
                    .gestures(GestureSet::default().tap(TapGesture::default())),
            )
            .semantics(
                semantics
                    .state(SemanticState {
                        selected,
                        disabled: !item.enabled,
                        ..SemanticState::default()
                    })
                    .position_in_set((index + 1) as u32, self.collection.len() as u32)
                    .action(SemanticAction::Click),
            )
            .background(if selected {
                theme.muted
            } else {
                theme.background
            })
            .border(Border::all(
                1.0,
                if active { theme.ring } else { theme.border },
            ))
            .when(VisualState::Hovered, QuadStyle::solid(theme.muted).into())
    }

    #[must_use]
    /// Builds rows by calling `row` for each index; `theme` supplies list styling.
    pub fn build(&self, theme: &WidgetTheme, mut row: impl FnMut(usize) -> Element) -> Element {
        self.root(Element::column(
            (0..self.collection.len()).map(|index| self.row(index, row(index), theme)),
        ))
    }

    /// Returns the next controlled state for a click or navigation key.
    /// The consumer scrolls `active` into view for virtual lists.
    #[must_use]
    /// Returns updated controlled selection or active-row state for `event`, if applicable.
    pub fn action(&self, event: &UiEvent) -> Option<ListState> {
        let key = event.target_key()?;
        let index = key
            .strip_prefix(&format!("{}::row::", self.key))
            .and_then(|value| self.collection.index_of(value))
            .filter(|index| *index < self.collection.len());
        if key != self.key && index.is_none() {
            return None;
        }
        if self.collection.is_empty() {
            return None;
        }
        let mut state = self.state.cloned().unwrap_or_default();
        match &event.kind {
            UiEventKind::Click(click) => {
                state.select(index?, self.collection, self.multiple, click.modifiers())
            }
            UiEventKind::KeyInput(input) if input.state == KeyState::Pressed => {
                let active = state
                    .active
                    .as_deref()
                    .and_then(|id| self.collection.index_of(id));
                let target = match &input.key {
                    Key::ArrowDown => self
                        .collection
                        .enabled_from(active.map_or(0, |i| i + 1), false)
                        .or(active)?,
                    Key::ArrowUp => self
                        .collection
                        .enabled_from(active.unwrap_or(0).saturating_sub(1), true)
                        .or(active)?,
                    Key::PageDown => {
                        let target = active
                            .unwrap_or(0)
                            .saturating_add(self.page_size?)
                            .min(self.collection.len() - 1);
                        self.collection
                            .enabled_from(target, false)
                            .or_else(|| self.collection.enabled_from(target, true))?
                    }
                    Key::PageUp => {
                        let target = active.unwrap_or(0).saturating_sub(self.page_size?);
                        self.collection
                            .enabled_from(target, true)
                            .or_else(|| self.collection.enabled_from(target, false))?
                    }
                    Key::Home => self.collection.enabled_from(0, false)?,
                    Key::End => self
                        .collection
                        .enabled_from(self.collection.len() - 1, true)?,
                    Key::Character(value)
                        if value.eq_ignore_ascii_case("a")
                            && input.modifiers.command()
                            && self.multiple =>
                    {
                        state.selected = self
                            .collection
                            .items()
                            .iter()
                            .filter(|item| item.enabled)
                            .map(|item| item.id.clone())
                            .collect();
                        return Some(state);
                    }
                    Key::Enter => {
                        state.select(
                            active.or_else(|| self.collection.enabled_from(0, false))?,
                            self.collection,
                            self.multiple,
                            input.modifiers,
                        );
                        return Some(state);
                    }
                    Key::Character(value) if value == " " => {
                        state.select(
                            active.or_else(|| self.collection.enabled_from(0, false))?,
                            self.collection,
                            self.multiple,
                            input.modifiers,
                        );
                        return Some(state);
                    }
                    _ => return None,
                };
                if input.modifiers.command() && !input.modifiers.shift {
                    state.active = Some(self.collection.get(target)?.id.clone());
                } else {
                    state.select(target, self.collection, self.multiple, input.modifiers);
                }
            }
            _ => return None,
        }
        Some(state)
    }
}

// Preserve labels supplied by consumers and the accessible name of a plain text row.
pub(crate) fn content_semantics(role: Role, element: &Element) -> Semantics {
    let mut semantics = element.semantics.as_deref().cloned().unwrap_or_default();
    semantics.role = role;
    if semantics.label.is_none()
        && let argui_ui::ElementKind::Text { content, .. } = &element.kind
    {
        semantics.label = Some(content.as_str().to_owned());
    }
    semantics
}
