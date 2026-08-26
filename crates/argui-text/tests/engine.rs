use argui_text::TextEngine;

#[test]
fn text_engine_owns_a_reusable_font_system() {
    let mut engine = TextEngine::new();
    let _fonts = engine.fonts_mut();
}
