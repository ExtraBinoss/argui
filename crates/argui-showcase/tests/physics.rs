use argui_runtime::WindowEnvironment;
use argui_showcase::StateShowcase;

fn contains_text(element: &argui_ui::Element, needle: &str) -> bool {
    if let argui_ui::ElementKind::Text { content, .. } = &element.kind
        && content.as_str().contains(needle)
    {
        return true;
    }
    element
        .children
        .iter()
        .any(|child| contains_text(child, needle))
}

#[test]
fn physics_showcase_exposes_a_public_status_and_controls() {
    let app = StateShowcase::default();
    let root = app.view(WindowEnvironment::default());
    assert!(contains_text(&root, "Physics"));
    assert!(contains_text(&root, "Spring retarget"));
    assert!(contains_text(&root, "Launch inertia"));
}
