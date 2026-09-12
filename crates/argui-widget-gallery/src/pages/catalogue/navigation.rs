use super::*;
use crate::app::text;
use argui::{
    ui::{Orientation, WritingDirection, length},
    widgets::*,
};

impl CatalogueDemo {
    fn accordion(&self, theme: &WidgetTheme) -> Accordion {
        let items = [
            ("keyboard", "Can I use the keyboard?", "Use Tab to reach a heading, Enter to expand it, and arrows to move between headings."),
            ("themes", "Does it support dark mode?", "Change the gallery theme to see every component adapt."),
            ("state", "Where is the state stored?", "Your application controls which sections stay open."),
        ].map(|(id, label, content)| {
            let mut item = AccordionItem::new(id, label, text(content, 14.0, theme.foreground, 400)); item.open = self.choices.contains(id); item
        });
        let mut accordion = Accordion::new("faq", items);
        accordion.mode = ChoiceMode::Multiple;
        accordion
    }

    fn toggles(&self) -> ToggleGroup {
        let mut group = ToggleGroup::new(
            "format",
            "Text formatting",
            ["Bold", "Italic", "Underline"].map(|label| {
                Toggle::new(
                    label.to_lowercase(),
                    label,
                    self.choices.contains(&label.to_lowercase()),
                )
            }),
        );
        group.mode = ChoiceMode::Multiple;
        group.active = self.active.clone();
        group
    }

    fn navigation_menu(&self, theme: &WidgetTheme) -> NavigationMenu {
        let mut guide = NavigationItem::new("guide", "Guide");
        guide.panel = Some(
            Element::column([
                text("Build your first interface", 17.0, theme.foreground, 600),
                text(
                    "Start with a window, a layout and a few controls.",
                    14.0,
                    theme.muted_foreground,
                    400,
                ),
                Button::new("guide-start", "Open getting started", theme.button()).build(),
            ])
            .gap(10.0),
        );
        let mut overview = NavigationItem::new("overview", "Overview");
        overview.current = true;
        let mut menu = NavigationMenu::new(
            "docs",
            "Documentation",
            [overview, guide, NavigationItem::new("examples", "Examples")],
        );
        menu.open = self.active.clone();
        menu
    }

    fn sidebar(&self, theme: &WidgetTheme) -> Sidebar {
        let mut sidebar = Sidebar::new(
            "workspace",
            "Workspace",
            Element::column(["Projects", "Activity", "Settings"].map(|name| {
                Button::new(format!("workspace-{name}"), name, theme.ghost_button()).build()
            }))
            .gap(4.0),
        );
        sidebar.header = Some(text("Your workspace", 15.0, theme.foreground, 600));
        sidebar.footer = Some(text("Personal account", 12.0, theme.muted_foreground, 400));
        sidebar.rail = Some(Button::new("workspace-Projects", "P", theme.ghost_button()).build());
        sidebar.collapsed = self.collapsed;
        sidebar
    }

    pub(super) fn navigation_view(&self, theme: &WidgetTheme) -> Element {
        match self.page {
            Page::Accordion => self.accordion(theme).build(theme),
            Page::Toggle => Toggle::new("bold", "Bold", self.open).build(theme),
            Page::ToggleGroup => self.toggles().build(theme),
            Page::NavigationMenu => self.navigation_menu(theme).build(theme),
            Page::Sidebar => self.sidebar(theme).build(theme),
            Page::ButtonGroup => {
                let group = || {
                    ["Back", "Refresh", "Forward"].map(|label| {
                        Button::new(
                            format!("group-{}", label.to_lowercase()),
                            label,
                            theme.outline_button(),
                        )
                        .build()
                    })
                };
                let horizontal = ButtonGroup::new("browser", "Browsing", group()).build();
                let mut vertical = ButtonGroup::new(
                    "tools",
                    "Editing",
                    ["Copy", "Paste"].map(|label| {
                        Button::new(
                            format!("group-{}", label.to_lowercase()),
                            label,
                            theme.outline_button(),
                        )
                        .build()
                    }),
                );
                vertical.orientation = Orientation::Vertical;
                Element::column([horizontal, vertical.build()]).gap(24.0)
            }
            Page::Direction => Element::column([WritingDirection::Ltr, WritingDirection::Rtl].map(
                |direction| {
                    Element::column([
                        text(
                            if direction == WritingDirection::Ltr {
                                "Left to right"
                            } else {
                                "Right to left"
                            },
                            17.0,
                            theme.foreground,
                            600,
                        ),
                        Direction::new(
                            direction,
                            Element::row(["First", "Second", "Third"].map(|label| {
                                text(label, 14.0, theme.foreground, 500)
                                    .padding(argui::ui::Sides::length(14.0))
                                    .background(theme.muted)
                            }))
                            .gap(12.0)
                            .width(argui::ui::percent(1.0)),
                        )
                        .build(),
                    ])
                    .gap(12.0)
                },
            ))
            .gap(24.0)
            .max_width(length(600.0)),
            _ => unreachable!("navigation page"),
        }
    }

