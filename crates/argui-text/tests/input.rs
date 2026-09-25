use argui_core::{Point, Size, TextPosition};
use argui_text::{
    CaretScroll, TextContent, TextEngine, TextInputScroll, TextSpan, TextSpanStyle, TextStyle,
    TextWrap,
};

const NOTO_SANS: &[u8] = include_bytes!("../../../assets/fonts/NotoSans-Regular.ttf");
const NOTO_ARABIC: &[u8] = include_bytes!("../../../assets/fonts/NotoSansArabic.ttf");

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

fn reveal(offset: Point) -> TextInputScroll {
    TextInputScroll::new(offset, CaretScroll::Reveal)
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
        reveal(Point::default()),
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
        reveal(Point::default()),
    );
    assert!(end.scroll_x > 0.0);

    let nearby = engine.input_layout(
        value,
        &style,
        Size::new(80.0, 24.0),
        pos(value.len() - 1),
        None,
        reveal(Point::new(end.scroll_x, end.scroll_y)),
    );
    assert_eq!(nearby.scroll_x, end.scroll_x);

    let start = engine.input_layout(
        value,
        &style,
        Size::new(80.0, 24.0),
        pos(0),
        None,
        reveal(Point::new(end.scroll_x, end.scroll_y)),
    );
    assert_eq!(start.scroll_x, 0.0);
}

#[test]
fn preserved_scroll_ignores_the_caret_and_only_clamps_to_new_content_bounds() {
    let value = "A deliberately long editable value";
    let style = TextStyle {
        wrap: TextWrap::None,
        ..TextStyle::default()
    };
    let mut engine = engine();
    let preserved = engine.input_layout(
        value,
        &style,
        Size::new(100.0, 24.0),
        pos(value.len()),
        None,
        TextInputScroll::new(Point::new(10.0, 0.0), CaretScroll::Preserve),
    );
    assert_eq!(preserved.scroll_x, 10.0);

    let clamped = engine.input_layout(
        value,
        &style,
        Size::new(100.0, 24.0),
        pos(value.len()),
        None,
        TextInputScroll::new(Point::new(f32::MAX, 0.0), CaretScroll::Preserve),
    );
    assert!(clamped.scroll_x.is_finite());
    assert!(clamped.scroll_x < f32::MAX);
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
        reveal(Point::default()),
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
        let cached = engine.input_layout(
            value,
            &style,
            viewport,
            pos(0),
            None,
            reveal(Point::default()),
        );
        assert!(!cached.stops.is_empty());
    }

    let inside_codepoint = engine.input_layout(
        "é",
        &style,
        Size::new(40.0, 24.0),
        pos(1),
        None,
        reveal(Point::default()),
    );
    assert_eq!(inside_codepoint.caret.origin.x, 0.0);
}

#[test]
fn combining_marks_emit_one_visual_stop_per_position() {
    let value = "Hello · مرحباً · שלום · 👋🏽";
    let style = TextStyle {
        wrap: TextWrap::None,
        ..TextStyle::default()
    };
    let layout = engine().input_layout(
        value,
        &style,
        Size::new(500.0, 24.0),
        pos(0),
        None,
        reveal(Point::default()),
    );

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
    let base = engine.input_layout(
        value,
        &style,
        Size::new(500.0, 24.0),
        pos(start),
        None,
        reveal(Point::default()),
    );
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
            reveal(Point::default()),
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
    let base = engine.input_layout(
        value,
        &style,
        Size::new(500.0, 24.0),
        pos(0),
        None,
        reveal(Point::default()),
    );
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
            reveal(Point::default()),
        );
        let rect = selection.selection[0];
        assert!((rect.origin.x - anchor.point.x).abs() < 0.01);
        assert!(rect.origin.x + rect.size.width + 0.01 >= previous_right);
        previous_right = stop.point.x;
    }
}

#[test]
fn multiline_layout_wraps_and_keeps_the_caret_vertically_visible() {
    let value = "first line wraps across the narrow viewport\nsecond line\nthird line";
    let style = TextStyle {
        font_size: 16.0,
        line_height: 20.0,
        wrap: TextWrap::WordOrGlyph,
        ..TextStyle::default()
    };
    let layout = engine().input_layout(
        value,
        &style,
        Size::new(120.0, 42.0),
        pos(value.len()),
        Some((pos(0), pos(value.len()))),
        reveal(Point::default()),
    );
    assert!(layout.scroll_y > 0.0);
    assert!(layout.caret.origin.y + layout.caret.size.height <= 44.0);
    assert!(layout.selection.len() >= 2);
    assert!(
        layout
            .stops
            .windows(2)
            .all(|pair| pair[0].point.y <= pair[1].point.y + 0.01)
    );
}

