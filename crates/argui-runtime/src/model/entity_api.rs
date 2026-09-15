use std::rc::Rc;

use argui_animation::Frame;
use argui_inspect::InspectorHandle;
use argui_paint::{ImageAsset, VectorAsset};
use argui_ui::{Element, EventHandlerId, UiEvent};

use super::{
    AnyEntity, ContextEffects, Entity, EntityCell, LayoutSnapshot, Presentation, WeakEntity,
    WindowEnvironment,
};

impl<T: 'static> WeakEntity<T> {
    /// Upgrades this weak reference while its model or presentation is alive.
    /// Returns `None` after both have been dropped.
    #[must_use]
    pub fn upgrade(&self) -> Option<Entity<T>> {
        if let Some(presentation) = self.presentation.upgrade() {
            return Some(Entity(presentation));
        }
        self.model.upgrade().map(|model| {
            Entity(Rc::new(EntityCell {
                presentation: Presentation::new(model.signal.clone()),
                model,
            }))
        })
    }
}

impl AnyEntity {
    /// Tests whether two erased entities refer to the same retained identity.
    ///
    /// `other` is the entity to compare with this one.
    #[must_use]
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.identity, &other.identity)
    }

    pub(crate) fn render(&self, environment: WindowEnvironment) -> Element {
        (self.render)(environment)
    }

    pub(crate) fn event(&self, event: &UiEvent) {
        with_event_handler(event, &mut |handler| {
            (self.store)((self.dispatch_handler)(handler, event));
        });
    }

    pub(crate) fn animation_frame(&self, frame: Frame) {
        (self.store)((self.frame)(frame));
    }

    pub(crate) fn layout_changed(&self, layout: &LayoutSnapshot) {
        (self.store)((self.layout)(layout));
    }

    pub(crate) fn wants_frame(&self) -> bool {
        (self.wants_frame)()
    }

    pub(crate) fn image_assets(&self) -> Vec<ImageAsset> {
        (self.image_assets)()
    }

    pub(crate) fn vector_assets(&self) -> Vec<VectorAsset> {
        (self.vector_assets)()
    }

    pub(crate) fn inspector(&self) -> Option<InspectorHandle> {
        (self.inspector)()
    }

    pub(crate) fn take_effects(&self) -> ContextEffects {
        (self.take_effects)()
    }
}

pub(super) fn with_event_handler(event: &UiEvent, dispatch: &mut dyn FnMut(EventHandlerId)) {
    if let Some(handler) = event.current_handler() {
        dispatch(handler);
    }
}
