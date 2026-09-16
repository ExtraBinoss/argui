use argui::{
    paint::{Border, CornerRadii},
    runtime::{Context, Render},
    text::TextStyle,
    ui::{AlignItems, Element, Sides, length, percent},
    widgets::{Button, default_theme},
};

const CHECKS: [&str; 3] = [
    "State is explicit",
    "Views stay pure",
    "Effects have owners",
];

#[derive(Default)]
pub struct Example {
    completed: usize,
}

impl Render for Example {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = default_theme(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let rows = CHECKS.into_iter().enumerate().map(|(index, label)| {
            let complete = index < self.completed;
            Element::row([
                Element::text(if complete { "Done" } else { "Pending" })
                    .width(length(68.0))
                    .text_style(TextStyle {
                        color: if complete {
                            theme.primary
                        } else {
                            theme.muted_foreground
                        },
                        font_size: 12.0,
                        weight: 700,
                        ..TextStyle::default()
                    }),
                Element::text(label).text_style(TextStyle {
                    color: theme.foreground,
                    ..TextStyle::default()
                }),
            ])
            .width(percent(1.0))
            .align_items(AlignItems::CENTER)
            .padding(Sides::length(12.0))
            .gap(12.0)
            .background(theme.card)
            .border(Border::all(1.0, theme.border))
            .radius(CornerRadii::all(8.0))
        });
        let next_label = if self.completed == CHECKS.len() {
            "All complete".to_owned()
        } else {
            format!("Complete next ({} of {})", self.completed + 1, CHECKS.len())
        };
        Element::column(
            rows.chain([Button::new("next", next_label, theme.button())
                .enabled(self.completed < CHECKS.len())
                .on_click(cx.callback(|app| {
                    app.completed = (app.completed + 1).min(CHECKS.len());
                }))
                .build()]),
        )
        .width(percent(1.0))
        .height(percent(1.0))
        .padding(Sides::length(28.0))
        .gap(14.0)
        .background(theme.background)
    }
}
