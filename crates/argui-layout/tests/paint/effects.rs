use argui_core::{Point, Rect, Size};
use argui_layout::{LayoutEngine, LayoutOutput};
use argui_paint::{
    Color, DisplayCommand, EffectId, EffectInstance, EffectValue, Filter, LayerStyle, QuadStyle,
};
use argui_text::TextEngine;
use argui_ui::{
    Axes, Element, Overflow, ScrollConfig, ScrollEffect, ScrollMetric, ScrollbarGutter,
    ScrollbarPartStyle, ScrollbarStyle, ScrollbarVisibility, Sides, UiTree, length,
};

fn effect(name: &'static str) -> ScrollEffect {
    ScrollEffect::new(
        LayerStyle::new(Rect::default()).filter(Filter::Effect(EffectInstance::new(
            EffectId::new(name),
            [("edges", EffectValue::Vec4([0.0; 4]))],
        ))),
    )
    .bind(
        0,
        "edges",
        ScrollMetric::Edges {
            strengths: [1.0; 4],
            threshold: 0.0,
            ramp: 12.0,
        },
    )
}

fn content(height: f32, gutter: ScrollbarGutter, both: bool) -> Element {
    let bar = ScrollbarStyle::new(
        ScrollbarPartStyle::new(QuadStyle::solid(Color::BLACK)),
        ScrollbarPartStyle::new(QuadStyle::solid(Color::WHITE)),
    )
    .visibility(ScrollbarVisibility::Always)
    .width(8.0)
    .insets(Sides::length(2.0));
    Element::column([Element::container([])
        .height(length(height))
        .width(length(if both { 300.0 } else { 50.0 }))
        .shrink(0.0)
        .background(Color::WHITE)])
    .keyed("scroll")
    .width(length(100.0))
    .height(length(80.0))
    .background(Color::BLACK)
    .overflow(Axes {
        x: if both {
            Overflow::Auto
        } else {
            Overflow::Hidden
        },
        y: Overflow::Auto,
    })
    .scroll_config(
        ScrollConfig::default()
            .scrollbar(bar)
            .effect(effect("test.first"))
            .effect(effect("test.second")),
    )
    .scrollbar_gutter(gutter)
}

fn layers(output: &LayoutOutput) -> Vec<&LayerStyle> {
    output
        .display_list
        .commands()
        .iter()
        .filter_map(|cmd| match cmd {
            DisplayCommand::BeginLayer(layer) => Some(layer),
            _ => None,
        })
        .collect()
}

#[test]
fn scroll_layers_exclude_background_and_scrollbars_and_preserve_effect_order() {
    let mut ui = UiTree::new(content(300.0, ScrollbarGutter::Stable, false));
    let mut text = TextEngine::from_embedded_fonts(
        [include_bytes!("../../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf").as_slice()],
        "Noto Sans",
        "Noto Sans",
        "Noto Sans",
    );
    let mut engine = LayoutEngine::new();
    let mut output = engine
        .compute(&mut ui, &mut text, Size::new(100.0, 80.0))
        .unwrap();
    let commands = output.display_list.commands();
    assert!(matches!(commands[0], DisplayCommand::Quad(_)));
    assert!(matches!(
        commands[commands.len() - 1],
        DisplayCommand::Quad(_)
    ));
    assert!(matches!(
        commands[commands.len() - 2],
        DisplayCommand::Quad(_)
    ));
    let initial = layers(&output);
    assert_eq!(initial.len(), 2);
    assert_eq!(initial[0].bounds.size.width, 100.0);
    let Filter::Effect(filter) = &initial[0].filters[0] else {
        panic!()
    };
    assert_eq!(filter.id, EffectId::new("test.second"));
    assert_eq!(
        filter.parameters[0].value,
        EffectValue::Vec4([0.0, 0.0, 0.0, 1.0])
    );
    let revision = ui.revision();
    let id = output.scroll_regions[0].node;
    for offset in [6.0, 12.0, 100.0, 220.0] {
        ui.set_scroll_offset(id, Point::new(0.0, offset));
        engine.apply_scroll(&ui, &mut output).unwrap();
        assert_eq!(ui.revision(), revision);
        let Filter::Effect(filter) = &layers(&output)[0].filters[0] else {
            panic!()
        };
        let EffectValue::Vec4(edges) = filter.parameters[0].value else {
            panic!()
        };
        assert_eq!(edges[1], if offset == 6.0 { 0.5 } else { 1.0 });
        assert_eq!(edges[3], if offset == 220.0 { 0.0 } else { 1.0 });
    }
}

