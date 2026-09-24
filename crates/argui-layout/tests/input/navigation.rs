use super::*;

#[test]
fn repeated_newlines_keep_the_multiline_caret_visible_and_scroll_monotonic() {
    let editor = |value: &str| {
        multiline_editor("notes", value, "notes", TextStyle::default()).height(length(96.0))
    };
    let mut ui = UiTree::new(editor("start"));
    let node = ui.node_id_at(0).unwrap();
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(260.0, 96.0))
        .unwrap();
    ui.pointer_moved(Point::new(20.0, 20.0), &output.hit_regions);
    ui.primary_pressed(&output.hit_regions);
    ui.place_text_cursor(node, "start".len(), false);
    let mut previous_scroll = 0.0;

    for repeat in 0..12 {
        ui.edit_text_input(&KeyInput {
            key: Key::Enter,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            repeat: repeat > 0,
            text: None,
        });
        let value = ui.text_input_value(node).unwrap().to_owned();
        ui.update(editor(&value));
        output = layout
            .compute(&mut ui, &mut text, Size::new(260.0, 96.0))
            .unwrap();
        let region = &output.text_inputs[0];
        let scroll = ui.scroll_offset(node).y;
        assert!(scroll >= previous_scroll);
        assert!(region.caret.unwrap().origin.y >= region.viewport.origin.y);
        assert!(
            region.caret.unwrap().origin.y + region.caret.unwrap().size.height
                <= region.viewport.origin.y + region.viewport.size.height
        );
        previous_scroll = scroll;
    }
    assert!(previous_scroll > 0.0);
}

#[test]
fn transformed_editor_hits_and_drags_resolve_the_same_text_positions() {
    use argui_core::Affine2D;
    let field = |key, value| single_line_editor(key, value, "", TextStyle::default());
    let mut ui = UiTree::new(Element::column([
        field("first", "other field"),
        field("second", "alpha beta gamma"),
    ]));
    let node = ui.node_ids()[2];
    let mut output = LayoutEngine::new()
        .compute(&mut ui, &mut text_engine(), Size::new(400.0, 160.0))
        .unwrap();
    let region = output
        .text_inputs
        .iter()
        .find(|region| region.node == node)
        .unwrap()
        .clone();
    let local = region
        .stops
        .iter()
        .find(|stop| stop.position.index == 7)
        .unwrap()
        .point;
    let expected = region.closest_position(local);
    for transform in [
        Affine2D::IDENTITY,
        Affine2D {
            matrix: [2.0, 0.0, 0.0, 1.5],
            translation: Point::new(120.0, 45.0),
        },
    ] {
        output
            .hit_regions
            .iter_mut()
            .find(|hit| hit.node == node)
            .unwrap()
            .transform = transform;
        let point = transform.transform_point(local);
        let mapped = output.local_point(node, point).unwrap();
        assert!((mapped.x - local.x).hypot(mapped.y - local.y) < 0.001);
        assert_eq!(region.hit_position(mapped), Some(expected));
        ui.begin_text_selection(
            node,
            region.closest_position(mapped),
            false,
            argui_ui::SelectionGranularity::Word,
        );
        assert_eq!(ui.text_input_selection(node), Some((6, 10)));
        // Captured drags may leave the editor while still needing its inverse transform.
        let outside = Point::new(region.bounds.origin.x - 50.0, local.y);
        let mapped = output
            .local_point(node, transform.transform_point(outside))
            .unwrap();
        assert!(region.hit_position(mapped).is_none());
        ui.drag_text_position(node, region.closest_position(mapped));
        assert_eq!(ui.text_input_selection(node), Some((0, 10)));
        ui.release_text_cursor();
    }
    output
        .hit_regions
        .iter_mut()
        .find(|hit| hit.node == node)
        .unwrap()
        .transform = Affine2D {
        matrix: [0.0; 4],
        ..Affine2D::IDENTITY
    };
    assert!(output.local_point(node, local).is_none());
    output.hit_regions.retain(|hit| hit.node != node);
    assert!(output.local_point(node, local).is_none());
}

