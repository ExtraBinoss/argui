use argui_runtime::{Context, Entity, EventEmitter, Render};
use argui_ui::Element;
use std::{cell::Cell, rc::Rc};

struct Child;
impl EventEmitter<()> for Child {}
impl Render for Child {
    fn render(&mut self, _: &mut Context<Self>) -> Element {
        Element::text("child")
    }
}
struct Parent(Entity<Child>, bool);
impl Render for Parent {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        if self.1 {
            let element = self.0.render();
            cx.route_events_to(self.0.erase());
            element
        } else {
            cx.entity(&self.0)
        }
    }
}

#[test]
fn rendered_and_routed_children_inherit_host_wakes_without_duplicate_rebinding() {
    for routed in [false, true] {
        let child = Entity::new(Child);
        let parent = Entity::new(Parent(child.clone(), routed));
        let wakes = Rc::new(Cell::new(0));
        let observed = wakes.clone();
        parent
            .runtime()
            .set_wake(move || observed.set(observed.get() + 1));
        let _ = parent.render();
        let mut subscription = None;
        child.update(|_, cx| {
            subscription = Some(
                cx.subscribe(&child, |_, _: &(), cx| cx.emit(()).unwrap())
                    .unwrap(),
            );
            cx.emit(()).unwrap();
        });
        assert_eq!(wakes.get(), 1);
        parent.update(|_, cx| cx.notify());
        assert_eq!(wakes.get(), 2); // The parent domain requests its own rebuild.
        let _ = parent.render();
        assert_eq!(wakes.get(), 2);
        child.runtime().dispatch_pending();
        assert_eq!(wakes.get(), 3);
        drop(subscription);
        child.runtime().dispatch_pending();
        assert_eq!(child.runtime().pending_events(), 0);
        assert_eq!(wakes.get(), 3);
    }
}

#[test]
fn completed_model_invalidation_still_wakes_an_idle_host_once() {
    let runtime = argui_runtime::ModelRuntime::default();
    let value = runtime.entity(0);
    let wakes = Rc::new(Cell::new(0));
    let observed = wakes.clone();
    runtime.set_wake(move || observed.set(observed.get() + 1));
    for expected in 1..=2 {
        for _ in 0..3 {
            value.update(|value, cx| {
                *value += 1;
                cx.notify();
            });
        }
        assert_eq!(wakes.get(), expected);
        assert_eq!(runtime.pending_invalidations(), 0);
        runtime.dispatch_pending();
        runtime.dispatch_pending();
        assert_eq!(wakes.get(), expected);
    }
    assert_eq!(value.read(|value| *value), 6);
}

#[test]
fn window_visibility_preserves_local_visibility_and_routed_mount_identity() {
    use argui_platform::{PlatformEvent, WindowKey};
    use argui_runtime::{AppModel, SingleWindowModel};
    let model = Entity::new(Child);
    let mut app = SingleWindowModel::from_entity(model.clone()).unwrap();
    let key = WindowKey::main();
    let original = app.view(&key, Default::default()).unwrap();
    let router = app.event_router(&key).unwrap();
    for _ in 0..10 {
        app.update(&argui_runtime::AppEvent::Window {
            window: key.clone(),
            event: PlatformEvent::VisibilityChanged(false),
        });
        assert_ne!(app.view(&key, Default::default()).unwrap(), original);
        assert!(!app.wants_animation_frame(&key));
        app.update(&argui_runtime::AppEvent::Window {
            window: key.clone(),
            event: PlatformEvent::VisibilityChanged(true),
        });
        assert_eq!(app.view(&key, Default::default()).unwrap(), original);
        assert!(router.ptr_eq(&app.event_router(&key).unwrap()));
    }
    app.update(&argui_runtime::AppEvent::Window {
        window: key.clone(),
        event: PlatformEvent::Closed,
    });
    assert_eq!(model.resources().resource_count(), 0);
    router.close_presentation();
}

