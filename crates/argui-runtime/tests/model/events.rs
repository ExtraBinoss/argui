use argui_runtime::{Context, Entity, EventEmitter, EventError, ModelRuntime, Subscription};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Default)]
struct Source;
impl EventEmitter<u32> for Source {}
impl EventEmitter<String> for Source {}

#[test]
fn view_observation_invalidates_only_its_mount_and_ends_on_unmount() {
    struct View(usize);
    impl argui_runtime::Render for View {
        fn render(&mut self, _: &mut Context<Self>) -> argui_ui::Element {
            self.0 += 1;
            argui_ui::Element::text(self.0.to_string())
        }
    }
    let runtime = ModelRuntime::default();
    let source = runtime.entity(Source);
    let model = runtime.entity(View(0));
    let observed = model.mount().unwrap();
    let independent = model.mount().unwrap();
    observed.render(Default::default()).unwrap();
    let original = independent.render(Default::default()).unwrap();
    let subscription = observed
        .update(|_, cx| cx.observe(&source))
        .unwrap()
        .unwrap();
    source.update(|_, cx| cx.notify());
    assert_eq!(model.revision(), 0);
    assert_eq!(independent.render(Default::default()).unwrap(), original);
    observed.render(Default::default()).unwrap();
    assert_eq!(model.read(|view| view.0), 3);
    observed.close();
    assert!(!subscription.is_active());
    source.update(|_, cx| cx.notify());
    assert_eq!(independent.render(Default::default()).unwrap(), original);
    assert_eq!(model.read(|view| view.0), 3);
}

#[test]
fn view_context_subscriptions_end_with_the_mount_but_model_subscriptions_survive() {
    struct Receiver(usize);
    impl argui_runtime::Render for Receiver {
        fn render(&mut self, _: &mut Context<Self>) -> argui_ui::Element {
            argui_ui::Element::text("receiver")
        }
    }
    let runtime = ModelRuntime::default();
    let source = runtime.entity(Source);
    let receiver = runtime.entity(Receiver(0));
    let mount = receiver.mount().unwrap();
    let view_subscription = mount
        .update(|_, cx| cx.subscribe(&source, |value, _: &u32, _| value.0 += 100))
        .unwrap()
        .unwrap();
    let model_subscription = receiver
        .update(|_, cx| cx.subscribe(&source, |value, _: &u32, _| value.0 += 1))
        .unwrap();
    source.update(|_, cx| cx.emit(1_u32)).unwrap();
    assert_eq!(receiver.read(|value| value.0), 101);
    runtime.transaction(|| {
        source.update(|_, cx| cx.emit(2_u32)).unwrap();
        mount.close();
    });
    assert!(!view_subscription.is_active());
    assert!(model_subscription.is_active());
    assert_eq!(receiver.read(|value| value.0), 102);
    receiver.resources().close();
    assert!(!model_subscription.is_active());
    assert_eq!(source.resources().resource_count(), 0);
}

#[test]
fn mount_subscriptions_keep_their_environment_and_cancel_queued_delivery_on_close() {
    use argui_core::ColorScheme;
    use argui_runtime::{Render, WindowEnvironment};
    struct Receiver(Vec<ColorScheme>);
    impl Render for Receiver {
        fn render(&mut self, _: &mut Context<Self>) -> argui_ui::Element {
            argui_ui::Element::container([])
        }
    }
    let runtime = ModelRuntime::default();
    let source = runtime.entity(Source);
    let model = runtime.entity(Receiver(Vec::new()));
    let left = model.mount().unwrap();
    let right = model.mount().unwrap();
    let _ = left
        .render(WindowEnvironment {
            color_scheme: ColorScheme::Dark,
            ..Default::default()
        })
        .unwrap();
    let _ = right.render(WindowEnvironment::default()).unwrap();
    let on_event = |model: &mut Receiver, _: &u32, cx: &mut Context<Receiver>| {
        model.0.push(cx.environment().color_scheme);
    };
    let first = left.subscribe(&source, on_event).unwrap();
    let second = right.subscribe(&source, on_event).unwrap();
    source.update(|_, cx| cx.emit(1_u32)).unwrap();
    assert_eq!(
        model.read(|model| model.0.clone()),
        [ColorScheme::Dark, ColorScheme::Light]
    );
    runtime.transaction(|| {
        source.update(|_, cx| cx.emit(2_u32)).unwrap();
        left.close();
    });
    assert!(!first.is_active());
    assert!(second.is_active());
    assert_eq!(
        model.read(|model| model.0.clone()),
        [ColorScheme::Dark, ColorScheme::Light, ColorScheme::Light]
    );
    assert!(matches!(
        left.subscribe(&source, on_event),
        Err(EventError::ScopeClosed)
    ));
    drop(right);
    assert!(!second.is_active());
    assert_eq!(source.resources().resource_count(), 0);
}

