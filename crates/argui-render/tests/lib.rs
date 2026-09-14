use naga::valid::{Capabilities, ValidationFlags, Validator};

const PRIMITIVES: [(&str, &str); 4] = [
    ("quad", include_str!("../src/shaders/primitives/quad.wgsl")),
    ("text", include_str!("../src/shaders/primitives/text.wgsl")),
    (
        "image",
        include_str!("../src/shaders/primitives/image.wgsl"),
    ),
    (
        "vector",
        include_str!("../src/shaders/primitives/vector.wgsl"),
    ),
];

fn validate(source: &str) -> Result<(), String> {
    let module =
        naga::front::wgsl::parse_str(source).map_err(|error| error.emit_to_string(source))?;
    Validator::new(ValidationFlags::all(), Capabilities::empty())
        .validate(&module)
        .map(|_| ())
        .map_err(|error| error.emit_to_string(source))
}

#[test]
fn primitive_shaders_validate_with_all_checks_enabled() {
    for (name, source) in PRIMITIVES {
        validate(source).unwrap_or_else(|error| panic!("{name}: {error}"));
    }
}

fn srgb_to_linear(value: f32) -> f32 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(value: f32) -> f32 {
    if value <= 0.003_130_8 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    }
}

#[test]
fn gamma_corrected_mask_keeps_opposite_text_polarities_symmetric() {
    for coverage in [0.05_f32, 0.25, 0.5, 0.75, 0.95] {
        let light_on_dark = linear_to_srgb(srgb_to_linear(coverage));
        let dark_coverage = 1.0 - srgb_to_linear(1.0 - coverage);
        let dark_on_light = linear_to_srgb(1.0 - dark_coverage);

        assert!((light_on_dark - coverage).abs() < 0.000_01);
        assert!((dark_on_light - (1.0 - coverage)).abs() < 0.000_01);
    }
}
