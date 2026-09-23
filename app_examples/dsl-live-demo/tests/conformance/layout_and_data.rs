//! AOT/live layout and data conformance proofs.

use super::*;

/// Compiled grid tracks place the dashboard regions without host coordinates.
#[test]
fn aot_grid_layout_positions_dashboard() {
    exercise_grid_layout(TestApp::new(LayoutProof::new()));
}

/// Live grid tracks produce the same dashboard geometry as compiled code.
#[cfg(feature = "argui-live")]
#[test]
fn live_grid_layout_positions_dashboard() {
    exercise_grid_layout(TestApp::new(live_fixture("LayoutProof")));
}

/// Verifies grid row, column, and span placement for either backend.
fn exercise_grid_layout<R: argui::runtime::Render>(app: TestApp<R>) {
    let heading = app.bounds("heading").unwrap();
    let sidebar = app.bounds("sidebar").unwrap();
    let content = app.bounds("content").unwrap();
    assert!(heading.size.width > sidebar.size.width);
    assert!(sidebar.origin.y >= heading.origin.y + heading.size.height);
    assert!(content.origin.x >= sidebar.origin.x + sidebar.size.width);
    assert_eq!(sidebar.origin.y, content.origin.y);
}

/// AOT container queries reflow the form when its measured container changes.
#[test]
fn aot_responsive_form_reflows() {
    exercise_responsive_form(TestApp::new(ResponsiveForm::new()));
}

/// Live container queries use the same measured container width.
#[cfg(feature = "argui-live")]
#[test]
fn live_responsive_form_reflows() {
    exercise_responsive_form(TestApp::new(live_fixture("ResponsiveForm")));
}

/// Resizes `app` across the breakpoint and checks grid placement after layout.
fn exercise_responsive_form<R: argui::runtime::Render>(mut app: TestApp<R>) {
    app.resize(argui::core::Size::new(400.0, 400.0)).unwrap();
    let first = app.bounds("first").unwrap();
    let second = app.bounds("second").unwrap();
    assert!(second.origin.y >= first.origin.y + first.size.height);
    app.resize(argui::core::Size::new(800.0, 400.0)).unwrap();
    let first = app.bounds("first").unwrap();
    let second = app.bounds("second").unwrap();
    assert!(second.origin.x >= first.origin.x + first.size.width);
    assert_eq!(second.origin.y, first.origin.y);
}

/// AOT safe indexing returns optional values and never traps out of bounds.
#[test]
fn aot_safe_indexing_is_optional() {
    exercise_indexing(TestApp::new(Indexed::new()));
}

/// Live safe indexing follows the same optional boundary contract.
#[cfg(feature = "argui-live")]
#[test]
fn live_safe_indexing_is_optional() {
    exercise_indexing(TestApp::new(live_fixture("Indexed")));
}

/// Checks present, positive out-of-range, and negative collection indices.
fn exercise_indexing<R: argui::runtime::Render>(app: TestApp<R>) {
    app.assert_text("first present");
    app.assert_text("out of bounds");
    app.assert_text("negative missing");
    app.assert_text("7:99");
    app.assert_text("optionals work");
}

/// AOT handler locals and branches mutate the same component property.
#[test]
fn aot_handler_locals_and_branches() {
    exercise_handler_locals(TestApp::new(HandlerLocals::new()));
}

/// Live handler locals and branches follow the same lexical flow.
#[cfg(feature = "argui-live")]
#[test]
fn live_handler_locals_and_branches() {
    exercise_handler_locals(TestApp::new(live_fixture("HandlerLocals")));
}

/// Clicks through both handler branches and verifies the retained result.
fn exercise_handler_locals<R: argui::runtime::Render>(mut app: TestApp<R>) {
    app.assert_text("0");
    app.get_by_role(Role::Button, "Advance").click().unwrap();
    app.assert_text("1");
    app.get_by_role(Role::Button, "Advance").click().unwrap();
    app.assert_text("5");
}

/// AOT user struct construction preserves typed field values.
#[test]
fn aot_struct_values() {
    TestApp::new(StructValues::new()).assert_text("7:seven");
}

/// Live user struct construction resolves the same stable field IDs.
#[cfg(feature = "argui-live")]
#[test]
fn live_struct_values() {
    TestApp::new(live_fixture("StructValues")).assert_text("7:seven");
}

/// AOT enum values retain variant identity through a handler update.
#[test]
fn aot_enum_values() {
    exercise_enum_values(TestApp::new(EnumValues::new()));
}

/// Live enum values agree with AOT before and after the handler update.
#[cfg(feature = "argui-live")]
#[test]
fn live_enum_values() {
    exercise_enum_values(TestApp::new(live_fixture("EnumValues")));
}

/// Exercises initial and changed enum variant values in either backend.
fn exercise_enum_values<R: argui::runtime::Render>(mut app: TestApp<R>) {
    app.assert_text("ready");
    app.get_by_role(Role::Button, "Change").click().unwrap();
    app.assert_text("waiting");
}

/// The real Dialog renders all three caller-authored named slots in AOT.
#[test]
fn aot_dialog_slots() {
    exercise_dialog_slots(TestApp::new(DialogSlotsProof::new()));
}

/// The real Dialog renders all three caller-authored named slots in live mode.
#[cfg(feature = "argui-live")]
#[test]
fn live_dialog_slots() {
    exercise_dialog_slots(TestApp::new(live_fixture("DialogSlotsProof")));
}

/// Checks the mounted header, body, and actions of one modal dialog.
fn exercise_dialog_slots<R: argui::runtime::Render>(app: TestApp<R>) {
    app.assert_text("Custom heading");
    app.assert_text("Custom body");
    app.assert_text("Custom action");
}

