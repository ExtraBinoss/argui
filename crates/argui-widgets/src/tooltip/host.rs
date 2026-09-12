use std::time::Duration;

use argui_core::{PointerKind, PointerPhase};
use argui_paint::{ImageAsset, LayerStyle, PaintStyle, VectorAsset};
use argui_runtime::{
    Context, Entity, LayoutSnapshot, Render,
    tasks::{self, TaskSlot},
};
use argui_ui::{Display, Element, ElementKind, EventType, Role, UiEvent, UiEventKind, WindowLayer};
use web_time::Instant;

use super::{Tooltip, TooltipState};

/// Install once around an application to present help declared by buttons or `Element::tooltip`.
/// A menu, popover or modal suspends automatic help until the panel closes.
pub struct TooltipHost<A: Render> {
    application: Entity<A>,
    root: Element,
    active: Option<(String, TooltipState)>,
    blocked: bool,
    timer: TaskSlot,
    origin: Instant,
    delay: Duration,
    paint: Option<PaintStyle>,
    layer: Option<LayerStyle>,
}

impl<A: Render> TooltipHost<A> {
    #[must_use]
    pub fn new(application: A) -> Self {
        Self::from_entity(Entity::new(application))
    }

    #[must_use]
    pub fn from_entity(application: Entity<A>) -> Self {
        Self {
            application,
            root: Element::container([]),
            active: None,
            blocked: false,
            timer: TaskSlot::default(),
            origin: Instant::now(),
            delay: Duration::from_millis(350),
            paint: None,
            layer: None,
        }
    }

    #[must_use]
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    #[must_use]
    pub fn paint(mut self, paint: PaintStyle) -> Self {
        self.paint = Some(paint);
        self
    }

    /// Customize the surface with any foreground/backdrop filter or registered effect.
    #[must_use]
    pub fn layer(mut self, layer: LayerStyle) -> Self {
        self.layer = Some(layer);
        self
    }

    fn clear(&mut self) {
        self.timer.cancel();
        self.active = None;
    }

    fn schedule(&mut self, cx: &mut Context<Self>) {
        self.timer.cancel();
        let Some(deadline) = self
            .active
            .as_ref()
            .and_then(|(_, state)| state.next_deadline())
        else {
            return;
        };
        if cx
            .spawn_latest(
                &mut self.timer,
                tasks::sleep(deadline.saturating_sub(self.origin.elapsed())),
                |host, _, cx| {
                    if let Some((_, state)) = &mut host.active {
                        state.advance(host.origin.elapsed());
                    }
                    host.schedule(cx);
                    cx.notify();
                },
            )
            .is_err()
        {
            // Detached applications without a task service still provide immediate help.
            if let Some((_, state)) = &mut self.active {
                state.advance(deadline);
            }
        }
    }

    fn event(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        if self.blocked {
            return;
        }
        let entering = matches!(event.kind, UiEventKind::Focused)
            || matches!(&event.kind, UiEventKind::Pointer(pointer)
                if pointer.phase == PointerPhase::Entered && pointer.kind != PointerKind::Touch);
        if entering
            && let Some(key) = event.target_key()
            && candidate(&self.root, key).is_some()
            && self.active.as_ref().is_none_or(|(active, _)| active != key)
        {
            self.clear();
            self.active = Some((key.into(), TooltipState::new(key).delay(self.delay)));
        }
        if let Some((_, state)) = &mut self.active
            && state.update(event, self.origin.elapsed())
        {
            self.schedule(cx);
            cx.notify();
        }
    }
}

