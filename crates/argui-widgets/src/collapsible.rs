use argui_ui::{
    Element, EventType, JustifyContent, Role, Semantics, UiEvent, ValueHandler, length, percent,
};

use crate::{Button, ButtonBehavior, WidgetTheme};

/// Controlled disclosure. Closed content is unmounted, including its focus targets.
/// Apply the boolean returned by `action` to application state, then rebuild.
#[derive(Clone, Debug)]
pub struct Collapsible {
    key: String,
    label: String,
    open: bool,
    enabled: bool,
    trigger: Option<Element>,
    indicator: Option<Element>,
    content: Element,
    open_handlers: Vec<ValueHandler<bool>>,
}

impl Collapsible {
    /// Creates a controlled disclosure with a trigger label and panel content.
    ///
    /// `key` identifies the disclosure, `open` supplies its current state, and `content`
    /// is mounted only while it is open.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        open: bool,
        content: Element,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            open,
            enabled: true,
            trigger: None,
            indicator: None,
            content,
            open_handlers: Vec::new(),
        }
    }

    #[must_use]
    /// Sets whether the disclosure trigger can be activated.
    /// `enabled` determines whether the trigger accepts activation.
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Replaces the visible trigger label. Supply non-interactive content.
    #[must_use]
    pub fn trigger(mut self, content: Element) -> Self {
        self.trigger = Some(content);
        self
    }

    #[must_use]
    /// Adds a decorative indicator to the trigger.
    pub fn indicator(mut self, indicator: Element) -> Self {
        self.indicator = Some(indicator);
        self
    }

    #[must_use]
    /// Returns the state key used by the disclosure trigger.
    pub fn trigger_key(&self) -> String {
        format!("{}::trigger", self.key)
    }

    #[must_use]
    /// Returns the state key used by the disclosure panel.
    pub fn content_key(&self) -> String {
        format!("{}::content", self.key)
    }

    /// Pointer, keyboard and accessibility activation share the button event path.
    #[must_use]
    pub fn action(&self, event: &UiEvent) -> Option<bool> {
        ButtonBehavior::new(self.trigger_key(), &self.label)
            .enabled(self.enabled)
            .action(event)
            .map(|_| !self.open)
    }

    /// Adds a callback receiving the requested controlled open state.
    #[must_use]
    pub fn on_open_change(mut self, handler: ValueHandler<bool>) -> Self {
        self.open_handlers.push(handler);
        self
    }

    #[must_use]
    /// Builds the disclosure using `theme` for trigger styling.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let trigger_key = self.trigger_key();
        let content_key = self.content_key();
        let mut style = theme.outline_button().focused(
            argui_paint::QuadStyle::solid(theme.card)
                .border(argui_paint::Border::all(2.0, theme.ring))
                .radius(argui_paint::CornerRadii::all(7.0)),
        );
        style.label.wrap = argui_text::TextWrap::WordOrGlyph;
        let content = self
            .trigger
            .unwrap_or_else(|| Element::text(self.label.clone()).text_style(style.label.clone()));
        let mut button = Button::new(&trigger_key, self.label, style)
            .enabled(self.enabled)
            .content(Element::column([content]).grow(1.0).min_width(length(0.0)));
        if let Some(indicator) = self.indicator {
            button = button.trailing(indicator.semantic_hidden(true));
        }
        let mut trigger = button
            .build()
            .width(percent(1.0))
            .height(argui_ui::auto())
            .min_height(length(36.0))
            .padding(argui_ui::sides(12.0, 8.0))
            .justify_content(JustifyContent::SPACE_BETWEEN)
            .paint_opacity(if self.enabled { 1.0 } else { 0.5 });
        if let Some(semantics) = &mut trigger.semantics {
            semantics.state.expanded = Some(self.open);
        }
        if self.open {
            trigger = trigger.controls([content_key.clone()]);
        }
        if self.enabled {
            for handler in self.open_handlers {
                trigger = trigger.on(handler.direct_listener_value(EventType::Click, !self.open));
            }
        }
        let mut children = vec![trigger];
        if self.open {
            children.push(
                Element::column([self.content])
                    .keyed(content_key)
                    .semantics(Semantics::new(Role::Group))
                    .labelled_by([trigger_key])
                    .min_width(length(0.0)),
            );
        }
        Element::column(children)
            .keyed(self.key)
            .width(percent(1.0))
            .min_width(length(0.0))
            .gap(8.0)
    }
}
