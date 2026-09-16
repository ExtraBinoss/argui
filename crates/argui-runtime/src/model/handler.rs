use std::rc::Rc;

use argui_ui::{
    EventHandler, EventHandlerId, EventListener, EventType, FromHandlerValue, UiEvent, ValueHandler,
};

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
    fn register_handler(
        &mut self,
        handler: impl Fn(&mut T, &UiEvent, &mut Context<T>) + 'static,
    ) -> EventHandler {
        let owner = self.owner.as_ref().map_or_else(
            || panic!("event handlers require an entity render context"),
            |owner| owner.0.get(),
        );
        let entity = self
            .entity
            .as_ref()
            .and_then(super::WeakEntity::upgrade)
            .expect("event handlers require a live entity render context");
        let slot = entity
            .0
            .presentation
            .handlers
            .borrow()
            .next_slot(self.handlers.len());
        self.handlers.push(Rc::new(handler));
        EventHandler::from_identity(EventHandlerId::new(argui_ui::EventOwnerId(owner), slot))
    }

    /// Registers a local state callback that invalidates this presentation after delivery.
    ///
    /// `callback` receives mutable application state. The returned opaque handler can be
    /// attached by a widget such as `Button::on_click`. The runtime calls
    /// [`Context::notify`] after `callback` returns.
    ///
    /// # Panics
    /// Panics if called outside a live entity render context.
    #[must_use]
    pub fn callback(&mut self, callback: impl Fn(&mut T) + 'static) -> EventHandler {
        self.register_handler(move |model, _, cx| {
            callback(model);
            cx.notify();
        })
    }

    /// Registers a full event callback without implicitly invalidating the presentation.
    ///
    /// `handler` receives mutable application state, the routed [`UiEvent`], and the
    /// presentation [`Context`]. It may inspect or stop propagation and request commands,
    /// focus, or invalidation explicitly. The returned opaque identity is bound to an event
    /// type by the widget that receives it.
    ///
    /// # Panics
    /// Panics if called outside a live entity render context.
    #[must_use]
    pub fn event_handler(
        &mut self,
        handler: impl Fn(&mut T, &UiEvent, &mut Context<T>) + 'static,
    ) -> EventHandler {
        self.register_handler(handler)
    }

    fn mapped_handler<V: 'static>(
        &mut self,
        map: impl Fn(&UiEvent) -> Option<V> + 'static,
        handler: impl Fn(&mut T, V, &UiEvent, &mut Context<T>) + 'static,
    ) -> ValueHandler<V> {
        ValueHandler::from_handler(self.register_handler(move |model, event, cx| {
            if let Some(value) = map(event) {
                handler(model, value, event, cx);
            }
        }))
    }

    /// Registers a typed callback for a value supplied by a widget listener.
    ///
    /// `callback` receives mutable application state plus the widget's domain
    /// value. The presentation is invalidated after the callback returns. A
    /// delivery without a compatible value is ignored.
    ///
    /// # Panics
    /// Panics if called outside a live entity render context.
    #[must_use]
    pub fn value_callback<V: FromHandlerValue>(
        &mut self,
        callback: impl Fn(&mut T, V) + 'static,
    ) -> ValueHandler<V> {
        self.mapped_handler(V::from_handler_event, move |model, value, _, cx| {
            callback(model, value);
            cx.notify();
        })
    }

    /// Registers a typed full event handler for a widget-supplied value.
    ///
    /// `handler` receives the domain value, routed event, and context. It must
    /// request invalidation explicitly when required. A delivery without a
    /// compatible value is ignored.
    ///
    /// # Panics
    /// Panics if called outside a live entity render context.
    #[must_use]
    pub fn value_event_handler<V: FromHandlerValue>(
        &mut self,
        handler: impl Fn(&mut T, V, &UiEvent, &mut Context<T>) + 'static,
    ) -> ValueHandler<V> {
        self.mapped_handler(V::from_handler_event, handler)
    }

    /// Registers a text-input callback receiving the newly edited string.
    ///
    /// `callback` receives mutable application state and the new controlled
    /// value. Non-input events are ignored. Successful delivery automatically
    /// invalidates the presentation.
    ///
    /// # Panics
    /// Panics if called outside a live entity render context.
    #[must_use]
    pub fn input_callback(
        &mut self,
        callback: impl Fn(&mut T, String) + 'static,
    ) -> ValueHandler<String> {
        self.value_callback(callback)
    }

    /// Registers a submit callback receiving the submitted string.
    ///
    /// `callback` receives mutable application state and the submitted value.
    /// Non-submit events are ignored. Successful delivery automatically
    /// invalidates the presentation.
    ///
    /// # Panics
    /// Panics if called outside a live entity render context.
    #[must_use]
    pub fn submit_callback(
        &mut self,
        callback: impl Fn(&mut T, String) + 'static,
    ) -> ValueHandler<String> {
        self.value_callback(callback)
    }

    /// Registers a text-input handler with access to the routed event and context.
    ///
    /// `handler` receives the edited string and does not implicitly invalidate.
    ///
    /// # Panics
    /// Panics if called outside a live entity render context.
    #[must_use]
    pub fn input_event_handler(
        &mut self,
        handler: impl Fn(&mut T, String, &UiEvent, &mut Context<T>) + 'static,
    ) -> ValueHandler<String> {
        self.value_event_handler(handler)
    }

    /// Registers a submit handler with access to the routed event and context.
    ///
    /// `handler` receives the submitted string and does not implicitly invalidate.
    ///
    /// # Panics
    /// Panics if called outside a live entity render context.
    #[must_use]
    pub fn submit_event_handler(
        &mut self,
        handler: impl Fn(&mut T, String, &UiEvent, &mut Context<T>) + 'static,
    ) -> ValueHandler<String> {
        self.value_event_handler(handler)
    }

    /// Creates an action binding whose callback runs when the matching action fires.
    ///
    /// `id` selects the action; `state` describes its current action state;
    /// `handler` updates the component in response to action events.
    /// Returns an action binding that pairs the metadata with its listener.
    ///
    /// # Panics
    /// Panics if called outside a live entity render context.
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
    ///
    /// `event` selects the event type; `handler` handles matching events.
    /// Returns the listener declaration for the current event-handler slot.
    ///
    /// # Panics
    /// Panics if called outside a live entity render context.
    #[must_use]
    pub fn listener(
        &mut self,
        event: EventType,
        handler: impl Fn(&mut T, &UiEvent, &mut Context<T>) + 'static,
    ) -> EventListener {
        self.event_handler(handler).listener(event)
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
            environment: self.0.presentation.environment.borrow().clone(),
            event_target: Some(event.current_target()),
            ..Context::default()
        };
        callback(&mut self.0.model.value.borrow_mut(), event, &mut cx);
        self.0.model.signal.apply_update(cx.effects.update);
        cx.effects
    }
}
