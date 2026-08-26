use argui_core::{Point, Rect, Size};
use argui_text::{FontFamily, TextBlock, TextEngine, TextScene, TextStyle, TextWrap};

const NOTO_SANS: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");
const NOTO_ARABIC: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSansArabic.ttf");
const NOTO_HEBREW: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSansHebrew.ttf");

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
