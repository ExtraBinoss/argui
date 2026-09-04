use argui_animation::{Duration, Tween};
use argui_core::{Point, ScrollDelta, Size};
use argui_layout::LayoutEngine;
use argui_paint::{CornerRadii, DisplayCommand, LayerStyle, QuadStyle};
use argui_text::TextEngine;
use argui_ui::{
    Axes, Color, Element, Overflow, Position, ScrollAnchoring, ScrollAxes, ScrollConfig,
    ScrollbarPartStyle, ScrollbarStyle, ScrollbarVisibility, Sides, StylePatch, StyleTransition,
    Transition, UiTree, VisualState, length, property,
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
    let initial = output.scroll_regions[0]
        .scrollbar
        .as_ref()
        .unwrap()
        .vertical
        .as_ref()
        .unwrap();

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
    let moved = output.scroll_regions[0]
        .scrollbar
        .as_ref()
        .unwrap()
        .vertical
        .as_ref()
        .unwrap();
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
    let thumb = output.scroll_regions[0]
        .scrollbar
        .as_ref()
        .unwrap()
        .vertical
        .as_ref()
        .unwrap()
        .thumb;
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

#[test]
fn horizontal_scrollbars_use_the_same_retained_geometry() {
    let style = ScrollbarStyle::new(
        ScrollbarPartStyle::new(QuadStyle::default()),
        ScrollbarPartStyle::new(QuadStyle::solid(Color::WHITE)),
    );
    let content = Element::row([Element::container([])
        .width(length(500.0))
        .height(length(40.0))
        .shrink(0.0)])
    .width(length(200.0))
    .height(length(80.0))
    .overflow(Axes {
        x: Overflow::Auto,
        y: Overflow::Hidden,
    })
    .scroll_config(
        ScrollConfig::default()
            .axes(ScrollAxes::Horizontal)
            .scrollbar(style),
    );
    let mut ui = UiTree::new(content);
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let output = layout
        .compute(&mut ui, &mut text, Size::new(200.0, 80.0))
        .unwrap();
    let scrollbar = output.scroll_regions[0].scrollbar.as_ref().unwrap();
    assert!(scrollbar.horizontal.is_some());
    assert!(scrollbar.vertical.is_none());
}

#[test]
fn hidden_and_geometry_free_scrollbars_do_not_paint() {
    let parts = || {
        ScrollbarStyle::new(
            ScrollbarPartStyle::new(QuadStyle::default()),
            ScrollbarPartStyle::new(QuadStyle::solid(Color::WHITE)),
        )
    };
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut hidden = UiTree::new(content(parts().visibility(ScrollbarVisibility::Hidden)));
    let hidden_output = layout
        .compute(&mut hidden, &mut text, Size::new(200.0, 100.0))
        .unwrap();
    assert!(hidden_output.scroll_regions[0].scrollbar.is_none());
    assert_eq!(hidden_output.display_list.quad_count(), 0);

    let both = Element::container([Element::container([])
        .width(length(500.0))
        .height(length(500.0))
        .shrink(0.0)])
    .width(length(200.0))
    .height(length(100.0))
    .overflow(Axes {
        x: Overflow::Auto,
        y: Overflow::Auto,
    })
    .scroll_config(
        ScrollConfig::default()
            .axes(ScrollAxes::Both)
            .scrollbar(parts().width(0.0)),
    );
    let mut both = UiTree::new(both);
    let mut layout = LayoutEngine::new();
    let output = layout
        .compute(&mut both, &mut text, Size::new(200.0, 100.0))
        .unwrap();
    let scrollbar = output.scroll_regions[0].scrollbar.as_ref().unwrap();
    assert!(scrollbar.vertical.is_none());
    assert!(scrollbar.horizontal.is_none());

    let mut no_vertical_track = UiTree::new(content(parts().insets(Sides::length(60.0))));
    let mut layout = LayoutEngine::new();
    let output = layout
        .compute(&mut no_vertical_track, &mut text, Size::new(200.0, 100.0))
        .unwrap();
    assert!(
        output.scroll_regions[0]
            .scrollbar
            .as_ref()
            .unwrap()
            .vertical
            .is_none()
    );

    let horizontal = Element::row([Element::container([])
        .width(length(500.0))
        .height(length(40.0))
        .shrink(0.0)])
    .width(length(200.0))
    .height(length(80.0))
    .overflow(Axes {
        x: Overflow::Auto,
        y: Overflow::Hidden,
    })
    .scroll_config(ScrollConfig::default().scrollbar(parts().insets(Sides {
        left: 120.0,
        right: 120.0,
        top: 4.0,
        bottom: 4.0,
    })));
    let mut horizontal = UiTree::new(horizontal);
    let mut layout = LayoutEngine::new();
    let output = layout
        .compute(&mut horizontal, &mut text, Size::new(200.0, 80.0))
        .unwrap();
    assert!(
        output.scroll_regions[0]
            .scrollbar
            .as_ref()
            .unwrap()
            .horizontal
            .is_none()
    );

    let only_vertical_overflow = Element::column([Element::container([])
        .width(length(200.0))
        .height(length(300.0))
        .shrink(0.0)])
    .width(length(200.0))
    .height(length(100.0))
    .overflow(Axes {
        x: Overflow::Auto,
        y: Overflow::Auto,
    })
    .scroll_config(ScrollConfig::default().scrollbar(parts()));
    let mut only_vertical_overflow = UiTree::new(only_vertical_overflow);
    let mut layout = LayoutEngine::new();
    let output = layout
        .compute(
            &mut only_vertical_overflow,
            &mut text,
            Size::new(200.0, 100.0),
        )
        .unwrap();
    let scrollbar = output.scroll_regions[0].scrollbar.as_ref().unwrap();
    assert!(scrollbar.vertical.is_some());
    assert!(scrollbar.horizontal.is_none());

    let mut y_clip = UiTree::new(Element::container([]).overflow(Axes {
        x: Overflow::Visible,
        y: Overflow::Hidden,
    }));
    let mut layout = LayoutEngine::new();
    layout
        .compute(&mut y_clip, &mut text, Size::new(200.0, 100.0))
        .unwrap();
}

#[test]
fn sticky_children_follow_scroll_without_recomputing_taffy() {
    let sticky = Element::text("Sticky")
        .keyed("sticky")
        .position(Position::Sticky)
        .inset(Sides {
            left: argui_ui::LengthPercentageAuto::auto(),
            right: argui_ui::LengthPercentageAuto::auto(),
            top: argui_ui::LengthPercentageAuto::length(0.0),
            bottom: argui_ui::LengthPercentageAuto::auto(),
        })
        .height(length(24.0));
    let root = Element::column([sticky, Element::container([]).height(length(300.0))])
        .height(length(100.0))
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        });
    let mut ui = UiTree::new(root);
    let sticky_node = ui.node_id_at(1).unwrap();
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(200.0, 100.0))
        .unwrap();
    let initial = output
        .nodes
        .iter()
        .find(|node| node.node == sticky_node)
        .unwrap()
        .bounds;
    ui.scroll(
        Point::new(20.0, 20.0),
        ScrollDelta::Pixels(Point::new(0.0, -60.0)),
        &output.scroll_regions,
    );
    layout.apply_scroll(&ui, &mut output).unwrap();
    let moved = output
        .nodes
        .iter()
        .find(|node| node.node == sticky_node)
        .unwrap()
        .bounds;
    assert_eq!(moved.origin.y, initial.origin.y);
}

