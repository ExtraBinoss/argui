use argui_animation::{Duration, Motion, Time, Tween};
use argui_core::{Affine2D, Point, Size, Transform2D};
use argui_layout::LayoutEngine;
use argui_paint::{CompositorId, DisplayCommand, LayerStyle};
use argui_text::TextEngine;
use argui_ui::{
    CaretStyle, Color, Element, FocusPolicy, FocusRequest, Interaction, TextEditorSpec,
    TextInputFilter, TreeUpdate, UiTree, length, property,
};

fn scene(transform: Transform2D, opacity: f32) -> Element {
    Element::container([Element::container([])
        .keyed("target")
        .width(length(20.0))
        .height(length(20.0))
        .interaction(Interaction::default())])
    .keyed("moving")
    .width(length(80.0))
    .height(length(40.0))
    .background(Color::WHITE)
    .transform(transform)
    .layer(LayerStyle::new(Default::default()).opacity(opacity))
}

#[test]
fn composition_updates_retained_layers_and_descendant_geometry_without_repaint() {
    let mut ui = UiTree::new(scene(Transform2D::IDENTITY, 0.8));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    let mut output = engine
        .compute(&mut ui, &mut text, Size::new(240.0, 120.0))
        .unwrap();
    let child = ui.node_id_at(1).expect("child node");
    let retained_quad = output
        .display_list
        .commands()
        .iter()
        .find_map(|command| match command {
            DisplayCommand::Quad(quad) => Some(quad.clone()),
            _ => None,
        })
        .expect("retained quad");
    let retained_text = output.text.clone();

    assert_eq!(
        ui.update(scene(Transform2D::IDENTITY.translate(36.0, 8.0), 0.4,)),
        TreeUpdate::Composite
    );
    assert!(engine.composite(&ui, &mut output));

    let compositor = output
        .display_list
        .commands()
        .iter()
        .find_map(|command| match command {
            DisplayCommand::BeginCompositor(layer) => Some(layer),
            _ => None,
        })
        .expect("compositor layer");
    assert_eq!(compositor.transform, Affine2D::translation(36.0, 8.0));
    assert_eq!(compositor.opacity, 0.4);
    assert_eq!(output.text, retained_text);
    assert_eq!(
        output
            .display_list
            .commands()
            .iter()
            .find_map(|command| match command {
                DisplayCommand::Quad(quad) => Some(quad),
                _ => None,
            }),
        Some(&retained_quad)
    );

    let hit = output
        .hit_regions
        .iter()
        .find(|region| region.node == child)
        .expect("child hit region");
    assert_eq!(hit.transform, Affine2D::translation(36.0, 8.0));
    assert!(hit.contains(Point::new(40.0, 12.0)));
}

#[test]
fn singular_compositor_transforms_request_the_safe_paint_fallback() {
    let mut ui = UiTree::new(scene(Transform2D::IDENTITY, 0.8));
    let mut engine = LayoutEngine::new();
    let mut output = engine
        .compute(&mut ui, &mut TextEngine::new(), Size::new(240.0, 120.0))
        .unwrap();

    assert_eq!(
        ui.update(scene(Transform2D::IDENTITY.scale(0.0, 1.0), 0.8)),
        TreeUpdate::Composite
    );
    assert!(!engine.composite(&ui, &mut output));
}

#[test]
fn moving_a_partially_clipped_layer_requests_the_safe_paint_fallback() {
    let mut ui = UiTree::new(scene(Transform2D::IDENTITY.translate(-20.0, 0.0), 0.8));
    let mut engine = LayoutEngine::new();
    let mut output = engine
        .compute(&mut ui, &mut TextEngine::new(), Size::new(240.0, 120.0))
        .unwrap();

    assert_eq!(
        ui.update(scene(Transform2D::IDENTITY, 0.8)),
        TreeUpdate::Composite
    );
    assert!(!engine.composite(&ui, &mut output));
}

#[test]
fn moving_an_offscreen_layer_without_revealing_it_stays_composition_only() {
    let mut ui = UiTree::new(scene(Transform2D::IDENTITY.translate(-100.0, 0.0), 0.8));
    let mut engine = LayoutEngine::new();
    let mut output = engine
        .compute(&mut ui, &mut TextEngine::new(), Size::new(240.0, 120.0))
        .unwrap();

    assert_eq!(
        ui.update(scene(Transform2D::IDENTITY.translate(-90.0, 0.0), 0.8,)),
        TreeUpdate::Composite
    );
    assert!(engine.composite(&ui, &mut output));
}

#[test]
fn rotating_a_fully_retained_layer_stays_composition_only() {
    let mut ui = UiTree::new(scene(Transform2D::IDENTITY, 0.8));
    let mut engine = LayoutEngine::new();
    let mut output = engine
        .compute(&mut ui, &mut TextEngine::new(), Size::new(240.0, 120.0))
        .unwrap();

    assert_eq!(
        ui.update(scene(
            Transform2D::IDENTITY.rotate(std::f32::consts::FRAC_PI_4),
            0.8,
        )),
        TreeUpdate::Composite
    );
    assert!(engine.composite(&ui, &mut output));
}

