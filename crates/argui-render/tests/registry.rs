use argui_paint::{EffectId, EffectInstance, EffectValue};
use argui_render::{
    EffectDefinition, EffectParameter, EffectParameterType, EffectPassDefinition, EffectRegistry,
};

const WGSL: &str = r#"
fn argui_effect(_uv: vec2<f32>, source: vec4<f32>, _backdrop: vec4<f32>) -> vec4<f32> {
    return source * argui_param_f32(0u);
}
"#;
const PARAMETERS: &[EffectParameter] = &[EffectParameter::new("amount", EffectParameterType::F32)];
const PASSES: &[EffectPassDefinition] = &[EffectPassDefinition::fragment("main", WGSL)];

fn definition(id: EffectId) -> EffectDefinition {
    EffectDefinition::new(id, PARAMETERS, PASSES)
}

#[test]
fn registry_requires_namespaces_and_unique_ids() {
    assert!(EffectRegistry::new([definition(EffectId::new("plain"))]).is_err());
    assert!(EffectRegistry::new([definition(EffectId::new(""))]).is_err());
    let id = EffectId::new("test.effect");
    assert!(EffectRegistry::new([definition(id), definition(id)]).is_err());
    assert_eq!(
        EffectRegistry::new([definition(id)])
            .unwrap()
            .definitions()
            .len(),
        1
    );
}

#[test]
fn instances_are_checked_by_parameter_name_and_type() {
    let definition = definition(EffectId::new("test.effect"));
    assert!(
        definition
            .validate_instance(&EffectInstance::new(
                definition.id,
                [("amount", EffectValue::F32(0.5))],
            ))
            .is_ok()
    );
    assert!(
        definition
            .validate_instance(&EffectInstance::new(
                definition.id,
                [("wrong", EffectValue::F32(0.5))],
            ))
            .is_err()
    );
    assert!(
        definition
            .validate_instance(&EffectInstance::new(
                definition.id,
                [("amount", EffectValue::U32(1))],
            ))
            .is_err()
    );
}

#[test]
fn parameter_widths_and_builders_are_publicly_consistent() {
    let cases = [
        (EffectParameterType::F32, 1),
        (EffectParameterType::I32, 1),
        (EffectParameterType::U32, 1),
        (EffectParameterType::Bool, 1),
        (EffectParameterType::Vec2, 2),
        (EffectParameterType::Vec3, 3),
        (EffectParameterType::Vec4, 4),
        (EffectParameterType::Mat3, 9),
        (EffectParameterType::Mat4, 16),
        (EffectParameterType::Color, 4),
        (EffectParameterType::LogicalPixels, 1),
    ];
    for (parameter_type, words) in cases {
        assert_eq!(parameter_type.words(), words);
    }
    assert_eq!(
        definition(EffectId::new("test.effect")).parameter_words(),
        1
    );
}

#[test]
fn invalid_public_definitions_are_rejected() {
    const DUPLICATE_PARAMETERS: &[EffectParameter] = &[
        EffectParameter::new("same", EffectParameterType::F32),
        EffectParameter::new("same", EffectParameterType::F32),
    ];
    const EMPTY_PARAMETER: &[EffectParameter] =
        &[EffectParameter::new("", EffectParameterType::F32)];
    const DUPLICATE_PASSES: &[EffectPassDefinition] = &[
        EffectPassDefinition::fragment("same", WGSL),
        EffectPassDefinition::fragment("same", WGSL),
    ];
    const EMPTY_PASS: &[EffectPassDefinition] = &[EffectPassDefinition::fragment("", WGSL)];
    const ZERO_SCALE: &[EffectPassDefinition] = &[EffectPassDefinition {
        name: "main",
        wgsl: WGSL,
        inputs: &[],
        scale_divisor: 0,
    }];
    const BAD_WGSL: &[EffectPassDefinition] = &[EffectPassDefinition::fragment("main", "not wgsl")];

    let configured = EffectPassDefinition::fragment("configured", WGSL)
        .inputs(&[])
        .downsampled(4);
    assert!(configured.inputs.is_empty());
    assert_eq!(configured.scale_divisor, 4);
    assert_eq!(configured.downsampled(0).scale_divisor, 1);

    for invalid in [
        EffectDefinition::new(EffectId::new("test.empty"), PARAMETERS, &[]),
        EffectDefinition::new(
            EffectId::new("test.empty-parameter"),
            EMPTY_PARAMETER,
            PASSES,
        ),
        EffectDefinition::new(
            EffectId::new("test.duplicate-parameters"),
            DUPLICATE_PARAMETERS,
            PASSES,
        ),
        EffectDefinition::new(EffectId::new("test.empty-pass"), PARAMETERS, EMPTY_PASS),
        EffectDefinition::new(
            EffectId::new("test.duplicate-passes"),
            PARAMETERS,
            DUPLICATE_PASSES,
        ),
        EffectDefinition::new(EffectId::new("test.zero-scale"), PARAMETERS, ZERO_SCALE),
        EffectDefinition::new(EffectId::new("test.bad-wgsl"), PARAMETERS, BAD_WGSL),
    ] {
        assert!(invalid.validate().is_err());
    }

    let registry = EffectRegistry::new([definition(EffectId::new("test.effect"))]).unwrap();
    assert!(registry.get(EffectId::new("test.effect")).is_some());
    assert!(registry.get(EffectId::new("test.missing")).is_none());
    assert!(!registry.is_empty());
    assert!(EffectRegistry::default().is_empty());
}
