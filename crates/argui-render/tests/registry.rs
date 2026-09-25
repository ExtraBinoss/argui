use argui_paint::{EffectId, EffectInstance, EffectValue};
use argui_render::{
    EffectDamage, EffectDefinition, EffectInput, EffectParameter, EffectParameterType,
    EffectPassDefinition, EffectRegistry, RendererError,
};

const WGSL: &str = r#"
fn argui_effect(_uv: vec2<f32>, source: vec4<f32>, _backdrop: vec4<f32>) -> vec4<f32> {
    return source * argui_param_f32(0u);
}
"#;

fn parameters() -> [EffectParameter; 1] {
    [EffectParameter::new("amount", EffectParameterType::F32)]
}

fn passes() -> [EffectPassDefinition; 1] {
    [EffectPassDefinition::fragment("main", WGSL)]
}

fn definition(id: EffectId) -> EffectDefinition {
    EffectDefinition::new(id, parameters(), passes())
}

#[test]
fn registry_requires_namespaces_unique_ids_and_nonzero_revisions() {
    assert!(EffectRegistry::new([definition(EffectId::new("plain"))]).is_err());
    assert!(EffectRegistry::new([definition(EffectId::new(""))]).is_err());
    let id = EffectId::new("test.effect");
    assert!(EffectRegistry::new([definition(id.clone()), definition(id.clone())]).is_err());
    assert_eq!(
        EffectRegistry::new([definition(id)])
            .unwrap()
            .definitions()
            .len(),
        1
    );
    assert!(
        definition(EffectId::new("test.zero-revision"))
            .with_revision(0)
            .validate()
            .is_err()
    );
}

#[test]
fn extending_a_registry_preserves_clones_order_and_validation() {
    let first = EffectId::new("test.first");
    let second = EffectId::new("test.second");
    let original = EffectRegistry::new([definition(first.clone())]).unwrap();
    let extended = original
        .clone()
        .with_definition(definition(second.clone()))
        .unwrap();
    assert_eq!(
        extended.definitions(),
        &[definition(first.clone()), definition(second.clone())]
    );
    assert_eq!(original.definitions(), &[definition(first.clone())]);
    assert!(original.get(&second).is_none());
    assert_eq!(extended.get(&second), Some(&definition(second.clone())));
    assert!(matches!(
        extended.clone().with_definition(definition(first)),
        Err(RendererError::DuplicateEffect(id)) if id == EffectId::new("test.first")
    ));
    let invalid = EffectDefinition::new(
        EffectId::new("test.unused"),
        parameters(),
        [EffectPassDefinition::fragment("main", "invalid shader")],
    );
    assert!(matches!(
        extended.clone().with_definition(invalid),
        Err(RendererError::InvalidShader(_))
    ));
    assert_eq!(extended.definitions().len(), 2);
}

#[test]
fn replacement_is_revisioned_transactional_and_removable() {
    let id = EffectId::new("test.live");
    let original = EffectRegistry::new([definition(id.clone())]).unwrap();
    let replacement = definition(id.clone()).with_revision(2);
    let updated = original
        .clone()
        .with_replacement(replacement.clone())
        .unwrap();
    assert_eq!(original.get(&id).unwrap().revision, 1);
    assert_eq!(updated.get(&id), Some(&replacement));

    let invalid = EffectDefinition::new(
        id.clone(),
        parameters(),
        [EffectPassDefinition::fragment("main", "broken")],
    )
    .with_revision(3);
    assert!(updated.clone().with_replacement(invalid).is_err());
    assert_eq!(updated.get(&id), Some(&replacement));
    assert!(matches!(
        original
            .clone()
            .with_replacement(definition(EffectId::new("test.missing"))),
        Err(RendererError::MissingEffect(_))
    ));
    assert_eq!(
        original
            .clone()
            .without_definition(&EffectId::new("test.missing"))
            .definitions(),
        original.definitions()
    );
    assert!(updated.without_definition(&id).is_empty());
}

#[test]
fn instances_are_checked_by_parameter_name_and_type() {
    let definition = definition(EffectId::new("test.effect"));
    assert!(
        definition
            .validate_instance(&EffectInstance::new(
                definition.id.clone(),
                [("amount", EffectValue::F32(0.5))],
            ))
            .is_ok()
    );
    assert!(
        definition
            .validate_instance(&EffectInstance::new(
                definition.id.clone(),
                [("wrong", EffectValue::F32(0.5))],
            ))
            .is_err()
    );
    assert!(
        definition
            .validate_instance(&EffectInstance::new(
                definition.id.clone(),
                [("amount", EffectValue::U32(1))],
            ))
            .is_err()
    );
    assert!(
        definition
            .validate_instance(&EffectInstance::new(
                definition.id.clone(),
                Vec::<(&str, EffectValue)>::new(),
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
    let definition = definition(EffectId::new("test.bounded")).damage(EffectDamage::Bounded);
    assert_eq!(definition.damage, EffectDamage::Bounded);
    assert_eq!(EffectDamage::default(), EffectDamage::Unbounded);
    let pass = EffectPassDefinition::fragment("configured", WGSL)
        .inputs(Vec::<EffectInput>::new())
        .downsampled(4);
    assert!(pass.inputs.is_empty());
    assert_eq!(pass.scale_divisor, 4);
    assert_eq!(pass.downsampled(0).scale_divisor, 1);
}

#[test]
fn invalid_public_definitions_are_rejected() {
    let invalid = vec![
        EffectDefinition::new(
            EffectId::new("test.empty"),
            parameters(),
            Vec::<EffectPassDefinition>::new(),
        ),
        EffectDefinition::new(
            EffectId::new("test.empty-parameter"),
            [EffectParameter::new("", EffectParameterType::F32)],
            passes(),
        ),
        EffectDefinition::new(
            EffectId::new("test.duplicate-parameters"),
            [
                EffectParameter::new("same", EffectParameterType::F32),
                EffectParameter::new("same", EffectParameterType::F32),
            ],
            passes(),
        ),
        EffectDefinition::new(
            EffectId::new("test.empty-pass"),
            parameters(),
            [EffectPassDefinition::fragment("", WGSL)],
        ),
        EffectDefinition::new(
            EffectId::new("test.duplicate-passes"),
            parameters(),
            [
                EffectPassDefinition::fragment("same", WGSL),
                EffectPassDefinition::fragment("same", WGSL),
            ],
        ),
        EffectDefinition::new(
            EffectId::new("test.zero-scale"),
            parameters(),
            [EffectPassDefinition {
                name: "main".into(),
                wgsl: WGSL.into(),
                inputs: Vec::new().into(),
                scale_divisor: 0,
            }],
        ),
        EffectDefinition::new(
            EffectId::new("test.bad-wgsl"),
            parameters(),
            [EffectPassDefinition::fragment("main", "not wgsl")],
        ),
    ];
    for definition in invalid {
        assert!(definition.validate().is_err());
    }

    let registry = EffectRegistry::new([definition(EffectId::new("test.effect"))]).unwrap();
    assert!(registry.get(&EffectId::new("test.effect")).is_some());
    assert!(registry.get(&EffectId::new("test.missing")).is_none());
    assert_eq!(registry.maximum_parameter_words(), 1);
    assert!(!registry.is_empty());
    assert!(EffectRegistry::default().is_empty());
}
