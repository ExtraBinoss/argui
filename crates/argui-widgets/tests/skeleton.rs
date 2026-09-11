use argui_animation::{Duration, Frame, Time};
use argui_core::{Color, ColorScheme, Size};
use argui_layout::LayoutEngine;
use argui_runtime::{Context, Entity, Render, WindowEnvironment};
use argui_text::TextEngine;
use argui_ui::{UiTree, length, percent};
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
    let frame = Frame {
        now: Time::ZERO,
        elapsed: Duration::from_millis(500),
    };
    assert!(skeleton.wants_animation_frame());
    skeleton.animation_frame(frame, &mut cx);
    let pulsed = skeleton.render(&mut cx);
    assert_ne!(initial.paint.quad.opacity, pulsed.paint.quad.opacity);
    assert_eq!(initial.style, pulsed.style);
    skeleton.set_animated(false);
    assert!(!skeleton.wants_animation_frame());
    skeleton.animation_frame(frame, &mut cx);
    assert_eq!(skeleton.render(&mut cx).paint.quad.opacity, 1.0);
    skeleton.set_animated(true);
    assert!(skeleton.wants_animation_frame());
}

#[test]
fn reduced_motion_freezes_the_pulse_and_normal_motion_can_resume() {
    let entity = Entity::new(Skeleton::new("reduced"));
    for reduced_motion in [true, false, true] {
        let root = entity.render_in(WindowEnvironment {
            reduced_motion,
            ..WindowEnvironment::default()
        });
        entity.read(|skeleton| assert_eq!(skeleton.wants_animation_frame(), !reduced_motion));
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
