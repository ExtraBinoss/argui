use argui_core::ColorScheme;
use argui_platform::{PreferenceOverrides, PreferenceSource, SystemPreferences};

#[test]
fn overrides_have_priority_and_report_their_source() {
    let preferences = SystemPreferences::detect(PreferenceOverrides {
        color_scheme: Some(ColorScheme::Dark),
        reduced_motion: Some(true),
        high_contrast: Some(false),
    });
    assert_eq!(preferences.color_scheme.value, ColorScheme::Dark);
    assert_eq!(preferences.color_scheme.source, PreferenceSource::Override);
    assert!(preferences.reduced_motion.value);
    assert_eq!(preferences.high_contrast.source, PreferenceSource::Override);
}

#[test]
fn preference_defaults_keep_the_default_provenance() {
    let preference = argui_platform::ResolvedPreference::<bool>::default();
    assert!(!preference.value);
    assert_eq!(preference.source, PreferenceSource::Default);

    let preferences = SystemPreferences::default();
    assert_eq!(preferences.color_scheme.source, PreferenceSource::Default);
    assert_eq!(preferences.reduced_motion.source, PreferenceSource::Default);
    assert_eq!(preferences.high_contrast.source, PreferenceSource::Default);
}
