use crate::app::text;
use argui::{
    runtime::{Context, Entity, Render},
    ui::{Element, FlexWrap, length},
    widgets::{Button, Progress, shadcn},
};

pub(crate) struct ProgressDemo {
    progress: Entity<Progress>,
}

impl Default for ProgressDemo {
    fn default() -> Self {
        Self {
            progress: Entity::new(Progress::new(
                "upload-progress",
                "Upload progress",
                Some(45.0),
            )),
        }
    }
}

impl Render for ProgressDemo {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let value = self.progress.read(Progress::value);
        let advance = cx.callback(|demo| {
            demo.progress.update(|progress, cx| {
                progress.set_value(Some((progress.value().unwrap_or(0.0) + 25.0).min(100.0)));
                cx.notify();
            });
        });
        let reset = cx.callback(|demo| {
            demo.progress.update(|progress, cx| {
                progress.set_value(Some(0.0));
                cx.notify();
            });
        });
        let mode = cx.callback(|demo| {
            demo.progress.update(|progress, cx| {
                progress.set_value(if progress.value().is_some() {
                    None
                } else {
                    Some(45.0)
                });
                cx.notify();
            });
        });
        super::preview(
            "Keep track of the work",
            "Use a percentage when the total is known, or an indeterminate indicator while waiting.",
            Element::column([
                text(
                    value.map_or_else(
                        || "Preparing your upload…".into(),
                        |value| format!("Uploading files · {value:.0}%"),
                    ),
                    14.0,
                    theme.foreground,
                    500,
                ),
                cx.entity(&self.progress),
                Element::row([
                    Button::new("progress-advance", "Advance", theme.button())
                        .enabled(value.is_some_and(|value| value < 100.0))
                        .on_click(advance)
                        .build(),
                    Button::new("progress-reset", "Reset", theme.outline_button())
                        .on_click(reset)
                        .build(),
                    Button::new(
                        "progress-mode",
                        if value.is_some() {
                            "Indeterminate"
                        } else {
                            "Determinate"
                        },
                        theme.outline_button(),
                    )
                    .on_click(mode)
                    .build(),
                ])
                .gap(8.0)
                .flex_wrap(FlexWrap::Wrap),
            ])
            .gap(20.0)
            .max_width(length(520.0)),
            theme,
        )
    }
}
