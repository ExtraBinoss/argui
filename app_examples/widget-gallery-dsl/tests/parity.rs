#![cfg(feature = "argui-live")]

use argui::{
    core::Point,
    paint::{Fill, Filter},
    runtime::Render,
    ui::{Element, ElementKind, Role},
};
use argui_cli::DevCompilerService;
use argui_dsl_protocol::LiveMessage;
use argui_dsl_runtime::{LivePackage, LiveRuntime};
use argui_example_widget_gallery_dsl::Main;
use argui_testing::{Selector, TestApp};

#[path = "support/navigation.rs"]
mod navigation;
use navigation::navigate_to_page;

#[path = "parity/performance.rs"]
mod performance;

/// Compiles the real gallery through the development service and mounts its root.
///
/// Returns a headless live app using the same package as `argui dev`.
fn live_gallery() -> TestApp<LiveRuntime> {
    let mut service =
        DevCompilerService::open(env!("CARGO_MANIFEST_DIR"), "ui/main.argui").unwrap();
    let LiveMessage::Package(envelope) = service.compile().message else {
        panic!("the gallery's live generation was rejected");
    };
    let root = envelope.roots[0];
    let package = LivePackage::from_envelope(*envelope).unwrap();
    let mut runtime = LiveRuntime::new(package).unwrap();
    runtime.mount(root, []).unwrap();
    TestApp::new(runtime)
}

/// Captures visible text, paint, portal and accessible labels without backend IDs.
///
/// `app` supplies a settled AOT or live tree; returns entries in paint order.
fn visual_signature<A: Render>(app: &TestApp<A>) -> Vec<String> {
    let tree = app.rendered_tree();
    let mut output = Vec::new();
    for id in tree.node_ids() {
        let element = tree.element_for(*id).unwrap();
        if let ElementKind::Text { content, style } = &element.kind {
            output.push(format!("text:{:?}:{style:?}", content.as_str()));
        }
        if let ElementKind::TextEditor {
            value, text: style, ..
        } = &element.kind
        {
            output.push(format!("editor:{value:?}:{style:?}"));
        }
        if let Some(fill) = &element.paint.quad.background {
            output.push(format!("fill:{fill:?}"));
        }
        if let Some(border) = &element.paint.quad.border {
            output.push(format!("border:{border:?}"));
        }
        if let Some(semantics) = &element.semantics {
            output.push(format!("role:{:?}:{:?}", semantics.role, semantics.label));
        }
        if let Some(portal) = &element.portal {
            output.push(format!("portal:{:?}:{:?}", portal.layer, portal.target));
        }
        if let Some(layer) = &element.layer
            && !layer.backdrop_filters.is_empty()
        {
            output.push(format!("backdrop:{:?}", layer.backdrop_filters));
        }
        if !element.effects.is_empty() {
            output.push(format!("effects:{:?}", element.effects));
        }
    }
    output
}

/// Finds the first painted surface in a component subtree.
///
/// `element` is a component root; returns its first brush when present.
fn first_fill(element: &Element) -> Option<Fill> {
    element
        .paint
        .quad
        .background
        .clone()
        .or_else(|| element.children.iter().find_map(first_fill))
}

/// Returns the painted fill of the button with the accessible `label`.
///
/// `app` is a settled gallery and `label` identifies one button.
fn button_fill<A: Render>(app: &TestApp<A>, label: &str) -> Fill {
    let tree = app.rendered_tree();
    tree.node_ids()
        .iter()
        .filter_map(|id| tree.element_for(*id))
        .find(|element| {
            element.semantics.as_ref().is_some_and(|semantics| {
                semantics.role == Role::Button && semantics.label.as_deref() == Some(label)
            })
        })
        .and_then(first_fill)
        .unwrap_or_else(|| panic!("{label} has no painted surface"))
}

/// The real gallery has identical AOT and live paint across every component page.
#[test]
fn gallery_pages_have_aot_live_visual_parity() {
    let mut aot = TestApp::new(Main::new());
    let mut live = live_gallery();
    assert_eq!(
        visual_signature(&aot),
        visual_signature(&live),
        "initial gallery"
    );
    assert_ne!(button_fill(&aot, "Primary"), button_fill(&aot, "Outline"));
    assert_eq!(button_fill(&aot, "Outline"), button_fill(&live, "Outline"));

    for page in [
        "Slider",
        "Input & Search",
        "Text area",
        "Overlays",
        "Selection",
        "Menu",
        "ScrollView",
        "Scrollbar styling",
        "Card",
        "Badge",
        "Separator",
        "Switch",
        "Media",
        "Visual composition",
        "VList",
        "Button",
    ] {
        navigate_to_page(&mut aot, page);
        navigate_to_page(&mut live, page);
        assert_eq!(
            visual_signature(&aot),
            visual_signature(&live),
            "page {page}"
        );
    }
}

