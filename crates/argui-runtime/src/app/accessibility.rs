use argui_accessibility::{
    Role, SemanticAction, SemanticNode, SemanticNodeId, SemanticRequest, SemanticTree, Semantics,
};
use argui_core::{Point, Rect, Size};
use argui_layout::LayoutOutput;
use argui_ui::{FocusRequest, InteractionUpdate, UiEventKind};
#[cfg(not(target_arch = "wasm32"))]
use winit::{event_loop::ActiveEventLoop, window::Window};

#[cfg(target_arch = "wasm32")]
use crate::RuntimeEvent;

use super::Application;

#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, Mutex};

#[cfg(not(target_arch = "wasm32"))]
use accesskit::ActivationHandler;
#[cfg(not(target_arch = "wasm32"))]
use argui_accessibility::AccessKitTree;

#[cfg(not(target_arch = "wasm32"))]
struct InitialTree {
    snapshot: Arc<Mutex<SemanticTree>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl ActivationHandler for InitialTree {
    fn request_initial_tree(&mut self) -> Option<accesskit::TreeUpdate> {
        self.snapshot
            .lock()
            .ok()
            .map(|snapshot| AccessKitTree::full(&snapshot))
    }
}

impl Application {
    fn apply_accessibility_action(
        &mut self,
        request: SemanticRequest,
        window: &dyn crate::host::WindowHost,
        event_loop: &dyn crate::host::LoopControl,
    ) {
        let Some(update) = self
            .ui_tree
            .as_mut()
            .and_then(|ui| accessibility_action_update(ui, self.ui_layout.as_ref(), request))
        else {
            return;
        };
        self.apply_ui_update(update, window, event_loop);
        self.sync_accessibility();
    }