/// The real Input renders leading, trailing, and error content in AOT.
#[test]
fn aot_input_slots() {
    exercise_input_slots(TestApp::new(InputSlotsProof::new()));
}

/// The real Input renders leading, trailing, and error content in live mode.
#[cfg(feature = "argui-live")]
#[test]
fn live_input_slots() {
    exercise_input_slots(TestApp::new(live_fixture("InputSlotsProof")));
}

/// Checks all three named adornment and validation slots.
fn exercise_input_slots<R: argui::runtime::Render>(app: TestApp<R>) {
    app.assert_text("Leading");
    app.assert_text("Trailing");
    app.assert_text("Validation error");
}

/// AOT model row edits keep stable identities and update only the mounted window.
#[test]
fn aot_model_row_edits_preserve_identity() {
    let model = argui::reactive::Model::new(["one".to_owned(), "two".to_owned()]);
    let component = EditableModel::new();
    assert!(component.set_rows(model));
    assert_eq!(component.rows().row_id(0), Some(1));
    assert_eq!(component.rows_insert(1, "middle".into()), Some(3));
    assert!(component.rows_move(2, 0));
    assert_eq!(component.rows().row_id(0), Some(2));
    assert!(component.rows_update(0, |row| {
        *row = "second".into();
        true
    }));
    assert_eq!(component.rows_remove(1).map(|(id, _)| id), Some(1));
    let app = TestApp::new(component);
    app.assert_text("second");
    app.assert_text("middle");
    app.assert_no_text("one");
}

/// AOT rich rows use measured extents and keep the mounted row count bounded.
#[test]
fn aot_variable_height_rows_are_measured_and_virtualized() {
    let component = RichRows::new();
    component.set_rows(argui::reactive::Model::new(
        (0..1000).map(|index| format!("row-{index}")),
    ));
    exercise_variable_rows(TestApp::new(component));
}

/// Checks the second row's measured offset and excludes distant rows.
fn exercise_variable_rows<R: argui::runtime::Render>(app: TestApp<R>) {
    app.assert_text("row-0");
    app.assert_text("row-1");
    app.assert_no_text("row-999");
    let first = app.bounds(argui_testing::Selector::text("row-0")).unwrap();
    let second = app.bounds(argui_testing::Selector::text("row-1")).unwrap();
    assert!(
        second.origin.y >= first.origin.y + 39.0,
        "second row was not measured: {first:?} {second:?}"
    );
}

/// Live rich rows share the same measured heights and bounded mounted window.
#[cfg(feature = "argui-live")]
#[test]
fn live_variable_height_rows_are_measured_and_virtualized() {
    use argui_dsl_compiler::{Compiler, SourceModule};
    use argui_dsl_runtime::{DslValue, LivePackage, LiveRuntime};
    let compiled = Compiler::compile(
        [SourceModule::new(
            "tests/fixtures/language.argui",
            include_str!("../fixtures/language.argui"),
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
        .find(|definition| definition.name == "RichRows")
        .unwrap()
        .id;
    let definition = compiled
        .ir
        .components
        .iter()
        .find(|component| component.id.raw() == symbol.raw())
        .unwrap();
    let root = definition.id;
    let rows = definition.properties[0].id;
    let package =
        LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, Default::default()).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    runtime
        .mount(
            root,
            [(
                rows,
                DslValue::Array(
                    (0..1000)
                        .map(|index| DslValue::String(format!("row-{index}")))
                        .collect(),
                ),
            )],
        )
        .unwrap();
    exercise_variable_rows(TestApp::new(runtime));
}

/// AOT reads the previous completed layout size from a generic container.
#[test]
fn aot_generic_container_reports_measured_dimensions() {
    exercise_measured_container(TestApp::new(MeasuredContainer::new()));
}

/// Live reads the same completed layout size without authored host coordinates.
#[cfg(feature = "argui-live")]
#[test]
fn live_generic_container_reports_measured_dimensions() {
    exercise_measured_container(TestApp::new(live_fixture("MeasuredContainer")));
}

/// Checks stable logical bounds after the observation feedback frame settles.
fn exercise_measured_container<R: argui::runtime::Render>(app: TestApp<R>) {
    app.assert_text("measured");
}

/// AOT host content replaces a named public root slot.
#[test]
fn aot_typed_root_slot_accepts_host_elements() {
    let component = HostSlots::new();
    component.set_header([argui::ui::Element::text("Host header")]);
    TestApp::new(component).assert_text("Host header");
}

/// Live generated slot setter uses the same element type and slot name.
#[cfg(feature = "argui-live")]
#[test]
fn live_typed_root_slot_accepts_host_elements() {
    let mut runtime = live_fixture("HostSlots");
    HostSlotsLive::attach(&runtime)
        .unwrap()
        .set_header(&mut runtime, [argui::ui::Element::text("Host header")])
        .unwrap();
    TestApp::new(runtime).assert_text("Host header");
}

/// AOT expands typed pure calls and tracks changing argument dependencies.
#[test]
fn aot_pure_functions_track_arguments() {
    exercise_pure_functions(TestApp::new(PureFunctions::new()));
}

/// Live bytecode evaluates the same expanded pure expression.
#[cfg(feature = "argui-live")]
#[test]
fn live_pure_functions_track_arguments() {
    exercise_pure_functions(TestApp::new(live_fixture("PureFunctions")));
}

/// Checks the initial and mutated results of a nested pure call.
///
/// `app` is an AOT or live root with the same public test surface.
fn exercise_pure_functions<R: argui::runtime::Render>(mut app: TestApp<R>) {
    app.assert_text("pure:7");
    app.get_by_role(Role::Button, "Advance pure")
        .click()
        .unwrap();
    app.assert_text("pure:9");
}
