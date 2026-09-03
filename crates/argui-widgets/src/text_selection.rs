use argui_core::{Rect, Size};
use argui_paint::{ImageAsset, VectorAsset};
use argui_runtime::{Context, LayoutSnapshot, Render};
use argui_ui::{
    Element, ElementKind, Interaction, SelectionCapabilities, SelectionCommand, Sides, UiEvent,
    UiEventKind, UserSelect, length,
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
                theme.button.clone(),
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
    application: A,
    active: Option<ActiveSelection>,
    viewport: Size,
    menu_key: String,
}

impl<A: Render> SelectionHost<A> {
    #[must_use]
    pub fn new(application: A) -> Self {
        Self {
            application,
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
        let mut child = cx.child_context();
        let mut root = self.application.render(&mut child);
        cx.propagate(child);
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

    fn event(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        match &event.kind {
            UiEventKind::Pressed if self.command_for(event).is_some() => {
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
            UiEventKind::Clicked if self.command_for(event).is_some() => {
                let command = self.command_for(event).unwrap();
                if let Some(target) = self.active.as_ref().and_then(|active| active.target) {
                    cx.selection_command_for(target, command);
                } else {
                    cx.selection_command(command);
                }
                self.active = None;
                cx.notify();
                event.stop_propagation();
                return;
            }
            _ => {}
        }
        let mut child = cx.child_context();
        self.application.event(event, &mut child);
        cx.propagate(child);
    }

    fn animation_frame(&mut self, frame: argui_animation::Frame, cx: &mut Context<Self>) {
        let mut child = cx.child_context();
        self.application.animation_frame(frame, &mut child);
        cx.propagate(child);
    }

    fn wants_animation_frame(&self) -> bool {
        self.application.wants_animation_frame()
    }

    fn layout_changed(&mut self, layout: &LayoutSnapshot, cx: &mut Context<Self>) {
        self.viewport = layout.viewport_size();
        let mut child = cx.child_context();
        self.application.layout_changed(layout, &mut child);
        cx.propagate(child);
    }

    fn image_assets(&self) -> Vec<ImageAsset> {
        self.application.image_assets()
    }

    fn vector_assets(&self) -> Vec<VectorAsset> {
        self.application.vector_assets()
    }

    fn inspector(&self) -> Option<argui_inspect::InspectorHandle> {
        self.application.inspector()
    }
}

impl<A: Render> SelectionHost<A> {
    fn command_for(&self, event: &UiEvent) -> Option<SelectionCommand> {
        let suffix = event
            .current_key()?
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

#[cfg(test)]
mod tests {
    use argui_core::{ColorScheme, Point};
    use argui_runtime::WindowEnvironment;

    use super::*;

    struct App {
        container: bool,
    }

    impl Render for App {
        fn render(&mut self, _cx: &mut Context<Self>) -> Element {
            if self.container {
                Element::container([Element::text("select me")])
            } else {
                Element::text("select me")
            }
        }
    }

    fn node() -> argui_ui::NodeId {
        argui_ui::UiTree::new(Element::container([]))
            .node_id_at(0)
            .unwrap()
    }

    #[test]
    fn host_composes_a_real_copy_button_for_touch_selection() {
        let mut host = SelectionHost::new(App { container: false });
        let mut cx = Context::default();
        host.viewport = Size::new(320.0, 200.0);
        host.active = Some(ActiveSelection {
            bounds: Rect::new(Point::new(100.0, 80.0), Size::new(60.0, 20.0)),
            capabilities: SelectionCapabilities {
                copy: true,
                select_all: true,
                ..SelectionCapabilities::default()
            },
            target: None,
        });
        let root = host.render(&mut cx);
        assert!(matches!(root.kind, ElementKind::Container));
        assert_eq!(root.children.len(), 2);
        assert_eq!(
            WindowEnvironment::default().color_scheme,
            ColorScheme::Light
        );
    }

    #[test]
    fn host_tracks_completed_touch_selections_without_rebuilding_during_drag() {
        let mut host = SelectionHost::new(App { container: true }).menu_key("selection");
        let bounds = Rect::new(Point::new(40.0, 60.0), Size::new(50.0, 18.0));
        let selection = |text, bounds, touch, dragging| {
            UiEvent::new(
                node(),
                None,
                UiEventKind::DocumentSelectionChanged {
                    text,
                    bounds,
                    touch,
                    dragging,
                },
            )
        };

        let mut cx = Context::default();
        host.event(
            &selection(Some("selected".into()), Some(bounds), true, false),
            &mut cx,
        );
        assert_eq!(cx.view_update(), argui_runtime::ViewUpdate::Rebuild);
        assert!(host.active.as_ref().unwrap().capabilities.copy);
        host.viewport = Size::new(240.0, 160.0);
        let root = host.render(&mut Context::default());
        assert!(matches!(root.kind, ElementKind::Container));
        assert_eq!(root.children.len(), 2);

        let mut unchanged = Context::default();
        host.event(
            &selection(Some("selected".into()), Some(bounds), true, false),
            &mut unchanged,
        );
        assert_eq!(unchanged.view_update(), argui_runtime::ViewUpdate::None);

        let mut dragging = Context::default();
        host.event(
            &selection(Some("selected".into()), Some(bounds), true, true),
            &mut dragging,
        );
        assert!(host.active.is_none());
        assert_eq!(dragging.view_update(), argui_runtime::ViewUpdate::Rebuild);

        host.event(
            &selection(Some("mouse".into()), Some(bounds), false, false),
            &mut Context::default(),
        );
        host.event(
            &selection(None, Some(bounds), true, false),
            &mut Context::default(),
        );
        host.event(
            &selection(Some("missing bounds".into()), None, true, false),
            &mut Context::default(),
        );
        assert!(host.active.is_none());
    }

    #[test]
    fn selection_commands_are_emitted_by_menu_items() {
        let mut host = SelectionHost::new(App { container: false }).menu_key("selection");
        let clicked = UiEvent::new(node(), Some("selection::copy".into()), UiEventKind::Clicked);
        host.event(&clicked, &mut Context::default());
        host.active = Some(ActiveSelection {
            bounds: Rect::default(),
            capabilities: SelectionCapabilities {
                copy: true,
                ..SelectionCapabilities::default()
            },
            target: None,
        });
        let mut cx = Context::default();
        host.event(&clicked, &mut cx);
        assert_eq!(cx.view_update(), argui_runtime::ViewUpdate::Rebuild);

        let unrelated = UiEvent::new(node(), Some("other".into()), UiEventKind::Clicked);
        host.event(&unrelated, &mut Context::default());
    }

    #[test]
    fn command_keys_cover_the_complete_editing_menu() {
        let host = SelectionHost::new(App { container: false }).menu_key("selection");
        for (suffix, expected) in [
            ("cut", SelectionCommand::Cut),
            ("copy", SelectionCommand::Copy),
            ("paste", SelectionCommand::Paste),
            ("select-all", SelectionCommand::SelectAll),
        ] {
            let event = UiEvent::new(
                node(),
                Some(format!("selection::{suffix}")),
                UiEventKind::Clicked,
            );
            assert_eq!(host.command_for(&event), Some(expected));
        }
        assert_eq!(
            host.command_for(&UiEvent::new(
                node(),
                Some("selection::unknown".into()),
                UiEventKind::Clicked,
            )),
            None
        );
        assert_eq!(
            host.command_for(&UiEvent::new(node(), None, UiEventKind::Clicked)),
            None
        );
    }

    #[test]
    fn context_menu_opens_only_for_available_commands_and_targets_the_editor() {
        let mut host = SelectionHost::new(App { container: false }).menu_key("selection");
        let target = node();
        let empty = UiEvent::new(
            target,
            None,
            UiEventKind::ContextMenu {
                position: Point::new(40.0, 30.0),
                capabilities: SelectionCapabilities::default(),
            },
        );
        host.event(&empty, &mut Context::default());
        assert!(host.active.is_none());
        assert!(!empty.default_prevented());

        let unrelated_press = UiEvent::new(target, Some("other".into()), UiEventKind::Pressed);
        host.event(&unrelated_press, &mut Context::default());
        assert!(!unrelated_press.default_prevented());

        let editable = UiEvent::new(
            target,
            None,
            UiEventKind::ContextMenu {
                position: Point::new(40.0, 30.0),
                capabilities: SelectionCapabilities {
                    editable: true,
                    paste: true,
                    ..SelectionCapabilities::default()
                },
            },
        );
        host.event(&editable, &mut Context::default());
        assert_eq!(host.active.as_ref().unwrap().target, Some(target));
        assert!(editable.default_prevented());

        let pressed = UiEvent::new(
            target,
            Some("selection::paste".into()),
            UiEventKind::Pressed,
        );
        host.event(&pressed, &mut Context::default());
        assert!(pressed.default_prevented());

        let clicked = UiEvent::new(
            target,
            Some("selection::paste".into()),
            UiEventKind::Clicked,
        );
        let mut cx = Context::default();
        host.event(&clicked, &mut cx);
        assert!(host.active.is_none());
        assert!(clicked.propagation_stopped());
        assert_eq!(cx.view_update(), argui_runtime::ViewUpdate::Rebuild);
    }

    #[test]
    fn each_context_menu_capability_can_open_the_menu_independently() {
        for capabilities in [
            SelectionCapabilities {
                cut: true,
                ..SelectionCapabilities::default()
            },
            SelectionCapabilities {
                copy: true,
                ..SelectionCapabilities::default()
            },
            SelectionCapabilities {
                select_all: true,
                ..SelectionCapabilities::default()
            },
        ] {
            let mut host = SelectionHost::new(App { container: false });
            let event = UiEvent::new(
                node(),
                None,
                UiEventKind::ContextMenu {
                    position: Point::default(),
                    capabilities,
                },
            );
            host.event(&event, &mut Context::default());
            assert!(host.active.is_some());
            assert!(event.default_prevented());
        }
    }

    #[test]
    fn inactive_host_and_editable_toolbar_cover_both_composition_modes() {
        let mut host = SelectionHost::new(App { container: false });
        assert!(matches!(
            host.render(&mut Context::default()).kind,
            ElementKind::Text { .. }
        ));

        let palette = shadcn(argui_core::Color::rgb(0.2, 0.4, 0.8));
        let theme = palette.resolve(ColorScheme::Light);
        let toolbar = TextSelectionToolbar::new(
            "editor",
            Rect::default(),
            Size::new(400.0, 200.0),
            SelectionCapabilities {
                editable: true,
                ..SelectionCapabilities::default()
            },
        )
        .build(theme);
        assert_eq!(toolbar.children.len(), 4);
    }

    #[test]
    fn toolbar_prefers_the_space_above_a_low_selection() {
        let palette = shadcn(argui_core::Color::rgb(0.2, 0.4, 0.8));
        let theme = palette.resolve(ColorScheme::Light);
        let toolbar = TextSelectionToolbar::new(
            "copy",
            Rect::new(Point::new(100.0, 140.0), Size::new(30.0, 12.0)),
            Size::new(400.0, 220.0),
            SelectionCapabilities {
                copy: true,
                ..SelectionCapabilities::default()
            },
        )
        .build(theme);
        assert_eq!(toolbar.children.len(), 2);
        assert_eq!(toolbar.style.inset.top, length(92.0));
    }
}
