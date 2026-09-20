use std::time::Duration;

use argui::{
    render::{DamageMode, DamageProfile, GpuFrameProfile, RenderProfile},
    runtime::{AnimationProfile, WindowEnvironment},
    widgets::shadcn,
};

#[path = "../../../src/pages/damage_control/metrics.rs"]
mod telemetry;

#[test]
fn rolling_damage_metrics_bound_samples_and_average_every_cpu_stage() {
    let handle = telemetry::DamageTelemetry::handle();
    assert_eq!(handle.borrow().metrics().samples, 0);
    let mut telemetry = telemetry::DamageTelemetry::default();
    assert_eq!(telemetry.metrics().samples, 0);

    for index in 0..=120 {
        let viewport_pixels = if index == 120 { 0 } else { 100 };
        telemetry.record(&RenderProfile {
            cpu_time: Duration::from_millis(2),
            viewport_pixels,
            damage: DamageProfile {
                mode: DamageMode::Partial,
                regions: 2,
                damaged_pixels: 25,
                retained_bytes: 4096,
            },
            gpu: (index % 2 == 0).then_some(GpuFrameProfile {
                total: Duration::from_millis(1),
                ..GpuFrameProfile::default()
            }),
            ..RenderProfile::default()
        });
        telemetry.record_animation(&AnimationProfile {
            model_time: Duration::from_millis(1),
            tree_time: Duration::from_millis(2),
            paint_time: Duration::from_millis(3),
            ..AnimationProfile::default()
        });
    }

    let measured = telemetry.metrics();
    assert_eq!(measured.samples, 120);
    assert_eq!(measured.animation_samples, 120);
    assert_eq!(measured.mode, DamageMode::Partial);
    assert_eq!(measured.regions, 2);
    assert_eq!(measured.damaged_pixels, 25);
    assert_eq!(measured.viewport_pixels, 0);
    assert_eq!(measured.retained_bytes, 4096);
    assert!(measured.average_ratio > 0.0);
    assert_eq!(measured.average_cpu_ms, 2.0);
    assert_eq!(measured.average_gpu_ms, Some(1.0));
    assert_eq!(measured.average_model_ms, 1.0);
    assert_eq!(measured.average_tree_ms, 2.0);
    assert_eq!(measured.average_paint_ms, 3.0);
    assert_eq!(
        telemetry::when_ready(true, "ready".into(), "pending"),
        "ready"
    );
    assert_eq!(
        telemetry::when_ready(false, "ready".into(), "pending"),
        "pending"
    );
    assert_eq!(telemetry::format_pixels(25, 100), "0.00M / 0.00M px");
    assert_eq!(telemetry::damage_mode(DamageMode::Partial), "Partial");
    let environment = WindowEnvironment::default();
    let themes = shadcn(&environment);
    let _card = telemetry::metric_card(
        "CPU pipeline",
        "6.000 ms".into(),
        "runtime stages",
        themes.resolve(environment.color_scheme),
    );

    telemetry.clear();
    assert_eq!(telemetry.metrics().samples, 0);
}
