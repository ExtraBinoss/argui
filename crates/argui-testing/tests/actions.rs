use argui_accessibility::{Role, SemanticAction, SemanticValue, Semantics};
use argui_core::{Key, Modifiers, Point, ScrollDelta, Size};
use argui_runtime::{Context, Render};
use argui_testing::{SelectorCount, TestApp, TestError};
use argui_ui::{Element, EventType, length};
use argui_widgets::{Button, Input, ScrollArea, default_theme};

#[derive(Default)]
struct ButtonMatrix {
    pointer: usize,
    touch: usize,
    keyboard: usize,
    accessibility: usize,
    disabled: usize,
    busy: usize,
    custom: usize,
    multiple: usize,
    direct_bubble: usize,
    parent_bubble: usize,
    cancelled: usize,
    cancelled_parent: usize,
    stale_replacement: usize,
    replace_handler: bool,
}

impl Render for ButtonMatrix {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let environment = cx.environment();
        let theme = default_theme(environment.clone())
            .resolve(environment.color_scheme)
            .clone();
        let button = |key, label| Button::new(key, label, theme.button());
        let replacement_increment = if self.replace_handler { 10 } else { 1 };

        let bubble = Element::container([button("bubble", "Bubble")
            .on_click(cx.callback(|app| app.direct_bubble += 1))
            .build()])
        .on(cx.listener(EventType::Click, |app, event, _| {
            if event.target_key() == Some("bubble") {
                app.parent_bubble += 1;
            }
        }));
        let cancelled = Element::container([button("cancelled", "Cancelled")
            .on_click(cx.event_handler(|app, event, cx| {
                app.cancelled += 1;
                event.stop_propagation();
                cx.notify();
            }))
            .build()])
        .on(cx.listener(EventType::Click, |app, event, _| {
            if event.target_key() == Some("cancelled") {
                app.cancelled_parent += 1;
            }
        }));

        Element::column([
            button("pointer", "Pointer")
                .on_click(cx.callback(|app| app.pointer += 1))
                .build(),
            button("touch", "Touch")
                .on_click(cx.callback(|app| app.touch += 1))
                .build(),
            button("keyboard", "Keyboard")
                .on_click(cx.callback(|app| app.keyboard += 1))
                .build(),
            button("accessibility", "Accessibility")
                .on_click(cx.callback(|app| app.accessibility += 1))
                .build(),
            button("disabled", "Disabled")
                .enabled(false)
                .on_click(cx.callback(|app| app.disabled += 1))
                .build(),
            button("busy", "Busy")
                .loading(Element::text("Loading"))
                .on_click(cx.callback(|app| app.busy += 1))
                .build(),
            button("custom", "Custom content")
                .content(Element::row([Element::text("Save"), Element::text("now")]))
                .on_click(cx.callback(|app| app.custom += 1))
                .build(),
            button("multiple", "Multiple")
                .on_click(cx.callback(|app| app.multiple += 1))
                .on_click(cx.callback(|app| app.multiple += 10))
                .build(),
            bubble,
            cancelled,
            button("replacement", "Replacement")
                .on_click(cx.callback(move |app| {
                    app.stale_replacement += replacement_increment;
                    app.replace_handler = true;
                }))
                .build(),
        ])
        .gap(4.0)
        .width(length(320.0))
    }
}

#[test]
fn direct_button_handlers_cover_every_activation_path_and_guard() {
    let mut app = TestApp::new(ButtonMatrix::default());

    app.click("pointer").unwrap();
    app.tap("touch").unwrap();
    app.focus("keyboard").unwrap();
    app.key(Key::Enter, Modifiers::default()).unwrap();
    app.key(Key::Character(" ".to_owned()), Modifiers::default())
        .unwrap();
    app.accessibility_action("accessibility", SemanticAction::Click, None)
        .unwrap();
    app.click("disabled").unwrap();
    app.accessibility_action("busy", SemanticAction::Click, None)
        .unwrap();
    app.click("custom").unwrap();
    app.click("multiple").unwrap();
    app.click("bubble").unwrap();
    app.click("cancelled").unwrap();
    app.click("replacement").unwrap();
    app.click("replacement").unwrap();

    assert_eq!(
        app.entity().read(|state| [
            state.pointer,
            state.touch,
            state.keyboard,
            state.accessibility,
            state.disabled,
            state.busy,
            state.custom,
            state.multiple,
            state.direct_bubble,
            state.parent_bubble,
            state.cancelled,
            state.cancelled_parent,
            state.stale_replacement,
        ]),
        [1, 1, 2, 1, 0, 0, 1, 11, 1, 1, 1, 0, 11]
    );
}

#[derive(Default)]
struct ActionErrors {
    value: String,
}

