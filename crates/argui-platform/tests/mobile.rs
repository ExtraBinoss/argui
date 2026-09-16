use argui_core::Insets;
use argui_platform::mobile::{
    MobileActivity, MobileActivityCapability, MobileActivityError, PhysicalInsets,
};

/// Converts each physical edge with the supplied Android-style density scale.
#[test]
fn physical_insets_convert_each_edge_to_logical_pixels() {
    let actual = PhysicalInsets {
        top: 48,
        right: 24,
        bottom: 96,
        left: 12,
    }
    .to_logical(2.0);

    assert_eq!(actual, Insets::new(24.0, 12.0, 48.0, 6.0));
}

/// Uses a neutral scale if a host reports a non-finite or non-positive factor.
#[test]
fn physical_insets_use_unit_scale_when_factor_is_invalid() {
    let physical = PhysicalInsets {
        top: 48,
        right: 24,
        bottom: 96,
        left: 12,
    };

    assert_eq!(
        physical.to_logical(0.0),
        Insets::new(48.0, 24.0, 96.0, 12.0)
    );
    assert_eq!(
        physical.to_logical(f32::NAN),
        Insets::new(48.0, 24.0, 96.0, 12.0)
    );
}

/// Reports the desktop fallback without trying to create native mobile state.
#[test]
fn desktop_reports_foreground_only_mobile_activity_capability() {
    assert_eq!(
        MobileActivity::capability(),
        MobileActivityCapability::ForegroundOnly
    );
}

/// Rejects native activity creation on a non-mobile target with a useful diagnostic.
#[test]
fn desktop_rejects_native_mobile_activity_creation() {
    let error = MobileActivity::begin("Build", "Starting").unwrap_err();

    assert_eq!(error, MobileActivityError::Unsupported);
    assert_eq!(
        error.to_string(),
        "mobile background activity is unsupported"
    );
}

/// Formats the platform-specific permission and native error variants for application UI.
#[test]
fn mobile_activity_errors_have_readable_messages() {
    assert_eq!(
        MobileActivityError::NotificationPermissionRequired.to_string(),
        "notification permission is required to start this activity"
    );
    assert_eq!(
        MobileActivityError::Native("native failure".into()).to_string(),
        "native failure"
    );
}
