use argui_animation::{Duration, Motion};
use argui_core::Transform2D;
use argui_ui::{Element, PointerEvents, property};

/// Retained overlay visibility. Advance once per presentation frame, not per event.
#[derive(Clone, Debug)]
pub struct Presence {
    open: bool,
    progress: f32,
    opacity: Motion<f32>,
    transform: Motion<Transform2D>,
}

impl Default for Presence {
    fn default() -> Self {
        Self {
            open: false,
            progress: 0.0,
            opacity: Motion::new(0.0),
            transform: Motion::new(Transform2D::IDENTITY.translate(0.0, 4.0)),
        }
    }
}

impl Presence {
    pub fn set_open(&mut self, open: bool, reduced_motion: bool) {
        self.open = open;
        if reduced_motion {
            self.progress = if open { 1.0 } else { 0.0 };
            self.sample();
        }
    }

    #[must_use]
    pub fn visible(&self) -> bool {
        self.open || self.progress > 0.0
    }

    #[must_use]
    pub fn animating(&self) -> bool {
        self.progress != if self.open { 1.0 } else { 0.0 }
    }

    /// Returns true when the mounted state changes, requiring reconciliation.
    pub fn advance(&mut self, elapsed: Duration) -> bool {
        let visible = self.visible();
        let duration = if self.open { 0.140 } else { 0.100 };
        let delta = (elapsed.as_secs_f64() / duration) as f32;
        self.progress = (self.progress + if self.open { delta } else { -delta }).clamp(0.0, 1.0);
        self.sample();
        visible != self.visible()
    }

    fn sample(&self) {
        let eased = self.progress * self.progress * (3.0 - 2.0 * self.progress);
        self.opacity.set(eased);
        self.transform
            .set(Transform2D::IDENTITY.translate(0.0, 4.0 * (1.0 - eased)));
    }

    #[must_use]
    pub fn decorate(&self, mut element: Element) -> Element {
        element = element
            .opacity(1.0)
            .bind(property::LayerOpacity, self.opacity.clone())
            .bind(property::Transform, self.transform.clone());
        if !self.open {
            element = element.semantic_hidden(true);
            element.hit_test.pointer_events = PointerEvents::None;
            if let Some(portal) = &mut element.portal {
                portal.dismiss = argui_ui::DismissPolicy::Manual;
            }
        }
        element
    }
}
