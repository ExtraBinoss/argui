use argui_ui::{ScrollConfig, ScrollPhysics};

#[test]
fn touch_scroll_direction_is_natural_by_default_and_configurable() {
    assert!(ScrollConfig::default().natural_touch_scroll);
    assert!(
        !ScrollConfig::default()
            .natural_touch_scroll(false)
            .natural_touch_scroll
    );
}

#[test]
fn scroll_containers_have_light_inertia_by_default() {
    assert!(matches!(
        ScrollConfig::default().physics,
        ScrollPhysics::Inertial(config) if (config.decay - 12.15).abs() < f32::EPSILON
    ));
}
