use argui_core::{Color, ColorInterpolation, ColorScheme, ParseColorError};

fn close(left: f32, right: f32) {
    assert!((left - right).abs() < 0.000_01, "{left} != {right}");
}

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

#[test]
fn system_bar_icons_follow_background_contrast() {
    assert_eq!(Color::WHITE.preferred_contrast_scheme(), ColorScheme::Light);
    assert_eq!(Color::BLACK.preferred_contrast_scheme(), ColorScheme::Dark);
    assert_eq!(
        Color::from_hex("#f7f9ff")
            .unwrap()
            .preferred_contrast_scheme(),
        ColorScheme::Light
    );
    assert_eq!(
        Color::from_hex("#141824")
            .unwrap()
            .preferred_contrast_scheme(),
        ColorScheme::Dark
    );
}

#[test]
fn srgb_uses_the_standard_transfer_curve() {
    let [threshold, mid, white, alpha] = Color::srgba(0.04045, 0.5, 1.0, 0.25).to_linear_rgba();
    close(threshold, 0.003_130_805);
    close(mid, 0.214_041_14);
    close(white, 1.0);
    close(alpha, 0.25);
    assert_eq!(Color::srgb(0.5, 0.5, 0.5).to_srgba8(), [128, 128, 128, 255]);
}

#[test]
fn hexadecimal_forms_are_css_compatible() {
    assert_eq!(
        Color::from_hex("#09f").unwrap().to_srgba8(),
        [0, 153, 255, 255]
    );
    assert_eq!(
        Color::from_hex("#09f8").unwrap().to_srgba8(),
        [0, 153, 255, 136]
    );
    assert_eq!(
        Color::from_hex("#102030").unwrap().to_srgba8(),
        [16, 32, 48, 255]
    );
    assert_eq!(
        Color::from_hex("#10203040").unwrap().to_srgba8(),
        [16, 32, 48, 64]
    );
    assert_eq!(Color::from_hex("102030"), Err(ParseColorError::MissingHash));
    assert_eq!(Color::from_hex("#12"), Err(ParseColorError::InvalidLength));
    assert_eq!(Color::from_hex("#xyz"), Err(ParseColorError::InvalidDigit));
}

#[test]
fn oklch_literals_preserve_the_official_theme_channels_and_alpha() {
    assert_eq!(
        Color::from_literal("oklch(1 0 0)").unwrap().to_srgba8(),
        [255, 255, 255, 255]
    );
    assert_eq!(
        Color::from_literal("oklch(1 0 0 / 10%)")
            .unwrap()
            .to_srgba8(),
        [255, 255, 255, 26]
    );
    assert_eq!(
        Color::from_literal("oklch(100% 0 360deg / 50%)")
            .unwrap()
            .to_srgba8(),
        [255, 255, 255, 128]
    );
    let blue = Color::from_literal("oklch(0.546 0.245 262.881)").unwrap();
    assert!(blue.to_srgba8()[2] > blue.to_srgba8()[0]);
    for invalid in [
        "oklch(1 0)",
        "oklch(1 0 0 / 110%)",
        "oklch(NaN 0 0)",
        "oklch(1 -1 0)",
        "oklch(1 0 0",
    ] {
        assert_eq!(
            Color::from_literal(invalid),
            Err(ParseColorError::InvalidOklch)
        );
    }
}

#[test]
fn rgb_literals_accept_css_forms_and_reject_malformed_channels() {
    for literal in ["rgb(255, 0, 128)", "rgb(255 0 128)", "rgb(100% 0% 50.196%)"] {
        assert_eq!(
            Color::from_literal(literal).unwrap().to_srgba8(),
            [255, 0, 128, 255]
        );
    }
    for literal in [
        "rgba(255, 0, 128, 0.5)",
        "rgb(255 0 128 / 50%)",
        "rgba(255 0 128 / 50%)",
    ] {
        assert_eq!(
            Color::from_literal(literal).unwrap().to_srgba8(),
            [255, 0, 128, 128]
        );
    }
    for invalid in [
        "rgb(255 0)",
        "rgb(256 0 0)",
        "rgb(NaN 0 0)",
        "rgb(255, 0, 0, 0.5)",
        "rgba(255, 0, 0)",
        "rgb(255 0 0 / 110%)",
        "rgb(255 0 0",
    ] {
        assert_eq!(
            Color::from_literal(invalid),
            Err(ParseColorError::InvalidRgb)
        );
    }
}

#[test]
fn interpolation_is_alpha_safe_and_space_specific() {
    let transparent_red = Color::srgba(1.0, 0.0, 0.0, 0.0);
    let blue = Color::srgb(0.0, 0.0, 1.0);
    let midpoint = transparent_red.mix(blue, 0.5, ColorInterpolation::LinearSrgb);
    let [red, green, blue, alpha] = midpoint.to_linear_rgba();
    close(red, 0.0);
    close(green, 0.0);
    close(blue, 1.0);
    close(alpha, 0.5);

    let linear = Color::BLACK.mix(Color::WHITE, 0.5, ColorInterpolation::LinearSrgb);
    let perceptual = Color::BLACK.mix(Color::WHITE, 0.5, ColorInterpolation::Oklab);
    assert!(perceptual.relative_luminance() < linear.relative_luminance());
    close(Color::BLACK.contrast_ratio(Color::WHITE), 21.0);
}

#[test]
fn extended_linear_channels_round_trip() {
    let color = Color::linear_rgba(-0.25, 0.5, 1.5, 0.75);
    let srgb = color.to_srgba();
    let round_trip = Color::srgba(srgb[0], srgb[1], srgb[2], srgb[3]).to_linear_rgba();
    for (actual, expected) in round_trip.into_iter().zip(color.to_linear_rgba()) {
        close(actual, expected);
    }
    assert_eq!(color.with_alpha(0.2).to_linear_rgba()[3], 0.2);
}
