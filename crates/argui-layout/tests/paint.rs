use argui_core::{Point, Size};
#[path = "paint/effects.rs"]
mod effects;
#[path = "paint/geometry.rs"]
mod geometry;
use argui_layout::LayoutEngine;
use argui_paint::{Border, Color, DisplayCommand, ImageFit, LayerStyle, VectorId};
use argui_text::TextEngine;
use argui_ui::{
    Axes, CornerRadii, CursorIcon, Element, HitTestStyle, ImageId, Interaction, Overflow,
    PointerEvents, StylePatch, Transform2D, TransformOrigin, UiTree, VisualState, length, property,
};

const NOTO_SANS: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");

fn text_engine() -> TextEngine {
    TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans")
}

fn effect() -> LayerStyle {
    LayerStyle::new(Default::default()).opacity(0.8)
}

#[test]
fn plain_elements_keep_one_combined_quad_and_no_layers() {
    let root = Element::text("Fast path")
        .background(Color::srgb(0.1, 0.2, 0.3))
        .border(Border::all(2.0, Color::WHITE));
    let mut ui = UiTree::new(root);
    let output = LayoutEngine::new()
        .compute(&mut ui, &mut text_engine(), Size::new(240.0, 80.0))
        .unwrap();

    assert_eq!(output.display_list.quad_count(), 1);
    assert!(matches!(
        output.display_list.commands(),
        [
            DisplayCommand::Quad(_),
            DisplayCommand::Text { block: 0, .. }
        ]
    ));
}

#[test]
fn pointer_event_policy_controls_own_and_descendant_regions() {
    let output_for = |pointer_events| {
        let child = Element::container([])
            .width(length(20.0))
            .height(length(20.0))
            .interaction(Interaction::default());
        let root = Element::container([child])
            .width(length(40.0))
            .height(length(40.0))
            .interaction(Interaction::default())
            .hit_test(HitTestStyle::default().pointer_events(pointer_events));
        let mut ui = UiTree::new(root);
        LayoutEngine::new()
            .compute(&mut ui, &mut text_engine(), Size::new(100.0, 100.0))
            .unwrap()
            .hit_regions
    };

    assert_eq!(output_for(PointerEvents::Auto).len(), 2);
    assert_eq!(output_for(PointerEvents::None).len(), 0);
    assert_eq!(output_for(PointerEvents::BoxOnly).len(), 1);
    assert_eq!(output_for(PointerEvents::ContentsOnly).len(), 1);
}

#[test]
fn paint_only_text_color_updates_without_reshaping() {
    let root = Element::text("Retained glyphs")
        .width(length(140.0))
        .height(length(30.0))
        .interaction(Interaction::default())
        .when(
            VisualState::Hovered,
            StylePatch::new().set(property::TextColor, Color::srgb(0.9, 0.2, 0.3)),
        );
    let mut ui = UiTree::new(root);
    let mut layout = LayoutEngine::new();
    let mut output = layout
        .compute(&mut ui, &mut text_engine(), Size::new(200.0, 80.0))
        .unwrap();
    let bounds = output.text.blocks()[0].bounds;
    assert!(!layout.repaint(&ui, &mut output));

    let update = ui.pointer_moved(Point::new(10.0, 10.0), &output.hit_regions);
    assert!(update.paint_changed);
    assert!(layout.repaint(&ui, &mut output));
    assert!(!layout.repaint(&ui, &mut output));

    assert_eq!(output.text.blocks()[0].bounds, bounds);
    assert_eq!(
        output.text.blocks()[0].style.color,
        Color::srgb(0.9, 0.2, 0.3)
    );
}

