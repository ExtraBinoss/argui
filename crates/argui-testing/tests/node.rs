use argui_accessibility::{SemanticAction, SemanticValue};
use argui_core::{Key, Modifiers, Point};
use argui_runtime::{Context, Render};
use argui_testing::{SemanticMatcher, TestApp};
use argui_ui::{Element, length};
use argui_widgets::{
    Breadcrumb, BreadcrumbLink, Button, Collapsible, Dialog, DialogBehavior, Input, Pagination,
    RangeConfig, Slider, Toggle, default_theme,
};

#[derive(Default)]
struct HandlerMatrix {
    toggled: bool,
    volume: f32,
    volume_commits: usize,
    open: bool,
    page: usize,
    destination: String,
    expanded: bool,
}

impl Render for HandlerMatrix {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let environment = cx.environment();
        let theme = default_theme(environment.clone())
            .resolve(environment.color_scheme)
            .clone();
        let dialog = DialogBehavior::new("settings", "Settings", self.open);
        Element::column([
            Toggle::new("notifications", "Notifications", self.toggled)
                .on_change(cx.value_callback(|app, value| app.toggled = value))
                .build(&theme),
            Slider::new(
                "volume",
                "Volume",
                self.volume,
                RangeConfig::new(0.0, 10.0, 1.0),
            )
            .on_change(cx.value_callback(|app, value| app.volume = value))
            .on_commit(cx.value_callback(|app, _| app.volume_commits += 1))
            .build(&theme)
            .width(length(240.0)),
            Pagination::new("pages", self.page.max(1), 4)
                .on_select(cx.value_callback(|app, page| app.page = page))
                .build(&theme),
            Breadcrumb::new("crumbs", [BreadcrumbLink::new("home", "Home")], "Current")
                .on_activate(cx.value_callback(|app, id| app.destination = id))
                .build(&theme),
            Collapsible::new(
                "details",
                "Details",
                self.expanded,
                Element::text("Expanded content"),
            )
            .on_open_change(cx.value_callback(|app, open| app.expanded = open))
            .build(&theme),
            Dialog::new(
                "settings",
                "Settings",
                self.open,
                Button::new(dialog.trigger_key(), "Open settings", theme.button()).build(),
                Element::text("Dialog content"),
            )
            .on_open_change(cx.value_callback(|app, open| app.open = open))
            .build(&theme),
        ])
        .gap(12.0)
        .width(length(480.0))
    }
}

#[test]
fn typed_handlers_cover_selection_range_navigation_and_overlay_families() {
    let mut app = TestApp::new(HandlerMatrix {
        page: 1,
        ..HandlerMatrix::default()
    });

    app.click("notifications").unwrap();
    let slider = app.bounds("volume").unwrap();
    app.drag(
        Point::new(
            slider.origin.x + 1.0,
            slider.origin.y + slider.size.height * 0.5,
        ),
        Point::new(
            slider.origin.x + slider.size.width * 0.5,
            slider.origin.y + slider.size.height * 0.5,
        ),
        3,
    )
    .unwrap();
    app.focus("volume").unwrap();
    app.key(Key::ArrowRight, Modifiers::default()).unwrap();
    app.click("pages::next").unwrap();
    app.click("crumbs::link::home").unwrap();
    app.click("details::trigger").unwrap();
    app.assert_text("Expanded content");
    app.click("settings::trigger").unwrap();

    app.assert_text("Dialog content");
    assert_eq!(
        app.entity().read(|state| (
            state.toggled,
            state.volume,
            state.volume_commits,
            state.page,
            state.destination.clone(),
            state.expanded,
            state.open,
        )),
        (true, 6.0, 2, 2, "home".to_owned(), true, true)
    );
}

#[derive(Default)]
struct NodeForm {
    value: String,
    submissions: usize,
}

impl Render for NodeForm {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let environment = cx.environment();
        let theme = default_theme(environment.clone())
            .resolve(environment.color_scheme)
            .clone();
        Element::column([
            Input::new("node-input", &self.value, "Node input", theme.input())
                .label("Node input")
                .on_input(cx.input_callback(|app, value| app.value = value))
                .on_submit(cx.submit_callback(|app, _| app.submissions += 1))
                .build(),
            Button::new("node-button", "Node button", theme.button()).build(),
            Button::new("node-disabled", "Node disabled", theme.button())
                .enabled(false)
                .build(),
        ])
        .width(length(300.0))
    }
}

#[test]
fn node_handles_forward_the_complete_action_surface() {
    let mut app = TestApp::new(NodeForm::default());

    app.get_by_key("node-button").click().unwrap();
    app.get_by_text("Node button").focus().unwrap();
    app.get_by_label("Node input").type_text("typed").unwrap();
    app.get_by_role(argui_accessibility::Role::TextInput, "Node input")
        .replace_text("replaced")
        .unwrap();
    app.get_by_key("node-input").paste(" pasted").unwrap();
    app.get_by_key("node-input").submit().unwrap();
    app.get_by_key("node-input")
        .accessibility_action(
            SemanticAction::SetValue,
            Some(SemanticValue::Text("semantic".to_owned())),
        )
        .unwrap();
    app.get_by_state(SemanticMatcher::Disabled)
        .accessibility_action(SemanticAction::Increment, None)
        .unwrap();
    app.focused().focus().unwrap();

    assert_eq!(
        app.entity()
            .read(|state| (state.value.clone(), state.submissions)),
        ("semantic".to_owned(), 1)
    );
}
