use argui_accessibility::{CheckedState, Role, SemanticAction};
use argui_core::{Color, ColorScheme, Key, Modifiers, Point, Rect, Size};
use argui_runtime::{AppCommand, Context, LayoutSnapshot, Render, ThemeRequest};
use argui_testing::{Selector, SelectorCount, TestApp, TestError, TestWindows};
use argui_ui::{ClipboardRequest, Element, ScrollAlignment, ScrollRequest, Sides, length};
use argui_widgets::{Button, Checkbox, Input, ScrollArea, VList, default_theme};

#[derive(Default)]
struct Counter {
    count: usize,
}

impl Render for Counter {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let environment = cx.environment();
        let theme = default_theme(environment.clone())
            .resolve(environment.color_scheme)
            .clone();
        let increment = cx.callback(|counter| counter.count += 1);
        Element::column([
            Element::text(format!("Count: {}", self.count)),
            Button::new("increment", "Increment", theme.button())
                .on_click(increment)
                .build(),
        ])
        .width(length(240.0))
    }
}

#[test]
fn click_uses_layout_hit_testing_and_settles_the_controlled_view() {
    let mut app = TestApp::new(Counter::default());

    app.get_by_role(Role::Button, "Increment").click().unwrap();

    app.assert_text("Count: 1");
    app.assert_focused("increment");
    app.run_until_idle().unwrap();
    app.assert_quiescent();
    assert_eq!(app.entity().read(|counter| counter.count), 1);
}

#[derive(Default)]
struct Form {
    email: String,
    accepted: CheckedState,
    submissions: usize,
}

impl Render for Form {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let environment = cx.environment();
        let theme = default_theme(environment.clone())
            .resolve(environment.color_scheme)
            .clone();
        let input = cx.input_callback(|form, email| form.email = email);
        let submit = cx.submit_callback(|form, _| form.submissions += 1);
        let accepted = cx.value_callback(|form, value| form.accepted = value);
        Element::column([
            Input::new("email", &self.email, "Email", theme.input())
                .label("Email")
                .on_input(input)
                .on_submit(submit)
                .build(),
            Checkbox::new("terms", "Accept terms", self.accepted)
                .on_change(accepted)
                .build(&theme),
        ])
        .width(length(320.0))
    }
}

#[test]
fn accessible_queries_edit_submit_and_toggle_through_typed_handlers() {
    let mut app = TestApp::new(Form::default());

    app.get_by_role(Role::TextInput, "Email")
        .type_text("person@example.com")
        .unwrap();
    app.get_by_role(Role::TextInput, "Email")
        .paste(".test")
        .unwrap();
    app.assert_clipboard(".test");
    app.get_by_role(Role::TextInput, "Email").submit().unwrap();
    app.key(Key::Tab, Modifiers::default()).unwrap();
    app.assert_focused("terms");
    app.get_by_role(Role::CheckBox, "Accept terms")
        .accessibility_action(SemanticAction::Click, None)
        .unwrap();

    app.assert_input_value(
        Selector::role(Role::TextInput, "Email"),
        "person@example.com.test",
    );
    assert_eq!(
        app.entity()
            .read(|form| (form.email.clone(), form.accepted, form.submissions)),
        (
            "person@example.com.test".to_owned(),
            CheckedState::Checked,
            1
        )
    );
}

struct Effects;

impl Render for Effects {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let environment = cx.environment();
        let theme = default_theme(environment.clone())
            .resolve(environment.color_scheme)
            .clone();
        Button::new("effects", "Apply effects", theme.button())
            .on_click(cx.event_handler(|_, _, cx| {
                cx.write_clipboard(ClipboardRequest::Write("copied".to_owned()));
                cx.set_theme(ThemeRequest {
                    color_scheme: Some(ColorScheme::Dark),
                    primary: None,
                });
                cx.command(AppCommand::Quit);
            }))
            .build()
    }
}

#[test]
fn effects_resize_environment_and_lifecycle_are_observable() {
    let mut app = TestApp::new(Effects);
    app.resize(Size::new(640.0, 360.0)).unwrap();
    let mut environment = app.environment().clone();
    environment.high_contrast = true;
    app.set_environment(environment).unwrap();

    app.click("effects").unwrap();

    app.assert_clipboard("copied");
    app.assert_commands(&[AppCommand::Quit]);
    assert_eq!(app.environment().color_scheme, ColorScheme::Dark);
    assert!(app.environment().high_contrast);
    app.lifecycle(argui_platform::PlatformEvent::VisibilityChanged(false))
        .unwrap();
    app.assert_no_text("Apply effects");
    app.lifecycle(argui_platform::PlatformEvent::VisibilityChanged(true))
        .unwrap();
    app.assert_text("Apply effects");
}

#[test]
fn duplicate_queries_return_actionable_diagnostics() {
    struct Duplicate;
    impl Render for Duplicate {
        fn render(&mut self, _cx: &mut Context<Self>) -> Element {
            Element::column([Element::text("same"), Element::text("same")])
        }
    }
    let mut app = TestApp::new(Duplicate);

    let error = app.get_by_text("same").click().unwrap_err();

    assert!(matches!(
        error,
        TestError::Selector {
            count: SelectorCount::Multiple(2),
            ..
        }
    ));
    assert!(error.to_string().contains("close candidates"));
}

