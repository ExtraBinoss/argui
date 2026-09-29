use argui_animation::{Duration, Time, Transition, Tween};
use argui_core::{Point, Size};
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{
    Element, Interaction, StylePatch, StyleTransition, TreeUpdate, UiTree, VisualState, length,
    property,
};

/// Builds a sized child with an optional native width transition.
fn child(width: f32, animated: bool) -> Element {
    let mut child = Element::container([])
        .width(length(width))
        .height(length(20.0));
    if animated {
        child = child.transition(StyleTransition::new(Transition::tween(Tween::new(
            Duration::from_millis(100),
        ))));
    }
    child
}

#[test]
fn removing_width_transitions_restores_cached_geometry_without_other_input() {
    for keep_other_transition in [false, true] {
        let root = |width, animated| {
            let mut children = vec![child(width, animated).keyed("resizing")];
            if keep_other_transition {
                children.push(child(40.0, true).keyed("other"));
            }
            Element::row(children)
        };
        let mut ui = UiTree::new(root(100.0, true));
        let mut engine = LayoutEngine::new();
        let mut text = TextEngine::from_embedded_fonts([], "sans", "sans", "mono");
        let viewport = Size::new(400.0, 200.0);
        engine.compute(&mut ui, &mut text, viewport).unwrap();
        ui.update(root(200.0, true));
        ui.advance_animations(Time::from_nanos(1));
        ui.advance_animations(Time::from_nanos(50_000_001));
        let intermediate = engine.compute(&mut ui, &mut text, viewport).unwrap();
        assert_eq!(intermediate.nodes[1].bounds.size.width, 150.0);

        let revision = ui.revision();
        assert_eq!(ui.update(root(200.0, false)), TreeUpdate::Layout);
        assert!(ui.revision() > revision);
        assert!(!ui.wants_animation_frame());
        let restored = engine.compute(&mut ui, &mut text, viewport).unwrap();
        assert_eq!(restored.nodes[1].bounds.size.width, 200.0);
        assert_eq!(
            engine.compute(&mut ui, &mut text, viewport).unwrap().nodes,
            restored.nodes
        );
        let fresh = LayoutEngine::new()
            .compute(&mut UiTree::new(root(200.0, false)), &mut text, viewport)
            .unwrap();
        for (actual, expected) in restored.nodes.iter().zip(&fresh.nodes) {
            assert_eq!(actual.bounds, expected.bounds);
        }
    }
}

#[test]
fn removing_hover_rules_restores_the_base_even_when_the_cached_description_matches_it() {
    let root = |hover: bool| {
        let mut surface = child(100.0, hover)
            .keyed("hovered")
            .interaction(Interaction::default());
        if hover {
            surface = surface.when(
                VisualState::Hovered,
                StylePatch::new().set(property::WidthPx, 200.0),
            );
        }
        Element::row([surface, child(40.0, true).keyed("other")])
    };
    let mut ui = UiTree::new(root(true));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::from_embedded_fonts([], "sans", "sans", "mono");
    let viewport = Size::new(400.0, 200.0);
    let initial = engine.compute(&mut ui, &mut text, viewport).unwrap();
    ui.pointer_moved(Point::new(10.0, 10.0), &initial.hit_regions);
    ui.advance_animations(Time::from_nanos(1));
    ui.advance_animations(Time::from_nanos(50_000_001));
    let intermediate = engine.compute(&mut ui, &mut text, viewport).unwrap();
    assert_eq!(intermediate.nodes[1].bounds.size.width, 150.0);

    assert_eq!(ui.update(root(false)), TreeUpdate::Layout);
    let restored = engine.compute(&mut ui, &mut text, viewport).unwrap();
    assert_eq!(restored.nodes[1].bounds.size.width, 100.0);
    assert!(!ui.wants_animation_frame());
    let fresh = LayoutEngine::new()
        .compute(&mut UiTree::new(root(false)), &mut text, viewport)
        .unwrap();
    for (actual, expected) in restored.nodes.iter().zip(&fresh.nodes) {
        assert_eq!(actual.bounds, expected.bounds);
    }
}
