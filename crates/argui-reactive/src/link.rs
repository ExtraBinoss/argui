use std::{cell::Cell, rc::Rc};

use crate::{Property, Subscription, transaction};

/// A pair of observer subscriptions that synchronizes two typed properties.
pub struct TwoWayLink {
    _left: Subscription,
    _right: Subscription,
}

impl TwoWayLink {
    /// Connects two properties and initializes `right` from `left`.
    ///
    /// Equal-value suppression and a shared reentrancy guard prevent feedback
    /// loops. Dropping the returned link disconnects both directions.
    ///
    /// * `left` — authoritative property for initial synchronization.
    /// * `right` — property updated immediately and then observed bidirectionally.
    #[must_use]
    pub fn new<T>(left: &Property<T>, right: &Property<T>) -> Self
    where
        T: Clone + PartialEq + 'static,
    {
        transaction(|| {
            right.set(left.get());
        });
        let updating = Rc::new(Cell::new(false));
        let right_target = right.clone();
        let left_guard = Rc::clone(&updating);
        let left_subscription = left.observe(move |value| {
            if left_guard.replace(true) {
                return;
            }
            right_target.set(value.clone());
            left_guard.set(false);
        });
        let left_target = left.clone();
        let right_guard = updating;
        let right_subscription = right.observe(move |value| {
            if right_guard.replace(true) {
                return;
            }
            left_target.set(value.clone());
            right_guard.set(false);
        });
        Self {
            _left: left_subscription,
            _right: right_subscription,
        }
    }
}
