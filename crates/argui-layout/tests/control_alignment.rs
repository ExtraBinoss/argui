use std::sync::Arc;

use argui_core::{Point, PointerEvent, PointerPhase, Rect, Size, Transform2D};
use argui_layout::{LayoutEngine, LayoutOutput};
use argui_paint::{Border, Color, DisplayCommand, Fill, VectorAsset, VectorId};
use argui_text::{TextEngine, TextStyle, TextWrap};
use argui_ui::{
    AlignItems, Axes, Element, EventHandler, EventHandlerId, EventOwnerId, EventType, HitTestStyle,
    Interaction, JustifyContent, LengthPercentageAuto, Overflow, PointerEvents, ScrollAxes,
    ScrollConfig, Sides, UiTree, length, percent,
};

const NOTO_SANS: &[u8] = include_bytes!("../../../assets/fonts/NotoSans-Regular.ttf");
const SQUARE_SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><rect width="24" height="24"/></svg>"#;

/// Creates font metrics independent of the machine running these headless tests.
fn text_engine() -> TextEngine {
    TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans")
}

/// Computes retained geometry and paint for `root` in `viewport` with optional SVG assets.
fn compute(root: Element, viewport: Size, vectors: &[VectorAsset]) -> (UiTree, LayoutOutput) {
    let mut tree = UiTree::new(root);
    let mut engine = LayoutEngine::new();
    engine.set_assets(&[], vectors);
    let output = engine
        .compute(&mut tree, &mut text_engine(), viewport)
        .unwrap();
    (tree, output)
}

/// Returns final logical bounds of the retained node identified by `key`.
fn bounds(tree: &UiTree, output: &LayoutOutput, key: &str) -> Rect {
    output
        .nodes
        .iter()
        .find(|node| tree.key(node.node) == Some(key))
        .unwrap_or_else(|| panic!("missing layout node {key}"))
        .bounds
}

/// Compares geometry allowing only the stated logical pixel rounding tolerance.
fn assert_close(actual: f32, expected: f32, tolerance: f32) {
    assert!(
        (actual - expected).abs() <= tolerance + 0.0001,
        "expected {expected} ± {tolerance}, got {actual}"
    );
}

/// Returns the geometric center of a logical rectangle.
fn center(rect: Rect) -> Point {
    Point::new(
        rect.origin.x + rect.size.width / 2.0,
        rect.origin.y + rect.size.height / 2.0,
    )
}

/// Supplies a square 24 px source so fixed SVG dimensions must override intrinsic size.
fn square_vector() -> VectorAsset {
    VectorAsset {
        id: VectorId(1),
        size: Size::new(24.0, 24.0),
        svg: Arc::from(SQUARE_SVG),
        tintable: true,
    }
}

/// Mirrors the mode control's border, padded row, and natively translated active surface.
fn segmented_control(width: f32, height: f32, active: usize) -> Element {
    let item_width = (width - 8.0) / 3.0;
    let highlight = Element::container([])
        .keyed("highlight")
        .absolute(Sides {
            left: LengthPercentageAuto::length(3.0),
            right: LengthPercentageAuto::auto(),
            top: LengthPercentageAuto::length(3.0),
            bottom: LengthPercentageAuto::auto(),
        })
        .width(length(item_width))
        .height(length(height - 8.0))
        .background(Color::WHITE)
        .transform(Transform2D::IDENTITY.translate(active as f32 * item_width, 0.0));
    let cells = (0..3).map(|index| {
        Element::container([])
            .keyed(format!("cell-{index}"))
            .width(length(item_width))
            .height(length(height - 8.0))
            .shrink(0.0)
    });
    Element::container([
        highlight,
        Element::row(cells)
            .keyed("cells")
            .width(percent(1.0))
            .height(percent(1.0))
            .padding(Sides::length(3.0)),
    ])
    .keyed("control")
    .width(length(width))
    .height(length(height))
    .border(Border::all(1.0, Color::BLACK))
}

