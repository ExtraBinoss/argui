use argui_core::{Color, Point, Size, Transform2D};
use argui_layout::LayoutEngine;
use argui_paint::{CornerRadii, DisplayCommand, Fill};
use argui_text::TextEngine;
use argui_ui::{Axes, DesktopBackdrop, DesktopBackdropState, Element, Overflow, UiTree, length};

#[test]
fn backdrop_regions_follow_rounded_clips_transforms_and_cached_repaints() {
    let panel = Element::container([Element::text("Readable")])
        .keyed("panel")
        .width(length(180.0))
        .height(length(120.0))
        .transform(Transform2D::IDENTITY.translate(12.0, 10.0))
        .radius(CornerRadii::all(12.0))
        .desktop_backdrop(DesktopBackdrop::new(
            Color::BLACK.with_alpha(0.6),
            Color::WHITE,
        ));
    let root = Element::container([panel])
        .width(length(120.0))
        .height(length(90.0))
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Hidden,
        });
    let mut ui = UiTree::new(root.clone());
    let mut engine = LayoutEngine::new();
    let mut output = engine
        .compute(&mut ui, &mut TextEngine::new(), Size::new(300.0, 200.0))
        .unwrap();
    assert_eq!(output.desktop_backdrops.len(), 1);
    let shape = output.desktop_backdrops[0].shape.clone();
    assert!(!shape.contains(Point::new(12.0, 10.0)));
    assert!(shape.contains(Point::new(45.0, 40.0)));
    assert!(!shape.contains(Point::new(150.0, 40.0)));
    let nodes = output.nodes.clone();
    engine.repaint(&ui, &mut output);
    assert!(output.paint_stats.reused_subtrees > 0);
    assert_eq!(output.desktop_backdrops.len(), 1);
    ui.set_desktop_backdrop_state(DesktopBackdropState {
        available: true,
        focused: true,
    });
    engine.repaint(&ui, &mut output);
    assert_eq!(output.nodes, nodes);
    assert_eq!(output.desktop_backdrops[0].shape, shape);
    assert!(output.display_list.commands().iter().any(|command| matches!(command,
        DisplayCommand::Quad(quad) if quad.background == Some(Fill::Solid(Color::BLACK.with_alpha(0.6))))));
    ui.update(Element::container([]));
    let output = engine
        .compute(&mut ui, &mut TextEngine::new(), Size::new(300.0, 200.0))
        .unwrap();
    assert!(output.desktop_backdrops.is_empty());
}

#[test]
fn hidden_backdrops_produce_no_native_regions() {
    let mut ui = UiTree::new(
        Element::container([])
            .display(argui_ui::Display::None)
            .desktop_backdrop(DesktopBackdrop::new(Color::TRANSPARENT, Color::WHITE)),
    );
    let output = LayoutEngine::new()
        .compute(&mut ui, &mut TextEngine::new(), Size::new(320.0, 200.0))
        .unwrap();
    assert!(output.desktop_backdrops.is_empty());
}

#[test]
fn toggling_native_blur_keeps_fallback_paint_without_stale_regions_or_relayout() {
    let tint = Color::BLACK.with_alpha(0.3);
    let fallback = Color::WHITE.with_alpha(0.7);
    let panel = |blur| {
        Element::container([])
            .width(length(180.0))
            .height(length(120.0))
            .desktop_backdrop(DesktopBackdrop::new(tint, fallback).blur(blur))
    };
    let mut ui = UiTree::new(panel(false));
    ui.set_desktop_backdrop_state(DesktopBackdropState {
        available: true,
        focused: true,
    });
    let mut engine = LayoutEngine::new();
    let mut output = engine
        .compute(&mut ui, &mut TextEngine::new(), Size::new(320.0, 200.0))
        .unwrap();
    let nodes = output.nodes.clone();
    for blur in [false, true, false] {
        let update = ui.update(panel(blur));
        assert!(matches!(
            update,
            argui_ui::TreeUpdate::None | argui_ui::TreeUpdate::Paint
        ));
        engine.repaint(&ui, &mut output);
        assert_eq!(output.nodes, nodes);
        assert_eq!(output.desktop_backdrops.len(), usize::from(blur));
        let expected = Some(Fill::Solid(if blur { tint } else { fallback }));
        assert!(
            output
                .display_list
                .commands()
                .iter()
                .any(|command| matches!(command,
            DisplayCommand::Quad(quad) if quad.background == expected))
        );
    }
}
