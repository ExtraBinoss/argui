use crate::app::text;
use argui::{
    core::Color,
    runtime::{
        Context, Render,
        tasks::{self, TaskSlot},
    },
    ui::{Element, EventType, UiEventKind, length, percent},
    widgets::{Button, Input, InputKind, VList, shadcn},
};
use std::time::Duration;

#[derive(Default)]
pub(crate) struct TasksDemo {
    query: String,
    slot: TaskSlot,
    results: Vec<String>,
    status: String,
    offset: f32,
}

async fn search(query: String) -> Result<Vec<String>, String> {
    tasks::sleep(Duration::from_millis(200)).await;
    // A numeric prefix is a real filter grammar, with a visible parse error.
    let minimum = query
        .strip_prefix("id:")
        .map(|value| value.parse::<usize>().map_err(|error| error.to_string()))
        .transpose()?;
    let words = [
        "Architecture",
        "Design review",
        "Release notes",
        "Meeting agenda",
        "Travel plans",
    ];
    let query = query.to_lowercase();
    let mut found = Vec::new();
    for index in 0..10_000 {
        let title = format!("Draft {index:05} · {}", words[index % words.len()]);
        if minimum.map_or_else(
            || title.to_lowercase().contains(&query),
            |minimum| index >= minimum,
        ) {
            found.push(title);
        }
        if index % 128 == 0 {
            tasks::yield_now().await;
        }
    }
    Ok(found)
}

impl TasksDemo {
    fn start(&mut self, cx: &mut Context<Self>) {
        self.status = "Searching…".into();
        let query = self.query.clone();
        if let Err(error) = cx.spawn_latest(&mut self.slot, search(query), |demo, result, cx| {
            match result {
                Ok(Ok(results)) => {
                    demo.status = format!("{} matching drafts", results.len());
                    demo.results = results;
                    demo.offset = 0.0;
                }
                Ok(Err(error)) => demo.status = format!("Invalid query: {error}"),
                Err(error) => demo.status = format!("Task failed: {error}"),
            }
            cx.notify();
        }) {
            self.status = error.to_string();
        }
        cx.notify();
    }
}
impl Render for TasksDemo {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        Element::column([
            text("Search 10,000 draft titles", 20.0, theme.foreground, 600),
            text("Type a title or id:500. Invalid id:abc produces a real parsing error. New searches cancel old results.", 13.0, theme.muted_foreground, 400),
            Input::new("tasks-query", &self.query, "Title or id:500", theme.input())
                .kind(InputKind::Search)
                .label("Draft search")
                .on_input(cx.input_event_handler(|demo, query, _, cx| {
                    demo.query = query;
                    demo.start(cx);
                }))
                .build().width(percent(1.0)),
            Element::row([
                Button::new("tasks-search", "Search", theme.button())
                    .on_click(cx.event_handler(|demo, _, cx| demo.start(cx)))
                    .build(),
                Button::new("tasks-cancel", "Cancel", theme.ghost_button())
                    .enabled(self.slot.is_running())
                    .on_click(cx.callback(|demo| {
                        demo.slot.cancel();
                        demo.status = "Cancelled".into();
                    }))
                    .build(),
            ]).gap(8.0),
            text(&self.status, 13.0, theme.foreground, 400),
            VList::new("tasks-results", 32.0, 256.0, self.offset)
                .build(self.results.len(), theme, |index| {
                    text(&self.results[index], 14.0, theme.foreground, 400).height(length(32.0))
                }),
            Element::container([]).height(length(1.0)).background(Color::TRANSPARENT),
        ]).gap(12.0).width(percent(1.0))
            .on(cx.listener(EventType::Scroll, |demo, event, cx| {
                if event.target_key() == Some("tasks-results")
                    && let UiEventKind::Scrolled { offset, .. } = event.kind {
                    demo.offset = offset.y;
                    cx.notify();
                }
            }))
    }
}
