use argui_accessibility::{
    Role, SemanticAction, SemanticNode, SemanticNodeId, SemanticRequest, SemanticTree, Semantics,
};
use argui_core::{Point, Rect, Size};
use argui_layout::LayoutOutput;
use argui_ui::{FocusRequest, InteractionUpdate, UiEventKind};
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
        window: &Window,
        event_loop: &ActiveEventLoop,
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
    let kind = if request.action == SemanticAction::Click {
        UiEventKind::Clicked
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
        let bounds = layout
            .nodes
            .iter()
            .map(|node| {
                let bounds = node.clip.map_or(node.bounds, |clip| {
                    node.bounds.intersection(clip).unwrap_or_default()
                });
                (node.node, bounds)
            })
            .collect::<Vec<_>>();
        return ui.semantic_tree(&bounds, scale_factor);
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
        let Some(proxy) = self.event_proxy.clone() else {
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

    pub(super) fn accessibility_event(
        &mut self,
        event: accesskit_winit::Event,
        window: &Window,
        event_loop: &ActiveEventLoop,
    ) {
        if event.window_id != window.id() {
            return;
        }
        if let accesskit_winit::WindowEvent::ActionRequested(request) = event.window_event
            && let Ok(action) = SemanticRequest::try_from(request)
        {
            self.apply_accessibility_action(action, window, event_loop);
        }
    }
}

#[cfg(test)]
mod tests {
    use argui_accessibility::{SemanticAction, SemanticNodeId, SemanticRequest, SemanticValue};
    use argui_core::{Affine2D, Point, Rect, Size};
    use argui_layout::{LayoutNode, LayoutOutput};
    use argui_paint::ClipChain;
    use argui_ui::{
        CursorIcon, Element, GestureSet, HitRegion, Interaction, Role, Semantics, UiEventKind,
        UiTree,
    };

    use super::{accessibility_action_update, semantic_tree};

    #[test]
    fn actions_resolve_stable_nodes_focus_clicks_and_values() {
        let mut ui = UiTree::new(
            Element::container([])
                .keyed("target")
                .interaction(Interaction::default().focusable(true))
                .semantics(Semantics::new(Role::Button)),
        );
        let target = ui.node_id_at(0).unwrap();
        let mut layout = LayoutOutput::default();
        layout.hit_regions.push(HitRegion {
            node: target,
            bounds: Rect::new(Point::default(), Size::new(100.0, 40.0)),
            transform: Affine2D::IDENTITY,
            clips: ClipChain::default(),
            shape: argui_ui::HitShape::Bounds,
            slop: argui_ui::HitTestStyle::default().slop,
            enabled: true,
            focusable: true,
            cursor: CursorIcon::Auto,
            gestures: GestureSet::NONE,
            window_drag: None,
        });
        let request = |action, value| SemanticRequest {
            target: SemanticNodeId::new(target.get()),
            action,
            value,
        };

        let focus = accessibility_action_update(
            &mut ui,
            Some(&layout),
            request(SemanticAction::Focus, None),
        )
        .unwrap();
        assert!(
            focus
                .events
                .iter()
                .any(|event| event.kind == UiEventKind::Focused)
        );
        let click =
            accessibility_action_update(&mut ui, None, request(SemanticAction::Click, None))
                .unwrap();
        assert_eq!(click.events[0].kind, UiEventKind::Clicked);
        let value = accessibility_action_update(
            &mut ui,
            None,
            request(
                SemanticAction::SetValue,
                Some(SemanticValue::Text("new".into())),
            ),
        )
        .unwrap();
        assert!(matches!(
            value.events[0].kind,
            UiEventKind::SemanticAction {
                action: SemanticAction::SetValue,
                ..
            }
        ));
        assert!(
            accessibility_action_update(
                &mut ui,
                None,
                SemanticRequest {
                    target: SemanticNodeId::new(999),
                    action: SemanticAction::Click,
                    value: None,
                }
            )
            .is_none()
        );
    }

    #[test]
    fn semantic_snapshots_clip_bounds_and_have_a_window_fallback() {
        let ui = UiTree::new(Element::text("visible"));
        let node = ui.node_id_at(0).unwrap();
        let mut layout = LayoutOutput {
            nodes: vec![LayoutNode {
                index: 0,
                node,
                bounds: Rect::new(Point::new(10.0, 10.0), Size::new(50.0, 30.0)),
                layout_bounds: Rect::default(),
                clip: Some(Rect::new(Point::new(20.0, 0.0), Size::new(20.0, 30.0))),
                text_index: None,
            }],
            ..LayoutOutput::default()
        };
        let snapshot = semantic_tree(Some(&ui), Some(&layout), Size::default(), 2.0);
        assert_eq!(snapshot.nodes[0].bounds.origin, Point::new(40.0, 20.0));
        assert_eq!(snapshot.nodes[0].bounds.size, Size::new(40.0, 40.0));

        layout.nodes[0].clip = Some(Rect::new(Point::new(100.0, 100.0), Size::new(2.0, 2.0)));
        assert_eq!(
            semantic_tree(Some(&ui), Some(&layout), Size::default(), 1.0).nodes[0].bounds,
            Rect::default()
        );
        layout.nodes[0].clip = None;
        assert_eq!(
            semantic_tree(Some(&ui), Some(&layout), Size::default(), 1.0).nodes[0].bounds,
            layout.nodes[0].bounds
        );

        let fallback = semantic_tree(None, None, Size::new(320.0, 200.0), 1.5);
        assert_eq!(fallback.nodes[0].semantics.role, Role::Window);
        assert_eq!(fallback.nodes[0].bounds.size, Size::new(480.0, 300.0));
    }
}

#[cfg(target_arch = "wasm32")]
impl Application {
    pub(crate) fn initialize_web_accessibility(&mut self) -> Result<(), String> {
        use winit::platform::web::WindowExtWebSys;

        let window = self
            .window
            .as_deref()
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
        if let Some(error) = error {
            (self.on_event)(RuntimeEvent::CommandFailed(format!(
                "failed to update accessibility DOM: {error:?}"
            )));
        }
    }

    pub(super) fn web_accessibility_action(
        &mut self,
        request: SemanticRequest,
        window: &Window,
        event_loop: &ActiveEventLoop,
    ) {
        self.apply_accessibility_action(request, window, event_loop);
    }
}