struct Editor {
    ui: UiTree,
    layout: LayoutEngine,
    text: TextEngine,
    node: argui_ui::NodeId,
    size: Size,
}

impl Editor {
    fn new(value: &str, width: f32, height: f32) -> Self {
        let ui = UiTree::new(
            multiline_editor("field", value, "", TextStyle::default())
                .width(length(width))
                .height(length(height)),
        );
        let node = ui.node_ids()[0];
        let mut editor = Self {
            ui,
            node,
            layout: LayoutEngine::new(),
            text: text_engine(),
            size: Size::new(width, height),
        };
        let output = editor
            .layout
            .compute(&mut editor.ui, &mut editor.text, editor.size)
            .unwrap();
        editor.ui.sync_focus(
            &output.hit_regions,
            Some(argui_ui::FocusRequest::Focus("field".into())),
        );
        editor
    }
    fn region(&mut self) -> TextInputRegion {
        self.layout
            .compute(&mut self.ui, &mut self.text, self.size)
            .unwrap()
            .text_inputs
            .remove(0)
    }
    fn go(&mut self, key: Key, control: bool, shift: bool) -> usize {
        let region = self.region();
        region
            .navigate(
                &mut self.ui,
                &KeyInput {
                    key,
                    state: KeyState::Pressed,
                    modifiers: Modifiers {
                        control,
                        shift,
                        ..Default::default()
                    },
                    repeat: false,
                    text: None,
                },
            )
            .expect("navigation is handled");
        self.ui.text_input_cursor(self.node).unwrap()
    }
    fn place(&mut self, index: usize) {
        self.ui.move_text_cursor(self.node, index, false);
    }
}

#[test]
fn arrows_cross_line_breaks_and_collapse_selections_in_both_directions() {
    let mut e = Editor::new("one\ntwo three", 400.0, 120.0);
    e.place(3);
    assert_eq!(e.go(Key::ArrowRight, false, false), 4);
    assert_eq!(e.go(Key::ArrowLeft, false, false), 3);
    e.place(0);
    assert_eq!(e.go(Key::ArrowLeft, false, false), 0);
    assert_eq!(e.go(Key::ArrowRight, true, true), 3);
    assert_eq!(e.go(Key::ArrowRight, true, true), 4);
    assert_eq!(e.go(Key::ArrowRight, true, true), 7);
    assert_eq!(e.ui.text_input_selection(e.node), Some((0, 7)));
    assert_eq!(e.go(Key::ArrowLeft, false, false), 0);
    assert_eq!(e.ui.text_input_selection(e.node), None);
    e.go(Key::End, true, false);
    assert_eq!(e.go(Key::ArrowRight, false, false), 13);
    e.go(Key::ArrowLeft, true, true);
    assert_eq!(e.go(Key::ArrowRight, false, false), 13);
    e.go(Key::ArrowLeft, true, true);
    assert_eq!(e.go(Key::ArrowLeft, false, false), 8);
    e.go(Key::ArrowRight, true, true);
    assert_eq!(e.go(Key::ArrowRight, false, false), 13);
}

#[test]
fn vertical_movement_retains_the_column_through_short_lines_and_resets_after_click() {
    let mut e = Editor::new("abcdef\nx\nabcdef", 400.0, 120.0);
    e.place(5);
    assert_eq!(e.go(Key::ArrowDown, false, false), 8);
    assert_eq!(e.go(Key::ArrowDown, false, false), 14);
    assert_eq!(e.go(Key::ArrowUp, false, true), 8);
    assert_eq!(e.go(Key::ArrowUp, false, true), 5);
    assert_eq!(e.ui.text_input_selection(e.node), Some((5, 14)));
    assert_eq!(e.go(Key::ArrowUp, false, false), 0);
    e.place(7);
    assert_eq!(e.go(Key::ArrowDown, false, false), 9);
    assert_eq!(e.go(Key::ArrowDown, false, false), 15);
    e.place(5);
    e.go(Key::ArrowDown, false, false);
    assert_eq!(e.go(Key::ArrowLeft, false, false), 7);
    assert_eq!(e.go(Key::ArrowDown, false, false), 9);
}

