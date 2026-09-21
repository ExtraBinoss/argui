use crate::{Property, Subscription, transaction};

/// A pair of observer subscriptions that synchronizes two typed properties.
pub struct TwoWayLink {
    _left: Subscription,
    _right: Subscription,
}

impl TwoWayLink {
    /// Connects two properties and initializes `right` from `left`.
    ///
    /// Equal-value suppression stops feedback between the queued notifications.
    /// Dropping the returned link disconnects both directions.
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
        let right_target = right.clone();
        let left_subscription = left.observe(move |value| {
            right_target.set(value.clone());
        });
        let left_target = left.clone();
        let right_subscription = right.observe(move |value| {
            left_target.set(value.clone());
        });
        Self {
            _left: left_subscription,
            _right: right_subscription,
        }
    }
}
