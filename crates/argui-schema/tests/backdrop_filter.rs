use argui_core::Color;
use argui_paint::{EffectValue, Filter};
use argui_schema::backdrop_filter::parse;

#[test]
fn parses_css_functions_in_order_with_css_units() {
    let filters = parse(
        "blur(2px) brightness(60%) contrast(40%) grayscale(30%) hue-rotate(120deg) \
         invert(70%) opacity(20%) sepia(90%) saturate(80%)",
    )
    .unwrap();
    assert_eq!(filters.len(), 9);
    assert_eq!(filters[0], Filter::Blur(2.0));
    assert_eq!(filters[1], Filter::Brightness(0.6));
    assert_eq!(filters[2], Filter::Contrast(0.4));
    assert!(matches!(filters[3], Filter::ColorMatrix(_)));
    assert!(matches!(filters[4], Filter::HueRotate(value) if (value - 2.094_395).abs() < 0.000_1));
    assert!(matches!(filters[5], Filter::ColorMatrix(_)));
    assert_eq!(filters[6], Filter::Opacity(0.2));
    assert!(matches!(filters[7], Filter::ColorMatrix(_)));
    assert_eq!(filters[8], Filter::Saturation(0.8));
}

#[test]
fn parses_shadow_and_argui_extensions() {
    let filters = parse("drop-shadow(4px 4px 10px blue) refraction(8px, 1px, 50%)").unwrap();
    let Filter::DropShadow(shadow) = filters[0] else {
        panic!("expected drop shadow");
    };
    assert_eq!(shadow.offset, [4.0, 4.0]);
    assert_eq!(shadow.blur, 10.0);
    assert_eq!(shadow.color, Color::srgb(0.0, 0.0, 1.0));
    assert!(
        matches!(filters[1], Filter::Refraction(value) if value.strength == 8.0 && value.chromatic_aberration == 1.0 && value.edge == 0.5)
    );

    let matrix = parse("color-matrix(1 0 0 0 0, 0 1 0 0 0, 0 0 1 0 0, 0 0 0 1 0)").unwrap();
    assert!(
        matches!(matrix[0], Filter::ColorMatrix(values) if values[0] == 1.0 && values[19] == 0.0)
    );
    let modern_color = parse("drop-shadow(0.0 4px 3px rgb(0 0 255 / 50%))").unwrap();
    assert!(
        matches!(modern_color[0], Filter::DropShadow(shadow) if shadow.color == Color::srgba(0.0, 0.0, 1.0, 0.5))
    );
}

#[test]
fn omitted_css_function_values_use_standard_defaults() {
    assert_eq!(
        parse("blur() brightness() contrast() hue-rotate() saturate() opacity()").unwrap(),
        vec![
            Filter::Blur(0.0),
            Filter::Brightness(1.0),
            Filter::Contrast(1.0),
            Filter::HueRotate(0.0),
            Filter::Saturation(1.0),
            Filter::Opacity(1.0),
        ]
    );
}

#[test]
fn custom_effect_keeps_imported_shader_identity_and_live_parameters() {
    let filters = parse("blur(2px) effect(gallery.examples.prism strength=0.35 phase=1.25)")
        .expect("custom effect filter");
    let Filter::Effect(effect) = &filters[1] else {
        panic!("expected custom effect");
    };
    assert_eq!(effect.id.as_str(), "gallery.examples.prism");
    assert_eq!(effect.parameters.len(), 2);
    assert_eq!(effect.parameters[0].name.as_str(), "strength");
    assert_eq!(effect.parameters[0].value, EffectValue::F32(0.35));
    assert_eq!(effect.parameters[1].value, EffectValue::F32(1.25));
    for invalid in [
        "effect()",
        "effect(prism strength=0.4)",
        "effect(gallery.prism strength=NaN)",
        "effect(gallery.prism strength)",
        "effect(gallery.prism a/b=1)",
    ] {
        assert!(parse(invalid).is_err(), "{invalid}");
    }
}

#[test]
fn css_color_matrices_keep_alpha_and_expected_channels() {
    let filters = parse("grayscale(100%) invert(100%) sepia(100%)").unwrap();
    let Filter::ColorMatrix(gray) = filters[0] else {
        panic!("expected grayscale matrix")
    };
    assert!((gray[0] - 0.2126).abs() < 0.0001);
    assert_eq!(gray[15], 1.0);
    let Filter::ColorMatrix(inverted) = filters[1] else {
        panic!("expected invert matrix")
    };
    assert_eq!(inverted[0], -1.0);
    assert_eq!(inverted[16], 1.0);
    assert_eq!(inverted[19], 0.0);
    let Filter::ColorMatrix(sepia) = filters[2] else {
        panic!("expected sepia matrix")
    };
    assert_eq!(sepia[0], 0.393);
    assert_eq!(sepia[15], 1.0);
}

#[test]
fn rejects_unresolved_or_invalid_css_without_silent_fallback() {
    for (input, expected) in [
        ("url(\"filters.svg#filter\")", "SVG url()"),
        ("inherit", "cascade"),
        ("revert-layer", "cascade"),
        ("blur(2em)", "pixel length"),
        ("blur(-2px)", "nonnegative"),
        ("hue-rotate(8px)", "CSS angle"),
        ("brightness(NaN)", "finite"),
        ("blur(2px)contrast(2)", "whitespace"),
        ("drop-shadow(4px)", "two offsets"),
        ("color-matrix(1,2)", "20 coefficients"),
    ] {
        assert!(parse(input).unwrap_err().contains(expected), "{input}");
    }
    for keyword in ["none", "initial", "unset"] {
        assert!(parse(keyword).unwrap().is_empty());
    }
}
