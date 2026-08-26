use argui_core::Color;

#[test]
fn colors_keep_linear_channels_explicit() {
    assert_eq!(Color::WHITE.as_array(), [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(Color::TRANSPARENT.as_array(), [0.0, 0.0, 0.0, 0.0]);
    assert_eq!(Color::rgb(0.1, 0.2, 0.3).as_array(), [0.1, 0.2, 0.3, 1.0]);
    assert_eq!(
        Color::rgba(0.1, 0.2, 0.3, 0.4).as_array(),
        [0.1, 0.2, 0.3, 0.4]
    );
}
