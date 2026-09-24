use argui_core::{Affine2D, CaretAffinity, Point, Rect, Size, TextPosition};
use argui_layout::LayoutEngine;
use argui_paint::{
    ClipChain, ClipRegion, CornerRadii, DisplayCommand, Fill, GradientStop, LinearGradient,
};
use argui_text::{TextEngine, TextStyle};
use argui_ui::{
    CaretStyle, DocumentSelectionEndpoint, DocumentTextPoint, Element, SelectionGranularity,
    TextEditorSpec, TextInputFilter, TextSelection, TextSelectionHighlight, TextSelectionRequest,
    TextSelectionStyle, UiTree, UserSelect, percent,
};

const NOTO_SANS: &[u8] = include_bytes!("../../../assets/fonts/NotoSans-Regular.ttf");

fn text_engine() -> TextEngine {
    TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans")
}

#[test]
fn text_editor_selection_highlight_keeps_authored_corner_radii() {
    let color = argui_core::Color::srgba(0.2, 0.5, 0.9, 0.4);
    let area = Element::text_editor(TextEditorSpec {
        value: "select me".to_owned(),
        placeholder: String::new(),
        multiline: true,
        read_only: false,
        filter: TextInputFilter::Any,
        text: TextStyle::default(),
        placeholder_text: TextStyle::default(),
        selection: argui_core::Color::WHITE,
        caret: CaretStyle::default(),
    })
    .keyed("rounded-selection")
    .width(percent(1.0))
    .selection_highlight(TextSelectionHighlight::solid(color).radius(5.0));
    let mut ui = UiTree::new(area);
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    layout
        .compute(&mut ui, &mut text, Size::new(240.0, 80.0))
        .unwrap();
    ui.select_text(TextSelectionRequest::new(
        "rounded-selection",
        TextSelection::All,
    ));
    let output = layout
        .compute(&mut ui, &mut text, Size::new(240.0, 80.0))
        .unwrap();

    assert!(output.display_list.commands().iter().any(|command| {
        matches!(command, DisplayCommand::Quad(quad)
            if quad.background == Some(Fill::Solid(color))
                && quad.radii.top_left == 5.0)
    }));
}

#[test]
fn transparent_overlays_block_text_hits_including_cached_portals() {
    use argui_ui::{CursorIcon, HitShape, HitTestStyle, Interaction, Sides, WindowLayer, length};
    for portal in [false, true] {
        let mut overlay = Element::container([])
            .keyed("glass")
            .width(length(100.0))
            .height(length(40.0))
            .absolute(Sides {
                left: length(0.0),
                top: length(0.0),
                right: argui_ui::auto(),
                bottom: argui_ui::auto(),
            })
            .user_select(UserSelect::None)
            .hit_test(
                HitTestStyle::default()
                    .shape(HitShape::RoundedRect(argui_paint::CornerRadii::all(20.0))),
            )
            .interaction(Interaction::blocker().cursor(CursorIcon::Grab));
        if portal {
            overlay = overlay.portal(WindowLayer::Popover).absolute(Sides {
                left: length(0.0),
                top: length(0.0),
                right: argui_ui::auto(),
                bottom: argui_ui::auto(),
            });
        }
        let mut ui = UiTree::new(Element::container([
            Element::text("selectable behind the glass"),
            overlay,
        ]));
        let mut engine = LayoutEngine::new();
        let mut text = text_engine();
        let mut output = engine
            .compute(&mut ui, &mut text, Size::new(300.0, 100.0))
            .unwrap();
        for _ in 0..3 {
            assert!(
                output.text_regions[0]
                    .hit_position(Point::new(40.0, 14.0))
                    .is_some()
            );
            assert!(
                output.text_at(Point::new(40.0, 14.0)).is_none(),
                "overlay owns pointer, portal={portal}, hits={:?}, text order={}",
                output.hit_regions,
                output.text_regions[0].interaction_order
            );
            assert!(
                output.text_at(Point::new(5.0, 5.0)).is_some(),
                "rounded cutout passes input"
            );
            assert!(
                output.text_at(Point::new(130.0, 8.0)).is_some(),
                "uncovered text remains selectable"
            );
            engine.repaint(&ui, &mut output);
        }
        assert!(output.paint_stats.reused_subtrees > 0);
    }
}

