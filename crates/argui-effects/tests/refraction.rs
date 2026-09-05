#![cfg(feature = "refraction")]

use argui_effects::Refraction;
use argui_paint::{Filter, Refraction as PaintRefraction};

#[test]
fn refraction_preset_composes_strength_and_chromatic_aberration() {
    assert_eq!(
        Refraction::new(8.0).chromatic_aberration(1.5).filter(),
        Filter::Refraction(PaintRefraction::new(8.0).chromatic_aberration(1.5))
    );
}
