#[path = "liquid_glass/controls.rs"]
mod controls;

use argui::{
    paint::Filter,
    runtime::{Entity, Mount},
    ui::{ClickEvent, Element, UiEventKind, UiTree},
};
use argui_widget_gallery::WidgetGallery;

fn find<'a>(root: &'a Element, key: &str) -> Option<&'a Element> {
    if root.key.as_deref() == Some(key) {
        Some(root)
    } else {
        root.children.iter().find_map(|child| find(child, key))
    }
}
fn click(app: &Mount<WidgetGallery>, key: &str) {
    let mut tree = UiTree::new(app.render(Default::default()).unwrap());
    let node = tree
        .node_ids()
        .iter()
        .copied()
        .find(|id| tree.key(*id) == Some(key))
        .unwrap();
    for event in tree.event_deliveries(node, UiEventKind::Click(ClickEvent::accessibility())) {
        if event.should_dispatch() {
            app.dispatch_event(&event).unwrap();
        }
    }
}
fn filter(app: &Mount<WidgetGallery>) -> Option<Filter> {
    find(
        &app.render(Default::default()).unwrap(),
        "liquid-glass-pane",
    )
    .unwrap()
    .layer
    .as_ref()
    .and_then(|layer| layer.backdrop_filters.first().cloned())
}
#[test]
fn gallery_glass_controls_update_real_filter_and_can_disable_it() {
    let app = Entity::new(WidgetGallery::default()).mount().unwrap();
    click(&app, "nav::liquid-glass");
    let Filter::Effect(initial) = filter(&app).unwrap() else {
        panic!()
    };
    assert_eq!(initial.id, argui_effects::LIQUID_GLASS_ID);
    for control in [
        "glass-depth",
        "glass-tint-blue",
        "glass-tint-rose",
        "glass-tint-white",
    ] {
        let before = filter(&app);
        click(&app, control);
        assert_ne!(filter(&app), before, "{control}");
    }
    click(&app, "glass-enable");
    assert!(filter(&app).is_none());
    click(&app, "glass-enable");
    assert!(filter(&app).is_some());
    click(&app, "nav::button");
    assert!(
        find(
            &app.render(Default::default()).unwrap(),
            "liquid-glass-pane"
        )
        .is_none()
    );
    click(&app, "nav::liquid-glass");
    assert!(filter(&app).is_some());
}

#[test]
fn colorful_feed_scrolls_under_a_fixed_responsive_navigation_bar() {
    use argui::{
        core::{Point, Size},
        layout::LayoutEngine,
        text::TextEngine,
    };
    let app = Entity::new(WidgetGallery::default()).mount().unwrap();
    click(&app, "nav::liquid-glass");
    for width in [1254.0, 800.0, 640.0] {
        let mut tree = UiTree::new(app.render(Default::default()).unwrap());
        let mut engine = LayoutEngine::new();
        let mut output = engine
            .compute(&mut tree, &mut TextEngine::new(), Size::new(width, 900.0))
            .unwrap();
        let node = |key| {
            tree.node_ids()
                .iter()
                .copied()
                .find(|n| tree.key(*n) == Some(key))
                .unwrap()
        };
        let (feed, bar, card, stage) = (
            node("liquid-glass-feed"),
            node("liquid-glass-pane"),
            node("glass-card-0"),
            node("liquid-glass-stage"),
        );
        let bounds = |output: &argui::layout::LayoutOutput, node| {
            output.nodes.iter().find(|n| n.node == node).unwrap().bounds
        };
        let bar_before = bounds(&output, bar);
        let card_before = bounds(&output, card);
        let stage_bounds = bounds(&output, stage);
        assert!(bar_before.origin.x >= stage_bounds.origin.x);
        assert!(
            bar_before.origin.x + bar_before.size.width
                <= stage_bounds.origin.x + stage_bounds.size.width
        );
        assert!((bar_before.size.height - 76.0).abs() < 0.1);
        let region = output
            .scroll_regions
            .iter()
            .find(|r| r.node == feed)
            .unwrap();
        assert!(region.max_offset.y > 800.0, "actual scrollable content");
        tree.set_scroll_offset(feed, Point::new(0.0, 280.0));
        engine.apply_scroll(&tree, &mut output).unwrap();
        assert_eq!(bounds(&output, bar), bar_before, "navigation stays fixed");
        assert!((bounds(&output, card).origin.y - card_before.origin.y + 280.0).abs() < 0.1);
        // The pane must be painted after the feed, to capture the scrolling
        // content as its backdrop rather than its own children.
        let root = app.render(Default::default()).unwrap();
        let stage = find(&root, "liquid-glass-stage").unwrap();
        assert_eq!(stage.children[0].key.as_deref(), Some("liquid-glass-feed"));
        assert_eq!(stage.children[1].key.as_deref(), Some("liquid-glass-pane"));
        assert!(filter(&app).is_some());
    }
}

