use argui::{
    runtime::{
        Context, Render,
        tasks::{self, TaskSlot},
    },
    ui::{Element, EventType, UiEvent, UiEventKind},
    widgets::{Button, Toast, ToastHost, ToastState, shadcn},
};
use std::time::Duration;
use web_time::Instant;

pub(crate) struct ToastDemo {
    state: ToastState,
    origin: Instant,
    timer: TaskSlot,
    serial: u64,
    error: String,
}
impl Default for ToastDemo {
    fn default() -> Self {
        Self {
            state: ToastState::new(3, 8, Duration::ZERO),
            origin: Instant::now(),
            timer: TaskSlot::default(),
            serial: 0,
            error: String::new(),
        }
    }
}
impl ToastDemo {
    fn schedule(&mut self, cx: &mut Context<Self>) {
        self.timer.cancel();
        if let Some(deadline) = self.state.next_deadline() {
            let wait = deadline.saturating_sub(self.origin.elapsed());
            if let Err(error) =
                cx.spawn_latest(&mut self.timer, tasks::sleep(wait), |demo, _, cx| {
                    demo.state.advance(demo.origin.elapsed());
                    demo.schedule(cx);
                    cx.notify();
                })
            {
                self.error = error.to_string();
            }
        }
    }
    fn event(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        if event.target_key() == Some("toast-add") && matches!(event.kind, UiEventKind::Click(_)) {
            self.serial += 1;
            if let Err(error) = self.state.insert(
                Toast::new(
                    self.serial.to_string(),
                    format!("Notification {}", self.serial),
                    Some(Duration::from_secs(5)),
                ),
                self.origin.elapsed(),
            ) {
                self.error = format!("{error:?}");
            }
        } else if let Some(id) = (ToastHost {
            key: "toasts",
            state: &self.state,
            close_label: "Close",
        })
        .close_action(event)
        {
            self.state.close(id, self.origin.elapsed());
        } else if let Some((id, reason, paused)) = (ToastHost {
            key: "toasts",
            state: &self.state,
            close_label: "Close",
        })
        .pause_action(event)
        {
            self.state.pause(&id, reason, paused, self.origin.elapsed());
        } else {
            return;
        }
        self.schedule(cx);
        cx.notify();
    }
}
impl Render for ToastDemo {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment().primary);
        let theme = themes.resolve(cx.environment().color_scheme);
        Element::column([
            Button::new("toast-add", "Show notification", theme.button()).build(),
            ToastHost {
                key: "toasts",
                state: &self.state,
                close_label: "Close",
            }
            .build(theme),
            Element::text(self.error.as_str()),
        ])
        .gap(12.0)
        .on(cx.listener(EventType::Click, Self::event))
        .on(cx
            .listener(EventType::PointerEnter, Self::event)
            .capture(true))
        .on(cx
            .listener(EventType::PointerLeave, Self::event)
            .capture(true))
        .on(cx.listener(EventType::Focus, Self::event).capture(true))
        .on(cx.listener(EventType::Blur, Self::event).capture(true))
    }
}
