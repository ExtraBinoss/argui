use argui_runtime::{UiApp, ViewUpdate};
use argui_ui::{Element, UiEvent, UiEventKind, UiTree};

#[derive(Default)]
struct Counter(u32);

impl UiApp for Counter {
    fn view(&self) -> Element {
        Element::text(self.0.to_string())
    }

    fn update(&mut self, event: &UiEvent) -> ViewUpdate {
        if event.kind == UiEventKind::Clicked {
            self.0 += 1;
            ViewUpdate::Rebuild
        } else {
            ViewUpdate::None
        }
    }
}

#[test]
fn apps_rebuild_only_when_their_state_changes() {
    let mut app = Counter::default();
    let tree = UiTree::new(Element::container([]));
    let moved = UiEvent {
        target: tree.node_id_at(0).unwrap(),
        key: Some("increment".into()),
        kind: UiEventKind::PointerEntered,
    };
    let clicked = UiEvent {
        kind: UiEventKind::Clicked,
        ..moved.clone()
    };

    assert_eq!(app.update(&moved), ViewUpdate::None);
    assert_eq!(app.update(&clicked), ViewUpdate::Rebuild);
    assert!(
        matches!(app.view().kind, argui_ui::ElementKind::Text { content, .. } if content == "1")
    );
}