#[test]
fn mount_subscription_rejects_another_runtime_without_retaining_resources() {
    use argui_runtime::Render;
    struct Receiver;
    impl Render for Receiver {
        fn render(&mut self, _: &mut Context<Self>) -> argui_ui::Element {
            argui_ui::Element::container([])
        }
    }
    let source = Entity::new(Source);
    let model = Entity::new(Receiver);
    let mount = model.mount().unwrap();
    let baseline = mount.resources().resource_count();
    assert!(matches!(
        mount.subscribe(&source, |_, _: &u32, _| {}),
        Err(EventError::DifferentRuntime)
    ));
    assert_eq!(mount.resources().resource_count(), baseline);
    assert_eq!(source.resources().resource_count(), 0);
}

fn subscribe(runtime: &ModelRuntime, source: &Entity<Source>) -> (Entity<Vec<u32>>, Subscription) {
    let receiver = runtime.entity(Vec::<u32>::new());
    let mut subscription = None;
    receiver.update(|_, cx| {
        subscription = Some(
            cx.subscribe(source, |values, value: &u32, _| values.push(*value))
                .unwrap(),
        );
    });
    (receiver, subscription.unwrap())
}

#[test]
fn typed_events_are_ordered_and_delivered_after_all_model_borrows_end() {
    let runtime = ModelRuntime::default();
    let source = runtime.entity(Source);
    let (receiver, _subscription) = subscribe(&runtime, &source);
    runtime.transaction(|| {
        source.update(|_, cx| {
            cx.emit(1_u32).unwrap();
            cx.emit(String::from("another event type")).unwrap();
            cx.emit(2_u32).unwrap();
            assert!(receiver.read(Vec::is_empty));
        });
        assert!(receiver.read(Vec::is_empty));
    });
    assert_eq!(receiver.read(Clone::clone), [1, 2]);
    assert_eq!(runtime.pending_events(), 0);
}

/// Emitting a declared event with no matching listeners does not queue delivery work.
#[test]
fn events_without_matching_listeners_are_ignored_without_queueing() {
    let runtime = ModelRuntime::default();
    let source = runtime.entity(Source);
    let receiver = runtime.entity(Vec::<String>::new());
    let subscription = receiver.update(|_, cx| {
        cx.subscribe(&source, |events, event: &String, _| {
            events.push(event.clone())
        })
        .unwrap()
    });

    source.update(|_, cx| cx.emit(7_u32).unwrap());

    assert!(subscription.is_active());
    assert_eq!(runtime.pending_events(), 0);
    assert!(receiver.read(Vec::is_empty));
}

#[test]
fn listener_can_update_the_source_without_a_reentrant_mutable_borrow() {
    let runtime = ModelRuntime::default();
    let source = runtime.entity(Source);
    let receiver = runtime.entity(Vec::<u32>::new());
    let weak = source.downgrade();
    let mut subscription = None;
    receiver.update(|_, cx| {
        subscription = Some(
            cx.subscribe(&source, move |values, event: &u32, _| {
                values.push(*event);
                if *event == 1 {
                    weak.upgrade()
                        .unwrap()
                        .update(|_, cx| cx.emit(2_u32).unwrap());
                }
            })
            .unwrap(),
        );
    });
    source.update(|_, cx| cx.emit(1_u32).unwrap());
    assert_eq!(receiver.read(Clone::clone), [1, 2]);
    drop(subscription);
}

#[test]
fn cancellation_during_delivery_suppresses_later_listener_in_same_snapshot() {
    let runtime = ModelRuntime::default();
    let source = runtime.entity(Source);
    let first = runtime.entity(());
    let later = Rc::new(RefCell::new(None::<Subscription>));
    let cancel = later.clone();
    let mut first_subscription = None;
    first.update(|_, cx| {
        first_subscription = Some(
            cx.subscribe(&source, move |_, _: &u32, _| {
                cancel.borrow().as_ref().unwrap().cancel();
            })
            .unwrap(),
        );
    });
    let (receiver, subscription) = subscribe(&runtime, &source);
    *later.borrow_mut() = Some(subscription);
    source.update(|_, cx| cx.emit(1_u32).unwrap());
    assert!(receiver.read(Vec::is_empty));
    assert!(!later.borrow().as_ref().unwrap().is_active());
    drop(first_subscription);
}

#[test]
fn queued_events_do_not_retain_either_endpoint_and_late_subscribers_miss_old_events() {
    let runtime = ModelRuntime::default();
    let source = runtime.entity(Source);
    let (first, subscription) = subscribe(&runtime, &source);
    let weak = first.downgrade();
    let mut late = None;
    runtime.transaction(|| {
        source.update(|_, cx| cx.emit(1_u32).unwrap());
        late = Some(subscribe(&runtime, &source));
        drop(first);
        assert!(weak.upgrade().is_none());
        assert!(!subscription.is_active());
    });
    let (late, _late_subscription) = late.unwrap();
    assert!(late.read(Vec::is_empty));
    runtime.transaction(|| {
        source.update(|_, cx| cx.emit(2_u32).unwrap());
        drop(source);
    });
    assert!(late.read(Vec::is_empty));
}

