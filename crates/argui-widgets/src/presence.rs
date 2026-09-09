use argui_animation::{Duration, Motion};
use argui_core::Transform2D;
use argui_ui::{Element, PointerEvents, property};

/// Retained overlay visibility. Advance once per presentation frame, not per event.
#[derive(Clone, Debug)]
pub struct Presence {
    open: bool,
    progress: f32,
    fade_progress: f32,
    fade_in: bool,
    opacity: Motion<f32>,
    transform: Motion<Transform2D>,
}

impl Default for Presence {
    fn default() -> Self {
        Self {
            open: false,
            progress: 0.0,
            fade_progress: 0.0,
            fade_in: true,
            opacity: Motion::new(0.0),
            transform: Motion::new(Transform2D::IDENTITY.translate(0.0, 4.0)),
        }
    }
}

impl Presence {
    /// Keep content immediately readable while retaining entry movement and exit fading.
    #[must_use]
    pub fn fade_in(mut self, enabled: bool) -> Self {
        self.fade_in = enabled;
        if self.open && !enabled {
            self.fade_progress = 1.0;
            self.sample();
        }
        self
    }

    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.open
    }

    pub fn set_open(&mut self, open: bool, reduced_motion: bool) {
        self.open = open;
        if open && !self.fade_in {
            self.fade_progress = 1.0;
        }
        if reduced_motion {
            self.progress = if open { 1.0 } else { 0.0 };
            self.fade_progress = self.progress;
        }
        self.sample();
    }

    #[must_use]
    pub fn visible(&self) -> bool {
        self.open || self.progress > 0.0 || self.fade_progress > 0.0
    }

    #[must_use]
    pub fn animating(&self) -> bool {
        let target = if self.open { 1.0 } else { 0.0 };
        self.progress != target || self.fade_progress != target
    }

    /// Returns true when the mounted state changes, requiring reconciliation.
    pub fn advance(&mut self, elapsed: Duration) -> bool {
        if !self.animating() {
            return false;
        }
        let visible = self.visible();
        let duration = if self.open { 0.140 } else { 0.100 };
        let delta = (elapsed.as_secs_f64() / duration) as f32;
        self.progress = (self.progress + if self.open { delta } else { -delta }).clamp(0.0, 1.0);
        self.fade_progress =
            (self.fade_progress + if self.open { delta } else { -delta }).clamp(0.0, 1.0);
        self.sample();
        visible != self.visible()
    }

    fn sample(&self) {
        let eased = self.progress * self.progress * (3.0 - 2.0 * self.progress);
        let fade = self.fade_progress;
        self.opacity.set(fade * fade * (3.0 - 2.0 * fade));
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
            disable_focus(&mut element);
            element = element.semantic_hidden(true);
            element.hit_test.pointer_events = PointerEvents::None;
            if let Some(portal) = &mut element.portal {
                portal.dismiss = argui_ui::DismissPolicy::Manual;
            }
        }
        element
    }
}

fn disable_focus(element: &mut Element) {
    element.focus_scope = None;
    if let Some(interaction) = &mut element.interaction {
        interaction.enabled = false;
        interaction.focus_policy = argui_ui::FocusPolicy::None;
    }
    for child in &mut element.children {
        disable_focus(child);
    }
}