#[test]
fn background_border_and_text_scopes_wrap_only_their_primitive() {
    let root = Element::text("Scoped")
        .background(Color::srgb(0.1, 0.2, 0.3))
        .border(Border::all(2.0, Color::WHITE))
        .radius(CornerRadii::all(8.0))
        .background_effect(effect())
        .border_effect(effect())
        .text_effect(effect());
    let mut ui = UiTree::new(root);
    let output = LayoutEngine::new()
        .compute(&mut ui, &mut text_engine(), Size::new(240.0, 80.0))
        .unwrap();
    let commands = output.display_list.commands();

    assert_eq!(output.display_list.quad_count(), 2);
    assert_eq!(commands.len(), 9);
    assert!(matches!(commands[0], DisplayCommand::BeginLayer(_)));
    assert!(matches!(
        &commands[1],
        DisplayCommand::Quad(quad) if quad.background.is_some()
            && quad.border.widths.left == 0.0
    ));
    assert!(matches!(commands[2], DisplayCommand::EndLayer));
    assert!(matches!(commands[3], DisplayCommand::BeginLayer(_)));
    assert!(matches!(
        &commands[4],
        DisplayCommand::Quad(quad) if quad.background.is_none()
            && quad.border.widths.left == 2.0
    ));
    assert!(matches!(commands[5], DisplayCommand::EndLayer));
    assert!(matches!(commands[6], DisplayCommand::BeginLayer(_)));
    assert!(matches!(commands[7], DisplayCommand::Text { block: 0, .. }));
    assert!(matches!(commands[8], DisplayCommand::EndLayer));
    assert!(output.display_list.validate().is_ok());
}

#[test]
fn scoped_effects_do_not_fabricate_missing_backgrounds_or_borders() {
    for background in [false, true] {
        let root = Element::container([])
            .width(length(40.0))
            .height(length(30.0))
            .background_effect(effect())
            .border_effect(effect());
        let root = if background {
            root.background(Color::WHITE)
        } else {
            root.border(Border::all(2.0, Color::WHITE))
        };
        let mut ui = UiTree::new(root);
        let output = LayoutEngine::new()
            .compute(&mut ui, &mut text_engine(), Size::new(100.0, 100.0))
            .unwrap();
        assert_eq!(output.display_list.quad_count(), 1);
        let commands = output.display_list.commands();
        assert_eq!(commands.len(), 3);
        assert!(matches!(commands[0], DisplayCommand::BeginLayer(_)));
        let DisplayCommand::Quad(quad) = &commands[1] else {
            panic!("missing scoped quad");
        };
        assert_eq!(quad.background.is_some(), background);
        assert_eq!(quad.border.widths.left, if background { 0.0 } else { 2.0 });
        assert!(matches!(commands[2], DisplayCommand::EndLayer));
        assert!(output.display_list.validate().is_ok());
    }
}

#[test]
fn whole_and_content_scopes_nest_around_children_in_stable_order() {
    let root = Element::container([Element::text("Child")])
        .whole_effect(effect())
        .content_effect(effect());
    let mut ui = UiTree::new(root);
    let output = LayoutEngine::new()
        .compute(&mut ui, &mut text_engine(), Size::new(240.0, 80.0))
        .unwrap();

    assert!(matches!(
        output.display_list.commands(),
        [
            DisplayCommand::BeginLayer(_),
            DisplayCommand::BeginLayer(_),
            DisplayCommand::Text { block: 0, .. },
            DisplayCommand::EndLayer,
            DisplayCommand::EndLayer
        ]
    ));
    assert!(output.display_list.validate().is_ok());
}

#[test]
fn transformed_images_share_exact_clips_layers_and_hit_geometry() {
    let image = Element::image(ImageId(42))
        .image_fit(ImageFit::Contain)
        .width(length(80.0))
        .height(length(40.0))
        .interaction(
            Interaction::default()
                .focus_policy(argui_ui::FocusPolicy::TabStop)
                .cursor(CursorIcon::Crosshair),
        )
        .transform(Transform2D::IDENTITY.translate(20.0, 10.0).rotate(0.1))
        .transform_origin(TransformOrigin::TOP_LEFT)
        .layer(effect());
    let root = Element::container([image])
        .width(length(100.0))
        .height(length(80.0))
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Hidden,
        });
    let mut ui = UiTree::new(root);
    let output = LayoutEngine::new()
        .compute(&mut ui, &mut text_engine(), Size::new(200.0, 120.0))
        .unwrap();

    assert!(matches!(
        output.display_list.commands(),
        [
            DisplayCommand::BeginLayer(_),
            DisplayCommand::Image(_),
            DisplayCommand::EndLayer
        ]
    ));
    let DisplayCommand::Image(image) = &output.display_list.commands()[1] else {
        unreachable!();
    };
    assert_eq!(image.image, ImageId(42));
    assert_eq!(image.fit, ImageFit::Contain);
    assert_eq!(image.clips.regions().len(), 2);
    let hit = &output.hit_regions[0];
    assert_eq!(hit.cursor, CursorIcon::Crosshair);
    let visual_point = image.transform.transform_point(Point::new(10.0, 10.0));
    assert!(hit.contains(visual_point));
    assert!(!hit.contains(Point::new(190.0, 110.0)));
}

