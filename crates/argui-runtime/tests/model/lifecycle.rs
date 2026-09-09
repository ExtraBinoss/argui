use argui_runtime::{Context, Entity, ModelRuntime, MountTransition, Render};
use argui_ui::Element;
use std::{cell::RefCell, rc::Rc};

#[test]
fn dropping_a_mount_with_failed_cleanup_preserves_its_model_and_final_event() {
    let runtime = ModelRuntime::new(|| {});
    let model = runtime.entity(View);
    let events = Rc::new(RefCell::new(Vec::new()));
    let log = events.clone();
    let _subscription =
        runtime.observe_mounts(move |event| log.borrow_mut().push(event.transition));
    let mount = model.mount().unwrap();
    let weak = mount.downgrade();
    let lease = mount.resources().defer(|| panic!("mount cleanup")).unwrap();
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(mount))).is_err());
    assert!(weak.upgrade().is_none());
    assert!(!lease.is_active());
    assert_eq!(model.resources().resource_count(), 0);
    runtime.dispatch_pending();
    assert_eq!(
        &*events.borrow(),
        &[MountTransition::Mounted, MountTransition::Unmounted]
    );
    assert!(model.mount().is_ok());
}

#[test]
fn shutdown_delivers_every_final_listener_even_after_multiple_panics() {
    for unwinding in [false, true] {
        let runtime = ModelRuntime::new(|| {});
        let _first = runtime.observe_mounts(|_| panic!("first listener"));
        let _second = runtime.observe_mounts(|_| panic!("second listener"));
        let delivered = Rc::new(RefCell::new(Vec::new()));
        let log = delivered.clone();
        let _last = runtime.observe_mounts(move |event| log.borrow_mut().push(event.transition));
        let model = runtime.entity(View);
        let mount = model.mount().unwrap();
        mount.close();
        struct Shutdown(ModelRuntime);
        impl Drop for Shutdown {
            fn drop(&mut self) {
                self.0.shutdown_host();
            }
        }
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if unwinding {
                let _shutdown = Shutdown(runtime.clone());
                panic!("outer failure");
            }
            runtime.shutdown_host();
        }));
        let payload = result.unwrap_err();
        assert_eq!(
            payload.downcast_ref::<&str>(),
            Some(&if unwinding {
                "outer failure"
            } else {
                "first listener"
            })
        );
        assert_eq!(
            &*delivered.borrow(),
            &[MountTransition::Mounted, MountTransition::Unmounted]
        );
        assert_eq!(runtime.pending_mount_events(), 0);
        runtime.shutdown_host();
    }
}
struct View;
impl Render for View {
    fn render(&mut self, _: &mut Context<Self>) -> Element {
        Element::text("view")
    }
}

#[test]
fn transitions_are_unique_ordered_and_delivered_after_transaction_borrows() {
    let runtime = ModelRuntime::default();
    let model = runtime.entity(View);
    let events = Rc::new(RefCell::new(Vec::new()));
    let log = events.clone();
    let weak = model.downgrade();
    let _subscription = runtime.observe_mounts(move |event| {
        weak.upgrade().unwrap().update(|_, _| {});
        log.borrow_mut().push(event);
    });
    runtime.transaction(|| {
        let mount = model.mount().unwrap();
        model.update(|_, _| {
            mount.set_visible(false).unwrap();
            mount.set_visible(false).unwrap();
            mount.set_visible(true).unwrap();
            mount.close();
            mount.close();
        });
        assert!(events.borrow().is_empty());
    });
    runtime.dispatch_pending();
    let events = events.borrow();
    assert_eq!(
        events
            .iter()
            .map(|event| event.transition)
            .collect::<Vec<_>>(),
        [
            MountTransition::Mounted,
            MountTransition::VisibilityChanged { visible: false },
            MountTransition::VisibilityChanged { visible: true },
            MountTransition::Unmounted,
        ]
    );
    assert!(
        events
            .iter()
            .all(|event| event.model == model.id() && event.mount == events[0].mount)
    );
}

#[test]
fn cancelled_observers_do_not_receive_queued_transitions() {
    let runtime = ModelRuntime::default();
    let subscription = runtime.observe_mounts(|_| panic!("cancelled lifecycle observer"));
    runtime.transaction(|| {
        let model = runtime.entity(View);
        let mount = model.mount().unwrap();
        drop(mount);
        subscription.cancel();
    });
    runtime.dispatch_pending();
    assert_eq!(runtime.pending_mount_events(), 0);
    assert!(!subscription.is_active());
}

#[test]
fn observer_can_close_parent_and_remount_without_reentering_delivery() {
    let runtime = ModelRuntime::default();
    let model = runtime.entity(View);
    let current = Rc::new(RefCell::new(None::<argui_runtime::Mount<View>>));
    let target = current.clone();
    let weak = model.downgrade();
    let transitions = Rc::new(RefCell::new(Vec::new()));
    let log = transitions.clone();
    let _subscription = runtime.observe_mounts(move |event| {
        log.borrow_mut().push(event.transition);
        if event.transition == (MountTransition::VisibilityChanged { visible: false }) {
            let old = target.borrow_mut().take().unwrap();
            old.close();
            *target.borrow_mut() = Some(weak.upgrade().unwrap().mount().unwrap());
        }
    });
    let first = model.mount().unwrap();
    *current.borrow_mut() = Some(first.clone());
    first.set_visible(false).unwrap();
    runtime.dispatch_pending();
    assert!(first.resources().is_closed());
    assert_ne!(first.id(), current.borrow().as_ref().unwrap().id());
    assert_eq!(
        &*transitions.borrow(),
        &[
            MountTransition::Mounted,
            MountTransition::VisibilityChanged { visible: false },
            MountTransition::Unmounted,
            MountTransition::Mounted,
        ]
    );
}

