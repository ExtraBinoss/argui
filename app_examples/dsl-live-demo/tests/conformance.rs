//! The same authored program is compiled by rustc and exercised in both backends.

include!(concat!(env!("OUT_DIR"), "/conformance.rs"));

use argui::ui::Role;
use argui_testing::TestApp;
use std::{cell::Cell, rc::Rc};
/// AOT handlers preserve short-circuiting, widening and self-referential updates.
#[test]
fn aot_language_conformance() {
    let component = Conformance::new();
    assert_eq!(component.ratio(), 1.0);
    assert_eq!(component.optional(), Some(2.0));
    assert_eq!(component.conditional(), 1.0);
    let calls = Rc::new(Cell::new(0));
    let observed = Rc::clone(&calls);
    component.on_probe(move || {
        observed.set(observed.get() + 1);
        true
    });
    let mut app = TestApp::new(component);
    app.assert_text("2:1:start");
    app.assert_text("16777216:true");
    app.get_by_role(Role::Button, "Run").click().unwrap();
    app.assert_text("4:5:start!");
    assert_eq!(calls.get(), 0);
}

/// Reusing row keys in different outer rows preserves independent retained state.
#[test]
fn aot_nested_keys_are_scoped_and_survive_reordering() {
    exercise_nested_keys(TestApp::new(KeyedRows::new()));
}

/// Exercises a rendered `app` and requires the same state after an outer reorder.
fn exercise_nested_keys<R: argui::runtime::Render + 'static>(mut app: TestApp<R>) {
    app.get_by_role(Role::Button, "11:0").click().unwrap();
    app.assert_text("11:1");
    app.assert_text("21:0");
    app.get_by_role(Role::Button, "Reverse").click().unwrap();
    app.assert_text("11:1");
    app.assert_text("21:0");
    app.get_by_role(Role::Button, "21:0").click().unwrap();
    app.assert_text("21:1");
    app.assert_text("11:1");
}

/// Live nested key ownership obeys the same contract as compiled AOT.
#[cfg(feature = "argui-live")]
#[test]
fn live_nested_keys_are_scoped_and_survive_reordering() {
    use argui_dsl_compiler::{Compiler, SourceModule};
    use argui_dsl_runtime::{LivePackage, LiveRuntime};
    use std::collections::HashMap;
    let compiled = Compiler::compile(
        [SourceModule::new(
            "tests/fixtures/language.argui",
            include_str!("fixtures/language.argui"),
        )],
        "tests/fixtures/language.argui",
        |_| Err("fixture has no assets".into()),
    )
    .unwrap();
    let id = compiled
        .semantic
        .modules
        .iter()
        .flat_map(|module| &module.definitions)
        .find(|definition| definition.name == "KeyedRows")
        .unwrap()
        .id
        .raw();
    let component = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id.raw() == id)
        .unwrap()
        .id;
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    runtime.mount(component, []).unwrap();
    exercise_nested_keys(TestApp::new(runtime));
}

/// Live runs the exact AOT fixture and observes the same values and side effects.
#[cfg(feature = "argui-live")]
#[test]
fn live_language_conformance() {
    use argui_dsl_compiler::{Compiler, SourceModule};
    use argui_dsl_runtime::{DslValue, LivePackage, LiveRuntime};
    use std::collections::HashMap;
    let compiled = Compiler::compile(
        [SourceModule::new(
            "tests/fixtures/language.argui",
            include_str!("fixtures/language.argui"),
        )],
        "tests/fixtures/language.argui",
        |_| Err("fixture has no assets".into()),
    )
    .unwrap();
    let root = compiled.roots[0];
    let definition = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id == root)
        .unwrap();
    let callback = definition.callbacks[0].id;
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    let instance = runtime.mount(root, []).unwrap();
    let calls = Rc::new(Cell::new(0));
    let observed = Rc::clone(&calls);
    runtime
        .bind_callback(instance, callback, move |_| {
            observed.set(observed.get() + 1);
            DslValue::Bool(true)
        })
        .unwrap();
    let mut app = TestApp::new(runtime);
    app.assert_text("2:1:start");
    app.assert_text("16777216:true");
    app.get_by_role(Role::Button, "Run").click().unwrap();
    app.assert_text("4:5:start!");
    assert_eq!(calls.get(), 0);
}

