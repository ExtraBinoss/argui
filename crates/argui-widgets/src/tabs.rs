use argui_paint::{Border, CornerRadii, PaintStyle, QuadStyle};
use argui_text::{TextStyle, TextWrap};
use argui_ui::{
    AlignItems, CursorIcon, Display, Element, GestureSet, Interaction, JustifyContent,
    KeyboardActivation, Orientation, Role, SemanticAction, SemanticState, Semantics,
    StyleTransition, UiEvent, UiEventKind, VisualState, auto,
};

use crate::WidgetTheme;

#[derive(Clone, Debug)]
pub struct Tab {
    pub label: String,
    pub panel: Element,
    pub enabled: bool,
}

impl Tab {
    #[must_use]
    pub fn new(label: impl Into<String>, panel: Element) -> Self {
        Self {
            label: label.into(),
            panel,
            enabled: true,
        }
    }

    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

#[derive(Clone, Debug)]
pub struct Tabs {
    key: String,
    tabs: Vec<Tab>,
    selected: usize,
}

impl Tabs {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        tabs: impl IntoIterator<Item = Tab>,
        selected: usize,
    ) -> Self {
        Self {
            key: key.into(),
            tabs: tabs.into_iter().collect(),
            selected,
        }
    }

    #[must_use]
    pub fn tab_key(key: &str, index: usize) -> String {
        format!("{key}::tab::{index}")
    }

    #[must_use]
    pub fn selection(key: &str, event: &UiEvent) -> Option<usize> {
        if !matches!(event.kind, UiEventKind::Clicked) {
            return None;
        }
        event
            .key
            .as_deref()?
            .strip_prefix(&format!("{key}::tab::"))?
            .parse()
            .ok()
    }

    #[must_use]
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let selected = self.selected.min(self.tabs.len().saturating_sub(1));
        let triggers = self.tabs.iter().enumerate().map(|(index, tab)| {
            let active = index == selected;
            let state = SemanticState {
                selected: active,
                disabled: !tab.enabled,
                ..SemanticState::default()
            };
            let resting = QuadStyle::solid(if active {
                theme.card
            } else {
                argui_core::Color::TRANSPARENT
            })
            .radius(CornerRadii::all(6.0));
            Element::container([Element::text(tab.label.clone())
                .text_style(TextStyle {
                    font_size: 14.0,
                    line_height: 20.0,
                    color: if active {
                        theme.foreground
                    } else {
                        theme.muted_foreground
                    },
                    weight: if active { 600 } else { 500 },
                    wrap: TextWrap::None,
                    ..TextStyle::default()
                })
                .semantic_hidden(true)])
            .keyed(Self::tab_key(&self.key, index))
            .padding(argui_ui::sides(12.0, 7.0))
            .display(Display::Flex)
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::CENTER)
            .paint_style(PaintStyle::new(resting.clone()))
            .interaction(
                Interaction::default()
                    .enabled(tab.enabled)
                    .focusable(tab.enabled)
                    .cursor(CursorIcon::Pointer)
                    .gestures(GestureSet::NONE.tap())
                    .keyboard_activation(KeyboardActivation::EnterOrSpace),
            )
            .state(
                VisualState::Hovered,
                QuadStyle::solid(theme.muted)
                    .radius(CornerRadii::all(6.0))
                    .into(),
            )
            .state(
                VisualState::Focused,
                resting.border(Border::all(2.0, theme.ring)).into(),
            )
            .transition(StyleTransition::default())
            .semantics(
                Semantics::new(Role::Tab)
                    .label(tab.label.clone())
                    .state(state)
                    .action(SemanticAction::Click)
                    .action(SemanticAction::Focus),
            )
        });
        let tab_list = Element::row(triggers)
            .padding(argui_ui::Sides::length(4.0))
            .gap(3.0)
            .width(auto())
            .background(theme.muted)
            .border(Border::all(1.0, theme.border))
            .radius(CornerRadii::all(8.0))
            .semantics(Semantics::new(Role::TabList).orientation(Orientation::Horizontal));
        let panel = self
            .tabs
            .into_iter()
            .nth(selected)
            .map(|tab| {
                tab.panel
                    .keyed(format!("{}::panel::{selected}", self.key))
                    .semantics(Semantics::new(Role::TabPanel).label(tab.label))
            })
            .unwrap_or_else(|| Element::container([]).semantics(Semantics::new(Role::TabPanel)));
        Element::column([tab_list, panel]).gap(14.0)
    }
}
