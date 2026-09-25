use argui_render::{
    BlurAlgorithm, DamageTracking, EffectQuality, EffectQualitySettings, RendererConfig,
    SurfaceAlphaMode,
};

#[test]
fn renderer_defaults_to_vsync_and_a_discrete_gpu() {
    let config = RendererConfig::default();

    assert_eq!(
        config.power_preference,
        wgpu::PowerPreference::HighPerformance
    );
    assert!(config.renderer_fallback);
    assert!(!config.clone().renderer_fallback(false).renderer_fallback);
    assert_eq!(config.present_mode, wgpu::PresentMode::AutoVsync);
    assert_eq!(config.maximum_frame_latency, 2);
    assert!(!config.wait_for_submitted_gpu_work);
    assert!(
        config
            .clone()
            .wait_for_submitted_gpu_work(true)
            .wait_for_submitted_gpu_work
    );
    assert_eq!(
        config.clear_color,
        argui_core::Color::srgb(0.055, 0.065, 0.09)
    );
    assert_eq!(config.surface_alpha, SurfaceAlphaMode::Opaque);
    assert!(!config.profiling);
    assert_eq!(config.blur_algorithm, BlurAlgorithm::Auto);
    assert!(config.clone().profiling(true).profiling);
    assert_eq!(
        config
            .clone()
            .maximum_frame_latency(0)
            .maximum_frame_latency,
        1
    );
    assert_eq!(
        config
            .clone()
            .maximum_frame_latency(9)
            .maximum_frame_latency,
        3
    );
    assert_eq!(config.image_cache_bytes, 64 * 1024 * 1024);
    assert_eq!(config.gpu_canvas_cache_bytes, 128 * 1024 * 1024);
    assert_eq!(config.gradient_stop_capacity, 65_536);
    assert_eq!(config.damage_tracking, DamageTracking::enabled());
    assert_eq!(
        config.clone().image_cache_bytes(1024).image_cache_bytes,
        1024
    );
    assert_eq!(
        config.gradient_stop_capacity(256).gradient_stop_capacity,
        256
    );
}

#[test]
fn transparent_surface_configuration_uses_a_transparent_clear() {
    let optional = RendererConfig::default().surface_alpha(SurfaceAlphaMode::PreferTransparent);
    assert_eq!(optional.clear_color, argui_core::Color::TRANSPARENT);
    let config = RendererConfig::default().surface_alpha(SurfaceAlphaMode::Transparent);
    assert_eq!(config.surface_alpha, SurfaceAlphaMode::Transparent);
    assert_eq!(config.clear_color, argui_core::Color::TRANSPARENT);

    let opaque = RendererConfig::default().surface_alpha(SurfaceAlphaMode::Opaque);
    assert_eq!(opaque.surface_alpha, SurfaceAlphaMode::Opaque);
    assert_eq!(
        opaque.clear_color,
        argui_core::Color::srgb(0.055, 0.065, 0.09)
    );
}

#[test]
fn effect_quality_and_configuration_builders_are_composable() {
    assert_eq!(
        EffectQuality::Normal.settings(),
        EffectQualitySettings {
            blur_downsample_bias: 1,
            spatial_effect_divisor: 1,
        }
    );
    assert_eq!(EffectQuality::Balanced.settings().blur_downsample_bias, 2);
    assert_eq!(
        EffectQuality::Performance.settings().spatial_effect_divisor,
        2
    );

    let custom = EffectQualitySettings {
        blur_downsample_bias: 7,
        spatial_effect_divisor: 5,
    };
    assert_eq!(EffectQuality::Custom(custom).settings(), custom);

    let config = RendererConfig::default()
        .clear_color(argui_core::Color::BLACK)
        .image_cache_bytes(4096)
        .gpu_canvas_cache_bytes(8192)
        .gradient_stop_capacity(32)
        .damage_tracking(
            DamageTracking::enabled()
                .max_regions(3)
                .max_area_ratio(0.25),
        )
        .effect_quality(EffectQuality::Performance);
    assert_eq!(config.clear_color, argui_core::Color::BLACK);
    assert_eq!(config.image_cache_bytes, 4096);
    assert_eq!(config.gpu_canvas_cache_bytes, 8192);
    assert_eq!(config.gradient_stop_capacity, 32);
    assert_eq!(config.effect_quality, EffectQuality::Performance);
    assert_eq!(
        config
            .clone()
            .blur_algorithm(BlurAlgorithm::Gaussian)
            .blur_algorithm,
        BlurAlgorithm::Gaussian
    );
    assert_eq!(
        config
            .clone()
            .blur_algorithm(BlurAlgorithm::DualKawase)
            .blur_algorithm,
        BlurAlgorithm::DualKawase
    );
    assert_eq!(config.damage_tracking.max_regions, 3);
    assert_eq!(config.damage_tracking.max_area_ratio, 0.25);
    assert!(!DamageTracking::disabled().enabled);
}

/// Automatic blur selection follows the measured medium-radius crossover.
#[test]
fn automatic_blur_selection_preserves_explicit_modes() {
    let area = [512, 512];
    assert_eq!(
        BlurAlgorithm::Auto.resolve(2.0, area),
        BlurAlgorithm::Gaussian
    );
    assert_eq!(
        BlurAlgorithm::Auto.resolve(3.0, area),
        BlurAlgorithm::DualKawase
    );
    assert_eq!(
        BlurAlgorithm::Auto.resolve(6.0, area),
        BlurAlgorithm::DualKawase
    );
    assert_eq!(
        BlurAlgorithm::Auto.resolve(7.0, area),
        BlurAlgorithm::Gaussian
    );
    assert_eq!(
        BlurAlgorithm::Auto.resolve(24.0, area),
        BlurAlgorithm::Gaussian
    );
    assert_eq!(
        BlurAlgorithm::Auto.resolve(64.0, area),
        BlurAlgorithm::Gaussian
    );
    assert_eq!(
        BlurAlgorithm::Auto.resolve(4.0, [8, 512]),
        BlurAlgorithm::Gaussian
    );
    assert_eq!(
        BlurAlgorithm::Gaussian.resolve(4.0, area),
        BlurAlgorithm::Gaussian
    );
    assert_eq!(
        BlurAlgorithm::DualKawase.resolve(24.0, area),
        BlurAlgorithm::DualKawase
    );
}

#[test]
fn invalid_damage_thresholds_fall_back_to_safe_limits() {
    assert_eq!(DamageTracking::enabled().max_regions(0).max_regions, 1);
    assert_eq!(
        DamageTracking::enabled()
            .max_area_ratio(f32::NAN)
            .max_area_ratio,
        DamageTracking::default().max_area_ratio
    );
    assert_eq!(
        DamageTracking::enabled()
            .max_area_ratio(-5.0)
            .max_area_ratio,
        0.05
    );
    assert_eq!(
        DamageTracking::enabled().max_area_ratio(5.0).max_area_ratio,
        1.0
    );
}