    pub(super) fn navigation_event(
        &mut self,
        event: &UiEvent,
        theme: &WidgetTheme,
        cx: &mut Context<Self>,
    ) -> bool {
        match self.page {
            Page::Accordion => {
                if let Some(action) = self.accordion(theme).action(event) {
                    match action {
                        AccordionAction::Change(ids) => self.choices = ids.into_iter().collect(),
                        AccordionAction::Focus(id) => {
                            cx.request_focus(self.accordion(theme).trigger_key(&id))
                        }
                    }
                    return true;
                }
            }
            Page::Toggle => {
                if let Some(value) = Toggle::new("bold", "Bold", self.open).action(event) {
                    self.open = value;
                    self.status = if value {
                        "Bold enabled"
                    } else {
                        "Bold disabled"
                    }
                    .into();
                    return true;
                }
            }
            Page::ToggleGroup => {
                if let Some(action) = self.toggles().action(event) {
                    match action {
                        ToggleGroupAction::Change(ids) => self.choices = ids.into_iter().collect(),
                        ToggleGroupAction::Focus(id) => {
                            cx.request_focus(self.toggles().item_key(&id));
                            self.active = Some(id);
                        }
                    }
                    return true;
                }
            }
            Page::NavigationMenu => {
                if let Some(action) = self.navigation_menu(theme).action(event) {
                    match action {
                        NavigationMenuAction::Activate(id) => self.status = format!("Opened {id}"),
                        NavigationMenuAction::Open { id, focus_panel } => {
                            if focus_panel {
                                cx.request_focus(format!(
                                    "{}::content",
                                    self.navigation_menu(theme).item_key(&id)
                                ));
                            }
                            self.active = Some(id);
                        }
                        NavigationMenuAction::Close => {
                            if let Some(id) = self.active.take() {
                                cx.request_focus(self.navigation_menu(theme).item_key(&id));
                            }
                        }
                        NavigationMenuAction::Focus(id) => {
                            cx.request_focus(self.navigation_menu(theme).item_key(&id))
                        }
                    }
                    return true;
                }
                if ButtonBehavior::new("guide-start", "Open getting started")
                    .action(event)
                    .is_some()
                {
                    self.status = "Getting started: create a window and add a Button.".into();
                    self.active = None;
                    return true;
                }
            }
            Page::Sidebar => {
                if let Some(SidebarAction::SetCollapsed(value)) = self.sidebar(theme).action(event)
                {
                    self.collapsed = value;
                    return true;
                }
                if matches!(event.kind, UiEventKind::Click(_))
                    && let Some(page) = event
                        .target_key()
                        .and_then(|key| key.strip_prefix("workspace-"))
                {
                    self.status = format!("Opened {page}");
                    return true;
                }
            }
            Page::ButtonGroup => {
                if matches!(event.kind, UiEventKind::Click(_))
                    && let Some(action) = event
                        .target_key()
                        .and_then(|key| key.strip_prefix("group-"))
                {
                    self.status = format!("Action: {action}");
                    return true;
                }
            }
            _ => {}
        }
        false
    }
}