/// Nested menu selection produces the same popup tree and action feedback in both paths.
#[test]
fn nested_menu_has_aot_live_parity() {
    let mut aot = TestApp::new(Main::new());
    let mut live = live_gallery();
    exercise_menu(&mut aot);
    exercise_menu(&mut live);
    assert_eq!(visual_signature(&aot), visual_signature(&live));
}

/// Opens File, Export and Image on either gallery renderer.
///
/// `app` receives the same interaction sequence for AOT and live rendering.
fn exercise_menu<A: Render>(app: &mut TestApp<A>) {
    navigate_to_page(app, "Menu");
    app.get_by_role(Role::ComboBox, "File").click().unwrap();
    app.get_by_role(Role::MenuItem, "Export").click().unwrap();
    app.get_by_role(Role::MenuItem, "Image").click().unwrap();
}

/// Opening a popover keeps following controls fixed in both execution paths.
#[test]
fn popover_preserves_layout_and_modal_backdrop_has_blur() {
    let mut aot = TestApp::new(Main::new());
    let mut live = live_gallery();
    exercise_overlay(&mut aot);
    exercise_overlay(&mut live);
    assert_eq!(visual_signature(&aot), visual_signature(&live));
}

/// Exercises one gallery's popup placement and modal backdrop.
///
/// `app` may use the generated or interpreted renderer.
fn exercise_overlay<A: Render>(app: &mut TestApp<A>) {
    navigate_to_page(app, "Overlays");
    let trigger = Selector::role(Role::Button, "Open dialog");
    let before = app.bounds(trigger.clone()).unwrap();
    app.get_by_role(Role::Button, "Open popover")
        .click()
        .unwrap();
    assert_eq!(app.bounds(trigger.clone()).unwrap(), before);
    assert!(has_backdrop_blur(app), "popover should blur its background");
    app.get_by_role(Role::Button, "Close popover")
        .click()
        .unwrap();
    app.get_by_role(Role::Button, "Open WGSL popover")
        .click()
        .unwrap();
    assert_eq!(app.bounds(trigger.clone()).unwrap(), before);
    assert!(
        has_scoped_border_effect(app),
        "custom popover border should run WGSL"
    );
    app.get_by_role(Role::Button, "Open WGSL popover")
        .click()
        .unwrap();
    app.get_by_role(Role::Button, "Open dialog")
        .click()
        .unwrap();
    assert!(has_backdrop_blur(app), "dialog should blur the viewport");
}

/// Reports whether a visible rectangle has a shader scoped to its border.
///
/// `app` supplies the rendered popup scene.
fn has_scoped_border_effect<A: Render>(app: &TestApp<A>) -> bool {
    let tree = app.rendered_tree();
    tree.node_ids()
        .iter()
        .filter_map(|id| tree.element_for(*id))
        .any(|element| {
            element.effects.iter().any(|effect| {
                effect.scope == argui::ui::EffectScope::Border
                    && effect
                        .layer
                        .filters
                        .iter()
                        .any(|filter| matches!(filter, Filter::Effect(_)))
            })
        })
}

/// Reports whether the settled tree contains an active backdrop blur layer.
///
/// `app` supplies the rendered popup or modal scene.
fn has_backdrop_blur<A: Render>(app: &TestApp<A>) -> bool {
    let tree = app.rendered_tree();
    tree.node_ids()
        .iter()
        .filter_map(|id| tree.element_for(*id))
        .any(|element| {
            element.layer.as_ref().is_some_and(|layer| {
                layer
                    .backdrop_filters
                    .iter()
                    .any(|filter| matches!(filter, Filter::Blur(radius) if *radius > 0.0))
            })
        })
}

/// Dragging the slider updates the value and the rendered track in AOT and live.
#[test]
fn slider_drag_has_aot_live_parity() {
    let mut aot = TestApp::new(Main::new());
    let mut live = live_gallery();
    exercise_slider(&mut aot);
    exercise_slider(&mut live);
    assert_eq!(visual_signature(&aot), visual_signature(&live));
}