#[test]
fn bottom_navigation_switches_real_collections_without_resetting_glass() {
    let app = Entity::new(WidgetGallery::default()).mount().unwrap();
    click(&app, "nav::liquid-glass");
    let initial = filter(&app);
    let root = app.render(Default::default()).unwrap();
    let explore = find(&root, "glass-explore").unwrap().paint.clone();
    let title = find(&root, "glass-feed-title").unwrap().clone();
    click(&app, "glass-collections");
    let root = app.render(Default::default()).unwrap();
    assert_ne!(find(&root, "glass-feed-title").unwrap(), &title);
    assert_ne!(find(&root, "glass-explore").unwrap().paint, explore);
    click(&app, "glass-saved");
    let root = app.render(Default::default()).unwrap();
    for index in 0..8 {
        assert_eq!(
            find(&root, &format!("glass-card-{index}")).is_some(),
            index % 2 == 0
        );
    }
    click(&app, "glass-explore");
    assert!(find(&app.render(Default::default()).unwrap(), "glass-card-7").is_some());
    assert_eq!(filter(&app), initial);
}

#[test]
fn navigation_icons_use_assets_registered_by_the_application() {
    use argui::{runtime::Render, ui::ElementKind};
    let gallery = Entity::new(WidgetGallery::default());
    // Native and WebGPU renderers register the root application's resources
    // before mounting child entities or opening the effects page.
    let registered = gallery.read(Render::vector_assets);
    let app = gallery.mount().unwrap();
    click(&app, "nav::liquid-glass");
    for key in [
        "glass-explore",
        "glass-collections",
        "glass-saved",
        "glass-reset",
    ] {
        click(&app, key);
        let root = app.render(Default::default()).unwrap();
        let tree = UiTree::new(find(&root, "liquid-glass-pane").unwrap().clone());
        let mut icons = 0;
        for index in 0..tree.node_ids().len() {
            if let ElementKind::Vector { vector, .. } = tree.element_at(index).unwrap().kind {
                assert!(
                    registered.iter().any(|asset| asset.id == vector),
                    "unregistered vector {}",
                    vector.0
                );
                icons += 1;
            }
        }
        assert_eq!(icons, 3);
    }
}

#[test]
fn simp_music_defaults_use_a_theme_tint_and_keep_custom_tints_on_theme_changes() {
    use argui::{core::ColorScheme, paint::EffectValue, runtime::WindowEnvironment};
    let app = Entity::new(WidgetGallery::default()).mount().unwrap();
    click(&app, "nav::liquid-glass");
    for (scheme, rgb) in [(ColorScheme::Light, 1.0), (ColorScheme::Dark, 0.0)] {
        let root = app
            .render(WindowEnvironment {
                color_scheme: scheme,
                ..Default::default()
            })
            .unwrap();
        let Filter::Effect(effect) = &find(&root, "liquid-glass-pane")
            .unwrap()
            .layer
            .as_ref()
            .unwrap()
            .backdrop_filters[0]
        else {
            panic!()
        };
        let parameter = |name| {
            &effect
                .parameters
                .iter()
                .find(|p| p.name == name)
                .unwrap()
                .value
        };
        assert_eq!(parameter("refraction"), &EffectValue::LogicalPixels(38.0));
        assert_eq!(parameter("edge-width"), &EffectValue::LogicalPixels(19.0));
        assert_eq!(parameter("blur"), &EffectValue::LogicalPixels(8.0));
        assert_eq!(parameter("saturation"), &EffectValue::F32(2.25));
        assert_eq!(parameter("brightness"), &EffectValue::F32(0.05));
        assert_eq!(parameter("depth-effect"), &EffectValue::Bool(false));
        assert_eq!(
            parameter("tint"),
            &EffectValue::Vec4([rgb, rgb, rgb, 0.272])
        );
    }
    click(&app, "glass-tint-blue");
    let custom = filter(&app);
    app.render(WindowEnvironment {
        color_scheme: ColorScheme::Dark,
        ..Default::default()
    })
    .unwrap();
    assert_eq!(filter(&app), custom);
    click(&app, "glass-tint-theme");
    assert_ne!(filter(&app), custom);
}
