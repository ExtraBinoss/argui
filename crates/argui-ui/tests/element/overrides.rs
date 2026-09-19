use argui_animation::{Duration, Motion, Transition, Tween};
use argui_core::Color;
use argui_paint::{Border, BorderWidths, Fill};
use argui_ui::{Element, StateName, StylePatch, StyleTransition, TransitionRule, UiTree, property};

const ACTIVE: StateName = StateName::new("active");

fn button(active: bool) -> Element {
    Element::container([])
        .background(Color::WHITE)
        .border(Border::all(1.0, Color::WHITE))
        .active_state(ACTIVE, active)
        .when(
            ACTIVE,
            StylePatch::new()
                .set(property::BackgroundColor, Color::BLACK)
                .set(property::BorderColor, Color::BLACK)
                .set(property::Opacity, 0.5),
        )
        .transition(
            StyleTransition::default().rule(
                TransitionRule::new(Transition::tween(Tween::new(Duration::from_millis(1000))))
                    .property(argui_ui::PropertyKey::BackgroundColor),
            ),
        )
        .bind(property::BackgroundColor, Motion::new(Color::BLACK))
        .bind(property::BorderColor, Motion::new(Color::BLACK))
        .bind(property::BorderWidths, Motion::new([3.0; 4]))
}

#[test]
fn color_overrides_remain_live_through_state_changes_and_reset_restores_authored_styles() {
    let color = Color::srgb(0.2, 0.8, 0.4);
    let mut tree = UiTree::new(button(false));
    let node = tree.node_ids()[0];
    for active in [false, true, false] {
        let mut edited = button(active);
        edited.override_background(Some(Fill::Solid(color)));
        edited.override_border(Some(Border::all(2.0, color)));
        tree.update(edited);
        let quad = tree.resolved_quad(node, tree.root());
        assert_eq!(quad.background, Some(Fill::Solid(color)));
        assert_eq!(quad.border, Some(Border::all(2.0, color)));
    }
    tree.update(button(true));
    let quad = tree.resolved_quad(node, tree.root());
    assert_eq!(quad.background, Some(Fill::Solid(Color::BLACK)));
    assert_eq!(quad.border, Some(Border::all(3.0, Color::BLACK)));
}

#[test]
fn removing_colors_keeps_unrelated_state_properties_and_bindings() {
    let mut element = button(true).bind(property::Opacity, Motion::new(0.25));
    element.override_background(None);
    element.override_border(None);
    let tree = UiTree::new(element);
    let quad = tree.resolved_quad(tree.node_ids()[0], tree.root());
    assert_eq!(quad.background, None);
    assert_eq!(quad.border, None);
    assert_eq!(quad.opacity, 0.25);

    let mut element = Element::container([]).when(
        ACTIVE,
        StylePatch::new()
            .set(property::Background, Some(Fill::Solid(Color::WHITE)))
            .set(property::Border, Some(Border::all(1.0, Color::WHITE))),
    );
    element.override_background(None);
    element.override_border(None);
    assert!(!element.has_state_animation());
}

#[test]
fn border_width_override_keeps_conditional_border_colors() {
    let widths = BorderWidths {
        left: 0.0,
        right: 1.0,
        top: 1.0,
        bottom: 1.0,
    };
    let mut element = button(true);
    element.override_border_widths(widths);
    let tree = UiTree::new(element);
    let border = tree
        .resolved_quad(tree.node_ids()[0], tree.root())
        .border
        .unwrap();

    assert_eq!(border.widths, widths);
    assert_eq!(border.color, Color::BLACK);
}

#[test]
fn disabling_and_reenabling_opacity_overrides_state_and_motion_values() {
    let authored = Element::image(argui_ui::ImageId(1))
        .paint_opacity(0.4)
        .active_state(ACTIVE, true)
        .when(ACTIVE, StylePatch::new().set(property::Opacity, 0.2))
        .transition(StyleTransition::default())
        .bind(property::Opacity, Motion::new(0.1));
    let mut tree = UiTree::new(authored.clone());
    let node = tree.node_ids()[0];
    for opacity in [0.3, 1.0, 0.3] {
        let mut edited = authored.clone();
        edited.override_paint_opacity(opacity);
        tree.update(edited);
        assert_eq!(tree.resolved_quad(node, tree.root()).opacity, opacity);
    }
    tree.update(authored);
    assert_eq!(tree.resolved_quad(node, tree.root()).opacity, 0.1);
}
