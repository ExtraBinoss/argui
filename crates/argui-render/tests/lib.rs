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
