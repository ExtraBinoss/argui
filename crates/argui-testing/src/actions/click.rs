use argui_core::{PointerButton, PointerEvent, PointerId, PointerPhase};
use argui_runtime::Render;
use argui_ui::UiEventKind;

use crate::{Selector, TestApp, TestError};

impl<A: Render> TestApp<A> {
    /// Clicks the unique element with application `key` using real hit testing.
    ///
    /// # Errors
    ///
    /// Returns a selector, bounds, layout, or stabilization error.
    pub fn click(&mut self, key: &str) -> Result<(), TestError> {
        self.click_selector(&Selector::key(key))
    }

    /// Clicks a resolved selector while allowing a render between press and release.
    ///
    /// `selector` identifies the element to hit.
    ///
    /// # Errors
    ///
    /// Returns a selector, bounds, layout, or stabilization error if the click
    /// cannot finish.
    pub(crate) fn click_selector(&mut self, selector: &Selector) -> Result<(), TestError> {
        let point = self.center(selector)?;
        let regions = self.output().hit_regions.clone();
        let moved = self.ui_mut().pointer_moved(point, &regions);
        self.dispatch_interaction(moved)?;
        self.settle()?;

        let regions = self.output().hit_regions.clone();
        let pressed = PointerEvent {
            button: Some(PointerButton::Primary),
            buttons: 1,
            timestamp: std::time::Duration::from_nanos(self.now.as_nanos()),
            ..PointerEvent::mouse(PointerPhase::Pressed, point)
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
        let default_prevented = press_delivery
            .as_ref()
            .is_some_and(|event| event.default_prevented());
        self.dispatch_interaction(update)?;
        let capture = self.ui_mut().pointer_press_default(
            PointerId::MOUSE,
            press_delivery.as_ref(),
            &regions,
        );
        self.dispatch_interaction(capture)?;
        if !default_prevented {
            let focus = self
                .ui_mut()
                .focus_pointer_default(PointerId::MOUSE, &regions);
            self.dispatch_interaction(focus)?;
        }
        self.settle()?;

        let regions = self.output().hit_regions.clone();
        let released = PointerEvent {
            button: Some(PointerButton::Primary),
            timestamp: std::time::Duration::from_nanos(self.now.as_nanos()),
            ..PointerEvent::mouse(PointerPhase::Released, point)
        };
        let update = self.ui_mut().pointer_event(released, &regions);
        self.dispatch_interaction(update)?;
        self.settle()
    }
}
