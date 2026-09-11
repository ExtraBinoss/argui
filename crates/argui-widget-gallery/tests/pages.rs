#[path = "pages/buttons.rs"]
mod buttons;
#[path = "pages/data.rs"]
mod data;
#[path = "pages/inputs.rs"]
mod inputs;
#[path = "pages/liquid_glass.rs"]
mod liquid_glass;
#[path = "pages/scroll_effects.rs"]
mod scroll_effects;
#[path = "pages/timeline.rs"]
mod timeline;
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
        ("badge", "Badge"),
        ("avatar", "Avatar"),
        ("empty", "Empty"),
        ("kbd", "Kbd"),
        ("label", "Label"),
        ("skeleton", "Skeleton"),
        ("breadcrumb", "Breadcrumb"),
        ("pagination", "Pagination"),
        ("progress", "Progress"),
        ("aspect-ratio", "Aspect ratio"),
        ("card", "Card"),
        ("alert", "Alert"),
        ("separator", "Separator"),
        ("collapsible", "Collapsible"),
        ("input", "Input & Search"),
        ("textarea", "Text area"),
        ("checkbox", "Checkbox"),
        ("switch", "Switch"),
        ("radio-group", "Radio group"),
        ("slider", "Slider"),
        ("tabs", "Tabs"),
        ("select", "Select"),
        ("dialog", "Dialog"),
        ("layout", "Web layout"),
        ("motion", "Motion & loading"),
        ("effects", "GPU effects / WGSL"),
        ("typography", "Typography & selection"),
        ("custom-timeline", "Custom Timeline"),
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

#[test]
fn compact_display_labels_remain_complete_with_the_gallery_embedded_font() {
    const FONT: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");
    let app = Entity::new(WidgetGallery::default());
    let mut text = TextEngine::from_embedded_fonts([FONT], "Noto Sans", "Noto Sans", "Noto Sans");
    let mut layout = LayoutEngine::new();
    let mut tree = UiTree::new(app.render());
    for (page, labels) in [
        ("separator", ["Or continue with", "Preferences"].as_slice()),
        (
            "kbd",
            ["Search components", "Change theme", "Enter", "Cmd"].as_slice(),
        ),
        ("aspect-ratio", ["1 : 1", "4 : 3", "16 : 9"].as_slice()),
    ] {
        click(&app, &format!("nav::{page}"));
        for width in [800.0, 1220.0] {
            tree.update(app.render());
            let output = layout
                .compute(&mut tree, &mut text, Size::new(width, 780.0))
                .unwrap();
            let prepared = text.prepare(&output.text, 1.0);
            for label in labels {
                let (index, block) = output
                    .text
                    .blocks()
                    .iter()
                    .enumerate()
                    .find(|(_, block)| block.content.as_str() == *label)
                    .unwrap_or_else(|| panic!("missing {label}"));
                let end = prepared
                    .glyphs
                    .iter()
                    .filter(|glyph| glyph.block == index)
                    .map(|glyph| glyph.end)
                    .max();
                assert_eq!(end, Some(label.len()), "truncated {label} at width {width}");
                let measured = text.measure(label, &block.style, Some(block.bounds.size.width));
                assert_eq!(
                    measured.height, block.style.line_height,
                    "wrapped {label} at width {width}"
                );
            }
        }
    }
}

fn dispatch(app: &Entity<WidgetGallery>, key: &str, kind: UiEventKind) {
    let mut tree = UiTree::new(app.render());
    let node = tree
        .node_ids()
        .iter()
        .copied()
        .find(|id| tree.key(*id) == Some(key))
        .unwrap_or_else(|| panic!("missing {key}"));
    for event in tree.event_deliveries(node, kind) {
        if event.should_dispatch() {
            app.dispatch_event(&event);
        }
    }
}
fn click(app: &Entity<WidgetGallery>, key: &str) {
    dispatch(app, key, UiEventKind::Click(ClickEvent::accessibility()));
}
fn keyboard(app: &Entity<WidgetGallery>, target: &str, key: argui::core::Key) {
    dispatch(
        app,
        target,
        UiEventKind::KeyInput(argui::core::KeyInput {
            key,
            state: argui::core::KeyState::Pressed,
            modifiers: Default::default(),
            repeat: false,
            text: None,
        }),
    );
}
fn keyed<'a>(root: &'a Element, key: &str) -> Option<&'a Element> {
    if root.key.as_deref() == Some(key) {
        Some(root)
    } else {
        root.children.iter().find_map(|child| keyed(child, key))
    }
}
#[path = "pages/actions.rs"]
mod actions;
#[path = "pages/async_tasks.rs"]
mod async_tasks;
#[path = "pages/data_table.rs"]
mod data_table;
#[path = "pages/dates.rs"]
mod dates;
#[path = "pages/editing.rs"]
mod editing;
#[path = "pages/menus.rs"]
mod menus;
#[path = "pages/toast.rs"]
mod toast;

#[path = "pages/card.rs"]
mod card;
#[path = "pages/collapsible.rs"]
mod collapsible;

#[path = "pages/empty.rs"]
mod empty;
#[path = "pages/progress.rs"]
mod progress;

#[path = "pages/breadcrumb.rs"]
mod breadcrumb;
#[path = "pages/label.rs"]
mod label;
#[path = "pages/pagination.rs"]
mod pagination;
#[path = "pages/skeleton.rs"]
mod skeleton;
