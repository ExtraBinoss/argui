use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use argui_core::{Color, ColorScheme};
use argui_theme::{
    ThemeError, ThemeImpact, ThemeMode, ThemeRuntime, ThemeSchema, ThemeSelection,
    ThemeTokenDefinition, ThemeValue,
};

fn runtime() -> ThemeRuntime {
    let schema = ThemeSchema::new([
        ThemeTokenDefinition::new("surface", ThemeValue::Color(Color::WHITE))
            .impact(ThemeImpact::Paint),
        ThemeTokenDefinition::new("space", ThemeValue::Length(8.0)).impact(ThemeImpact::Layout),
    ])
    .unwrap();
    ThemeRuntime::new(Arc::new(schema))
}

#[test]
fn batch_commits_once_and_rejected_edit_rolls_back_every_layer() {
    let runtime = runtime();
    let surface = runtime.schema().token("surface").unwrap();
    let space = runtime.schema().token("space").unwrap();
    let receiver = runtime.subscribe();
    let change = runtime
        .update(|draft| {
            draft.set_override(surface, ThemeValue::Color(Color::BLACK))?;
            draft.set_override(space, ThemeValue::Length(12.0))?;
            Ok(())
        })
        .unwrap();
    assert_eq!(change.tokens(), [surface, space]);
    assert_eq!(change.impact(), Some(ThemeImpact::Layout));
    assert_eq!(change.revision(), 1);
    assert_eq!(receiver.try_recv().unwrap(), change);
    assert!(receiver.try_recv().is_err());
    let snapshot = runtime.snapshot();
    assert_eq!(
        snapshot.value(surface),
        Some(&ThemeValue::Color(Color::BLACK))
    );
    assert_eq!(snapshot.value(space), Some(&ThemeValue::Length(12.0)));
    assert_eq!(snapshot.token_revision(surface), Some(1));
    assert_eq!(snapshot.token_revision(space), Some(1));

    assert!(matches!(
        runtime.update(|draft| {
            draft.set_override(surface, ThemeValue::Color(Color::WHITE))?;
            draft.set_override(space, ThemeValue::Bool(true))?;
            Ok(())
        }),
        Err(ThemeError::TypeMismatch { .. })
    ));
    assert_eq!(runtime.snapshot(), snapshot);
    assert!(receiver.try_recv().is_err());
}

#[test]
fn system_and_custom_variants_follow_preference_and_override_priority() {
    let runtime = runtime();
    let surface = runtime.schema().token("surface").unwrap();
    runtime
        .update(|draft| {
            draft.define_variant("day", [(surface, ThemeValue::Color(Color::WHITE))])?;
            draft.define_variant("night", [(surface, ThemeValue::Color(Color::BLACK))])?;
            draft.set_system_variants("day", "night");
            Ok(())
        })
        .unwrap();
    assert_eq!(runtime.snapshot().resolved_variant(), Some("day"));
    assert_eq!(runtime.snapshot().mode(), Some(ThemeMode::System));
    let dark = runtime.set_system_scheme(ColorScheme::Dark);
    assert_eq!(dark.tokens(), [surface]);
    assert_eq!(runtime.snapshot().resolved_variant(), Some("night"));
    assert_eq!(
        runtime.value(surface),
        Some(ThemeValue::Color(Color::BLACK))
    );
    runtime
        .set_override(surface, ThemeValue::Color(Color::TRANSPARENT))
        .unwrap();
    let no_value_change = runtime.set_mode(ThemeMode::Light);
    assert!(no_value_change.is_empty());
    assert_eq!(runtime.snapshot().active_variant(), Some("day"));
    assert_eq!(runtime.snapshot().mode(), Some(ThemeMode::Light));
    assert_eq!(
        runtime.value(surface),
        Some(ThemeValue::Color(Color::TRANSPARENT))
    );
    runtime.remove_override(surface);
    assert_eq!(
        runtime.value(surface),
        Some(ThemeValue::Color(Color::WHITE))
    );
    runtime.activate("night").unwrap();
    assert_eq!(runtime.selection(), ThemeSelection::Variant("night".into()));
    assert_eq!(runtime.snapshot().resolved_variant(), Some("night"));
    runtime.clear_variant();
    assert_eq!(runtime.selection(), ThemeSelection::Defaults);
    assert_eq!(runtime.snapshot().resolved_variant(), None);
}

#[test]
fn metadata_revision_notifies_global_observers_without_token_invalidation() {
    let runtime = runtime();
    let surface = runtime.schema().token("surface").unwrap();
    let global = runtime.subscribe();
    let targeted = runtime.subscribe_tokens(&[surface]).unwrap();
    let start = runtime.revision();
    let change = runtime.set_system_scheme(ColorScheme::Dark);
    assert!(change.is_empty());
    assert_eq!(change.revision(), start + 1);
    assert_eq!(global.try_recv().unwrap(), change);
    assert!(targeted.try_recv().is_err());
    assert_eq!(runtime.snapshot().system_scheme(), ColorScheme::Dark);
    let no_op = runtime.set_system_scheme(ColorScheme::Dark);
    assert!(no_op.is_empty());
    assert_eq!(no_op.revision(), change.revision());
    assert!(global.try_recv().is_err());
}