impl<A: Render> Render for TooltipHost<A> {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let mut root = cx.entity(&self.application);
        if !root.ptr_eq(&self.root) {
            self.blocked = has_popup(&root);
            if self.blocked
                || self
                    .active
                    .as_ref()
                    .is_some_and(|(key, _)| candidate(&root, key).is_none())
            {
                self.clear();
            }
            self.root = root.clone();
        }
        if let Some((key, state)) = &self.active
            && state.is_open()
            && let Some(description) = candidate(&root, key)
        {
            let environment = cx.environment();
            let themes = crate::shadcn(environment.primary);
            let mut tooltip = Tooltip::new(key, description, true, Element::container([]));
            if let Some(paint) = &self.paint {
                tooltip = tooltip.paint(paint.clone());
            }
            if let Some(layer) = &self.layer {
                tooltip = tooltip.layer(layer.clone());
            }
            let panel = tooltip
                .build(themes.resolve(environment.color_scheme))
                .children[1]
                .clone();
            mount_tooltip(&mut root, key, panel);
        }
        for kind in [
            EventType::PointerEnter,
            EventType::PointerLeave,
            EventType::PointerDown,
            EventType::PointerCancel,
            EventType::Focus,
            EventType::Blur,
            EventType::Click,
            EventType::Key,
        ] {
            root = root.on(cx.listener(kind, Self::event).capture(true));
        }
        root
    }

    fn layout_changed(&mut self, layout: &LayoutSnapshot, cx: &mut Context<Self>) {
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

fn hidden(element: &Element) -> bool {
    element.semantic_hidden || element.style.display == Display::None
}

fn has_popup(element: &Element) -> bool {
    if hidden(element) {
        return false;
    }
    if element.portal.as_ref().is_some_and(|portal| {
        matches!(portal.layer, WindowLayer::Popover | WindowLayer::Modal)
            && (element.focus_scope.is_some()
                || portal.dismiss == argui_ui::DismissPolicy::OutsidePointer
                || element.semantics.as_ref().is_some_and(|semantics| {
                    matches!(
                        semantics.role,
                        Role::Tooltip
                            | Role::Menu
                            | Role::Dialog
                            | Role::ListBox
                            | Role::AlertDialog
                    )
                }))
    }) {
        return true;
    }
    element.children.iter().any(has_popup)
}

fn candidate<'a>(element: &'a Element, key: &str) -> Option<&'a str> {
    if hidden(element) {
        return None;
    }
    if element.key.as_deref() == Some(key) {
        if element.semantics.as_ref().is_some_and(|semantics| {
            semantics.state.disabled
                || semantics.state.busy
                || semantics.state.expanded == Some(true)
        }) {
            return None;
        }
        return element
            .tooltip
            .as_deref()
            .filter(|description| !description.is_empty());
    }
    element
        .children
        .iter()
        .find_map(|child| candidate(child, key))
}

fn describe(element: &mut Element, key: &str, content_key: &str) -> bool {
    if element.key.as_deref() == Some(key) {
        element
            .semantics
            .get_or_insert_with(|| argui_ui::Semantics::new(Role::Group));
        *element = element.clone().described_by([content_key]);
        return true;
    }
    element
        .children
        .iter_mut()
        .any(|child| describe(child, key, content_key))
}

fn scope_path(element: &Element, key: &str) -> Option<Vec<usize>> {
    if element.key.as_deref() == Some(key) {
        return Some(Vec::new());
    }
    for (index, child) in element.children.iter().enumerate() {
        if let Some(mut path) = scope_path(child, key) {
            if child.semantic_scope || !path.is_empty() {
                path.insert(0, index);
            }
            return Some(path);
        }
    }
    None
}

fn mount_tooltip(root: &mut Element, key: &str, panel: Element) {
    let path = scope_path(root, key).expect("the tooltip trigger is mounted");
    let mut scope = root;
    for index in path {
        scope = &mut scope.children[index];
    }
    describe(
        scope,
        key,
        panel.key.as_deref().expect("tooltip content key"),
    );
    if matches!(scope.kind, ElementKind::Container) {
        scope.children.push(panel);
    } else {
        let semantic_scope = scope.semantic_scope;
        scope.semantic_scope = false;
        let mut wrapper = Element::container([scope.clone(), panel]);
        wrapper.semantic_scope = semantic_scope;
        *scope = wrapper;
    }
}
