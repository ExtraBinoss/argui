use argui_core::{Point, PointerEvent, PointerId, PointerPhase, Rect, Size};
use argui_runtime::{
    Context, Entity, LayoutBounds, LayoutSnapshot, Render, ThemeRequest, ViewUpdate,
    WindowEnvironment,
};
use argui_ui::{Element, EventType, UiEventKind, UiTree};

#[path = "model/cache.rs"]
mod cache;
#[cfg(feature = "tasks")]
#[path = "model/context.rs"]
mod context;
#[path = "model/dispatch.rs"]
mod dispatch;
#[path = "model/entity_api.rs"]
mod effect;
#[path = "model/entity.rs"]
mod entity;
#[path = "model/events.rs"]
mod events;
#[path = "model/handler.rs"]
mod handler;
#[path = "model/host.rs"]
mod host;
#[path = "model/lifecycle.rs"]
mod lifecycle;
#[path = "model/mount.rs"]
mod mount;
#[path = "model/observation.rs"]
mod observation;
#[path = "model/presentation.rs"]
mod presentation;
#[path = "model/scope.rs"]
mod scope;
#[path = "model/services.rs"]
mod services;
#[path = "model/signal.rs"]
mod signal;
#[path = "model/visibility.rs"]
mod visibility;

#[test]
fn frames_visit_only_active_children_and_stop_after_their_last_frame() {
    use argui_animation::{Duration, Frame, Time};
    use argui_platform::WindowKey;
    use argui_runtime::{AppModel, SingleWindowModel};
    use std::{cell::Cell, rc::Rc};
    struct Child {
        active: bool,
        calls: Rc<Cell<usize>>,
    }
    impl Render for Child {
        fn render(&mut self, _: &mut Context<Self>) -> Element {
            Element::container([])
        }
        fn wants_animation_frame(&self) -> bool {
            self.active
        }
        fn animation_frame(&mut self, _: Frame, cx: &mut Context<Self>) {
            self.calls.set(self.calls.get() + 1);
            self.active = false;
            cx.request_paint();
        }
    }
    struct Parent {
        children: [Entity<Child>; 2],
        active: Rc<Cell<bool>>,
    }

    impl Render for Parent {
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            Element::column(self.children.iter().map(|child| cx.entity(child)))
        }
        fn wants_animation_frame(&self) -> bool {
            self.active.get()
        }
    }
    let inactive_calls = Rc::new(Cell::new(0));
    let active_calls = Rc::new(Cell::new(0));
    let parent_active = Rc::new(Cell::new(false));
    let mut model = SingleWindowModel::new(Parent {
        children: [
            Entity::new(Child {
                active: false,
                calls: inactive_calls.clone(),
            }),
            Entity::new(Child {
                active: true,
                calls: active_calls.clone(),
            }),
        ],
        active: parent_active.clone(),
    });
    let window = WindowKey::main();
    assert!(model.view(&window, WindowEnvironment::default()).is_some());
    assert!(model.wants_animation_frame(&window));
    model.animation_frame(
        &window,
        Frame {
            now: Time::ZERO,
            elapsed: Duration::from_millis(16),
        },
    );
    assert_eq!(inactive_calls.get(), 0);
    assert_eq!(active_calls.get(), 1);
    assert!(!model.wants_animation_frame(&window));
    parent_active.set(true);
    assert!(model.wants_animation_frame(&window));
    model.animation_frame(
        &window,
        Frame {
            now: Time::ZERO,
            elapsed: Duration::ZERO,
        },
    );
    assert_eq!(inactive_calls.get(), 0);
    assert_eq!(active_calls.get(), 1);
}

#[derive(Default)]
struct Counter(u32);

impl Render for Counter {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        Element::text(self.0.to_string())
            .keyed("increment")
            .on(cx.listener(EventType::Click, |counter, _event, cx| {
                counter.0 += 1;
                cx.notify();
            }))
    }
}

