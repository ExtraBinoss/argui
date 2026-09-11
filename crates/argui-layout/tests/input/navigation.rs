use super::*;

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
            TextArea::new(
                "field",
                value,
                "",
                InputStyle::new(PaintStyle::default(), TextStyle::default()),
            )
            .build()
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
