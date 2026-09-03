use argui_core::{CaretAffinity, Point, Rect, Size, TextPosition};
use argui_text::{CaretStop, TextLayout, TextLineLayout, line_range, word_range};

fn position(index: usize) -> TextPosition {
    TextPosition::new(index, CaretAffinity::After)
}

fn layout() -> TextLayout {
    TextLayout {
        stops: vec![
            CaretStop {
                position: position(0),
                point: Point::new(0.0, 0.0),
                word_boundary: true,
            },
            CaretStop {
                position: position(2),
                point: Point::new(20.0, 0.0),
                word_boundary: false,
            },
            CaretStop {
                position: position(4),
                point: Point::new(40.0, 0.0),
                word_boundary: true,
            },
            CaretStop {
                position: position(5),
                point: Point::new(0.0, 20.0),
                word_boundary: true,
            },
            CaretStop {
                position: position(8),
                point: Point::new(30.0, 20.0),
                word_boundary: true,
            },
        ],
        lines: vec![
            TextLineLayout {
                source: 0..4,
                bounds: Rect::new(Point::new(0.0, 0.0), Size::new(40.0, 16.0)),
            },
            TextLineLayout {
                source: 5..8,
                bounds: Rect::new(Point::new(0.0, 20.0), Size::new(30.0, 16.0)),
            },
        ],
        content_size: Size::new(40.0, 36.0),
    }
}

#[test]
fn hit_testing_is_exact_while_closest_positions_clamp_to_visual_lines() {
    let layout = layout();

    assert_eq!(
        layout.hit_position(Point::new(19.0, 8.0)),
        Some(position(2))
    );
    assert_eq!(layout.hit_position(Point::new(50.0, 8.0)), None);
    assert_eq!(layout.hit_position(Point::new(10.0, 18.0)), None);
    assert_eq!(
        layout.closest_position(Point::new(-20.0, -30.0)),
        position(0)
    );
    assert_eq!(
        layout.closest_position(Point::new(100.0, 100.0)),
        position(8)
    );
    assert_eq!(
        TextLayout::default().closest_position(Point::default()),
        TextPosition::default()
    );
}

#[test]
fn selection_rects_support_reverse_and_multiline_ranges() {
    let layout = layout();

    assert!(layout.selection_rects(position(2), position(2)).is_empty());
    let forward = layout.selection_rects(position(2), position(8));
    let reverse = layout.selection_rects(position(8), position(2));
    assert_eq!(forward, reverse);
    assert_eq!(forward.len(), 2);
    assert_eq!(forward[0].origin, Point::new(20.0, 0.0));
    assert_eq!(forward[1].size.width, 30.0);

    let line_without_stops = TextLayout {
        lines: vec![TextLineLayout {
            source: 20..30,
            bounds: Rect::new(Point::default(), Size::new(40.0, 16.0)),
        }],
        ..TextLayout::default()
    };
    assert!(
        line_without_stops
            .selection_rects(position(20), position(30))
            .is_empty()
    );
}

#[test]
fn semantic_ranges_handle_separators_unicode_and_invalid_offsets() {
    let text = "hello, café\nlast line";
    assert_eq!(word_range(text, 1), 0..5);
    assert_eq!(word_range(text, 5), 5..5);
    assert_eq!(word_range(text, 9), 7..12);
    assert_eq!(word_range(text, usize::MAX), text.len()..text.len());
    assert_eq!(line_range(text, 0), 0..12);
    assert_eq!(line_range(text, 14), 13..22);
    assert_eq!(line_range(text, usize::MAX), 13..22);
}
