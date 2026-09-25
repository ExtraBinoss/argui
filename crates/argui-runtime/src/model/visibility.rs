use super::{
    Context, Entity, Mount, Render, ScopeClosed, ViewUpdate, WeakEntity, inherit_environment_use,
    inherit_theme_reads,
};
use argui_ui::Element;

impl<T> super::Presentation<T> {
    pub(super) fn visible_effects(&self, effects: super::ContextEffects) -> super::ContextEffects {
        if self.is_visible() {
            effects
        } else {
            super::ContextEffects {
                commands: effects.commands,
                ..Default::default()
            }
        }
    }

    pub(super) fn is_visible(&self) -> bool {
        self.visible.get() && self.host_visible.get() && !self.resources.is_closed()
    }
}

impl<T: 'static> Mount<T> {
    /// Whether this live mount is shown. A hidden mount keeps its identity,
    /// subscriptions and tasks; closing it is a separate, irreversible operation.
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.entity.0.presentation.is_visible()
    }

    /// Changes presentation visibility without cancelling any work. Repeated
    /// requests are inert. Showing renders the latest data on the next frame.
    ///
    /// `visible` selects whether this mount should be shown.
    ///
    /// # Errors
    /// Returns [`ScopeClosed`] if this mount has already closed.
    pub fn set_visible(&self, visible: bool) -> Result<(), ScopeClosed> {
        if self.resources().is_closed() {
            return Err(ScopeClosed);
        }
        let _transaction = self.entity.0.model.runtime.enter();
        let before = self.is_visible();
        if self.entity.0.presentation.visible.replace(visible) != visible {
            if before != self.is_visible() {
                self.entity
                    .0
                    .presentation
                    .lifecycle
                    .visibility(self.is_visible());
            }
            self.entity
                .0
                .presentation
                .cache
                .apply_update(ViewUpdate::Rebuild);
        }
        Ok(())
    }
}

impl<T: Render> Context<T> {
    /// Renders a child entity and reuses its exact subtree while it is clean.
    ///
    /// `entity` is the child model to present. Returns its current retained subtree.
    pub fn entity<U: Render>(&mut self, entity: &Entity<U>) -> Element {
        self.entity_visible(entity, true)
    }

    /// Renders a child with an environment inherited by its descendants.
    ///
    /// `entity` is the child model to present; `environment` is the environment for
    /// this child subtree. Returns the rendered child subtree.
    pub fn entity_in<U: Render>(
        &mut self,
        entity: &Entity<U>,
        environment: crate::WindowEnvironment,
    ) -> Element {
        let parent = std::mem::replace(&mut self.environment, environment);
        let element = self.entity(entity);
        self.environment = parent;
        element
    }

    /// Retains a child while hidden, without rendering or accepting UI input.
    /// Omitting this call on a later render unmounts the child instead.
    ///
    /// `entity` is the child model; `visible` controls whether its subtree is shown.
    /// Returns the retained subtree, which is empty while hidden.
    pub fn entity_visible<U: Render>(&mut self, entity: &Entity<U>, visible: bool) -> Element {
        let entity = self.child_presentation(entity);
        let before = entity.0.presentation.is_visible();
        entity.0.presentation.visible.set(visible);
        if before != entity.0.presentation.is_visible() {
            entity
                .0
                .presentation
                .lifecycle
                .visibility(entity.0.presentation.is_visible());
        }
        if let Some(owner) = self.entity.as_ref().and_then(WeakEntity::upgrade) {
            entity.bind_model_wake(&owner.0.model.runtime);
        }
        #[cfg(feature = "tasks")]
        self.inherit_tasks(&entity);
        let element = entity
            .render_with_observations(self.environment.clone(), self.observations.borrow().clone());
        inherit_environment_use(
            &self.environment_read,
            &entity.0.presentation.environment_used,
        );
        inherit_theme_reads(&self.theme_reads, &entity.0.presentation.theme_reads);
        if visible && let Some((_, observer)) = &self.owner {
            self.dependencies
                .push(entity.0.model.signal.subscribe(observer.clone()));
            self.dependencies
                .push(entity.0.presentation.cache.subscribe(observer.clone()));
        }
        self.effects.children.push(entity.erase());
        element
    }
}

impl<T: 'static> Entity<T> {
    pub(super) fn host_visibility(&self, visible: bool) {
        let presentation = &self.0.presentation;
        let before = presentation.is_visible();
        presentation.host_visible.set(visible);
        let after = presentation.is_visible();
        if before != after {
            presentation.lifecycle.visibility(after);
            presentation.cache.apply_update(ViewUpdate::Rebuild);
        }
        let children: Vec<_> = presentation
            .children
            .borrow()
            .iter()
            .chain(presentation.event_routes.borrow().iter())
            .cloned()
            .collect();
        for child in children {
            child.set_host_visible(visible);
        }
    }

    pub(super) fn close_host(&self) {
        if self.0.presentation.resources.is_closed() {
            return;
        }
        let children: Vec<_> = self
            .0
            .presentation
            .children
            .borrow()
            .iter()
            .chain(self.0.presentation.event_routes.borrow().iter())
            .cloned()
            .collect();
        let mut panic = None;
        for child in children {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                child.close_presentation()
            }));
            if panic.is_none() {
                panic = result.err();
            }
        }
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.0.presentation.resources.close()
        }));
        self.0.presentation.clear();
        if let Some(payload) = panic.or(result.err())
            && !std::thread::panicking()
        {
            std::panic::resume_unwind(payload);
        }
    }
}

impl super::AnyEntity {
    /// Applies window visibility independently of each child's local hide/show state.
    ///
    /// `visible` is the host-level visibility to apply to this presentation and its
    /// descendants.
    pub fn set_host_visible(&self, visible: bool) {
        (self.host_visibility)(visible);
    }
    /// Closes this presentation and routed presentations, even if external handles remain.
    /// Data models and their resources retain their separate ownership.
    ///
    /// # Panics
    /// Resumes the first panic raised while closing a presentation or its resources.
    pub fn close_presentation(&self) {
        (self.close_host)();
    }
}
