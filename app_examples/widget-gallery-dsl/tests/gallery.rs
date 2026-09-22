use argui::{
    core::{Key, Modifiers, Point, ScrollDelta, Size},
    paint::{EffectValue, Filter},
    render::EffectDamage,
    runtime::Render,
    ui::Role,
};
use argui_example_widget_gallery_dsl::Main;
use argui_testing::{Selector, TestApp};

#[path = "support/navigation.rs"]
mod navigation;
use navigation::navigate_to_page;

/// The DSL gallery renders responsive navigation and real button variants.
#[test]
fn gallery_shows_button_page_and_accessible_variants() {
    let app = TestApp::new(Main::new());
    app.assert_text("Widget Gallery");
    app.assert_text("COMPONENTS");
    app.assert_text("Variants");
    for label in ["Primary", "Secondary", "Outline", "Ghost", "Delete"] {
        assert!(app.semantics().nodes.iter().any(|node| {
            node.semantics.role == Role::Button && node.semantics.label.as_deref() == Some(label)
        }));
    }
}

/// Hovering the glass example opens its tooltip with the authored backdrop filter.
#[test]
fn gallery_tooltip_glass_example_blurs_its_backdrop() {
    let mut app = TestApp::new(Main::new());
    navigate_to_page(&mut app, "Tooltip");
    let trigger = app
        .bounds(Selector::role(Role::Button, "Glass tooltip"))
        .unwrap();
    app.pointer_move(Point::new(
        trigger.origin.x + trigger.size.width * 0.5,
        trigger.origin.y + trigger.size.height * 0.5,
    ))
    .unwrap();
    assert!(app.semantics().nodes.iter().any(|node| {
        node.semantics.role == Role::Tooltip
            && node.semantics.label.as_deref()
                == Some("Colored detail stays soft behind this hint.")
    }));
    let tree = app.rendered_tree();
    assert!(
        tree.node_ids()
            .iter()
            .filter_map(|id| tree.element_for(*id))
            .any(|element| {
                element
                    .layer
                    .as_ref()
                    .is_some_and(|layer| layer.backdrop_filters == [Filter::Blur(18.0)])
            })
    );
}

/// Navigation opens every separately authored component page.
#[test]
fn gallery_navigation_opens_component_pages() {
    let mut app = TestApp::new(Main::new());
    for (navigation, title) in [
        ("Input & Search", "Controlled values, search"),
        ("Card", "A composable surface"),
        ("Badge", "Compact status markers"),
        ("Separator", "A quiet divider"),
        ("Switch", "A theme-aware toggle"),
        ("Media", "Project images and SVGs"),
        ("VList", "Only nearby rows are mounted"),
    ] {
        navigate_to_page(&mut app, navigation);
        assert!(
            app.semantics().nodes.iter().any(|node| {
                node.semantics
                    .label
                    .as_deref()
                    .is_some_and(|label| label.contains(title))
            }),
            "missing {navigation} page"
        );
    }
}

/// The menu bar opens nested flyouts and reports the chosen leaf action.
#[test]
fn gallery_menu_bar_selects_a_third_level_action() {
    let mut app = TestApp::new(Main::new());
    navigate_to_page(&mut app, "Menu");
    app.assert_text("Nothing selected yet");
    app.get_by_role(Role::ComboBox, "File").click().unwrap();
    app.get_by_role(Role::MenuItem, "Export").click().unwrap();
    app.get_by_role(Role::MenuItem, "Image").click().unwrap();
    app.get_by_role(Role::MenuItem, "PNG…").click().unwrap();
    app.assert_text("File › Export › Image › PNG");
}

/// The examples category includes a shader thumb that remains draggable.
#[test]
fn gallery_scrollbar_styles_include_a_draggable_effect() {
    let mut app = TestApp::new(Main::new());
    app.assert_text("EXAMPLES");
    navigate_to_page(&mut app, "Scrollbar styling");
    for title in ["Quiet", "High contrast", "Colored rail", "WGSL thumb"] {
        app.assert_text(title);
    }
    let track = app.bounds("effect_drag").unwrap();
    let x = track.origin.x + track.size.width / 2.0;
    let strength = |app: &TestApp<Main>| {
        let tree = app.rendered_tree();
        tree.node_ids()
            .iter()
            .filter_map(|id| tree.element_for(*id))
            .flat_map(|element| element.effects.iter())
            .flat_map(|effect| effect.layer.filters.iter())
            .find_map(|filter| match filter {
                Filter::Effect(instance) => match instance.parameters[0].value {
                    EffectValue::F32(value) => Some(value),
                    _ => None,
                },
                _ => None,
            })
            .unwrap()
    };
    let idle_strength = strength(&app);
    app.pointer_move(Point::new(x, track.origin.y + 12.0))
        .unwrap();
    assert!(strength(&app) > idle_strength);
    app.drag(
        Point::new(x, track.origin.y + 8.0),
        Point::new(x, track.origin.y + track.size.height - 8.0),
        5,
    )
    .unwrap();
    assert!(app.scroll_offset("effect_viewport").unwrap().y > 0.0);
}

