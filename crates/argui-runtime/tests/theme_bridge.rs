use std::sync::Arc;

use argui_core::{Color, ColorScheme};
use argui_platform::WindowKey;
use argui_runtime::{
    AppModel, Context, Render, SingleWindowModel, ThemeBridge, ThemeRuntime, ThemeSchema,
    ThemeTokenDefinition, ThemeValue, WindowEnvironment,
};
use argui_ui::Element;
use serde_json::{Value, json};

const SCHEMA: &str = r##"{
  "tokens": {
    "foreground": {"type":"Color","default":"#ffffff","impact":"Paint"},
    "spacing": {"type":"Length","default":8,"impact":"Layout"}
  },
  "variants": {
    "day": {"foreground":"#eeeeee"},
    "night": {"foreground":"#111111"}
  },
  "initialVariant":"system",
  "systemVariants":{"light":"day","dark":"night"}
}"##;

/// Parses a bridge JSON result for assertions and returns its value tree.
fn parsed(json: &str) -> Value {
    serde_json::from_str(json).unwrap()
}

/// A patch changes multiple resolved tokens and emits one coherent revision.
#[test]
fn bridge_commits_one_coherent_revision_for_many_token_edits() {
    let mut bridge = ThemeBridge::default();
    let created = parsed(&bridge.create(SCHEMA).unwrap());
    let id = created["id"].as_u64().unwrap();
    assert_eq!(created["snapshot"]["values"]["foreground"], "#eeeeee");
    assert_eq!(created["snapshot"]["resolvedVariant"], "day");
    let revision = created["snapshot"]["revision"].as_u64().unwrap();
    let receiver = bridge.subscribe(id).unwrap();
    let next = parsed(
        &bridge
            .update(
                id,
                &json!({
                    "systemScheme":"dark",
                    "overrides":{"foreground":"#123456","spacing":12}
                })
                .to_string(),
            )
            .unwrap(),
    );
    assert_eq!(next["revision"], revision + 1);
    assert_eq!(next["systemScheme"], "dark");
    assert_eq!(next["resolvedVariant"], "night");
    assert_eq!(next["values"]["foreground"], "#123456");
    assert_eq!(next["values"]["spacing"], 12.0);
    assert_eq!(next["change"]["tokens"], json!(["foreground", "spacing"]));
    assert_eq!(next["change"]["impact"], "Layout");
    let delivered = receiver.try_recv().unwrap();
    assert_eq!(delivered.revision(), revision + 1);
    assert_eq!(delivered.tokens().len(), 2);
    assert!(receiver.try_recv().is_err());
}