#[test]
fn apps_rebuild_only_when_their_state_changes() {
    let app = Entity::new(Counter::default());
    let mut tree = UiTree::new(app.render());
    let target = tree.node_id_at(0).unwrap();
    assert!(
        tree.event_deliveries(
            target,
            UiEventKind::Pointer(PointerEvent::mouse(PointerPhase::Entered, Point::default())),
        )
        .is_empty()
    );
    for event in tree.event_deliveries(
        target,
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    ) {
        app.dispatch_event(&event);
    }
    assert!(
        matches!(&app.render().kind, argui_ui::ElementKind::Text { content, .. } if content.as_str() == "1")
    );
}

#[test]
fn layout_snapshots_expose_viewport_and_keyed_logical_bounds() {
    let tree = UiTree::new(Element::container([]).keyed("anchor"));
    let node = tree.node_id_at(0).unwrap();
    let bounds = Rect::new(Point::new(20.0, 30.0), Size::new(80.0, 40.0));
    let snapshot = LayoutSnapshot {
        viewport: Rect::new(Point::default(), Size::new(640.0, 480.0)),
        nodes: vec![LayoutBounds {
            node,
            key: Some("anchor".into()),
            retained_identity: None,
            bounds,
        }],
    };

    assert_eq!(snapshot.bounds("anchor"), Some(bounds));
    assert_eq!(snapshot.bounds("missing"), None);
    assert_eq!(snapshot.viewport_size(), Size::new(640.0, 480.0));
    let mut app = Counter::default();
    let mut cx = Context::default();
    app.layout_changed(&snapshot, &mut cx);
    assert_eq!(cx.view_update(), ViewUpdate::None);
}

struct PlainRender;

impl Render for PlainRender {
    fn render(&mut self, _cx: &mut Context<Self>) -> Element {
        Element::container([])
    }
}

#[test]
fn render_defaults_are_noop_and_do_not_publish_assets_or_inspection() {
    let mut render = PlainRender;
    let mut context = Context::default();
    render.animation_frame(
        argui_animation::Frame {
            now: argui_animation::Time::ZERO,
            elapsed: argui_animation::Duration::from_millis(16),
        },
        &mut context,
    );
    render.layout_changed(&LayoutSnapshot::default(), &mut context);

    assert!(!render.wants_animation_frame());
    assert!(render.image_assets().is_empty());
    assert!(render.vector_assets().is_empty());
    assert!(render.effect_definitions().is_empty());
    assert!(render.inspector().is_none());
    assert_eq!(context.view_update(), ViewUpdate::None);
}

#[test]
fn context_exposes_environment_children_and_invalidation_without_internal_state() {
    let mut context = Context::<PlainRender>::default();
    assert_eq!(context.environment(), WindowEnvironment::default());
    assert!(!context.capture_pointer(PointerId::MOUSE));
    assert!(!context.release_pointer(PointerId::MOUSE));

    context.request_paint();
    assert_eq!(context.view_update(), ViewUpdate::Paint);
    context.request_animation_frame();
    assert_eq!(context.view_update(), ViewUpdate::Paint);
    context.set_theme(ThemeRequest::default());
    assert_eq!(context.view_update(), ViewUpdate::Rebuild);
    context.route_events_to(Entity::new(PlainRender).erase());
}

#[test]
#[should_panic(expected = "child views require a retained parent context")]
fn detached_context_cannot_own_a_child_mount() {
    let mut context = Context::<PlainRender>::default();
    let child = context.new_entity(PlainRender);
    let _ = context.entity(&child);
}

mod entity_tests {
    use std::{cell::Cell, rc::Rc};

    use argui_ui::{Element, ElementKind, EventType, UiEvent, UiEventKind, UiTree};

    use argui_core::{ColorScheme, Point, Rect, Size};

    use argui_runtime::{Context, Entity, LayoutBounds, LayoutSnapshot, Render, WindowEnvironment};

    struct Counted {
        renders: Rc<Cell<u32>>,
    }

    impl Render for Counted {
        fn render(&mut self, _cx: &mut Context<Self>) -> Element {
            self.renders.set(self.renders.get() + 1);
            Element::text("retained")
        }
    }

    #[test]
    fn clean_entity_returns_the_exact_cached_subtree() {
        let renders = Rc::new(Cell::new(0));
        let entity = Entity::new(Counted {
            renders: renders.clone(),
        });
        let first = entity.render();
        let second = entity.render();
        assert!(first.ptr_eq(&second));
        assert_eq!(renders.get(), 1);
    }

