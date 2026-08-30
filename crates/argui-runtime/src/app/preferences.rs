use argui_core::ColorScheme;
use argui_platform::{PlatformEvent, PreferenceOverrides, SystemPreferences};
use argui_ui::TreeUpdate;

use crate::RuntimeEvent;

use super::Application;

impl Application {
    pub(crate) fn preference_overrides(mut self, overrides: PreferenceOverrides) -> Self {
        self.preference_overrides = overrides;
        if let Some(color_scheme) = overrides.color_scheme {
            self.environment.color_scheme = color_scheme;
        }
        if let Some(reduced_motion) = overrides.reduced_motion {
            self.environment.reduced_motion = reduced_motion;
        }
        if let Some(high_contrast) = overrides.high_contrast {
            self.environment.high_contrast = high_contrast;
        }
        self
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn initialize_preferences(&mut self) {
        let Some(proxy) = self.event_proxy.clone() else {
            return;
        };
        let window = self.window_key.clone();
        let overrides = self.preference_overrides;
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::spawn(move || {
            SystemPreferences::watch(overrides, move |preferences| {
                let _ = proxy.send_event(crate::event::UserEvent::Preferences {
                    window: window.clone(),
                    preferences,
                });
            });
        });
        #[cfg(target_arch = "wasm32")]
        {
            let preferences = SystemPreferences::detect(overrides);
            let _ = proxy.send_event(crate::event::UserEvent::Preferences {
                window,
                preferences,
            });
        }
    }

    pub(super) fn apply_color_scheme(&mut self, color_scheme: ColorScheme) {
        if self.theme_request.color_scheme.is_some() {
            return;
        }
        if self.environment.color_scheme == color_scheme {
            return;
        }
        self.environment.color_scheme = color_scheme;
        self.pending_ui_frame.request_rebuild();
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    pub(super) fn apply_preferences(&mut self, preferences: SystemPreferences) {
        if self.preferences == preferences {
            return;
        }
        self.preferences = preferences;
        self.environment = crate::WindowEnvironment {
            color_scheme: self
                .theme_request
                .color_scheme
                .unwrap_or(preferences.color_scheme.value),
            primary: self.environment.primary,
            reduced_motion: preferences.reduced_motion.value,
            high_contrast: preferences.high_contrast.value,
        };
        let animation_update = self.ui_tree.as_mut().map_or(TreeUpdate::None, |tree| {
            tree.set_reduced_motion(preferences.reduced_motion.value)
        });
        match animation_update {
            TreeUpdate::Layout => self.pending_ui_frame.request_layout(),
            TreeUpdate::Scroll => self.pending_ui_frame.request_scroll_update(),
            TreeUpdate::Paint => self.pending_ui_frame.request_paint(),
            TreeUpdate::None | TreeUpdate::Semantics => {}
        }
        self.pending_ui_frame.request_rebuild();
        if let Some(window) = &self.window {
            window.request_redraw();
        }
        (self.on_event)(RuntimeEvent::Platform(PlatformEvent::PreferencesChanged(
            preferences,
        )));
    }

    pub(super) fn apply_theme_request(&mut self, request: crate::ThemeRequest) {
        self.theme_request = request;
        let color_scheme = request
            .color_scheme
            .unwrap_or(self.preferences.color_scheme.value);
        let primary = request.primary.unwrap_or(self.environment.primary);
        if self.environment.color_scheme == color_scheme && self.environment.primary == primary {
            return;
        }
        self.environment.color_scheme = color_scheme;
        self.environment.primary = primary;
        self.pending_ui_frame.request_rebuild();
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use argui_core::{Color, ColorScheme};
    use argui_platform::{
        PreferenceOverrides, PreferenceSource, ResolvedPreference, SystemPreferences, WindowConfig,
    };
    use argui_render::RendererConfig;
    use argui_text::TextEngine;
    use argui_ui::{Element, UiTree};

    use crate::RuntimeEvent;

    use super::Application;

    fn preferences() -> SystemPreferences {
        SystemPreferences {
            color_scheme: ResolvedPreference {
                value: ColorScheme::Dark,
                source: PreferenceSource::System,
            },
            reduced_motion: ResolvedPreference {
                value: true,
                source: PreferenceSource::Override,
            },
            high_contrast: ResolvedPreference {
                value: true,
                source: PreferenceSource::System,
            },
        }
    }

    #[test]
    fn preference_changes_emit_once_and_update_environment() {
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
        assert_eq!(application.environment.color_scheme, ColorScheme::Dark);
        assert!(matches!(
            events.borrow()[0],
            RuntimeEvent::Platform(argui_platform::PlatformEvent::PreferencesChanged(_))
        ));
    }

    #[test]
    fn explicit_scheme_change_rebuilds_once() {
        let mut application = Application::new(
            WindowConfig::default(),
            RendererConfig::default(),
            TextEngine::new(),
            None,
            None,
            None,
            |_| {},
        )
        .preference_overrides(PreferenceOverrides {
            color_scheme: Some(ColorScheme::Dark),
            ..PreferenceOverrides::default()
        });
        assert_eq!(
            application.preference_overrides.color_scheme,
            Some(ColorScheme::Dark)
        );
        application.apply_color_scheme(ColorScheme::Dark);
        assert_eq!(application.environment.color_scheme, ColorScheme::Dark);
        application.apply_color_scheme(ColorScheme::Light);
        assert_eq!(application.environment.color_scheme, ColorScheme::Light);
    }

    #[test]
    fn every_override_and_preferences_without_a_tree_update_the_environment() {
        let mut application = Application::new(
            WindowConfig::default(),
            RendererConfig::default(),
            TextEngine::new(),
            None,
            None,
            None,
            |_| {},
        )
        .preference_overrides(PreferenceOverrides {
            color_scheme: Some(ColorScheme::Dark),
            reduced_motion: Some(true),
            high_contrast: Some(true),
        });
        assert!(application.environment.reduced_motion);
        assert!(application.environment.high_contrast);

        application.apply_preferences(preferences());
        assert_eq!(application.environment.color_scheme, ColorScheme::Dark);
        assert!(application.environment.reduced_motion);
        assert!(application.environment.high_contrast);
    }

    #[test]
    fn application_theme_overrides_system_scheme_and_primary_together() {
        let mut application = Application::new(
            WindowConfig::default(),
            RendererConfig::default(),
            TextEngine::new(),
            None,
            None,
            None,
            |_| {},
        );
        let primary = Color::rgb(0.88, 0.11, 0.28);
        application.apply_theme_request(crate::ThemeRequest {
            color_scheme: Some(ColorScheme::Dark),
            primary: Some(primary),
        });
        assert_eq!(application.environment.color_scheme, ColorScheme::Dark);
        assert_eq!(application.environment.primary, primary);

        application.apply_color_scheme(ColorScheme::Light);
        assert_eq!(application.environment.color_scheme, ColorScheme::Dark);

        application.apply_theme_request(crate::ThemeRequest {
            color_scheme: None,
            primary: Some(primary),
        });
        assert_eq!(
            application.environment.color_scheme,
            application.preferences.color_scheme.value
        );
    }
}