/// Compiled styles supply values, respect inline overrides, and observe pointer state.
#[test]
fn aot_styles_are_applied() {
    exercise_styles(TestApp::new(Styled::new()));
}

/// AOT style observations are isolated by the current row's retained key.
#[test]
fn aot_repeated_styles_are_independent() {
    exercise_row_styles(TestApp::new(StyledRows::new()));
}

/// Live repeats the same style without leaking hover to neighboring rows.
#[cfg(feature = "argui-live")]
#[test]
fn live_repeated_styles_are_independent() {
    exercise_row_styles(TestApp::new(live_fixture("StyledRows")));
}

/// Keyed AOT rows retain lexical access to an outer native observation.
#[test]
fn aot_rows_observe_the_enclosing_instance() {
    exercise_outer_observation(TestApp::new(OuterObservation::new()));
}

/// Live resolves the same outer native instead of synthesizing a row-local site.
#[cfg(feature = "argui-live")]
#[test]
fn live_rows_observe_the_enclosing_instance() {
    exercise_outer_observation(TestApp::new(live_fixture("OuterObservation")));
}

/// Hovers the outer touch region of `app` and verifies both repeated bindings.
fn exercise_outer_observation<R: argui::runtime::Render>(mut app: TestApp<R>) {
    app.assert_text("outside:1");
    let bounds = app.bounds("outer").unwrap();
    app.pointer_move(argui::core::Point::new(
        bounds.origin.x + 2.0,
        bounds.origin.y + 2.0,
    ))
    .unwrap();
    app.assert_text("inside:1");
    app.assert_text("inside:2");
}

/// Moves the pointer between rows in `app` and checks each independent style.
fn exercise_row_styles<R: argui::runtime::Render>(mut app: TestApp<R>) {
    for (hover, rest) in [("1", "2"), ("2", "1")] {
        let bounds = app.bounds(argui_testing::Selector::text(hover)).unwrap();
        app.pointer_move(argui::core::Point::new(
            bounds.origin.x + 2.0,
            bounds.origin.y + 2.0,
        ))
        .unwrap();
        app.assert_text("hovered");
        app.assert_text(rest);
    }
}

/// Requires shared `app` styles to produce visible base, inline, and hover text.
fn exercise_styles<R: argui::runtime::Render + 'static>(mut app: TestApp<R>) {
    app.assert_text("styled");
    app.assert_text("inline");
    app.assert_text("rest");
    let bounds = app.bounds(argui_testing::Selector::text("rest")).unwrap();
    app.pointer_move(argui::core::Point::new(
        bounds.origin.x + 2.0,
        bounds.origin.y + 2.0,
    ))
    .unwrap();
    app.assert_text("hovered");
}

/// Live consumes the same expanded style bindings and interaction predicates.
#[cfg(feature = "argui-live")]
#[test]
fn live_styles_are_applied() {
    use argui_dsl_compiler::{Compiler, SourceModule};
    use argui_dsl_runtime::{LivePackage, LiveRuntime};
    use std::collections::HashMap;
    let compiled = Compiler::compile(
        [SourceModule::new(
            "tests/fixtures/language.argui",
            include_str!("fixtures/language.argui"),
        )],
        "tests/fixtures/language.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    let symbol = compiled
        .semantic
        .modules
        .iter()
        .flat_map(|module| &module.definitions)
        .find(|definition| definition.name == "Styled")
        .unwrap()
        .id
        .raw();
    let id = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id.raw() == symbol)
        .unwrap()
        .id;
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    runtime.mount(id, []).unwrap();
    exercise_styles(TestApp::new(runtime));
}

