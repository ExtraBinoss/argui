use argui_runtime::{Context, Entity, Render, ScopeClosed};
use argui_ui::{ClickEvent, Element, ElementKind, EventType, UiEventKind, UiTree};
use std::{cell::Cell, rc::Rc};

#[derive(Default)]
struct View {
    value: usize,
    renders: usize,
    frames: usize,
    layouts: usize,
    clicks: usize,
}
impl Render for View {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        self.renders += 1;
        Element::text(self.value.to_string()).on(cx.listener(EventType::Click, |view, _, _| {
            view.clicks += 1;
        }))
    }
    fn wants_animation_frame(&self) -> bool {
        true
    }
    fn animation_frame(&mut self, _: argui_animation::Frame, _: &mut Context<Self>) {
        self.frames += 1;
    }
    fn layout_changed(&mut self, _: &argui_runtime::LayoutSnapshot, _: &mut Context<Self>) {
        self.layouts += 1;
    }
}

#[test]
fn hidden_mount_retains_identity_and_resources_and_resumes_with_current_data() {
    let model = Entity::new(View::default());
    let mount = model.mount().unwrap();
    let id = mount.id();
    let released = Rc::new(Cell::new(false));
    let mark = released.clone();
    let _lease = mount.resources().defer(move || mark.set(true)).unwrap();
    let resources = mount.resources().resource_count();
    let mut tree = UiTree::new(mount.render(Default::default()).unwrap());
    let events = tree.event_deliveries(
        tree.node_ids()[0],
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    for value in 1..=10 {
        mount.set_visible(false).unwrap();
        mount.set_visible(false).unwrap();
        assert!(!mount.is_visible());
        model.update(|view, cx| {
            view.value = value;
            cx.notify();
        });
        let _ = mount.render(Default::default()).unwrap();
        mount.layout_changed(&Default::default()).unwrap();
        mount
            .animation_frame(argui_animation::Frame {
                now: argui_animation::Time::ZERO,
                elapsed: argui_animation::Duration::ZERO,
            })
            .unwrap();
        for event in &events {
            mount.dispatch_event(event).unwrap();
        }
        assert_eq!(
            model.read(|view| (view.renders, view.layouts, view.frames, view.clicks)),
            (value, 0, 0, 0)
        );
        assert_eq!(mount.resources().resource_count(), resources);
        assert!(!released.get());
        mount.set_visible(true).unwrap();
        mount.set_visible(true).unwrap();
        assert!(mount.is_visible());
        let shown = mount.render(Default::default()).unwrap();
        assert!(
            matches!(&shown.kind, ElementKind::Text { content, .. } if content.as_str() == value.to_string())
        );
        assert!(mount.render(Default::default()).unwrap().ptr_eq(&shown));
        assert_eq!(mount.id(), id);
    }
    mount.close();
    assert!(released.get());
    assert!(!mount.is_visible());
    assert_eq!(mount.set_visible(true), Err(ScopeClosed));
    assert!(model.mount().unwrap().is_visible());
}

struct Parent {
    child: Entity<View>,
    visible: bool,
    mounted: bool,
}
impl Render for Parent {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        if self.mounted {
            cx.entity_visible(&self.child, self.visible)
        } else {
            Element::container([])
        }
    }
}

#[test]
fn hiding_a_child_retains_its_mount_without_invalidating_the_hidden_parent() {
    let child = Entity::new(View::default());
    let parent = Entity::new(Parent {
        child: child.clone(),
        visible: true,
        mounted: true,
    });
    let first = parent.render();
    parent.update(|parent, cx| {
        parent.visible = false;
        cx.notify();
    });
    let hidden = parent.render();
    assert_eq!(child.resources().resource_count(), 1);
    child.update(|view, cx| {
        view.value = 9;
        cx.notify();
    });
    assert!(parent.render().ptr_eq(&hidden));
    assert_eq!(child.read(|view| view.renders), 1);
    parent.update(|parent, cx| {
        parent.visible = true;
        cx.notify();
    });
    let shown = parent.render();
    assert!(matches!(&shown.kind, ElementKind::Text { content, .. } if content.as_str() == "9"));
    assert_eq!(first.event_listeners[0], shown.event_listeners[0]);
    parent.update(|parent, cx| {
        parent.mounted = false;
        cx.notify();
    });
    let _ = parent.render();
    assert_eq!(child.resources().resource_count(), 0);
    parent.update(|parent, cx| {
        parent.mounted = true;
        cx.notify();
    });
    let remounted = parent.render();
    assert_ne!(shown.event_listeners[0], remounted.event_listeners[0]);
}

