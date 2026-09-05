use argui_core::{Affine2D, Point, Rect, ScrollDelta, Size};
use argui_paint::{ClipChain, ClipRegion};
use argui_runtime::{Entity, WindowEnvironment};
use argui_showcase::StateShowcase;
use argui_ui::{ScrollConfig, ScrollRegion, UiTree};

fn node_index(root: &argui_ui::Element, key: &str) -> usize {
    fn visit(element: &argui_ui::Element, key: &str, index: &mut usize) -> Option<usize> {
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

#[test]
fn virtual_scroll_rebuilds_only_when_the_visible_window_changes() {
    let app = Entity::new(StateShowcase::default());
    let root = app.render_in(WindowEnvironment::default());
    let index = node_index(&root, "million-list");
    let mut tree = UiTree::new(root);
    let node = tree.node_id_at(index).unwrap();
    let bounds = Rect::new(Point::default(), Size::new(300.0, 260.0));
    let regions = [ScrollRegion {
        node,
        bounds,
        clip: bounds,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::from_regions([ClipRegion::new(bounds, Affine2D::IDENTITY)]),
        max_offset: Point::new(0.0, 1_000.0),
        config: ScrollConfig::default().line_size(36.0),
        scrollbar: None,
        interaction_order: 0,
    }];
    for step in 1..=9 {
        let update = tree.scroll(
            Point::new(10.0, 10.0),
            ScrollDelta::Lines(Point::new(0.0, -1.0)),
            &regions,
        );
        for event in &update.events {
            if event.should_dispatch() {
                app.dispatch_event(event);
            }
        }
        let expected = if step == 9 {
            argui_ui::TreeUpdate::Layout
        } else {
            argui_ui::TreeUpdate::None
        };
        assert_eq!(tree.update(app.render()), expected);
    }
    let sub_row = tree.scroll(
        Point::new(10.0, 10.0),
        ScrollDelta::Pixels(Point::new(0.0, -1.0)),
        &regions,
    );
    for event in &sub_row.events {
        if event.should_dispatch() {
            app.dispatch_event(event);
        }
    }
    assert_eq!(tree.update(app.render()), argui_ui::TreeUpdate::None);
}
