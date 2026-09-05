#![cfg(feature = "blur")]

use argui_effects::Blur;
use argui_paint::Filter;

#[test]
fn blur_preset_builds_the_matching_filter() {
    assert_eq!(Blur(12.5).filter(), Filter::Blur(12.5));
}
