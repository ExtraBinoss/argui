use super::*;
use argui::core::Point;

#[test]
fn conversation_controls_update_reactions_file_details_and_reading_position() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::attachment");
    click(&app, "attachment-action-2");
    assert!(contains_text(&app.render(), "File details are ready."));
    click(&app, "nav::item");
    click(&app, "item-open");
    assert!(contains_text(&app.render(), "Opened Website redesign"));
    click(&app, "nav::message");
    click(&app, "message-like");
    assert!(contains_text(&app.render(), "Message liked"));
    click(&app, "message-like");
    assert!(contains_text(&app.render(), "Reaction removed"));
    click(&app, "nav::message-scroller");
    let mounted = app.mount().unwrap();
    let mut tree = UiTree::new(mounted.render(Default::default()).unwrap());
    let output = LayoutEngine::new()
        .compute(&mut tree, &mut TextEngine::new(), Size::new(1000.0, 800.0))
        .unwrap();
    let layout = argui::runtime::LayoutSnapshot {
        viewport: output.viewport,
        nodes: output
            .nodes
            .iter()
            .map(|node| argui::runtime::LayoutBounds {
                node: node.node,
                key: tree.key(node.node).map(str::to_owned),
                bounds: node.bounds,
            })
            .collect(),
    };
    mounted.layout_changed(&layout).unwrap();
    dispatch(
        &app,
        "chat",
        UiEventKind::Scrolled {
            delta: Point::new(0.0, -50.0),
            offset: Point::default(),
        },
    );
    assert!(keyed(&app.render(), "chat::latest").is_some());
    click(&app, "chat-add");
    assert!(keyed(&app.render(), "message-8").is_some());
    // The count is applied by the layout callback and must invalidate the retained view.
    mounted.layout_changed(&layout).unwrap();
    assert!(contains_text(&app.render(), "Jump to latest (1)"));
    click(&app, "chat::latest");
    assert!(keyed(&app.render(), "chat::latest").is_none());
}