#[test]
fn automatic_scroll_anchoring_preserves_the_first_visible_keyed_row() {
    let view = |first_height| {
        Element::column([
            Element::text("first")
                .keyed("first")
                .height(length(first_height))
                .shrink(0.0),
            Element::text("anchor")
                .keyed("anchor")
                .height(length(30.0))
                .shrink(0.0),
            Element::container([]).height(length(200.0)).shrink(0.0),
        ])
        .keyed("list")
        .height(length(60.0))
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        })
    };
    let mut ui = UiTree::new(view(30.0));
    let list = ui.node_id_at(0).unwrap();
    ui.set_scroll_offset(list, Point::new(0.0, 30.0));
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let first = layout
        .compute(&mut ui, &mut text, Size::new(200.0, 60.0))
        .unwrap();
    let anchor = ui
        .resolve_node(&argui_ui::FocusTarget::from("anchor"))
        .unwrap();
    let initial_y = first
        .nodes
        .iter()
        .find(|node| node.node == anchor)
        .unwrap()
        .bounds
        .origin
        .y;

    ui.update(view(50.0));
    let second = layout
        .compute(&mut ui, &mut text, Size::new(200.0, 60.0))
        .unwrap();
    let anchored_y = second
        .nodes
        .iter()
        .find(|node| node.node == anchor)
        .unwrap()
        .bounds
        .origin
        .y;
    assert_eq!(anchored_y, initial_y);
    assert_eq!(ui.scroll_offset(list), Point::new(0.0, 50.0));
}

#[test]
fn disabled_scroll_anchoring_leaves_the_explicit_offset_unchanged() {
    let view = |first_height| {
        Element::column([
            Element::text("first")
                .keyed("first")
                .height(length(first_height))
                .shrink(0.0),
            Element::text("second")
                .keyed("second")
                .height(length(30.0))
                .shrink(0.0),
            Element::container([]).height(length(200.0)).shrink(0.0),
        ])
        .keyed("list")
        .height(length(60.0))
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        })
        .scroll_config(ScrollConfig::default().anchoring(ScrollAnchoring::None))
    };
    let mut ui = UiTree::new(view(30.0));
    let list = ui.node_id_at(0).unwrap();
    ui.set_scroll_offset(list, Point::new(0.0, 30.0));
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    layout
        .compute(&mut ui, &mut text, Size::new(200.0, 60.0))
        .unwrap();
    ui.update(view(50.0));
    layout
        .compute(&mut ui, &mut text, Size::new(200.0, 60.0))
        .unwrap();
    assert_eq!(ui.scroll_offset(list), Point::new(0.0, 30.0));
}

#[test]
fn variable_virtual_lists_measure_visible_rows_during_layout() {
    let list = argui_ui::VirtualList::variable(6, 20.0, 60.0).overscan(1);
    let view = || {
        list.build("variable-list", 0.0, |index| {
            Element::container([])
                .keyed(format!("variable-{index}"))
                .height(length(12.0 + index as f32 * 4.0))
        })
        .width(length(200.0))
    };
    let mut ui = UiTree::new(view());
    assert!(ui.root().children[0].children[1].virtual_item().is_some());
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();

    let first = layout
        .compute(&mut ui, &mut text, Size::new(200.0, 100.0))
        .unwrap();
    assert_eq!(list.item_extent(0), Some(12.0));
    assert_eq!(list.item_extent(3), Some(24.0));
    assert!(first.virtualization_changed);

    ui.update(view());
    let settled = layout
        .compute(&mut ui, &mut text, Size::new(200.0, 100.0))
        .unwrap();
    assert!(!settled.virtualization_changed);
}
