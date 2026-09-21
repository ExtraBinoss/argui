use std::{cell::Cell, cell::RefCell, rc::Rc};

use argui_reactive::{Computed, Property, ReactiveError, transaction};

#[test]
fn bindings_recompute_only_after_a_dependency_changes() {
    let source = Property::new(2_i32);
    let unrelated = Property::new(10_i32);
    let evaluations = Rc::new(Cell::new(0));
    let counter = Rc::clone(&evaluations);
    let input = source.clone();
    let doubled = Computed::new("doubled", move || {
        counter.set(counter.get() + 1);
        input.get() * 2
    });

    assert_eq!(doubled.get(), Ok(4));
    assert_eq!(doubled.get(), Ok(4));
    assert_eq!(evaluations.get(), 1);
    unrelated.set(20);
    assert_eq!(doubled.get(), Ok(4));
    assert_eq!(evaluations.get(), 1);
    source.set(3);
    assert_eq!(doubled.get(), Ok(6));
    assert_eq!(evaluations.get(), 2);
}

#[test]
fn dynamic_dependencies_replace_obsolete_edges() {
    let choose_left = Property::new(true);
    let left = Property::new(1_i32);
    let right = Property::new(10_i32);
    let evaluations = Rc::new(Cell::new(0));
    let counter = Rc::clone(&evaluations);
    let selector = choose_left.clone();
    let left_input = left.clone();
    let right_input = right.clone();
    let selected = Computed::new("selected", move || {
        counter.set(counter.get() + 1);
        if selector.get() {
            left_input.get()
        } else {
            right_input.get()
        }
    });

    assert_eq!(selected.get(), Ok(1));
    choose_left.set(false);
    assert_eq!(selected.get(), Ok(10));
    let after_switch = evaluations.get();
    left.set(2);
    assert_eq!(selected.get(), Ok(10));
    assert_eq!(evaluations.get(), after_switch);
    right.set(11);
    assert_eq!(selected.get(), Ok(11));
    assert_eq!(evaluations.get(), after_switch + 1);
}

#[test]
fn transaction_recomputes_and_notifies_once_with_the_final_value() {
    let source = Property::new(1_i32);
    let input = source.clone();
    let derived = Computed::new("triple", move || input.get() * 3);
    assert_eq!(derived.get(), Ok(3));
    let values = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&values);
    let _subscription = derived.observe(move |value| output.borrow_mut().push(*value));

    transaction(|| {
        source.set(2);
        source.set(3);
        source.set(4);
    });

    assert_eq!(derived.get(), Ok(12));
    assert_eq!(values.borrow().as_slice(), [12]);
    assert_eq!(derived.revision(), 2);
}

#[test]
fn cycles_report_the_complete_deterministic_path() {
    let first_slot = Rc::new(RefCell::new(None::<Computed<i32>>));
    let second_slot = Rc::new(RefCell::new(None::<Computed<i32>>));
    let second_for_first = Rc::clone(&second_slot);
    let first = Computed::try_new("first", move || {
        second_for_first
            .borrow()
            .as_ref()
            .expect("second binding installed")
            .get()
    });
    let first_for_second = Rc::clone(&first_slot);
    let second = Computed::try_new("second", move || {
        first_for_second
            .borrow()
            .as_ref()
            .expect("first binding installed")
            .get()
    });
    *first_slot.borrow_mut() = Some(first.clone());
    *second_slot.borrow_mut() = Some(second);

    assert_eq!(
        first.get(),
        Err(ReactiveError::Cycle {
            path: vec![
                String::from("first"),
                String::from("second"),
                String::from("first")
            ],
        })
    );
}

#[test]
fn failed_evaluation_recovers_without_publishing_a_stale_value() {
    let source = Property::new(1_i32);
    let input = source.clone();
    let derived = Computed::try_new("positive", move || {
        let value = input.get();
        if value < 0 {
            Err(ReactiveError::evaluation("negative input"))
        } else {
            Ok(value * 2)
        }
    });
    assert_eq!(derived.get(), Ok(2));
    source.set(-1);
    assert_eq!(
        derived.get(),
        Err(ReactiveError::evaluation("negative input"))
    );
    assert_eq!(derived.revision(), 1);
    source.set(3);
    assert_eq!(derived.get(), Ok(6));
    assert_eq!(derived.revision(), 2);
    assert_eq!(ReactiveError::evaluation("bad").to_string(), "bad");
    assert_eq!(
        ReactiveError::Cycle {
            path: vec!["a".into(), "b".into()]
        }
        .to_string(),
        "reactive cycle: a -> b"
    );
}

#[test]
fn unchanged_derived_values_do_not_notify_or_increment_revision() {
    let source = Property::new(2_i32);
    let input = source.clone();
    let parity = Computed::new("parity", move || input.get() % 2);
    assert_eq!(parity.get(), Ok(0));
    let notifications = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&notifications);
    let subscription = parity.observe(move |value| output.borrow_mut().push(*value));
    source.set(4);
    assert_eq!(parity.get(), Ok(0));
    assert_eq!(parity.revision(), 1);
    assert!(notifications.borrow().is_empty());
    source.set(5);
    assert_eq!(parity.get(), Ok(1));
    assert_eq!(notifications.borrow().as_slice(), [1]);
    drop(subscription);
    source.set(6);
    assert_eq!(parity.get(), Ok(0));
    assert_eq!(notifications.borrow().as_slice(), [1]);
}

/// Repeated reads in one evaluation create one graph edge and a dead observer detaches safely.
#[test]
fn duplicate_dependency_reads_and_dropped_observers_are_safe() {
    let source = Property::new(3_i32);
    let input = source.clone();
    let total = Computed::new("double read", move || input.get() + input.get());
    assert_eq!(total.get(), Ok(6));
    source.set(4);
    assert_eq!(total.get(), Ok(8));
    let subscription = total.observe(|_| {});
    drop(total);
    drop(subscription);
}
