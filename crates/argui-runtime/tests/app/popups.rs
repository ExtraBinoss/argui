// Opt-in: each execution creates its own hidden X11 compositor, including from the coverage gate.
#[cfg(target_os = "linux")]
fn exercise() {
    use argui_core::{Color, Point};
    use argui_platform::{ApplicationConfig, ApplicationIdentity, WindowConfig};
    use argui_runtime::{Context, Render, RuntimeEvent, WindowRuntimeEvent, run_app};
    use argui_ui::{
        Axes, DismissPolicy, Element, FloatingPlacement, FocusPolicy, Interaction, Overflow,
        OverlaySurface, Placement, ScrollConfig, TextEditorSpec, UiEventKind, WindowLayer, length,
    };
    use std::{cell::RefCell, rc::Rc};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    if std::env::var_os("ARGUI_POPUP_TEST_CHILD").is_none() {
        let status = std::process::Command::new(root.join("scripts/linux-hidden-display.sh"))
            .env("ARGUI_TEST_BACKEND", "x11")
            .env("ARGUI_POPUP_TEST_CHILD", "1")
            .current_dir(&root)
            .arg("timeout")
            .arg("30s")
            .arg(std::env::current_exe().unwrap())
            .status()
            .unwrap();
        assert!(status.success(), "hidden native popup test failed");
        return;
    }
    assert_eq!(std::env::var("ARGUI_HIDDEN_DISPLAY").as_deref(), Ok("1"));
    assert!(std::env::var_os("WAYLAND_DISPLAY").is_none());
    let editor = Element::text_editor(TextEditorSpec {
        value: "native".into(),
        placeholder: String::new(),
        multiline: false,
        read_only: false,
        filter: Default::default(),
        text: argui_text::TextStyle {
            color: Color::BLACK,
            ..Default::default()
        },
        placeholder_text: Default::default(),
        selection: Color::srgb(0.4, 0.5, 0.8),
        caret: Default::default(),
    })
    .keyed("editor")
    .width(length(210.0))
    .height(length(42.0))
    .shrink(0.0)
    .background(Color::srgb(0.86, 0.89, 0.94))
    .interaction(Interaction::default().focus_policy(FocusPolicy::TabStop));
    let nested = Element::column([
        Element::text("Nested popup")
            .text_style(argui_text::TextStyle {
                color: Color::BLACK,
                ..Default::default()
            })
            .height(length(35.0)),
        Element::container([])
            .keyed("nested-action")
            .width(length(140.0))
            .height(length(45.0))
            .background(Color::srgb(0.2, 0.6, 0.35))
            .interaction(
                Interaction::default()
                    .gestures(argui_ui::GestureSet::default().tap(argui_ui::TapGesture::default())),
            ),
    ])
    .keyed("nested")
    .width(length(180.0))
    .height(length(140.0))
    .background(Color::srgb(0.9, 0.95, 0.9))
    .interaction(Interaction::blocker())
    .anchored_portal(
        WindowLayer::Popover,
        "nested-anchor",
        FloatingPlacement::new(Placement::RightStart),
    )
    .portal_dismiss(DismissPolicy::OutsidePointer);
    let parent = Element::column([
        editor,
        Element::text("More settings")
            .text_style(argui_text::TextStyle {
                color: Color::BLACK,
                ..Default::default()
            })
            .keyed("nested-anchor")
            .height(length(32.0))
            .shrink(0.0),
        nested,
        Element::text("Scroll inside this native panel")
            .text_style(argui_text::TextStyle {
                color: Color::BLACK,
                ..Default::default()
            })
            .height(length(400.0))
            .shrink(0.0),
    ])
    .keyed("parent")
    .width(length(260.0))
    .height(length(230.0))
    .background(Color::WHITE)
    .interaction(Interaction::blocker())
    .overflow(Axes {
        x: Overflow::Auto,
        y: Overflow::Auto,
    })
    .scroll_config(ScrollConfig::default().propagation(argui_ui::ScrollPropagation::Contain))
    .anchored_portal(
        WindowLayer::Popover,
        "anchor",
        FloatingPlacement::new(Placement::RightStart),
    )
    .portal_surface(OverlaySurface::PreferNative)
    .portal_dismiss(DismissPolicy::OutsidePointer);
    let root_element = Element::column([
        Element::text("Application window")
            .text_style(argui_text::TextStyle {
                color: Color::BLACK,
                ..Default::default()
            })
            .keyed("anchor")
            .width(length(200.0))
            .height(length(35.0)),
        parent,
    ])
    .background(Color::srgb(0.8, 0.84, 0.9))
    .height(argui_ui::percent(1.0));
    struct TestApp(Element);
    impl Render for TestApp {
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            let mut root = self.0.clone();
            for kind in argui_ui::EventType::ALL {
                root = root.on(cx.listener(kind, |_, _, _| {}).capture(true));
            }
            root
        }
    }
    std::fs::create_dir_all(root.join("target/native-popups")).unwrap();
    let events = Rc::new(RefCell::new(Vec::new()));
    let recorded = events.clone();
    let driver = std::thread::spawn(move || {
        std::process::Command::new("python3")
            .arg(root.join("crates/argui-runtime/tests/app/popups/input.py"))
            .current_dir(root)
            .status()
            .unwrap()
    });
    run_app(
        ApplicationConfig::new(
            ApplicationIdentity::development("Native popup test"),
            WindowConfig {
                title: "Argui native popup test".into(),
                width: 220.0,
                height: 180.0,
                ..Default::default()
            },
        ),
        Default::default(),
        TestApp(root_element),
        move |event| {
            if matches!(
                &event,
                RuntimeEvent::Window {
                    event: WindowRuntimeEvent::Platform(argui_platform::PlatformEvent::Focused(
                        true
                    )),
                    ..
                }
            ) {
                std::fs::write(
                    std::path::Path::new(&std::env::var("XDG_RUNTIME_DIR").unwrap())
                        .join("native-ready"),
                    "focused",
                )
                .unwrap();
            }
            if let RuntimeEvent::Window {
                event: WindowRuntimeEvent::Ui(event),
                ..
            } = event
            {
                recorded
                    .borrow_mut()
                    .push((event.target_key().map(str::to_owned), event.kind));
            }
        },
    )
    .unwrap();
    let driven = driver.join().unwrap();
    let events = events.borrow();
    assert!(driven.success(), "native input driver failed: {events:?}");
    assert!(
        events
            .iter()
            .any(|(key, kind)| key.as_deref() == Some("editor")
                && matches!(kind, UiEventKind::TextChanged(value) if value == "native ergonomics")),
        "editor input was not routed into the native popup: {events:?}"
    );
    assert!(
        events
            .iter()
            .any(|(key, kind)| key.as_deref() == Some("nested-action")
                && matches!(kind, UiEventKind::Click(_)))
    );
    assert!(
        events
            .iter()
            .any(|(key, kind)| key.as_deref() == Some("parent")
                && matches!(kind, UiEventKind::Scrolled { offset, .. } if offset.y > 0.0))
    );
    assert!(
        !events
            .iter()
            .any(|(key, kind)| key.as_deref() == Some("parent")
                && matches!(kind, UiEventKind::PointerOutside(_))),
        "clicks in descendants must remain inside the parent"
    );
    assert!(events.iter().any(|(_, kind)| matches!(kind, UiEventKind::KeyInput(input) if input.key == argui_core::Key::ArrowLeft && input.modifiers.control && input.modifiers.shift)));
    assert!(events.iter().any(|(_, kind)| matches!(kind, UiEventKind::Pointer(event) if event.position.x > 220.0 && event.position != Point::default())));
    for key in ["parent", "nested"] {
        assert!(
            events
                .iter()
                .any(|(target, kind)| target.as_deref() == Some(key)
                    && matches!(kind, UiEventKind::DismissRequested))
        );
    }
    eprintln!(
        "native popups: outside-window geometry, nested ownership, input, selection shortcut, clicks and scroll passed"
    );
}
fn main() {
    let enabled = std::env::var_os("ARGUI_NATIVE_TESTS").is_some() && cfg!(target_os = "linux");
    if std::env::args().any(|arg| arg == "--list") {
        if enabled && !std::env::args().any(|arg| arg == "--ignored") {
            println!("native_popups: test");
        }
        return;
    }
    #[cfg(target_os = "linux")]
    if enabled {
        exercise();
    }
}
