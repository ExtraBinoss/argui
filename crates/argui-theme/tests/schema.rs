use std::sync::Arc;

use argui_core::Color;
use argui_theme::{
    ThemeError, ThemeImpact, ThemeRuntime, ThemeSchema, ThemeTokenDefinition, ThemeValue,
    ThemeValueType,
};

fn schema() -> Arc<ThemeSchema> {
    Arc::new(
        ThemeSchema::new([
            ThemeTokenDefinition::new("background", ThemeValue::Color(Color::WHITE))
                .impact(ThemeImpact::Paint),
            ThemeTokenDefinition::derived("button-background", ThemeValueType::Color, "background")
                .impact(ThemeImpact::Paint),
            ThemeTokenDefinition::new("spacing", ThemeValue::Length(8.0))
                .impact(ThemeImpact::Layout),
        ])
        .unwrap(),
    )
}

#[test]
fn schema_assigns_dense_typed_ids_and_resolves_derived_defaults() {
    let schema = schema();
    let background = schema.token("background").unwrap();
    let button = schema.token("button-background").unwrap();
    let spacing = schema.token("spacing").unwrap();

    assert_eq!(background.index(), 0);
    assert_eq!(button.index(), 1);
    assert_eq!(spacing.index(), 2);
    assert_eq!(
        schema.default_value(button),
        Some(&ThemeValue::Color(Color::WHITE))
    );
    assert_eq!(
        schema.definition(spacing).unwrap().value_type(),
        ThemeValueType::Length
    );
}

#[test]
fn schema_rejects_type_mismatches_invalid_values_and_cycles() {
    assert!(matches!(
        ThemeSchema::new([
            ThemeTokenDefinition::new("color", ThemeValue::Color(Color::WHITE)),
            ThemeTokenDefinition::derived("space", ThemeValueType::Length, "color"),
        ]),
        Err(ThemeError::TypeMismatch { .. })
    ));
    assert_eq!(
        ThemeSchema::new([ThemeTokenDefinition::new(
            "duration",
            ThemeValue::DurationMillis(-1.0),
        )])
        .unwrap_err(),
        ThemeError::InvalidValue(String::from("duration"))
    );
    assert_eq!(
        ThemeSchema::new([
            ThemeTokenDefinition::derived("a", ThemeValueType::Color, "b"),
            ThemeTokenDefinition::derived("b", ThemeValueType::Color, "a"),
        ])
        .unwrap_err(),
        ThemeError::ReferenceCycle(vec![
            String::from("a"),
            String::from("b"),
            String::from("a"),
        ])
    );
}

#[test]
fn variants_and_overrides_update_derived_tokens_atomically() {
    let schema = schema();
    let background = schema.token("background").unwrap();
    let button = schema.token("button-background").unwrap();
    let spacing = schema.token("spacing").unwrap();
    let runtime = ThemeRuntime::new(schema);
    runtime
        .define_variant(
            "dark",
            [
                (background, ThemeValue::Color(Color::BLACK)),
                (spacing, ThemeValue::Length(12.0)),
            ],
        )
        .unwrap();

    let change = runtime.activate("dark").unwrap();
    assert_eq!(change.tokens(), [background, button, spacing]);
    assert_eq!(change.impact(), Some(ThemeImpact::Layout));
    assert_eq!(runtime.value(button), Some(ThemeValue::Color(Color::BLACK)));

    let override_change = runtime
        .set_override(background, ThemeValue::Color(Color::TRANSPARENT))
        .unwrap();
    assert_eq!(override_change.tokens(), [background, button]);
    assert_eq!(
        runtime.value(button),
        Some(ThemeValue::Color(Color::TRANSPARENT))
    );
    assert_eq!(
        runtime.remove_override(background).tokens(),
        [background, button]
    );
}

#[test]
fn token_notifications_are_selective() {
    let schema = schema();
    let background = schema.token("background").unwrap();
    let spacing = schema.token("spacing").unwrap();
    let runtime = ThemeRuntime::new(schema);
    let reader = runtime.clone();
    let receiver = reader.subscribe_tokens(&[background]).unwrap();
    assert_eq!(
        reader.value(background),
        Some(ThemeValue::Color(Color::WHITE))
    );

    runtime
        .set_override(spacing, ThemeValue::Length(16.0))
        .unwrap();
    assert!(receiver.try_recv().is_err());
    assert_eq!(
        reader.value(background),
        Some(ThemeValue::Color(Color::WHITE))
    );

    runtime
        .set_override(background, ThemeValue::Color(Color::BLACK))
        .unwrap();
    assert_eq!(
        receiver.try_recv().unwrap().tokens(),
        [background, schema_token_button(&reader)]
    );
    assert_eq!(
        reader.value(background),
        Some(ThemeValue::Color(Color::BLACK))
    );
}

