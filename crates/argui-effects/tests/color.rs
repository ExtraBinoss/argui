#![cfg(feature = "color")]

use argui_effects::{Brightness, Contrast, HueRotate, Opacity, Saturation};
use argui_paint::Filter;

#[test]
fn color_presets_preserve_their_authored_values() {
    assert_eq!(Brightness(1.2).filter(), Filter::Brightness(1.2));
    assert_eq!(Contrast(0.8).filter(), Filter::Contrast(0.8));
    assert_eq!(Saturation(1.4).filter(), Filter::Saturation(1.4));
    assert_eq!(HueRotate(0.25).filter(), Filter::HueRotate(0.25));
    assert_eq!(Opacity(0.6).filter(), Filter::Opacity(0.6));
}
