use super::*;

#[test]
/// Hover growth animates the scrollbar's painted and draggable geometry together.
fn scrollbar_hover_width_animates_native_geometry_without_relayout() {
    let style = ScrollbarStyle::new(
        ScrollbarPartStyle::new(QuadStyle::default()),
        ScrollbarPartStyle::new(QuadStyle::solid(Color::WHITE)),
    )
    .width(4.0)
    .hover_width(10.0)
    .insets(Sides::length(0.0))
    .visibility(ScrollbarVisibility::Always);
    let mut ui = UiTree::new(content(style));
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(200.0, 100.0))
        .unwrap();
    let node = ui.node_ids()[0];
    let width = |output: &argui_layout::LayoutOutput| {
        output
            .scroll_regions
            .iter()
            .find(|region| region.node == node)
            .unwrap()
            .scrollbar
            .as_ref()
            .unwrap()
            .vertical
            .as_ref()
            .unwrap()
            .thumb
            .size
            .width
    };
    assert_eq!(width(&output), 4.0);
    ui.scrollbar_pointer_moved(Some(Point::new(198.0, 10.0)), &output.scroll_regions);
    assert!(ui.wants_scroll_frame());
    assert!(
        ui.advance_scroll_physics(0.07, &output.scroll_regions)
            .scroll_changed
    );
    layout.apply_scroll(&ui, &mut output).unwrap();
    assert!(width(&output) > 4.0 && width(&output) < 10.0);
    assert!(
        ui.advance_scroll_physics(0.07, &output.scroll_regions)
            .scroll_changed
    );
    layout.apply_scroll(&ui, &mut output).unwrap();
    assert_eq!(width(&output), 10.0);
    assert!(!ui.wants_scroll_frame());

    assert!(
        ui.scrollbar_pressed(Point::new(198.0, 10.0), &output.scroll_regions)
            .is_some()
    );
    ui.scrollbar_pointer_moved(None, &[]);
    ui.advance_scroll_physics(0.14, &output.scroll_regions);
    layout.apply_scroll(&ui, &mut output).unwrap();
    assert_eq!(width(&output), 10.0);
    ui.scrollbar_released();
    ui.scrollbar_pointer_moved(None, &[]);
    ui.advance_scroll_physics(0.14, &output.scroll_regions);
    layout.apply_scroll(&ui, &mut output).unwrap();
    assert_eq!(width(&output), 4.0);

    ui.scrollbar_pointer_moved(Some(Point::new(198.0, 10.0)), &output.scroll_regions);
    ui.advance_scroll_physics(0.14, &output.scroll_regions);
    layout.apply_scroll(&ui, &mut output).unwrap();
    assert_eq!(width(&output), 10.0);
    ui.scrollbar_pointer_moved(None, &[]);
    ui.advance_scroll_physics(0.14, &output.scroll_regions);
    layout.apply_scroll(&ui, &mut output).unwrap();
    assert_eq!(width(&output), 4.0);
}

#[test]
/// Switching scrollbar sides refreshes painted and draggable geometry immediately.
fn scrollbar_side_change_repositions_without_layout_or_pointer_input() {
    let style = ScrollbarStyle::new(
        ScrollbarPartStyle::new(QuadStyle::default()),
        ScrollbarPartStyle::new(QuadStyle::solid(Color::WHITE)),
    )
    .width(6.0)
    .hover_width(10.0)
    .insets(Sides::length(0.0))
    .visibility(ScrollbarVisibility::Always);
    let mut ui = UiTree::new(content(style.clone().side(ScrollbarSide::Right)));
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(200.0, 100.0))
        .unwrap();
    let track_x = |output: &argui_layout::LayoutOutput| {
        output.scroll_regions[0]
            .scrollbar
            .as_ref()
            .unwrap()
            .vertical
            .as_ref()
            .unwrap()
            .track
            .origin
            .x
    };
    assert_eq!(track_x(&output), 194.0);
    assert_eq!(
        ui.update(content(style.side(ScrollbarSide::Left))),
        argui_ui::TreeUpdate::Scroll
    );
    layout.apply_scroll(&ui, &mut output).unwrap();
    assert_eq!(track_x(&output), 0.0);
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
            DisplayCommand::BeginCompositor(_),
            DisplayCommand::BeginLayer(_),
            DisplayCommand::Quad(_),
            DisplayCommand::Quad(_),
            DisplayCommand::EndLayer,
            DisplayCommand::EndCompositor
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
fn vertical_scrollbar_can_attach_to_the_left_edge() {
    let style = ScrollbarStyle::new(
        ScrollbarPartStyle::new(QuadStyle::default()),
        ScrollbarPartStyle::new(QuadStyle::solid(Color::WHITE)),
    )
    .side(ScrollbarSide::Left)
    .width(4.0)
    .insets(Sides {
        left: 0.0,
        right: 0.0,
        top: 0.0,
        bottom: 0.0,
    });
    let mut ui = UiTree::new(content(style));
    let output = LayoutEngine::new()
        .compute(&mut ui, &mut text_engine(), Size::new(200.0, 100.0))
        .unwrap();
    let thumb = output.scroll_regions[0]
        .scrollbar
        .as_ref()
        .and_then(|bar| bar.vertical)
        .expect("overflowing content has a vertical scrollbar");
    assert_eq!(thumb.track.origin.x, 0.0);
    assert_eq!(thumb.track.size.width, 4.0);
}