#[test]
fn segmented_highlights_follow_cells_with_four_pixel_outer_insets() {
    for width in [330.0, 126.0, 335.5] {
        for height in [42.0, 32.0] {
            for active in 0..3 {
                let (tree, output) = compute(
                    segmented_control(width, height, active),
                    Size::new(400.0, 60.0),
                    &[],
                );
                let control = bounds(&tree, &output, "control");
                let first = bounds(&tree, &output, "cell-0");
                let last = bounds(&tree, &output, "cell-2");
                let cell = bounds(&tree, &output, &format!("cell-{active}"));
                let highlight = bounds(&tree, &output, "highlight");
                assert_eq!(highlight.origin, first.origin);
                assert_eq!(first.origin, Point::new(4.0, 4.0));
                assert_close(
                    control.size.width - last.origin.x - last.size.width,
                    4.0,
                    0.0,
                );
                let painted = output
                    .display_list
                    .commands()
                    .iter()
                    .find_map(|command| match command {
                        DisplayCommand::Quad(quad)
                            if quad.background == Some(Fill::Solid(Color::WHITE)) =>
                        {
                            Some(quad.transform.transform_rect(quad.bounds))
                        }
                        _ => None,
                    })
                    .expect("missing painted active surface");
                // Box edges are rounded by Taffy, while native translations retain fractions.
                assert_close(painted.origin.x, cell.origin.x, 1.0);
                assert_close(painted.size.width, cell.size.width, 1.0);
                assert_close(painted.origin.y - control.origin.y, 4.0, 0.0);
                assert_close(
                    control.size.height - painted.origin.y - painted.size.height,
                    4.0,
                    0.0,
                );
                if active == 0 {
                    assert_close(painted.origin.x - control.origin.x, 4.0, 0.0);
                }
                if active == 2 {
                    assert_close(
                        control.size.width - painted.origin.x - painted.size.width,
                        4.0,
                        1.0,
                    );
                }
            }
        }
    }
}

#[test]
fn full_content_rows_center_fixed_svg_boxes_inside_bordered_icon_buttons() {
    let vector = square_vector();
    for (button_size, icon_size) in [(24.0, 15.0), (28.0, 14.0)] {
        let root = Element::container([Element::row([Element::vector(vector.id)
            .keyed("icon")
            .width(length(icon_size))
            .height(length(icon_size))])
        .keyed("content")
        .width(percent(1.0))
        .height(percent(1.0))
        .align_items(AlignItems::CENTER)
        .justify_content(JustifyContent::CENTER)])
        .keyed("button")
        .width(length(button_size))
        .height(length(button_size))
        .padding(Sides::length(0.0))
        .border(Border::all(1.0, Color::TRANSPARENT));
        let (tree, output) = compute(root, Size::new(80.0, 40.0), std::slice::from_ref(&vector));
        let button = bounds(&tree, &output, "button");
        let content = bounds(&tree, &output, "content");
        let icon = bounds(&tree, &output, "icon");
        assert_eq!(content.origin, Point::new(1.0, 1.0));
        assert_eq!(
            content.size,
            Size::new(button_size - 2.0, button_size - 2.0)
        );
        assert_eq!(icon.size, Size::new(icon_size, icon_size));
        assert_close(center(icon).x, center(button).x, 0.5);
        assert_close(center(icon).y, center(button).y, 0.5);
        let painted = output
            .display_list
            .commands()
            .iter()
            .find_map(|command| match command {
                DisplayCommand::Vector(primitive) => Some(primitive.bounds),
                _ => None,
            })
            .expect("missing painted SVG");
        assert_eq!(painted, icon);
    }
}

#[test]
fn select_twenty_pixel_line_boxes_center_text_and_icons_in_twenty_eight_pixel_rows() {
    let vector = square_vector();
    let style = TextStyle {
        font_size: 13.0,
        line_height: 20.0,
        ..TextStyle::default()
    };
    let viewport = Element::container([Element::text("Dark")
        .keyed("label")
        .text_style(style.clone())])
    .keyed("viewport")
    .width(length(160.0))
    .height(length(20.0));
    let root = Element::row([
        viewport,
        Element::vector(vector.id)
            .keyed("check")
            .width(length(14.0))
            .height(length(14.0)),
    ])
    .keyed("option")
    .width(length(200.0))
    .height(length(28.0))
    .align_items(AlignItems::CENTER);
    let (tree, output) = compute(root, Size::new(220.0, 40.0), std::slice::from_ref(&vector));
    let option = bounds(&tree, &output, "option");
    let viewport = bounds(&tree, &output, "viewport");
    let label = bounds(&tree, &output, "label");
    let check = bounds(&tree, &output, "check");
    assert_eq!(viewport.origin.y, 4.0);
    assert_eq!(label.origin.y, viewport.origin.y);
    assert_eq!(label.size.height, 20.0);
    assert_eq!(center(label).y, center(option).y);
    assert_eq!(check.origin.y, 7.0);
    assert_eq!(center(check).y, center(option).y);
    let block = &output.text.blocks()[0];
    assert_eq!(block.bounds, label);
    assert_eq!(block.style.font_size, 13.0);
    assert_eq!(block.style.line_height, 20.0);

    let mut text = text_engine();
    let line = text.measure_layout("Dark", &style, None);
    let natural = text.measure_layout(
        "Dark",
        &TextStyle {
            line_height: 16.25,
            ..style
        },
        None,
    );
    assert_eq!(line.size.height, 20.0);
    assert_close(
        line.first_baseline.unwrap() - natural.first_baseline.unwrap(),
        1.875,
        0.0001,
    );
}

