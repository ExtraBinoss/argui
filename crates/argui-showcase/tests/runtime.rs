use argui_animation::{Duration, Frame, Time};
use argui_core::ColorScheme;
use argui_runtime::{Entity, ViewUpdate, WindowEnvironment};
use argui_showcase::StateShowcase;
use argui_ui::{ClickEvent, Element, UiEventKind, UiTree};

fn has_key(element: &Element, key: &str) -> bool {
    element.key.as_deref() == Some(key) || element.children.iter().any(|child| has_key(child, key))
}

fn click(app: &Entity<StateShowcase>, key: &str) {
    let mut tree = UiTree::new(app.render());
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some(key))
        .expect("showcase control should be present");
    for event in tree.event_deliveries(target, UiEventKind::Click(ClickEvent::accessibility())) {
        if event.should_dispatch() {
            app.dispatch_event(&event);
        }
    }
}

fn advance(app: &Entity<StateShowcase>, frame_index: u64) {
    app.update(|showcase, cx| {
        let update = showcase.animation_frame(Frame {
            now: Time::from_nanos(frame_index * 16_000_000),
            elapsed: Duration::from_millis(16),
        });
        if update == ViewUpdate::Rebuild {
            cx.notify();
        }
    });
}

#[test]
fn runtime_render_uses_the_same_tree_for_light_and_dark_environments() {
    let app = Entity::new(StateShowcase::default());
    let light = app.render_in(WindowEnvironment {
        color_scheme: ColorScheme::Light,
        ..WindowEnvironment::default()
    });
    let dark = app.render_in(WindowEnvironment {
        color_scheme: ColorScheme::Dark,
        ..WindowEnvironment::default()
    });
    assert!(has_key(&light, "page-scroll"));
    assert!(has_key(&dark, "page-scroll"));
    assert!(has_key(&light, "popover-toggle"));
    assert!(has_key(&dark, "popover-toggle"));
}

#[test]
fn runtime_dispatch_rebuilds_the_effects_popover_through_public_events() {
    let app = Entity::new(StateShowcase::default());
    assert!(!has_key(&app.render(), "effects-popover"));
    click(&app, "popover-toggle");
    assert!(!has_key(&app.render(), "effects-popover"));
    advance(&app, 1);
    assert!(has_key(&app.render(), "effects-popover"));
    click(&app, "popover-toggle");
    for frame in 2..=120 {
        advance(&app, frame);
    }
    assert!(!has_key(&app.render(), "effects-popover"));
}
