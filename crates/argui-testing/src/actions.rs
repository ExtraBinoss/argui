use argui_accessibility::{Role, SemanticAction, SemanticValue};
use argui_core::{
    Key, KeyInput, KeyState, Modifiers, Point, PointerEvent, PointerId, PointerKind, PointerPhase,
    ScrollDelta, Size,
};
use argui_platform::PlatformEvent;
use argui_runtime::{AppEvent, AppModel, Render, WindowEnvironment};
use argui_ui::{FocusRequest, InteractionUpdate, UiEventKind};

use crate::{Selector, SemanticMatcher, TestApp, TestError, TestNode};

mod click;

impl<A: Render> TestApp<A> {
    /// Returns an operation handle for a unique application `key`.
    #[must_use]
    pub fn get_by_key(&mut self, key: impl Into<String>) -> TestNode<'_, A> {
        TestNode::new(self, Selector::key(key))
    }

    /// Returns an operation handle for a unique accessible `role` and exact `name`.
    #[must_use]
    pub fn get_by_role(&mut self, role: Role, name: impl Into<String>) -> TestNode<'_, A> {
        TestNode::new(self, Selector::role(role, name))
    }

    /// Returns an operation handle for unique visible `text`.
    #[must_use]
    pub fn get_by_text(&mut self, text: impl Into<String>) -> TestNode<'_, A> {
        TestNode::new(self, Selector::text(text))
    }

    /// Returns an operation handle for a unique accessible `label`.
    #[must_use]
    pub fn get_by_label(&mut self, label: impl Into<String>) -> TestNode<'_, A> {
        TestNode::new(self, Selector::label(label))
    }

    /// Returns an operation handle for a unique accessible semantic `state`.
    #[must_use]
    pub fn get_by_state(&mut self, state: SemanticMatcher) -> TestNode<'_, A> {
        TestNode::new(self, Selector::state(state))
    }

    /// Returns an operation handle for the currently focused element.
    #[must_use]
    pub fn focused(&mut self) -> TestNode<'_, A> {
        TestNode::new(self, Selector::Focused)
    }

    /// Taps `selector` with a primary touch contact through gesture and hit testing.
    ///
    /// # Errors
    ///
    /// Returns a selector, bounds, layout, or stabilization error.
    pub fn tap(&mut self, selector: impl Into<Selector>) -> Result<(), TestError> {
        let selector = selector.into();
        let point = self.center(&selector)?;
        let regions = self.output().hit_regions.clone();
        for phase in [PointerPhase::Pressed, PointerPhase::Released] {
            let event = PointerEvent {
                id: PointerId::new(1),
                kind: PointerKind::Touch,
                phase,
                position: point,
                button: None,
                buttons: 0,
                pressure: Some(if phase == PointerPhase::Pressed {
                    1.0
                } else {
                    0.0
                }),
                primary: true,
                modifiers: Modifiers::default(),
                timestamp: std::time::Duration::from_nanos(self.now.as_nanos()),
            };
            let update = self.ui_mut().pointer_event(event, &regions);
            let deliveries = update.events.clone();
            self.dispatch_interaction(update)?;
            if phase == PointerPhase::Pressed
                && deliveries.iter().all(|event| !event.default_prevented())
            {
                let focus = self
                    .ui_mut()
                    .focus_pointer_default(PointerId::new(1), &regions);
                self.dispatch_interaction(focus)?;
            }
        }
        self.settle()
    }

    /// Moves the mouse pointer to `point` and delivers enter, leave, and move events.
    ///
    /// # Errors
    ///
    /// Returns a stabilization error. Points outside the viewport are rejected as
    /// [`TestError::MissingBounds`].
    pub fn pointer_move(&mut self, point: Point) -> Result<(), TestError> {
        if !self.point_in_viewport(point) {
            return Err(TestError::MissingBounds {
                selector: Selector::Focused,
                tree: self.compact_dump(),
            });
        }
        let regions = self.output().hit_regions.clone();
        let update = self.ui_mut().pointer_moved(point, &regions);
        self.dispatch_interaction(update)?;
        self.settle()
    }

    /// Drags one primary pointer from `start` to `end` through `steps` move samples.
    ///
    /// # Errors
    ///
    /// Returns a stabilization error or rejects coordinates outside the viewport.
    pub fn drag(&mut self, start: Point, end: Point, steps: usize) -> Result<(), TestError> {
        if !self.point_in_viewport(start) || !self.point_in_viewport(end) {
            return Err(TestError::MissingBounds {
                selector: Selector::Focused,
                tree: self.compact_dump(),
            });
        }
        let regions = self.output().hit_regions.clone();
        let pressed = PointerEvent {
            button: Some(argui_core::PointerButton::Primary),
            buttons: 1,
            ..PointerEvent::mouse(PointerPhase::Pressed, start)
        };
        let update = self.ui_mut().pointer_event(pressed, &regions);
        let press_delivery = update
            .events
            .iter()
            .find(|event| {
                matches!(
                    event.kind,
                    UiEventKind::Pointer(PointerEvent {
                        phase: PointerPhase::Pressed,
                        ..
                    })
                )
            })
            .cloned();
        self.dispatch_interaction(update)?;
        let capture = self.ui_mut().pointer_press_default(
            PointerId::MOUSE,
            press_delivery.as_ref(),
            &regions,
        );
        self.dispatch_interaction(capture)?;
        for index in 1..=steps.max(1) {
            let ratio = index as f32 / steps.max(1) as f32;
            let point = Point::new(
                start.x + (end.x - start.x) * ratio,
                start.y + (end.y - start.y) * ratio,
            );
            let moved = PointerEvent {
                buttons: 1,
                ..PointerEvent::mouse(PointerPhase::Moved, point)
            };
            let update = self.ui_mut().pointer_event(moved, &regions);
            self.dispatch_interaction(update)?;
        }
        let released = PointerEvent {
            button: Some(argui_core::PointerButton::Primary),
            ..PointerEvent::mouse(PointerPhase::Released, end)
        };
        let update = self.ui_mut().pointer_event(released, &regions);
        self.dispatch_interaction(update)?;
        self.settle()
    }

    /// Focuses the unique element selected by `selector`.
    ///
    /// # Errors
    ///
    /// Returns a selector or stabilization error.
    pub fn focus(&mut self, selector: impl Into<Selector>) -> Result<(), TestError> {
        let selector = selector.into();
        let node = self.resolve_unique(&selector)?.node;
        let regions = self.output().hit_regions.clone();
        let update = self
            .ui_mut()
            .sync_focus(&regions, Some(FocusRequest::Focus(node.into())));
        self.dispatch_interaction(update)?;
        self.settle()
    }

    /// Clears focus and delivers the resulting blur event.
    ///
    /// # Errors
    ///
    /// Returns a stabilization error.
    pub fn blur(&mut self) -> Result<(), TestError> {
        let regions = self.output().hit_regions.clone();
        let update = self
            .ui_mut()
            .sync_focus(&regions, Some(FocusRequest::Clear));
        self.dispatch_interaction(update)?;
        self.settle()
    }

    /// Sends a complete key press and release to the focused element.
    ///
    /// `key` is normalized platform-independent input; `modifiers` remain active
    /// for both transitions.
    ///
    /// # Errors
    ///
    /// Returns a stabilization error.
    pub fn key(&mut self, key: Key, modifiers: Modifiers) -> Result<(), TestError> {
        self.key_transition(key.clone(), KeyState::Pressed, modifiers, None)?;
        self.key_transition(key, KeyState::Released, modifiers, None)
    }

    /// Sends a key shortcut using the supplied `key` and `modifiers`.
    ///
    /// # Errors
    ///
    /// Returns a stabilization error.
    pub fn shortcut(&mut self, key: Key, modifiers: Modifiers) -> Result<(), TestError> {
        self.key(key, modifiers)
    }

    /// Types Unicode text into the unique text input selected by `selector`.
    ///
    /// # Errors
    ///
    /// Returns a selector error, [`TestError::NotTextInput`], or a stabilization error.
    pub fn type_text(
        &mut self,
        selector: impl Into<Selector>,
        text: &str,
    ) -> Result<(), TestError> {
        let selector = selector.into();
        let resolved = self.resolve_unique(&selector)?;
        self.require_text_input(&selector, &resolved)?;
        self.focus(selector.clone())?;
        for character in text.chars() {
            let text = character.to_string();
            self.key_transition(
                Key::Character(text.clone()),
                KeyState::Pressed,
                Modifiers::default(),
                Some(text),
            )?;
            self.key_transition(
                Key::Character(character.to_string()),
                KeyState::Released,
                Modifiers::default(),
                None,
            )?;
        }
        Ok(())
    }

    /// Replaces the entire controlled text value selected by `selector`.
    ///
    /// # Errors
    ///
    /// Returns a selector, input-kind, or stabilization error.
    pub fn replace_text(
        &mut self,
        selector: impl Into<Selector>,
        value: &str,
    ) -> Result<(), TestError> {
        let selector = selector.into();
        let resolved = self.resolve_unique(&selector)?;
        self.require_text_input(&selector, &resolved)?;
        let update = self.ui_mut().replace_text_input(resolved.node, value);
        self.dispatch_interaction(update)?;
        self.settle()
    }

    /// Pastes `text` into the unique selected text input and records it as clipboard data.
    ///
    /// # Errors
    ///
    /// Returns a selector, input-kind, or stabilization error.
    pub fn paste(&mut self, selector: impl Into<Selector>, text: &str) -> Result<(), TestError> {
        let selector = selector.into();
        let resolved = self.resolve_unique(&selector)?;
        self.require_text_input(&selector, &resolved)?;
        self.clipboard = Some(text.to_owned());
        self.focus(selector)?;
        let update = self.ui_mut().paste_text(Some(resolved.node), text);
        self.dispatch_interaction(update)?;
        self.settle()
    }

    /// Submits the unique selected text input with Enter.
    ///
    /// # Errors
    ///
    /// Returns a selector, input-kind, or stabilization error.
    pub fn submit(&mut self, selector: impl Into<Selector>) -> Result<(), TestError> {
        let selector = selector.into();
        let resolved = self.resolve_unique(&selector)?;
        self.require_text_input(&selector, &resolved)?;
        self.focus(selector)?;
        self.key(Key::Enter, Modifiers::default())
    }

    /// Sends wheel input at `point` through real scroll routing.
    ///
    /// # Errors
    ///
    /// Returns a stabilization error.
    pub fn wheel(&mut self, point: Point, delta: ScrollDelta) -> Result<(), TestError> {
        let regions = self.output().scroll_regions.clone();
        let wheel = self.ui_mut().wheel_event(point, delta, &regions);
        let wheel_event = wheel.events.first().cloned();
        self.dispatch_interaction(wheel)?;
        if wheel_event.is_some_and(|event| event.default_prevented()) {
            return self.settle();
        }
        let scroll = self.ui_mut().scroll(point, delta, &regions);
        if scroll.scroll_changed {
            self.layout = None;
            self.pending = true;
        }
        self.dispatch_interaction(scroll)?;
        self.settle()
    }

    /// Invokes an accessibility `action` on the unique selected node.
    ///
    /// `value` supplies data for actions such as `SetValue`.
    ///
    /// # Errors
    ///
    /// Returns a selector, unsupported-action, or stabilization error.
    pub fn accessibility_action(
        &mut self,
        selector: impl Into<Selector>,
        action: SemanticAction,
        value: Option<SemanticValue>,
    ) -> Result<(), TestError> {
        let selector = selector.into();
        let resolved = self.resolve_unique(&selector)?;
        if action == SemanticAction::Focus {
            return self.focus(selector);
        }
        if action == SemanticAction::Blur {
            return self.blur();
        }
        if action == SemanticAction::SetValue
            && let Some(SemanticValue::Text(value)) = &value
            && matches!(
                resolved.role,
                Role::TextInput | Role::TextArea | Role::SearchInput | Role::ComboBox
            )
        {
            return self.replace_text(selector, value);
        }
        let kind = if action == SemanticAction::Click {
            UiEventKind::Click(argui_ui::ClickEvent::accessibility())
        } else {
            UiEventKind::SemanticAction { action, value }
        };
        let events = self.ui_mut().event_deliveries(resolved.node, kind);
        if events.is_empty() && !resolved.state.disabled {
            return Err(TestError::UnsupportedAction {
                selector,
                action,
                tree: self.compact_dump(),
            });
        }
        self.dispatch_interaction(InteractionUpdate {
            events,
            ..InteractionUpdate::default()
        })?;
        self.settle()
    }

    /// Changes the logical viewport and settles responsive and virtualized content.
    ///
    /// # Errors
    ///
    /// Returns a layout or stabilization error.
    pub fn resize(&mut self, viewport: Size) -> Result<(), TestError> {
        self.viewport = Size::new(viewport.width.max(1.0), viewport.height.max(1.0));
        self.layout = None;
        self.pending = true;
        self.settle()
    }

    /// Replaces the platform environment and settles environment-dependent views.
    ///
    /// # Errors
    ///
    /// Returns a layout or stabilization error.
    pub fn set_environment(&mut self, environment: WindowEnvironment) -> Result<(), TestError> {
        self.environment = environment;
        self.pending = true;
        self.settle()
    }

    /// Delivers one application lifecycle event for the main test window.
    ///
    /// # Errors
    ///
    /// Returns a stabilization error.
    pub fn lifecycle(&mut self, event: PlatformEvent) -> Result<(), TestError> {
        let update = self.model.update(&AppEvent::Window {
            window: self.window.clone(),
            event,
        });
        self.record_app_update(update);
        self.settle()
    }

    /// Advances the injected animation clock without sleeping.
    ///
    /// # Errors
    ///
    /// Returns a layout or stabilization error.
    pub fn advance(&mut self, duration: std::time::Duration) -> Result<(), TestError> {
        #[cfg(all(feature = "tasks", not(target_arch = "wasm32")))]
        {
            self.tasks
                .advance_time(duration)
                .map_err(|error| TestError::Task {
                    message: error.to_string(),
                })?;
            let update = self.model.tasks_ready(&self.window);
            self.record_app_update(update);
        }
        self.frame(duration)
    }

    /// Drains completed tasks and requested animation frames until no work remains.
    ///
    /// # Errors
    ///
    /// Returns [`TestError::DidNotSettle`] if work remains after the settle limit.
    pub fn run_until_idle(&mut self) -> Result<(), TestError> {
        for _ in 0..self.settle_limit() {
            #[cfg(all(feature = "tasks", not(target_arch = "wasm32")))]
            self.tasks
                .advance_time(std::time::Duration::ZERO)
                .map_err(|error| TestError::Task {
                    message: error.to_string(),
                })?;
            let update = self.model.tasks_ready(&self.window);
            self.record_app_update(update);
            if self.model.wants_animation_frame(&self.window)
                || self.ui().wants_animation_frame()
                || self.ui().wants_scroll_frame()
            {
                self.frame(std::time::Duration::from_millis(16))?;
                continue;
            }
            self.settle()?;
            if !self.pending {
                return Ok(());
            }
        }
        Err(TestError::DidNotSettle {
            limit: self.settle_limit(),
            tree: self.compact_dump(),
        })
    }

    /// Returns the number of asynchronous operations awaiting completion or cancellation.
    #[must_use]
    pub fn pending_tasks(&self) -> usize {
        #[cfg(feature = "tasks")]
        {
            self.tasks.pending()
        }
        #[cfg(not(feature = "tasks"))]
        {
            0
        }
    }

    /// Removes and returns application commands emitted since the last drain.
    pub fn take_commands(&mut self) -> Vec<argui_runtime::AppCommand> {
        std::mem::take(&mut self.commands)
    }

    /// Returns application commands emitted since the last drain.
    #[must_use]
    pub fn commands(&self) -> &[argui_runtime::AppCommand] {
        &self.commands
    }

    /// Returns scroll requests emitted by application callbacks.
    #[must_use]
    pub fn scroll_requests(&self) -> &[argui_ui::ScrollRequest] {
        &self.scroll_requests
    }

    fn center(&self, selector: &Selector) -> Result<Point, TestError> {
        let resolved = self.resolve_unique(selector)?;
        let bounds = resolved
            .bounds
            .filter(|bounds| bounds.size.width > 0.0 && bounds.size.height > 0.0)
            .ok_or_else(|| TestError::MissingBounds {
                selector: selector.clone(),
                tree: self.compact_dump(),
            })?;
        Ok(Point::new(
            bounds.origin.x + bounds.size.width * 0.5,
            bounds.origin.y + bounds.size.height * 0.5,
        ))
    }

    fn key_transition(
        &mut self,
        key: Key,
        state: KeyState,
        modifiers: Modifiers,
        text: Option<String>,
    ) -> Result<(), TestError> {
        let regions = self.output().hit_regions.clone();
        let update = self.ui_mut().key_input(
            &KeyInput {
                key,
                state,
                modifiers,
                repeat: false,
                text,
            },
            &regions,
        );
        self.dispatch_interaction(update)?;
        self.settle()
    }

    fn require_text_input(
        &self,
        selector: &Selector,
        resolved: &crate::inspect::ResolvedNode,
    ) -> Result<(), TestError> {
        if matches!(
            resolved.role,
            Role::TextInput | Role::TextArea | Role::SearchInput | Role::ComboBox
        ) && self.ui().text_input_value(resolved.node).is_some()
        {
            return Ok(());
        }
        Err(TestError::NotTextInput {
            selector: selector.clone(),
            role: resolved.role,
            tree: self.compact_dump(),
        })
    }

    const fn settle_limit(&self) -> usize {
        self.settle_limit
    }
}
