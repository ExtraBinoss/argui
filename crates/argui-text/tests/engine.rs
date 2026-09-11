use std::num::NonZeroUsize;

use argui_core::{Point, Rect, Size};
use argui_text::{
    FontFamily, FontStyle, LetterSpacing, TextBlock, TextContent, TextDecoration, TextEngine,
    TextOverflow, TextScene, TextSpan, TextSpanStyle, TextStyle, TextWrap, UnderlineStyle,
};

const NOTO_SANS: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");
const NOTO_ARABIC: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSansArabic.ttf");
const NOTO_HEBREW: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSansHebrew.ttf");
const NOTO_EMOJI: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoEmoji-Regular.ttf");

fn bounds(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect::new(Point::new(x, y), Size::new(width, height))
}

#[test]
fn text_engine_owns_a_reusable_font_system() {
    let mut engine = TextEngine::new();
    let _fonts = engine.fonts_mut();
}

#[test]
fn measured_labels_survive_pixel_rounding_without_losing_the_last_glyph() {
    let mut engine =
        TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans");
    let style = TextStyle {
        font_size: 14.0,
        line_height: 20.0,
        wrap: TextWrap::WordOrGlyph,
        ..TextStyle::default()
    };
    for label in [
        "Components",
        "Breadcrumb",
        "Component library",
        "12",
        "A shared component library with a longer name",
    ] {
        let size = engine.measure(label, &style, None);
        let mut block = TextBlock::new(label, bounds(0.0, 0.0, size.width.round(), size.height));
        block.style = style.clone();
        let prepared = engine.prepare(&TextScene::new().with(block), 1.0);
        assert_eq!(
            prepared.glyphs.iter().map(|glyph| glyph.end).max(),
            Some(label.len()),
            "{label}"
        );
        assert_eq!(
            engine.measure(label, &style, Some(size.width)).height,
            style.line_height
        );
    }
}

#[test]
fn embedded_fonts_shape_rasterize_and_keep_selection_metadata() {
    let mut engine = TextEngine::from_embedded_fonts(
        [NOTO_SANS, NOTO_ARABIC, NOTO_HEBREW],
        "Noto Sans",
        "Noto Sans",
        "Noto Sans",
    );
    let scene = TextScene::new()
        .with(TextBlock::new(
            "Hello العربية שלום",
            bounds(10.0, 20.0, 300.0, 80.0),
        ))
        .with(
            TextBlock::new("Named", bounds(0.0, 0.0, 100.0, 30.0))
                .family(FontFamily::Named("Noto Sans".into())),
        )
        .with(TextBlock::new("Serif", bounds(0.0, 0.0, 100.0, 30.0)).family(FontFamily::Serif))
        .with(TextBlock::new("Mono", bounds(0.0, 0.0, 100.0, 30.0)).family(FontFamily::Monospace));

    let prepared = engine.prepare(&scene, 2.0);

    assert!(!prepared.glyphs.is_empty());
    assert_eq!(prepared.glyphs[0].clip, [20.0, 40.0, 620.0, 200.0]);
    assert!(prepared.glyphs.iter().any(|glyph| glyph.rtl));
    assert!(
        prepared
            .glyphs
            .iter()
            .all(|glyph| glyph.start < glyph.end && glyph.block < 4)
    );
    assert!(
        prepared
            .glyphs
            .iter()
            .any(|glyph| engine.rasterize(glyph.key).is_some())
    );
}

#[test]
fn no_wrap_labels_keep_their_intrinsic_width() {
    let mut engine =
        TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans");
    let wrapped = TextStyle {
        font_size: 17.0,
        line_height: 22.0,
        ..TextStyle::default()
    };
    let unwrapped = TextStyle {
        wrap: TextWrap::None,
        ..wrapped.clone()
    };

    let wrapped_size = engine.measure("Primary action", &wrapped, Some(55.0));
    let unwrapped_size = engine.measure("Primary action", &unwrapped, Some(55.0));

    assert!(wrapped_size.width <= 55.0);
    assert!(wrapped_size.height > wrapped.line_height);
    assert!(unwrapped_size.width > 55.0);
    assert_eq!(unwrapped_size.height, unwrapped.line_height);
}

#[test]
fn ellipsis_replaces_overflowing_graphemes_and_tracks_available_width() {
    let mut engine =
        TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans");
    let clipped_style = TextStyle {
        wrap: TextWrap::None,
        ..TextStyle::default()
    };
    let ellipsis_style = TextStyle {
        overflow: TextOverflow::Ellipsis(argui_text::EllipsisPosition::End),
        ..clipped_style.clone()
    };
    let scene = |text: &str, width: f32, style: TextStyle| {
        let mut block = TextBlock::new(text, bounds(0.0, 0.0, width, 24.0));
        block.style = style;
        TextScene::new().with(block)
    };

    let clipped = engine.prepare(
        &scene("Search components with accents é🙂", 110.0, clipped_style),
        1.0,
    );
    let shortened = engine.prepare(
        &scene("Search components with accents é🙂", 110.0, ellipsis_style),
        1.0,
    );
    let mark = engine.prepare(
        &scene(
            "…",
            110.0,
            TextStyle {
                wrap: TextWrap::None,
                ..TextStyle::default()
            },
        ),
        1.0,
    );
    let wide = engine.prepare(
        &scene(
            "Search components with accents é🙂",
            500.0,
            TextStyle {
                wrap: TextWrap::None,
                overflow: TextOverflow::Ellipsis(argui_text::EllipsisPosition::End),
                ..TextStyle::default()
            },
        ),
        1.0,
    );

    assert!(shortened.glyphs.len() < clipped.glyphs.len());
    assert_eq!(shortened.glyphs.last().unwrap().key, mark.glyphs[0].key);
    assert_eq!(wide.glyphs.len(), clipped.glyphs.len());
}

