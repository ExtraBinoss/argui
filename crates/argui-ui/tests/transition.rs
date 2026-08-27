use argui_animation::{CubicBezier, Duration, Easing, Time};
use argui_core::Point;
use argui_ui::{
    Border, Color, CornerRadii, Element, Fill, GradientStop, LinearGradient, RadialGradient,
    Transition, TreeUpdate, UiTree,
};

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
        Fill::Linear(_) | Fill::Radial(_) => panic!("test panel must remain solid"),
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

fn gradient_panel(fill: Fill) -> Element {
    Element::container([])
        .keyed("gradient")
        .fill(fill)
        .transition(Transition::new(Duration::from_millis(100)))
}

fn gradient_stops(first: Color, last: Color) -> [GradientStop; 2] {
    [GradientStop::new(0.0, first), GradientStop::new(1.0, last)]
}

#[test]
fn matching_linear_and_radial_gradients_interpolate_without_becoming_solid() {
    let red = Color::rgb(1.0, 0.0, 0.0);
    let blue = Color::rgb(0.0, 0.0, 1.0);
    let green = Color::rgb(0.0, 1.0, 0.0);
    let linear = |start, end, first, last| {
        Fill::Linear(LinearGradient::new(start, end, gradient_stops(first, last)).unwrap())
    };
    let mut tree = UiTree::new(gradient_panel(linear(
        Point::default(),
        Point::new(1.0, 0.0),
        red,
        blue,
    )));
    tree.update(gradient_panel(linear(
        Point::new(0.2, 0.2),
        Point::new(0.8, 1.0),
        green,
        red,
    )));
    tree.advance_animations(Time::ZERO);
    tree.advance_animations(Time::from_nanos(50_000_000));
    let node = tree.node_id_at(0).unwrap();
    let Fill::Linear(linear) = tree.resolved_quad(node, tree.root()).background.unwrap() else {
        panic!("linear gradients must stay gradients");
    };
    assert_eq!(linear.start, Point::new(0.1, 0.1));
    assert_eq!(linear.stops.as_slice()[0].color, Color::rgb(0.5, 0.5, 0.0));

    let radial = |center, radius, first, last| {
        Fill::Radial(RadialGradient::new(center, radius, gradient_stops(first, last)).unwrap())
    };
    let mut tree = UiTree::new(gradient_panel(radial(
        Point::default(),
        Point::new(0.5, 0.5),
        red,
        blue,
    )));
    tree.update(gradient_panel(radial(
        Point::new(1.0, 1.0),
        Point::new(1.0, 1.0),
        blue,
        green,
    )));
    tree.advance_animations(Time::ZERO);
    tree.advance_animations(Time::from_nanos(50_000_000));
    let node = tree.node_id_at(0).unwrap();
    let Fill::Radial(radial) = tree.resolved_quad(node, tree.root()).background.unwrap() else {
        panic!("radial gradients must stay gradients");
    };
    assert_eq!(radial.center, Point::new(0.5, 0.5));
    assert_eq!(radial.radius, Point::new(0.75, 0.75));
}

#[test]
fn incompatible_gradients_crossfade_from_their_first_color() {
    let red = Color::rgb(1.0, 0.0, 0.0);
    let blue = Color::rgb(0.0, 0.0, 1.0);
    let base = Fill::Linear(
        LinearGradient::new(
            Point::default(),
            Point::new(1.0, 0.0),
            gradient_stops(red, blue),
        )
        .unwrap(),
    );
    let target = Fill::Radial(
        RadialGradient::new(
            Point::new(0.5, 0.5),
            Point::new(1.0, 1.0),
            gradient_stops(blue, red),
        )
        .unwrap(),
    );
    let mut tree = UiTree::new(gradient_panel(base));
    tree.update(gradient_panel(target));
    tree.advance_animations(Time::ZERO);
    tree.advance_animations(Time::from_nanos(50_000_000));
    let node = tree.node_id_at(0).unwrap();
    assert_eq!(
        tree.resolved_quad(node, tree.root()).background,
        Some(Fill::Solid(Color::rgb(0.5, 0.0, 0.5)))
    );

    tree.update(
        Element::container([])
            .keyed("gradient")
            .transition(Transition::new(Duration::from_millis(100))),
    );
    tree.advance_animations(Time::from_nanos(50_000_000));
    tree.advance_animations(Time::from_nanos(150_000_000));
    assert!(tree.resolved_quad(node, tree.root()).background.is_none());
}
