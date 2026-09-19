use super::*;

#[test]
fn forms_edit_validate_filter_and_complete_the_questionnaire() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::field");
    click(&app, "email-field::label");
    dispatch(
        &app,
        "email-control",
        UiEventKind::TextChanged("invalid".into()),
    );
    assert!(contains_text(
        &app.render(),
        "Include an @ in the email address."
    ));
    dispatch(
        &app,
        "email-control",
        UiEventKind::TextChanged("ada@example.com".into()),
    );
    assert!(!contains_text(
        &app.render(),
        "Include an @ in the email address."
    ));
    click(&app, "nav::input-group");
    dispatch(&app, "domain", UiEventKind::TextChanged("argui.dev".into()));
    click(&app, "visit");
    assert!(contains_text(&app.render(), "Preview: https://argui.dev"));
    click(&app, "nav::input-otp");
    for (value, status) in [("123", "Keep typing"), ("123456", "Code complete")] {
        dispatch(&app, "code", UiEventKind::TextChanged(value.into()));
        assert!(contains_text(&app.render(), status));
    }
    click(&app, "nav::combobox");
    click(&app, "framework");
    dispatch(&app, "framework", UiEventKind::TextChanged("s".into()));
    keyboard(&app, "framework", Key::ArrowDown);
    keyboard(&app, "framework", Key::Enter);
    assert!(contains_text(&app.render(), "Selected SvelteKit"));
    click(&app, "framework");
    assert!(
        keyed(&app.render(), "framework::option::1")
            .and_then(|option| option.semantics.as_ref())
            .is_some_and(|semantics| semantics.state.selected)
    );
    dispatch(&app, "framework", UiEventKind::TextChanged(String::new()));
    assert!(
        keyed(&app.render(), "framework::option::1")
            .and_then(|option| option.semantics.as_ref())
            .is_some_and(|semantics| !semantics.state.selected)
    );
    keyboard(&app, "framework", Key::Escape);
    assert!(keyed(&app.render(), "framework::list").is_none());
    click(&app, "nav::native-select");
    click(&app, "language");
    keyboard(&app, "language", Key::ArrowDown);
    click(&app, "language::option::1");
    assert!(contains_text(&app.render(), "Selected Français"));
    click(&app, "language");
    keyboard(&app, "language", Key::Escape);
    click(&app, "nav::questionnaire");
    click(&app, "survey::next");
    assert!(contains_text(
        &app.render(),
        "Choose or enter a valid answer."
    ));
    click(&app, "survey::project::choice::0");
    click(&app, "survey::next");
    click(&app, "survey::previous");
    dispatch(
        &app,
        "survey::project::input",
        UiEventKind::TextChanged("A native workspace".into()),
    );
    click(&app, "survey::next");
    click(&app, "survey::skip");
    click(&app, "survey::submit");
    assert!(contains_text(&app.render(), "Your answers are ready."));
}

#[test]
fn tab_closes_combobox_without_canceling_focus_navigation() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::combobox");
    click(&app, "framework");
    let mut tree = UiTree::new(app.render());
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some("framework"))
        .unwrap();
    let events = tree.event_deliveries(
        target,
        UiEventKind::KeyInput(KeyInput {
            key: Key::Tab,
            state: KeyState::Pressed,
            repeat: false,
            modifiers: Default::default(),
            text: None,
        }),
    );
    for event in &events {
        if event.should_dispatch() {
            app.dispatch_event(event);
        }
    }
    assert!(events.iter().all(|event| !event.default_prevented()));
    assert!(keyed(&app.render(), "framework::list").is_none());
    click(&app, "nav::input-otp");
    keyboard(&app, "code", Key::ArrowLeft);
    assert!(contains_text(
        &app.render(),
        "Enter six digits. You can paste the whole code."
    ));
}
