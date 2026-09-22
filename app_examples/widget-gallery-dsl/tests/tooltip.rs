use argui::{core::Point, paint::Fill, ui::Role};
use argui_example_widget_gallery_dsl::Main;
use argui_testing::{Selector, TestApp};

#[path = "support/navigation.rs"]
mod navigation;
use navigation::navigate_to_page;

/// The ordinary tooltip opens with an opaque surface and no backdrop filters.
#[test]
fn gallery_default_tooltip_uses_an_opaque_unfiltered_surface() {
    let mut app = TestApp::new(Main::new());
    navigate_to_page(&mut app, "Tooltip");
    let trigger = app
        .bounds(Selector::role(Role::Button, "Save draft"))
        .unwrap();
    app.pointer_move(Point::new(
        trigger.origin.x + trigger.size.width * 0.5,
        trigger.origin.y + trigger.size.height * 0.5,
    ))
    .unwrap();
    let description = "Save a local copy of your current draft.";
    app.assert_exists(Selector::role(Role::Tooltip, description));

    let tree = app.rendered_tree();
    let tooltip = tree
        .node_ids()
        .iter()
        .filter_map(|id| tree.element_for(*id))
        .find(|element| {
            element.semantics.as_ref().is_some_and(|semantics| {
                semantics.role == Role::Tooltip && semantics.label.as_deref() == Some(description)
            })
        })
        .unwrap();
    let surface = tooltip.children.first().unwrap();
    assert!(
        surface
            .layer
            .as_ref()
            .is_none_or(|layer| layer.backdrop_filters.is_empty())
    );
    assert!(
        matches!(surface.paint.quad.background, Some(Fill::Solid(color)) if color.to_linear_rgba()[3] == 1.0)
    );
}
