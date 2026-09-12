use crate::{Button, ButtonBehavior, WidgetTheme};
use argui_core::{Key, KeyState};
use argui_ui::{
    Element, FocusPolicy, GestureKind, GesturePhase, GestureSet, Interaction, LiveRegion, PanAxis,
    PanGesture, Role, Semantics, UiEvent, UiEventKind,
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
}

impl Carousel {
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
        }
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
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        let selected = self.selected.min(self.slides.len().saturating_sub(1));
        let slide = self.slides.get(selected).cloned();
        let position = if self.slides.is_empty() {
            "0 / 0".into()
        } else {
            format!("{} / {}", selected + 1, self.slides.len())
        };
        let viewport = Element::column(slide)
            .keyed(format!("{}::viewport", self.key))
            .interaction(
                Interaction::default()
                    .focus_policy(FocusPolicy::TabStop)
                    .gestures(
                        GestureSet::default().pan(PanGesture::default().axis(PanAxis::Horizontal)),
                    ),
            )
            .semantics(Semantics::new(Role::Group).label(&self.label));
        let controls = Element::row([
            Button::new(
                format!("{}::previous", self.key),
                &self.previous_label,
                theme.outline_button(),
            )
            .enabled(self.next(false).is_some())
            .build(),
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
            Button::new(
                format!("{}::next", self.key),
                &self.next_label,
                theme.outline_button(),
            )
            .enabled(self.next(true).is_some())
            .build(),
        ])
        .gap(12.0)
        .align_items(argui_ui::AlignItems::CENTER);
        Element::column([viewport, controls])
            .keyed(&self.key)
            .gap(12.0)
    }
}
