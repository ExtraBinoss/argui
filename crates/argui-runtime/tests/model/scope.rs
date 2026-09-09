use argui_runtime::{Entity, EventEmitter, EventError, ModelRuntime, ResourceScope, ScopeClosed};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[test]
fn multiple_cleanup_panics_preserve_reverse_order_and_the_first_failure() {
    let scope = ResourceScope::default();
    let log = Rc::new(RefCell::new(Vec::new()));
    let leases: Vec<_> = (0..3)
        .map(|index| {
            let log = log.clone();
            scope
                .defer(move || {
                    log.borrow_mut().push(index);
                    std::panic::panic_any(index);
                })
                .unwrap()
        })
        .collect();
    let error =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| scope.close())).unwrap_err();
    assert_eq!(error.downcast_ref::<i32>(), Some(&2));
    assert_eq!(&*log.borrow(), &[2, 1, 0]);
    assert!(leases.iter().all(|lease| !lease.is_active()));
    assert_eq!(scope.resource_count(), 0);
    scope.close();
}

#[test]
fn cleanup_runs_once_in_reverse_order_and_leases_do_not_retain_the_scope() {
    let scope = ResourceScope::default();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let mut leases = Vec::new();
    for index in 0..3 {
        let calls = calls.clone();
        leases.push(scope.defer(move || calls.borrow_mut().push(index)).unwrap());
    }
    assert_eq!(scope.resource_count(), 3);
    drop(leases.remove(1));
    assert_eq!(*calls.borrow(), [1]);
    scope.close();
    scope.close();
    assert_eq!(*calls.borrow(), [1, 2, 0]);
    assert!(leases.iter().all(|lease| !lease.is_active()));
    assert_eq!(scope.resource_count(), 0);
    assert!(matches!(scope.defer(|| {}), Err(ScopeClosed)));
    assert!(!ScopeClosed.to_string().is_empty());

    let scope = ResourceScope::default();
    let count = Rc::new(Cell::new(0));
    let released = count.clone();
    let lease = scope
        .defer(move || released.set(released.get() + 1))
        .unwrap();
    assert!(lease.is_active());
    drop(scope);
    assert!(!lease.is_active());
    assert_eq!(count.get(), 1);
    drop(lease);
    assert_eq!(count.get(), 1);
}

#[test]
fn closing_entity_resources_releases_owned_values_even_with_surviving_scope_handles() {
    struct Value(Rc<Cell<usize>>);
    impl Drop for Value {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let entity = Entity::new(());
    let scope = entity.resources().clone();
    let count = Rc::new(Cell::new(0));
    let lease = scope.own(Value(count.clone())).unwrap();
    drop(entity);
    assert!(scope.is_closed());
    assert!(!lease.is_active());
    assert_eq!(count.get(), 1);
    assert!(scope.own(Value(count.clone())).is_err());
    assert_eq!(count.get(), 2);
}

#[test]
fn cleanup_can_close_again_or_drop_another_lease_without_registry_borrow_conflicts() {
    let scope = ResourceScope::default();
    let other = scope.defer(|| {}).unwrap();
    let closed = scope.clone();
    let lease = scope
        .defer(move || {
            closed.close();
            drop(other);
            assert!(closed.defer(|| {}).is_err());
        })
        .unwrap();
    scope.close();
    assert!(!lease.is_active());
    assert_eq!(scope.resource_count(), 0);
}

#[test]
fn an_unwinding_cleanup_does_not_prevent_other_resources_from_being_released() {
    let scope = ResourceScope::default();
    let released = Rc::new(Cell::new(false));
    let observed = released.clone();
    let first = scope.defer(move || observed.set(true)).unwrap();
    let last = scope.defer(|| panic!("cleanup failure")).unwrap();
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| scope.close())).is_err());
    assert!(released.get());
    assert!(!first.is_active());
    assert!(!last.is_active());
    assert_eq!(scope.resource_count(), 0);
}

struct Source;
impl EventEmitter<()> for Source {}

