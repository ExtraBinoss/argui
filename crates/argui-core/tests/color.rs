use argui_core::Color;

#[test]
fn colors_keep_linear_channels_explicit() {
    assert_eq!(Color::WHITE.to_linear_rgba(), [1.0, 1.0, 1.0, 1.0]);
    assert_eq!(Color::TRANSPARENT.to_linear_rgba(), [0.0, 0.0, 0.0, 0.0]);
    assert_eq!(
        Color::linear_rgb(0.1, 0.2, 0.3).to_linear_rgba(),
        [0.1, 0.2, 0.3, 1.0]
    );
    assert_eq!(
        Color::linear_rgba(0.1, 0.2, 0.3, 0.4).to_linear_rgba(),
        [0.1, 0.2, 0.3, 0.4]
    );
}