#[test]
fn layer_opacity_binding_promotes_plain_content_without_an_effect_layer() {
    let opacity = Motion::new(1.0_f32);
    let element = Element::container([])
        .width(length(80.0))
        .height(length(40.0))
        .background(Color::WHITE)
        .bind(property::LayerOpacity, opacity.clone());
    let mut ui = UiTree::new(element);
    let mut engine = LayoutEngine::new();
    let mut output = engine
        .compute(&mut ui, &mut TextEngine::new(), Size::new(240.0, 120.0))
        .unwrap();
    assert!(
        output
            .display_list
            .commands()
            .iter()
            .any(|command| matches!(command, DisplayCommand::BeginCompositor(_)))
    );
    assert!(
        output
            .display_list
            .commands()
            .iter()
            .all(|command| !matches!(command, DisplayCommand::BeginLayer(_)))
    );

    opacity.animate_to(0.4, Tween::new(Duration::from_millis(100)));
    ui.advance_animations(Time::from_nanos(1));
    assert_eq!(
        ui.advance_animations(Time::from_nanos(100_000_001)),
        TreeUpdate::Composite
    );
    assert!(engine.composite(&ui, &mut output));
    assert_eq!(
        output
            .display_list
            .compositor_layers()
            .next()
            .expect("promoted compositor")
            .opacity,
        0.4
    );
}

#[test]
fn caret_blink_updates_its_retained_layer_without_repainting_text() {
    let input = Element::text_editor(TextEditorSpec {
        value: "hello".to_owned(),
        placeholder: String::new(),
        multiline: false,
        read_only: false,
        filter: TextInputFilter::Any,
        text: Default::default(),
        placeholder_text: Default::default(),
        selection: Color::WHITE,
        caret: CaretStyle::default(),
    })
    .keyed("editor")
    .width(length(180.0))
    .height(length(32.0))
    .interaction(Interaction::default().focus_policy(FocusPolicy::TabStop));
    let mut ui = UiTree::new(input);
    let node = ui.node_id_at(0).unwrap();
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    let mut output = engine
        .compute(&mut ui, &mut text, Size::new(240.0, 80.0))
        .unwrap();
    ui.sync_focus(
        &output.hit_regions,
        Some(FocusRequest::Focus("editor".into())),
    );
    engine.update_text_inputs(&mut ui, &mut text, &mut output);
    let retained_text = output.text.clone();

    assert_eq!(
        ui.advance_animations(Time::from_nanos(1)),
        TreeUpdate::Composite
    );
    assert!(engine.composite(&ui, &mut output));
    assert_eq!(
        ui.advance_animations(Time::from_nanos(500_000_001)),
        TreeUpdate::Composite
    );
    assert!(engine.composite(&ui, &mut output));

    let caret = output
        .display_list
        .compositor_layers()
        .find(|layer| layer.id == CompositorId::subpart(node.get(), 1))
        .expect("retained caret layer");
    assert_eq!(caret.opacity, 0.0);
    assert_eq!(output.text, retained_text);
}

/// A transform's terminal paint rebases text commands instead of retaining scaled bitmaps.
#[test]
fn settled_text_transform_repaints_at_the_current_physical_scale() {
    let motion = Motion::new(Transform2D::IDENTITY);
    let mut ui = UiTree::new(
        Element::text("Crisp at rest")
            .width(length(120.0))
            .height(length(32.0))
            .bind(property::Transform, motion.clone()),
    );
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    let mut output = engine
        .compute(&mut ui, &mut text, Size::new(360.0, 120.0))
        .unwrap();
    let terminal = Transform2D::IDENTITY.scale(1.5, 1.5).translate(0.25, 0.0);
    motion.animate_to(terminal, Tween::new(Duration::from_millis(100)));
    ui.advance_animations(Time::from_nanos(1));
    assert_eq!(
        ui.advance_animations(Time::from_nanos(50_000_001)),
        TreeUpdate::Composite
    );
    assert!(engine.composite(&ui, &mut output));
    assert_ne!(
        output
            .display_list
            .compositor_layers()
            .next()
            .unwrap()
            .transform,
        Affine2D::IDENTITY
    );
    assert_eq!(
        ui.advance_animations(Time::from_nanos(100_000_001)),
        TreeUpdate::Paint
    );
    engine.repaint(&ui, &mut output);

    let layer = output.display_list.compositor_layers().next().unwrap();
    assert_eq!(layer.transform, Affine2D::IDENTITY);
    assert_eq!(layer.base_transform.matrix, [1.5, 0.0, 0.0, 1.5]);
    let text_transform = output
        .display_list
        .commands()
        .iter()
        .find_map(|command| match command {
            DisplayCommand::Text { transform, .. } => Some(*transform),
            _ => None,
        })
        .unwrap();
    assert_eq!(text_transform, layer.base_transform);
    assert!(!ui.wants_animation_frame());
}
