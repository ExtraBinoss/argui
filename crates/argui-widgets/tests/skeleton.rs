use argui_animation::Time;
use argui_core::{Color, ColorScheme, Size};
use argui_layout::LayoutEngine;
use argui_runtime::{Context, Entity, Render, WindowEnvironment};
use argui_text::TextEngine;
use argui_ui::{TreeUpdate, UiTree, length, percent};
use argui_widgets::{Skeleton, shadcn};

#[test]
fn shapes_reserve_their_space_and_stay_decorative() {
    let themes = shadcn(Color::WHITE);
    for scheme in [ColorScheme::Light, ColorScheme::Dark] {
        for width in [160.0, 400.0] {
            let skeleton = Skeleton::new("cover")
                .size(percent(0.5), length(48.0))
                .radius(24.0);
            let mut tree = UiTree::new(skeleton.build(themes.resolve(scheme)));
            let output = LayoutEngine::new()
                .compute(&mut tree, &mut TextEngine::new(), Size::new(width, 200.0))
                .unwrap();
            assert_eq!(output.nodes[0].bounds.size, Size::new(width * 0.5, 48.0));
            assert!(output.hit_regions.is_empty());
            assert!(
                tree.semantic_tree(&output.semantic_bounds, 1.0)
                    .nodes
                    .is_empty()
            );
        }
    }
}

#[test]
fn pulse_changes_paint_without_changing_layout_and_can_be_stopped() {
    let mut skeleton = Skeleton::new("loading");
    let mut cx = Context::default();
    let initial = skeleton.render(&mut cx);
    assert!(!skeleton.wants_animation_frame());
    let mut tree = UiTree::new(initial.clone());
    tree.advance_animations(Time::from_nanos(1));
    assert_eq!(
        tree.advance_animations(Time::from_nanos(500_000_001)),
        TreeUpdate::Composite
    );
    let layer = argui_paint::LayerStyle::new(Default::default());
    assert_ne!(
        tree.resolved_layer(tree.node_ids()[0], &initial, &layer)
            .opacity,
        1.0
    );
    skeleton.set_animated(false);
    tree.update(skeleton.render(&mut cx));
    assert!(!tree.wants_animation_frame());
    assert_eq!(tree.element_at(0).unwrap().paint.quad.opacity, 1.0);
    skeleton.set_animated(true);
    tree.update(skeleton.render(&mut cx));
    assert!(tree.wants_animation_frame());
}

#[test]
fn reduced_motion_freezes_the_pulse_and_normal_motion_can_resume() {
    let entity = Entity::new(Skeleton::new("reduced"));
    for reduced_motion in [true, false, true] {
        let root = entity.render_in(WindowEnvironment {
            reduced_motion,
            ..WindowEnvironment::default()
        });
        let tree = UiTree::new(root.clone());
        assert_eq!(tree.wants_animation_frame(), !reduced_motion);
        if reduced_motion {
            assert_eq!(root.paint.quad.opacity, 1.0);
        }
    }
}

#[test]
fn invalid_radii_are_rejected() {
    for radius in [-1.0, f32::NAN, f32::INFINITY] {
        assert!(std::panic::catch_unwind(|| Skeleton::new("bad").radius(radius)).is_err());
    }
}
