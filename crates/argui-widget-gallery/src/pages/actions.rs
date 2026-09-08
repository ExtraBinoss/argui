use super::action_menu::ActionMenu;
use crate::app::text;
use argui::{
    runtime::{Context, Render},
    ui::{
        ActionId, ActionInvocation, ActionScope, ActionState, Element, EventType, Shortcut,
        UiEventKind, percent,
    },
    widgets::{Button, Dialog, DialogAction, DialogBehavior, Input, MenuItem, shadcn},
};

const RECORD: ActionId = ActionId("gallery.record");
const PALETTE: ActionId = ActionId("gallery.palette");

#[derive(Default)]
pub(crate) struct ActionsDemo {
    menu: ActionMenu,
    left: String,
    right: String,
    log: String,
    disabled: bool,
    modal: bool,
}

fn record_state(enabled: bool) -> ActionState {
    ActionState::new("Record action")
        .enabled(enabled)
        .shortcut(Shortcut::primary("j"))
}
impl ActionsDemo {
    fn items(&self) -> Vec<MenuItem> {
        vec![MenuItem::new(
            ActionInvocation::new(RECORD),
            record_state(!self.disabled),
        )]
    }
}

impl Render for ActionsDemo {
    fn wants_animation_frame(&self) -> bool {
        self.menu.animating()
    }

    fn animation_frame(&mut self, frame: argui::animation::Frame, cx: &mut Context<Self>) {
        self.menu.advance(frame, cx);
    }

    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        self.menu.sync(cx);
        let themes = shadcn(cx.environment().primary);
        let theme = themes.resolve(cx.environment().color_scheme);
        let mut panels = Vec::new();
        for (key, label, value) in [
            ("scope-left", "Left scope", &self.left),
            ("scope-right", "Right scope", &self.right),
        ] {
            let binding = cx.on_action(RECORD, record_state(!self.disabled), move |demo, _, cx| {
                demo.log = format!("Executed in {label}");
                cx.notify();
            });
            panels.push(
                Element::column([
                    text(label, 18.0, theme.foreground, 600),
                    Input::new(
                        key,
                        value,
                        "Focus here, then Ctrl+J or open Commands",
                        theme.input(),
                    )
                    .build(),
                    Button::new(format!("{key}-record"), "Record action", theme.button())
                        .enabled(!self.disabled)
                        .build()
                        .action(RECORD),
                ])
                .gap(10.0)
                .width(percent(1.0))
                .action_scope(ActionScope::new([binding]).expect("unique scope action")),
            );
        }
        let root_scope = ActionScope::new([
            cx.on_action(RECORD, record_state(true), |demo, _, cx| {
                demo.log = "Executed at window scope".into();
                cx.notify();
            }),
            cx.on_action(
                PALETTE,
                ActionState::new("Commands").shortcut(Shortcut::primary("k")),
                |demo, event, cx| {
                    demo.menu.show_palette(event);
                    cx.notify();
                },
            ),
        ])
        .expect("distinct shortcuts");
        let modal_scope =
            ActionScope::new([cx.on_action(RECORD, record_state(true), |demo, _, cx| {
                demo.log = "Executed in modal scope".into();
                cx.notify();
            })])
            .expect("unique action");
        panels.extend([
            self.menu.build(self.items(), theme),
            Button::new(
                "actions-disable",
                if self.disabled {
                    "Enable local actions"
                } else {
                    "Disable local actions"
                },
                theme.outline_button(),
            )
            .build(),
            Dialog::new(
                "actions-modal",
                "Modal action scope",
                self.modal,
                Button::new("modal-open", "Open modal", theme.outline_button()).build(),
                Element::column([
                    text(
                        "Ctrl+J resolves here. Window commands cannot escape the modal.",
                        14.0,
                        theme.foreground,
                        400,
                    ),
                    Button::new("modal-record", "Record action", theme.button())
                        .build()
                        .action(RECORD),
                    Button::new("modal-close", "Close", theme.ghost_button()).build(),
                ])
                .gap(10.0)
                .action_scope(modal_scope),
            )
            .build(theme),
            text(&self.log, 16.0, theme.foreground, 500),
        ]);
        let mut root = Element::column(panels)
            .gap(16.0)
            .width(percent(1.0))
            .action_scope(root_scope);
        for kind in [
            EventType::Click,
            EventType::PointerDown,
            EventType::PointerOutside,
            EventType::Key,
            EventType::Input,
            EventType::Focus,
        ] {
            root = root.on(cx
                .listener(kind, |demo, event, cx| {
                    demo.menu.handle(event, demo.items(), cx);
                    if matches!(event.kind, UiEventKind::Focused)
                        && matches!(event.target_key(), Some("scope-left" | "scope-right"))
                    {
                        demo.menu.origin = Some(event.target);
                    }
                    if let Some(action) =
                        DialogBehavior::new("actions-modal", "Modal action scope", demo.modal)
                            .action(event)
                    {
                        demo.modal = match action {
                            DialogAction::Open => true,
                            DialogAction::Close => false,
                        };
                        cx.notify();
                    }
                    match (&event.kind, event.target_key()) {
                        (UiEventKind::Click(_), Some("actions-disable")) => {
                            demo.disabled = !demo.disabled
                        }
                        (UiEventKind::Click(_), Some("modal-close")) => demo.modal = false,
                        (UiEventKind::TextChanged(value), Some("scope-left")) => {
                            demo.left = value.clone()
                        }
                        (UiEventKind::TextChanged(value), Some("scope-right")) => {
                            demo.right = value.clone()
                        }
                        _ => return,
                    }
                    cx.notify();
                })
                .capture(kind == EventType::Focus));
        }
        root
    }
}
