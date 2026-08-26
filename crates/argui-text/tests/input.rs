use argui_core::{Size, TextPosition};
use argui_text::{TextEngine, TextStyle, TextWrap};

const NOTO_SANS: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");
const NOTO_ARABIC: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSansArabic.ttf");

fn engine() -> TextEngine {
    TextEngine::from_embedded_fonts(
        [NOTO_SANS, NOTO_ARABIC],
        "Noto Sans",
        "Noto Sans",
        "Noto Sans",
    )
}

fn pos(index: usize) -> TextPosition {
    TextPosition {
        index,
        ..TextPosition::default()
    }
}

#[test]
fn cosmic_input_geometry_handles_bidi_selection_and_overflow() {
    let value = "Hello مرحباً with a deliberately long suffix";
    let style = TextStyle {
        font_size: 18.0,
        line_height: 24.0,
        wrap: TextWrap::None,
        ..TextStyle::default()
    };
    let layout = engine().input_layout(
        value,
        &style,
        Size::new(120.0, 28.0),
        pos(value.len()),
        Some((pos(0), pos("Hello".len()))),
        0.0,
    );

    assert!(layout.scroll_x > 0.0);
    assert!(layout.caret.origin.x <= 120.0);
    assert!(!layout.selection.is_empty());
    assert!(
        layout
            .stops
            .iter()
            .any(|stop| stop.position.index == value.len())
    );
}

#[test]
fn horizontal_scroll_moves_only_when_the_caret_leaves_the_viewport() {
    let value = "A deliberately long editable value";
    let style = TextStyle {
        wrap: TextWrap::None,
        ..TextStyle::default()
    };
    let mut engine = engine();
    let end = engine.input_layout(
        value,
        &style,
        Size::new(80.0, 24.0),
        pos(value.len()),
        None,
        0.0,
    );
    assert!(end.scroll_x > 0.0);

    let nearby = engine.input_layout(
        value,
        &style,
        Size::new(80.0, 24.0),
        pos(value.len() - 1),
        None,
        end.scroll_x,
    );
    assert_eq!(nearby.scroll_x, end.scroll_x);

    let start = engine.input_layout(
        value,
        &style,
        Size::new(80.0, 24.0),
        pos(0),
        None,
        end.scroll_x,
    );
    assert_eq!(start.scroll_x, 0.0);
}

#[test]
fn mixed_bidi_selection_keeps_valid_visual_spans() {
    let value = "abc العربية xyz";
    let style = TextStyle {
        wrap: TextWrap::None,
        ..TextStyle::default()
    };
    let mut engine = engine();
    let layout = engine.input_layout(
        value,
        &style,
        Size::new(400.0, 24.0),
        pos(2),
        Some((pos(2), pos("abc العربية".len()))),
        0.0,
    );

    assert!(!layout.selection.is_empty());
    assert!(layout.selection.iter().all(|rect| rect.size.width > 0.0));
    assert!(layout.stops.iter().enumerate().any(|(index, stop)| {
        layout.stops[index + 1..].iter().any(|candidate| {
            candidate.position.index == stop.position.index
                && candidate.position.affinity != stop.position.affinity
                && (candidate.point.x - stop.point.x).abs() > 1.0
        })
    }));
    assert!(layout.stops.iter().enumerate().all(|(index, stop)| {
        layout.stops[index + 1..].iter().all(|candidate| {
            candidate.position != stop.position || (candidate.point.x - stop.point.x).abs() < 0.01
        })
    }));

    for index in 0..10 {
        let viewport = Size::new(400.0 + index as f32, 24.0);
        let cached = engine.input_layout(value, &style, viewport, pos(0), None, 0.0);
        assert!(!cached.stops.is_empty());
    }

    let inside_codepoint =
        engine.input_layout("é", &style, Size::new(40.0, 24.0), pos(1), None, 0.0);
    assert_eq!(inside_codepoint.caret.origin.x, 0.0);
}

#[test]
fn combining_marks_emit_one_visual_stop_per_position() {
    let value = "Hello · مرحباً · שלום · 👋🏽";
    let style = TextStyle {
        wrap: TextWrap::None,
        ..TextStyle::default()
    };
    let layout = engine().input_layout(value, &style, Size::new(500.0, 24.0), pos(0), None, 0.0);

    assert!(layout.stops.iter().enumerate().all(|(index, stop)| {
        layout.stops[index + 1..]
            .iter()
            .all(|candidate| candidate.position != stop.position)
    }));
}

#[test]
fn dragging_across_marked_arabic_grows_selection_monotonically() {
    let value = "Hello · مرحباً · שלום · 👋🏽";
    let style = TextStyle {
        wrap: TextWrap::None,
        ..TextStyle::default()
    };
    let start = value.find('م').unwrap();
    let end = start + "مرحباً".len();
    let mut engine = engine();
    let base = engine.input_layout(value, &style, Size::new(500.0, 24.0), pos(start), None, 0.0);
    let mut positions = base
        .stops
        .iter()
        .filter(|stop| (start..=end).contains(&stop.position.index))
        .collect::<Vec<_>>();
    positions.sort_by(|a, b| b.point.x.total_cmp(&a.point.x));

    let mut previous_width = 0.0;
    let anchor = positions
        .iter()
        .find(|stop| stop.position.index == start)
        .unwrap()
        .position;
    for stop in positions {
        if stop.position.index == start {
            continue;
        }
        let selection = engine.input_layout(
            value,
            &style,
            Size::new(500.0, 24.0),
            stop.position,
            Some((anchor, stop.position)),
            0.0,
        );
        let width = selection
            .selection
            .iter()
            .map(|rect| rect.size.width)
            .sum::<f32>();
        assert!(width + 0.01 >= previous_width);
        previous_width = width;
    }
}

#[test]
fn left_to_right_drag_never_selects_a_later_visual_run_first() {
    let value = "Hello · مرحباً · שלום · 👋🏽";
    let style = TextStyle {
        wrap: TextWrap::None,
        ..TextStyle::default()
    };
    let mut engine = engine();
    let base = engine.input_layout(value, &style, Size::new(500.0, 24.0), pos(0), None, 0.0);
    let anchor = base.stops.first().unwrap();
    let mut previous_right = anchor.point.x;

    for stop in &base.stops[1..] {
        if stop.point.x <= previous_right + 0.01 {
            continue;
        }
        let selection = engine.input_layout(
            value,
            &style,
            Size::new(500.0, 24.0),
            stop.position,
            Some((anchor.position, stop.position)),
            0.0,
        );
        let rect = selection.selection[0];
        assert!((rect.origin.x - anchor.point.x).abs() < 0.01);
        assert!(rect.origin.x + rect.size.width + 0.01 >= previous_right);
        previous_right = stop.point.x;
    }
}
