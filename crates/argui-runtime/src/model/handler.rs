use std::rc::Rc;

use argui_ui::{EventHandlerId, EventListener, EventType, UiEvent};

use super::{Context, Entity, Render};

pub(super) type LocalHandler<T> = Rc<dyn Fn(&mut T, &UiEvent, &mut Context<T>)>;

pub(super) struct HandlerRegistry<T> {
    slots: Vec<LocalHandler<T>>,
}

impl<T> Default for HandlerRegistry<T> {
    fn default() -> Self {
        Self { slots: Vec::new() }
    }
}

impl<T> HandlerRegistry<T> {
    pub(super) fn replace(&mut self, handlers: Vec<LocalHandler<T>>) {
        self.slots = handlers;
    }

    pub(super) fn get(&self, slot: u32) -> Option<LocalHandler<T>> {
        self.slots.get(slot as usize).cloned()
    }
}

impl<T: Render> Context<T> {
    /// Declares an event handler owned by the component currently being rendered.
    #[must_use]
    pub fn listener(
        &mut self,
        event: EventType,
        handler: impl Fn(&mut T, &UiEvent, &mut Context<T>) + 'static,
    ) -> EventListener {
        let owner = self.owner.as_ref().map_or_else(
            || panic!("listeners require an entity render context"),
            |owner| owner.0,
        );
        let slot = u32::try_from(self.handlers.len()).expect("handler slot count must fit in u32");
        self.handlers.push(Rc::new(handler));
        EventListener::new(
            event,
            EventHandlerId::new(argui_ui::EventOwnerId(owner), slot),
        )
    }
}

impl<T: Render> Entity<T> {
    pub(super) fn dispatch_handler(
        &self,
        handler: EventHandlerId,
        event: &UiEvent,
    ) -> super::effects::ContextEffects {
        if handler.owner() != self.owner_id() {
            if let Some(child) = self
                .0
                .children
                .borrow()
                .iter()
                .find(|child| (child.owns)(handler.owner()))
            {
                return (child.dispatch_handler)(handler, event);
            }
            return self
                .0
                .event_routes
                .borrow()
                .iter()
                .find(|route| (route.owns)(handler.owner()))
                .map_or_else(super::effects::ContextEffects::default, |route| {
                    (route.dispatch_handler)(handler, event)
                });
        }
        let Some(callback) = self.0.handlers.borrow().get(handler.slot()) else {
            return super::effects::ContextEffects::default();
        };
        let mut cx = Context {
            environment: self.0.environment.get(),
            event_target: Some(event.current_target()),
            ..Context::default()
        };
        callback(&mut self.0.value.borrow_mut(), event, &mut cx);
        self.0.cache.apply_update(cx.effects.update);
        cx.effects
    }
}
