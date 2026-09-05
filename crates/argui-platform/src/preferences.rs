use argui_core::ColorScheme;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PreferenceSource {
    Override,
    System,
    #[default]
    Default,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolvedPreference<T> {
    pub value: T,
    pub source: PreferenceSource,
}

impl<T: Default> Default for ResolvedPreference<T> {
    fn default() -> Self {
        Self {
            value: T::default(),
            source: PreferenceSource::Default,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PreferenceOverrides {
    pub color_scheme: Option<ColorScheme>,
    pub reduced_motion: Option<bool>,
    pub high_contrast: Option<bool>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SystemPreferences {
    pub color_scheme: ResolvedPreference<ColorScheme>,
    pub reduced_motion: ResolvedPreference<bool>,
    pub high_contrast: ResolvedPreference<bool>,
}

impl SystemPreferences {
    #[must_use]
    pub fn detect(overrides: PreferenceOverrides) -> Self {
        let system = system_preferences();
        Self {
            color_scheme: resolve(overrides.color_scheme, system.0),
            reduced_motion: resolve(overrides.reduced_motion, system.1),
            high_contrast: resolve(overrides.high_contrast, system.2),
        }
    }

    /// Publishes the current preferences and subsequent native system changes.
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn watch(overrides: PreferenceOverrides, mut publish: impl FnMut(Self) + Send + 'static) {
        let current = Self::detect(overrides);
        publish(current);
        #[cfg(target_os = "linux")]
        if overrides.color_scheme.is_none() {
            use ashpd::desktop::settings::{ColorScheme as PortalScheme, Settings};
            use futures_util::StreamExt;

            let mut current = current;
            pollster::block_on(async move {
                let Ok(settings) = Settings::new().await else {
                    return;
                };
                let Ok(mut changes) = settings.receive_color_scheme_changed().await else {
                    return;
                };
                while let Some(scheme) = changes.next().await {
                    let value = match scheme {
                        PortalScheme::PreferDark => ColorScheme::Dark,
                        PortalScheme::PreferLight | PortalScheme::NoPreference => {
                            ColorScheme::Light
                        }
                    };
                    current.color_scheme = ResolvedPreference {
                        value,
                        source: PreferenceSource::System,
                    };
                    publish(current);
                }
            });
        }
    }
}

fn resolve<T: Copy + Default>(
    override_value: Option<T>,
    system: Option<T>,
) -> ResolvedPreference<T> {
    if let Some(value) = override_value {
        ResolvedPreference {
            value,
            source: PreferenceSource::Override,
        }
    } else if let Some(value) = system {
        ResolvedPreference {
            value,
            source: PreferenceSource::System,
        }
    } else {
        ResolvedPreference::default()
    }
}

#[cfg(target_os = "linux")]
#[cfg_attr(coverage_nightly, coverage(off))]
fn system_preferences() -> (Option<ColorScheme>, Option<bool>, Option<bool>) {
    use ashpd::desktop::settings::{
        ColorScheme as PortalScheme, Contrast, ReducedMotion, Settings,
    };

    pollster::block_on(async {
        let Ok(settings) = Settings::new().await else {
            return (None, None, None);
        };
        let scheme = settings
            .color_scheme()
            .await
            .ok()
            .and_then(|value| match value {
                PortalScheme::PreferDark => Some(ColorScheme::Dark),
                PortalScheme::PreferLight => Some(ColorScheme::Light),
                PortalScheme::NoPreference => None,
            });
        let motion = settings
            .reduced_motion()
            .await
            .ok()
            .map(|value| value == ReducedMotion::ReducedMotion);
        let contrast = settings
            .contrast()
            .await
            .ok()
            .map(|value| value == Contrast::High);
        (scheme, motion, contrast)
    })
}

#[cfg(target_os = "windows")]
fn system_preferences() -> (Option<ColorScheme>, Option<bool>, Option<bool>) {
    use windows::UI::ViewManagement::{AccessibilitySettings, UISettings};

    let motion = UISettings::new()
        .and_then(|settings| settings.AnimationsEnabled())
        .ok()
        .map(|enabled| !enabled);
    let contrast = AccessibilitySettings::new()
        .and_then(|settings| settings.HighContrast())
        .ok();
    (None, motion, contrast)
}

#[cfg(target_os = "macos")]
fn system_preferences() -> (Option<ColorScheme>, Option<bool>, Option<bool>) {
    let workspace = objc2_app_kit::NSWorkspace::sharedWorkspace();
    (
        None,
        Some(workspace.accessibilityDisplayShouldReduceMotion()),
        Some(workspace.accessibilityDisplayShouldIncreaseContrast()),
    )
}

#[cfg(target_arch = "wasm32")]
fn system_preferences() -> (Option<ColorScheme>, Option<bool>, Option<bool>) {
    let Some(window) = web_sys::window() else {
        return (None, None, None);
    };
    let query = |value: &str| {
        window
            .match_media(value)
            .ok()
            .flatten()
            .map(|media| media.matches())
    };
    (
        query("(prefers-color-scheme: dark)").map(|dark| {
            if dark {
                ColorScheme::Dark
            } else {
                ColorScheme::Light
            }
        }),
        query("(prefers-reduced-motion: reduce)"),
        query("(prefers-contrast: more)"),
    )
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "windows",
    target_os = "macos",
    target_arch = "wasm32"
)))]
fn system_preferences() -> (Option<ColorScheme>, Option<bool>, Option<bool>) {
    (None, None, None)
}
