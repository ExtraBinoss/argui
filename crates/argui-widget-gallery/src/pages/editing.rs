use crate::app::text;
use argui::{
    runtime::{Context, Render},
    ui::{ActionId, Element, EventType, TextPrivacy, UiEventKind, length, percent},
    widgets::{Button, Input, InputKind, TextArea, shadcn},
};

pub(crate) struct EditingDemo {
    values: [String; 5],
    active: Option<argui::ui::NodeId>,
    history: (bool, bool),
    revealed: bool,
    resets: usize,
}

impl Default for EditingDemo {
    fn default() -> Self {
        Self {
            values: [
                "Hello 👋🏽 — مرحبا".into(),
                "A multiline draft.\nTry typing, pasting, then undo.".into(),
                "12.5".into(),
                "(50 + 10) / 2".into(),
                String::new(),
            ],
            active: None,
            history: (false, false),
            revealed: false,
            resets: 0,
        }
    }
}

impl Render for EditingDemo {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment().primary);
        let theme = themes.resolve(cx.environment().color_scheme);
        let fields = [
            InputKind::Text,
            InputKind::Text,
            InputKind::Number,
            InputKind::Arithmetic,
            InputKind::Password,
        ];
        let labels = [
            "Unicode text",
            "Multiline draft",
            "Number",
            "Arithmetic expression",
            "Password",
        ];
        let inputs = fields
            .into_iter()
            .enumerate()
            .map(|(i, kind)| {
                let key = format!("editing-{i}");
                let input = if i == 1 {
                    TextArea::new(&key, &self.values[i], labels[i], theme.input())
                        .build()
                        .height(length(130.0))
                } else {
                    Input::new(&key, &self.values[i], labels[i], theme.input())
                        .kind(kind)
                        .label(labels[i])
                        .build()
                };
                let input = if i == 4 && self.revealed {
                    input.text_privacy(TextPrivacy::RevealedPassword)
                } else {
                    input
                };
                Element::column([text(labels[i], 13.0, theme.muted_foreground, 500), input])
                    .gap(6.0)
            })
            .collect::<Vec<_>>();
        let mut children = vec![text(
            "Focus a field, then use the shared editing commands. Typing groups for 750 ms; paste and IME commits are separate transactions.",
            14.0,
            theme.muted_foreground,
            400,
        )];
        children.extend(inputs);
        let action = |id| argui::ui::ActionInvocation {
            id,
            origin: self.active,
        };
        children.push(
            Element::row([
                Button::new("editing-undo", "Undo", theme.outline_button())
                    .enabled(self.history.0)
                    .build()
                    .action_from(action(ActionId::UNDO)),
                Button::new("editing-redo", "Redo", theme.outline_button())
                    .enabled(self.history.1)
                    .build()
                    .action_from(action(ActionId::REDO)),
                Button::new(
                    "editing-replace",
                    "Undoable replacement",
                    theme.outline_button(),
                )
                .build(),
                Button::new("editing-reset", "External reset", theme.ghost_button()).build(),
                Button::new(
                    "editing-reveal",
                    if self.revealed {
                        "Hide password"
                    } else {
                        "Show password"
                    },
                    theme.ghost_button(),
                )
                .build(),
            ])
            .gap(8.0)
            .flex_wrap(argui::ui::FlexWrap::Wrap),
        );
        children.push(text("Password: no undo history, Copy or Cut; Paste and Select all remain available. Values stay in memory; nothing is saved.", 13.0, theme.muted_foreground, 400));
        let mut root = Element::column(children).gap(14.0).width(percent(1.0));
        for event in [EventType::Focus, EventType::Input, EventType::Click] {
            root = root.on(cx
                .listener(event, |demo, event, cx| {
                    if let Some(index) = event
                        .target_key()
                        .and_then(|key| key.strip_prefix("editing-"))
                        .and_then(|key| key.parse::<usize>().ok())
                        .filter(|i| *i < 5)
                    {
                        demo.active = Some(event.target);
                        demo.history = event.edit_history().unwrap_or_default();
                        if let UiEventKind::TextChanged(value) = &event.kind {
                            demo.values[index] = value.clone();
                        }
                        cx.notify();
                    }
                    if matches!(event.kind, UiEventKind::Click(_)) {
                        match event.target_key() {
                            Some("editing-reveal") => demo.revealed = !demo.revealed,
                            Some("editing-replace") => {
                                cx.edit_text("editing-0", "One atomic replacement 👩‍🚀")
                            }
                            Some("editing-reset") => {
                                demo.resets += 1;
                                demo.values[0] = format!("External reset {}", demo.resets);
                                demo.history = (false, false);
                            }
                            _ => return,
                        }
                        cx.notify();
                    }
                })
                .capture(event == EventType::Focus));
        }
        root
    }
}