#[test]
fn home_and_end_use_wrapped_visual_lines_while_control_selects_the_document() {
    let mut e = Editor::new(
        "one two three four five six seven eight nine ten",
        120.0,
        120.0,
    );
    e.place(12);
    let start = e.go(Key::Home, false, false);
    let end = e.go(Key::End, false, true);
    assert!(start > 0 && end < 48 && end > start);
    assert_eq!(e.ui.text_input_selection(e.node), Some((start, end)));
    assert_eq!(e.go(Key::Home, true, false), 0);
    assert_eq!(e.go(Key::End, true, true), 48);
    assert_eq!(e.ui.text_input_selection(e.node), Some((0, 48)));
}

#[test]
fn page_navigation_moves_by_a_viewport_and_ignores_unrelated_or_composing_keys() {
    let value = (0..20).map(|_| "abcdefgh").collect::<Vec<_>>().join("\n");
    let mut e = Editor::new(&value, 200.0, 90.0);
    e.place(4);
    let down = e.go(Key::PageDown, false, true);
    assert!(down > 13 && down < value.len());
    assert_eq!(down % 9, 4);
    assert_eq!(e.go(Key::PageUp, false, false), 4);
    let region = e.region();
    let mut input = KeyInput {
        key: Key::Other,
        state: KeyState::Pressed,
        modifiers: Default::default(),
        repeat: false,
        text: None,
    };
    assert!(region.navigate(&mut e.ui, &input).is_none());
    input.key = Key::ArrowLeft;
    input.state = KeyState::Released;
    assert!(region.navigate(&mut e.ui, &input).is_none());
    input.state = KeyState::Pressed;
    e.ui.ime_input(argui_core::ImeInput::Preedit {
        text: "é".into(),
        cursor: None,
    });
    assert!(region.navigate(&mut e.ui, &input).is_none());
    assert!(e.ui.text_input_composing(e.node));
    e.ui.sync_focus(&[], Some(argui_ui::FocusRequest::Clear));
    assert!(region.navigate(&mut e.ui, &input).is_none());
}

#[test]
fn editing_and_undo_reset_the_column_retained_by_vertical_navigation() {
    let mut e = Editor::new("abcdef\nx\nabcdef", 400.0, 120.0);
    e.place(5);
    e.go(Key::ArrowDown, false, false);
    assert!(e.ui.text_input_goal_x(e.node).is_some());
    e.ui.paste_text(Some(e.node), "z");
    assert_eq!(e.ui.text_input_goal_x(e.node), None);
    e.go(Key::ArrowDown, false, false);
    assert!(e.ui.text_input_goal_x(e.node).is_some());
    e.ui.edit_text_input(&KeyInput {
        key: Key::Character("z".into()),
        state: KeyState::Pressed,
        modifiers: Modifiers {
            control: true,
            ..Default::default()
        },
        repeat: false,
        text: None,
    });
    assert_eq!(e.ui.text_input_goal_x(e.node), None);
    assert_eq!(e.ui.text_input_value(e.node), Some("abcdef\nx\nabcdef"));
}