#[test]
fn a_non_settling_layout_callback_hits_the_documented_bound() {
    struct Loop(bool);
    impl Render for Loop {
        fn render(&mut self, _cx: &mut Context<Self>) -> Element {
            Element::text("loop").width(length(if self.0 { 100.0 } else { 101.0 }))
        }

        fn layout_changed(&mut self, _layout: &LayoutSnapshot, cx: &mut Context<Self>) {
            self.0 = !self.0;
            cx.notify();
        }
    }

    let error = match TestApp::try_new(Loop(false)) {
        Ok(_) => panic!("layout loop unexpectedly settled"),
        Err(error) => error,
    };

    assert!(matches!(error, TestError::DidNotSettle { limit: 32, .. }));
}

#[test]
fn shared_model_has_independent_window_presentations() {
    let mut windows = TestWindows::new(Counter::default());
    let auxiliary = argui_platform::WindowKey::new("auxiliary");
    windows.open(auxiliary.clone()).unwrap();

    windows
        .window(&auxiliary)
        .unwrap()
        .click("increment")
        .unwrap();
    windows.settle().unwrap();

    windows
        .window(&argui_platform::WindowKey::main())
        .unwrap()
        .assert_text("Count: 1");
    assert_eq!(windows.entity().read(|counter| counter.count), 1);
    windows.close(&auxiliary).unwrap();
    assert_eq!(windows.window_keys(), [argui_platform::WindowKey::main()]);
}

#[cfg(feature = "tasks")]
struct TimedTask {
    completed: bool,
    task: Option<argui_runtime::tasks::TaskHandle>,
}

#[cfg(feature = "tasks")]
impl Render for TimedTask {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let environment = cx.environment();
        let theme = default_theme(environment.clone())
            .resolve(environment.color_scheme)
            .clone();
        let start = cx.event_handler(|task, _, cx| {
            task.task = Some(
                cx.spawn(
                    async {
                        argui_runtime::tasks::sleep(std::time::Duration::from_millis(500)).await;
                    },
                    |task, result, cx| {
                        result.unwrap();
                        task.completed = true;
                        task.task = None;
                        cx.notify();
                    },
                )
                .unwrap(),
            );
        });
        Element::column([
            Button::new("start", "Start", theme.button())
                .on_click(start)
                .build(),
            Element::text(if self.completed { "Done" } else { "Waiting" }),
        ])
    }
}

#[cfg(all(feature = "tasks", not(target_arch = "wasm32")))]
#[test]
fn controlled_time_completes_tasks_without_wall_clock_sleep() {
    let mut app = TestApp::new(TimedTask {
        completed: false,
        task: None,
    });
    app.click("start").unwrap();
    assert_eq!(app.pending_tasks(), 1);

    app.advance(std::time::Duration::from_millis(499)).unwrap();
    app.assert_text("Waiting");
    app.advance(std::time::Duration::from_millis(2)).unwrap();

    assert_eq!(app.pending_tasks(), 0);
    app.assert_text("Done");
    app.run_until_idle().unwrap();
    app.assert_quiescent();
}

#[cfg(all(feature = "tasks", not(target_arch = "wasm32")))]
#[test]
fn closing_a_window_cancels_its_presentation_task() {
    let mut windows = TestWindows::new(TimedTask {
        completed: false,
        task: None,
    });
    let auxiliary = argui_platform::WindowKey::new("task-window");
    windows
        .open(auxiliary.clone())
        .unwrap()
        .click("start")
        .unwrap();
    let token = windows.entity().read(|task| {
        task.task
            .as_ref()
            .expect("task started")
            .cancellation_token()
    });

    windows.close(&auxiliary).unwrap();

    assert!(token.is_cancelled());
    assert!(!windows.entity().read(|task| task.completed));
}

struct RevealDemo;

impl Render for RevealDemo {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let environment = cx.environment();
        let theme = default_theme(environment.clone())
            .resolve(environment.color_scheme)
            .clone();
        let reveal = cx.event_handler(|_, _, cx| {
            cx.scroll(argui_ui::ScrollRequest::reveal("target"));
        });
        let content = Element::column([
            Element::container([]).height(length(320.0)),
            Element::text("Target").keyed("target").height(length(32.0)),
        ]);
        Element::column([
            Button::new("reveal", "Reveal", theme.button())
                .on_click(reveal)
                .build(),
            ScrollArea::new("viewport", "Results", 96.0, content).build(&theme),
        ])
        .width(length(320.0))
    }
}

#[test]
fn programmatic_reveal_applies_to_real_scroll_layout() {
    let mut app = TestApp::new(RevealDemo);
    assert_eq!(app.scroll_offset("viewport").unwrap().y, 0.0);

    app.click("reveal").unwrap();

    assert!(app.scroll_offset("viewport").unwrap().y > 0.0);
    assert!(app.scroll_requests().iter().any(|request| matches!(
        &request.target,
        argui_ui::ScrollTarget::Element(argui_ui::FocusTarget::Key(key)) if key == "target"
    )));
}