#[test]
fn fitting_content_skips_effects_and_active_effects_cover_the_full_viewport() {
    let mut text = TextEngine::from_embedded_fonts(
        [include_bytes!("../../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf").as_slice()],
        "Noto Sans",
        "Noto Sans",
        "Noto Sans",
    );
    for (height, gutter, both, expected) in [
        (30.0, ScrollbarGutter::Auto, false, 0),
        (300.0, ScrollbarGutter::Auto, false, 2),
        (300.0, ScrollbarGutter::Stable, true, 2),
    ] {
        let mut ui = UiTree::new(content(height, gutter, both));
        let output = LayoutEngine::new()
            .compute(&mut ui, &mut text, Size::new(100.0, 80.0))
            .unwrap();
        let layers = layers(&output);
        assert_eq!(layers.len(), expected);
        if both {
            assert_eq!(layers[0].bounds.size, Size::new(100.0, 80.0));
        }
    }
}

#[test]
fn horizontal_axes_survive_repaint_and_resizing_removes_obsolete_effects() {
    let mut root = content(30.0, ScrollbarGutter::Stable, true);
    root.style.overflow.y = Overflow::Hidden;
    let mut ui = UiTree::new(root.clone());
    let mut text = TextEngine::new();
    let mut engine = LayoutEngine::new();
    let mut output = engine
        .compute(&mut ui, &mut text, Size::new(400.0, 100.0))
        .unwrap();
    assert_eq!(
        output.scroll_regions[0].config.axes,
        argui_ui::ScrollAxes::Horizontal
    );
    assert_eq!(layers(&output)[0].bounds.size, Size::new(100.0, 80.0));
    let node = output.scroll_regions[0].node;
    ui.set_scroll_offset(node, Point::new(6.0, 50.0));
    engine.apply_scroll(&ui, &mut output).unwrap();
    let Filter::Effect(filter) = &layers(&output)[0].filters[0] else {
        panic!()
    };
    assert_eq!(
        filter.parameters[0].value,
        EffectValue::Vec4([0.5, 0.0, 1.0, 0.0])
    );
    root.style.size.width = length(400.0);
    ui.update(root);
    output = engine
        .compute(&mut ui, &mut text, Size::new(400.0, 100.0))
        .unwrap();
    assert!(layers(&output).is_empty());
}

#[test]
fn rounded_content_keeps_clips_and_portals_escape_scroll_layers() {
    let mut root =
        content(300.0, ScrollbarGutter::Stable, false).radius(argui_ui::CornerRadii::all(12.0));
    root.children.push(
        Element::container([])
            .width(length(30.0))
            .height(length(20.0))
            .background(Color::srgb(1.0, 0.0, 0.0))
            .portal(argui_ui::WindowLayer::Popover),
    );
    let mut ui = UiTree::new(root);
    let output = LayoutEngine::new()
        .compute(&mut ui, &mut TextEngine::new(), Size::new(100.0, 80.0))
        .unwrap();
    let mut depth = 0;
    let mut saw_clipped = false;
    let mut saw_portal = false;
    for command in output.display_list.commands() {
        match command {
            DisplayCommand::BeginLayer(_) => depth += 1,
            DisplayCommand::EndLayer => depth -= 1,
            DisplayCommand::Quad(quad) if depth > 0 => {
                assert!(
                    quad.clips
                        .regions()
                        .iter()
                        .any(|clip| clip.radii == argui_ui::CornerRadii::all(12.0))
                );
                saw_clipped = true;
            }
            DisplayCommand::Quad(quad)
                if quad.background
                    == Some(argui_paint::Fill::Solid(Color::srgb(1.0, 0.0, 0.0))) =>
            {
                assert_eq!(depth, 0);
                saw_portal = true;
            }
            _ => {}
        }
    }
    assert!(saw_clipped && saw_portal);
    assert_eq!(depth, 0);
}

/// A descendant backdrop filter inherits the scroll viewport's surface clip.
#[test]
fn scrolling_backdrop_layer_is_clipped_before_fixed_header() {
    let card = Element::container([])
        .width(length(100.0))
        .height(length(120.0))
        .layer(LayerStyle::new(Rect::default()).backdrop(Filter::Blur(8.0)));
    let scroll = Element::column([card])
        .width(length(100.0))
        .height(length(80.0))
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        });
    let root = Element::column([
        Element::container([])
            .width(length(100.0))
            .height(length(32.0)),
        scroll,
    ]);
    let mut ui = UiTree::new(root);
    let mut engine = LayoutEngine::new();
    let mut output = engine
        .compute(&mut ui, &mut TextEngine::new(), Size::new(100.0, 112.0))
        .unwrap();
    let node = output.scroll_regions[0].node;
    for offset in [0.0, 20.0, 40.0] {
        ui.set_scroll_offset(node, Point::new(0.0, offset));
        engine.apply_scroll(&ui, &mut output).unwrap();
        let filtered = output
            .display_list
            .commands()
            .iter()
            .find_map(|command| match command {
                DisplayCommand::BeginLayer(layer) if !layer.backdrop_filters.is_empty() => {
                    Some(layer)
                }
                _ => None,
            })
            .unwrap();
        assert_eq!(
            filtered.clip,
            Some(Rect::new(Point::new(0.0, 32.0), Size::new(100.0, 80.0)))
        );
    }
}
