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
    #[must_use]
    pub fn on_action(
        &mut self,
        id: argui_ui::ActionId,
        state: argui_ui::ActionState,
        handler: impl Fn(&mut T, &UiEvent, &mut Context<T>) + 'static,
    ) -> argui_ui::ActionBinding {
        argui_ui::ActionBinding {
            id,
            state,
            listener: self.listener(EventType::Action, handler),
        }
    }
    /// Declares an event handler owned by the component currently being rendered.
    #[must_use]
    pub fn listener(
        &mut self,
        event: EventType,
        handler: impl Fn(&mut T, &UiEvent, &mut Context<T>) + 'static,
    ) -> EventListener {
        let owner = self.owner.as_ref().map_or_else(
            || panic!("listeners require an entity render context"),
            |owner| owner.0.get(),
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
        let _transaction = self.0.model.runtime.enter();
        if !self.0.presentation.is_visible() {
            return super::effects::ContextEffects::default();
        }
        if handler.owner() != self.owner_id() {
            let child = self
                .0
                .presentation
                .children
                .borrow()
                .iter()
                .find(|child| (child.owns)(handler.owner()))
                .cloned();
            if let Some(child) = child {
                return (child.dispatch_handler)(handler, event);
            }
            let route = self
                .0
                .presentation
                .event_routes
                .borrow()
                .iter()
                .find(|route| (route.owns)(handler.owner()))
                .cloned();
            return route.map_or_else(super::effects::ContextEffects::default, |route| {
                (route.dispatch_handler)(handler, event)
            });
        }
        let Some(callback) = self.0.presentation.handlers.borrow().get(handler.slot()) else {
            return super::effects::ContextEffects::default();
        };
        let mut cx = Context {
            entity: Some(self.downgrade()),
            environment: self.0.presentation.environment.get(),
            event_target: Some(event.current_target()),
            ..Context::default()
        };
        callback(&mut self.0.model.value.borrow_mut(), event, &mut cx);
        self.0.model.signal.apply_update(cx.effects.update);
        cx.effects
    }
}