#[test]
fn interactive_parent_does_not_occlude_its_own_selectable_text() {
    let mut ui = UiTree::new(
        Element::container([Element::text("select me")])
            .interaction(argui_ui::Interaction::blocker()),
    );
    let mut engine = LayoutEngine::new();
    let mut output = engine
        .compute(&mut ui, &mut text_engine(), Size::new(300.0, 100.0))
        .unwrap();
    for _ in 0..3 {
        assert!(output.text_at(Point::new(20.0, 8.0)).is_some());
        engine.repaint(&ui, &mut output);
    }
}

#[test]
fn document_selection_uses_glyph_regions_and_paints_without_relayout() {
    let color = argui_core::Color::srgba(0.2, 0.5, 0.9, 0.4);
    let mut ui = UiTree::new(
        Element::column([
            Element::text("first line"),
            Element::text("not selectable").user_select(UserSelect::None),
            Element::text("last line"),
        ])
        .width(percent(1.0))
        .selection_style(TextSelectionStyle {
            background: color,
            handle: color,
        }),
    );
    let mut engine = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = engine
        .compute(&mut ui, &mut text, Size::new(400.0, 200.0))
        .unwrap();
    assert_eq!(output.text_regions.len(), 2);

    let first = output.text_regions[0].node;
    let last = output.text_regions[1].node;
    ui.begin_document_selection(
        DocumentTextPoint::new(first, TextPosition::new(2, CaretAffinity::After)),
        false,
        SelectionGranularity::Character,
    );
    ui.drag_document_selection(DocumentTextPoint::new(
        last,
        TextPosition::new(4, CaretAffinity::After),
    ));
    engine.repaint(&ui, &mut output);

    assert_eq!(
        ui.selected_document_text().as_deref(),
        Some("rst line\nlast")
    );
    assert!(output.document_selection_bounds(&ui).is_some());
    assert!(output.display_list.commands().iter().any(|command| {
        matches!(command, DisplayCommand::Quad(quad)
            if quad.background == Some(Fill::Solid(color))
                && quad.radii == CornerRadii::all(3.0))
    }));
}

#[test]
fn document_selection_paints_gradient_fills_with_fragment_radii() {
    let radii = CornerRadii {
        top_left: 2.0,
        top_right: 5.0,
        bottom_right: 8.0,
        bottom_left: 3.0,
    };
    let gradient = LinearGradient::new(
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
        argui_core::ColorInterpolation::Oklab,
        [
            GradientStop::new(0.0, argui_core::Color::srgb(0.9, 0.2, 0.5)),
            GradientStop::new(1.0, argui_core::Color::srgb(0.2, 0.7, 1.0)),
        ],
    )
    .unwrap();
    let mut ui = UiTree::new(
        Element::text("rounded rainbow selection")
            .selection_highlight(
                TextSelectionHighlight::new(Fill::Linear(gradient.clone())).radii(radii),
            )
            .width(percent(1.0)),
    );
    let mut engine = LayoutEngine::new();
    let mut output = engine
        .compute(&mut ui, &mut text_engine(), Size::new(300.0, 80.0))
        .unwrap();
    let node = output.text_regions[0].node;
    ui.begin_document_selection(
        DocumentTextPoint::new(node, TextPosition::new(0, CaretAffinity::Before)),
        false,
        SelectionGranularity::Character,
    );
    ui.drag_document_selection(DocumentTextPoint::new(
        node,
        TextPosition::new(7, CaretAffinity::After),
    ));
    engine.repaint(&ui, &mut output);

    assert!(output.display_list.commands().iter().any(|command| {
        matches!(command, DisplayCommand::Quad(quad)
            if quad.background == Some(Fill::Linear(gradient.clone())) && quad.radii == radii)
    }));

    let replacement = LinearGradient::new(
        Point::new(0.0, 0.0),
        Point::new(0.0, 1.0),
        argui_core::ColorInterpolation::Oklab,
        [
            GradientStop::new(0.0, argui_core::Color::srgb(0.2, 0.9, 0.5)),
            GradientStop::new(1.0, argui_core::Color::srgb(0.8, 0.3, 1.0)),
        ],
    )
    .unwrap();
    ui.update(
        Element::text("rounded rainbow selection")
            .selection_highlight(
                TextSelectionHighlight::new(Fill::Linear(replacement.clone())).radius(12.0),
            )
            .width(percent(1.0)),
    );
    engine.repaint(&ui, &mut output);
    assert!(output.display_list.commands().iter().any(|command| {
        matches!(command, DisplayCommand::Quad(quad)
            if quad.background == Some(Fill::Linear(replacement.clone()))
                && quad.radii == CornerRadii::all(12.0))
    }));
}

