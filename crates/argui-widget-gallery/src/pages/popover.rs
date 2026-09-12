use argui::{
    core::{Key, KeyState},
    runtime::{Context, Render},
    ui::{Element, EventType, FlexWrap, FloatingPlacement, Placement, UiEvent, UiEventKind},
    widgets::{Button, Input, Popover, PopoverAction, PopoverBehavior, Switch, shadcn},
};

use super::overlay_effects::Surface;
use crate::app::text;

const KEYS: [&str; 3] = ["popover-project", "popover-sharing", "popover-color"];
const LABELS: [&str; 3] = ["Rename project", "Sharing settings", "Choose accent"];
const NESTED: &str = "popover-link-options";

pub(crate) struct PopoverDemo {
    open: Option<usize>,
    name: String,
    saved: String,
    sharing: bool,
    link_options: bool,
    downloads: bool,
    color: usize,
}

impl Default for PopoverDemo {
    fn default() -> Self {
        Self {
            open: None,
            name: "Studio notes".into(),
            saved: "Studio notes".into(),
            sharing: false,
            link_options: false,
            downloads: false,
            color: 0,
        }
    }
}

impl PopoverDemo {
    pub(crate) fn close(&mut self) {
        self.open = None;
        self.link_options = false;
    }

    pub(crate) fn dismiss(&mut self, event: &UiEvent) -> bool {
        if !matches!(&event.kind, UiEventKind::KeyInput(input)
            if input.key == Key::Escape && input.state == KeyState::Pressed)
        {
            return false;
        }
        if self.link_options {
            self.link_options = false;
            true
        } else {
            self.open.take().is_some()
        }
    }

    fn event(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        // Address the inner panel before considering the parent.
        if self.open == Some(1)
            && let Some(action) =
                PopoverBehavior::new(NESTED, "Link options", self.link_options).action(event)
        {
            self.link_options = action == PopoverAction::Toggle && !self.link_options;
            event.stop_propagation();
            cx.notify();
            return;
        }
        for (index, key) in KEYS.into_iter().enumerate() {
            if let Some(action) =
                PopoverBehavior::new(key, LABELS[index], self.open == Some(index)).action(event)
            {
                self.open = match action {
                    PopoverAction::Toggle if self.open != Some(index) => Some(index),
                    _ => None,
                };
                self.link_options = false;
                event.stop_propagation();
                cx.notify();
                return;
            }
        }
        match (&event.kind, event.target_key()) {
            (UiEventKind::TextChanged(value), Some("popover-name")) => self.name.clone_from(value),
            (UiEventKind::Click(_), Some("popover-save")) => {
                self.saved.clone_from(&self.name);
                self.close();
            }
            (UiEventKind::Click(_), Some("popover-sharing-toggle")) => self.sharing = !self.sharing,
            (UiEventKind::Click(_), Some("popover-downloads-toggle")) => {
                self.downloads = !self.downloads
            }
            (UiEventKind::Click(_), Some(key)) if key.starts_with("popover-accent-") => {
                self.color = match key {
                    "popover-accent-rose" => 1,
                    "popover-accent-violet" => 2,
                    _ => 0,
                };
            }
            _ => return,
        }
        cx.notify();
    }
}

impl Render for PopoverDemo {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment().primary);
        let theme = themes.resolve(cx.environment().color_scheme);
        let content = [
            Element::column([
                text("Project name", 15.0, theme.foreground, 600),
                Input::new("popover-name", &self.name, "Project name", theme.input())
                    .label("Project name")
                    .build(),
                Button::new("popover-save", "Save name", theme.button()).build(),
            ])
            .gap(12.0),
            Element::column([
                text("Link access", 15.0, theme.foreground, 600),
                text(
                    "Choose who can read these notes.",
                    13.0,
                    theme.foreground,
                    400,
                ),
                Switch::new(
                    "popover-sharing-toggle",
                    "Anyone with the link",
                    self.sharing,
                )
                .build(theme),
                text(
                    if self.sharing {
                        "Public link enabled"
                    } else {
                        "Only invited people"
                    },
                    13.0,
                    theme.foreground,
                    400,
                ),
                Popover::new(
                    NESTED,
                    "Link options",
                    self.link_options,
                    Button::new(NESTED, "Link options", theme.outline_button()).build(),
                    Element::column([
                        text("Download permissions", 15.0, theme.foreground, 600),
                        text(
                            "Choose what visitors can save.",
                            13.0,
                            theme.foreground,
                            400,
                        ),
                        Switch::new(
                            "popover-downloads-toggle",
                            "Allow downloads",
                            self.downloads,
                        )
                        .build(theme),
                        text(
                            if self.downloads {
                                "Downloads allowed"
                            } else {
                                "View only"
                            },
                            13.0,
                            theme.foreground,
                            400,
                        ),
                    ])
                    .gap(12.0),
                )
                .placement(FloatingPlacement::new(Placement::RightEnd))
                .size(236.0, 240.0)
                .paint(Surface::Frosted.paint(theme, cx.environment().color_scheme))
                .layer(Surface::Frosted.layer(theme))
                .build(theme),
            ])
            .gap(12.0),
            Element::column([
                text("Workspace accent", 15.0, theme.foreground, 600),
                Element::column(
                    [("blue", "Blue"), ("rose", "Rose"), ("violet", "Violet")]
                        .into_iter()
                        .enumerate()
                        .map(|(index, (key, name))| {
                            Button::new(
                                format!("popover-accent-{key}"),
                                name,
                                if self.color == index {
                                    theme.button()
                                } else {
                                    theme.ghost_button()
                                },
                            )
                            .build()
                        }),
                )
                .gap(4.0),
            ])
            .gap(10.0),
        ];
        let cards =
            Surface::ALL
                .into_iter()
                .zip(content)
                .enumerate()
                .map(|(index, (surface, content))| {
                    let trigger =
                        Button::new(KEYS[index], LABELS[index], theme.outline_button()).build();
                    let popover = Popover::new(
                        KEYS[index],
                        LABELS[index],
                        self.open == Some(index),
                        trigger,
                        content,
                    )
                    .size(236.0, 300.0)
                    .paint(surface.paint(theme, cx.environment().color_scheme))
                    .layer(surface.layer(theme))
                    .build(theme);
                    surface.card(popover, theme)
                });
        Element::column([
            Element::row(cards).flex_wrap(FlexWrap::Wrap).gap(16.0),
            text(
                format!(
                    "Project: {} · Accent: {}",
                    self.saved,
                    ["Blue", "Rose", "Violet"][self.color]
                ),
                14.0,
                theme.foreground,
                400,
            ),
        ])
        .gap(18.0)
        .on(cx.listener(EventType::Click, Self::event))
        .on(cx.listener(EventType::Input, Self::event))
        .on(cx.listener(EventType::PointerOutside, Self::event))
    }
}