fn schema_token_button(runtime: &ThemeRuntime) -> argui_theme::ThemeTokenId {
    runtime.schema().token("button-background").unwrap()
}

#[test]
fn schema_reports_duplicate_unknown_and_introspected_tokens() {
    assert!(matches!(
        ThemeSchema::new([
            ThemeTokenDefinition::new("same", ThemeValue::Bool(true)),
            ThemeTokenDefinition::new("same", ThemeValue::Bool(false)),
        ]),
        Err(ThemeError::DuplicateToken(_))
    ));
    assert_eq!(
        ThemeSchema::new([ThemeTokenDefinition::derived(
            "derived",
            ThemeValueType::Color,
            "missing"
        )])
        .unwrap_err(),
        ThemeError::UnknownToken("missing".into())
    );
    let empty = ThemeSchema::new([]).unwrap();
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);
    let schema = schema();
    assert!(!schema.is_empty());
    assert_eq!(schema.definitions().len(), schema.len());
    assert_eq!(schema.token("missing"), None);
    let definition = schema.definition(schema.token("spacing").unwrap()).unwrap();
    assert_eq!(definition.key(), "spacing");
    assert_eq!(definition.impact_class(), ThemeImpact::Layout);
}

#[test]
fn variant_and_override_noops_rejections_and_replacements_are_explicit() {
    let schema = schema();
    let background = schema.token("background").unwrap();
    let spacing = schema.token("spacing").unwrap();
    let runtime = ThemeRuntime::new(schema.clone());
    assert_eq!(runtime.schema().len(), 3);
    assert_eq!(
        runtime.value(background),
        Some(ThemeValue::Color(Color::WHITE))
    );
    assert_eq!(runtime.token_revision(background), Some(0));
    assert!(runtime.clear_variant().is_empty());
    assert!(runtime.remove_override(background).is_empty());
    assert_eq!(
        runtime.activate("missing"),
        Err(ThemeError::UnknownVariant("missing".into()))
    );
    assert!(matches!(
        runtime.define_variant("bad", [(spacing, ThemeValue::Bool(true))]),
        Err(ThemeError::TypeMismatch { .. })
    ));
    assert!(matches!(
        runtime
            .set_override(background, ThemeValue::Color(Color::WHITE))
            .unwrap()
            .tokens(),
        []
    ));
    assert!(
        runtime
            .set_override(background, ThemeValue::Color(Color::WHITE))
            .unwrap()
            .is_empty()
    );
    assert_eq!(runtime.token_revision(background), Some(0));
    assert!(matches!(
        runtime.set_override(background, ThemeValue::Length(2.0)),
        Err(ThemeError::TypeMismatch { .. })
    ));
    runtime
        .define_variant("dark", [(background, ThemeValue::Color(Color::BLACK))])
        .unwrap();
    let changed = runtime.activate("dark").unwrap();
    assert!(changed.is_empty());
    assert!(runtime.activate("dark").unwrap().is_empty());
    assert_eq!(
        runtime.value(background),
        Some(ThemeValue::Color(Color::WHITE))
    );
    runtime.remove_override(background);
    assert_eq!(
        runtime.value(background),
        Some(ThemeValue::Color(Color::BLACK))
    );
    let change = runtime
        .define_variant(
            "dark",
            [(background, ThemeValue::Color(Color::TRANSPARENT))],
        )
        .unwrap();
    assert!(!change.is_empty());
    assert_eq!(
        runtime.value(background),
        Some(ThemeValue::Color(Color::TRANSPARENT))
    );
    assert!(!runtime.clear_variant().is_empty());
    assert!(runtime.clear_variant().is_empty());
    assert_eq!(
        runtime.value(background),
        Some(ThemeValue::Color(Color::WHITE))
    );
}

#[test]
fn theme_errors_explain_every_public_failure_class() {
    let errors = [
        ThemeError::DuplicateToken("a".into()),
        ThemeError::UnknownToken("b".into()),
        ThemeError::UnknownVariant("dark".into()),
        ThemeError::InvalidValue("size".into()),
        ThemeError::TypeMismatch {
            token: "size".into(),
            expected: ThemeValueType::Length,
            actual: ThemeValueType::Bool,
        },
        ThemeError::ReferenceCycle(vec!["a".into(), "b".into(), "a".into()]),
    ];
    for error in errors {
        assert!(!error.to_string().is_empty());
    }
}
