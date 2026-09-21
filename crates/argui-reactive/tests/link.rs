use std::{cell::RefCell, rc::Rc};

use argui_reactive::{Property, TwoWayLink, transaction};

#[test]
fn two_way_links_initialize_propagate_and_disconnect_without_feedback() {
    let left = Property::new(String::from("initial"));
    let right = Property::new(String::from("discarded"));
    let left_values = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&left_values);
    let _observer = left.observe(move |value| output.borrow_mut().push(value.clone()));
    let link = TwoWayLink::new(&left, &right);

    assert_eq!(right.get(), "initial");
    left.set(String::from("from left"));
    assert_eq!(right.get(), "from left");
    right.set(String::from("from right"));
    assert_eq!(left.get(), "from right");
    assert_eq!(left_values.borrow().as_slice(), ["from left", "from right"]);

    drop(link);
    right.set(String::from("disconnected"));
    assert_eq!(left.get(), "from right");
}

#[test]
fn equal_values_in_a_transaction_do_not_loop_or_duplicate_notifications() {
    let left = Property::new(1_u32);
    let right = Property::new(1_u32);
    let _link = TwoWayLink::new(&left, &right);

    transaction(|| {
        left.set(2);
        left.set(3);
    });

    assert_eq!(left.get(), 3);
    assert_eq!(right.get(), 3);
    assert!(left.revision() < 4);
    assert!(right.revision() < 4);
}
