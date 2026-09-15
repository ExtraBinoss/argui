use super::{Entity, EntityCell, EntityId, Presentation, Render, ResourceScope, WindowEnvironment};
use argui_ui::{Element, UiEvent};
use std::rc::{Rc, Weak};

/// Identity of one retained presentation, distinct from the shared model identity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MountId(pub(super) usize);
impl MountId {
    #[must_use]
    /// Returns the numeric value of this presentation identity.
    pub fn get(self) -> usize {
        self.0
    }
}

/// An independently retained presentation of a shared model. Cloning this handle
/// retains this same mount; calling `Entity::mount` creates a different one.
pub struct Mount<T> {
    pub(crate) entity: Entity<T>,
}

impl<T: 'static> super::Context<T> {
    /// Identity of the presentation owning this context, absent for a detached context.
    #[must_use]
    pub fn mount_id(&self) -> Option<MountId> {
        self.entity
            .as_ref()
            .and_then(super::WeakEntity::upgrade)
            .map(|entity| MountId(entity.0.presentation.id.get()))
    }
}

impl<T: Render> super::Context<T> {
    /// Delivers layout only to this parent's retained presentations of `model`.
    ///
    /// `model` identifies the child model; `layout` contains the new layout snapshot.
    ///
    /// # Panics
    /// Panics when called outside a retained parent context.
    pub fn layout_entity<U: Render>(&mut self, model: &Entity<U>, layout: &super::LayoutSnapshot) {
        let parent = self
            .entity
            .as_ref()
            .and_then(super::WeakEntity::upgrade)
            .expect("child layout requires a retained parent context");
        let children: Vec<_> = parent
            .0
            .presentation
            .children
            .borrow()
            .iter()
            .chain(parent.0.presentation.event_routes.borrow().iter())
            .filter(|child| child.model_id == model.id())
            .cloned()
            .collect();
        for child in children {
            self.merge_effects((child.layout)(layout));
        }
    }

    pub(super) fn child_presentation<U: Render>(&self, model: &Entity<U>) -> Entity<U> {
        let parent = self
            .entity
            .as_ref()
            .and_then(super::WeakEntity::upgrade)
            .expect("child views require a retained parent context");
        let existing = parent
            .0
            .presentation
            .children
            .borrow()
            .iter()
            .find(|child| {
                child.model_id == model.id()
                    && !self.effects.children.iter().any(|used| used.ptr_eq(child))
            })
            .cloned();
        if let Some(existing) = existing {
            let cell = existing
                .identity
                .downcast::<EntityCell<U>>()
                .unwrap_or_else(|_| panic!("model identity changed its data type"));
            return Entity(cell);
        }
        let child = model
            .mount()
            .expect("cannot mount a closed child model")
            .entity;
        let resources = child.0.presentation.resources.clone();
        let lease = parent
            .0
            .presentation
            .resources
            .defer(move || resources.close())
            .expect("cannot mount a child in a closed parent");
        *child.0.presentation.parent_lease.borrow_mut() = Some(lease);
        child
    }
}
impl<T> Clone for Mount<T> {
    fn clone(&self) -> Self {
        Self {
            entity: self.entity.clone(),
        }
    }
}

/// Does not retain the presentation or resurrect it through its model.
pub struct WeakMount<T>(Weak<EntityCell<T>>);
impl<T> Clone for WeakMount<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T> WeakMount<T> {
    #[must_use]
    /// Upgrades to a strong handle if this exact presentation is still alive.
    pub fn upgrade(&self) -> Option<Mount<T>> {
        self.0.upgrade().map(|cell| Mount {
            entity: Entity(cell),
        })
    }
}

impl<T: Render> Entity<T> {
    /// Creates a new independently retained presentation of this shared model.
    ///
    /// # Errors
    /// Returns [`super::ScopeClosed`] if the model's resources have already closed.
    pub fn mount(&self) -> Result<Mount<T>, super::ScopeClosed> {
        let _transaction = self.0.model.runtime.enter();
        let presentation = Presentation::new(self.0.model.signal.clone());
        let resources = presentation.resources.clone();
        *presentation.model_lease.borrow_mut() =
            Some(self.resources().defer(move || resources.close())?);
        let mount = Mount {
            entity: Entity(Rc::new(EntityCell {
                model: self.0.model.clone(),
                presentation,
            })),
        };
        let weak = mount.downgrade();
        let cleanup = mount.resources().defer(move || {
            if let Some(mount) = weak.upgrade() {
                mount.entity.0.presentation.clear();
            }
        })?;
        *mount.entity.0.presentation.cleanup_lease.borrow_mut() = Some(cleanup);
        mount.entity.0.presentation.lifecycle.start();
        Ok(mount)
    }
}