#[test]
fn empty_multiline_runs_keep_a_real_caret_stop_and_scroll_position() {
    let value = "start\n\n\n\n\n";
    let style = TextStyle {
        font_size: 16.0,
        line_height: 20.0,
        wrap: TextWrap::WordOrGlyph,
        ..TextStyle::default()
    };
    let layout = engine().input_layout(
        value,
        &style,
        Size::new(120.0, 42.0),
        pos(value.len()),
        None,
        reveal(Point::default()),
    );

    assert!(layout.scroll_y > 0.0);
    assert!(layout.caret.origin.y > 0.0);
    assert!(layout.caret.origin.y + layout.caret.size.height <= 42.0);
    assert!(
        layout
            .stops
            .iter()
            .any(|stop| stop.position == pos(value.len()))
    );
}

#[test]
fn large_code_buffers_build_unique_caret_stops_in_source_order() {
    let value = (0..600)
        .map(|line| format!("let value_{line} = compute({line}); // retained editor geometry"))
        .collect::<Vec<_>>()
        .join("\n");
    let style = TextStyle {
        font_size: 14.0,
        line_height: 20.0,
        wrap: TextWrap::None,
        ..TextStyle::default()
    };
    let layout = engine().input_layout(
        &value,
        &style,
        Size::new(720.0, 400.0),
        pos(value.len()),
        None,
        reveal(Point::default()),
    );
    let unique = layout
        .stops
        .iter()
        .map(|stop| (stop.position.index, stop.position.affinity))
        .collect::<Vec<_>>();

    assert!(layout.content_size.height > 10_000.0);
    assert!(unique.windows(2).all(|pair| pair[0] != pair[1]));
    assert!(layout.stops.iter().any(|stop| stop.word_boundary));
}

#[test]
fn scrolled_caret_stops_keep_their_source_lines() {
    let mut value = (0..25)
        .map(|line| format!("    let line_{line:03} = {line};"))
        .collect::<Vec<_>>()
        .join("\n");
    value.push('\n');
    let rich = TextContent::rich(value.split_inclusive('\n').enumerate().map(|(line, text)| {
        TextSpan::new(text).style(TextSpanStyle::default().color(if line % 2 == 0 {
            argui_core::Color::BLACK
        } else {
            argui_core::Color::WHITE
        }))
    }));
    let style = TextStyle {
        font_size: 13.5,
        line_height: 21.0,
        wrap: TextWrap::None,
        ..TextStyle::default()
    };
    let scroll = 63.0;
    let line = 11;
    let index: usize = value.lines().take(line).map(|line| line.len() + 1).sum();
    for content in [TextContent::plain(value), rich] {
        let layout = engine().input_layout_content(
            &content,
            &style,
            Size::new(580.0, 483.0),
            pos(0),
            None,
            TextInputScroll::new(Point::new(0.0, scroll), CaretScroll::Preserve),
        );
        let stop = layout
            .stops
            .iter()
            .find(|stop| stop.position.index == index)
            .unwrap();

        assert!(
            (stop.point.y - (line as f32 * style.line_height - scroll)).abs() < 0.01,
            "stop {stop:?}, resolved scroll {}",
            layout.scroll_y
        );
    }
}

#[test]
fn million_character_input_burst_shapes_a_bounded_source_window() {
    let mut value = "a".repeat(1_000_000);
    let mut text = engine();
    let style = TextStyle {
        wrap: TextWrap::None,
        ..TextStyle::default()
    };
    let viewport = Size::new(400.0, 40.0);
    let start_content = TextContent::plain(value.as_str());
    let at_start = text.input_layout_content(
        &start_content,
        &style,
        viewport,
        pos(0),
        None,
        reveal(Point::default()),
    );
    assert!(at_start.content_size.width > viewport.width * 1_000.0);
    for _ in 0..32 {
        let content = TextContent::plain(value.as_str());
        let layout = text.input_layout_content(
            &content,
            &style,
            viewport,
            pos(value.len()),
            None,
            reveal(Point::default()),
        );
        let window = text
            .input_window(&content, &style, viewport, layout.scroll_y)
            .unwrap();
        assert!(window.byte_range.end - window.byte_range.start < 1_000);
        assert!(window.x > 0.0);
        assert!(
            layout
                .stops
                .iter()
                .any(|stop| stop.position.index == value.len())
        );
        assert!(layout.caret.origin.x >= 0.0 && layout.caret.origin.x <= viewport.width);
        value.push('x');
    }
}

#[test]
fn million_byte_unicode_window_preserves_character_boundaries() {
    let value = "é".repeat(500_000);
    let content = TextContent::plain(&value);
    let style = TextStyle {
        wrap: TextWrap::None,
        ..TextStyle::default()
    };
    let viewport = Size::new(400.0, 40.0);
    let mut text = engine();
    let layout = text.input_layout_content(
        &content,
        &style,
        viewport,
        pos(value.len()),
        None,
        reveal(Point::default()),
    );
    let window = text
        .input_window(&content, &style, viewport, layout.scroll_y)
        .unwrap();
    assert!(value.is_char_boundary(window.byte_range.start));
    assert!(value.is_char_boundary(window.byte_range.end));
    assert!(window.byte_range.end - window.byte_range.start < 1_000);
    assert!(
        layout
            .stops
            .iter()
            .any(|stop| stop.position.index == value.len())
    );
}
