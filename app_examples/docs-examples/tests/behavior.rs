use argui::{
    accessibility::Role,
    core::{Key, Point, ScrollDelta},
};
use argui_example_docs::examples;
use argui_testing::TestApp;

#[test]
fn counter_uses_a_direct_button_callback() {
    let mut app = TestApp::new(examples::counter::Example::default());
    app.get_by_role(Role::Button, "Increment").click().unwrap();
    app.assert_text("Count: 1");
}

#[test]
fn accessibility_example_announces_each_activation() {
    let mut app = TestApp::new(examples::accessibility::Example::default());
    app.click("announce").unwrap();
    app.assert_text("1 accessible activations");
}

#[test]
fn animation_example_toggles_locally() {
    let mut app = TestApp::new(examples::animation::Example::default());
    app.click("toggle-animation").unwrap();
    app.assert_text("Animations paused");
    app.assert_text("Resume animations");
}

#[test]
fn clean_code_example_completes_one_item() {
    let mut app = TestApp::new(examples::clean_code::Example::default());
    app.click("next").unwrap();
    app.assert_text("✓  State is explicit");
}

#[test]
fn event_delegation_example_remains_explicit() {
    let mut app = TestApp::new(examples::events::Example::default());
    app.click("save").unwrap();
    app.assert_text("Received Click(save)");
}

#[test]
fn virtual_data_example_updates_from_real_wheel_scrolling() {
    let mut app = TestApp::new(examples::data::Example::default());
    let bounds = app.bounds("records").unwrap();
    let point = Point::new(
        bounds.origin.x + bounds.size.width * 0.5,
        bounds.origin.y + bounds.size.height * 0.5,
    );

    app.wheel(point, ScrollDelta::Pixels(Point::new(0.0, -900.0)))
        .unwrap();

    app.assert_no_text("Record #00001");
    app.assert_text("Record #00022");
}

#[test]
fn i18n_controls_update_the_controlled_locale_and_count() {
    let mut app = TestApp::new(examples::i18n::Example::default());
    app.click("fr").unwrap();
    app.assert_text("Bonjour");
    app.click("more").unwrap();
    app.assert_text("messages");
}

#[test]
fn overlays_open_and_close_through_their_typed_state_handlers() {
    let mut app = TestApp::new(examples::overlays::Example::default());
    app.click("solid-popover").unwrap();
    app.assert_text("An opaque panel with backdrop blur disabled.");
    app.click("close-solid").unwrap();
    app.assert_no_text("An opaque panel with backdrop blur disabled.");

    app.click("blurred-popover").unwrap();
    app.assert_text("Translucent paint keeps the colored backdrop visible through the blur.");
    app.click("close-blurred").unwrap();
    app.assert_no_text("Translucent paint keeps the colored backdrop visible through the blur.");

    app.click("example-dialog::trigger").unwrap();
    app.assert_text("A real modal portal");
    app.get_by_role(Role::Button, "Close dialog")
        .click()
        .unwrap();
    app.assert_no_text("A real modal portal");
}

#[test]
fn platform_support_distinguishes_supported_and_preview_targets() {
    let app = TestApp::new(examples::platform_support::Example);
    app.assert_text("Supported · runtime-tested");
    app.assert_text("Supported · browser-tested");
    app.assert_text("Preview");
}

#[test]
fn platform_roadmap_reveals_capabilities_marked_as_planned() {
    let mut app = TestApp::new(examples::platform_roadmap::Example::default());
    app.assert_no_text("Health integrations");
    app.click("toggle-roadmap").unwrap();
    app.assert_text("Health integrations");
    app.assert_text("Later");
}

#[test]
fn styling_controls_replace_their_own_label() {
    let mut app = TestApp::new(examples::styling::Example::default());
    app.click("tokens").unwrap();
    app.assert_text("Reset tokens");
}

#[test]
fn async_example_enters_loading_without_blocking_input() {
    let mut app = TestApp::new(examples::tasks::Example::default());
    app.click("load").unwrap();
    app.assert_text("Loading…");
    app.key(Key::Tab, Default::default()).unwrap();
}