/// The examples heading shares the scrollable sidebar with component entries.
#[test]
fn gallery_sidebar_wheel_reaches_the_example_category() {
    let mut app = TestApp::new(Main::new());
    app.resize(Size::new(960.0, 670.0)).unwrap();
    let sidebar = app.bounds("sidebar").unwrap();
    app.wheel(
        Point::new(
            sidebar.origin.x + sidebar.size.width / 2.0,
            sidebar.origin.y + sidebar.size.height / 2.0,
        ),
        ScrollDelta::Pixels(Point::new(0.0, -500.0)),
    )
    .unwrap();
    assert!(app.scroll_offset("sidebar").unwrap().y > 0.0);
    app.assert_text("EXAMPLES");
    navigate_to_page(&mut app, "Scrollbar styling");
    app.assert_text("WGSL thumb");
}

/// Every enabled button click advances the same counter and identifies its variant.
#[test]
fn gallery_button_variants_report_each_click() {
    let mut app = TestApp::new(Main::new());
    app.assert_text("Clicks: 0 · Last used: None");
    for (index, label) in ["Primary", "Secondary", "Outline", "Ghost", "Delete", "Star"]
        .into_iter()
        .enumerate()
    {
        app.get_by_role(Role::Button, label).click().unwrap();
        app.assert_text(&format!("Clicks: {} · Last used: {label}", index + 1));
    }
    app.get_by_role(Role::Button, "Star").click().unwrap();
    app.assert_text("Clicks: 7 · Last used: Star");
}

/// The composed slider changes its value through pointer coordinates on the track.
#[test]
fn gallery_slider_drags_without_a_native_slider_widget() {
    let mut app = TestApp::new(Main::new());
    navigate_to_page(&mut app, "Slider");
    app.assert_text("Volume: 25");
    let bounds = app
        .bounds(argui_testing::Selector::role(Role::Slider, "Volume"))
        .unwrap();
    let y = bounds.origin.y + bounds.size.height / 2.0;
    app.drag(
        Point::new(bounds.origin.x + 20.0, y),
        Point::new(bounds.origin.x + bounds.size.width - 20.0, y),
        4,
    )
    .unwrap();
    assert!(app.semantics().nodes.iter().any(|node| {
        node.semantics
            .label
            .as_deref()
            .is_some_and(|label| label.starts_with("Volume: ") && label != "Volume: 25")
    }));
    assert!(app.semantics().nodes.iter().any(|node| {
        node.semantics
            .label
            .as_deref()
            .is_some_and(|label| label.starts_with("Updates: ") && label != "Updates: 0")
    }));
}

/// The composed text area keeps a multiline controlled value and emits edits.
#[test]
fn gallery_text_area_edits_multiple_lines() {
    let mut app = TestApp::new(Main::new());
    let sidebar = app.bounds("sidebar").unwrap();
    app.wheel(
        Point::new(sidebar.origin.x + 120.0, sidebar.origin.y + 100.0),
        ScrollDelta::Pixels(Point::new(0.0, -2_000.0)),
    )
    .unwrap();
    app.get_by_role(Role::Button, "Text area").click().unwrap();
    let notes = argui_testing::Selector::role(Role::TextInput, "Notes");
    app.replace_text(notes.clone(), "A first line\nSecond line")
        .unwrap();
    app.assert_input_value(notes, "A first line\nSecond line");
    app.assert_text("Edits: 1");
}