impl Render for ActionErrors {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let environment = cx.environment();
        let theme = default_theme(environment.clone())
            .resolve(environment.color_scheme)
            .clone();
        let input = cx.input_callback(|state, value| state.value = value);
        let content = Element::column([
            Element::container([]).height(length(280.0)),
            Element::text("Scroll target"),
        ]);
        Element::column([
            Input::new("field", &self.value, "Value", theme.input())
                .label("Value")
                .on_input(input)
                .build(),
            Button::new("plain", "Plain", theme.button()).build(),
            Button::new("prevented", "Prevented", theme.button())
                .on_click(cx.event_handler(|_, event, _| {
                    let _ = event.prevent_default();
                }))
                .build(),
            Button::new("disabled-action", "Disabled action", theme.button())
                .enabled(false)
                .build(),
            Element::container([])
                .keyed("semantic-input")
                .semantics(Semantics::new(Role::TextInput).label("Semantic input")),
            Element::container([])
                .keyed("zero-bounds")
                .width(length(0.0))
                .height(length(0.0)),
            ScrollArea::new("scroll", "Scroll", 80.0, content).build(&theme),
            ScrollArea::new(
                "blocked-scroll",
                "Blocked scroll",
                80.0,
                Element::container([]).height(length(280.0)),
            )
            .build(&theme)
            .on(cx.listener(EventType::Wheel, |_, event, _| {
                let _ = event.prevent_default();
            })),
        ])
        .gap(4.0)
        .width(length(300.0))
    }
}

#[test]
fn action_helpers_cover_validation_accessibility_and_scroll_branches() {
    let mut app = TestApp::new(ActionErrors::default());
    app.set_settle_limit(0);

    app.pointer_move(Point::new(4.0, 4.0)).unwrap();
    assert!(matches!(
        app.pointer_move(Point::new(-1.0, 4.0)).unwrap_err(),
        TestError::MissingBounds { .. }
    ));
    assert!(matches!(
        app.drag(Point::new(-1.0, 0.0), Point::new(1.0, 1.0), 1)
            .unwrap_err(),
        TestError::MissingBounds { .. }
    ));
    assert!(matches!(
        app.drag(Point::new(1.0, 1.0), Point::new(4000.0, 1.0), 1)
            .unwrap_err(),
        TestError::MissingBounds { .. }
    ));
    let button = app.bounds("plain").unwrap();
    let center = Point::new(
        button.origin.x + button.size.width * 0.5,
        button.origin.y + button.size.height * 0.5,
    );
    app.drag(center, center, 0).unwrap();
    app.click("prevented").unwrap();
    app.tap("prevented").unwrap();
    app.focus("plain").unwrap();
    app.shortcut(
        Key::Character("k".to_owned()),
        Modifiers {
            control: true,
            ..Modifiers::default()
        },
    )
    .unwrap();
    app.blur().unwrap();

    for error in [
        app.type_text("plain", "x").unwrap_err(),
        app.replace_text("plain", "x").unwrap_err(),
        app.paste("plain", "x").unwrap_err(),
        app.submit("plain").unwrap_err(),
    ] {
        assert!(matches!(error, TestError::NotTextInput { .. }));
    }
    app.accessibility_action("plain", SemanticAction::Focus, None)
        .unwrap();
    app.accessibility_action("plain", SemanticAction::Blur, None)
        .unwrap();
    app.accessibility_action(
        "field",
        SemanticAction::SetValue,
        Some(SemanticValue::Text("replacement".to_owned())),
    )
    .unwrap();
    app.assert_input_value("field", "replacement");
    assert!(matches!(
        app.accessibility_action("plain", SemanticAction::SetValue, None)
            .unwrap_err(),
        TestError::UnsupportedAction { .. }
    ));
    assert!(matches!(
        app.accessibility_action(
            "semantic-input",
            SemanticAction::SetValue,
            Some(SemanticValue::Text("missing editor".to_owned())),
        )
        .unwrap_err(),
        TestError::NotTextInput { .. }
    ));
    assert!(matches!(
        app.accessibility_action("plain", SemanticAction::Increment, None)
            .unwrap_err(),
        TestError::UnsupportedAction { .. }
    ));
    app.accessibility_action("disabled-action", SemanticAction::Increment, None)
        .unwrap();

    let scroll = app.bounds("scroll").unwrap();
    let scroll_point = Point::new(scroll.origin.x + 8.0, scroll.origin.y + 8.0);
    app.wheel(scroll_point, ScrollDelta::Pixels(Point::new(0.0, -80.0)))
        .unwrap();
    assert!(app.scroll_offset("scroll").unwrap().y > 0.0);
    let blocked = app.bounds("blocked-scroll").unwrap();
    let blocked_point = Point::new(blocked.origin.x + 8.0, blocked.origin.y + 8.0);
    app.wheel(blocked_point, ScrollDelta::Pixels(Point::new(0.0, -80.0)))
        .unwrap();
    assert_eq!(app.scroll_offset("blocked-scroll").unwrap().y, 0.0);
    app.wheel(
        Point::new(900.0, 700.0),
        ScrollDelta::Pixels(Point::new(0.0, -80.0)),
    )
    .unwrap();

    assert!(matches!(
        app.click("zero-bounds").unwrap_err(),
        TestError::MissingBounds { .. }
    ));

    app.resize(Size::new(-10.0, 0.0)).unwrap();
    assert_eq!(app.take_commands(), []);
    assert!(matches!(
        app.click("missing").unwrap_err(),
        TestError::Selector {
            count: SelectorCount::None,
            ..
        }
    ));
}
