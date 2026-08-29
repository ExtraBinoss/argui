use argui_platform::{AccessibilityOverrides, AccessibilityPreferences, PlatformEvent};
use argui_ui::TreeUpdate;

use crate::RuntimeEvent;

use super::Application;

impl Application {
    pub(crate) const fn accessibility_overrides(
        mut self,
        overrides: AccessibilityOverrides,
    ) -> Self {
        self.accessibility_overrides = overrides;
        self
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn initialize_preferences(&mut self) {
        let Some(proxy) = self.event_proxy.clone() else {
            return;
        };
        let window = self.window_key.clone();
        let overrides = self.accessibility_overrides;
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || {
            let preferences = AccessibilityPreferences::detect(overrides);
            let _ = proxy.send_event(crate::event::UserEvent::Preferences {
                window,
                preferences,
            });
        });
        #[cfg(target_arch = "wasm32")]
        {
            let preferences = AccessibilityPreferences::detect(overrides);
            let _ = proxy.send_event(crate::event::UserEvent::Preferences {
                window,
                preferences,
            });
        }
    }

    pub(super) fn apply_preferences(&mut self, preferences: AccessibilityPreferences) {
        if self.preferences == preferences {
            return;
        }
        self.preferences = preferences;
        let animation_update = self.ui_tree.as_mut().map_or(TreeUpdate::None, |tree| {
            tree.set_reduced_motion(preferences.reduced_motion.enabled)
        });
        match animation_update {
            TreeUpdate::Layout => self.pending_ui_frame.request_layout(),
            TreeUpdate::Scroll => self.pending_ui_frame.request_scroll_update(),
            TreeUpdate::Paint => self.pending_ui_frame.request_paint(),
            TreeUpdate::None | TreeUpdate::Semantics => {}
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        (self.on_event)(RuntimeEvent::Platform(
            PlatformEvent::AccessibilityPreferences(preferences),
        ));
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use argui_platform::{
        AccessibilityOverrides, AccessibilityPreferences, PreferenceSource, ResolvedPreference,
        WindowConfig,
    };
    use argui_render::RendererConfig;
    use argui_text::TextEngine;
    use argui_ui::{Element, UiTree};

    use crate::RuntimeEvent;

    use super::Application;

    fn preferences() -> AccessibilityPreferences {
        AccessibilityPreferences {
            reduced_motion: ResolvedPreference {
                enabled: true,
                source: PreferenceSource::Override,
            },
            high_contrast: ResolvedPreference {
                enabled: true,
                source: PreferenceSource::System,
            },
        }
    }

    #[test]
    fn preference_changes_emit_once_with_or_without_a_ui_tree() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let captured = Rc::clone(&events);
        let mut application = Application::new(
            WindowConfig::default(),
            RendererConfig::default(),
            TextEngine::new(),
            None,
            Some(UiTree::new(Element::container([]))),
            None,
            move |event| captured.borrow_mut().push(event),
        );
        application.apply_preferences(preferences());
        application.apply_preferences(preferences());
        assert_eq!(events.borrow().len(), 1);
        assert!(matches!(
            events.borrow()[0],
            RuntimeEvent::Platform(argui_platform::PlatformEvent::AccessibilityPreferences(_))
        ));

        let mut headless = Application::new(
            WindowConfig::default(),
            RendererConfig::default(),
            TextEngine::new(),
            None,
            None,
            None,
            |_| {},
        )
        .accessibility_overrides(AccessibilityOverrides {
            reduced_motion: Some(false),
            high_contrast: Some(true),
        });
        assert_eq!(headless.accessibility_overrides.high_contrast, Some(true));
        headless.apply_preferences(preferences());
        assert_eq!(headless.preferences, preferences());
    }
}
