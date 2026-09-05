use argui_core::{Rect, Size};
use argui_paint::{ImageAsset, VectorAsset};
use argui_runtime::{Context, Entity, LayoutSnapshot, Render, WindowEnvironment};
use argui_ui::{
    Element, ElementKind, EventType, Interaction, SelectionCapabilities, SelectionCommand, Sides,
    UiEvent, UiEventKind, UserSelect, length,
};

use crate::{Button, WidgetTheme, shadcn};

const TOOLBAR_HEIGHT: f32 = 40.0;
const COMMAND_WIDTH: f32 = 88.0;
const VIEWPORT_MARGIN: f32 = 8.0;
const SELECTION_GAP: f32 = 8.0;

#[derive(Clone, Debug)]
pub struct TextSelectionToolbar {
    key_prefix: String,
    selection: Rect,
    viewport: Size,
    capabilities: SelectionCapabilities,
}

impl TextSelectionToolbar {
    #[must_use]
    pub fn new(
        key_prefix: impl Into<String>,
        selection: Rect,
        viewport: Size,
        capabilities: SelectionCapabilities,
    ) -> Self {
        Self {
            key_prefix: key_prefix.into(),
            selection,
            viewport,
            capabilities,
        }
    }

    #[must_use]
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let commands = [
            ("cut", "Cut", self.capabilities.cut),
            ("copy", "Copy", self.capabilities.copy),
            ("paste", "Paste", self.capabilities.paste),
            ("select-all", "Select all", self.capabilities.select_all),
        ]
        .into_iter()
        .filter(|(command, _, _)| {
            self.capabilities.editable || matches!(*command, "copy" | "select-all")
        })
        .collect::<Vec<_>>();
        let toolbar_width = COMMAND_WIDTH * commands.len().max(1) as f32;
        let left = (self.selection.origin.x + self.selection.size.width * 0.5
            - toolbar_width * 0.5)
            .clamp(
                VIEWPORT_MARGIN,
                (self.viewport.width - toolbar_width - VIEWPORT_MARGIN).max(VIEWPORT_MARGIN),
            );
        let above = self.selection.origin.y - TOOLBAR_HEIGHT - SELECTION_GAP;
        let top = if above >= VIEWPORT_MARGIN {
            above
        } else {
            (self.selection.origin.y + self.selection.size.height + SELECTION_GAP)
                .min((self.viewport.height - TOOLBAR_HEIGHT - VIEWPORT_MARGIN).max(VIEWPORT_MARGIN))
        };
        let buttons = commands.into_iter().map(|(suffix, label, enabled)| {
            Button::new(
                format!("{}::{suffix}", self.key_prefix),
                label,
                theme.button(),
            )
            .enabled(enabled)
            .build()
        });
        Element::row(buttons)
            .absolute(Sides {
                left: length(left),
                right: argui_ui::auto(),
                top: length(top),
                bottom: argui_ui::auto(),
            })
            .width(length(toolbar_width))
            .height(length(TOOLBAR_HEIGHT))
            .user_select(UserSelect::None)
            .interaction(Interaction::blocker())
            .z_index(i32::MAX)
    }
}

#[derive(Clone, Debug)]
struct ActiveSelection {
    bounds: Rect,
    capabilities: SelectionCapabilities,
    target: Option<argui_ui::NodeId>,
}

pub struct SelectionHost<A: Render> {
    application: Entity<A>,
    active: Option<ActiveSelection>,
    viewport: Size,
    menu_key: String,
}

impl<A: Render> SelectionHost<A> {
    #[must_use]
    pub fn new(application: A) -> Self {
        Self {
            application: Entity::new(application),
            active: None,
            viewport: Size::default(),
            menu_key: "argui::selection-menu".into(),
        }
    }

    #[must_use]
    pub fn menu_key(mut self, key: impl Into<String>) -> Self {
        self.menu_key = key.into();
        self
    }
}

