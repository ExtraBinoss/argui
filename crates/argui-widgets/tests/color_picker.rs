use argui_core::{Color, ColorScheme, Key, KeyInput, KeyState, Modifiers, Point, Rect, Size};
use argui_runtime::{LayoutBounds, LayoutSnapshot};
use argui_ui::{
    Element, ElementKind, GestureEvent, GestureKind, GesturePhase, UiEvent, UiEventKind, UiTree,
};
use argui_widgets::{ColorFormat, ColorPicker, ColorPickerState, shadcn};

#[path = "color_picker/input.rs"]
mod input;
#[path = "color_picker/paint.rs"]
mod paint;

fn event(key: &str, kind: UiEventKind) -> UiEvent {
    UiEvent::new(
        UiTree::new(Element::container([])).node_ids()[0],
        Some(key.into()),
        kind,
    )
}

fn edit(state: &mut ColorPickerState, index: usize, text: &str) {
    assert!(state.update(
        "color",
        &event(
            &format!("color::field::{index}"),
            UiEventKind::TextChanged(text.into())
        )
    ));
}

fn key(target: &str, key: Key) -> UiEvent {
    event(
        target,
        UiEventKind::KeyInput(KeyInput {
            key,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            repeat: false,
            text: None,
        }),
    )
}

fn find<'a>(element: &'a Element, key: &str) -> Option<&'a Element> {
    if element.key.as_deref() == Some(key) {
        return Some(element);
    }
    element.children.iter().find_map(|child| find(child, key))
}

fn field(state: &ColorPickerState, index: usize) -> String {
    let theme = shadcn(Color::BLACK);
    let view = ColorPicker::new("color", "Accent", state).build(theme.resolve(ColorScheme::Light));
    let ElementKind::TextEditor { value, .. } =
        &find(&view, &format!("color::field::{index}")).unwrap().kind
    else {
        panic!("text editor");
    };
    value.clone()
}

fn close(actual: Color, expected: [f32; 4]) {
    for (actual, expected) in actual.to_srgba().into_iter().zip(expected) {
        assert!((actual - expected).abs() < 0.0001, "{actual} != {expected}");
    }
}

#[test]
fn formats_round_trip_primary_secondary_gray_and_transparent_colors() {
    for rgb in [
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 1.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [0.5; 3],
        [0.0; 3],
        [1.0; 3],
    ] {
        let [r, g, b] = rgb;
        let mut state = ColorPickerState::new(Color::srgba(r, g, b, 0.4));
        close(state.color(), [r, g, b, 0.4]);
        for format in [ColorFormat::Hsv, ColorFormat::Hsl, ColorFormat::Rgb] {
            state.set_format(format);
            let values: Vec<_> = (0..4).map(|index| field(&state, index)).collect();
            for (index, value) in values.iter().enumerate() {
                edit(&mut state, index, value);
            }
            close(state.color(), [r, g, b, 0.4]);
        }
        state.set_format(ColorFormat::Hex);
        let hex = field(&state, 0);
        edit(&mut state, 0, &hex);
        assert_eq!(
            state.color().to_srgba8(),
            Color::srgba(r, g, b, 0.4).to_srgba8()
        );
    }
}

#[test]
fn fields_keep_invalid_drafts_and_recover_without_corrupting_the_color() {
    let mut state = ColorPickerState::new(Color::BLACK);
    for hex in ["#f00", "#ff000080", "#0f08", "#00ff00"] {
        edit(&mut state, 0, hex);
        assert_eq!(state.color(), Color::from_hex(hex).unwrap());
        assert_eq!(field(&state, 0), hex);
    }
    let previous = state.color();
    edit(&mut state, 0, "#ff");
    assert_eq!(state.color(), previous);
    assert_eq!(field(&state, 0), "#ff");
    assert!(state.update("color", &key("color::field::0", Key::Escape)));
    assert_ne!(field(&state, 0), "#ff");
    for format in [ColorFormat::Rgb, ColorFormat::Hsl, ColorFormat::Hsv] {
        state.set_format(format);
        for index in 0..4 {
            for invalid in ["", "-", "-1", "NaN", "inf", "999"] {
                edit(&mut state, index, invalid);
                assert_eq!(state.color(), previous);
                assert_eq!(field(&state, index), invalid);
                assert!(state.update(
                    "color",
                    &event(&format!("color::field::{index}"), UiEventKind::Blurred)
                ));
                assert_ne!(field(&state, index), invalid);
            }
        }
    }
    state.set_format(ColorFormat::Rgb);
    edit(&mut state, 0, "12.");
    assert_eq!(field(&state, 0), "12.");
    close(state.color(), [12.0 / 255.0, 1.0, 0.0, 1.0]);
}

#[test]
fn hue_survives_black_gray_and_external_updates_and_hsl_matches_known_values() {
    let mut state = ColorPickerState::new(Color::BLACK);
    state.set_format(ColorFormat::Hsv);
    edit(&mut state, 0, "240");
    edit(&mut state, 1, "100");
    state.set_color(Color::BLACK);
    edit(&mut state, 2, "100");
    close(state.color(), [0.0, 0.0, 1.0, 1.0]);
    state.set_color(Color::srgb(0.5, 0.5, 0.5));
    assert_eq!(field(&state, 0), "240");
    edit(&mut state, 1, "100");
    close(state.color(), [0.0, 0.0, 0.5, 1.0]);
    state.set_format(ColorFormat::Hsl);
    edit(&mut state, 0, "60");
    edit(&mut state, 1, "100");
    edit(&mut state, 2, "50");
    edit(&mut state, 3, "25");
    close(state.color(), [1.0, 1.0, 0.0, 0.25]);
    edit(&mut state, 2, "0");
    close(state.color(), [0.0, 0.0, 0.0, 0.25]);
}