#[test]
fn closing_routed_presentations_delivers_final_events_once_across_domains() {
    use argui_runtime::{MountTransition, shutdown_presentations};
    let child = Entity::new(Child);
    let parent = Entity::new(Parent(child.clone(), true));
    let events = Rc::new(std::cell::RefCell::new(Vec::new()));
    let captured = events.clone();
    let _child_events = child
        .runtime()
        .observe_mounts(move |event| captured.borrow_mut().push(event));
    let captured = events.clone();
    let _parent_events = parent
        .runtime()
        .observe_mounts(move |event| captured.borrow_mut().push(event));
    let _ = parent.render();
    parent.runtime().dispatch_pending();
    child.runtime().dispatch_pending();
    events.borrow_mut().clear();
    shutdown_presentations(&[parent.erase()]);
    assert!(
        events
            .borrow()
            .iter()
            .all(|event| event.transition == MountTransition::Unmounted)
    );
    // Each domain preserves order; cross-domain delivery is not a global ordering guarantee.
    assert_eq!(events.borrow().len(), 2);
    assert_eq!(parent.runtime().pending_mount_events(), 0);
    assert_eq!(child.runtime().pending_mount_events(), 0);
    assert!(child.render().children.is_empty());
    shutdown_presentations(&[parent.erase()]);
    assert_eq!(events.borrow().len(), 2);
}

#[test]
fn removed_child_domain_is_still_drained_by_its_live_host() {
    struct Conditional(Entity<Child>, bool);
    impl Render for Conditional {
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            if self.1 {
                cx.entity(&self.0)
            } else {
                Element::text("removed")
            }
        }
    }
    let child = Entity::new(Child);
    let events = Rc::new(Cell::new(0));
    let captured = events.clone();
    let _subscription = child
        .runtime()
        .observe_mounts(move |_| captured.set(captured.get() + 1));
    let parent = Entity::new(Conditional(child, true)).mount().unwrap();
    parent.render(Default::default()).unwrap();
    parent.dispatch_models();
    assert_eq!(events.get(), 1);
    parent
        .update(|parent, cx| {
            parent.1 = false;
            cx.notify();
        })
        .unwrap();
    parent.render(Default::default()).unwrap();
    parent.dispatch_models();
    assert_eq!(events.get(), 2);
}

#[cfg(all(feature = "tasks", not(target_arch = "wasm32")))]
#[test]
fn application_shutdown_cancels_queued_results_but_preserves_external_models_and_services() {
    use argui_platform::WindowKey;
    use argui_runtime::{
        AppModel, ModelRuntime, SingleWindowModel, shutdown_presentations,
        tasks::{TaskError, TaskRuntime},
    };
    let runtime = ModelRuntime::default();
    let model = runtime.entity(0usize);
    let service = runtime.register_service(String::from("shared")).unwrap();
    let (send, wake) = std::sync::mpsc::channel();
    let tasks = TaskRuntime::new(move || {
        let _ = send.send(());
    });
    runtime.set_task_runtime(tasks.clone());
    struct View(Entity<usize>);
    impl Render for View {
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            Element::text(cx.read(&self.0, |value| value.to_string()))
        }
    }
    let view = runtime.entity(View(model.clone()));
    let app = SingleWindowModel::from_entity(view.clone()).unwrap();
    let key = WindowKey::main();
    app.view(&key, Default::default()).unwrap();
    let router = app.event_router(&key).unwrap();
    let handle = model.update(|_, cx| {
        cx.spawn(async { 17 }, |value, result, cx| {
            *value = result.unwrap();
            cx.notify();
        })
        .unwrap()
    });
    wake.recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();
    shutdown_presentations(&[router]);
    tasks.drain();
    assert_eq!(tasks.pending(), 0);
    assert_eq!(model.read(|value| *value), 0);
    assert!(handle.cancellation_token().is_cancelled());
    assert_eq!(view.resources().resource_count(), 0);
    assert!(runtime.service::<String>().is_some());
    assert!(matches!(
        model.update(|_, cx| cx.spawn(async {}, |_, _, _| {})),
        Err(TaskError::Unavailable)
    ));
    model.update(|value, cx| {
        *value = 23;
        cx.notify();
    });
    assert_eq!(model.read(|value| *value), 23);
    drop(service);
    assert!(runtime.service::<String>().is_none());
}