/// Style values, inline bindings, authored states, and theme tokens have stable precedence in AOT.
#[test]
fn aot_style_state_inline_and_theme_precedence() {
    exercise_style_cascade(TestApp::new(StyleCascade::new()));
}

/// The live backend resolves the same style precedence and theme token updates as AOT.
#[cfg(feature = "argui-live")]
#[test]
fn live_style_state_inline_and_theme_precedence() {
    exercise_style_cascade(TestApp::new(live_fixture("StyleCascade")));
}

/// Checks style base, style hover, inline, authored state, and theme mode in `app`.
fn exercise_style_cascade<R: argui::runtime::Render + 'static>(mut app: TestApp<R>) {
    use argui::core::Point;
    app.assert_text("base-light");
    app.assert_text("inline");

    let bounds = app.bounds("token").unwrap();
    app.pointer_move(Point::new(bounds.origin.x + 2.0, bounds.origin.y + 2.0))
        .unwrap();
    app.assert_text("hover-light");

    let bounds = app.bounds("layered").unwrap();
    app.pointer_move(Point::new(bounds.origin.x + 2.0, bounds.origin.y + 2.0))
        .unwrap();
    app.assert_text("hover-light");
    app.assert_no_text("inline");
    app.get_by_role(Role::Button, "Choose state")
        .click()
        .unwrap();
    app.assert_text("authored");
    let bounds = app.bounds("layered").unwrap();
    app.pointer_move(Point::new(bounds.origin.x + 2.0, bounds.origin.y + 2.0))
        .unwrap();
    app.assert_text("authored");
    app.assert_no_text("hover-light");

    app.get_by_role(Role::Button, "Switch theme")
        .click()
        .unwrap();
    app.assert_text("base-dark");
    app.assert_no_text("base-light");
    app.assert_text("authored");
    let bounds = app.bounds("token").unwrap();
    app.pointer_move(Point::new(bounds.origin.x + 2.0, bounds.origin.y + 2.0))
        .unwrap();
    app.assert_text("hover-dark");
    let bounds = app.bounds("layered").unwrap();
    app.pointer_move(Point::new(bounds.origin.x + 2.0, bounds.origin.y + 2.0))
        .unwrap();
    app.assert_text("authored");
    app.assert_no_text("hover-dark");
}

/// Rust-compiled slots preserve their caller's data and handlers through forwarding.
#[test]
fn aot_named_slots_forward_and_default() {
    exercise_slots(TestApp::new(NamedSlots::new()));
}

/// Requires `app` to project three slots and preserve a caller-owned mutation.
fn exercise_slots<R: argui::runtime::Render + 'static>(mut app: TestApp<R>) {
    app.assert_text("supplied header");
    app.assert_text("default header");
    app.assert_text("default actions");
    app.assert_text("body:0");
    app.get_by_role(Role::Button, "Add from slot")
        .click()
        .unwrap();
    app.assert_text("body:1");
}

/// Live named slots share the AOT content, defaults, forwarding, and handler contract.
#[cfg(feature = "argui-live")]
#[test]
fn live_named_slots_forward_and_default() {
    use argui_dsl_compiler::{Compiler, SourceModule};
    use argui_dsl_runtime::{LivePackage, LiveRuntime};
    use std::collections::HashMap;
    let compiled = Compiler::compile(
        [SourceModule::new(
            "tests/fixtures/language.argui",
            include_str!("fixtures/language.argui"),
        )],
        "tests/fixtures/language.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    let symbol = compiled
        .semantic
        .modules
        .iter()
        .flat_map(|module| &module.definitions)
        .find(|definition| definition.name == "NamedSlots")
        .unwrap()
        .id
        .raw();
    let id = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id.raw() == symbol)
        .unwrap()
        .id;
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    runtime.mount(id, []).unwrap();
    exercise_slots(TestApp::new(runtime));
}

