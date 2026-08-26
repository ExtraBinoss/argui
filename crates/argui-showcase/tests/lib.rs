use argui_runtime::UiApp;
use argui_showcase::{StateShowcase, text_engine};
use argui_text::TextStyle;

#[test]
fn shared_showcase_builds_one_tree_and_embeds_its_fonts() {
    let app = StateShowcase::default();
    let mut text = text_engine();

    assert_eq!(app.view().children.len(), 1);
    assert!(text.measure("Argui", &TextStyle::default(), None).width > 0.0);
}
