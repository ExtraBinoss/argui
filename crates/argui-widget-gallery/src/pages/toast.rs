use argui::{
    runtime::{
        Context, Render,
        tasks::{self, TaskSlot},
    },
    ui::{Element, EventType, UiEvent},
    widgets::{Button, Toast, ToastHost, ToastState, ToastVariant, shadcn},
};
use std::time::Duration;
use web_time::Instant;

pub(crate) struct ToastDemo {
    icons: argui::widgets::WidgetAssets,
    state: ToastState,
    origin: Instant,
    timer: TaskSlot,
    serial: u64,
    error: String,
}
impl ToastDemo {
    pub(crate) fn new(icons: &argui::widgets::WidgetAssets) -> Self {
        Self {
            icons: icons.clone(),
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
                    cx.notify();
                })
            {
                self.error = error.to_string();
            }
        }
    }
    fn add(&mut self) {
        self.serial += 1;
        let mut toast = Toast::new(
            self.serial.to_string(),
            format!("Notification {}", self.serial),
            Some(Duration::from_secs(5)),
        );
        toast.description = "Your changes have been saved successfully.".into();
        toast.variant = ToastVariant::Success;
        if let Err(error) = self.state.insert(toast, self.origin.elapsed()) {
            self.error = format!("{error:?}");
        }
    }
    fn pause_event(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        if let Some((id, reason, paused)) =
            ToastHost::new("toasts", &self.state, "Close").pause_action(event)
        {
            self.state.pause(&id, reason, paused, self.origin.elapsed());
        } else {
            return;
        }
        cx.notify();
    }
}
impl Render for ToastDemo {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        self.schedule(cx);
        let themes = shadcn(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        ToastHost::new("toasts", &self.state, "Close")
            .on_close(cx.value_callback(|demo, id: String| {
                demo.state.close(&id, demo.origin.elapsed());
            }))
            .build(theme, &self.icons)
            .on(cx
                .listener(EventType::PointerEnter, Self::pause_event)
                .capture(true))
            .on(cx
                .listener(EventType::PointerLeave, Self::pause_event)
                .capture(true))
            .on(cx
                .listener(EventType::Focus, Self::pause_event)
                .capture(true))
            .on(cx
                .listener(EventType::Blur, Self::pause_event)
                .capture(true))
    }
}

pub(crate) fn controls(
    entity: &argui::runtime::Entity<ToastDemo>,
    theme: &argui::widgets::WidgetTheme,
    cx: &mut Context<crate::WidgetGallery>,
) -> Element {
    let error = entity.read(|demo| demo.error.clone());
    let add = {
        let entity = entity.clone();
        cx.callback(move |_| {
            entity.update(|demo, cx| {
                demo.add();
                cx.notify();
            });
        })
    };
    Element::column([
        Button::new("toast-add", "Show notification", theme.button())
            .on_click(add)
            .build(),
        Element::text(error).text_style(argui::text::TextStyle {
            color: theme.muted_foreground,
            ..Default::default()
        }),
    ])
    .gap(12.0)
    .align_items(argui::ui::AlignItems::START)
}