impl<T: 'static> Mount<T> {
    /// Advance one bounded batch in each model domain currently attached to this
    /// presentation. Custom hosts call this on a model wake, outside rendering.
    ///
    /// # Panics
    /// Propagates a panic raised by a dispatched listener or lifecycle callback.
    pub fn dispatch_models(&self) {
        let mut runtimes = Vec::new();
        self.entity.collect_model_runtimes(&mut runtimes);
        for runtime in runtimes {
            runtime.dispatch_pending();
        }
    }

    /// Read the shared model retained by this handle, even after unmounting.
    /// This does not grant access to the presentation's services.
    ///
    /// `read` computes a result from the shared model value; that result is returned.
    ///
    /// # Panics
    /// Propagates a panic raised by `read` or a conflicting reentrant mutable borrow.
    pub fn read<R>(&self, read: impl FnOnce(&T) -> R) -> R {
        self.entity.read(read)
    }

    /// Updates the model through this presentation's context.
    ///
    /// `update` receives mutable model state and context; its result is returned when
    /// the presentation is open.
    ///
    /// # Errors
    /// Returns [`super::ScopeClosed`] if this presentation has closed.
    ///
    /// # Panics
    /// Propagates a panic raised by `update` or a conflicting model borrow.
    pub fn update<R>(
        &self,
        update: impl FnOnce(&mut T, &mut super::Context<T>) -> R,
    ) -> Result<R, super::ScopeClosed> {
        if self.resources().is_closed() {
            return Err(super::ScopeClosed);
        }
        Ok(self.entity.update_view(update))
    }
    /// Ends this presentation immediately, even if handles remain retained.
    ///
    /// # Panics
    /// Resumes the first panic raised by a registered cleanup after cleanup completes.
    pub fn close(&self) {
        let _transaction = self.entity.0.model.runtime.enter();
        self.resources().close();
    }
    #[must_use]
    /// Returns this presentation's identity.
    pub fn id(&self) -> MountId {
        MountId(self.entity.0.presentation.id.get())
    }
    #[must_use]
    /// Returns the identity of the shared model presented by this mount.
    pub fn model_id(&self) -> EntityId {
        self.entity.id()
    }
    #[must_use]
    /// Returns the resource scope owned by this presentation.
    pub fn resources(&self) -> &ResourceScope {
        &self.entity.0.presentation.resources
    }
    #[must_use]
    /// Creates a weak handle that does not retain this presentation.
    pub fn downgrade(&self) -> WeakMount<T> {
        WeakMount(Rc::downgrade(&self.entity.0))
    }
}

impl<T: Render> Mount<T> {
    /// Retains this exact presentation for a type-erased host.
    /// Returns a cloneable handle that preserves presentation identity.
    pub fn erase(&self) -> super::AnyEntity {
        self.entity.erase()
    }

    /// Deliver a frame to this live presentation using its window context.
    ///
    /// `frame` describes the animation step being delivered.
    ///
    /// # Errors
    /// Returns [`super::ScopeClosed`] if the presentation has closed.
    ///
    /// # Panics
    /// Propagates a panic raised by the component's animation callback.
    pub fn animation_frame(&self, frame: argui_animation::Frame) -> Result<(), super::ScopeClosed> {
        if self.resources().is_closed() {
            return Err(super::ScopeClosed);
        }
        self.entity.animation_frame(frame);
        Ok(())
    }

    /// Delivers a new layout snapshot to this presentation.
    ///
    /// `layout` contains the current geometry.
    ///
    /// # Errors
    /// Returns [`super::ScopeClosed`] if the presentation has closed.
    ///
    /// # Panics
    /// Propagates a panic raised by the component's layout callback.
    pub fn layout_changed(&self, layout: &super::LayoutSnapshot) -> Result<(), super::ScopeClosed> {
        if self.resources().is_closed() {
            return Err(super::ScopeClosed);
        }
        self.entity.layout_changed(layout);
        Ok(())
    }

    /// Renders this presentation in `environment`.
    ///
    /// Returns the retained subtree, or an empty container if the mount is hidden.
    ///
    /// # Errors
    /// Returns [`super::ScopeClosed`] if the presentation has closed.
    ///
    /// # Panics
    /// Propagates a panic raised while rendering the component.
    pub fn render(&self, environment: WindowEnvironment) -> Result<Element, super::ScopeClosed> {
        if self.resources().is_closed() {
            return Err(super::ScopeClosed);
        }
        Ok(self.entity.render_in(environment))
    }
    /// Dispatches the current listener in `event` to this presentation.
    ///
    /// # Errors
    /// Returns [`super::ScopeClosed`] if the presentation has closed.
    ///
    /// # Panics
    /// Propagates a panic raised by the event handler.
    pub fn dispatch_event(&self, event: &UiEvent) -> Result<(), super::ScopeClosed> {
        if self.resources().is_closed() {
            return Err(super::ScopeClosed);
        }
        self.entity.dispatch_event(event);
        Ok(())
    }
}