#[test]
fn panic_preserves_remaining_delivery_and_does_not_prevent_cleanup() {
    let runtime = ModelRuntime::default();
    let model = runtime.entity(View);
    let mount = model.mount().unwrap();
    let _panic = runtime.observe_mounts(|_| panic!("observer failure"));
    let events = Rc::new(RefCell::new(Vec::new()));
    let log = events.clone();
    let _remaining = runtime.observe_mounts(move |event| log.borrow_mut().push(event.transition));
    mount.close();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| runtime.dispatch_pending()))
            .is_err()
    );
    assert!(mount.resources().is_closed());
    assert_eq!(model.resources().resource_count(), 0);
    runtime.dispatch_pending();
    assert_eq!(&*events.borrow(), &[MountTransition::Unmounted]);
    assert_eq!(runtime.pending_mount_events(), 0);
}

#[test]
fn parents_mount_before_children_and_unmount_after_them_for_close_and_drop() {
    struct Parent(Entity<View>);
    impl Render for Parent {
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            cx.entity(&self.0)
        }
    }
    for explicit in [false, true] {
        let runtime = ModelRuntime::default();
        let child = runtime.entity(View);
        let parent = runtime.entity(Parent(child.clone()));
        let events = Rc::new(RefCell::new(Vec::new()));
        let log = events.clone();
        let _subscription = runtime
            .observe_mounts(move |event| log.borrow_mut().push((event.model, event.transition)));
        let mount = parent.mount().unwrap();
        mount.render(Default::default()).unwrap();
        if explicit {
            mount.close();
        }
        drop(mount);
        runtime.dispatch_pending();
        assert_eq!(
            &*events.borrow(),
            &[
                (parent.id(), MountTransition::Mounted),
                (child.id(), MountTransition::Mounted),
                (child.id(), MountTransition::Unmounted),
                (parent.id(), MountTransition::Unmounted),
            ]
        );
    }
}

#[test]
fn child_domain_notifications_wait_until_parent_render_has_released_its_borrow() {
    struct Parent(Entity<View>);
    impl Render for Parent {
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            cx.entity(&self.0)
        }
    }
    let child_runtime = ModelRuntime::default();
    let child = child_runtime.entity(View);
    let parent = Entity::new(Parent(child));
    let weak = parent.downgrade();
    let calls = Rc::new(std::cell::Cell::new(0));
    let observed = calls.clone();
    let _subscription = child_runtime.observe_mounts(move |_| {
        weak.upgrade().unwrap().update(|_, cx| cx.notify());
        observed.set(observed.get() + 1);
    });
    let _ = parent.render();
    assert_eq!(calls.get(), 0);
    child_runtime.dispatch_pending();
    assert_eq!(calls.get(), 1);
}

#[test]
fn lifecycle_budget_yields_to_the_host_and_returns_to_idle() {
    let wakes = Rc::new(std::cell::Cell::new(0));
    let wake = wakes.clone();
    let runtime = ModelRuntime::new(move || wake.set(wake.get() + 1));
    let delivered = Rc::new(std::cell::Cell::new(0));
    let count = delivered.clone();
    let _subscription = runtime.observe_mounts(move |_| count.set(count.get() + 1));
    let model = runtime.entity(View);
    for _ in 0..40 {
        drop(model.mount().unwrap());
    }
    assert_eq!(delivered.get(), 0);
    assert_eq!(runtime.pending_mount_events(), 80);
    assert_eq!(wakes.get(), 1);
    runtime.dispatch_pending();
    assert_eq!(delivered.get(), 64);
    assert_eq!(runtime.pending_mount_events(), 16);
    assert_eq!(wakes.get(), 2);
    runtime.dispatch_pending();
    assert_eq!(delivered.get(), 80);
    assert_eq!(runtime.pending_mount_events(), 0);
    runtime.dispatch_pending();
    assert_eq!(wakes.get(), 2);
}

#[test]
fn interrupted_first_render_still_reports_balanced_mount_and_unmount() {
    struct Broken;
    impl Render for Broken {
        fn render(&mut self, _: &mut Context<Self>) -> Element {
            panic!("interrupted render");
        }
    }
    let runtime = ModelRuntime::default();
    let model = runtime.entity(Broken);
    let events = Rc::new(RefCell::new(Vec::new()));
    let log = events.clone();
    let _subscription =
        runtime.observe_mounts(move |event| log.borrow_mut().push(event.transition));
    let mut weak = None;
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mount = model.mount().unwrap();
            weak = Some(mount.downgrade());
            let _ = mount.render(Default::default());
        }))
        .is_err()
    );
    assert!(weak.unwrap().upgrade().is_none());
    assert_eq!(model.resources().resource_count(), 0);
    assert!(events.borrow().is_empty());
    runtime.dispatch_pending();
    assert_eq!(
        &*events.borrow(),
        &[MountTransition::Mounted, MountTransition::Unmounted]
    );
}