#[test]
fn text_cursor_only_appears_over_shaped_line_geometry() {
    let mut ui = UiTree::new(Element::text("short").width(percent(1.0)));
    let mut engine = LayoutEngine::new();
    let mut text = text_engine();
    let output = engine
        .compute(&mut ui, &mut text, Size::new(400.0, 100.0))
        .unwrap();
    let region = &output.text_regions[0];
    assert!(region.hit_position(Point::new(5.0, 5.0)).is_some());
    assert!(region.hit_position(Point::new(350.0, 5.0)).is_none());

    let center = region.layout.lines[0].bounds.origin;
    assert_eq!(region.distance_squared(center), 0.0);
    assert!(region.distance_squared(Point::new(-20.0, center.y)) > 0.0);
    assert!(region.distance_squared(Point::new(500.0, center.y)) > 0.0);
    assert!(region.distance_squared(Point::new(center.x, 80.0)) > 0.0);

    let mut transformed = region.clone();
    transformed.transform = Affine2D::translation(40.0, 20.0);
    let local_hit = region.hit_position(Point::new(5.0, 5.0)).unwrap();
    assert_eq!(
        transformed.hit_position(Point::new(45.0, 25.0)),
        Some(local_hit)
    );
    transformed.clips = ClipChain::from_regions([ClipRegion::new(
        Rect::new(Point::new(100.0, 100.0), Size::new(20.0, 20.0)),
        Affine2D::IDENTITY,
    )]);
    assert!(transformed.hit_position(Point::new(45.0, 25.0)).is_none());

    transformed.transform = Affine2D {
        matrix: [0.0; 4],
        translation: Point::default(),
    };
    let closest = transformed.closest_position(Point::new(5.0, 5.0));
    assert!(closest.position.index <= 5);
}

#[test]
fn touch_selection_paints_both_handles_in_forward_and_reverse_order() {
    let background = argui_core::Color::srgba(0.2, 0.5, 0.9, 0.4);
    let handle = argui_core::Color::srgb(0.9, 0.2, 0.4);
    let mut ui = UiTree::new(
        Element::text("select these words")
            .width(percent(1.0))
            .selection_style(TextSelectionStyle { background, handle }),
    );
    let mut engine = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = engine
        .compute(&mut ui, &mut text, Size::new(300.0, 80.0))
        .unwrap();
    assert!(output.document_selection_bounds(&ui).is_none());
    let node = output.text_regions[0].node;

    ui.begin_touch_document_selection(
        DocumentTextPoint::new(node, TextPosition::new(1, CaretAffinity::After)),
        SelectionGranularity::Character,
    );
    ui.drag_document_selection(DocumentTextPoint::new(
        node,
        TextPosition::new(12, CaretAffinity::After),
    ));
    engine.repaint(&ui, &mut output);
    assert_eq!(solid_quad_count(&output, handle), 2);

    ui.begin_touch_document_selection(
        DocumentTextPoint::new(node, TextPosition::new(15, CaretAffinity::After)),
        SelectionGranularity::Character,
    );
    ui.drag_document_selection(DocumentTextPoint::new(
        node,
        TextPosition::new(3, CaretAffinity::After),
    ));
    engine.repaint(&ui, &mut output);
    assert_eq!(solid_quad_count(&output, handle), 2);
    assert!(output.document_selection_bounds(&ui).is_some());
}

