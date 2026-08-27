use argui_render::RendererConfig;

#[test]
fn renderer_defaults_to_vsync_and_a_discrete_gpu() {
    let config = RendererConfig::default();

    assert_eq!(
        config.power_preference,
        wgpu::PowerPreference::HighPerformance
    );
    assert_eq!(config.present_mode, wgpu::PresentMode::AutoVsync);
    assert_eq!(config.maximum_frame_latency, 2);
    assert_eq!(config.clear_color, [0.055, 0.065, 0.09, 1.0]);
    assert!(!config.profiling);
    assert!(config.profiling(true).profiling);
    assert_eq!(config.maximum_frame_latency(0).maximum_frame_latency, 1);
    assert_eq!(config.maximum_frame_latency(9).maximum_frame_latency, 3);
    assert_eq!(config.image_cache_bytes, 64 * 1024 * 1024);
    assert_eq!(config.gradient_stop_capacity, 65_536);
    assert_eq!(config.image_cache_bytes(1024).image_cache_bytes, 1024);
    assert_eq!(
        config.gradient_stop_capacity(256).gradient_stop_capacity,
        256
    );
}
