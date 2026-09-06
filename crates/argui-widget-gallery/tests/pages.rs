#[path = "pages/buttons.rs"]
mod buttons;
#[path = "pages/inputs.rs"]
mod inputs;
#[path = "pages/scroll_effects.rs"]
mod scroll_effects;
#[path = "pages/webview.rs"]
mod webview;

use argui::{
    core::{ColorScheme, Size},
    layout::LayoutEngine,
    runtime::{Entity, WindowEnvironment},
    text::TextEngine,
    ui::{ClickEvent, Element, UiEventKind, UiTree},
};
use argui_widget_gallery::WidgetGallery;

fn contains_text(element: &Element, expected: &str) -> bool {
    matches!(&element.kind, argui::ui::ElementKind::Text { content, .. }
        if content.as_str() == expected)
        || element
            .children
            .iter()
            .any(|child| contains_text(child, expected))
}

#[test]
fn all_pages_keep_finite_layout_when_narrow_dark_or_reduced_motion() {
    let app = Entity::new(WidgetGallery::default());
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    let mut tree = UiTree::new(app.render());
    for (slug, label) in [
        ("button", "Button"),
        ("input", "Input & Search"),
        ("textarea", "Text area"),
        ("checkbox", "Checkbox"),
        ("switch", "Switch"),
        ("radio-group", "Radio group"),
        ("slider", "Slider"),
        ("tabs", "Tabs"),
        ("select", "Select"),
        ("dialog", "Dialog"),
        ("form", "Profile form"),
        ("settings", "Settings panel"),
        ("layout", "Web layout"),
        ("motion", "Motion & loading"),
        ("effects", "GPU effects / WGSL"),
        ("composition", "Advanced composition"),
        ("typography", "Typography & selection"),
    ] {
        let key = format!("nav::{slug}");
        tree.update(app.render());
        let target = tree
            .node_ids()
            .iter()
            .copied()
            .find(|node| tree.key(*node) == Some(key.as_str()))
            .unwrap();
        for event in tree.event_deliveries(target, UiEventKind::Click(ClickEvent::accessibility()))
        {
            if event.should_dispatch() {
                app.dispatch_event(&event);
            }
        }
        for (color_scheme, width, reduced_motion) in [
            (ColorScheme::Light, 1220.0, false),
            (ColorScheme::Dark, 320.0, true),
        ] {
            let root = app.render_in(WindowEnvironment {
                color_scheme,
                reduced_motion,
                ..WindowEnvironment::default()
            });
            let content = find_content(&root).unwrap();
            assert!(contains_text(content, label), "wrong page content: {slug}");
            tree.update(root);
            let output = engine
                .compute(&mut tree, &mut text, Size::new(width, 780.0))
                .unwrap();
            for node in &output.nodes {
                let bounds = node.bounds;
                assert!(
                    bounds.origin.x.is_finite() && bounds.origin.y.is_finite(),
                    "{slug}"
                );
                assert!(
                    bounds.size.width.is_finite() && bounds.size.width >= 0.0,
                    "{slug}"
                );
                assert!(
                    bounds.size.height.is_finite() && bounds.size.height >= 0.0,
                    "{slug}"
                );
            }
        }
    }
}

fn find_content(element: &Element) -> Option<&Element> {
    if element.key.as_deref() == Some("gallery-content-scroll") {
        return Some(element);
    }
    element.children.iter().find_map(find_content)
}