impl<A: Render> Render for SelectionHost<A> {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let environment = cx.environment();
        let application = cx.entity(&self.application);
        let mut root = self.compose(application, environment);
        for event in EventType::ALL {
            root = root.on(cx
                .listener(event, |host, event, cx| host.handle_event(event, cx))
                .capture(true));
        }
        root
    }

    fn layout_changed(&mut self, layout: &LayoutSnapshot, cx: &mut Context<Self>) {
        self.viewport = layout.viewport_size();
        cx.layout_entity(&self.application, layout);
    }

    fn image_assets(&self) -> Vec<ImageAsset> {
        self.application.read(Render::image_assets)
    }

    fn vector_assets(&self) -> Vec<VectorAsset> {
        self.application.read(Render::vector_assets)
    }

    fn inspector(&self) -> Option<argui_inspect::InspectorHandle> {
        self.application.read(Render::inspector)
    }
}

impl<A: Render> SelectionHost<A> {
    fn compose(&self, mut root: Element, environment: WindowEnvironment) -> Element {
        if let Some(selection) = &self.active {
            let theme = shadcn(environment.primary);
            let toolbar = TextSelectionToolbar::new(
                &self.menu_key,
                selection.bounds,
                self.viewport,
                selection.capabilities,
            )
            .build(theme.resolve(environment.color_scheme));
            if matches!(root.kind, ElementKind::Container) {
                root.children.push(toolbar);
            } else {
                root = Element::container([root, toolbar]);
            }
        }
        root
    }
    fn handle_event(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        match &event.kind {
            UiEventKind::Pointer(argui_core::PointerEvent {
                phase: argui_core::PointerPhase::Pressed,
                ..
            }) if self.command_for(event).is_some() => {
                let _ = event.prevent_default();
            }
            UiEventKind::DocumentSelectionChanged {
                text,
                bounds,
                touch,
                dragging,
            } => {
                let next = (*touch && !*dragging)
                    .then(|| {
                        Some(ActiveSelection {
                            capabilities: SelectionCapabilities {
                                copy: text.as_ref().is_some_and(|value| !value.is_empty()),
                                select_all: true,
                                ..SelectionCapabilities::default()
                            },
                            bounds: (*bounds)?,
                            target: None,
                        })
                    })
                    .flatten();
                let changed = self
                    .active
                    .as_ref()
                    .map(|value| (value.capabilities, value.bounds))
                    != next
                        .as_ref()
                        .map(|value| (value.capabilities, value.bounds));
                self.active = next;
                if changed {
                    cx.notify();
                }
            }
            UiEventKind::ContextMenu {
                position,
                capabilities,
            } => {
                if capabilities.cut
                    || capabilities.copy
                    || capabilities.paste
                    || capabilities.select_all
                {
                    self.active = Some(ActiveSelection {
                        bounds: Rect::new(*position, Size::new(1.0, 1.0)),
                        capabilities: *capabilities,
                        target: Some(event.target),
                    });
                    cx.notify();
                    let _ = event.prevent_default();
                }
            }
            UiEventKind::Click(_) if self.command_for(event).is_some() => {
                let command = self.command_for(event).unwrap();
                if let Some(target) = self.active.as_ref().and_then(|active| active.target) {
                    cx.selection_command_for(target, command);
                } else {
                    cx.selection_command(command);
                }
                self.active = None;
                cx.notify();
                event.stop_propagation();
            }
            _ => {}
        }
    }

    fn command_for(&self, event: &UiEvent) -> Option<SelectionCommand> {
        let suffix = event
            .target_key()?
            .strip_prefix(self.menu_key.as_str())?
            .strip_prefix("::")?;
        match suffix {
            "cut" => Some(SelectionCommand::Cut),
            "copy" => Some(SelectionCommand::Copy),
            "paste" => Some(SelectionCommand::Paste),
            "select-all" => Some(SelectionCommand::SelectAll),
            _ => None,
        }
    }
}
