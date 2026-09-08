use argui_runtime::{EventEmitter, EventError, ModelRuntime};
use std::{cell::Cell, rc::Rc};

struct Model;
impl EventEmitter<()> for Model {}

#[test]
fn pending_work_coalesces_wakes_until_the_host_acknowledges_them() {
    let runtime = ModelRuntime::default();
    let model = runtime.entity(Model);
    let mut subscription = None;
    model.update(|_, cx| {
        subscription = Some(
            cx.subscribe(&model, |_, _: &(), cx| cx.emit(()).unwrap())
                .unwrap(),
        );
        cx.emit(()).unwrap();
    });
    assert_eq!(runtime.pending_events(), 1);
    let wakes = Rc::new(Cell::new(0));
    let observed = wakes.clone();
    runtime.set_wake(move || observed.set(observed.get() + 1));
    assert_eq!(wakes.get(), 1);
    for _ in 0..10 {
        model.read(|_| ());
    }
    assert_eq!(wakes.get(), 1);
    runtime.dispatch_pending();
    assert_eq!(wakes.get(), 2);
    let replacement = Rc::new(Cell::new(0));
    let observed = replacement.clone();
    runtime.set_wake(move || observed.set(observed.get() + 1));
    assert_eq!(replacement.get(), 1);
    drop(subscription);
    runtime.dispatch_pending();
    runtime.dispatch_pending();
    assert_eq!(runtime.pending_events(), 0);
    assert_eq!(replacement.get(), 1);
    assert_eq!(wakes.get(), 2);
}

#[test]
fn attaching_a_host_to_an_idle_runtime_does_not_wake_it() {
    let runtime = ModelRuntime::default();
    runtime.set_wake(|| panic!("idle runtime woke its host"));
    runtime.entity(Model).read(|_| ());
    runtime.dispatch_pending();
}

#[test]
fn feedback_yields_after_a_bounded_batch_and_requests_a_host_wake() {
    let wakes = Rc::new(Cell::new(0));
    let wake = wakes.clone();
    let runtime = ModelRuntime::new(move || wake.set(wake.get() + 1));
    let model = runtime.entity(Model);
    let count = Rc::new(Cell::new(0));
    let delivered = count.clone();
    let mut subscription = None;
    model.update(|_, cx| {
        subscription = Some(
            cx.subscribe(&model, move |_, _: &(), cx| {
                delivered.set(delivered.get() + 1);
                cx.emit(()).unwrap();
            })
            .unwrap(),
        );
        cx.emit(()).unwrap();
    });
    assert_eq!(count.get(), 64);
    assert_eq!(wakes.get(), 1);
    assert_eq!(runtime.pending_events(), 1);
    drop(subscription);
    runtime.dispatch_pending();
    assert_eq!(runtime.pending_events(), 0);
    assert_eq!(count.get(), 64);
}

#[test]
fn capacity_rejection_does_not_silently_drop_previously_accepted_events() {
    let runtime = ModelRuntime::default();
    let model = runtime.entity(Model);
    let count = Rc::new(Cell::new(0));
    let delivered = count.clone();
    let mut subscription = None;
    model.update(|_, cx| {
        subscription = Some(
            cx.subscribe(&model, move |_, _: &(), _| {
                delivered.set(delivered.get() + 1)
            })
            .unwrap(),
        );
        for _ in 0..1024 {
            cx.emit(()).unwrap();
        }
        assert_eq!(cx.emit(()), Err(EventError::CapacityExceeded));
    });
    while runtime.pending_events() > 0 {
        runtime.dispatch_pending();
    }
    assert_eq!(count.get(), 1024);
    drop(subscription);
}

#[test]
fn a_single_event_with_many_listeners_yields_without_reordering_events() {
    use std::cell::RefCell;
    struct Source;
    impl EventEmitter<u32> for Source {}
    let runtime = ModelRuntime::default();
    let model = runtime.entity(Source);
    let delivered = Rc::new(RefCell::new(Vec::new()));
    let mut subscriptions = Vec::new();
    model.update(|_, cx| {
        for listener in 0..130 {
            let delivered = delivered.clone();
            subscriptions.push(
                cx.subscribe(&model, move |_, event: &u32, _| {
                    delivered.borrow_mut().push((*event, listener));
                })
                .unwrap(),
            );
        }
        cx.emit(1_u32).unwrap();
        cx.emit(2_u32).unwrap();
    });
    assert_eq!(delivered.borrow().len(), 64);
    assert_eq!(runtime.pending_events(), 2);
    while runtime.pending_events() > 0 {
        runtime.dispatch_pending();
    }
    let expected: Vec<_> = [1, 2]
        .into_iter()
        .flat_map(|event| (0..130).map(move |listener| (event, listener)))
        .collect();
    assert_eq!(*delivered.borrow(), expected);
    drop(subscriptions);
}

#[test]
fn admission_counts_the_event_currently_delivering() {
    let runtime = ModelRuntime::default();
    let model = runtime.entity(Model);
    let count = Rc::new(Cell::new(0));
    let delivered = count.clone();
    let mut subscription = None;
    model.update(|_, cx| {
        subscription = Some(
            cx.subscribe(&model, move |_, _: &(), cx| {
                delivered.set(delivered.get() + 1);
                if delivered.get() == 1 {
                    for _ in 0..1023 {
                        cx.emit(()).unwrap();
                    }
                    assert_eq!(cx.emit(()), Err(EventError::CapacityExceeded));
                }
            })
            .unwrap(),
        );
        cx.emit(()).unwrap();
    });
    while runtime.pending_events() > 0 {
        runtime.dispatch_pending();
    }
    assert_eq!(count.get(), 1024);
    drop(subscription);
}
