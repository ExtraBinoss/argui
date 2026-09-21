use std::{cell::RefCell, rc::Rc};

use argui_reactive::{Property, transaction};

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
