use super::{
    AnyEntity, Entity, ModelRuntime, RetainedIdentity, ViewUpdate,
    effects::{ContextEffects, merge_effects},
};
use std::collections::HashSet;

pub(super) type ModelRuntimeVisitor = dyn Fn(&mut Vec<ModelRuntime>);

impl<T: 'static> Entity<T> {
    /// Publishes `snapshot` to this presentation and descendant handler contexts.
    pub(crate) fn set_interaction_snapshot(&self, snapshot: &super::InteractionSnapshot) {
        let presentation = &self.0.presentation;
        presentation.interaction_snapshot.replace(snapshot.clone());
        for child in presentation
            .children
            .borrow()
            .iter()
            .chain(presentation.event_routes.borrow().iter())
        {
            (child.set_interaction_snapshot)(snapshot);
        }
    }

    /// Adds source identities read by this presentation and its children to `identities`.
    pub(crate) fn collect_observed(&self, identities: &mut HashSet<RetainedIdentity>) {
        let presentation = &self.0.presentation;
        identities.extend(presentation.observed_identities.borrow().iter().cloned());
        for child in presentation
            .children
            .borrow()
            .iter()
            .chain(presentation.event_routes.borrow().iter())
        {
            (child.collect_observed)(identities);
        }
    }

    /// Invalidates presentations reading identities in `changed`.
    /// Returns whether this presentation or a descendant was affected.
    pub(crate) fn invalidate_observed(&self, changed: &HashSet<RetainedIdentity>) -> bool {
        let presentation = &self.0.presentation;
        let mut affected = presentation
            .observed_identities
            .borrow()
            .iter()
            .any(|identity| changed.contains(identity));
        for child in presentation
            .children
            .borrow()
            .iter()
            .chain(presentation.event_routes.borrow().iter())
        {
            affected |= (child.invalidate_observed)(changed);
        }
        if affected {
            presentation.cache.apply_update(ViewUpdate::Rebuild);
        }
        affected
    }

    pub(super) fn bind_model_wake(&self, runtime: &ModelRuntime) {
        self.0.model.runtime.inherit_wake(runtime);
        let children: Vec<_> = self
            .0
            .presentation
            .children
            .borrow()
            .iter()
            .chain(self.0.presentation.event_routes.borrow().iter())
            .cloned()
            .collect();
        for child in children {
            (child.bind_model_wake)(runtime);
        }
    }

    pub(super) fn collect_model_runtimes(&self, visited: &mut Vec<ModelRuntime>) {
        self.0.model.runtime.collect_linked(visited);
        for child in self
            .0
            .presentation
            .children
            .borrow()
            .iter()
            .chain(self.0.presentation.event_routes.borrow().iter())
        {
            (child.model_runtimes)(visited);
        }
    }

    pub(super) fn collect_model_effects(&self) -> ContextEffects {
        let children: Vec<_> = self
            .0
            .presentation
            .children
            .borrow()
            .iter()
            .chain(self.0.presentation.event_routes.borrow().iter())
            .cloned()
            .collect();
        let mut effects = self.take_effects();
        for child in children {
            merge_effects(&mut effects, (child.model_effects)());
        }
        self.0.presentation.cache.apply_update(effects.update);
        self.0.presentation.visible_effects(effects)
    }
}

impl AnyEntity {
    /// Publishes `snapshot` to all current event handlers before dispatch.
    pub(crate) fn set_interaction_snapshot(&self, snapshot: &super::InteractionSnapshot) {
        (self.set_interaction_snapshot)(snapshot);
    }

    /// Adds observed source identities in this presentation tree to `identities`.
    pub(crate) fn collect_observed(&self, identities: &mut HashSet<RetainedIdentity>) {
        (self.collect_observed)(identities);
    }

    /// Invalidates presentation caches reading identities in `changed`.
    /// Returns whether any presentation was affected.
    pub(crate) fn invalidate_observed(&self, changed: &HashSet<RetainedIdentity>) -> bool {
        (self.invalidate_observed)(changed)
    }

    pub(crate) fn set_model_wake(&self, wake: impl Fn() + 'static) {
        self.runtime.set_wake(wake);
        (self.bind_model_wake)(&self.runtime);
    }

    pub(crate) fn collect_model_runtimes(&self, visited: &mut Vec<ModelRuntime>) {
        (self.model_runtimes)(visited);
    }

    pub(crate) fn take_model_effects(&self) -> ContextEffects {
        (self.model_effects)()
    }
}

/// Ends an application's presentations and executor before releasing its windows.
/// Retained models remain readable; their old executor rejects new work.
/// Queued model deliveries are cancelled; final lifecycle records are delivered
/// outside rendering. All roots are closed even if cleanup panics.
///
/// `roots` are the entity roots whose related presentations and runtimes should stop.
///
/// # Panics
/// Resumes the first panic raised while closing a presentation or delivering final lifecycle records.
pub fn shutdown_presentations(roots: &[AnyEntity]) {
    let mut runtimes = Vec::new();
    for root in roots {
        root.collect_model_runtimes(&mut runtimes);
    }
    #[cfg(feature = "tasks")]
    for runtime in &runtimes {
        if let Some(tasks) = runtime.task_runtime() {
            tasks.shutdown();
        }
    }
    for runtime in &runtimes {
        runtime.detach_host();
    }
    let mut panic = None;
    for root in roots {
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| root.close_presentation()));
        if panic.is_none() {
            panic = result.err();
        }
    }
    for runtime in &runtimes {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            runtime.lifecycle().finish()
        }));
        runtime.detach_host();
        if panic.is_none() {
            panic = result.err();
        }
    }
    if let Some(payload) = panic
        && !std::thread::panicking()
    {
        std::panic::resume_unwind(payload);
    }
}
