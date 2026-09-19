use super::*;

#[test]
fn collection_examples_toggle_and_navigate_using_public_widget_actions() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::accordion");
    click(&app, "faq::item::keyboard::trigger");
    assert!(contains_text(
        &app.render(),
        "Use Tab to reach a heading, Enter to expand it, and arrows to move between headings."
    ));
    keyboard(&app, "faq::item::keyboard::trigger", Key::ArrowDown);
    click(&app, "faq::item::keyboard::trigger");
    click(&app, "nav::toggle");
    click(&app, "bold");
    assert!(contains_text(&app.render(), "Bold enabled"));
    click(&app, "bold");
    assert!(contains_text(&app.render(), "Bold disabled"));
    click(&app, "nav::toggle-group");
    click(&app, "format::item::bold");
    keyboard(&app, "format::item::bold", Key::ArrowRight);
    let root = app.render();
    assert_eq!(
        keyed(&root, "format::item::bold")
            .unwrap()
            .semantics
            .as_ref()
            .unwrap()
            .state
            .pressed,
        Some(true)
    );
    click(&app, "nav::button-group");
    click(&app, "group-copy");
    assert!(contains_text(&app.render(), "Action: copy"));
    click(&app, "group-follow-menu");
    assert!(keyed(&app.render(), "group-follow-menu::content").is_some());
    click(&app, "group-follow-menu::item::mentions");
    assert!(contains_text(&app.render(), "Follow option: mentions"));
    assert!(keyed(&app.render(), "group-follow-menu::content").is_none());
    click(&app, "group-currency");
    assert!(keyed(&app.render(), "group-currency::list").is_some());
    click(&app, "group-currency::option::2");
    assert!(contains_text(&app.render(), "Currency: £"));
    click(&app, "group-copilot-menu");
    assert!(keyed(&app.render(), "group-copilot-menu::content").is_some());
    keyboard(&app, "group-copilot-menu::content", Key::Escape);
    assert!(keyed(&app.render(), "group-copilot-menu::content").is_none());
    click(&app, "nav::navigation-menu");
    click(&app, "docs::item::overview");
    assert!(contains_text(&app.render(), "Opened overview"));
    keyboard(&app, "docs::item::overview", Key::ArrowRight);
    keyboard(&app, "docs::item::guide", Key::ArrowDown);
    assert!(keyed(&app.render(), "docs::item::guide::content").is_some());
    keyboard(&app, "docs::item::guide", Key::Escape);
    assert!(keyed(&app.render(), "docs::item::guide::content").is_none());
    click(&app, "docs::item::guide");
    click(&app, "guide-start");
    assert!(contains_text(
        &app.render(),
        "Getting started: create a window and add a Button."
    ));
    click(&app, "nav::sidebar");
    click(&app, "workspace::toggle");
    assert!(contains_text(&app.render(), "P"));
    click(&app, "workspace-Projects");
    assert!(contains_text(&app.render(), "Opened Projects"));
    click(&app, "workspace::toggle");
    assert!(contains_text(&app.render(), "Activity"));
}

#[test]
fn button_group_examples_edit_toggle_and_dismiss_every_control() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::button-group");

    for (key, value) in [
        ("group-search", "buttons"),
        ("group-composer-input", "Hello"),
        ("voice-message-input", "Voice note"),
        ("group-amount", "42.00"),
    ] {
        dispatch(&app, key, UiEventKind::TextChanged(value.into()));
    }
    dispatch(
        &app,
        "group-copy",
        UiEventKind::TextChanged("ignored".into()),
    );

    click(&app, "group-voice");
    assert!(contains_text(&app.render(), "Voice mode enabled"));
    click(&app, "group-voice");
    assert!(contains_text(&app.render(), "Voice mode disabled"));

    click(&app, "group-follow-menu");
    keyboard(&app, "group-follow-menu::item::all", Key::ArrowDown);
    keyboard(&app, "group-follow-menu::item::mentions", Key::Escape);
    assert!(keyed(&app.render(), "group-follow-menu::content").is_none());

    click(&app, "group-currency");
    keyboard(&app, "group-currency", Key::ArrowDown);
    keyboard(&app, "group-currency::option::1", Key::Escape);
    assert!(keyed(&app.render(), "group-currency::list").is_none());

    click(&app, "group-copilot-menu");
    click(&app, "group-copilot-menu");
    assert!(keyed(&app.render(), "group-copilot-menu::content").is_none());
    click(&app, "group-copilot-menu");
    click(&app, "group-copilot-start");
    assert!(contains_text(&app.render(), "Describe a task first"));
    click(&app, "group-copilot-menu");
    dispatch(
        &app,
        "group-copilot-task",
        UiEventKind::TextChanged("Polish the release".into()),
    );
    click(&app, "group-copilot-start");
    assert!(contains_text(&app.render(), "Started: Polish the release"));
}

#[test]
fn button_group_inputs_and_overlay_triggers_share_one_control_height() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::button-group");
    let mut tree = UiTree::new(app.render());
    let mut layout = LayoutEngine::new();
    let mut text = TextEngine::new();
    let output = layout
        .compute(&mut tree, &mut text, Size::new(900.0, 1_800.0))
        .unwrap();
    let height = |key: &str| {
        let node = tree
            .node_ids()
            .iter()
            .copied()
            .find(|node| tree.key(*node) == Some(key))
            .unwrap_or_else(|| panic!("missing {key}"));
        output
            .nodes
            .iter()
            .find(|layout| layout.node == node)
            .unwrap()
            .bounds
            .size
            .height
    };

    for (left, right) in [
        ("group-search", "group-search-submit"),
        ("group-composer-add", "voice-message"),
        ("group-currency", "group-amount"),
        ("group-follow", "group-follow-menu"),
        ("group-copilot", "group-copilot-menu"),
    ] {
        assert_eq!(height(left), height(right), "{left} and {right}");
    }
}

#[test]
fn button_group_voice_action_is_a_compact_circular_icon() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::button-group");
    let root = app.render();
    let voice = keyed(&root, "group-voice").unwrap();

    assert_eq!(voice.style.size.width, argui::ui::length(28.0));
    assert_eq!(voice.style.size.height, argui::ui::length(28.0));
    assert_eq!(
        voice.paint.quad.radii,
        argui::paint::CornerRadii::all(999.0)
    );
    assert_eq!(
        voice.semantics.as_ref().unwrap().label.as_deref(),
        Some("Start voice input")
    );
    assert!(matches!(
        voice.children[0].kind,
        argui::ui::ElementKind::Vector { .. }
    ));

    click(&app, "group-voice");
    let active = app.render();
    assert_eq!(
        keyed(&active, "group-voice")
            .unwrap()
            .semantics
            .as_ref()
            .unwrap()
            .label
            .as_deref(),
        Some("Stop recording")
    );
}
