use argui_core::ColorScheme;
use argui_platform::{
    PlatformEvent, PreferenceOverrides, PreferenceSource, ResolvedPreference, SystemPreferences,
};
use argui_ui::TreeUpdate;

use crate::RuntimeEvent;

use super::Application;

impl Application {
    /// Assigns `theme` while building a named window application.
    /// Returns this application with its initial resolved theme snapshot.
    pub(crate) fn with_theme(mut self, theme: Option<argui_theme::ThemeRuntime>) -> Self {
        self.install_theme(theme);
        self
    }

    /// Installs the typed theme used by this window and captures its initial values.
    /// `theme` is shared with the application model; clones observe the same revisions.
    pub(crate) fn install_theme(&mut self, theme: Option<argui_theme::ThemeRuntime>) {
        self.theme_subscription = None;
        self.environment.theme = theme.as_ref().map(argui_theme::ThemeRuntime::snapshot);
        self.theme_runtime = theme;
        if let Some(proxy) = self.event_proxy.clone() {
            self.watch_theme(&proxy);
        }
    }

    /// Wakes this window when `theme_runtime` publishes a committed change.
    /// `proxy` sends notifications to the owning event loop.
    pub(crate) fn watch_theme(&mut self, proxy: &crate::host::EventProxy) {
        self.theme_subscription = self.theme_runtime.as_ref().map(|theme| {
            let wake = proxy.clone();
            let window = self.window_key.clone();
            theme.watch(move |change| {
                let _ = wake.send_event(crate::event::UserEvent::ThemeChanged {
                    window: window.clone(),
                    change,
                });
            })
        });
    }