#[test]
fn prepared_glyphs_reposition_without_reshaping() {
    let mut engine =
        TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans");
    let scene = TextScene::new().with(TextBlock::new("Scroll", bounds(10.0, 20.0, 100.0, 30.0)));
    let mut prepared = engine.prepare(&scene, 2.0);
    let before = prepared.glyphs[0];
    let clip = bounds(0.0, 0.0, 200.0, 100.0);

    prepared.reposition_block(0, Point::new(10.0, 5.0), clip);

    assert_eq!(prepared.glyphs[0].key, before.key);
    assert_eq!(prepared.glyphs[0].y, before.y - 30);
    assert_eq!(prepared.glyphs[0].clip, [0.0, 0.0, 400.0, 200.0]);
}

#[test]
fn embedded_emoji_fallback_rasterizes_without_system_fonts() {
    let mut engine = TextEngine::from_embedded_fonts(
        [NOTO_SANS, NOTO_EMOJI],
        "Noto Sans",
        "Noto Sans",
        "Noto Sans",
    );
    let scene = TextScene::new().with(TextBlock::new("👋🏽 🎉", bounds(0.0, 0.0, 120.0, 30.0)));
    let prepared = engine.prepare(&scene, 1.0);

    assert!(!prepared.glyphs.is_empty());
    let visible = prepared
        .glyphs
        .iter()
        .filter_map(|glyph| engine.rasterize(glyph.key))
        .filter(|image| image.width > 0 && image.height > 0)
        .count();
    assert!(visible >= 2);
}

#[test]
fn rich_spans_keep_per_run_color_metrics_and_decorations() {
    let mut engine =
        TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans");
    let red = argui_core::Color::srgb(1.0, 0.0, 0.0);
    let green = argui_core::Color::srgb(0.0, 1.0, 0.0);
    let content = TextContent::rich([
        TextSpan::new("rich ").style(
            TextSpanStyle::default()
                .color(red)
                .weight(700)
                .font_style(FontStyle::Italic),
        ),
        TextSpan::new("text").style(TextSpanStyle::default().color(green).decoration(
            TextDecoration {
                underline: UnderlineStyle::Double,
                strikethrough: true,
                ..TextDecoration::default()
            },
        )),
    ]);
    let scene = TextScene::new().with(TextBlock::new(content, bounds(0.0, 0.0, 240.0, 40.0)));
    let prepared = engine.prepare(&scene, 1.0);

    assert!(prepared.glyphs.iter().any(|glyph| glyph.color[0] > 0.9));
    assert!(prepared.glyphs.iter().any(|glyph| glyph.color[1] > 0.9));
    assert!(prepared.decorations.len() >= 3);
}

#[test]
fn empty_rich_spans_do_not_create_empty_shape_runs() {
    let content = TextContent::rich([
        TextSpan::new(""),
        TextSpan::new("visible").style(TextSpanStyle::default().weight(600)),
    ]);
    assert_eq!(content.as_str(), "visible");
    assert!(content.is_rich());
}

#[test]
fn tracking_changes_intrinsic_measurement_and_layout_cache_is_reused() {
    let mut engine =
        TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans");
    let normal = TextStyle::default();
    let tracked = TextStyle {
        letter_spacing: LetterSpacing::Px(4.0),
        ..normal.clone()
    };
    assert!(
        engine.measure("tracking", &tracked, None).width
            > engine.measure("tracking", &normal, None).width
    );
    let content = TextContent::plain("same allocation");
    let first = engine.layout_text(&content, &tracked, Size::new(200.0, 80.0));
    let second = engine.layout_text(&content, &tracked, Size::new(200.0, 80.0));
    assert!(std::sync::Arc::ptr_eq(&first, &second));
}

#[test]
fn multiline_ellipsis_clamps_shaping_and_selection_to_visual_lines() {
    let mut engine =
        TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans");
    let content = TextContent::plain(
        "one two three four five six seven eight nine ten eleven twelve thirteen fourteen",
    );
    let style = TextStyle {
        font_size: 16.0,
        line_height: 20.0,
        wrap: TextWrap::Word,
        overflow: TextOverflow::Ellipsis(argui_text::EllipsisPosition::End),
        line_clamp: NonZeroUsize::new(2),
        ..TextStyle::default()
    };

    let layout = engine.layout_text(&content, &style, Size::new(120.0, 200.0));

    assert_eq!(layout.lines.len(), 2);
    assert!(layout.content_size.height <= 40.0);
    assert!(layout.lines.last().unwrap().source.end < content.as_str().len());
}
