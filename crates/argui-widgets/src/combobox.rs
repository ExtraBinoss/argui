use crate::{Button, Input, InputKind, SelectOption, WidgetTheme, choice_navigation::navigate};
use argui_core::{Key, KeyState};
use argui_ui::{
    AnchorWidth, DismissPolicy, Element, EventFilter, EventType, FloatingPlacement, FocusPolicy,
    Orientation, Placement, Role, Semantics, UiEvent, UiEventKind, ValueHandler, WindowLayer,
    length,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComboboxAction {
    Query(String),
    Open,
    Close,
    Highlight(usize),
    Select(usize),
}

/// Editable controlled combobox. Option indices refer to the unfiltered source collection.
/// The editor retains focus during arrow navigation; apply Highlight and rebuild.
#[derive(Clone, Debug)]
pub struct Combobox {
    pub key: String,
    pub label: String,
    pub options: Vec<SelectOption>,
    pub query: String,
    pub selected: Option<usize>,
    pub highlighted: Option<usize>,
    pub open: bool,
    pub enabled: bool,
    pub empty_label: String,
    input_handlers: Vec<ValueHandler<String>>,
    select_handlers: Vec<ValueHandler<usize>>,
    open_handlers: Vec<ValueHandler<bool>>,
}

impl Combobox {
    /// Creates an enabled combobox from options in source order.
    ///
    /// `key` identifies the control and `label` names it for accessibility.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        options: impl IntoIterator<Item = SelectOption>,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            options: options.into_iter().collect(),
            query: String::new(),
            selected: None,
            highlighted: None,
            open: false,
            enabled: true,
            empty_label: "No results".into(),
            input_handlers: Vec::new(),
            select_handlers: Vec::new(),
            open_handlers: Vec::new(),
        }
    }

    /// Adds a callback receiving the edited query.
    #[must_use]
    pub fn on_input(mut self, handler: ValueHandler<String>) -> Self {
        self.input_handlers.push(handler);
        self
    }

    /// Adds a callback receiving an enabled option's source index.
    #[must_use]
    pub fn on_select(mut self, handler: ValueHandler<usize>) -> Self {
        self.select_handlers.push(handler);
        self
    }

    /// Adds a callback receiving the requested popup open state.
    #[must_use]
    pub fn on_open_change(mut self, handler: ValueHandler<bool>) -> Self {
        self.open_handlers.push(handler);
        self
    }

    #[must_use]
    /// Returns source indices whose labels contain the current query, ignoring case.
    pub fn visible_indices(&self) -> Vec<usize> {
        let query = self.query.to_lowercase();
        self.options
            .iter()
            .enumerate()
            .filter(|(_, option)| option.label.to_lowercase().contains(&query))
            .map(|(index, _)| index)
            .collect()
    }

    fn active(&self, visible: &[usize]) -> Option<usize> {
        self.highlighted
            .filter(|index| visible.contains(index) && self.options[*index].enabled)
            .or_else(|| {
                visible
                    .iter()
                    .copied()
                    .find(|index| self.options[*index].enabled)
            })
    }

    #[must_use]
    /// Returns the interaction key for the source option at `index`.
    pub fn option_key(&self, index: usize) -> String {
        format!("{}::option::{index}", self.key)
    }

    #[must_use]
    /// Interprets `event` as a query, popup, highlight or selection action.
    pub fn action(&self, event: &UiEvent) -> Option<ComboboxAction> {
        if !self.enabled {
            return None;
        }
        let key = event.target_key()?;
        let visible = self.visible_indices();
        if self.open
            && key == format!("{}::list", self.key)
            && matches!(
                event.kind,
                UiEventKind::PointerOutside(_) | UiEventKind::DismissRequested
            )
        {
            return Some(ComboboxAction::Close);
        }
        if self.open
            && matches!(event.kind, UiEventKind::Click(_))
            && let Some(index) = visible
                .iter()
                .copied()
                .find(|index| key == self.option_key(*index) && self.options[*index].enabled)
        {
            return Some(ComboboxAction::Select(index));
        }
        if key != self.key {
            return None;
        }
        match &event.kind {
            UiEventKind::TextChanged(value) => Some(ComboboxAction::Query(value.clone())),
            UiEventKind::Click(_) => Some(ComboboxAction::Open),
            UiEventKind::KeyInput(input) if input.state == KeyState::Pressed => {
                if input.key == Key::Escape && self.open {
                    return Some(ComboboxAction::Close);
                }
                if input.key == Key::Tab && self.open {
                    return Some(ComboboxAction::Close);
                }
                if matches!(input.key, Key::ArrowDown | Key::ArrowUp) {
                    if !self.open {
                        return Some(ComboboxAction::Open);
                    }
                    let current = self
                        .active(&visible)
                        .and_then(|index| visible.iter().position(|item| *item == index))
                        .unwrap_or(0);
                    return navigate(
                        event,
                        current,
                        visible.len(),
                        Orientation::Vertical,
                        false,
                        |index| self.options[visible[index]].enabled,
                    )
                    .map(|index| ComboboxAction::Highlight(visible[index]));
                }
                if input.key == Key::Enter && self.open {
                    return self.active(&visible).map(ComboboxAction::Select);
                }
                None
            }
            _ => None,
        }
    }

    #[must_use]
    /// Builds the editable combobox and its popup using `theme` for control styling.
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        let open = self.open && self.enabled;
        let visible = self.visible_indices();
        let active = self.active(&visible);
        let list_key = format!("{}::list", self.key);
        let mut input_builder = Input::new(&self.key, &self.query, &self.label, theme.input())
            .kind(InputKind::Search)
            .label(&self.label)
            .enabled(self.enabled);
        for handler in &self.input_handlers {
            input_builder = input_builder.on_input(*handler);
        }
        let mut input = input_builder.build();
        if self.enabled {
            for handler in &self.open_handlers {
                input = input
                    .on(handler.direct_listener_value(EventType::Click, true))
                    .on(handler
                        .direct_listener_value(EventType::Key, true)
                        .filter(EventFilter::VerticalArrowPressed));
                if open {
                    input = input
                        .on(handler
                            .direct_listener_value(EventType::Key, false)
                            .filter(EventFilter::EscapePressed))
                        .on(handler
                            .direct_listener_value(EventType::Key, false)
                            .filter(EventFilter::TabPressed));
                }
            }
            if open && let Some(index) = active {
                for handler in &self.select_handlers {
                    input = input.on(handler
                        .direct_listener_value(EventType::Key, index)
                        .filter(EventFilter::EnterPressed));
                }
            }
        }
        let semantics = input.semantics.as_mut().expect("input semantics");
        semantics.role = Role::ComboBox;
        semantics.popup = Some(argui_ui::PopupKind::ListBox);
        semantics.state.expanded = Some(open);
        if open {
            input = input.controls([list_key.clone()]);
            if let Some(index) = active {
                input = input.active_descendant(self.option_key(index));
            }
        }
        let panel = open.then(|| {
            let mut rows: Vec<_> = visible
                .iter()
                .map(|index| {
                    let option = &self.options[*index];
                    let mut row = Button::new(
                        self.option_key(*index),
                        &option.label,
                        if active == Some(*index) {
                            theme.secondary_button()
                        } else {
                            theme.ghost_button()
                        },
                    )
                    .enabled(option.enabled)
                    .build();
                    row.interaction
                        .as_mut()
                        .expect("option interaction")
                        .focus_policy = FocusPolicy::None;
                    let semantics = row.semantics.as_mut().expect("option semantics");
                    semantics.role = Role::Option;
                    semantics.state.selected = self.selected == Some(*index);
                    for handler in &self.select_handlers {
                        row = row.on(handler.direct_listener_value(EventType::Click, *index));
                    }
                    row
                })
                .collect();
            if rows.is_empty() {
                rows.push(Element::text(self.empty_label.clone()).text_style(
                    argui_text::TextStyle {
                        color: theme.muted_foreground,
                        ..Default::default()
                    },
                ));
            }
            let mut panel = crate::ScrollArea::new(
                &list_key,
                &self.label,
                (rows.len() as f32 * 38.0 + 12.0).min(280.0),
                Element::column(rows).gap(2.0),
            )
            .build(theme)
            .width(length(280.0))
            .padding(argui_ui::Sides::length(6.0))
            .background(theme.popover)
            .border(argui_paint::Border::all(1.0, theme.popover_border))
            .radius(argui_paint::CornerRadii::all(8.0))
            .layer(theme.overlay_layer(8.0, theme.overlay_blur))
            .anchored_portal(
                WindowLayer::Popover,
                &self.key,
                FloatingPlacement::new(Placement::BottomStart)
                    .anchor_width(AnchorWidth::AtLeastAnchor),
            )
            .portal_dismiss(DismissPolicy::OutsidePointer)
            .semantics(Semantics::new(Role::ListBox).label(&self.label));
            panel
                .interaction
                .as_mut()
                .expect("scroll area interaction")
                .focus_policy = FocusPolicy::None;
            for handler in &self.open_handlers {
                panel = panel
                    .on(handler.direct_listener_value(EventType::PointerOutside, false))
                    .on(handler.direct_listener_value(EventType::Dismiss, false));
            }
            panel
        });
        Element::column(std::iter::once(input).chain(panel))
    }
}
