use argui_core::{Point, Size};
use argui_layout::LayoutEngine;
use argui_paint::{Border, ClipBehavior, Color, DisplayCommand, ImageFit, LayerStyle, VectorId};
use argui_text::TextEngine;
use argui_ui::{
    CornerRadii, Element, ImageId, Interaction, Length, Transform2D, TransformOrigin, UiTree,
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
        .background(Color::rgb(0.1, 0.2, 0.3))
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
fn background_border_and_text_scopes_wrap_only_their_primitive() {
    let root = Element::text("Scoped")
        .background(Color::rgb(0.1, 0.2, 0.3))
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
        .width(Length::Px(80.0))
        .height(Length::Px(40.0))
        .interaction(Interaction::default().focusable(true))
        .transform(Transform2D::IDENTITY.translate(20.0, 10.0).rotate(0.1))
        .transform_origin(TransformOrigin::TOP_LEFT)
        .layer(effect());
    let root = Element::container([image])
        .width(Length::Px(100.0))
        .height(Length::Px(80.0))
        .clip(ClipBehavior::Bounds);
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
    let visual_point = image.transform.transform_point(Point::new(10.0, 10.0));
    assert!(hit.contains(visual_point));
    assert!(!hit.contains(Point::new(190.0, 110.0)));
}

#[test]
fn vectors_lower_to_the_shared_clipped_transformed_display_list() {
    let vector = Element::vector(VectorId(7))
        .vector_progress(0.35)
        .paint_opacity(0.6)
        .width(Length::Px(24.0))
        .height(Length::Px(24.0));
    let mut ui = UiTree::new(
        Element::container([vector])
            .width(Length::Px(40.0))
            .height(Length::Px(40.0))
            .clip(ClipBehavior::Bounds),
    );
    let output = LayoutEngine::new()
        .compute(&mut ui, &mut text_engine(), Size::new(80.0, 80.0))
        .unwrap();
    let DisplayCommand::Vector(vector) = &output.display_list.commands()[0] else {
        panic!("vector element must lower to a vector command");
    };
    assert_eq!(vector.vector, VectorId(7));
    assert_eq!(vector.progress, 0.35);
    assert_eq!(vector.opacity, 0.6);
    assert_eq!(vector.clips.regions().len(), 2);
}