#[test]
fn touch_selection_handles_have_transformed_comfortable_hit_targets() {
    let mut ui = UiTree::new(Element::text("alpha beta gamma").width(percent(1.0)));
    let mut engine = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = engine
        .compute(&mut ui, &mut text, Size::new(300.0, 80.0))
        .unwrap();
    let node = output.text_regions[0].node;
    ui.begin_touch_document_selection(
        DocumentTextPoint::new(node, TextPosition::new(7, CaretAffinity::After)),
        SelectionGranularity::Word,
    );
    ui.release_document_selection();
    engine.repaint(&ui, &mut output);

    let handles = output.selection_handles(&ui);
    assert_eq!(handles.len(), 2);
    for handle in &handles {
        assert_eq!(handle.hit_bounds.size, Size::new(44.0, 44.0));
        assert_eq!(handle.visual_bounds.size, Size::new(10.0, 10.0));
        assert_eq!(
            output
                .selection_handle_at(&ui, handle.center)
                .map(|hit| hit.endpoint),
            Some(handle.endpoint)
        );
        assert!(
            handle
                .hit_bounds
                .contains(Point::new(handle.center.x, handle.center.y + 21.0))
        );
    }

    let anchor = handles
        .iter()
        .find(|handle| handle.endpoint == DocumentSelectionEndpoint::Anchor)
        .unwrap();
    output.text_regions[0].transform = Affine2D::translation(2.0, 3.0);
    let moved_anchor = output
        .selection_handles(&ui)
        .into_iter()
        .find(|handle| handle.endpoint == DocumentSelectionEndpoint::Anchor)
        .unwrap();
    assert_eq!(
        moved_anchor.center,
        Point::new(anchor.center.x + 2.0, anchor.center.y + 3.0)
    );
    assert_eq!(
        output
            .selection_handle_at(&ui, moved_anchor.center)
            .map(|hit| hit.endpoint),
        Some(DocumentSelectionEndpoint::Anchor)
    );
}

#[test]
fn touch_handles_attach_to_opposite_ends_across_text_nodes() {
    let background = argui_core::Color::srgba(0.2, 0.5, 0.9, 0.4);
    let handle = argui_core::Color::srgb(0.9, 0.2, 0.4);
    let mut ui = UiTree::new(
        Element::column([Element::text("first"), Element::text("second")])
            .width(percent(1.0))
            .selection_style(TextSelectionStyle { background, handle }),
    );
    let mut engine = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = engine
        .compute(&mut ui, &mut text, Size::new(300.0, 100.0))
        .unwrap();
    let first = output.text_regions[0].node;
    let second = output.text_regions[1].node;

    ui.begin_touch_document_selection(
        DocumentTextPoint::new(first, TextPosition::new(1, CaretAffinity::After)),
        SelectionGranularity::Character,
    );
    ui.drag_document_selection(DocumentTextPoint::new(
        second,
        TextPosition::new(4, CaretAffinity::After),
    ));
    engine.repaint(&ui, &mut output);
    assert_eq!(solid_quad_count(&output, handle), 2);

    ui.begin_touch_document_selection(
        DocumentTextPoint::new(second, TextPosition::new(5, CaretAffinity::After)),
        SelectionGranularity::Character,
    );
    ui.drag_document_selection(DocumentTextPoint::new(
        first,
        TextPosition::new(2, CaretAffinity::After),
    ));
    engine.repaint(&ui, &mut output);
    assert_eq!(solid_quad_count(&output, handle), 2);
}

fn solid_quad_count(output: &argui_layout::LayoutOutput, color: argui_core::Color) -> usize {
    output
        .display_list
        .commands()
        .iter()
        .filter(|command| {
            matches!(command, DisplayCommand::Quad(quad) if quad.background == Some(Fill::Solid(color)))
        })
        .count()
}
