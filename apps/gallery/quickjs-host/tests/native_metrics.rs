use std::time::Duration;

use argui_gallery_quickjs::{parse_control, profile_json};
use argui_render::{DamageMode, DamageTracking, GpuFrameProfile, RenderProfile};
use argui_runtime::NativeHostControl;
use serde_json::Value;

#[test]
fn damage_control_requests_select_real_renderer_modes() {
    assert!(matches!(
        parse_control(r#"{"kind":"damageTracking","enabled":true}"#).unwrap(),
        NativeHostControl::SetDamageTracking(tracking) if tracking == DamageTracking::enabled()
    ));
    assert!(matches!(
        parse_control(r#"{"kind":"damageTracking","enabled":false}"#).unwrap(),
        NativeHostControl::SetDamageTracking(tracking) if tracking == DamageTracking::disabled()
    ));
    for invalid in [
        r#"{"kind":"damageTracking","enabled":"yes"}"#,
        r#"{"kind":"other","enabled":true}"#,
        "{",
    ] {
        assert!(parse_control(invalid).is_err(), "{invalid}");
    }
}

#[test]
fn development_svg_control_decodes_on_demand_and_rejects_invalid_source() {
    let request = r#"{"kind":"registerSvg","id":4534,"svg":"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\"><path d=\"M1 1h2\"/></svg>"}"#;
    let control = parse_control(request).expect("valid development SVG");
    assert!(matches!(control, NativeHostControl::RegisterVector(vector)
        if vector.id.0 == 4534 && vector.size.width == 24.0));
    for invalid in [
        r#"{"kind":"registerSvg","id":0,"svg":"<svg/>"}"#,
        r#"{"kind":"registerSvg","id":1,"svg":"broken"}"#,
        r#"{"kind":"registerSvg","id":1}"#,
    ] {
        assert!(parse_control(invalid).is_err(), "{invalid}");
    }
}

#[test]
fn profile_bridge_preserves_measured_values_and_unavailable_gpu() {
    let mut profile = RenderProfile {
        cpu_time: Duration::from_micros(2175),
        viewport_pixels: 800_000,
        ..RenderProfile::default()
    };
    profile.damage.mode = DamageMode::Partial;
    profile.damage.regions = 2;
    profile.damage.damaged_pixels = 64_000;
    let sample: Value = serde_json::from_str(&profile_json(&profile)).unwrap();
    assert_eq!(sample["cpuMs"], 2.175);
    assert!(sample["gpuMs"].is_null());
    assert_eq!(sample["damageMode"], "partial");
    assert_eq!(sample["damagedPixels"], 64_000);
    profile.gpu = Some(GpuFrameProfile {
        total: Duration::from_micros(950),
        ..GpuFrameProfile::default()
    });
    let sample: Value = serde_json::from_str(&profile_json(&profile)).unwrap();
    assert_eq!(sample["gpuMs"], 0.95);
}
