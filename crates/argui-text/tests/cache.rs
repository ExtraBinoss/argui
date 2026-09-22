use argui_core::{Color, Point, Rect, Size, TextPosition};
use argui_text::{
    CaretScroll, TextAlign, TextBlock, TextContent, TextDecoration, TextEngine, TextInputScroll,
    TextScene, TextSpan, TextSpanStyle, TextStyle, UnderlineStyle,
};

const FONT: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");
const MONO: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/FiraMono-Medium.ttf");

/// Creates a deterministic engine with two font families for invalidation checks.
fn engine() -> TextEngine {
    TextEngine::from_embedded_fonts([FONT, MONO], "Noto Sans", "Noto Sans", "Fira Mono")
}

/// Makes one block with a fixed viewport for logical-cache checks.
fn scene(content: impl Into<TextContent>) -> TextScene {
    TextScene::new().with(TextBlock::new(
        content,
        Rect::new(Point::new(0.0, 0.0), Size::new(300.0, 60.0)),
    ))
}

/// Paint, DPI and translation reuse shaping while preserving precise colors.
#[test]
fn paint_and_physical_changes_reuse_logical_layout() {
    let mut engine = engine();
    let mut scene = scene("Precision ffi العربية");
    let first = engine.prepare(&scene, 1.0);
    let misses = engine.stats().layout_misses;
    let color = Color::srgba(0.03, 0.12, 0.37, 0.63);
    scene.blocks_mut()[0].style.color = color;
    let recolored = engine.prepare(&scene, 1.0);
    assert_eq!(first.glyphs[0].key, recolored.glyphs[0].key);
    assert_eq!(recolored.glyphs[0].color, color.to_linear_rgba());
    scene.blocks_mut()[0].bounds.origin = Point::new(12.25, -3.5);
    let moved = engine.prepare(&scene, 1.5);
    assert_ne!(first.glyphs[0].key, moved.glyphs[0].key);
    assert_eq!(engine.stats().layout_misses, misses);
    assert_eq!(engine.stats().layout_hits, 2);
}

/// Rich span text and decoration colors are resolved after the layout-cache lookup.
#[test]
fn rich_recolor_reuses_measurement_selection_and_shape() {
    let mut engine = engine();
    let red = Color::srgb(0.03, 0.02, 0.01);
    let blue = Color::srgba(0.17, 0.41, 0.83, 0.59);
    let content = |color| {
        TextContent::rich([
            TextSpan::new("first ").style(TextSpanStyle::default().color(color)),
            TextSpan::new("second").style(TextSpanStyle::default().decoration(TextDecoration {
                underline: UnderlineStyle::Double,
                underline_color: Some(color),
                strikethrough: true,
                strikethrough_color: Some(color),
            })),
        ])
    };
    let mut scene = scene(content(red));
    let style = scene.blocks()[0].style.clone();
    let size = scene.blocks()[0].bounds.size;
    let _ = engine.measure_content(&content(red), &style, Some(size.width));
    let original_layout = engine.layout_text(&content(red), &style, size);
    engine.prepare(&scene, 1.0);
    let before = engine.stats().layout_misses;
    scene.blocks_mut()[0].content = content(blue);
    let _ = engine.measure_content(&content(blue), &style, Some(size.width));
    let next_layout = engine.layout_text(&content(blue), &style, size);
    let prepared = engine.prepare(&scene, 1.0);
    assert!(std::sync::Arc::ptr_eq(&original_layout, &next_layout));
    assert_eq!(engine.stats().layout_misses, before);
    assert_eq!(prepared.glyphs[0].color, blue.to_linear_rgba());
    assert!(prepared.decorations.len() >= 3);
    assert!(
        prepared
            .decorations
            .iter()
            .all(|item| item.color == blue.to_linear_rgba())
    );
}

