use crate::{Button, ButtonBehavior, WidgetTheme};
use argui_core::{Key, KeyState};
use argui_ui::{
    Element, EventFilter, EventType, FocusPolicy, GestureKind, GesturePhase, GestureSet,
    Interaction, LiveRegion, PanAxis, PanGesture, Role, Semantics, UiEvent, UiEventKind,
    ValueHandler,
};

/// Controlled, non-autoplaying carousel. Only the active slide contributes focus targets.
#[derive(Clone, Debug)]
pub struct Carousel {
    pub key: String,
    pub label: String,
    pub slides: Vec<Element>,
    pub selected: usize,
    pub looping: bool,
    pub rtl: bool,
    pub previous_label: String,
    pub next_label: String,
    select_handlers: Vec<ValueHandler<usize>>,
}

impl Carousel {
    /// Creates a controlled carousel with slides in display order and a selected index.
    ///
    /// `key` identifies the carousel and `label` names it to assistive technology.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        slides: impl IntoIterator<Item = Element>,
        selected: usize,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            slides: slides.into_iter().collect(),
            selected,
            looping: false,
            rtl: false,
            previous_label: "Previous slide".into(),
            next_label: "Next slide".into(),
            select_handlers: Vec::new(),
        }
    }

    /// Adds a callback receiving the requested zero-based slide index.
    #[must_use]
    pub fn on_select(mut self, handler: ValueHandler<usize>) -> Self {
        self.select_handlers.push(handler);
        self
    }

    fn next(&self, forward: bool) -> Option<usize> {
        let count = self.slides.len();
        if count < 2 {
            return None;
        }
        let current = self.selected.min(count - 1);
        if forward && current + 1 < count {
            Some(current + 1)
        } else if !forward && current > 0 {
            Some(current - 1)
        } else if self.looping {
            Some(if forward { 0 } else { count - 1 })
        } else {
            None
        }
    }

    #[must_use]
    /// Returns the new selected slide index when `event` requests navigation.
    pub fn action(&self, event: &UiEvent) -> Option<usize> {
        for (part, forward) in [("previous", false), ("next", true)] {
            if ButtonBehavior::new(format!("{}::{part}", self.key), part)
                .action(event)
                .is_some()
            {
                return self.next(forward);
            }
        }
        if event.target_key() != Some(format!("{}::viewport", self.key).as_str()) {
            return None;
        }
        match &event.kind {
            UiEventKind::KeyInput(input)
                if input.state == KeyState::Pressed
                    && !input.modifiers.command()
                    && !input.modifiers.alt =>
            {
                match input.key {
                    Key::ArrowLeft => self.next(self.rtl),
                    Key::ArrowRight => self.next(!self.rtl),
                    Key::Home => (!self.slides.is_empty()).then_some(0),
                    Key::End => self.slides.len().checked_sub(1),
                    _ => None,
                }
            }
            UiEventKind::Gesture(gesture) if gesture.phase == GesturePhase::Ended => {
                if let GestureKind::Pan { total, .. } = gesture.kind
                    && total.x.abs() >= 40.0
                {
                    self.next((total.x < 0.0) != self.rtl)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    #[must_use]
    /// Builds the carousel and its controls using `theme` for button styling.
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        let selected = self.selected.min(self.slides.len().saturating_sub(1));
        let slide = self.slides.get(selected).cloned();
        let position = if self.slides.is_empty() {
            "0 / 0".into()
        } else {
            format!("{} / {}", selected + 1, self.slides.len())
        };
        let mut viewport = Element::column(slide)
            .keyed(format!("{}::viewport", self.key))
            .interaction(
                Interaction::default()
                    .focus_policy(FocusPolicy::TabStop)
                    .gestures(
                        GestureSet::default().pan(PanGesture::default().axis(PanAxis::Horizontal)),
                    ),
            )
            .semantics(Semantics::new(Role::Group).label(&self.label));
        for (filter, value) in [
            (
                EventFilter::HomePressed,
                (!self.slides.is_empty()).then_some(0),
            ),
            (EventFilter::EndPressed, self.slides.len().checked_sub(1)),
            (EventFilter::ArrowLeftPressed, self.next(self.rtl)),
            (EventFilter::ArrowRightPressed, self.next(!self.rtl)),
            (EventFilter::PanEndedLeft, self.next(!self.rtl)),
            (EventFilter::PanEndedRight, self.next(self.rtl)),
        ] {
            if let Some(value) = value {
                for handler in &self.select_handlers {
                    let event = if matches!(
                        filter,
                        EventFilter::PanEndedLeft | EventFilter::PanEndedRight
                    ) {
                        EventType::Gesture
                    } else {
                        EventType::Key
                    };
                    viewport =
                        viewport.on(handler.direct_listener_value(event, value).filter(filter));
                }
            }
        }
        let mut previous = Button::new(
            format!("{}::previous", self.key),
            &self.previous_label,
            theme.outline_button(),
        )
        .enabled(self.next(false).is_some())
        .build();
        if let Some(value) = self.next(false) {
            for handler in &self.select_handlers {
                previous = previous.on(handler.direct_listener_value(EventType::Click, value));
            }
        }
        let mut next = Button::new(
            format!("{}::next", self.key),
            &self.next_label,
            theme.outline_button(),
        )
        .enabled(self.next(true).is_some())
        .build();
        if let Some(value) = self.next(true) {
            for handler in &self.select_handlers {
                next = next.on(handler.direct_listener_value(EventType::Click, value));
            }
        }
        let controls = Element::row([
            previous,
            Element::text(position.clone())
                .text_style(argui_text::TextStyle {
                    color: theme.muted_foreground,
                    ..Default::default()
                })
                .semantics(
                    Semantics::new(Role::Status)
                        .label(position)
                        .live(LiveRegion::Polite),
                ),
            next,
        ])
        .gap(12.0)
        .align_items(argui_ui::AlignItems::CENTER);
        Element::column([viewport, controls])
            .keyed(&self.key)
            .gap(12.0)
    }
}
