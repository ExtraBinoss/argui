use argui::{
    runtime::{Context, Entity, Render},
    ui::{Element, EventType, FlexWrap, UiEvent, length},
    widgets::{AnimatedText, Button, TextAnimation, shadcn},
};

pub(crate) struct AnimatedTextDemo {
    value: i32,
    numbers: [Entity<AnimatedText>; 3],
    status: Entity<AnimatedText>,
    saved: bool,
}

impl Default for AnimatedTextDemo {
    fn default() -> Self {
        Self {
            value: 10,
            numbers: [
                TextAnimation::Roll,
                TextAnimation::Slide,
                TextAnimation::Fade,
            ]
            .map(|animation| {
                Entity::new(
                    AnimatedText::new(format!("animated-{animation:?}"), "10")
                        .animation(animation)
                        .font_size(44.0),
                )
            }),
            status: Entity::new(
                AnimatedText::new("animated-status", "Draft")
                    .animation(TextAnimation::Fade)
                    .align_end(false),
            ),
            saved: false,
        }
    }
}

impl AnimatedTextDemo {
    fn input(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        match event.target_key() {
            Some("animated-increment") => self.value = self.value.saturating_add(1),
            Some("animated-decrement") => self.value = self.value.saturating_sub(1),
            Some("animated-carry") => self.value = if self.value == 99 { 100 } else { 99 },
            Some("animated-reset") => self.value = 10,
            Some("animated-save") => {
                self.saved = !self.saved;
                self.status.update(|status, cx| {
                    status.set_text(if self.saved { "Saved" } else { "Draft" });
                    cx.notify();
                });
            }
            _ => return,
        }
        for number in &self.numbers {
            number.update(|number, cx| {
                number.set_text(self.value.to_string());
                cx.notify();
            });
        }
        cx.notify();
    }
}

impl Render for AnimatedTextDemo {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let cards = ["Roll", "Slide", "Fade"]
            .into_iter()
            .zip(&self.numbers)
            .map(|(label, number)| {
                Element::column([
                    crate::app::text(label, 14.0, theme.muted_foreground, 500),
                    cx.entity(number),
                ])
                .padding(argui::ui::Sides::length(18.0))
                .gap(12.0)
                .width(length(210.0))
                .background(theme.card)
                .border(argui::paint::Border::all(1.0, theme.border))
                .radius(argui::paint::CornerRadii::all(12.0))
            })
            .collect::<Vec<_>>();
        super::preview("Only the changing digits move",
            "Try 10 to 11, a carry from 99 to 100, or several quick clicks. Reduced motion shows the final value immediately.",
            Element::column([
                Element::row(cards).gap(16.0).flex_wrap(FlexWrap::Wrap),
                Element::row([
                    Button::new("animated-decrement", "−1", theme.outline_button()).build(),
                    Button::new("animated-increment", "+1", theme.button()).build(),
                    Button::new("animated-carry", "99 / 100", theme.outline_button()).build(),
                    Button::new("animated-reset", "Reset to 10", theme.ghost_button()).build(),
                ]).gap(8.0).flex_wrap(FlexWrap::Wrap),
                Element::row([
                    Button::new("animated-save", "Toggle saved status", theme.outline_button()).build(),
                    cx.entity(&self.status),
                ]).gap(16.0).align_items(argui::ui::AlignItems::CENTER).flex_wrap(FlexWrap::Wrap),
            ]).gap(24.0), theme)
            .on(cx.listener(EventType::Click, Self::input))
    }
}