/// Nonvirtual geometry must not trigger a redundant full AOT tree rebuild.
#[test]
fn aot_ignores_unrelated_layout_changes() {
    assert_unrelated_layout_is_idle(&mut Styled::new());
}

/// Nonvirtual live geometry obeys the same no-rebuild contract.
#[cfg(feature = "argui-live")]
#[test]
fn live_ignores_unrelated_layout_changes() {
    assert_unrelated_layout_is_idle(&mut live_fixture("Styled"));
}

/// Playback pauses and resumes in the generated backend, including later cycles.
#[test]
fn aot_playback_preserves_phase() {
    exercise_playback(TestApp::new(Playback::new()));
}

/// Playback pauses and resumes using the same live-authored specification.
#[cfg(feature = "argui-live")]
#[test]
fn live_playback_preserves_phase() {
    exercise_playback(TestApp::new(live_fixture("Playback")));
}

/// AOT keeps unrelated native subtrees shared across property mutations.
#[test]
fn aot_reuses_unchanged_subtrees() {
    exercise_retention(TestApp::new(Retained::new()));
}

/// Live preserves sharing without retaining stale event callbacks.
#[cfg(feature = "argui-live")]
#[test]
fn live_reuses_unchanged_subtrees() {
    exercise_retention(TestApp::new(live_fixture("Retained")));
}

/// Mutates `app` twice and checks both current callbacks and shared static text.
fn exercise_retention<R: argui::runtime::Render>(mut app: TestApp<R>) {
    let static_text = |app: &TestApp<R>| {
        let tree = app.rendered_tree();
        tree.node_ids()
            .iter()
            .filter_map(|id| tree.element_for(*id))
            .find(|element| element.key.as_deref() == Some("unchanged"))
            .unwrap()
            .clone()
    };
    let before = static_text(&app);
    for count in ["1", "2"] {
        app.get_by_role(Role::Button, "Increment").click().unwrap();
        app.assert_text(count);
        assert!(before.ptr_eq(&static_text(&app)));
    }
}

/// Compiles and mounts the exported `name` from the shared conformance fixture.
#[cfg(feature = "argui-live")]
fn live_fixture(name: &str) -> argui_dsl_runtime::LiveRuntime {
    use argui_dsl_compiler::{Compiler, SourceModule};
    use argui_dsl_runtime::{LivePackage, LiveRuntime};
    let compiled = Compiler::compile(
        [SourceModule::new(
            "tests/fixtures/language.argui",
            include_str!("fixtures/language.argui"),
        )],
        "tests/fixtures/language.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    let symbol = compiled
        .semantic
        .modules
        .iter()
        .flat_map(|module| &module.definitions)
        .find(|definition| definition.name == name)
        .unwrap()
        .id
        .raw();
    let id = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id.raw() == symbol)
        .unwrap()
        .id;
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, Default::default()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    runtime.mount(id, []).unwrap();
    runtime
}

/// Requires `app` to hold its value without frame requests and resume its phase.
fn exercise_playback<R: argui::runtime::Render>(mut app: TestApp<R>) {
    use argui::runtime::Render;
    use std::time::Duration;
    app.advance(Duration::from_millis(1)).unwrap();
    for _ in 0..3 {
        app.advance(Duration::from_millis(250)).unwrap();
        app.assert_text("0.25");
        app.get_by_role(Role::Button, "Pause").click().unwrap();
        assert!(!app.entity().read(Render::wants_animation_frame));
        app.advance(Duration::from_secs(20)).unwrap();
        app.assert_text("0.25");
        app.get_by_role(Role::Button, "Resume").click().unwrap();
        assert!(app.entity().read(Render::wants_animation_frame));
        app.advance(Duration::from_millis(1)).unwrap();
        app.assert_text("0.25");
        app.advance(Duration::from_millis(750)).unwrap();
        app.assert_text("0");
    }
}

