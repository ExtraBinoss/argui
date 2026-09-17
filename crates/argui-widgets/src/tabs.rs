use argui_paint::{Border, CornerRadii, PaintStyle, QuadStyle};
use argui_text::{TextStyle, TextWrap};
use argui_ui::{
    AlignItems, Display, Element, EventType, JustifyContent, StyleTransition, ValueHandler,
    VisualState, auto,
};

use crate::{TabsBehavior, TabsPart, WidgetTheme};

#[derive(Clone, Debug)]
pub struct Tab {
    pub label: String,
    pub panel: Element,
    pub enabled: bool,
}

impl Tab {
    /// Creates a tab with a trigger `label` and its panel element.
    #[must_use]
    pub fn new(label: impl Into<String>, panel: Element) -> Self {
        Self {
            label: label.into(),
            panel,
            enabled: true,
        }
    }

    #[must_use]
    /// Sets whether this tab can be selected.
    /// `enabled` controls whether the tab is interactive.
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
    select_handlers: Vec<ValueHandler<usize>>,
}

impl Tabs {
    /// Creates a controlled tab set with tabs in display order and selected source index.
    /// `key` identifies the set and `label` names it accessibly.
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
            select_handlers: Vec::new(),
        }
    }

    /// Adds a callback receiving the selected source tab index.
    ///
    /// `handler` is normally created with `Context::value_callback`. Disabled
    /// tabs never deliver it.
    #[must_use]
    pub fn on_select(mut self, handler: ValueHandler<usize>) -> Self {
        self.select_handlers.push(handler);
        self
    }

    #[must_use]
    /// Builds the tab triggers and selected panel using `theme` for styling.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let selected = self.selected.min(self.tabs.len().saturating_sub(1));
        let behavior = TabsBehavior::new(
            &self.key,
            self.tabs.iter().map(|tab| (tab.label.clone(), tab.enabled)),
            selected,
        );
        let triggers = self.tabs.iter().enumerate().map(|(index, tab)| {
            let active = index == selected;
            let resting = QuadStyle::solid(if active {
                theme.card
            } else {
                argui_core::Color::TRANSPARENT
            })
            .radius(CornerRadii::all(6.0));
            let label = behavior.decorate(
                TabsPart::Label,
                Element::text(tab.label.clone()).text_style(TextStyle {
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
                }),
            );
            let mut trigger = behavior.decorate(
                TabsPart::Trigger(index),
                Element::container([label])
                    .padding(argui_ui::sides(12.0, 7.0))
                    .display(Display::Flex)
                    .align_items(AlignItems::CENTER)
                    .justify_content(JustifyContent::CENTER)
                    .paint_style(PaintStyle::new(resting.clone()))
                    .when(
                        VisualState::Hovered,
                        QuadStyle::solid(if active { theme.card } else { theme.background })
                            .radius(CornerRadii::all(6.0))
                            .into(),
                    )
                    .when(
                        VisualState::FocusVisible,
                        resting.border(Border::all(2.0, theme.ring)).into(),
                    )
                    .transition(crate::theme::instant_hover(
                        StyleTransition::default(),
                        VisualState::Hovered.into(),
                    )),
            );
            for handler in self.select_handlers.iter().copied() {
                trigger = trigger.on(handler.direct_listener_value(EventType::Click, index));
            }
            trigger
        });
        let tab_list = behavior.decorate(
            TabsPart::List,
            Element::row(triggers)
                .padding(argui_ui::Sides::length(4.0))
                .gap(3.0)
                .width(auto())
                .background(theme.muted)
                .border(Border::all(1.0, theme.border))
                .radius(CornerRadii::all(8.0)),
        );
        let panel = self
            .tabs
            .into_iter()
            .nth(selected)
            .map(|tab| behavior.decorate(TabsPart::Panel(selected), tab.panel))
            .unwrap_or_else(|| {
                behavior.decorate(TabsPart::Panel(selected), Element::container([]))
            });
        behavior.decorate(TabsPart::Root, Element::column([tab_list, panel]).gap(14.0))
    }
}