/// Invalid input rolls back and a selection-only update advances metadata once.
#[test]
fn bridge_validates_before_mutation_and_tracks_metadata_only_changes() {
    let mut bridge = ThemeBridge::default();
    assert!(
        bridge
            .create(
                &json!({
                    "tokens": {"ink": {"type": "Color", "default": "#000000"}},
                    "systemVariants": {"light": "missing", "dark": "missing"}
                })
                .to_string(),
            )
            .unwrap_err()
            .contains("unknown system theme variant")
    );
    let id = parsed(&bridge.create(SCHEMA).unwrap())["id"]
        .as_u64()
        .unwrap();
    let before = parsed(&bridge.snapshot(id, None).unwrap());
    let bad = bridge.update(
        id,
        &json!({"overrides":{"spacing":14,"unknown":"#000000"}}).to_string(),
    );
    assert!(bad.unwrap_err().contains("unknown theme token"));
    assert_eq!(parsed(&bridge.snapshot(id, None).unwrap()), before);

    let metadata = parsed(&bridge.update(id, r#"{"variant":"day"}"#).unwrap());
    assert_eq!(metadata["variant"], "day");
    assert_eq!(metadata["change"]["tokens"], json!([]));
    assert!(metadata["revision"].as_u64().unwrap() > before["revision"].as_u64().unwrap());
    assert!(bridge.dispose(id));
    assert!(!bridge.dispose(id));
    assert!(bridge.snapshot(id, None).is_err());
}

#[test]
fn theme_color_tokens_accept_hex_oklch_and_rgba_without_losing_alpha() {
    let mut bridge = ThemeBridge::default();
    let definition = json!({
        "tokens": {"surface": {"type": "Color", "default": "#ffffff"}},
        "variants": {"light": {"surface": "oklch(1 0 0 / 90%)"}},
        "initialVariant": "light"
    });
    let created = parsed(&bridge.create(&definition.to_string()).unwrap());
    let id = created["id"].as_u64().unwrap();
    assert_eq!(created["snapshot"]["values"]["surface"], "#ffffffe6");
    for literal in ["rgba(255, 0, 128, 0.5)", "rgb(255 0 128 / 50%)"] {
        let next = parsed(
            &bridge
                .update(id, &json!({"overrides":{"surface":literal}}).to_string())
                .unwrap(),
        );
        assert_eq!(next["values"]["surface"], "#ff008080");
    }
    let before = parsed(&bridge.snapshot(id, None).unwrap());
    assert!(
        bridge
            .update(
                id,
                &json!({"overrides":{"surface":"rgb(300 0 0)"}}).to_string()
            )
            .is_err()
    );
    assert_eq!(parsed(&bridge.snapshot(id, None).unwrap()), before);
}

/// Separate themes do not share state, while one Rust app view receives its snapshot.
#[test]
fn one_rust_model_reads_its_window_theme_without_cross_window_state() {
    struct Themed;
    impl Render for Themed {
        fn render(&mut self, context: &mut Context<Self>) -> Element {
            let environment = context.environment();
            let foreground = environment
                .theme
                .as_ref()
                .and_then(|theme| theme.values().first())
                .cloned();
            Element::text(format!("{foreground:?}"))
        }
    }

    let schema = Arc::new(
        ThemeSchema::new([ThemeTokenDefinition::new(
            "foreground",
            ThemeValue::Color(Color::WHITE),
        )])
        .unwrap(),
    );
    let first = ThemeRuntime::new(Arc::clone(&schema));
    let second = ThemeRuntime::new(schema);
    let model = SingleWindowModel::new(Themed).with_theme(first.clone());
    let main = WindowKey::main();
    assert!(model.theme(&main).is_some());
    assert!(model.theme(&WindowKey::new("other")).is_none());
    first.set_system_scheme(ColorScheme::Dark);
    let id = first.schema().token("foreground").unwrap();
    first
        .set_override(id, ThemeValue::Color(Color::BLACK))
        .unwrap();
    assert_eq!(second.value(id), Some(ThemeValue::Color(Color::WHITE)));
    let root = model.view(
        &main,
        WindowEnvironment {
            theme: Some(first.snapshot()),
            ..WindowEnvironment::default()
        },
    );
    assert!(root.is_some());
    assert_eq!(model.theme(&main).unwrap().value(id), first.value(id));
}

/// The bridge applies a detected scheme before mount and updates every live session.
#[test]
fn detected_scheme_applies_before_first_session_and_updates_live_sessions() {
    let mut bridge = ThemeBridge::default();
    assert!(
        bridge
            .set_system_scheme(ColorScheme::Dark)
            .unwrap()
            .is_empty()
    );
    let first = parsed(&bridge.create(SCHEMA).unwrap());
    let first_id = first["id"].as_u64().unwrap();
    assert_eq!(first["snapshot"]["systemScheme"], "dark");
    assert_eq!(first["snapshot"]["resolvedVariant"], "night");
    assert_eq!(first["snapshot"]["values"]["foreground"], "#111111");
    let second_id = parsed(&bridge.create(SCHEMA).unwrap())["id"]
        .as_u64()
        .unwrap();
    let updates = bridge.set_system_scheme(ColorScheme::Light).unwrap();
    assert_eq!(updates.len(), 2);
    assert_eq!(updates[0].0, first_id);
    assert_eq!(updates[1].0, second_id);
    for (_, snapshot) in &updates {
        let snapshot = parsed(snapshot);
        assert_eq!(snapshot["resolvedVariant"], "day");
        assert_eq!(snapshot["change"]["tokens"], json!(["foreground"]));
    }
    assert!(
        bridge
            .set_system_scheme(ColorScheme::Light)
            .unwrap()
            .is_empty()
    );
}

/// Every supported scalar type round-trips through one atomic host revision.
#[test]
fn scalar_values_round_trip_through_the_typed_engine() {
    let mut bridge = ThemeBridge::default();
    let definition = json!({
        "tokens": {
            "color": {"type":"Color","default":"#10203080","impact":"Semantics"},
            "brush": {"type":"Brush","default":"#ffffff","impact":"Composite"},
            "float": {"type":"Float","default":1.5,"impact":"Paint"},
            "int": {"type":"Int","default":3,"impact":"Scroll"},
            "bool": {"type":"Bool","default":true,"impact":"Layout"},
            "length": {"type":"Length","default":4},
            "percentage": {"type":"Percentage","default":0.5},
            "duration": {"type":"Duration","default":120},
            "angle": {"type":"Angle","default":0.25},
            "fontFamily": {"type":"FontFamily","default":"Inter"},
            "fontWeight": {"type":"FontWeight","default":500},
            "fontSize": {"type":"FontSize","default":16},
            "lineHeight": {"type":"LineHeight","default":1.2}
        }
    });
    let created = parsed(&bridge.create(&definition.to_string()).unwrap());
    let id = created["id"].as_u64().unwrap();
    let values = &created["snapshot"]["values"];
    assert_eq!(values["color"], "#10203080");
    assert_eq!(values["brush"], "#ffffff");
    assert_eq!(values["fontFamily"], "Inter");
    assert_eq!(values["fontWeight"], 500);
    assert_eq!(values["bool"], true);
    let revision = created["snapshot"]["revision"].as_u64().unwrap();
    let updated = parsed(
        &bridge
            .update(
                id,
                &json!({"overrides": {
                    "color":"#abcdef", "brush":"#123456", "float":2.25,
                    "int":8, "bool":false, "length":6, "percentage":0.75,
                    "duration":250, "angle":0.5, "fontFamily":"Atkinson",
                    "fontWeight":600, "fontSize":18, "lineHeight":1.5
                }})
                .to_string(),
            )
            .unwrap(),
    );
    assert_eq!(updated["revision"], revision + 1);
    assert_eq!(updated["change"]["impact"], "Layout");
    assert_eq!(updated["change"]["tokens"].as_array().unwrap().len(), 13);
    assert_eq!(updated["values"]["color"], "#abcdef");
    assert_eq!(updated["values"]["brush"], "#123456");
    assert_eq!(updated["values"]["int"], 8);
    assert_eq!(updated["values"]["duration"], 250.0);
    assert_eq!(updated["values"]["lineHeight"], 1.5);
}

/// Malformed definitions and patches leave live sessions coherent and usable.
#[test]
fn invalid_wire_values_roll_back_and_valid_updates_still_apply() {
    let mut bridge = ThemeBridge::default();
    for (definition, expected) in [
        ("{", "eof"),
        (r#"{"tokens":{"ink":0}}"#, "must be an object"),
        (r##"{"tokens":{"ink":{"default":"#fff"}}}"##, "needs a type"),
        (r#"{"tokens":{"ink":{"type":"Color"}}}"#, "needs a default"),
        (
            r#"{"tokens":{"ink":{"type":"Unknown","default":1}}}"#,
            "unsupported",
        ),
        (
            r#"{"tokens":{"ink":{"type":"Color","default":"oops"}}}"#,
            "hex",
        ),
        (
            r##"{"tokens":{"ink":{"type":"Color","default":"#fff","impact":"Wrong"}}}"##,
            "impact",
        ),
        (
            r##"{"tokens":{"ink":{"type":"Color","default":"#fff","impact":5}}}"##,
            "invalid impact",
        ),
        (
            r##"{"tokens":{"ink":{"type":5,"default":"#fff"}}}"##,
            "needs a type",
        ),
        (
            r#"{"tokens":{"fill":{"type":"Brush","default":"not a color"}}}"#,
            "hex",
        ),
        (
            r##"{"tokens":{"ink":{"type":"Color","default":"#fff","extra":1}}}"##,
            "unknown field",
        ),
        (
            r#"{"tokens":{"weight":{"type":"FontWeight","default":0}}}"#,
            "invalid value",
        ),
        (
            r##"{"tokens":{"ink":{"type":"Color","default":"#fff"}},"variants":{"dark":0}}"##,
            "must be an object",
        ),
        (
            r##"{"tokens":{"ink":{"type":"Color","default":"#fff"}},"variants":{"dark":{"other":"#000"}}}"##,
            "unknown theme token",
        ),
        (
            r##"{"tokens":{"ink":{"type":"Color","default":"#fff"}},"initialVariant":"unknown"}"##,
            "unknown theme variant",
        ),
        (
            r#"{"tokens":{"size":{"type":"FontSize","default":16}},"variants":{"bad":{"size":-1}}}"#,
            "invalid value",
        ),
    ] {
        let error = bridge.create(definition).unwrap_err();
        assert!(
            error.to_lowercase().contains(expected),
            "{definition}: {error}"
        );
    }
    let schema = json!({
        "tokens": {
            "ink": {"type":"Color","default":"#ffffff"},
            "spacing": {"type":"Length","default":8}
        },
        "variants":{"night":{"ink":"#111111"}},
        "initialVariant":"night"
    });
    let id = parsed(&bridge.create(&schema.to_string()).unwrap())["id"]
        .as_u64()
        .unwrap();
    let before = parsed(&bridge.snapshot(id, None).unwrap());
    for (patch, expected) in [
        ("{", "eof"),
        (r#"{"variant":"unknown"}"#, "unknown theme variant"),
        (r#"{"systemScheme":"sepia"}"#, "unknown system"),
        (r#"{"overrides":{"spacing":"wide"}}"#, "finite number"),
        (r#"{"overrides":{"ink":"oops"}}"#, "hex"),
        (r#"{"removeOverrides":["missing"]}"#, "unknown theme token"),
    ] {
        let error = bridge.update(id, patch).unwrap_err();
        assert!(error.to_lowercase().contains(expected), "{patch}: {error}");
        assert_eq!(parsed(&bridge.snapshot(id, None).unwrap()), before);
    }
    assert!(
        bridge
            .update(999, "{}")
            .unwrap_err()
            .contains("unknown theme session")
    );
    assert!(bridge.subscribe(999).is_err());
    assert!(bridge.runtime(999).is_none());
    assert!(bridge.runtime(id).is_some());

    let defaults = parsed(&bridge.update(id, r#"{"variant":null}"#).unwrap());
    assert_eq!(defaults["variant"], Value::Null);
    assert_eq!(defaults["values"]["ink"], "#ffffff");
    let explicit = parsed(&bridge.update(id, r#"{"variant":"night"}"#).unwrap());
    assert_eq!(explicit["resolvedVariant"], "night");
    let overridden = parsed(
        &bridge
            .update(id, r#"{"overrides":{"spacing":20}}"#)
            .unwrap(),
    );
    assert_eq!(overridden["values"]["spacing"], 20.0);
    let restored = parsed(
        &bridge
            .update(id, r#"{"removeOverrides":["spacing"]}"#)
            .unwrap(),
    );
    assert_eq!(restored["values"]["spacing"], 8.0);
}

/// A failed typed override never changes variant selection or published values.
#[test]
fn invalid_typed_override_does_not_commit_a_mixed_patch() {
    let mut bridge = ThemeBridge::default();
    let definition = json!({
        "tokens":{"weight":{"type":"FontWeight","default":500}},
        "variants":{"heavy":{"weight":700}}
    });
    let id = parsed(&bridge.create(&definition.to_string()).unwrap())["id"]
        .as_u64()
        .unwrap();
    let before = parsed(&bridge.snapshot(id, None).unwrap());
    let error = bridge
        .update(id, r#"{"variant":"heavy","overrides":{"weight":0}}"#)
        .unwrap_err();
    assert!(error.contains("invalid value"), "{error}");
    assert_eq!(parsed(&bridge.snapshot(id, None).unwrap()), before);
    let good = parsed(
        &bridge
            .update(id, r#"{"variant":"heavy","overrides":{"weight":800}}"#)
            .unwrap(),
    );
    assert_eq!(good["variant"], "heavy");
    assert_eq!(good["values"]["weight"], 800);
}

/// Rust-side theme mutations remain observable through the host's JSON snapshot.
#[test]
fn external_runtime_change_has_the_same_revision_as_bridge_subscription() {
    let mut bridge = ThemeBridge::default();
    let definition = json!({"tokens":{"enabled":{"type":"Bool","default":false}}});
    let id = parsed(&bridge.create(&definition.to_string()).unwrap())["id"]
        .as_u64()
        .unwrap();
    let receiver = bridge.subscribe(id).unwrap();
    let runtime = bridge.runtime(id).unwrap();
    let token = runtime.schema().token("enabled").unwrap();
    let change = runtime
        .set_override(token, argui_runtime::ThemeValue::Bool(true))
        .unwrap();
    assert_eq!(receiver.try_recv().unwrap(), change);
    let published = parsed(&bridge.snapshot(id, Some(&change)).unwrap());
    assert_eq!(published["revision"], change.revision());
    assert_eq!(published["values"]["enabled"], true);
    assert_eq!(published["change"]["tokens"], json!(["enabled"]));
    assert_eq!(published["change"]["impact"], "Paint");
}
