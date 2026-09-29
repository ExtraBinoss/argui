use super::*;

#[test]
fn disabling_a_transition_settles_its_unchanged_conditional_target() {
    let element = Element::container([])
        .width(length(100.0))
        .interaction(Interaction::default())
        .when(
            VisualState::Hovered,
            StylePatch::new().set(property::WidthPx, 200.0),
        )
        .transition(transition());
    let mut ui = UiTree::new(element.clone());
    let node = ui.node_ids()[0];
    ui.pointer_moved(Point::new(10.0, 10.0), &[region(node)]);
    ui.advance_animations(Time::from_nanos(1));
    ui.advance_animations(Time::from_nanos(50_000_001));
    assert_eq!(
        ui.resolved_layout_style(node, ui.root()).size.width,
        length(150.0)
    );

    let mut disabled = element;
    disabled.style_transition = None;
    let revision = ui.revision();
    assert_eq!(ui.update(disabled), TreeUpdate::Layout);
    assert!(ui.revision() > revision);
    assert_eq!(
        ui.resolved_layout_style(node, ui.root()).size.width,
        length(200.0)
    );
    assert!(!ui.wants_animation_frame());
    assert_eq!(
        ui.advance_animations(Time::from_nanos(150_000_001)),
        TreeUpdate::None
    );
}

#[test]
fn removing_a_settled_transition_keeps_the_authored_size_and_node_identity() {
    let element = Element::container([])
        .width(length(100.0))
        .transition(transition());
    let mut ui = UiTree::new(element);
    let node = ui.node_ids()[0];
    assert_eq!(
        ui.update(Element::container([]).width(length(100.0))),
        TreeUpdate::Layout
    );
    assert_eq!(ui.node_ids(), &[node]);
    assert_eq!(
        ui.resolved_layout_style(node, ui.root()).size.width,
        length(100.0)
    );
    assert!(ui.layout_animation_indices().is_empty());
    assert!(!ui.wants_animation_frame());
}
