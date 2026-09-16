use argui_ui::ScrollConfig;

#[test]
fn touch_scroll_direction_is_natural_by_default_and_configurable() {
    assert!(ScrollConfig::default().natural_touch_scroll);
    assert!(
        !ScrollConfig::default()
            .natural_touch_scroll(false)
            .natural_touch_scroll
    );
}