/// Mirrors a Select option with its naturally measured label inside a clipped viewport.
fn select_option_with_label_target(blocking_wrapper: bool) -> Element {
    let handler = EventHandler::from_identity(EventHandlerId::new(EventOwnerId(1), 1));
    let area = Element::container([Element::container([Element::text(
        "A very long native audio input device name extending beyond the option viewport",
    )
    .keyed("label-text")
    .text_style(TextStyle {
        font_size: 13.0,
        line_height: 20.0,
        wrap: TextWrap::None,
        ..TextStyle::default()
    })])
    .height(length(20.0))])
    .keyed("label-area")
    .interaction(Interaction::default())
    .hit_test(HitTestStyle::default().pointer_events(PointerEvents::BoxOnly))
    .on(handler.direct_listener(EventType::PointerEnter))
    .on(handler.direct_listener(EventType::PointerLeave));
    let viewport = Element::container([
        Element::row([Element::container([area]).shrink(0.0)]).height(length(20.0))
    ])
    .keyed("label-viewport")
    .width(length(160.0))
    .height(length(20.0))
    .overflow(Axes {
        x: Overflow::Auto,
        y: Overflow::Hidden,
    })
    .scroll_config(ScrollConfig::default().axes(ScrollAxes::Horizontal));
    let surface = Element::row([
        viewport,
        Element::container([]).width(length(16.0)).shrink(0.0),
    ])
    .keyed("option-surface")
    .width(percent(1.0))
    .height(length(28.0))
    .padding(Sides {
        left: length(6.0),
        right: length(6.0),
        top: length(0.0),
        bottom: length(0.0),
    })
    .align_items(AlignItems::CENTER)
    .interaction(Interaction::default());
    let content = if blocking_wrapper {
        Element::container([surface])
            .keyed("blocking-option-area")
            .width(percent(1.0))
            .height(length(28.0))
            .interaction(Interaction::default())
            .hit_test(HitTestStyle::default().pointer_events(PointerEvents::BoxOnly))
    } else {
        surface
    };
    Element::container([content])
        .width(length(200.0))
        .height(length(28.0))
}

#[test]
fn select_label_hit_targets_receive_enter_and_leave_without_a_box_only_ancestor() {
    for blocking_wrapper in [false, true] {
        let (mut tree, output) = compute(
            select_option_with_label_target(blocking_wrapper),
            Size::new(220.0, 40.0),
            &[],
        );
        let viewport = bounds(&tree, &output, "label-viewport");
        let area = bounds(&tree, &output, "label-area");
        let text = bounds(&tree, &output, "label-text");
        assert!(area.size.width > viewport.size.width);
        assert_eq!(area.size.width, text.size.width);
        assert_eq!(area.size.height, 20.0);
        let inside = Point::new(viewport.origin.x + 5.0, center(viewport).y);
        let outside = Point::new(
            viewport.origin.x + viewport.size.width + 1.0,
            center(viewport).y,
        );
        let label_region = output
            .hit_regions
            .iter()
            .find(|region| tree.key(region.node) == Some("label-area"));
        if blocking_wrapper {
            assert!(
                label_region.is_none(),
                "BoxOnly must suppress descendant hits"
            );
            let hit = output
                .hit_regions
                .iter()
                .rev()
                .find(|region| region.contains(inside))
                .unwrap();
            assert_eq!(tree.key(hit.node), Some("blocking-option-area"));
            continue;
        }
        let region = label_region.expect("the unwrapped label must own a native hit region");
        assert_eq!(region.bounds, area);
        assert!(region.contains(inside));
        assert!(
            area.contains(outside),
            "the intrinsic text extends beyond the viewport"
        );
        assert!(
            !region.contains(outside),
            "the viewport must clip the label hit region"
        );
        let label = region.node;
        let entered = tree.pointer_event(
            PointerEvent::mouse(PointerPhase::Moved, inside),
            &output.hit_regions,
        );
        assert_eq!(
            entered
                .events
                .iter()
                .filter(|event| {
                    event.target == label && event.kind.event_type() == EventType::PointerEnter
                })
                .count(),
            1
        );
        let left = tree.pointer_event(
            PointerEvent::mouse(PointerPhase::Moved, outside),
            &output.hit_regions,
        );
        assert_eq!(
            left.events
                .iter()
                .filter(|event| {
                    event.target == label && event.kind.event_type() == EventType::PointerLeave
                })
                .count(),
            1
        );
    }
}
