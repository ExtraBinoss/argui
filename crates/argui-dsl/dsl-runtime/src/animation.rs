use std::collections::HashMap;

use argui_dsl_ir::{AnimationId, ComponentId, PropertyTargetId, SiteId};

use crate::InstanceId;

/// Persistent live-animation key specified by the DSL architecture contract.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AnimationKey {
    pub instance: InstanceId,
    pub component: ComponentId,
    pub site: SiteId,
    pub property: PropertyTargetId,
    pub animation: AnimationId,
}

/// Type-erased scalar motion state retained across compatible driver reloads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimationState {
    pub value: f64,
    pub velocity: f64,
    pub specification: u64,
}

/// Persistent animation records keyed independently from syntax offsets.
#[derive(Default)]
pub struct AnimationStore {
    values: HashMap<AnimationKey, AnimationState>,
}

impl AnimationStore {
    /// Creates an empty animation store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reuses or creates a motion while preserving current value and velocity.
    ///
    /// * `key` — stable component/site/property/slot identity.
    /// * `initial` — initial value for a newly introduced animation.
    /// * `specification` — hash of the current driver parameters.
    pub fn retarget(
        &mut self,
        key: AnimationKey,
        initial: f64,
        specification: u64,
    ) -> &mut AnimationState {
        let state = self.values.entry(key).or_insert(AnimationState {
            value: initial,
            velocity: 0.0,
            specification,
        });
        state.specification = specification;
        state
    }

    /// Removes animation records not present in the next accepted package.
    pub fn retain(&mut self, mut keep: impl FnMut(&AnimationKey) -> bool) {
        self.values.retain(|key, _| keep(key));
    }

    /// Returns one animation state for inspection.
    #[must_use]
    pub fn get(&self, key: AnimationKey) -> Option<AnimationState> {
        self.values.get(&key).copied()
    }

    /// Updates a motion sample produced by the host animation engine.
    ///
    /// * `key` — stable live animation identity.
    /// * `value` — current scalar presentation value.
    /// * `velocity` — current scalar velocity retained for future retargets.
    ///
    /// Returns `false` when the animation has not been created by rendering yet.
    pub fn update(&mut self, key: AnimationKey, value: f64, velocity: f64) -> bool {
        let Some(state) = self.values.get_mut(&key) else {
            return false;
        };
        state.value = value;
        state.velocity = velocity;
        true
    }

    /// Returns the number of retained animation slots.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns whether no animation slots are retained.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}
