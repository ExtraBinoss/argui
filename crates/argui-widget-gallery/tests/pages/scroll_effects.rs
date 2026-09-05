use argui::{
    core::Point,
    runtime::Entity,
    ui::{ClickEvent, Element, UiEventKind, UiTree},
};
use argui_widget_gallery::WidgetGallery;

fn find<'a>(element: &'a Element, key: &str) -> Option<&'a Element> {
    if element.key.as_deref() == Some(key) {
        Some(element)
    } else {
        element.children.iter().find_map(|child| find(child, key))
    }
}

fn dispatch(app: &Entity<WidgetGallery>, key: &str, kind: UiEventKind) {
    let mut tree = UiTree::new(app.render());
    let node = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some(key))
        .unwrap();
    for event in tree.event_deliveries(node, kind) {
        if event.should_dispatch() {
            app.dispatch_event(&event);
        }
    }
}

#[test]
fn scroll_demo_cycles_presets_parameters_and_virtual_windows() {
    let app = Entity::new(WidgetGallery::default());
    let click = || UiEventKind::Click(ClickEvent::accessibility());
    dispatch(&app, "nav::effects", click());
    for (mode, expected) in [
        "argui.scroll.edge-fade",
        "argui.scroll.edge-shadow",
        "gallery.scroll.progress-tint",
    ]
    .into_iter()
    .enumerate()
    {
        dispatch(&app, &format!("scroll-mode-{mode}"), click());
        let root = app.render();
        let list = find(&root, "scroll-demo-list").unwrap();
        assert!(list.children.len() < 40);
        let argui::paint::Filter::Effect(effect) =
            &list.scroll.as_ref().unwrap().effects[0].layer.filters[0]
        else {
            panic!()
        };
        assert_eq!(effect.id.0, expected);
    }
    assert!(find(&app.render(), "scroll-width-less").is_none());
    dispatch(&app, "scroll-mode-0", click());
    for suffix in ["less", "more"] {
        for _ in 0..7 {
            dispatch(&app, &format!("scroll-width-{suffix}"), click());
            dispatch(&app, &format!("scroll-intensity-{suffix}"), click());
        }
        let root = app.render();
        let argui::paint::Filter::Effect(effect) = &find(&root, "scroll-demo-list")
            .unwrap()
            .scroll
            .as_ref()
            .unwrap()
            .effects[0]
            .layer
            .filters[0]
        else {
            panic!()
        };
        assert_eq!(
            effect.parameters[0].value,
            argui::paint::EffectValue::LogicalPixels(if suffix == "less" { 0.0 } else { 60.0 })
        );
        assert_eq!(
            effect.parameters[1].value,
            argui::paint::EffectValue::F32(if suffix == "less" { 0.0 } else { 1.0 })
        );
    }
    for offset in [1.0, 2.0, 2400.0, 319_700.0, 0.0] {
        dispatch(
            &app,
            "scroll-demo-list",
            UiEventKind::Scrolled {
                delta: Point::new(0.0, 1.0),
                offset: Point::new(0.0, offset),
            },
        );
        assert!(
            find(&app.render(), "scroll-demo-list")
                .unwrap()
                .children
                .len()
                < 40
        );
    }
    assert!(find(&app.render(), "scroll-demo-nested").is_some());
    assert!(find(&app.render(), "scroll-horizontal").is_some());
}

#[test]
fn scroll_examples_keep_visible_cards_and_real_scrollports_at_narrow_widths() {
    let app = Entity::new(WidgetGallery::default());
    dispatch(
        &app,
        "nav::effects",
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    let root = app.render();
    let demo = find(&root, "scroll-effects-demo").unwrap();
    let row = find(demo, "scroll-row-0").unwrap();
    assert_eq!(
        row.style.justify_content,
        Some(argui::ui::JustifyContent::START)
    );
    for width in [280.0, 760.0] {
        let mut tree = UiTree::new(demo.clone());
        let output = argui::layout::LayoutEngine::new()
            .compute(
                &mut tree,
                &mut argui::text::TextEngine::new(),
                argui::core::Size::new(width, 1000.0),
            )
            .unwrap();
        let bounds = |key: &str| {
            output
                .nodes
                .iter()
                .find(|node| tree.key(node.node) == Some(key))
                .unwrap()
                .bounds
        };
        let horizontal = bounds("scroll-horizontal");
        let nested = bounds("scroll-demo-nested");
        let card = bounds("scroll-card-0");
        assert!(nested.size.width <= width);
        assert!(horizontal.size.width <= nested.size.width);
        assert_eq!(horizontal.size.height, 110.0);
        assert_eq!(card.size.height, 88.0);
        assert_eq!(card.size.width, 150.0);
        assert!(card.origin.y >= nested.origin.y);
        assert!(card.origin.y + card.size.height <= nested.origin.y + nested.size.height);
        assert_eq!(bounds("scroll-demo-list").size.height, 220.0);
        let list = bounds("scroll-demo-list");
        assert!(
            (list.size.width - (width - 2.0)).abs() < 0.1,
            "the list must fill its panel, excluding only its two borders: {list:?}"
        );
        let list_region = output
            .scroll_regions
            .iter()
            .find(|region| tree.key(region.node) == Some("scroll-demo-list"))
            .unwrap();
        let scrollbar = list_region.scrollbar.as_ref().unwrap();
        let track = scrollbar.vertical.as_ref().unwrap().track;
        assert!(
            (track.origin.x + track.size.width + scrollbar.style.insets.right
                - (list.origin.x + list.size.width))
                .abs()
                < 0.1
        );
        let region = output
            .scroll_regions
            .iter()
            .find(|region| tree.key(region.node) == Some("scroll-horizontal"))
            .unwrap();
        assert_eq!(region.config.axes, argui::ui::ScrollAxes::Horizontal);
        assert!(
            region.max_offset.x > 0.0,
            "width={width}, horizontal={horizontal:?}, nested={nested:?}, region={region:?}"
        );
        let nested_region = output
            .scroll_regions
            .iter()
            .find(|region| tree.key(region.node) == Some("scroll-demo-nested"))
            .unwrap();
        assert!(nested_region.max_offset.y > 0.0);
    }
}

#[test]
fn effects_page_uses_the_available_width_instead_of_the_reading_column_limit() {
    let app = Entity::new(WidgetGallery::default());
    dispatch(
        &app,
        "nav::effects",
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    for width in [1220.0, 1800.0] {
        let mut tree = UiTree::new(app.render());
        let output = argui::layout::LayoutEngine::new()
            .compute(
                &mut tree,
                &mut argui::text::TextEngine::new(),
                argui::core::Size::new(width, 780.0),
            )
            .unwrap();
        let page = output
            .nodes
            .iter()
            .find(|node| tree.key(node.node) == Some("gallery-content-scroll"))
            .unwrap()
            .bounds;
        assert!(
            (page.origin.x + page.size.width - width).abs() < 1.0,
            "page should reach the right edge: {page:?}"
        );
        let demo = output
            .nodes
            .iter()
            .find(|node| tree.key(node.node) == Some("scroll-effects-demo"))
            .unwrap()
            .bounds;
        assert!(
            (demo.size.width - (page.size.width - 60.0)).abs() < 1.0,
            "demo should fill the page's padded content: {demo:?}"
        );
    }
}
