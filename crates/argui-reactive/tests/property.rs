use std::{cell::RefCell, rc::Rc};

use argui_reactive::{Computed, Property, transaction};

#[test]
fn borrowed_reads_do_not_clone_large_models_and_track_dependencies() {
    let rows = Property::new((0..10_000).collect::<Vec<_>>());
    let original = rows.with(|items| items.as_ptr());
    assert_eq!(rows.with(|items| items.as_ptr()), original);
    let observed = rows.clone();
    let count = Computed::new("row count", move || observed.with(Vec::len));
    assert_eq!(count.get(), Ok(10_000));
    rows.set(vec![1, 2, 3]);
    assert_eq!(count.get(), Ok(3));
}

#[test]
fn transactions_coalesce_nested_writes_into_one_notification() {
    let value = Property::new(1_i32);
    let observed = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&observed);
    let _subscription = value.observe(move |value| output.borrow_mut().push(*value));

    transaction(|| {
        assert!(value.set(2));
        transaction(|| {
            assert!(value.set(3));
            assert!(!value.set(3));
        });
        assert_eq!(value.get(), 3);
    });

    assert_eq!(observed.borrow().as_slice(), [3]);
    assert_eq!(value.revision(), 2);
}

#[test]
fn updating_and_cancelling_observers_are_explicit() {
    let value = Property::new(String::from("a"));
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&calls);
    let subscription = value.observe(move |value| output.borrow_mut().push(value.clone()));

    assert!(value.update(|value| value.push('b')));
    subscription.cancel();
    assert!(value.set(String::from("c")));

    assert_eq!(calls.borrow().as_slice(), ["ab"]);
}

#[test]
fn in_place_mutation_keeps_string_storage_and_notifies_after_releasing_borrow() {
    let mut initial = String::with_capacity(32);
    initial.push_str("é🙂");
    let value = Property::new(initial);
    let pointer = value.with(|text| text.as_ptr());
    let observed = value.clone();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&calls);
    let _subscription = value.observe(move |_| {
        output.borrow_mut().push(observed.with(|text| text.clone()));
    });

    assert!(value.mutate(|text| {
        text.replace_range(2..2, "中");
        true
    }));
    assert_eq!(value.with(|text| text.as_ptr()), pointer);
    assert_eq!(value.revision(), 1);
    assert_eq!(calls.borrow().as_slice(), ["é中🙂"]);
    assert!(!value.mutate(|_| false));
    assert_eq!(value.revision(), 1);
    assert_eq!(calls.borrow().len(), 1);
}
