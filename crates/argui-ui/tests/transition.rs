use argui_animation::{CubicBezier, Duration, Easing, Time};
use argui_ui::{Border, Color, CornerRadii, Element, Fill, Transition, TreeUpdate, UiTree};

fn panel(color: Color, radius: f32, opacity: f32) -> Element {
    Element::container([])
        .keyed("panel")
        .background(color)
        .border(Border::all(2.0, color))
        .radius(CornerRadii::all(radius))
        .paint_opacity(opacity)
        .transition(
            Transition::new(Duration::from_millis(100)).easing(Easing::CubicBezier(
                CubicBezier::new(0.0, 0.0, 1.0, 1.0).unwrap(),
            )),
        )
}

fn background(tree: &UiTree) -> Color {
    let node = tree.node_id_at(0).unwrap();
    match tree.resolved_quad(node, tree.root()).background.unwrap() {
        Fill::Solid(color) => color,
    }
}

#[test]
fn paint_transition_is_retained_by_node_and_retargets_without_a_jump() {
    let red = Color::rgb(1.0, 0.0, 0.0);
    let blue = Color::rgb(0.0, 0.0, 1.0);
    let green = Color::rgb(0.0, 1.0, 0.0);
    let mut tree = UiTree::new(panel(red, 0.0, 1.0));

    assert_eq!(tree.update(panel(blue, 20.0, 0.5)), TreeUpdate::Paint);
    assert!(tree.wants_animation_frame());
    assert_eq!(background(&tree), red);
    assert!(tree.advance_animations(Time::ZERO));
    tree.advance_animations(Time::from_nanos(50_000_000));
    let halfway = background(&tree);
    assert_eq!(halfway, Color::rgb(0.5, 0.0, 0.5));

    assert_eq!(tree.update(panel(green, 10.0, 0.75)), TreeUpdate::Paint);
    tree.advance_animations(Time::from_nanos(50_000_000));
    assert_eq!(background(&tree), halfway);
    tree.advance_animations(Time::from_nanos(100_000_000));
    assert_eq!(background(&tree), Color::rgb(0.25, 0.5, 0.25));
    tree.advance_animations(Time::from_nanos(150_000_000));
    assert_eq!(background(&tree), green);
    assert!(!tree.wants_animation_frame());
}

#[test]
fn transition_interpolates_border_radii_opacity_and_optional_paint() {
    let transition = Transition::new(Duration::from_millis(100)).delay(Duration::from_millis(20));
    let base = Element::container([])
        .keyed("optional")
        .transition(transition.clone());
    let target = Element::container([])
        .keyed("optional")
        .background(Color::WHITE)
        .border(Border::all(4.0, Color::WHITE))
        .radius(CornerRadii::all(12.0))
        .paint_opacity(0.4)
        .transition(transition);
    let mut tree = UiTree::new(base);
    assert_eq!(tree.update(target), TreeUpdate::Paint);
    tree.advance_animations(Time::ZERO);
    tree.advance_animations(Time::from_nanos(20_000_000));
    let node = tree.node_id_at(0).unwrap();
    let start = tree.resolved_quad(node, tree.root());
    assert_eq!(start.radii, CornerRadii::all(0.0));
    tree.advance_animations(Time::from_nanos(70_000_000));
    let middle = tree.resolved_quad(node, tree.root());
    assert_eq!(middle.radii, CornerRadii::all(6.0));
    assert_eq!(middle.opacity, 0.7);
    assert_eq!(middle.border.unwrap().widths.left, 2.0);
}

#[test]
fn zero_duration_and_removed_specs_apply_immediately() {
    let red = Color::rgb(1.0, 0.0, 0.0);
    let blue = Color::rgb(0.0, 0.0, 1.0);
    let mut tree = UiTree::new(
        Element::container([])
            .keyed("panel")
            .background(red)
            .transition(Transition::new(Duration::ZERO)),
    );
    tree.update(
        Element::container([])
            .keyed("panel")
            .background(blue)
            .transition(Transition::new(Duration::ZERO)),
    );
    assert_eq!(background(&tree), blue);
    assert!(!tree.wants_animation_frame());

    tree.update(Element::container([]).keyed("panel").background(red));
    assert_eq!(background(&tree), red);
}