    /// Applies a committed `change` from this window's theme to the render environment.
    /// A revision already captured by a platform preference event needs no second frame.
    pub(crate) fn apply_theme_change(&mut self, change: &argui_theme::ThemeChange) {
        let Some(theme) = &self.theme_runtime else {
            return;
        };
        let snapshot = theme.snapshot();
        if self.environment.theme.as_ref() == Some(&snapshot) {
            return;
        }
        self.environment.theme = Some(snapshot);
        if change.is_empty() {
            return;
        }
        self.pending_ui_frame.request_rebuild();
        if matches!(change.impact(), Some(argui_theme::ThemeImpact::Layout)) {
            self.pending_ui_frame.request_layout();
        }
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

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

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn initialize_preference_snapshot(
        &mut self,
        window_theme: Option<winit::window::Theme>,
    ) {
        self.system_color_scheme = window_theme.map(|theme| match theme {
            winit::window::Theme::Light => ColorScheme::Light,
            winit::window::Theme::Dark => ColorScheme::Dark,
        });
        let preferences =
            self.resolve_preferences(SystemPreferences::detect(self.preference_overrides));
        self.preferences = preferences;
        self.environment.color_scheme = self
            .theme_request
            .color_scheme
            .unwrap_or(preferences.color_scheme.value);
        if let Some(theme) = &self.theme_runtime {
            theme.set_system_scheme(preferences.color_scheme.value);
            self.environment.theme = Some(theme.snapshot());
        }
        self.environment.reduced_motion = preferences.reduced_motion.value;
        self.environment.high_contrast = preferences.high_contrast.value;
    }

    fn resolve_preferences(&self, mut preferences: SystemPreferences) -> SystemPreferences {
        if self.preference_overrides.color_scheme.is_none()
            && preferences.color_scheme.source == PreferenceSource::Default
            && let Some(color_scheme) = self.system_color_scheme
        {
            preferences.color_scheme = ResolvedPreference {
                value: color_scheme,
                source: PreferenceSource::System,
            };
        }
        preferences
    }

    pub(super) fn apply_color_scheme(&mut self, color_scheme: ColorScheme) {
        let previous_scheme = self.environment.color_scheme;
        self.system_color_scheme = Some(color_scheme);
        if self.preference_overrides.color_scheme.is_none() {
            self.preferences.color_scheme = ResolvedPreference {
                value: color_scheme,
                source: PreferenceSource::System,
            };
        }
        let mut tokens_changed = false;
        if let Some(theme) = &self.theme_runtime {
            tokens_changed = !theme.set_system_scheme(color_scheme).is_empty();
            self.environment.theme = Some(theme.snapshot());
        }
        if self.theme_request.color_scheme.is_none() {
            self.environment.color_scheme = color_scheme;
        }
        if self.environment.color_scheme == previous_scheme && !tokens_changed {
            return;
        }
        self.pending_ui_frame.request_rebuild();
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    pub(super) fn apply_preferences(&mut self, preferences: SystemPreferences) {
        let preferences = self.resolve_preferences(preferences);
        if self.preferences == preferences {
            return;
        }
        self.preferences = preferences;
        let previous_scheme = self.environment.color_scheme;
        let previous_motion = self.environment.reduced_motion;
        let previous_contrast = self.environment.high_contrast;
        self.environment = crate::WindowEnvironment {
            color_scheme: self
                .theme_request
                .color_scheme
                .unwrap_or(preferences.color_scheme.value),
            primary: self.environment.primary,
            reduced_motion: preferences.reduced_motion.value,
            high_contrast: preferences.high_contrast.value,
            desktop_backdrop_available: self.environment.desktop_backdrop_available,
            safe_area_insets: self.environment.safe_area_insets,
            ui_zoom: self.environment.ui_zoom,
            theme_overrides: self.environment.theme_overrides.clone(),
            theme: self.environment.theme.clone(),
        };
        let mut tokens_changed = false;
        if let Some(theme) = &self.theme_runtime {
            tokens_changed = !theme
                .set_system_scheme(preferences.color_scheme.value)
                .is_empty();
            self.environment.theme = Some(theme.snapshot());
        }
        let animation_update = self.ui_tree.as_mut().map_or(TreeUpdate::None, |tree| {
            tree.set_reduced_motion(preferences.reduced_motion.value)
        });
        match animation_update {
            TreeUpdate::Layout => self.pending_ui_frame.request_layout(),
            TreeUpdate::Scroll => self.pending_ui_frame.request_scroll_update(),
            TreeUpdate::Paint => self.pending_ui_frame.request_paint(),
            TreeUpdate::Composite => self.pending_ui_frame.request_composite(),
            TreeUpdate::None | TreeUpdate::Semantics => {}
        }
        if previous_scheme != self.environment.color_scheme
            || previous_motion != self.environment.reduced_motion
            || previous_contrast != self.environment.high_contrast
            || tokens_changed
        {
            self.pending_ui_frame.request_rebuild();
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
        (self.on_event)(RuntimeEvent::Platform(PlatformEvent::PreferencesChanged(
            preferences,
        )));
    }

    pub(super) fn apply_theme_request(&mut self, request: crate::ThemeRequest) {
        let previous_scheme = self.environment.color_scheme;
        let previous_primary = self.environment.primary;
        self.theme_request = request;
        let color_scheme = request
            .color_scheme
            .unwrap_or(self.preferences.color_scheme.value);
        let primary = request.primary.unwrap_or(self.environment.primary);
        self.environment.color_scheme = color_scheme;
        self.environment.primary = primary;
        let mut tokens_changed = false;
        if let Some(theme) = &self.theme_runtime {
            tokens_changed = !theme
                .set_mode(match request.color_scheme {
                    Some(ColorScheme::Light) => argui_theme::ThemeMode::Light,
                    Some(ColorScheme::Dark) => argui_theme::ThemeMode::Dark,
                    None => argui_theme::ThemeMode::System,
                })
                .is_empty();
            self.environment.theme = Some(theme.snapshot());
        }
        if self.environment.color_scheme == previous_scheme
            && self.environment.primary == previous_primary
            && !tokens_changed
        {
            return;
        }
        self.pending_ui_frame.request_rebuild();
    }
}