#[cfg(all(feature = "tasks", not(target_arch = "wasm32")))]
#[test]
fn hiding_keeps_view_tasks_alive_until_unmount() {
    let tasks = argui_runtime::tasks::TaskRuntime::new(|| {});
    let model = Entity::new(View::default());
    model.set_task_runtime(tasks.clone());
    let mount = model.mount().unwrap();
    let handle = mount
        .update(|_, cx| {
            cx.spawn(std::future::pending::<()>(), |_, _, _| {})
                .unwrap()
        })
        .unwrap();
    mount.set_visible(false).unwrap();
    assert!(!handle.cancellation_token().is_cancelled());
    mount.set_visible(true).unwrap();
    assert!(!handle.cancellation_token().is_cancelled());
    mount.close();
    assert!(handle.cancellation_token().is_cancelled());
    tasks.shutdown();
}

#[cfg(all(feature = "tasks", not(target_arch = "wasm32")))]
#[test]
fn a_thousand_visibility_close_and_reopen_cycles_release_all_owned_work() {
    use argui_runtime::{EventEmitter, ModelRuntime, ResourceScope, tasks::TaskRuntime};
    use std::{sync::mpsc, time::Duration};
    struct Source;
    impl EventEmitter<()> for Source {}
    struct Tracked(Rc<()>, usize);
    impl Render for Tracked {
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            let captured = self.0.clone();
            Element::text("tracked").on(cx.listener(EventType::Click, move |model, _, _| {
                let _retained = &captured;
                model.1 += 1;
            }))
        }
    }
    let (send, wake) = mpsc::channel();
    let tasks = TaskRuntime::new(move || {
        let _ = send.send(());
    });
    let runtime = ModelRuntime::default();
    runtime.set_task_runtime(tasks.clone());
    let source = runtime.entity(Source);
    let marker = Rc::new(());
    let model = runtime.entity(Tracked(marker.clone(), 0));
    let baseline = Rc::strong_count(&marker);
    let started = web_time::Instant::now();
    for cycle in 0..1_000 {
        let scope = ResourceScope::default();
        let service = runtime.register_service(String::from("shared")).unwrap();
        let service_weak = Rc::downgrade(&service.service());
        let _publication = scope.own(service).unwrap();
        let left = model.mount().unwrap();
        let right = model.mount().unwrap();
        let mut subscriptions = Vec::new();
        let mut handles = Vec::new();
        for mount in [&left, &right] {
            let _ = mount.render(Default::default()).unwrap();
            let captured = marker.clone();
            subscriptions.push(
                mount
                    .update(|_, cx| {
                        assert_eq!(cx.service::<String>().unwrap().as_str(), "shared");
                        cx.subscribe(&source, move |_, _: &(), _| {
                            let _retained = &captured;
                        })
                    })
                    .unwrap()
                    .unwrap(),
            );
            let captured = marker.clone();
            handles.push(
                mount
                    .update(|_, cx| {
                        cx.spawn(std::future::pending::<()>(), move |_, _, _| {
                            let _retained = &captured;
                            panic!("cancelled view result was delivered");
                        })
                    })
                    .unwrap()
                    .unwrap(),
            );
        }
        left.set_visible(false).unwrap();
        source.update(|_, cx| cx.emit(())).unwrap();
        assert!(
            subscriptions
                .iter()
                .all(|subscription| subscription.is_active())
        );
        assert!(
            handles
                .iter()
                .all(|handle| !handle.cancellation_token().is_cancelled())
        );
        left.set_visible(true).unwrap();
        let mut tree = UiTree::new(left.render(Default::default()).unwrap());
        let events = tree.event_deliveries(
            tree.node_ids()[0],
            UiEventKind::Click(ClickEvent::accessibility()),
        );
        runtime.transaction(|| {
            source.update(|_, cx| cx.emit(())).unwrap();
            left.close();
        });
        assert!(!subscriptions[0].is_active());
        assert!(subscriptions[1].is_active());
        assert!(runtime.service::<String>().is_some());
        let reopened = model.mount().unwrap();
        assert_ne!(left.id(), reopened.id());
        let _ = reopened.render(Default::default()).unwrap();
        for event in &events {
            reopened.dispatch_event(event).unwrap();
        }
        assert_eq!(model.read(|model| model.1), 0);
        right.close();
        reopened.close();
        left.close();
        assert!(
            subscriptions
                .iter()
                .all(|subscription| !subscription.is_active())
        );
        assert!(
            handles
                .iter()
                .all(|handle| handle.cancellation_token().is_cancelled())
        );
        while tasks.pending() != 0 {
            tasks.drain();
            if tasks.pending() != 0 {
                wake.recv_timeout(Duration::from_secs(5)).unwrap();
            }
        }
        runtime.dispatch_pending();
        for mount in [&left, &right, &reopened] {
            assert_eq!(mount.resources().resource_count(), 0);
        }
        let weak = reopened.downgrade();
        drop((left, right, reopened));
        assert!(weak.upgrade().is_none());
        scope.close();
        assert!(runtime.service::<String>().is_none());
        assert!(service_weak.upgrade().is_none());
        assert_eq!(source.resources().resource_count(), 0, "cycle {cycle}");
        assert_eq!(model.resources().resource_count(), 0, "cycle {cycle}");
        assert_eq!(Rc::strong_count(&marker), baseline, "cycle {cycle}");
        assert_eq!(runtime.pending_events(), 0);
        assert_eq!(runtime.pending_invalidations(), 0);
    }
    eprintln!("1,000 combined mount cycles: {:?}", started.elapsed());
    tasks.shutdown();
}