/// Delivers changing bounds to `root` and checks that it requests no rebuild.
fn assert_unrelated_layout_is_idle<R: argui::runtime::Render>(root: &mut R) {
    use argui::core::{Point, Rect, Size};
    use argui::runtime::{Context, LayoutBounds, LayoutSnapshot, ViewUpdate};
    let element = root.render(&mut Context::default());
    let tree = argui::ui::UiTree::new(element);
    for height in [100.0, 110.0, 120.0] {
        let snapshot = LayoutSnapshot {
            viewport: Rect::new(Point::default(), Size::new(400.0, 300.0)),
            nodes: vec![LayoutBounds {
                node: tree.node_id_at(0).unwrap(),
                key: None,
                retained_identity: tree.element_at(0).unwrap().source_identity().cloned(),
                bounds: Rect::new(Point::default(), Size::new(200.0, height)),
            }],
        };
        let mut context = Context::default();
        root.layout_changed(&snapshot, &mut context);
        assert_eq!(
            context.view_update(),
            ViewUpdate::None,
            "ordinary geometry caused a full rebuild"
        );
    }
}

/// AOT resolves an identified child output within each repeater row.
#[test]
fn aot_row_child_outputs_are_lexically_scoped() {
    let app = TestApp::new(LexicalRows::new());
    app.assert_text("2");
    app.assert_text("4");
}

/// Live resolves the same output separately for every retained row.
#[cfg(feature = "argui-live")]
#[test]
fn live_row_child_outputs_are_lexically_scoped() {
    use argui_dsl_compiler::{Compiler, SourceModule};
    use argui_dsl_runtime::{LivePackage, LiveRuntime};
    use std::collections::HashMap;
    let compiled = Compiler::compile(
        [SourceModule::new(
            "tests/fixtures/language.argui",
            include_str!("fixtures/language.argui"),
        )],
        "tests/fixtures/language.argui",
        |_| Err("fixture has no assets".into()),
    )
    .unwrap();
    let symbol = compiled
        .semantic
        .modules
        .iter()
        .flat_map(|module| &module.definitions)
        .find(|definition| definition.name == "LexicalRows")
        .unwrap()
        .id;
    let root = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id.raw() == symbol.raw())
        .unwrap()
        .id;
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    runtime.mount(root, []).unwrap();
    let app = TestApp::new(runtime);
    app.assert_text("2");
    app.assert_text("4");
}

/// AOT reports a conditional child's absence and reappearance through an optional read.
#[test]
fn aot_conditional_child_absence_is_optional() {
    exercise_optional_child(TestApp::new(OptionalChild::new()));
}

/// Both backends expose the same presence transition for a conditional child.
fn exercise_optional_child<R: argui::runtime::Render + 'static>(mut app: TestApp<R>) {
    app.assert_text("missing");
    app.get_by_role(Role::Button, "Toggle").click().unwrap();
    app.assert_text("present");
    app.get_by_role(Role::Button, "Toggle").click().unwrap();
    app.assert_text("missing");
}

/// Live returns null for an absent conditional child and restores its output when mounted.
#[cfg(feature = "argui-live")]
#[test]
fn live_conditional_child_absence_is_optional() {
    use argui_dsl_compiler::{Compiler, SourceModule};
    use argui_dsl_runtime::{LivePackage, LiveRuntime};
    use std::collections::HashMap;
    let compiled = Compiler::compile(
        [SourceModule::new(
            "tests/fixtures/language.argui",
            include_str!("fixtures/language.argui"),
        )],
        "tests/fixtures/language.argui",
        |_| Err("fixture has no assets".into()),
    )
    .unwrap();
    let symbol = compiled
        .semantic
        .modules
        .iter()
        .flat_map(|module| &module.definitions)
        .find(|definition| definition.name == "OptionalChild")
        .unwrap()
        .id;
    let root = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id.raw() == symbol.raw())
        .unwrap()
        .id;
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    runtime.mount(root, []).unwrap();
    exercise_optional_child(TestApp::new(runtime));
}

#[path = "conformance/layout_and_data.rs"]
mod layout_and_data;