#[test]
fn disabled_interactions_keep_cursor_hit_geometry_but_reject_input() {
    let root = Element::container([])
        .width(length(120.0))
        .height(length(40.0))
        .interaction(
            Interaction::default()
                .enabled(false)
                .focus_policy(argui_ui::FocusPolicy::TabStop)
                .cursor(CursorIcon::NotAllowed),
        );
    let mut ui = UiTree::new(root);
    let output = LayoutEngine::new()
        .compute(&mut ui, &mut text_engine(), Size::new(120.0, 40.0))
        .unwrap();

    assert!(matches!(
        output.hit_regions.as_slice(),
        [region]
            if !region.enabled
                && !region.focus_policy.is_focusable()
                && region.cursor == CursorIcon::NotAllowed
    ));
}

#[test]
fn vectors_lower_to_the_shared_clipped_transformed_display_list() {
    let vector = Element::vector(VectorId(7))
        .vector_fit(ImageFit::Contain)
        .vector_color(Color::srgb(0.2, 0.4, 0.6))
        .paint_opacity(0.6)
        .width(length(24.0))
        .height(length(24.0));
    let mut ui = UiTree::new(
        Element::container([vector])
            .width(length(40.0))
            .height(length(40.0))
            .overflow(Axes {
                x: Overflow::Hidden,
                y: Overflow::Hidden,
            }),
    );
    let output = LayoutEngine::new()
        .compute(&mut ui, &mut text_engine(), Size::new(80.0, 80.0))
        .unwrap();
    let DisplayCommand::Vector(vector) = &output.display_list.commands()[0] else {
        panic!("vector element must lower to a vector command");
    };
    assert_eq!(vector.vector, VectorId(7));
    assert_eq!(vector.fit, ImageFit::Contain);
    assert_eq!(vector.color, Color::srgb(0.2, 0.4, 0.6));
    assert_eq!(vector.opacity, 0.6);
    assert_eq!(vector.clips.regions().len(), 2);
}

#[test]
fn cached_rows_keep_their_text_when_preceding_text_is_added_or_removed() {
    let row = Element::row([Element::text("Item 1")])
        .keyed("row")
        .width(length(200.0))
        .height(length(40.0));
    let view = |show_label| {
        Element::column([
            if show_label {
                Element::text("Header")
            } else {
                Element::container([])
            }
            .keyed("header")
            .height(length(30.0)),
            row.clone(),
        ])
        .width(length(200.0))
        .height(length(100.0))
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        })
    };
    let mut ui = UiTree::new(view(false));
    let mut engine = LayoutEngine::new();
    let mut text = text_engine();
    engine
        .compute(&mut ui, &mut text, Size::new(300.0, 150.0))
        .unwrap();
    for show_label in [true, false, true] {
        ui.update(view(show_label));
        let mut warm = engine
            .compute(&mut ui, &mut text, Size::new(300.0, 150.0))
            .unwrap();
        let cold = LayoutEngine::new()
            .compute(&mut ui, &mut text, Size::new(300.0, 150.0))
            .unwrap();
        assert_eq!(warm.text, cold.text);
        assert_eq!(warm.display_list, cold.display_list);
        engine.repaint(&ui, &mut warm);
        assert_eq!(warm.display_list, cold.display_list);
        assert!(warm.paint_stats.reused_subtrees > 0);
    }
}
