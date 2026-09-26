use web_time::Instant;

use argui_core::{Point, Size, TextPosition};
use argui_text::{CaretScroll, TextContent, TextEngine, TextInputScroll, TextStyle, TextWrap};

const NOTO_SANS: &[u8] = include_bytes!("../../../../assets/fonts/NotoSans-Regular.ttf");

fn report(label: &str, mut samples: Vec<std::time::Duration>) {
    samples.sort();
    eprintln!(
        "{label}: p50={:?} p95={:?} max={:?} / {} keys",
        samples[samples.len() / 2],
        samples[samples.len() * 95 / 100],
        samples[samples.len() - 1],
        samples.len()
    );
}

#[test]
#[ignore = "manual CPU benchmark; run with --ignored --nocapture"]
fn million_character_input_burst() {
    let mut engine =
        TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans");
    let style = TextStyle {
        wrap: TextWrap::None,
        ..TextStyle::default()
    };
    let viewport = Size::new(480.0, 30.0);
    let mut value = "a".repeat(1_000_000);
    let mut scroll = Point::default();
    let mut samples = Vec::new();
    for _ in 0..100 {
        value.push('a');
        let start = Instant::now();
        let layout = engine.input_layout_content(
            &TextContent::plain(value.clone()),
            &style,
            viewport,
            TextPosition::new(value.len(), Default::default()),
            None,
            TextInputScroll::new(scroll, CaretScroll::Reveal),
        );
        scroll = Point::new(layout.scroll_x, layout.scroll_y);
        assert!(layout.stops.len() < 500);
        samples.push(start.elapsed());
    }
    report("1M ASCII append + layout", samples);

    let mut value = "line\n".repeat(200_000);
    let mut scroll = Point::default();
    let mut samples = Vec::new();
    for _ in 0..100 {
        value.push('a');
        let start = Instant::now();
        let layout = engine.input_layout_content(
            &TextContent::plain(value.clone()),
            &style,
            Size::new(480.0, 240.0),
            TextPosition::new(value.len(), Default::default()),
            None,
            TextInputScroll::new(scroll, CaretScroll::Reveal),
        );
        scroll = Point::new(layout.scroll_x, layout.scroll_y);
        assert!(layout.stops.len() < 500);
        samples.push(start.elapsed());
    }
    report("1M multiline append + layout", samples);

    let mut value = "مرحبا".repeat(200_000);
    let mut samples = Vec::new();
    for key in 0..100 {
        let start = Instant::now();
        value.insert(1_000_000 + key, 'x');
        let layout = engine.input_layout_content(
            &TextContent::plain(value.clone()),
            &style,
            viewport,
            TextPosition::new(1_000_001 + key, Default::default()),
            None,
            TextInputScroll::new(Point::default(), CaretScroll::Reveal),
        );
        assert!(layout.stops.len() < 500);
        samples.push(start.elapsed());
    }
    report(
        "1M Unicode scalars / 2M bytes middle insert + layout",
        samples,
    );

    let mut value = "line\n".repeat(200_000);
    let mut samples = Vec::new();
    let mut scroll = Point::default();
    for key in 0..100 {
        let start = Instant::now();
        value.insert(500_000 + key, 'x');
        let layout = engine.input_layout_content(
            &TextContent::plain(value.clone()),
            &style,
            Size::new(480.0, 240.0),
            TextPosition::new(500_001 + key, Default::default()),
            None,
            TextInputScroll::new(scroll, CaretScroll::Reveal),
        );
        scroll = Point::new(layout.scroll_x, layout.scroll_y);
        assert!(layout.stops.len() < 500);
        samples.push(start.elapsed());
    }
    report("1M multiline middle insert + layout", samples);
}
