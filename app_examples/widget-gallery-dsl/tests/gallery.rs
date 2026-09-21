use argui::{
    core::{Point, ScrollDelta, Size},
    ui::Role,
};
use argui_example_widget_gallery_dsl::Main;
use argui_testing::TestApp;

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

/// Navigation opens every separately authored component page.
#[test]
fn gallery_navigation_opens_component_pages() {
    let mut app = TestApp::new(Main::new());
    for (navigation, title) in [
        ("Input", "Controlled values, search"),
        ("Card", "A composable surface"),
        ("Badge", "Compact status markers"),
        ("Separator", "A quiet divider"),
        ("Switch", "A theme-aware toggle"),
        ("Media", "Project images and SVGs"),
        ("Virtual List", "One reusable DSL component"),
    ] {
        app.get_by_role(Role::Button, navigation).click().unwrap();
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

/// The restored input examples expose controlled values and locked semantics.
#[test]
fn gallery_input_page_restores_controlled_and_locked_examples() {
    let mut app = TestApp::new(Main::new());
    app.get_by_role(Role::Button, "Input").click().unwrap();
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
    app.get_by_role(Role::Button, "Virtual List")
        .click()
        .unwrap();
    app.assert_text("Scroll through 30 catalogue entries");
    app.assert_text("Scroll offset: 0 px");
    app.wheel(
        Point::new(500.0, 420.0),
        ScrollDelta::Pixels(Point::new(0.0, -400.0)),
    )
    .unwrap();
    assert!(app.semantics().nodes.iter().any(|node| {
        node.semantics
            .label
            .as_deref()
            .is_some_and(|label| label.starts_with("Scroll offset: ") && label != "Scroll offset: 0 px")
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
    app.get_by_role(Role::Button, "Switch").click().unwrap();
    app.assert_text("Notifications enabled");
    app.get_by_role(Role::Switch, "Notifications")
        .click()
        .unwrap();
    app.assert_text("Notifications disabled");
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
    app.wheel(
        Point::new(100.0, 300.0),
        ScrollDelta::Pixels(Point::new(0.0, -700.0)),
    )
    .unwrap();
    assert!(app.scroll_offset("workspace").unwrap().y > 0.0);
    let (_, narrow_heading) = coordinates(&app);
    assert!(
        narrow_heading.y > 100.0,
        "narrow heading={narrow_heading:?}"
    );
}
