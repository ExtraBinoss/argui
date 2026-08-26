use argui_render::RendererConfig;

#[test]
fn renderer_defaults_to_vsync_and_a_discrete_gpu() {
    let config = RendererConfig::default();

    assert_eq!(
        config.power_preference,
        wgpu::PowerPreference::HighPerformance
    );
    assert_eq!(config.present_mode, wgpu::PresentMode::AutoVsync);
    assert_eq!(config.clear_color, [0.055, 0.065, 0.09, 1.0]);
}