/// A filled editor keeps its content scrolling inside a fixed viewport and exposes a usable corner.
#[test]
fn gallery_text_area_scrolls_and_resizes_with_its_visible_handle() {
    let mut app = TestApp::new(Main::new());
    let sidebar = app.bounds("sidebar").unwrap();
    app.wheel(
        Point::new(sidebar.origin.x + 120.0, sidebar.origin.y + 100.0),
        ScrollDelta::Pixels(Point::new(0.0, -2_000.0)),
    )
    .unwrap();
    app.get_by_role(Role::Button, "Text area").click().unwrap();
    let notes = argui_testing::Selector::role(Role::TextInput, "Notes");
    let value = (0..36)
        .map(|line| format!("Note {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    app.replace_text(notes.clone(), &value).unwrap();
    app.assert_input_value(notes.clone(), &value);
    app.focus(notes.clone()).unwrap();
    for _ in 0..16 {
        app.key(Key::Enter, Modifiers::default()).unwrap();
    }
    app.assert_input_value(notes.clone(), &format!("{value}{}", "\n".repeat(16)));
    let editor = app.bounds(notes.clone()).unwrap();
    assert!(editor.size.height <= 170.0);
    assert!(app.scroll_offset(notes).unwrap().y > 0.0);

    let handle = argui_testing::Selector::role(Role::Separator, "Resize notes editor");
    let before = app.bounds(handle.clone()).unwrap();
    assert!(before.size.width >= 18.0 && before.size.height >= 18.0);
    let start = Point::new(before.origin.x + 9.0, before.origin.y + 9.0);
    app.drag(start, Point::new(start.x + 45.0, start.y + 35.0), 5)
        .unwrap();
    let after = app.bounds(handle).unwrap();
    assert!(after.origin.x >= before.origin.x + 40.0);
    assert!(after.origin.y >= before.origin.y + 30.0);
}

/// The DSL overlays open, close, and handle Escape through portal policy.
#[test]
fn gallery_overlays_use_popup_window_lifecycle() {
    let mut app = TestApp::new(Main::new());
    navigate_to_page(&mut app, "Overlays");
    app.get_by_role(Role::Button, "Open popover")
        .click()
        .unwrap();
    app.assert_text("Popover content");
    app.get_by_role(Role::Button, "Close popover")
        .click()
        .unwrap();
    app.assert_no_text("Popover content");

    app.get_by_role(Role::Button, "Open dialog")
        .click()
        .unwrap();
    app.assert_text("Dialog title");
    app.key(Key::Escape, Modifiers::default()).unwrap();
    app.assert_no_text("Dialog title");
}

/// Menu, select, and editable combobox own their interaction state in DSL.
#[test]
fn gallery_selection_controls_choose_and_filter_options() {
    let mut app = TestApp::new(Main::new());
    navigate_to_page(&mut app, "Selection");
    app.get_by_role(Role::Button, "Open menu").click().unwrap();
    app.get_by_role(Role::MenuItem, "Share").click().unwrap();
    app.assert_text("Menu choice: Share");

    app.get_by_role(Role::ComboBox, "Color").click().unwrap();
    app.get_by_role(Role::MenuItem, "Emerald").click().unwrap();
    app.assert_text("Selected color: Emerald");

    let fruit = argui_testing::Selector::role(Role::SearchInput, "Fruit");
    app.focus(fruit.clone()).unwrap();
    app.replace_text(fruit, "Ap").unwrap();
    app.assert_text("Apple");
    app.assert_text("Apricot");
    app.assert_no_text("Banana");
    app.get_by_role(Role::MenuItem, "Apricot").click().unwrap();
    app.assert_text("Fruit: Apricot");
}

/// A composed menu moves focus with arrow keys and activates the selected item.
#[test]
fn gallery_menu_supports_keyboard_navigation() {
    let mut app = TestApp::new(Main::new());
    navigate_to_page(&mut app, "Selection");
    app.get_by_role(Role::Button, "Open menu").click().unwrap();
    app.key(Key::ArrowDown, Modifiers::default()).unwrap();
    app.key(Key::Enter, Modifiers::default()).unwrap();
    app.assert_text("Menu choice: Share");
}

/// A DSL ScrollView moves its native viewport while the author draws its chrome.
#[test]
fn gallery_scroll_view_updates_observed_position() {
    let mut app = TestApp::new(Main::new());
    navigate_to_page(&mut app, "ScrollView");
    app.assert_text("Scroll position: 0 px");
    let bounds = app.bounds("scroll_preview").unwrap();
    app.wheel(
        Point::new(
            bounds.origin.x + bounds.size.width / 2.0,
            bounds.origin.y + bounds.size.height / 2.0,
        ),
        ScrollDelta::Pixels(Point::new(0.0, -180.0)),
    )
    .unwrap();
    assert!(app.semantics().nodes.iter().any(|node| {
        node.semantics.label.as_deref().is_some_and(|label| {
            label.starts_with("Scroll position: ") && label != "Scroll position: 0 px"
        })
    }));
}

/// The scroll thumb is authored in DSL and drives the viewport through a child reference.
#[test]
fn gallery_scroll_view_thumb_drags() {
    let mut app = TestApp::new(Main::new());
    navigate_to_page(&mut app, "ScrollView");
    let track = app.bounds("scrollbar_drag").unwrap();
    let x = track.origin.x + track.size.width / 2.0;
    app.drag(
        Point::new(x, track.origin.y + 10.0),
        Point::new(x, track.origin.y + track.size.height - 10.0),
        5,
    )
    .unwrap();
    assert!(
        app.scroll_offset("viewport").unwrap().y > 0.0,
        "requests: {:?}",
        app.scroll_requests()
    );
}

/// Authored geometry and project assets mount with the interactive motion control.
#[test]
fn gallery_visual_composition_controls_motion() {
    let main = Main::new();
    let definitions = main.effect_definitions();
    let aurora = definitions
        .iter()
        .find(|definition| {
            definition
                .parameters
                .iter()
                .any(|parameter| parameter.name.as_str() == "strength")
        })
        .expect("visual composition effect must be registered");
    assert!(aurora.passes[0].wgsl.contains("fn argui_effect"));
    assert_eq!(aurora.damage, EffectDamage::Bounded);
    let mut app = TestApp::new(main);
    navigate_to_page(&mut app, "Visual composition");
    app.assert_text("A small universe");
    let bounds = app.bounds("visual_path").unwrap();
    assert!(bounds.size.width > 0.0 && bounds.size.height > 0.0);
    let strength = |app: &TestApp<Main>| {
        let root = app.entity().render();
        let mut nodes = vec![&root];
        while let Some(element) = nodes.pop() {
            if element.key.as_deref() == Some("visual_path") {
                let [Filter::Effect(instance)] = element.effects[0].layer.filters.as_slice() else {
                    panic!("visual path must carry its authored effect");
                };
                assert!(
                    definitions
                        .iter()
                        .any(|definition| definition.id == instance.id)
                );
                let EffectValue::F32(value) = &instance.parameters[0].value else {
                    panic!("strength must be a float");
                };
                return *value;
            }
            nodes.extend(&element.children);
        }
        panic!("visual path was not mounted");
    };
    assert_eq!(strength(&app), 0.7);
    app.get_by_role(Role::Button, "Pause motion")
        .click()
        .unwrap();
    app.assert_text("Motion is paused");
    assert_eq!(strength(&app), 0.1);
    app.get_by_role(Role::Button, "Play motion")
        .click()
        .unwrap();
    app.assert_text("Motion is playing");
    assert_eq!(strength(&app), 0.7);
}

/// The restored input examples expose controlled values and locked semantics.
#[test]
fn gallery_input_page_restores_controlled_and_locked_examples() {
    let mut app = TestApp::new(Main::new());
    navigate_to_page(&mut app, "Input & Search");
    app.assert_text("Controlled fields");
    for label in [
        "Full name",
        "Email address",
        "Invalid field",
        "Read-only",
        "Disabled",
        "Standard caret",
        "Rounded dot caret",
        "Gradient ellipsis caret",
        "Linear selection highlight",
        "Radial selection highlight",
        "Conic selection highlight",
    ] {
        assert!(
            app.semantics().nodes.iter().any(|node| {
                node.semantics.role == Role::TextInput
                    && node.semantics.label.as_deref() == Some(label)
            }),
            "missing {label} input"
        );
    }
    assert!(app.semantics().nodes.iter().any(|node| {
        node.semantics.role == Role::SearchInput
            && node.semantics.label.as_deref() == Some("Search the GPU graph")
    }));
    app.assert_input_value(
        argui_testing::Selector::role(Role::TextInput, "Full name"),
        "Ada Lovelace",
    );
    app.assert_text("Enter an email with @ and a domain.");
    let nodes = &app.semantics().nodes;
    assert!(nodes.iter().any(|node| {
        node.semantics.label.as_deref() == Some("Read-only") && node.semantics.state.read_only
    }));
    assert!(nodes.iter().any(|node| {
        node.semantics.label.as_deref() == Some("Disabled") && node.semantics.state.disabled
    }));
}

/// The dedicated gallery page uses the shared virtualized list and scrolls rows.
#[test]
fn gallery_virtual_list_page_scrolls_independently() {
    let mut app = TestApp::new(Main::new());
    navigate_to_page(&mut app, "VList");
    app.assert_text("Scroll through 10,000 catalogue entries");
    app.assert_text("Scroll offset: 0 px");
    let mounted_options = app
        .semantics()
        .nodes
        .iter()
        .filter(|node| node.semantics.role == Role::Option)
        .count();
    assert!((1..30).contains(&mounted_options));
    app.wheel(
        Point::new(500.0, 420.0),
        ScrollDelta::Pixels(Point::new(0.0, -400.0)),
    )
    .unwrap();
    assert!(app.semantics().nodes.iter().any(|node| {
        node.semantics.label.as_deref().is_some_and(|label| {
            label.contains("Scroll offset: ") && !label.contains("Scroll offset: 0 px")
        })
    }));
}

/// The virtual list thumb changes its DSL-owned offset while rows stay windowed.
#[test]
fn gallery_virtual_list_thumb_drags() {
    let mut app = TestApp::new(Main::new());
    navigate_to_page(&mut app, "VList");
    let bounds = app.bounds("examples_frame").unwrap();
    let x = bounds.origin.x + bounds.size.width - 6.0;
    app.drag(
        Point::new(x, bounds.origin.y + 10.0),
        Point::new(x, bounds.origin.y + bounds.size.height - 10.0),
        5,
    )
    .unwrap();
    assert!(app.semantics().nodes.iter().any(|node| {
        node.semantics.label.as_deref().is_some_and(|label| {
            label.contains("Scroll offset: ") && !label.contains("Scroll offset: 0 px")
        })
    }));
}

/// Appearance selection updates the trigger and shared gallery palette.
#[test]
fn gallery_theme_selector_changes_mode_and_accent() {
    let mut app = TestApp::new(Main::new());
    app.get_by_role(Role::ComboBox, "Light · appearance")
        .click()
        .unwrap();
    app.assert_text("Appearance");
    app.get_by_role(Role::Button, "Dark").click().unwrap();
    app.assert_text("Dark · appearance");
    app.get_by_role(Role::ComboBox, "Dark · appearance")
        .click()
        .unwrap();
    app.get_by_role(Role::Button, "Emerald").click().unwrap();
    app.assert_text("Dark · appearance");
}

/// The DSL switch retains controlled state and exposes an accessible toggle.
#[test]
fn gallery_switch_page_toggles_without_native_widget_source() {
    let mut app = TestApp::new(Main::new());
    navigate_to_page(&mut app, "Switch");
    app.assert_text("Notifications enabled");
    assert!(app.semantics().nodes.iter().any(|node| {
        node.semantics.role == Role::Switch
            && node.semantics.label.as_deref() == Some("Notifications")
            && node.semantics.state.checked == Some(argui::ui::CheckedState::Checked)
    }));
    app.get_by_role(Role::Switch, "Notifications")
        .click()
        .unwrap();
    app.assert_text("Notifications disabled");
    assert!(app.semantics().nodes.iter().any(|node| {
        node.semantics.role == Role::Switch
            && node.semantics.label.as_deref() == Some("Notifications")
            && node.semantics.state.checked == Some(argui::ui::CheckedState::Unchecked)
    }));
}

/// The content sits next to navigation when wide and below it when narrow.
#[test]
fn gallery_wraps_sidebar_and_content_responsively() {
    let mut app = TestApp::new(Main::new());
    let coordinates = |app: &TestApp<Main>| {
        let nodes = &app.semantics().nodes;
        let navigation = nodes
            .iter()
            .find(|node| {
                node.semantics.role == Role::Button
                    && node.semantics.label.as_deref() == Some("Button")
            })
            .unwrap()
            .bounds
            .origin;
        let heading = nodes
            .iter()
            .find(|node| {
                node.semantics.role == Role::Text
                    && node.semantics.label.as_deref() == Some("Button")
            })
            .unwrap()
            .bounds
            .origin;
        (navigation, heading)
    };
    let (wide_nav, wide_heading) = coordinates(&app);
    assert!(wide_heading.x > wide_nav.x);
    assert!(
        wide_heading.y < wide_nav.y + 100.0,
        "wide navigation={wide_nav:?}, heading={wide_heading:?}"
    );

    app.resize(Size::new(420.0, 760.0)).unwrap();
    for _ in 0..10 {
        app.wheel(
            Point::new(100.0, 300.0),
            ScrollDelta::Pixels(Point::new(0.0, -700.0)),
        )
        .unwrap();
        if app.scroll_offset("workspace").unwrap().y > 0.0 {
            break;
        }
    }
    assert!(app.scroll_offset("workspace").unwrap().y > 0.0);
    let (_, narrow_heading) = coordinates(&app);
    assert!(
        narrow_heading.y > 100.0,
        "narrow heading={narrow_heading:?}"
    );
}
