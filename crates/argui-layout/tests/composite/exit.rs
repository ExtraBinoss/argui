use super::*;

/// Closing a popup keeps its trigger's rotating layer until the return motion settles.
#[test]
fn removing_a_popup_keeps_the_trigger_transform_layer_during_exit() {
    let scene = |open| {
        let chevron = Element::container([Element::text("⌄")])
            .keyed("chevron")
            .width(length(24.0))
            .height(length(24.0))
            .transform(Transform2D::IDENTITY.rotate(if open { std::f32::consts::PI } else { 0.0 }))
            .transition(StyleTransition::new(Transition::tween(Tween::new(
                Duration::from_millis(180),
            ))));
        let mut children = vec![chevron];
        if open {
            children.push(Element::container([]).keyed("popup").height(length(80.0)));
        }
        Element::column(children)
    };
    let mut ui = UiTree::new(scene(true));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    engine
        .compute(&mut ui, &mut text, Size::new(240.0, 120.0))
        .unwrap();

    assert_eq!(ui.update(scene(false)), TreeUpdate::Layout);
    assert_eq!(ui.advance_animations(Time::from_nanos(1)), TreeUpdate::None);
    let mut output = engine
        .compute(&mut ui, &mut text, Size::new(240.0, 120.0))
        .unwrap();
    let chevron = ui.node_id_at(1).unwrap();
    assert!(
        output
            .display_list
            .compositor_layers()
            .any(|layer| layer.id == CompositorId::new(chevron.get()))
    );

    assert_eq!(
        ui.advance_animations(Time::from_nanos(90_000_001)),
        TreeUpdate::Composite
    );
    assert!(engine.composite(&ui, &mut output));
    assert_ne!(
        output
            .display_list
            .compositor_layers()
            .find(|layer| layer.id == CompositorId::new(chevron.get()))
            .unwrap()
            .transform,
        Affine2D::IDENTITY
    );
    assert_eq!(
        ui.advance_animations(Time::from_nanos(180_000_001)),
        TreeUpdate::Paint
    );
    engine.repaint(&ui, &mut output);
    assert!(output.display_list.compositor_layers().next().is_none());
}

/// Opacity transitions keep the same layer when a structural update targets full opacity.
#[test]
fn removing_a_popup_keeps_the_trigger_opacity_layer_during_exit() {
    let scene = |open| {
        let trigger = Element::container([])
            .keyed("trigger")
            .width(length(24.0))
            .height(length(24.0))
            .background(Color::WHITE)
            .layer(LayerStyle::new(Default::default()).opacity(if open { 0.5 } else { 1.0 }))
            .transition(StyleTransition::new(Transition::tween(Tween::new(
                Duration::from_millis(180),
            ))));
        let mut children = vec![trigger];
        if open {
            children.push(Element::container([]).keyed("popup").height(length(80.0)));
        }
        Element::column(children)
    };
    let mut ui = UiTree::new(scene(true));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    engine
        .compute(&mut ui, &mut text, Size::new(240.0, 120.0))
        .unwrap();

    assert_eq!(ui.update(scene(false)), TreeUpdate::Layout);
    assert_eq!(ui.advance_animations(Time::from_nanos(1)), TreeUpdate::None);
    let mut output = engine
        .compute(&mut ui, &mut text, Size::new(240.0, 120.0))
        .unwrap();
    assert!(output.display_list.compositor_layers().next().is_some());
    assert_eq!(
        ui.advance_animations(Time::from_nanos(90_000_001)),
        TreeUpdate::Composite
    );
    assert!(engine.composite(&ui, &mut output));
    let opacity = output
        .display_list
        .compositor_layers()
        .next()
        .unwrap()
        .opacity;
    assert!(opacity > 0.5 && opacity < 1.0);
}

/// Repainting a Switch track must retain its moving thumb's compositor layer.
#[test]
fn parent_paint_change_keeps_a_child_transform_layer_during_exit() {
    let scene = |checked| {
        Element::container([Element::container([])
            .keyed("thumb")
            .width(length(16.0))
            .height(length(16.0))
            .background(Color::WHITE)
            .transform(Transform2D::IDENTITY.translate(if checked { 14.0 } else { 0.0 }, 0.0))
            .transition(StyleTransition::new(Transition::tween(Tween::new(
                Duration::from_millis(150),
            ))))])
        .width(length(32.0))
        .height(length(18.0))
        .background(if checked { Color::WHITE } else { Color::BLACK })
    };
    let mut ui = UiTree::new(scene(true));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    let mut output = engine
        .compute(&mut ui, &mut text, Size::new(240.0, 120.0))
        .unwrap();

    assert_eq!(ui.update(scene(false)), TreeUpdate::Paint);
    ui.advance_animations(Time::from_nanos(1));
    engine.repaint(&ui, &mut output);
    assert!(output.display_list.compositor_layers().next().is_some());
    assert_eq!(
        ui.advance_animations(Time::from_nanos(75_000_001)),
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
}
