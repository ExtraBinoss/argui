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
fn scroll_shadow_controls_update_parameters_and_virtual_windows() {
    let app = Entity::new(WidgetGallery::default());
    let click = || UiEventKind::Click(ClickEvent::accessibility());
    dispatch(&app, "nav::scroll-shadow", click());
    let root = app.render();
    let list = find(&root, "scroll-demo-list").unwrap();
    let argui::paint::Filter::Effect(effect) =
        &list.scroll.as_ref().unwrap().effects[0].layer.filters[0]
    else {
        panic!()
    };
    assert_eq!(effect.id, argui_effects::EDGE_SHADOW_ID);
    assert!(find(&root, "scroll-mode-0").is_none());
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
        "nav::scroll-shadow",
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
        "nav::scroll-shadow",
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

#[test]
fn scrolling_effect_panels_never_moves_the_page_even_at_edges_or_after_row_rebuilds() {
    let app = Entity::new(WidgetGallery::default());
    dispatch(
        &app,
        "nav::scroll-shadow",
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    let mut tree = UiTree::new(app.render());
    let mut engine = argui::layout::LayoutEngine::new();
    let mut text = argui::text::TextEngine::new();
    let size = argui::core::Size::new(1220.0, 780.0);
    let mut output = engine.compute(&mut tree, &mut text, size).unwrap();
    let page = output
        .scroll_regions
        .iter()
        .find(|region| tree.key(region.node) == Some("gallery-content-scroll"))
        .unwrap()
        .node;
    for key in ["scroll-demo-list", "scroll-demo-nested"] {
        // Locate the example after any preceding gallery sections instead
        // of assuming a fixed page position.
        let target = output
            .nodes
            .iter()
            .find(|node| tree.key(node.node) == Some(key))
            .unwrap()
            .bounds;
        let page_region = output
            .scroll_regions
            .iter()
            .find(|region| region.node == page)
            .unwrap();
        let page_offset =
            (tree.scroll_offset(page).y + target.origin.y - page_region.bounds.origin.y - 24.0)
                .clamp(0.0, page_region.max_offset.y);
        tree.set_scroll_offset(page, Point::new(0.0, page_offset));
        engine.apply_scroll(&tree, &mut output).unwrap();
        let region = output
            .scroll_regions
            .iter()
            .find(|region| tree.key(region.node) == Some(key))
            .unwrap()
            .clone();
        assert_eq!(
            region.config.propagation,
            argui::ui::ScrollPropagation::Contain
        );
        for (offset, dy) in [
            (0.0, 40.0),
            (0.0, -40.3),
            (2000.0, -300.7),
            (region.max_offset.y - 1.0, -500.0),
            (region.max_offset.y, -500.0),
        ] {
            tree.set_scroll_offset(
                region.node,
                Point::new(0.0, offset.min(region.max_offset.y)),
            );
            engine.apply_scroll(&tree, &mut output).unwrap();
            let update = tree.scroll_from(
                region.node,
                region.bounds.origin,
                argui::core::ScrollDelta::Pixels(Point::new(0.25, dy)),
                &output.scroll_regions,
            );
            let expected = (offset.min(region.max_offset.y) - dy).clamp(0.0, region.max_offset.y);
            assert!((tree.scroll_offset(region.node).y - expected).abs() < 0.01);
            assert_eq!(
                tree.scroll_offset(page).y,
                page_offset,
                "{key}, offset {offset}, delta {dy}"
            );
            for event in update.events {
                if event.should_dispatch() {
                    app.dispatch_event(&event);
                }
            }
            tree.update(app.render());
            output = engine.compute(&mut tree, &mut text, size).unwrap();
            assert_eq!(
                tree.scroll_offset(page).y,
                page_offset,
                "page moved during a virtual row rebuild"
            );
        }
    }
}

#[test]
fn every_scroll_axis_uses_the_current_theme_shadow_color() {
    use argui::{
        core::ColorScheme,
        paint::{EffectValue, Filter},
        runtime::WindowEnvironment,
    };
    let app = Entity::new(WidgetGallery::default());
    dispatch(
        &app,
        "nav::scroll-shadow",
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    for color_scheme in [ColorScheme::Light, ColorScheme::Dark] {
        let environment = WindowEnvironment {
            color_scheme,
            ..Default::default()
        };
        let themes = argui::widgets::shadcn(&environment);
        let expected = themes.resolve(color_scheme).foreground.with_alpha(
            if color_scheme == ColorScheme::Light {
                0.22
            } else {
                0.06
            },
        );
        let root = app.render_in(environment);
        for key in [
            "scroll-demo-list",
            "scroll-horizontal",
            "scroll-demo-nested",
        ] {
            let scroll = find(&root, key).unwrap().scroll.as_ref().unwrap();
            let Filter::Effect(effect) = &scroll.effects[0].layer.filters[0] else {
                panic!()
            };
            assert!(
                effect
                    .parameters
                    .iter()
                    .any(|parameter| parameter.value == EffectValue::Color(expected))
            );
        }
    }
}