/// Alignment must change caret geometry even when text and viewport stay equal.
#[test]
fn alignment_has_distinct_selection_geometry() {
    let mut engine = engine();
    let content = TextContent::plain("same text");
    let size = Size::new(300.0, 60.0);
    let left = engine.layout_text(&content, &TextStyle::default(), size);
    for align in [TextAlign::Center, TextAlign::Right, TextAlign::End] {
        let style = TextStyle {
            align,
            ..TextStyle::default()
        };
        let aligned = engine.layout_text(&content, &style, size);
        assert!(aligned.lines[0].bounds.origin.x > left.lines[0].bounds.origin.x);
        assert!(aligned.stops[0].point.x > left.stops[0].point.x);
    }
}

/// Font mutations invalidate retained editors, logical layouts and GPU/CPU glyph identities.
#[test]
fn font_mutation_invalidates_all_text_tiers() {
    let mut engine = engine();
    engine
        .fonts_mut()
        .db_mut()
        .set_monospace_family("Noto Sans");
    let mut scene = scene("iiiiWWWW");
    let style = TextStyle {
        family: argui_text::FontFamily::Monospace,
        ..TextStyle::default()
    };
    scene.blocks_mut()[0].style = style.clone();
    let size = Size::new(300.0, 60.0);
    let scroll = TextInputScroll::new(Point::new(0.0, 0.0), CaretScroll::Preserve);
    let cursor = TextPosition::default();
    let before = engine.input_layout("iiiiWWWW", &style, size, cursor, None, scroll);
    let prepared = engine.prepare(&scene, 1.0);
    engine.rasterize(prepared.glyphs[0].key);
    let misses = engine.stats().layout_misses;
    let fonts = engine.fonts_mut();
    let family = fonts
        .db()
        .faces()
        .find(|face| face.post_script_name.starts_with("FiraMono"))
        .unwrap()
        .families[0]
        .0
        .clone();
    fonts.db_mut().set_monospace_family(family);
    assert_eq!(engine.stats().raster_entries, 0);
    let after = engine.input_layout("iiiiWWWW", &style, size, cursor, None, scroll);
    let refreshed = engine.prepare(&scene, 1.0);
    assert_ne!(before.stops, after.stops);
    assert_ne!(prepared.glyphs[0].key, refreshed.glyphs[0].key);
    assert_eq!(engine.stats().layout_misses, misses + 2);
    assert_ne!(
        refreshed.glyphs[0].key,
        self::engine().prepare(&scene, 1.0).glyphs[0].key
    );
}

/// Rich editor colors do not discard persistent shaping or caret geometry.
#[test]
fn editor_recolor_reuses_its_buffer() {
    let mut engine = engine();
    let size = Size::new(300.0, 60.0);
    let scroll = TextInputScroll::new(Point::new(0.0, 0.0), CaretScroll::Preserve);
    let content = |color| {
        TextContent::rich([TextSpan::new("editable").style(TextSpanStyle::default().color(color))])
    };
    let style = TextStyle::default();
    let first = engine.input_layout_content(
        &content(Color::WHITE),
        &style,
        size,
        TextPosition::default(),
        None,
        scroll,
    );
    let misses = engine.stats().layout_misses;
    let recolored = TextStyle {
        color: Color::BLACK,
        ..style
    };
    let second = engine.input_layout_content(
        &content(Color::BLACK),
        &recolored,
        size,
        TextPosition::default(),
        None,
        scroll,
    );
    assert_eq!(first, second);
    assert_eq!(engine.stats().layout_misses, misses);
    assert!(engine.stats().layout_hits > 0);
}

/// Resizing an unwrapped start-aligned label only changes its clip, not its layout.
#[test]
fn unconstrained_labels_reuse_layout_across_width_changes() {
    let mut engine = engine();
    let mut scene = scene("stable label");
    scene.blocks_mut()[0].style.wrap = argui_text::TextWrap::None;
    let first = engine.prepare(&scene, 1.0);
    scene.blocks_mut()[0].bounds.size.width = 40.0;
    let second = engine.prepare(&scene, 1.0);
    assert_eq!(first, second);
    assert_eq!(engine.stats().layout_misses, 1);
}