#[test]
fn subscription_drop_and_explicit_cancel_are_idempotent() {
    let runtime = ModelRuntime::default();
    let source = runtime.entity(Source);
    let (receiver, subscription) = subscribe(&runtime, &source);
    runtime.transaction(|| {
        source.update(|_, cx| cx.emit(1_u32).unwrap());
        subscription.cancel();
        subscription.cancel();
    });
    assert!(receiver.read(Vec::is_empty));
    let (receiver, subscription) = subscribe(&runtime, &source);
    drop(subscription);
    source.update(|_, cx| cx.emit(2_u32).unwrap());
    assert!(receiver.read(Vec::is_empty));
}

#[test]
fn detached_contexts_and_unrelated_runtimes_fail_explicitly() {
    let mut cx = Context::<Source>::default();
    assert_eq!(cx.emit(1_u32), Err(EventError::DetachedContext));
    let source = Entity::new(Source);
    assert!(matches!(
        cx.subscribe(&source, |_, _: &u32, _| {}),
        Err(EventError::DetachedContext)
    ));
    let other = Entity::new(());
    other.update(|_, cx| {
        assert!(matches!(
            cx.subscribe(&source, |_, _: &u32, _| {}),
            Err(EventError::DifferentRuntime)
        ));
    });
    for error in [
        EventError::DetachedContext,
        EventError::DifferentRuntime,
        EventError::CapacityExceeded,
    ] {
        assert!(!error.to_string().is_empty());
    }
}

#[test]
fn new_entities_created_in_context_share_the_event_owner() {
    let parent = Entity::new(());
    let mut child = None;
    parent.update(|_, cx| {
        child = Some(cx.new_entity(Source));
    });
    let child = child.unwrap();
    let count = Rc::new(Cell::new(0));
    let updated = count.clone();
    let mut subscription = None;
    parent.update(|_, cx| {
        subscription = Some(
            cx.subscribe(&child, move |_, _: &u32, _| updated.set(updated.get() + 1))
                .unwrap(),
        );
    });
    child.update(|_, cx| cx.emit(1_u32).unwrap());
    assert_eq!(count.get(), 1);
    drop(subscription);
}

/// Closed model scopes reject new event publication and new observations.
#[test]
fn closed_source_scope_rejects_emission_and_observation() {
    let runtime = ModelRuntime::default();
    let source = runtime.entity(Source);
    let receiver = runtime.entity(());
    source.resources().close();
    source.update(|_, cx| {
        assert_eq!(cx.emit(1_u32), Err(EventError::ScopeClosed));
    });
    receiver.update(|_, cx| {
        assert!(matches!(cx.observe(&source), Err(EventError::ScopeClosed)));
        assert!(matches!(
            cx.subscribe(&source, |_, _: &u32, _| {}),
            Err(EventError::ScopeClosed)
        ));
    });
    assert_eq!(runtime.pending_events(), 0);
    assert_eq!(source.resources().resource_count(), 0);
}

/// Observation reports detached, foreign-runtime, and closed-target failures.
#[test]
fn observations_validate_both_endpoints_before_retaining_resources() {
    let source = Entity::new(Source);
    let mut detached = Context::<()>::default();
    assert!(matches!(
        detached.observe(&source),
        Err(EventError::DetachedContext)
    ));

    let other = Entity::new(());
    other.update(|_, cx| {
        assert!(matches!(
            cx.observe(&source),
            Err(EventError::DifferentRuntime)
        ));
    });

    let runtime = ModelRuntime::default();
    let source = runtime.entity(Source);
    let receiver = runtime.entity(());
    receiver.resources().close();
    receiver.update(|_, cx| {
        assert!(matches!(cx.observe(&source), Err(EventError::ScopeClosed)));
        assert!(matches!(
            cx.subscribe(&source, |_, _: &u32, _| {}),
            Err(EventError::ScopeClosed)
        ));
    });
    assert_eq!(source.resources().resource_count(), 0);
}

struct TrackedReader(Entity<usize>);

impl argui_runtime::Render for TrackedReader {
    /// Reads a dependency from this view so ownership checks occur at render time.
    fn render(&mut self, cx: &mut Context<Self>) -> argui_ui::Element {
        argui_ui::Element::text(cx.read(&self.0, |value| value.to_string()))
    }
}

/// Tracked reads reject both detached contexts and foreign model runtimes.
#[test]
fn tracked_reads_require_an_attached_same_runtime_view() {
    let source = Entity::new(7_usize);
    let mut detached = Context::<TrackedReader>::default();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            detached.read(&source, |value| *value);
        }))
        .is_err()
    );

    let foreign = Entity::new(TrackedReader(source));
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = foreign.render();
        }))
        .is_err()
    );
}

/// A closed dependency cannot be subscribed by a retained render.
#[test]
fn tracked_reads_reject_closed_source_scope() {
    let runtime = ModelRuntime::default();
    let source = runtime.entity(7_usize);
    let reader = runtime.entity(TrackedReader(source.clone()));
    source.resources().close();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = reader.render();
        }))
        .is_err()
    );
    assert_eq!(source.resources().resource_count(), 0);
}
