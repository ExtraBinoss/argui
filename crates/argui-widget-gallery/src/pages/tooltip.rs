use argui::{
    runtime::{
        Context, Render,
        tasks::{self, TaskSlot},
    },
    ui::{Element, EventType, FlexWrap, FloatingPlacement, Placement, UiEvent, UiEventKind},
    widgets::{Button, Tooltip, TooltipState, shadcn},
};
use web_time::Instant;

use super::overlay_effects::Surface;
use crate::app::text;

const KEYS: [&str; 3] = ["tooltip-save", "tooltip-preview", "tooltip-history"];
const LABELS: [&str; 3] = ["Save draft", "Preview page", "View history"];
const DESCRIPTIONS: [&str; 3] = [
    "Save a local copy of your current draft.",
    "Preview the page before sharing it.",
    "See the previous versions of this project.",
];

pub(crate) struct TooltipDemo {
    tips: [TooltipState; 3],
    timer: TaskSlot,
    origin: Instant,
    status: String,
}

impl Default for TooltipDemo {
    fn default() -> Self {
        Self {
            tips: KEYS.map(TooltipState::new),
            timer: TaskSlot::default(),
            origin: Instant::now(),
            status: "Hover a button or reach it with Tab. Escape dismisses the hint.".into(),
        }
    }
}

impl TooltipDemo {
    pub(crate) fn reset(&mut self) {
        self.timer.cancel();
        for tip in &mut self.tips {
            tip.reset();
        }
    }

    fn schedule(&mut self, cx: &mut Context<Self>) {
        self.timer.cancel();
        if let Some(deadline) = self
            .tips
            .iter()
            .filter_map(TooltipState::next_deadline)
            .min()
        {
            let delay = deadline.saturating_sub(self.origin.elapsed());
            if let Err(error) =
                cx.spawn_latest(&mut self.timer, tasks::sleep(delay), |demo, _, cx| {
                    let now = demo.origin.elapsed();
                    for tip in &mut demo.tips {
                        tip.advance(now);
                    }
                    demo.schedule(cx);
                    cx.notify();
                })
            {
                self.status = error.to_string();
            }
        }
    }

    pub(crate) fn dismiss(&mut self, event: &UiEvent) -> bool {
        let now = self.origin.elapsed();
        let mut changed = false;
        for tip in &mut self.tips {
            changed |= tip.update(event, now);
        }
        if changed {
            self.timer.cancel();
        }
        changed
    }

    fn event(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        let now = self.origin.elapsed();
        let mut changed = false;
        for tip in &mut self.tips {
            changed |= tip.update(event, now);
        }
        if let UiEventKind::Click(_) = event.kind
            && let Some(index) = KEYS.iter().position(|key| Some(*key) == event.target_key())
        {
            self.status = [
                "Draft saved locally.",
                "Page preview is ready.",
                "History: three saved versions.",
            ][index]
                .into();
            changed = true;
        }
        if changed {
            self.schedule(cx);
            cx.notify();
        }
    }
}

impl Render for TooltipDemo {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment().primary);
        let theme = themes.resolve(cx.environment().color_scheme);
        let cards = Surface::ALL
            .into_iter()
            .enumerate()
            .map(|(index, surface)| {
                let trigger =
                    Button::new(KEYS[index], LABELS[index], theme.outline_button()).build();
                let tooltip = Tooltip::new(
                    KEYS[index],
                    DESCRIPTIONS[index],
                    self.tips[index].is_open(),
                    trigger,
                )
                .placement(FloatingPlacement::new(Placement::BottomStart))
                .max_width(236.0)
                .paint(surface.paint(theme))
                .layer(surface.layer(theme))
                .build(theme);
                surface.card(tooltip, theme)
            });
        let mut root = Element::column([
            Element::row(cards).flex_wrap(FlexWrap::Wrap).gap(16.0),
            text(&self.status, 14.0, theme.foreground, 400),
        ])
        .gap(18.0);
        for kind in [
            EventType::PointerEnter,
            EventType::PointerLeave,
            EventType::PointerDown,
            EventType::PointerCancel,
            EventType::Focus,
            EventType::Blur,
            EventType::Click,
            EventType::Key,
        ] {
            root = root.on(cx.listener(kind, Self::event).capture(true));
        }
        root
    }
}