#[cfg(feature = "tasks")]
#[test]
fn hidden_child_task_updates_do_not_request_parent_frames() {
    use argui_runtime::{AppModel, SingleWindowModel};
    struct Worker;
    impl Render for Worker {
        fn render(&mut self, _: &mut Context<Self>) -> Element {
            Element::text("worker")
        }
        fn tasks_ready(&mut self, cx: &mut Context<Self>) {
            cx.notify();
        }
    }
    struct Container(Entity<Worker>);
    impl Render for Container {
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            cx.entity_visible(&self.0, false)
        }
    }
    let mut app = SingleWindowModel::new(Container(Entity::new(Worker)));
    let key = argui_platform::WindowKey::main();
    let _ = app.view(&key, Default::default()).unwrap();
    assert!(app.tasks_ready(&key).windows.is_empty());
    assert!(!app.wants_animation_frame(&key));
}

#[cfg(feature = "tasks")]
#[test]
fn closing_the_model_prevents_late_host_task_callbacks() {
    use argui_runtime::{AppModel, SingleWindowModel};
    struct Worker;
    impl Render for Worker {
        fn render(&mut self, _: &mut Context<Self>) -> Element {
            Element::text("worker")
        }
        fn tasks_ready(&mut self, _: &mut Context<Self>) {
            panic!("closed presentation callback");
        }
    }
    let model = Entity::new(Worker);
    let mut app = SingleWindowModel::from_entity(model.clone()).unwrap();
    let key = argui_platform::WindowKey::main();
    let _ = app.view(&key, Default::default()).unwrap();
    model.resources().close();
    assert!(app.tasks_ready(&key).windows.is_empty());
}
