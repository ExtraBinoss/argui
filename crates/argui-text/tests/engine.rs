use argui_core::{Point, Rect, Size};
use argui_text::{FontFamily, TextBlock, TextEngine, TextScene, TextStyle, TextWrap};

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