    fn current_semantic_tree(&self) -> SemanticTree {
        semantic_tree(
            self.ui_tree.as_ref(),
            self.ui_layout.as_ref(),
            self.viewport,
            self.scale_factor,
        )
    }
}

fn accessibility_action_update(
    ui: &mut argui_ui::UiTree,
    layout: Option<&LayoutOutput>,
    request: SemanticRequest,
) -> Option<InteractionUpdate> {
    let target = ui
        .node_ids()
        .iter()
        .copied()
        .find(|node| node.get() == request.target.get())?;
    let semantics = ui.element_for(target)?.semantics.as_ref()?;
    if semantics.state.disabled || !semantics.actions.contains(&request.action) {
        return None;
    }
    if matches!(request.action, SemanticAction::Focus | SemanticAction::Blur) {
        return Some(layout.map_or_else(InteractionUpdate::default, |layout| {
            ui.sync_focus(
                &layout.hit_regions,
                Some(if request.action == SemanticAction::Focus {
                    FocusRequest::Focus(target.into())
                } else {
                    FocusRequest::Clear
                }),
            )
        }));
    }
    if request.action == SemanticAction::SetValue
        && ui.text_input_value(target).is_some()
        && let Some(argui_accessibility::SemanticValue::Text(value)) = &request.value
    {
        return Some(ui.replace_text_input(target, value));
    }
    let kind = if request.action == SemanticAction::Click {
        UiEventKind::Click(argui_ui::ClickEvent::accessibility())
    } else {
        UiEventKind::SemanticAction {
            action: request.action,
            value: request.value,
        }
    };
    Some(InteractionUpdate {
        events: ui.event_deliveries(target, kind),
        ..InteractionUpdate::default()
    })
}

fn semantic_tree(
    ui: Option<&argui_ui::UiTree>,
    layout: Option<&LayoutOutput>,
    viewport: Size,
    scale_factor: f32,
) -> SemanticTree {
    if let (Some(ui), Some(layout)) = (ui, layout) {
        return ui.semantic_tree(&layout.semantic_bounds, scale_factor);
    }
    let root = SemanticNodeId::new(1);
    SemanticTree {
        root,
        focus: root,
        nodes: vec![SemanticNode {
            id: root,
            bounds: Rect::new(
                Point::default(),
                Size::new(
                    viewport.width * scale_factor,
                    viewport.height * scale_factor,
                ),
            ),
            semantics: Semantics::new(Role::Window),
            children: Vec::new(),
        }],
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg_attr(coverage_nightly, coverage(off))]
impl Application {
    pub(super) fn initialize_accessibility(
        &mut self,
        event_loop: &ActiveEventLoop,
        window: &Window,
    ) {
        let Some(crate::host::EventProxy::Winit(proxy)) = self.event_proxy.clone() else {
            return;
        };
        let snapshot = Arc::new(Mutex::new(self.current_semantic_tree()));
        let adapter = accesskit_winit::Adapter::with_mixed_handlers(
            event_loop,
            window,
            InitialTree {
                snapshot: Arc::clone(&snapshot),
            },
            proxy,
        );
        self.semantic_snapshot = Some(snapshot);
        self.accessibility = Some(adapter);
    }

    pub(super) fn sync_accessibility(&mut self) {
        let next = self.current_semantic_tree();
        let (Some(snapshot), Some(adapter)) =
            (self.semantic_snapshot.as_ref(), self.accessibility.as_mut())
        else {
            return;
        };
        let Ok(mut current) = snapshot.lock() else {
            return;
        };
        let patch = current.diff(&next);
        if patch.is_empty() {
            return;
        }
        *current = next.clone();
        adapter.update_if_active(|| AccessKitTree::patch(&patch, &next));
    }

    /// Refreshes accessibility when semantic content, focus, or bounds changed.
    ///
    /// `presentation_only` indicates that only compositor presentation changed.
    /// `focus_unchanged` indicates that focus processing made no semantic change.
    pub(super) fn sync_accessibility_if_needed(
        &mut self,
        presentation_only: bool,
        focus_unchanged: bool,
    ) {
        let (Some(snapshot), Some(layout)) =
            (self.semantic_snapshot.as_ref(), self.ui_layout.as_ref())
        else {
            self.sync_accessibility();
            return;
        };
        let Ok(current) = snapshot.lock() else {
            return;
        };
        let needs_sync = super::semantic_sync::needs_semantic_sync(
            &current,
            layout
                .semantic_bounds
                .iter()
                .map(|(node, rect)| (node.get(), *rect)),
            self.scale_factor,
            presentation_only,
            focus_unchanged,
        );
        drop(current);
        if needs_sync {
            self.sync_accessibility();
        }
    }

    pub(super) fn accessibility_event(
        &mut self,
        event: accesskit_winit::Event,
        window: &dyn crate::host::WindowHost,
        event_loop: &dyn crate::host::LoopControl,
    ) {
        if Some(crate::host::HostId::Winit(event.window_id)) != self.window_id() {
            return;
        }
        if let accesskit_winit::WindowEvent::ActionRequested(request) = event.window_event
            && let Ok(action) = SemanticRequest::try_from(request)
        {
            self.apply_accessibility_action(action, window, event_loop);
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl Application {
    pub(crate) fn initialize_web_accessibility(&mut self) -> Result<(), String> {
        use winit::platform::web::WindowExtWebSys;

        let window = self
            .window
            .as_deref()
            .and_then(crate::host::WindowHost::winit)
            .ok_or_else(|| "window is not initialized".to_owned())?;
        let canvas = window
            .canvas()
            .ok_or_else(|| "window has no HTML canvas".to_owned())?;
        let snapshot = self.current_semantic_tree();
        let proxy = self
            .event_proxy
            .clone()
            .ok_or_else(|| "event loop proxy is not initialized".to_owned())?;
        let window_key = self.window_key.clone();
        self.dom_accessibility = Some(
            argui_accessibility::DomTree::new(canvas, snapshot, move |request| {
                let _ = proxy.send_event(crate::event::UserEvent::Accessibility {
                    window: window_key.clone(),
                    request,
                });
            })
            .map_err(|error| format!("failed to create accessibility DOM: {error:?}"))?,
        );
        Ok(())
    }

    pub(super) fn sync_accessibility(&mut self) {
        let next = self.current_semantic_tree();
        let error = self
            .dom_accessibility
            .as_mut()
            .and_then(|dom| dom.sync(next).err());
        if let (Some(ui), Some(dom)) = (&self.ui_tree, &self.dom_accessibility) {
            for &node in ui.node_ids() {
                if ui.text_input_protected(node)
                    && let Some(value) = ui.text_input_value(node)
                {
                    dom.set_protected_value(SemanticNodeId::new(node.get()), value);
                }
            }
        }
        if let Some(error) = error {
            (self.on_event)(RuntimeEvent::CommandFailed(format!(
                "failed to update accessibility DOM: {error:?}"
            )));
        }
    }

    pub(super) fn web_accessibility_action(
        &mut self,
        request: SemanticRequest,
        window: &dyn crate::host::WindowHost,
        event_loop: &dyn crate::host::LoopControl,
    ) {
        self.apply_accessibility_action(request, window, event_loop);
    }
}