#[test]
fn application_shutdown_finishes_other_roots_when_cleanup_panics() {
    let first = Entity::new(Child).mount().unwrap();
    let second = Entity::new(Child).mount().unwrap();
    let _lease = first
        .resources()
        .defer(|| panic!("presentation cleanup"))
        .unwrap();
    let closed = Rc::new(Cell::new(false));
    let captured = closed.clone();
    let _second_lease = second
        .resources()
        .defer(move || captured.set(true))
        .unwrap();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        argui_runtime::shutdown_presentations(&[first.erase(), second.erase()]);
    }));
    assert!(result.is_err());
    assert!(first.resources().is_closed());
    assert!(second.resources().is_closed());
    assert!(closed.get());
    argui_runtime::shutdown_presentations(&[first.erase(), second.erase()]);
}

#[test]
fn host_show_does_not_reveal_a_locally_hidden_mount() {
    let mount = Entity::new(Child).mount().unwrap();
    let id = mount.id();
    let host = mount.erase();
    host.set_host_visible(false);
    mount.set_visible(false).unwrap();
    host.set_host_visible(true);
    assert!(!mount.is_visible());
    mount.set_visible(true).unwrap();
    assert!(mount.is_visible());
    assert_eq!(mount.id(), id);
}

#[test]
fn measured_fanout_rebuilds_dependents_and_returns_to_idle() {
    use argui_runtime::{AppModel, ModelRuntime, SingleWindowModel};
    struct Counter {
        data: Entity<usize>,
        renders: Rc<Cell<usize>>,
    }
    impl Render for Counter {
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            self.renders.set(self.renders.get() + 1);
            Element::text(cx.read(&self.data, |data| data.to_string()))
        }
    }
    let runtime = ModelRuntime::default();
    let data = runtime.entity(0);
    let unrelated = runtime.entity(0);
    let renders = Rc::new(Cell::new(0));
    let unrelated_renders = Rc::new(Cell::new(0));
    let model = runtime.entity(Counter {
        data: data.clone(),
        renders: renders.clone(),
    });
    let other = SingleWindowModel::from_entity(runtime.entity(Counter {
        data: unrelated,
        renders: unrelated_renders.clone(),
    }))
    .unwrap();
    let windows: Vec<_> = (0..16)
        .map(|_| SingleWindowModel::from_entity(model.clone()).unwrap())
        .collect();
    let key = argui_platform::WindowKey::main();
    for window in &windows {
        window.view(&key, Default::default()).unwrap();
    }
    other.view(&key, Default::default()).unwrap();
    let memory_before = std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status
                .lines()
                .find(|line| line.starts_with("VmRSS:"))
                .map(str::to_owned)
        });
    let start = web_time::Instant::now();
    for value in 1..=1000 {
        data.update(|data, cx| {
            *data = value;
            cx.notify();
        });
        while runtime.pending_invalidations() > 0 {
            runtime.dispatch_pending();
        }
        for window in &windows {
            window.view(&key, Default::default()).unwrap();
        }
        other.view(&key, Default::default()).unwrap();
    }
    assert_eq!(renders.get(), 16 * 1001);
    assert_eq!(unrelated_renders.get(), 1);
    for window in &windows {
        window.view(&key, Default::default()).unwrap();
    }
    assert_eq!(renders.get(), 16 * 1001);
    drop(windows);
    assert_eq!(model.resources().resource_count(), 0);
    assert_eq!(runtime.pending_events(), 0);
    assert_eq!(runtime.pending_invalidations(), 0);
    let memory_after = std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status
                .lines()
                .find(|line| line.starts_with("VmRSS:"))
                .map(str::to_owned)
        });
    eprintln!(
        "16 views × 1,000 mutations: {:?}; RSS before={memory_before:?}, after={memory_after:?}; unrelated view rendered once",
        start.elapsed()
    );
}