struct Virtualized;

impl Render for Virtualized {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let environment = cx.environment();
        let theme = default_theme(environment.clone())
            .resolve(environment.color_scheme)
            .clone();
        VList::new("virtual", 20.0, 100.0, 800.0).build(100, &theme, |index| {
            Element::text(format!("Virtual row {index}")).height(length(20.0))
        })
    }
}

#[test]
fn virtualized_content_mounts_only_the_current_window() {
    let app = TestApp::new(Virtualized);

    app.assert_no_text("Virtual row 0");
    app.assert_text("Virtual row 40");
    app.assert_visible("virtual");
}

struct EffectMatrix;

impl Render for EffectMatrix {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let environment = cx.environment();
        let theme = default_theme(environment.clone())
            .resolve(environment.color_scheme)
            .clone();
        let button = |key, label| Button::new(key, label, theme.button());
        let content = Element::column([
            Element::text("Top").height(length(20.0)),
            Element::container([]).height(length(300.0)),
            Element::text("Bottom")
                .keyed("effect-target")
                .height(length(20.0)),
        ]);
        Element::column([
            Input::new("effect-input", "seed", "Effect input", theme.input())
                .label("Effect input")
                .build(),
            Element::row([
                button("focus-effect", "Focus effect")
                    .on_click(cx.event_handler(|_, _, cx| {
                        cx.request_focus("effect-input");
                    }))
                    .build(),
                button("clear-effect", "Clear effect")
                    .on_click(cx.event_handler(|_, _, cx| cx.clear_focus()))
                    .build(),
                button("selection-effect", "Selection effect")
                    .on_click(cx.event_handler(|_, _, cx| {
                        cx.select_text("effect-input", argui_ui::TextSelection::All);
                    }))
                    .build(),
                button("read-effect", "Read clipboard")
                    .on_click(cx.event_handler(|_, _, cx| {
                        cx.write_clipboard(ClipboardRequest::Read { target: None });
                    }))
                    .build(),
            ]),
            Element::row([
                button("theme-effect", "Theme")
                    .on_click(cx.event_handler(|_, _, cx| {
                        cx.set_theme(ThemeRequest {
                            color_scheme: None,
                            primary: Some(Color::srgb(0.8, 0.2, 0.3)),
                        });
                    }))
                    .build(),
                button("offset-effect", "Offset")
                    .on_click(cx.event_handler(|_, _, cx| {
                        cx.scroll(ScrollRequest::offset(
                            "effect-scroll",
                            Point::new(0.0, 120.0),
                        ));
                    }))
                    .build(),
                button("missing-effect", "Missing")
                    .on_click(cx.event_handler(|_, _, cx| {
                        cx.scroll(ScrollRequest::reveal("missing-target"));
                    }))
                    .build(),
            ]),
            ScrollArea::new("effect-scroll", "Effects", 90.0, content).build(&theme),
            Element::row([
                scroll_button(cx, &theme, "start-effect", ScrollAlignment::Start),
                scroll_button(cx, &theme, "center-effect", ScrollAlignment::Center),
                scroll_button(cx, &theme, "end-effect", ScrollAlignment::End),
                scroll_button(cx, &theme, "nearest-effect", ScrollAlignment::Nearest),
            ]),
        ])
        .width(length(640.0))
    }
}

fn scroll_button(
    cx: &mut Context<EffectMatrix>,
    theme: &argui_widgets::WidgetTheme,
    key: &'static str,
    alignment: ScrollAlignment,
) -> Element {
    Button::new(key, key, theme.button())
        .on_click(cx.event_handler(move |_, _, cx| {
            cx.scroll(
                ScrollRequest::rect(
                    "effect-scroll",
                    Rect::new(Point::new(0.0, 300.0), Size::new(20.0, 20.0)),
                )
                .align(alignment, alignment)
                .margin(Sides {
                    left: 2.0,
                    right: 2.0,
                    top: 2.0,
                    bottom: 2.0,
                }),
            );
        }))
        .build()
}

#[test]
fn model_effect_matrix_applies_focus_clipboard_theme_and_scroll_targets() {
    let mut app = TestApp::new(EffectMatrix);
    app.click("read-effect").unwrap();
    app.paste("effect-input", " clipboard").unwrap();
    app.click("focus-effect").unwrap();
    app.assert_focused("effect-input");
    app.click("selection-effect").unwrap();
    app.click("read-effect").unwrap();
    app.click("clear-effect").unwrap();
    app.click("theme-effect").unwrap();
    assert_eq!(app.environment().primary, Color::srgb(0.8, 0.2, 0.3));

    app.click("missing-effect").unwrap();
    app.click("offset-effect").unwrap();
    assert!(app.scroll_offset("effect-scroll").unwrap().y > 0.0);
    for key in [
        "start-effect",
        "center-effect",
        "end-effect",
        "nearest-effect",
    ] {
        app.click(key).unwrap();
    }
    assert!(app.scroll_requests().len() >= 6);
}
