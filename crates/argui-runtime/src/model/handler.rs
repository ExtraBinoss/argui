use std::rc::Rc;

use argui_ui::{EventHandlerId, EventListener, EventType, UiEvent};

use super::{Context, Entity, Render};

pub(super) type LocalHandler<T> = Rc<dyn Fn(&mut T, &UiEvent, &mut Context<T>)>;

pub(super) struct HandlerRegistry<T> {
    slots: Vec<LocalHandler<T>>,
    ids: Vec<u32>,
    next: u32,
}

impl<T> Default for HandlerRegistry<T> {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            ids: Vec::new(),
            next: 0,
        }
    }
}

impl<T> HandlerRegistry<T> {
    fn next_slot(&self, index: usize) -> u32 {
        self.ids.get(index).copied().unwrap_or_else(|| {
            self.next
                .checked_add(
                    u32::try_from(index - self.ids.len()).expect("handler count must fit in u32"),
                )
                .expect("handler identity space exhausted")
        })
    }
    pub(super) fn replace(&mut self, handlers: Vec<LocalHandler<T>>) {
        self.ids.truncate(handlers.len());
        while self.ids.len() < handlers.len() {
            self.ids.push(self.next);
            self.next = self
                .next
                .checked_add(1)
                .expect("handler identity space exhausted");
        }
        self.slots = handlers;
    }

    pub(super) fn get(&self, slot: u32) -> Option<LocalHandler<T>> {
        let index = self.ids.binary_search(&slot).ok()?;
        Some(self.slots[index].clone())
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
    /// Declares a handler slot for this render. Removed slots are never reused.
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
        let entity = self
            .entity
            .as_ref()
            .and_then(super::WeakEntity::upgrade)
            .expect("listeners require a live entity render context");
        let slot = entity
            .0
            .presentation
            .handlers
            .borrow()
            .next_slot(self.handlers.len());
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
