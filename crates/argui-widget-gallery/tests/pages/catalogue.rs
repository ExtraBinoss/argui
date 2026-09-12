use super::*;
use argui::core::{Key, KeyInput, KeyState};

#[path = "catalogue/conversation.rs"]
mod conversation;
#[path = "catalogue/forms.rs"]
mod forms;
#[path = "catalogue/navigation.rs"]
mod navigation;
#[path = "catalogue/surfaces.rs"]
mod surfaces;

fn keyboard(app: &Entity<WidgetGallery>, target: &str, key: Key) {
    dispatch(
        app,
        target,
        UiEventKind::KeyInput(KeyInput {
            key,
            state: KeyState::Pressed,
            repeat: false,
            modifiers: Default::default(),
            text: None,
        }),
    );
}

#[test]
fn added_pages_are_reachable_and_keep_finite_layout_and_valid_accessible_relations() {
    let app = Entity::new(WidgetGallery::default());
    let mut layout = LayoutEngine::new();
    let mut text = TextEngine::new();
    let mut tree = UiTree::new(app.render());
    for slug in [
        "accordion",
        "alert-dialog",
        "attachment",
        "bubble",
        "button-group",
        "carousel",
        "chart",
        "combobox",
        "direction",
        "drawer",
        "field",
        "hover-card",
        "input-group",
        "input-otp",
        "item",
        "marker",
        "message",
        "message-scroller",
        "native-select",
        "navigation-menu",
        "questionnaire",
        "scroll-area",
        "sheet",
        "sidebar",
        "toggle",
        "toggle-group",
    ] {
        click(&app, &format!("nav::{slug}"));
        for scheme in [ColorScheme::Light, ColorScheme::Dark] {
            let root = app.render_in(WindowEnvironment {
                color_scheme: scheme,
                reduced_motion: true,
                ..Default::default()
            });
            assert!(keyed(&root, "catalogue-demo").is_some(), "{slug}");
            tree.update(root);
            let output = layout
                .compute(&mut tree, &mut text, Size::new(800.0, 720.0))
                .unwrap();
            assert!(
                tree.semantic_diagnostics().is_empty(),
                "{slug}: {:?}",
                tree.semantic_diagnostics()
            );
            assert!(
                output
                    .nodes
                    .iter()
                    .all(|node| node.bounds.origin.x.is_finite()
                        && node.bounds.origin.y.is_finite()
                        && node.bounds.size.width.is_finite()),
                "{slug}"
            );
        }
    }
}
