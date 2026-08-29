#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PreferenceSource {
    Override,
    System,
    #[default]
    Default,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolvedPreference {
    pub enabled: bool,
    pub source: PreferenceSource,
}

impl Default for ResolvedPreference {
    fn default() -> Self {
        Self {
            enabled: false,
            source: PreferenceSource::Default,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AccessibilityOverrides {
    pub reduced_motion: Option<bool>,
    pub high_contrast: Option<bool>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AccessibilityPreferences {
    pub reduced_motion: ResolvedPreference,
    pub high_contrast: ResolvedPreference,
}

impl AccessibilityPreferences {
    #[must_use]
    pub fn detect(overrides: AccessibilityOverrides) -> Self {
        let system = if overrides.reduced_motion.is_some() && overrides.high_contrast.is_some() {
            (None, None)
        } else {
            system_preferences()
        };
        Self {
            reduced_motion: resolve(overrides.reduced_motion, system.0),
            high_contrast: resolve(overrides.high_contrast, system.1),
        }
    }
}

fn resolve(override_value: Option<bool>, system: Option<bool>) -> ResolvedPreference {
    if let Some(enabled) = override_value {
        ResolvedPreference {
            enabled,
            source: PreferenceSource::Override,
        }
    } else if let Some(enabled) = system {
        ResolvedPreference {
            enabled,
            source: PreferenceSource::System,
        }
    } else {
        ResolvedPreference::default()
    }
}

#[cfg(target_os = "linux")]
fn system_preferences() -> (Option<bool>, Option<bool>) {
    use ashpd::desktop::settings::{Contrast, ReducedMotion, Settings};

    pollster::block_on(async {
        let Ok(settings) = Settings::new().await else {
            return (None, None);
        };
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
        (motion, contrast)
    })
}

#[cfg(target_os = "windows")]
fn system_preferences() -> (Option<bool>, Option<bool>) {
    use windows::UI::ViewManagement::{AccessibilitySettings, UISettings};

    let motion = UISettings::new()
        .and_then(|settings| settings.AnimationsEnabled())
        .ok()
        .map(|enabled| !enabled);
    let contrast = AccessibilitySettings::new()
        .and_then(|settings| settings.HighContrast())
        .ok();
    (motion, contrast)
}

#[cfg(target_os = "macos")]
fn system_preferences() -> (Option<bool>, Option<bool>) {
    let workspace = objc2_app_kit::NSWorkspace::sharedWorkspace();
    (
        Some(workspace.accessibilityDisplayShouldReduceMotion()),
        Some(workspace.accessibilityDisplayShouldIncreaseContrast()),
    )
}

#[cfg(target_arch = "wasm32")]
fn system_preferences() -> (Option<bool>, Option<bool>) {
    let Some(window) = web_sys::window() else {
        return (None, None);
    };
    let query = |value: &str| {
        window
            .match_media(value)
            .ok()
            .flatten()
            .map(|media| media.matches())
    };
    (
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
fn system_preferences() -> (Option<bool>, Option<bool>) {
    (None, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overrides_have_priority_and_report_their_source() {
        let preferences = AccessibilityPreferences::detect(AccessibilityOverrides {
            reduced_motion: Some(true),
            high_contrast: Some(false),
        });
        assert_eq!(
            preferences.reduced_motion,
            ResolvedPreference {
                enabled: true,
                source: PreferenceSource::Override,
            }
        );
        assert_eq!(preferences.high_contrast.source, PreferenceSource::Override);
    }

    #[test]
    fn system_and_default_resolution_are_distinct() {
        assert_eq!(
            resolve(None, Some(true)),
            ResolvedPreference {
                enabled: true,
                source: PreferenceSource::System,
            }
        );
        assert_eq!(resolve(None, None), ResolvedPreference::default());
        assert_eq!(
            resolve(Some(false), Some(true)).source,
            PreferenceSource::Override
        );
    }
}