#[test]
fn callback_is_post_commit_and_raii_scoped_while_fork_is_independent() {
    let runtime = runtime();
    let surface = runtime.schema().token("surface").unwrap();
    let reader = runtime.clone();
    let calls = Arc::new(AtomicUsize::new(0));
    let seen = Arc::clone(&calls);
    let subscription = runtime.watch(move |change| {
        assert_eq!(reader.snapshot().revision(), change.revision());
        seen.fetch_add(1, Ordering::SeqCst);
    });
    runtime
        .set_override(surface, ThemeValue::Color(Color::BLACK))
        .unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let fork = runtime.fork();
    fork.set_override(surface, ThemeValue::Color(Color::TRANSPARENT))
        .unwrap();
    assert_eq!(
        runtime.value(surface),
        Some(ThemeValue::Color(Color::BLACK))
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    drop(subscription);
    runtime.remove_override(surface);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn override_sets_and_replacements_validate_before_publishing() {
    let runtime = runtime();
    let surface = runtime.schema().token("surface").unwrap();
    let space = runtime.schema().token("space").unwrap();
    let initial = runtime.snapshot();
    assert!(matches!(
        runtime.set_overrides([
            (surface, ThemeValue::Color(Color::BLACK)),
            (space, ThemeValue::Bool(true)),
        ]),
        Err(ThemeError::TypeMismatch { .. })
    ));
    assert_eq!(runtime.snapshot(), initial);
    assert_eq!(
        runtime
            .set_overrides([
                (surface, ThemeValue::Color(Color::BLACK)),
                (space, ThemeValue::Length(16.0)),
            ])
            .unwrap()
            .tokens(),
        [surface, space]
    );
    assert_eq!(
        runtime
            .replace_overrides([(space, ThemeValue::Length(20.0))])
            .unwrap()
            .tokens(),
        [surface, space]
    );
    assert_eq!(
        runtime.value(surface),
        Some(ThemeValue::Color(Color::WHITE))
    );
    assert_eq!(runtime.value(space), Some(ThemeValue::Length(20.0)));
    assert!(matches!(
        runtime.replace_overrides([(space, ThemeValue::Length(f32::NAN))]),
        Err(ThemeError::InvalidValue(_))
    ));
    assert_eq!(runtime.value(space), Some(ThemeValue::Length(20.0)));
    assert_eq!(runtime.replace_overrides([]).unwrap().tokens(), [space]);
    assert!(runtime.replace_overrides([]).unwrap().is_empty());
}

#[test]
fn invalid_subscriptions_and_variants_leave_existing_state_intact() {
    let runtime = runtime();
    let surface = runtime.schema().token("surface").unwrap();
    let larger = ThemeSchema::new([
        ThemeTokenDefinition::new("a", ThemeValue::Bool(true)),
        ThemeTokenDefinition::new("b", ThemeValue::Bool(true)),
        ThemeTokenDefinition::new("c", ThemeValue::Bool(true)),
    ])
    .unwrap();
    let foreign = larger.token("c").unwrap();
    assert!(matches!(
        runtime.subscribe_tokens(&[foreign]),
        Err(ThemeError::UnknownToken(_))
    ));
    assert!(matches!(
        runtime.activate("missing"),
        Err(ThemeError::UnknownVariant(_))
    ));
    assert!(matches!(
        runtime.define_variant("bad", [(surface, ThemeValue::Length(1.0))]),
        Err(ThemeError::TypeMismatch { .. })
    ));
    assert_eq!(runtime.snapshot().resolved_variant(), None);
    let before = runtime.revision();
    runtime
        .define_variant("custom", [(surface, ThemeValue::Color(Color::BLACK))])
        .unwrap();
    assert!(runtime.revision() > before);
    assert!(
        runtime
            .activate("custom")
            .unwrap()
            .tokens()
            .contains(&surface)
    );
    assert_eq!(runtime.mode(), None);
    assert_eq!(runtime.snapshot().active_variant(), Some("custom"));
    assert_eq!(runtime.snapshot().values().len(), 2);
    assert_eq!(runtime.system_scheme(), ColorScheme::Light);
    assert!(runtime.clear_variant().tokens().contains(&surface));
    assert!(runtime.clear_variant().is_empty());
}

#[test]
fn metadata_watchers_receive_only_real_changes() {
    let runtime = runtime();
    let seen = Arc::new(AtomicUsize::new(0));
    let calls = Arc::clone(&seen);
    let subscription = runtime.watch(move |change| {
        assert!(change.is_empty());
        calls.fetch_add(1, Ordering::SeqCst);
    });
    runtime.set_system_scheme(ColorScheme::Dark);
    runtime.set_system_scheme(ColorScheme::Dark);
    runtime.set_system_variants("sun", "moon");
    assert_eq!(seen.load(Ordering::SeqCst), 2);
    runtime.set_mode(ThemeMode::Light);
    runtime.set_system_variants("dawn", "dusk");
    assert_eq!(runtime.snapshot().active_variant(), Some("dawn"));
    assert_eq!(runtime.snapshot().mode(), Some(ThemeMode::Light));
    assert_eq!(seen.load(Ordering::SeqCst), 4);
    drop(subscription);
}
