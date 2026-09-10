use argui_animation::{Duration, Frame, Time};
use argui_core::{Color, ColorScheme, Size};
use argui_layout::LayoutEngine;
use argui_runtime::{Context, Entity, Render, WindowEnvironment};
use argui_text::TextEngine;
use argui_ui::{Role, SemanticValue, UiTree};
use argui_widgets::{Progress, shadcn};

#[test]
fn percentages_clamp_to_the_track_and_unknown_values_remain_indeterminate() {
    let palette = shadcn(Color::WHITE);
    let theme = palette.resolve(ColorScheme::Dark);
    for (value, expected) in [
        (Some(-10.0), Some(0.0)),
        (Some(40.0), Some(40.0)),
        (Some(120.0), Some(100.0)),
        (None, None),
        (Some(f32::NAN), None),
        (Some(f32::INFINITY), None),
    ] {
        let progress = Progress::new("progress", "Upload", value);
        assert_eq!(progress.value(), expected);
        let mut tree = UiTree::new(progress.build(theme));
        let output = LayoutEngine::new()
            .compute(&mut tree, &mut TextEngine::new(), Size::new(200.0, 100.0))
            .unwrap();
        let semantic = tree.semantic_tree(&output.semantic_bounds, 1.0);
        assert_eq!(semantic.nodes.len(), 1);
        let semantics = &semantic.nodes[0].semantics;
        assert_eq!(semantics.role, Role::Progress);
        assert_eq!(semantics.state.busy, expected != Some(100.0));
        assert!(output.hit_regions.is_empty());
        let indicator = output
            .nodes
            .iter()
            .find(|node| tree.key(node.node) == Some("progress::indicator"))
            .unwrap()
            .bounds;
        if let Some(value) = expected {
            assert_eq!(
                semantics.value,
                Some(SemanticValue::Number {
                    value: value as f64,
                    minimum: Some(0.0),
                    maximum: Some(100.0),
                    step: None
                })
            );
            assert!((indicator.size.width - value * 2.0).abs() < 1.0);
        } else {
            assert!(semantics.value.is_none());
            assert_eq!(indicator.size.width, 60.0);
        }
    }
}

#[test]
fn only_mounted_indeterminate_progress_requests_animation_and_reduced_motion_stops_it() {
    let mut progress = Progress::new("progress", "Loading", None);
    let mut cx = Context::default();
    let first = progress.render(&mut cx);
    assert!(progress.wants_animation_frame());
    let frame = Frame {
        now: Time::ZERO,
        elapsed: Duration::from_millis(400),
    };
    progress.animation_frame(frame, &mut cx);
    assert_ne!(
        first.children[0].style.inset,
        progress.render(&mut cx).children[0].style.inset
    );
    progress.set_value(Some(100.0));
    assert!(!progress.wants_animation_frame());
    let complete = progress.render(&mut cx);
    progress.animation_frame(frame, &mut cx);
    assert_eq!(
        complete.children[0].style.inset,
        progress.render(&mut cx).children[0].style.inset
    );
    let entity = Entity::new(Progress::new("reduced", "Loading", None));
    let _ = entity.render_in(WindowEnvironment {
        reduced_motion: true,
        ..WindowEnvironment::default()
    });
    entity.read(|progress| assert!(!progress.wants_animation_frame()));
    let themes = shadcn(Color::WHITE);
    entity.read(|progress| {
        let rendered = progress.build(themes.resolve(ColorScheme::Light));
        assert_eq!(
            rendered.children[0].style.inset.left,
            argui_ui::percent(0.35)
        );
    });
}