#[test]
fn scopes_cancel_queued_events_and_both_endpoint_registrations_are_removed() {
    let runtime = ModelRuntime::default();
    let source = runtime.entity(Source);
    let target = runtime.entity(0);
    let scope = ResourceScope::default();
    let mut subscription = None;
    target.update(|_, cx| {
        subscription = Some(
            cx.subscribe(&source, |count, _: &(), _| *count += 1)
                .unwrap()
                .in_scope(&scope)
                .unwrap(),
        );
    });
    assert_eq!(source.resources().resource_count(), 1);
    assert_eq!(target.resources().resource_count(), 1);
    runtime.transaction(|| {
        source.update(|_, cx| cx.emit(()).unwrap());
        scope.close();
    });
    assert_eq!(target.read(|count| *count), 0);
    assert!(!subscription.unwrap().is_active());
    assert_eq!(source.resources().resource_count(), 0);
    assert_eq!(target.resources().resource_count(), 0);
    assert_eq!(scope.resource_count(), 0);
}

#[test]
fn closed_endpoint_scopes_reject_subscriptions_without_leaking_registrations() {
    let runtime = ModelRuntime::default();
    let source = runtime.entity(Source);
    let target = runtime.entity(());
    source.resources().close();
    target.update(|_, cx| {
        assert!(matches!(
            cx.subscribe(&source, |_, _: &(), _| {}),
            Err(EventError::ScopeClosed)
        ));
        assert!(matches!(cx.observe(&source), Err(EventError::ScopeClosed)));
    });
    assert_eq!(target.resources().resource_count(), 0);
    let source = runtime.entity(Source);
    target.resources().close();
    target.update(|_, cx| {
        assert!(matches!(
            cx.subscribe(&source, |_, _: &(), _| {}),
            Err(EventError::ScopeClosed)
        ));
    });
    assert_eq!(source.resources().resource_count(), 0);
    assert!(!EventError::ScopeClosed.to_string().is_empty());
}

#[test]
fn repeated_registration_and_cancellation_has_no_retained_scope_entries() {
    let runtime = ModelRuntime::default();
    let source = runtime.entity(Source);
    let target = runtime.entity(());
    for _ in 0..1000 {
        target.update(|_, cx| {
            let subscription = cx.subscribe(&source, |_, _: &(), _| {}).unwrap();
            subscription.cancel();
            subscription.cancel();
        });
        assert_eq!(source.resources().resource_count(), 0);
        assert_eq!(target.resources().resource_count(), 0);
    }
}

#[test]
fn a_closed_source_rejects_emission_even_without_listeners() {
    let source = Entity::new(Source);
    source.resources().close();
    source.update(|_, cx| assert_eq!(cx.emit(()), Err(EventError::ScopeClosed)));
    assert_eq!(source.runtime().pending_events(), 0);
}

#[test]
fn source_closure_discards_a_large_queued_delivery_in_one_step() {
    struct Payload(Rc<Cell<bool>>);
    impl Drop for Payload {
        fn drop(&mut self) {
            self.0.set(true);
        }
    }
    impl EventEmitter<Payload> for Source {}
    let runtime = ModelRuntime::default();
    let source = runtime.entity(Source);
    let target = runtime.entity(());
    let mut subscriptions = Vec::new();
    target.update(|_, cx| {
        for _ in 0..200 {
            subscriptions.push(
                cx.subscribe(&source, |_, _: &Payload, _| {
                    panic!("delivery from closed source");
                })
                .unwrap(),
            );
        }
    });
    let released = Rc::new(Cell::new(false));
    runtime.transaction(|| {
        source.update(|_, cx| cx.emit(Payload(released.clone())).unwrap());
        source.resources().close();
        assert!(!released.get());
    });
    assert!(released.get());
    assert_eq!(runtime.pending_events(), 0);
    assert_eq!(target.resources().resource_count(), 0);
    assert!(
        subscriptions
            .iter()
            .all(|subscription| !subscription.is_active())
    );
}

#[test]
fn cancelled_subscriptions_cannot_be_attached_to_a_closed_scope() {
    let runtime = ModelRuntime::default();
    let source = runtime.entity(Source);
    let target = runtime.entity(());
    let closed = ResourceScope::default();
    closed.close();
    target.update(|_, cx| {
        let subscription = cx.subscribe(&source, |_, _: &(), _| {}).unwrap();
        subscription.cancel();
        assert!(matches!(subscription.in_scope(&closed), Err(ScopeClosed)));
    });
    assert_eq!(source.resources().resource_count(), 0);
    assert_eq!(target.resources().resource_count(), 0);
}
