#![cfg(feature = "shadow")]

use argui_effects::{DropShadow, Glow};
use argui_paint::Color;

#[test]
fn shadow_presets_encode_drop_and_glow_geometry() {
    let drop = DropShadow::new([2.0, 3.0], 8.0, Color::WHITE).0;
    assert_eq!(drop.offset, [2.0, 3.0]);
    assert_eq!(drop.blur, 8.0);
    assert!(!drop.inset);

    let glow = Glow::new(6.0, Color::BLACK).0;
    assert_eq!(glow.offset, [0.0, 0.0]);
    assert_eq!(glow.blur, 6.0);
    assert!(!glow.inset);
}
