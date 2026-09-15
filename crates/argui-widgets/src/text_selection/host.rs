use super::TextSelectionToolbar;
use crate::{TablerIcon, WidgetAssets, shadcn};
use argui_core::{Rect, Size};
use argui_paint::{Filter, ImageAsset, VectorAsset};
use argui_runtime::{Context, Entity, LayoutSnapshot, Render, WindowEnvironment};
use argui_ui::{
    Element, ElementKind, EventType, SelectionCapabilities, SelectionCommand, UiEvent, UiEventKind,
};
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
    icons: WidgetAssets,
    presence: crate::Presence,
    closing: Option<ActiveSelection>,
    backdrop: Option<Filter>,
}

impl<A: Render> SelectionHost<A> {
    /// Wraps a render application with text-selection toolbar presentation.
    #[must_use]
    pub fn new(application: A) -> Self {
        Self {
            application: Entity::new(application),
            active: None,
            viewport: Size::default(),
            menu_key: "argui::selection-menu".into(),
            icons: WidgetAssets::tabler_subset(
                argui_core::Color::WHITE,
                [
                    TablerIcon::Cut,
                    TablerIcon::Copy,
                    TablerIcon::Paste,
                    TablerIcon::SelectAll,
                ],
            ),
            presence: crate::Presence::default().fade_in(false),
            closing: None,
            backdrop: None,
        }
    }

    #[must_use]
    /// Sets the key used to identify the selection toolbar menu.
    pub fn menu_key(mut self, key: impl Into<String>) -> Self {
        self.menu_key = key.into();
        self
    }

    #[must_use]
    /// Sets the filter applied behind the selection toolbar.
    pub fn backdrop_filter(mut self, filter: Filter) -> Self {
        self.backdrop = Some(filter);
        self
    }
}

impl<A: Render> Render for SelectionHost<A> {
    fn animation_frame(&mut self, frame: argui_animation::Frame, cx: &mut Context<Self>) {
        if self.presence.advance(frame.elapsed) {
            self.closing = None;
            cx.notify();
        } else {
            cx.request_paint();
        }
        if self.presence.animating() {
            cx.request_animation_frame();
        }
    }

    fn wants_animation_frame(&self) -> bool {
        self.presence.animating()
    }

    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let environment = cx.environment();
        if environment.reduced_motion {
            self.presence.set_open(self.active.is_some(), true);
            self.closing = None;
        }
        let application = cx.entity(&self.application);
        let mut root = self.compose(application, environment);
        for event in EventType::ALL {
            root = root.on(cx
                .listener(event, |host, event, cx| host.handle_event(event, cx))
                .capture(event != EventType::ContextMenu));
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
        let mut assets = self.application.read(Render::vector_assets);
        assets.extend_from_slice(self.icons.assets());
        assets
    }

    fn inspector(&self) -> Option<argui_inspect::InspectorHandle> {
        self.application.read(Render::inspector)
    }
}

impl<A: Render> SelectionHost<A> {
    fn compose(&self, mut root: Element, environment: WindowEnvironment) -> Element {
        if let Some(selection) = self.active.as_ref().or(self.closing.as_ref()) {
            let theme = shadcn(&environment);
            let mut toolbar = TextSelectionToolbar::new(
                &self.menu_key,
                selection.bounds,
                self.viewport,
                selection.capabilities,
            )
            .icons(&self.icons);
            if let Some(filter) = &self.backdrop {
                toolbar = toolbar.backdrop_filter(filter.clone());
            }
            let toolbar = toolbar.build(theme.resolve(environment.color_scheme));
            let toolbar = self.presence.decorate(toolbar);
            if matches!(root.kind, ElementKind::Container) {
                root.children.push(toolbar);
            } else {
                root = Element::container([root, toolbar]);
            }
        }
        root
    }
    fn handle_event(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        let previous = self.active.clone();
        match &event.kind {
            UiEventKind::PointerOutside(_) | UiEventKind::DismissRequested
                if event.target_key() == Some(self.menu_key.as_str()) =>
            {
                if self.active.take().is_some() {
                    cx.notify();
                }
            }
            UiEventKind::KeyInput(input)
                if input.key == argui_core::Key::Escape
                    && input.state == argui_core::KeyState::Pressed =>
            {
                if self.active.take().is_some() {
                    cx.notify();
                }
            }
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
                // Layout refreshes may report the same mouse selection after ContextMenu.
                // They must not turn a newly opened context menu into an exiting touch menu.
                if !*touch
                    && !*dragging
                    && self
                        .active
                        .as_ref()
                        .is_some_and(|active| active.target.is_some())
                {
                    return;
                }
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
            } if !event.default_prevented() => {
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
                } else if self.active.take().is_some() {
                    cx.notify();
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
        if previous.is_some() != self.active.is_some() {
            self.presence
                .set_open(self.active.is_some(), cx.environment().reduced_motion);
            self.closing = if self.active.is_none() && self.presence.visible() {
                previous
            } else {
                None
            };
            if self.presence.animating() {
                cx.request_animation_frame();
            }
        }
    }

    fn command_for(&self, event: &UiEvent) -> Option<SelectionCommand> {
        self.active.as_ref()?;
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
