use super::*;
use argui::core::{Key, KeyInput, KeyState, Modifiers};

fn typing(text: &str) -> UiEventKind {
    UiEventKind::KeyInput(KeyInput {
        key: Key::Character(text.into()),
        text: Some(text.into()),
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
        repeat: false,
    })
}

fn search(app: &Entity<WidgetGallery>) -> String {
    let root = app.render();
    let argui::ui::ElementKind::TextEditor { value, .. } =
        &find_key(&root, "gallery-search").unwrap().kind
    else {
        panic!("search editor");
    };
    value.clone()
}

#[test]
fn typing_on_a_button_or_empty_page_starts_a_new_component_search() {
    let app = Entity::new(WidgetGallery::default());
    dispatch(&app, "nav::button", typing("s"));
    assert_eq!(search(&app), "s");
    assert!(has_key(&app.render(), "nav::slider"));
    // Subsequent typing belongs to the focused editor and its standard edit protocol.
    dispatch(&app, "gallery-search", typing("l"));
    assert_eq!(search(&app), "s");
    dispatch(
        &app,
        "gallery-search",
        UiEventKind::TextChanged("slide".into()),
    );
    assert_eq!(search(&app), "slide");
    assert!(!has_key(&app.render(), "nav::button"));
    dispatch(&app, "gallery-root", typing("É"));
    assert_eq!(search(&app), "É");
    let UiEventKind::KeyInput(mut input) = typing("B") else {
        unreachable!()
    };
    input.text = None;
    input.modifiers.shift = true;
    dispatch(&app, "gallery-root", UiEventKind::KeyInput(input));
    assert_eq!(search(&app), "B");
    dispatch(
        &app,
        "gallery-search",
        UiEventKind::KeyInput(KeyInput {
            key: Key::Escape,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            repeat: false,
            text: None,
        }),
    );
    assert_eq!(search(&app), "");
}

#[test]
fn gallery_has_an_initial_keyboard_target_without_selecting_an_editor() {
    use argui::platform::WindowKey;
    use argui::runtime::{AppModel, SingleWindowModel, WindowEnvironment};
    let app =
        argui_devtools::DevtoolsApp::new(SingleWindowModel::new(argui::widgets::TooltipHost::new(
            argui::widgets::SelectionHost::new(WidgetGallery::default()),
        )));
    let mut tree = UiTree::new(
        app.view(&WindowKey::main(), WindowEnvironment::default())
            .unwrap(),
    );
    let layout = argui::layout::LayoutEngine::new()
        .compute(
            &mut tree,
            &mut argui::text::TextEngine::new(),
            argui::core::Size::new(1220.0, 900.0),
        )
        .unwrap();
    // Browser startup focuses a zero-size canvas before its first resize.
    tree.window_focused(&[]);
    assert!(tree.focused_node().is_none());
    tree.sync_focus(&layout.hit_regions, None);
    assert_eq!(
        tree.focused_node().and_then(|node| tree.key(node)),
        Some("gallery-root")
    );
}

#[test]
fn editing_shortcuts_activation_and_popup_typeahead_do_not_start_global_search() {
    let app = Entity::new(WidgetGallery::default());
    for text in [" ", "", "\n"] {
        dispatch(&app, "gallery-root", typing(text));
    }
    for modifier in [
        Modifiers {
            control: true,
            ..Default::default()
        },
        Modifiers {
            alt: true,
            ..Default::default()
        },
        Modifiers {
            super_key: true,
            ..Default::default()
        },
    ] {
        let UiEventKind::KeyInput(mut input) = typing("a") else {
            unreachable!()
        };
        input.modifiers = modifier;
        dispatch(&app, "gallery-root", UiEventKind::KeyInput(input));
    }
    let UiEventKind::KeyInput(mut input) = typing("a") else {
        unreachable!()
    };
    input.state = KeyState::Released;
    dispatch(&app, "gallery-root", UiEventKind::KeyInput(input));
    assert_eq!(search(&app), "");
    dispatch(
        &app,
        "nav::input",
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    dispatch(&app, "name", typing("a"));
    assert_eq!(search(&app), "");
    dispatch(
        &app,
        "nav::textarea",
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    dispatch(&app, "notes", typing("a"));
    assert_eq!(search(&app), "");
    dispatch(
        &app,
        "nav::popover",
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    dispatch(
        &app,
        "popover-sharing",
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    dispatch(&app, "popover-link-options", typing("a"));
    assert_eq!(search(&app), "");
    dispatch(
        &app,
        "nav::select",
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    dispatch(&app, "backend", typing("a"));
    assert_eq!(search(&app), "");
}
