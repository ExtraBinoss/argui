use std::time::Duration;

use argui_core::Point;

use crate::{NodeId, ScrollRegion};

/// Keeps a wheel gesture on its initial viewport while content moves beneath it.
#[derive(Clone, Debug, Default)]
pub struct ScrollGesture {
    target: Option<NodeId>,
    last_input: Option<Duration>,
}

impl ScrollGesture {
    /// Selects a target after 180 ms of inactivity or an explicit gesture start.
    /// Hosts should pass a monotonic timestamp and not feed synthetic inertia here.
    pub fn target(
        &mut self,
        point: Point,
        now: Duration,
        started: bool,
        regions: &[ScrollRegion],
    ) -> Option<NodeId> {
        let continuing = !started
            && self.last_input.is_some_and(|last| {
                now.checked_sub(last)
                    .is_some_and(|elapsed| elapsed < Duration::from_millis(180))
            });
        if !continuing {
            self.target = regions
                .iter()
                .rev()
                .find(|region| region.config.enabled && region.contains(point))
                .map(|region| region.node);
        } else if !regions
            .iter()
            .any(|region| Some(region.node) == self.target && region.config.enabled)
        {
            // A removed viewport must not redirect the rest of a gesture elsewhere.
            self.target = None;
        }
        self.last_input = Some(now);
        self.target
    }
}