#[test]
fn visual_neighbors_follow_lines_directions_and_word_boundaries() {
    let region = region(vec![
        stop(0, 0.0, 0.0, true),
        stop(1, 10.0, 0.0, false),
        stop(2, 20.0, 0.0, true),
        stop(3, 0.0, 20.0, true),
        stop(4, 11.0, 20.0, false),
        stop(5, 22.0, 20.0, true),
    ]);
    let middle = TextPosition::new(1, CaretAffinity::Before);
    assert_eq!(
        region.visual_neighbor(middle, true, false),
        TextPosition::new(0, CaretAffinity::Before)
    );
    assert_eq!(
        region.visual_neighbor(middle, false, true),
        TextPosition::new(2, CaretAffinity::Before)
    );
    assert_eq!(
        region.visual_neighbor(TextPosition::new(2, CaretAffinity::Before), false, false),
        TextPosition::new(3, CaretAffinity::Before)
    );

    let affinity_fallback = TextPosition::new(1, CaretAffinity::After);
    assert_eq!(
        region.visual_neighbor(affinity_fallback, false, false),
        TextPosition::new(2, CaretAffinity::Before)
    );
    let missing = TextPosition::new(99, CaretAffinity::Before);
    assert_eq!(region.visual_neighbor(missing, false, false), missing);

    assert_eq!(
        region.closest_position(Point::new(19.0, 19.0)),
        TextPosition::new(2, CaretAffinity::Before)
    );
    assert_eq!(
        region.closest_position(Point::new(9.0, 20.0)),
        TextPosition::new(4, CaretAffinity::Before)
    );
}

#[test]
fn a_resize_handle_painted_over_a_scrollbar_keeps_pointer_priority() {
    let value = (0..30)
        .map(|line| format!("line {line:02}"))
        .collect::<Vec<_>>()
        .join("\n");
    let area = multiline_editor("notes", value, "notes", TextStyle::default()).height(percent(1.0));
    let area = area.scroll_config(
        ScrollConfig::default()
            .propagation(ScrollPropagation::Contain)
            .scrollbar(ScrollbarStyle::new(
                ScrollbarPartStyle::new(QuadStyle::default()),
                ScrollbarPartStyle::new(QuadStyle::solid(Color::WHITE)),
            )),
    );
    let handle = Element::container([])
        .keyed("resize")
        .absolute(Sides {
            left: auto(),
            right: length(0.0),
            top: auto(),
            bottom: length(0.0),
        })
        .width(length(18.0))
        .height(length(18.0))
        .on(EventListener::new(
            EventType::Gesture,
            EventHandlerId::new(EventOwnerId(1), 0),
        ))
        .interaction(
            Interaction::default()
                .cursor(CursorIcon::NwseResize)
                .gestures(
                    GestureSet::EMPTY.pan(
                        PanGesture::default()
                            .immediate()
                            .capture(GestureCapture::OnPress),
                    ),
                ),
        );
    let mut ui = UiTree::new(
        Element::container([area, handle])
            .width(length(260.0))
            .height(length(96.0))
            .position(Position::Relative),
    );
    let handle = ui.node_id_at(2).unwrap();
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let output = layout
        .compute(&mut ui, &mut text, Size::new(260.0, 96.0))
        .unwrap();
    let bounds = output
        .nodes
        .iter()
        .find(|node| node.node == handle)
        .unwrap()
        .bounds;
    let point = Point::new(
        bounds.origin.x + bounds.size.width * 0.5,
        bounds.origin.y + bounds.size.height * 0.5,
    );

    assert!(output.scroll_regions[0].scrollbar_contains(point));
    assert!(scrollbar_at(point, &output.scroll_regions, &output.hit_regions).is_none());
    assert_eq!(
        output
            .hit_regions
            .iter()
            .rev()
            .find(|region| region.contains(point))
            .unwrap()
            .node,
        handle
    );

    ui.pointer_event(
        PointerEvent::mouse(PointerPhase::Moved, point),
        &output.hit_regions,
    );
    ui.pointer_event(
        PointerEvent::mouse(PointerPhase::Pressed, point),
        &output.hit_regions,
    );
    let dragged = ui.pointer_event(
        PointerEvent::mouse(
            PointerPhase::Moved,
            Point::new(point.x + 36.0, point.y + 24.0),
        ),
        &output.hit_regions,
    );
    assert!(dragged.events.iter().any(|event| {
        event.target_key() == Some("resize")
            && matches!(
                event.kind,
                UiEventKind::Gesture(argui_ui::GestureEvent {
                    kind: GestureKind::Pan { total, .. },
                    ..
                }) if total == Point::new(36.0, 24.0)
            )
    }));
}