    #[test]
    fn notify_rebuilds_only_on_the_next_render() {
        let renders = Rc::new(Cell::new(0));
        let entity = Entity::new(Counted {
            renders: renders.clone(),
        });
        let first = entity.render();
        entity.update(|_, cx| cx.notify());
        assert_eq!(renders.get(), 1);
        let second = entity.render();
        assert!(!first.ptr_eq(&second));
        assert_eq!(renders.get(), 2);
    }

    struct Child {
        events: Rc<Cell<u32>>,
        stop: bool,
    }

    impl Render for Child {
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            Element::text("child").keyed("deep-child").on(cx.listener(
                EventType::Click,
                |child, event, cx| {
                    child.events.set(child.events.get() + 1);
                    cx.notify();
                    if child.stop {
                        event.stop_propagation();
                    }
                },
            ))
        }
    }

    struct Parent {
        child: Entity<Child>,
        events: Rc<Cell<u32>>,
    }

    impl Render for Parent {
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            Element::column([cx.entity(&self.child)]).on(cx.listener(
                EventType::Click,
                |parent, _event, cx| {
                    parent.events.set(parent.events.get() + 1);
                    cx.notify();
                },
            ))
        }
    }

    struct Routed(Entity<Child>);
    impl Render for Routed {
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            let element = self.0.render();
            cx.route_events_to(self.0.erase());
            element
        }
    }

    fn unkeyed_event() -> UiEvent {
        UiEvent::new(
            UiTree::new(Element::container([])).node_id_at(0).unwrap(),
            None,
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        )
    }

    fn dispatch_click<T: Render>(root: &Entity<T>, key: &str) -> Vec<UiEvent> {
        let mut tree = UiTree::new(root.render());
        let target = tree
            .node_ids()
            .iter()
            .copied()
            .find(|node| tree.key(*node) == Some(key))
            .unwrap();
        let deliveries = tree.event_deliveries(
            target,
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        );
        for event in &deliveries {
            if event.should_dispatch() {
                root.dispatch_event(event);
            }
        }
        deliveries
    }

    #[test]
    fn listener_identity_routes_directly_to_the_owning_entity() {
        let child_events = Rc::new(Cell::new(0));
        let parent_events = Rc::new(Cell::new(0));
        let root = Entity::new(Parent {
            child: Entity::new(Child {
                events: child_events.clone(),
                stop: false,
            }),
            events: parent_events.clone(),
        });
        let deliveries = dispatch_click(&root, "deep-child");
        assert_eq!(child_events.get(), 1);
        assert_eq!(parent_events.get(), 1);
        assert_eq!(deliveries.len(), 2);
    }

    #[test]
    fn propagation_control_lives_on_the_dispatched_event() {
        let child_events = Rc::new(Cell::new(0));
        let parent_events = Rc::new(Cell::new(0));
        let root = Entity::new(Parent {
            child: Entity::new(Child {
                events: child_events.clone(),
                stop: true,
            }),
            events: parent_events.clone(),
        });
        let deliveries = dispatch_click(&root, "deep-child");
        assert_eq!(child_events.get(), 1);
        assert_eq!(parent_events.get(), 0);
        assert!(deliveries[0].propagation_stopped());
    }

    #[test]
    fn routed_subtrees_dispatch_through_the_declared_event_owner() {
        let events = Rc::new(Cell::new(0));
        let root = Entity::new(Routed(Entity::new(Child {
            events: events.clone(),
            stop: false,
        })));
        let deliveries = dispatch_click(&root, "deep-child");
        assert_eq!(events.get(), 1);
        assert!(deliveries.iter().any(UiEvent::should_dispatch));
    }

    #[test]
    fn child_notification_invalidates_its_composed_ancestors() {
        let root = Entity::new(Parent {
            child: Entity::new(Child {
                events: Rc::new(Cell::new(0)),
                stop: false,
            }),
            events: Rc::new(Cell::new(0)),
        });
        let first = root.render();
        let child = root.read(|parent| parent.child.clone());
        child.update(|_, cx| cx.notify());
        let second = root.render();
        assert!(!first.ptr_eq(&second));
    }

    #[test]
    fn child_notifications_discard_observers_whose_parent_was_dropped() {
        let child = Entity::new(Child {
            events: Rc::new(Cell::new(0)),
            stop: false,
        });
        let root = Entity::new(Parent {
            child: child.clone(),
            events: Rc::new(Cell::new(0)),
        });
        let _ = root.render();
        drop(root);

        child.update(|_, cx| cx.notify());
        assert!(matches!(child.render().kind, ElementKind::Text { .. }));
    }

    #[test]
    fn weak_and_erased_entities_keep_explicit_identity() {
        let entity = Entity::new(Counted {
            renders: Rc::new(Cell::new(0)),
        });
        let weak = entity.downgrade();
        let erased = entity.erase();
        assert!(weak.upgrade().is_some());
        assert!(erased.ptr_eq(&entity.erase()));
        drop(entity);
        assert!(weak.upgrade().is_some());
        drop(erased);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn events_without_a_declared_listener_are_ignored() {
        let parent_events = Rc::new(Cell::new(0));
        let root = Entity::new(Parent {
            child: Entity::new(Child {
                events: Rc::new(Cell::new(0)),
                stop: false,
            }),
            events: parent_events.clone(),
        });
        let _ = root.render();
        root.dispatch_event(&unkeyed_event());
        assert_eq!(parent_events.get(), 0);
    }

    #[test]
    fn removed_event_route_drops_a_queued_child_delivery() {
        struct OptionalRoute {
            child: Entity<Child>,
            routed: bool,
        }
        impl Render for OptionalRoute {
            /// Publishes the child event route only while the child is visible.
            fn render(&mut self, cx: &mut Context<Self>) -> Element {
                if self.routed {
                    let child = self.child.render();
                    cx.route_events_to(self.child.erase());
                    child
                } else {
                    Element::container([])
                }
            }
        }
        let hits = Rc::new(Cell::new(0));
        let root = Entity::new(OptionalRoute {
            child: Entity::new(Child {
                events: hits.clone(),
                stop: false,
            }),
            routed: true,
        });
        let mut tree = UiTree::new(root.render());
        let event = tree
            .event_deliveries(
                tree.node_ids()[0],
                UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
            )
            .into_iter()
            .next()
            .unwrap();
        root.update(|model, cx| {
            model.routed = false;
            cx.notify();
        });
        tree.replace(root.render());
        root.dispatch_event(&event);
        assert_eq!(hits.get(), 0);
    }

    #[test]
    fn context_observes_stable_layout_bounds() {
        let context = Context::<super::Counter>::default();
        let bounds = Rect::new(Point::new(3.0, 5.0), Size::new(20.0, 30.0));
        let snapshot = LayoutSnapshot {
            viewport: Rect::default(),
            nodes: vec![LayoutBounds {
                node: UiTree::new(Element::container([])).node_id_at(0).unwrap(),
                key: Some("observed".into()),
                retained_identity: None,
                bounds,
            }],
        };
        assert_eq!(context.observe_bounds(&snapshot, "observed"), Some(bounds));
        assert_eq!(context.observe_bounds(&snapshot, "missing"), None);
    }

    #[test]
    fn environment_changes_rebuild_only_reading_subtrees() {
        struct EnvironmentReader {
            renders: Rc<Cell<u32>>,
            reads: bool,
        }
        impl Render for EnvironmentReader {
            fn render(&mut self, cx: &mut Context<Self>) -> Element {
                self.renders.set(self.renders.get() + 1);
                if self.reads {
                    Element::text(format!("{:?}", cx.environment().color_scheme))
                } else {
                    Element::text("static")
                }
            }
        }
        let dark = WindowEnvironment {
            color_scheme: ColorScheme::Dark,
            ..WindowEnvironment::default()
        };
        for (reads, expected) in [(false, 1), (true, 2)] {
            let renders = Rc::new(Cell::new(0));
            let entity = Entity::new(EnvironmentReader {
                renders: renders.clone(),
                reads,
            });
            let _ = entity.render_in(WindowEnvironment::default());
            let _ = entity.render_in(dark.clone());
            assert_eq!(renders.get(), expected);
        }
    }
}
