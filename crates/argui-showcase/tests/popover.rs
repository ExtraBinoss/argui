use argui_animation::{Duration, Frame, Time};
use argui_core::Size;
use argui_layout::LayoutEngine;
use argui_runtime::{LayoutSnapshot, ViewUpdate, WindowEnvironment};
use argui_showcase::{StateShowcase, text_engine};
use argui_ui::{ClickEvent, Element, UiEvent, UiEventKind, UiTree};

#[test]
fn layout_is_resolved_without_a_model_rebuild() {
    let mut app = StateShowcase::default();
    assert_eq!(
        app.layout_changed(&LayoutSnapshot::default()),
        ViewUpdate::None
    );
    open_popover(&mut app);
    let view = app.view(WindowEnvironment::default());
    let popover = element_by_key(&view, "effects-popover").unwrap();
    assert!(popover.interaction.is_some());
    assert!(popover.scroll.is_some());
    assert_eq!(popover.style.size.height, argui_ui::length(430.0));

    let mut tree = UiTree::new(view);
    let mut layout = LayoutEngine::new();
    let output = layout
        .compute(&mut tree, &mut text_engine(), Size::new(500.0, 400.0))
        .unwrap();
    let bounds = |key: &str| {
        output
            .nodes
            .iter()
            .find(|node| tree.key(node.node) == Some(key))
            .map(|node| node.bounds)
            .unwrap()
    };
    let anchor = bounds("popover-toggle");
    let placed = bounds("effects-popover");
    assert!(placed.origin.y + placed.size.height <= output.viewport.size.height - 14.0);
    assert!(placed.origin.y + placed.size.height <= anchor.origin.y - 12.0);
    assert_eq!(
        app.layout_changed(&LayoutSnapshot::default()),
        ViewUpdate::None
    );
}

fn open_popover(app: &mut StateShowcase) {
    let view = app.view(WindowEnvironment::default());
    let index = node_index(&view, "popover-toggle");
    let tree = UiTree::new(view);
    let event = UiEvent::new(
        tree.node_id_at(index).unwrap(),
        Some("popover-toggle".into()),
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    assert_eq!(app.update(&event), ViewUpdate::None);
    assert_eq!(
        app.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::ZERO,
        }),
        ViewUpdate::None
    );
    assert_eq!(
        app.animation_frame(Frame {
            now: Time::from_nanos(16_000_000),
            elapsed: Duration::from_millis(16),
        }),
        ViewUpdate::Rebuild
    );
}

fn node_index(root: &Element, key: &str) -> usize {
    fn visit(element: &Element, key: &str, index: &mut usize) -> Option<usize> {
        let current = *index;
        *index += 1;
        if element.key.as_deref() == Some(key) {
            return Some(current);
        }
        element
            .children
            .iter()
            .find_map(|child| visit(child, key, index))
    }
    visit(root, key, &mut 0).unwrap()
}

fn element_by_key<'a>(element: &'a Element, key: &str) -> Option<&'a Element> {
    if element.key.as_deref() == Some(key) {
        return Some(element);
    }
    element
        .children
        .iter()
        .find_map(|child| element_by_key(child, key))
}