/// Text area and split pane resize handles settle at the same sizes in AOT and live mode.
#[test]
fn resize_handles_have_aot_live_parity() {
    let mut aot = TestApp::new(Main::new());
    let mut live = live_gallery();
    exercise_resize_handles(&mut aot);
    exercise_resize_handles(&mut live);
    assert_eq!(visual_signature(&aot), visual_signature(&live));
}

/// Drags the two-axis editor corner and a horizontal pane separator.
///
/// `app` may use the generated or interpreted DSL renderer.
fn exercise_resize_handles<A: Render>(app: &mut TestApp<A>) {
    navigate_to_page(app, "Text area");
    let editor = Selector::role(Role::Separator, "Resize notes editor");
    let before = app.bounds(editor.clone()).unwrap();
    let start = Point::new(before.origin.x + 9.0, before.origin.y + 9.0);
    app.drag(start, Point::new(start.x + 35.0, start.y + 25.0), 12)
        .unwrap();
    let after = app.bounds(editor).unwrap();
    assert!(after.origin.x >= before.origin.x + 30.0);
    assert!(after.origin.y >= before.origin.y + 20.0);

    navigate_to_page(app, "Split Pane");
    let splitter = Selector::role(Role::Separator, "Resize file explorer");
    let before = app.bounds(splitter.clone()).unwrap();
    let start = Point::new(before.origin.x + 4.0, before.origin.y + 50.0);
    app.drag(start, Point::new(start.x + 35.0, start.y), 12)
        .unwrap();
    let after = app.bounds(splitter).unwrap();
    assert!(after.origin.x >= before.origin.x + 30.0);
}

/// Drags one gallery slider and verifies its displayed value changed.
///
/// `app` may use the generated or interpreted renderer.
fn exercise_slider<A: Render>(app: &mut TestApp<A>) {
    navigate_to_page(app, "Slider");
    let bounds = app.bounds(Selector::role(Role::Slider, "Volume")).unwrap();
    let y = bounds.origin.y + bounds.size.height / 2.0;
    app.drag(
        Point::new(bounds.origin.x + 20.0, y),
        Point::new(bounds.origin.x + bounds.size.width - 20.0, y),
        4,
    )
    .unwrap();
    assert!(
        !app.semantics()
            .nodes
            .iter()
            .any(|node| { node.semantics.label.as_deref() == Some("Volume: 25") })
    );
}

/// State changes keep AOT and live visual output equivalent across common controls.
#[test]
fn gallery_dynamic_states_have_aot_live_visual_parity() {
    let mut aot = TestApp::new(Main::new());
    let mut live = live_gallery();
    aot.get_by_role(Role::Button, "Primary").click().unwrap();
    live.get_by_role(Role::Button, "Primary").click().unwrap();
    assert_eq!(
        visual_signature(&aot),
        visual_signature(&live),
        "button click"
    );

    aot.get_by_role(Role::ComboBox, "Light · appearance")
        .click()
        .unwrap();
    live.get_by_role(Role::ComboBox, "Light · appearance")
        .click()
        .unwrap();
    assert_eq!(
        visual_signature(&aot),
        visual_signature(&live),
        "theme popover"
    );
    aot.get_by_role(Role::Button, "Dark").click().unwrap();
    live.get_by_role(Role::Button, "Dark").click().unwrap();
    assert_eq!(
        visual_signature(&aot),
        visual_signature(&live),
        "dark theme"
    );
    aot.get_by_role(Role::ComboBox, "Dark · appearance")
        .click()
        .unwrap();
    live.get_by_role(Role::ComboBox, "Dark · appearance")
        .click()
        .unwrap();
    aot.get_by_role(Role::Button, "Emerald").click().unwrap();
    live.get_by_role(Role::Button, "Emerald").click().unwrap();
    assert_eq!(
        visual_signature(&aot),
        visual_signature(&live),
        "emerald theme"
    );

    navigate_to_page(&mut aot, "Switch");
    navigate_to_page(&mut live, "Switch");
    aot.get_by_role(Role::Switch, "Notifications")
        .click()
        .unwrap();
    live.get_by_role(Role::Switch, "Notifications")
        .click()
        .unwrap();
    assert_eq!(
        visual_signature(&aot),
        visual_signature(&live),
        "switch state"
    );

    navigate_to_page(&mut aot, "Input & Search");
    navigate_to_page(&mut live, "Input & Search");
    let field = Selector::role(Role::TextInput, "Full name");
    aot.replace_text(field.clone(), "Grace Hopper").unwrap();
    live.replace_text(field, "Grace Hopper").unwrap();
    assert_eq!(
        visual_signature(&aot),
        visual_signature(&live),
        "controlled input"
    );
}
