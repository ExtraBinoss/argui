use argui_animation::{Duration, Tween};
use argui_core::{Point, ScrollDelta, Size};
use argui_layout::LayoutEngine;
use argui_paint::{CornerRadii, DisplayCommand, LayerStyle, QuadStyle};
use argui_text::TextEngine;
use argui_ui::{
    Axes, Color, Element, Overflow, ScrollConfig, ScrollbarPartStyle, ScrollbarStyle, Sides,
    StylePatch, StyleTransition, Transition, UiTree, VisualState, length, property,
};

const NOTO_SANS: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");

fn text_engine() -> TextEngine {
    TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans")
}

fn content(scrollbar: ScrollbarStyle) -> Element {
    Element::column([
        Element::container([]).height(length(100.0)).shrink(0.0),
        Element::container([]).height(length(200.0)).shrink(0.0),
    ])
    .keyed("scroll")
    .width(length(200.0))
    .height(length(100.0))
    .overflow(Axes {
        x: Overflow::Hidden,
        y: Overflow::Auto,
    })
    .scroll_config(ScrollConfig::default().scrollbar(scrollbar))
    .layer(LayerStyle::new(Default::default()).opacity(0.8))
}

#[test]
fn scrollbar_is_regular_paint_with_geometry_from_the_scroll_state() {
    let style = ScrollbarStyle::new(
        ScrollbarPartStyle::new(
            QuadStyle::solid(Color::srgb(0.0, 0.0, 0.0)).radius(CornerRadii::all(4.0)),
        ),
        ScrollbarPartStyle::new(QuadStyle::solid(Color::WHITE).radius(CornerRadii::all(4.0))),
    )
    .width(8.0)
    .insets(Sides::length(4.0))
    .min_thumb(20.0);
    let mut ui = UiTree::new(content(style));
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(200.0, 100.0))
        .unwrap();
    let initial = output.scroll_regions[0].scrollbar.as_ref().unwrap();

    assert_eq!(initial.track.size, Size::new(8.0, 92.0));
    assert_eq!(initial.track.origin, Point::new(188.0, 4.0));
    assert!(initial.thumb.size.height > 20.0);
    let initial_thumb_y = initial.thumb.origin.y;
    assert_eq!(output.display_list.quad_count(), 2);
    assert!(matches!(
        output.display_list.commands(),
        [
            DisplayCommand::BeginLayer(_),
            DisplayCommand::Quad(_),
            DisplayCommand::Quad(_),
            DisplayCommand::EndLayer
        ]
    ));

    ui.scroll(
        Point::new(10.0, 10.0),
        ScrollDelta::Pixels(Point::new(0.0, -100.0)),
        &output.scroll_regions,
    );
    layout.apply_scroll(&ui, &mut output).unwrap();
    let moved = output.scroll_regions[0].scrollbar.as_ref().unwrap();
    assert!(moved.thumb.origin.y > initial_thumb_y);
}

#[test]
fn scrollbar_state_transition_repaints_the_resolved_thumb() {
    let hovered = Color::srgb(0.8, 0.3, 0.2);
    let style = ScrollbarStyle::new(
        ScrollbarPartStyle::new(QuadStyle::default()),
        ScrollbarPartStyle::new(QuadStyle::solid(Color::WHITE))
            .when(
                VisualState::Hovered,
                StylePatch::new().set(property::BackgroundColor, hovered),
            )
            .transition(StyleTransition::new(Transition::tween(Tween::new(
                Duration::from_millis(100),
            )))),
    )
    .width(12.0);
    let mut ui = UiTree::new(content(style));
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(200.0, 100.0))
        .unwrap();
    let thumb = output.scroll_regions[0].scrollbar.as_ref().unwrap().thumb;
    let point = Point::new(thumb.origin.x + 2.0, thumb.origin.y + 2.0);
    ui.scrollbar_pointer_moved(Some(point), &output.scroll_regions);
    ui.set_reduced_motion(true);
    layout.repaint(&ui, &mut output);

    assert!(
        output
            .display_list
            .commands()
            .iter()
            .any(|command| matches!(
                command,
                DisplayCommand::Quad(quad)
                    if quad.background == Some(argui_paint::Fill::Solid(hovered))
            )),
        "{:?}",
        output.display_list.commands()
    );
}
