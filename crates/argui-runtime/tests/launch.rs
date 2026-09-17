#[cfg(all(feature = "tasks", feature = "webview", target_os = "linux"))]
#[path = "app/gtk.rs"]
mod gtk_input;
// Opt-in OS integration: ARGUI_NATIVE_TESTS=1 cargo nextest run -p argui-runtime --all-features --test launch
#[cfg(all(feature = "tasks", not(target_arch = "wasm32")))]
mod native {
    use argui_platform::{
        ApplicationConfig, ApplicationId, ApplicationIdentity, CloseBehavior, IconSet,
        PlatformEvent, WindowConfig, WindowKey, WindowSpec,
    };
    use argui_runtime::{
        AppCommand, AppEvent, AppModel, AppUpdate, Context, Entity, LayoutSnapshot, ModelRuntime,
        MountId, Render, RuntimeEvent, SingleWindowModel, WindowEnvironment,
        tasks::{TaskHandle, sleep},
    };
    use argui_ui::Element;
    use std::{cell::RefCell, collections::HashMap, rc::Rc, time::Duration};

    struct Data {
        phase: usize,
        scrolled: bool,
        edited: bool,
        themed: bool,
        safe_area: bool,
        keys: Vec<argui_core::KeyInput>,
        pointer: Vec<argui_core::PointerEvent>,
        wheel: Vec<argui_core::ScrollDelta>,
    }
    type Visits = Rc<RefCell<Vec<(bool, usize, MountId)>>>;
    struct Panel {
        data: Entity<Data>,
        main: bool,
        issued: usize,
        layout_rebuilt: bool,
        virtual_resets: usize,
        list: argui_ui::VirtualList,
        timer: Option<TaskHandle>,
        visits: Visits,
        pending: Rc<RefCell<Vec<TaskHandle>>>,
    }
    impl Panel {
        fn schedule(&mut self) {
            self.timer = Some(self.data.update(|_, cx| {
                cx.spawn(
                    async {
                        sleep(Duration::from_millis(120)).await;
                    },
                    |data, result, cx| {
                        result.unwrap();
                        data.phase += 1;
                        cx.notify();
                    },
                )
                .unwrap()
            }));
        }
    }
    impl Render for Panel {
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            let phase = cx.read(&self.data, |data| data.phase);
            if self.virtual_resets > 0 {
                self.virtual_resets -= 1;
                self.list = argui_ui::VirtualList::variable(3, 120.0, 240.0);
            }
            if cx.environment().safe_area_insets.top == 11.0 {
                self.data.update(|data, _| data.safe_area = true);
            }
            if cx.environment().primary == argui_core::Color::BLACK {
                self.data.update(|data, _| data.themed = true);
            }
            self.visits
                .borrow_mut()
                .push((self.main, phase, cx.mount_id().unwrap()));
            if self.main && phase > self.issued {
                self.issued = phase;
                exercise_scroll(phase, cx);
                eprintln!("native lifecycle phase {phase}");
            }
            if self.main && phase > 0 && self.pending.borrow().is_empty() {
                self.pending.borrow_mut().push(
                    cx.spawn(
                        async {
                            sleep(Duration::from_secs(30)).await;
                        },
                        |_, _, _| panic!("closed view callback"),
                    )
                    .unwrap(),
                );
                self.pending.borrow_mut().push(self.data.update(|_, cx| {
                    cx.spawn(
                        async {
                            sleep(Duration::from_secs(30)).await;
                        },
                        |_, _, _| panic!("exited application callback"),
                    )
                    .unwrap()
                }));
            }
            native_view(phase, cx, &self.list)
        }
        fn layout_changed(&mut self, layout: &LayoutSnapshot, cx: &mut Context<Self>) {
            if self.main
                && layout
                    .bounds("native-first")
                    .is_some_and(|bounds| bounds.origin.y < 0.0)
            {
                self.data.update(|data, _| data.scrolled = true);
            }
            if self.main && self.timer.is_none() {
                eprintln!("native lifecycle initial layout");
                self.schedule();
            }
            if self.main && !self.layout_rebuilt {
                self.layout_rebuilt = true;
                self.virtual_resets = 2;
                cx.scroll(argui_ui::ScrollRequest::reveal("native-editor"));
                cx.notify();
            }
        }
    }
    fn native_view(phase: usize, cx: &mut Context<Panel>, list: &argui_ui::VirtualList) -> Element {
        use argui_ui::{FocusPolicy, Interaction, TextEditorSpec, length};
        let editor = Element::text_editor(TextEditorSpec {
            value: format!("phase {phase}"),
            placeholder: String::new(),
            multiline: false,
            read_only: false,
            filter: Default::default(),
            text: Default::default(),
            placeholder_text: Default::default(),
            selection: argui_core::Color::WHITE,
            caret: Default::default(),
        })
        .on(cx.listener(argui_ui::EventType::Input, |panel, event, _| {
            if let argui_ui::UiEventKind::TextChanged(value) = &event.kind
                && value == "native replacement"
            {
                panel.data.update(|data, _| data.edited = true);
            }
        }))
        .keyed("native-editor")
        .height(length(40.0))
        .interaction(Interaction::default().focus_policy(FocusPolicy::TabStop));
        list.build("native-scroll", 0.0, |index| match index {
            0 => Element::text("First")
                .keyed("native-first")
                .height(length(500.0)),
            1 => editor.clone(),
            _ => Element::text("Last")
                .keyed("native-last")
                .height(length(500.0)),
        })
        .width(length(400.0))
    }
    fn exercise_scroll(phase: usize, cx: &mut Context<Panel>) {
        use argui_core::{Point, Rect, Size};
        use argui_ui::{ScrollAlignment, ScrollRequest};
        let request = match phase {
            1 => ScrollRequest::offset("native-scroll", Point::new(0.0, 80.0)),
            2 => {
                cx.request_focus("native-editor");
                ScrollRequest::rect(
                    "native-scroll",
                    Rect::new(Point::new(0.0, 400.0), Size::new(100.0, 80.0)),
                )
                .align(ScrollAlignment::Start, ScrollAlignment::Center)
            }
            3 => ScrollRequest::reveal("native-last")
                .align(ScrollAlignment::End, ScrollAlignment::Nearest),
            4 => ScrollRequest::reveal("native-first"),
            5 => ScrollRequest::reveal("missing"),
            6 => ScrollRequest::offset("missing", Point::new(0.0, 10.0)),
            _ => ScrollRequest::reveal("native-editor"),
        };
        cx.scroll(request);
    }
    struct App {
        issued: usize,
        edit_issued: bool,
        theme_issued: bool,
        timer: Option<TaskHandle>,
        data: Entity<Data>,
        windows: RefCell<HashMap<WindowKey, SingleWindowModel<Panel>>>,
        visits: Visits,
        pending: Rc<RefCell<Vec<TaskHandle>>>,
        closed: Rc<RefCell<Vec<WindowKey>>>,
        #[cfg(all(feature = "webview", target_os = "linux"))]
        animation: crate::gtk_input::AnimationProbe,
    }
    impl AppModel for App {
        fn view(&self, key: &WindowKey, environment: WindowEnvironment) -> Option<Element> {
            let mut windows = self.windows.borrow_mut();
            let app = windows.entry(key.clone()).or_insert_with(|| {
                SingleWindowModel::from_entity(self.data.runtime().entity(Panel {
                    data: self.data.clone(),
                    main: key == &WindowKey::main(),
                    issued: 0,
                    layout_rebuilt: false,
                    virtual_resets: 0,
                    list: argui_ui::VirtualList::variable(3, 120.0, 240.0),
                    timer: None,
                    visits: self.visits.clone(),
                    pending: self.pending.clone(),
                }))
                .unwrap()
                .window_key(key.clone())
            });
            app.view(key, environment)
        }
        fn tasks_ready(&mut self, key: &WindowKey) -> AppUpdate {
            let phase = self.data.read(|data| data.phase);
            if key != &WindowKey::main() || phase == self.issued {
                return AppUpdate::none();
            }
            self.issued = phase;
            eprintln!("native lifecycle command {phase}");
            if phase < 6 {
                self.timer = Some(self.data.update(|_, cx| {
                    cx.spawn(
                        async {
                            sleep(Duration::from_secs(1)).await;
                        },
                        |data, result, cx| {
                            result.unwrap();
                            data.phase += 1;
                            cx.notify();
                        },
                    )
                    .unwrap()
                }));
            }
            #[cfg(all(feature = "webview", target_os = "linux"))]
            if phase == 5 {
                crate::gtk_input::send();
            }
            let auxiliary = WindowKey::new("auxiliary");
            let command = match phase {
                1 | 5 => AppCommand::OpenWindow(WindowSpec::new(
                    auxiliary,
                    WindowConfig {
                        title: "Argui lifecycle auxiliary".into(),
                        close_behavior: CloseBehavior::CloseWindow,
                        ..Default::default()
                    },
                )),
                2 => AppCommand::HideWindow(auxiliary),
                3 => AppCommand::ShowWindow(auxiliary),
                4 => AppCommand::CloseWindow(auxiliary),
                6 => AppCommand::MinimizeWindow(auxiliary),
                7 => AppCommand::FocusWindow(auxiliary),
                8 => AppCommand::Quit,
                _ => panic!("unexpected phase"),
            };
            let update = AppUpdate::none().command(command);
            match phase {
                2 => update
                    .command(AppCommand::SetWindowTitle {
                        window: WindowKey::main(),
                        title: "Argui updated lifecycle check".into(),
                    })
                    .command(AppCommand::SetSafeAreaInsets {
                        window: WindowKey::main(),
                        insets: Some(argui_core::Insets::new(11.0, 1.0, 2.0, 3.0)),
                    }),
                3 | 4 => update.command(AppCommand::SetWindowMaximized {
                    window: WindowKey::main(),
                    maximized: phase == 3,
                }),
                6 => update
                    .command(AppCommand::FocusWindow(WindowKey::new("auxiliary")))
                    .command(AppCommand::Quit),
                _ => update,
            }
        }
        fn take_ui_commands(&mut self, key: &WindowKey) -> Vec<argui_ui::UiCommand> {
            if key == &WindowKey::main() && self.issued == 3 && !self.edit_issued {
                self.edit_issued = true;
                vec![argui_ui::UiCommand::ReplaceText {
                    target: "native-editor".into(),
                    value: "native replacement".into(),
                }]
            } else {
                Vec::new()
            }
        }
        // Assert rendered effects during the main window maximize/presentation phase.
        fn take_theme_request(&mut self, key: &WindowKey) -> Option<argui_runtime::ThemeRequest> {
            if key == &WindowKey::main() && self.issued == 3 && !self.theme_issued {
                self.theme_issued = true;
                Some(argui_runtime::ThemeRequest {
                    primary: Some(argui_core::Color::BLACK),
                    ..Default::default()
                })
            } else {
                None
            }
        }
        fn take_scroll_request(&mut self, key: &WindowKey) -> Option<argui_ui::ScrollRequest> {
            self.windows
                .borrow_mut()
                .get_mut(key)?
                .take_scroll_request(key)
        }
        fn take_focus_request(&mut self, key: &WindowKey) -> Option<argui_ui::FocusRequest> {
            self.windows
                .borrow_mut()
                .get_mut(key)?
                .take_focus_request(key)
        }
        fn event_router(&self, key: &WindowKey) -> Option<argui_runtime::AnyEntity> {
            self.windows.borrow().get(key)?.event_router(key)
        }
        fn update(&mut self, event: &AppEvent) -> AppUpdate {
            if let AppEvent::Window { event, .. } = event {
                self.data.update(|data, _| match event {
                    PlatformEvent::Keyboard(input) => data.keys.push(input.clone()),
                    PlatformEvent::Pointer(input) => data.pointer.push(*input),
                    PlatformEvent::PointerScrolled(delta) => data.wheel.push(*delta),
                    _ => (),
                });
            }
            if let AppEvent::Window {
                window,
                event: PlatformEvent::Closed,
            } = event
            {
                self.windows.borrow_mut().remove(window);
                self.closed.borrow_mut().push(window.clone());
            }
            AppUpdate::none()
        }
        fn layout_changed(&mut self, key: &WindowKey, layout: &LayoutSnapshot) -> AppUpdate {
            self.windows
                .borrow_mut()
                .get_mut(key)
                .unwrap()
                .layout_changed(key, layout)
        }
        fn animation_frame(&mut self, _key: &WindowKey, _: argui_animation::Frame) -> AppUpdate {
            #[cfg(all(feature = "webview", target_os = "linux"))]
            if _key == &WindowKey::main() {
                self.animation.frame();
            }
            AppUpdate::none()
        }
        fn wants_animation_frame(&self, key: &WindowKey) -> bool {
            #[cfg(all(feature = "webview", target_os = "linux"))]
            {
                key == &WindowKey::main() && self.animation.wants_frame()
            }
            #[cfg(not(all(feature = "webview", target_os = "linux")))]
            {
                let _ = key;
                false
            }
        }
    }
    pub fn run() {
        let runtime = ModelRuntime::default();
        let data = runtime.entity(Data {
            phase: 0,
            scrolled: false,
            edited: false,
            themed: false,
            safe_area: false,
            keys: Vec::new(),
            pointer: Vec::new(),
            wheel: Vec::new(),
        });
        let service = runtime
            .register_service(String::from("application service"))
            .unwrap();
        let visits = Visits::default();
        let pending = Rc::new(RefCell::new(Vec::new()));
        let closed = Rc::new(RefCell::new(Vec::new()));
        #[cfg(all(feature = "webview", target_os = "linux"))]
        let animation = crate::gtk_input::AnimationProbe::default();
        let errors = Rc::new(RefCell::new(Vec::new()));
        let captured = errors.clone();
        let edited = data.clone();
        let config = ApplicationConfig::new(
            ApplicationIdentity::new(
                ApplicationId::new("dev.argui.lifecycle").unwrap(),
                "Lifecycle integration",
                IconSet::default(),
            ),
            WindowConfig {
                title: "Argui lifecycle check".into(),
                desktop_backdrop: Some(argui_core::BackdropMaterial::Sidebar),
                ..Default::default()
            },
        );
        argui_runtime::run_application_with_text_engine(
            config,
            Default::default(),
            argui_text::TextEngine::from_embedded_fonts(
                [
                    include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf")
                        .as_slice(),
                ],
                "Noto Sans",
                "Noto Sans",
                "Noto Sans",
            ),
            App {
                issued: 0,
                edit_issued: false,
                theme_issued: false,
                timer: None,
                data: data.clone(),
                windows: RefCell::default(),
                visits: visits.clone(),
                pending: pending.clone(),
                closed: closed.clone(),
                #[cfg(all(feature = "webview", target_os = "linux"))]
                animation: animation.clone(),
            },
            move |event| match event {
                RuntimeEvent::Window {
                    event: argui_runtime::WindowRuntimeEvent::Ui(event),
                    ..
                } => {
                    if let argui_ui::UiEventKind::TextChanged(value) = event.kind
                        && value == "native replacement"
                    {
                        edited.update(|data, _| data.edited = true);
                    }
                }
                RuntimeEvent::RendererFailed(error)
                | RuntimeEvent::LayoutFailed(error)
                | RuntimeEvent::CommandFailed(error) => captured.borrow_mut().push(error),
                RuntimeEvent::Window {
                    event:
                        argui_runtime::WindowRuntimeEvent::RendererFailed(error)
                        | argui_runtime::WindowRuntimeEvent::LayoutFailed(error),
                    ..
                } => captured.borrow_mut().push(error),
                _ => {}
            },
        )
        .unwrap();
        assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
        assert_eq!(data.read(|data| data.phase), 6);
        #[cfg(all(feature = "webview", target_os = "linux"))]
        animation.assert_smooth();
        assert!(
            data.read(|data| data.edited),
            "native edit command was delivered"
        );
        assert!(
            data.read(|data| data.themed),
            "theme changes reach the rendered view"
        );
        assert!(
            data.read(|data| data.safe_area),
            "safe-area changes reach the rendered view"
        );
        assert!(
            data.read(|data| data.scrolled),
            "native scroll requests must change the layout"
        );
        #[cfg(all(feature = "webview", target_os = "linux"))]
        data.read(|data| {
            crate::gtk_input::assert_pointer(&data.pointer, &data.wheel);
            let expected: Vec<_> = crate::gtk_input::keys()
                .into_iter()
                .flat_map(|(_, key)| {
                    [
                        argui_core::KeyState::Pressed,
                        argui_core::KeyState::Released,
                    ]
                    .map(|state| (key.clone(), state))
                })
                .collect();
            assert_eq!(
                data.keys
                    .iter()
                    .map(|input| (input.key.clone(), input.state))
                    .collect::<Vec<_>>(),
                expected
            );
        });
        let visits = visits.borrow();
        let original = visits
            .iter()
            .find(|(main, phase, _)| !main && *phase == 1)
            .unwrap()
            .2;
        let shown = visits
            .iter()
            .find(|(main, phase, _)| !main && *phase == 3)
            .unwrap()
            .2;
        let reopened = visits
            .iter()
            .find(|(main, phase, _)| !main && *phase == 5)
            .unwrap()
            .2;
        assert_eq!(original, shown);
        assert_ne!(original, reopened);
        assert_eq!(closed.borrow().len(), 3);
        assert_eq!(pending.borrow().len(), 2);
        assert!(
            pending
                .borrow()
                .iter()
                .all(|task| task.cancellation_token().is_cancelled())
        );
        assert_eq!(data.resources().resource_count(), 0);
        assert!(runtime.service::<String>().is_some());
        drop(service);
        assert!(runtime.service::<String>().is_none());
        eprintln!(
            "native lifecycle passed: two windows, hide/show, shared data wake, close/reopen, quit with two pending tasks; {} renders",
            visits.len()
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod model_free {
    use argui_platform::WindowConfig;

    pub fn run() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        if std::env::var_os("ARGUI_MODEL_FREE_TEST_CHILD").is_none() {
            let success = root
                .join("target/native-launch")
                .join(format!("success-{}", std::process::id()));
            std::fs::create_dir_all(success.parent().unwrap()).unwrap();
            let _ = std::fs::remove_file(&success);
            let status = std::process::Command::new(root.join("scripts/linux-hidden-display.sh"))
                .env("ARGUI_TEST_BACKEND", "x11")
                .env("ARGUI_MODEL_FREE_TEST_CHILD", "1")
                .env("ARGUI_MODEL_FREE_TEST_SUCCESS", &success)
                .current_dir(&root)
                .arg("timeout")
                .arg("20s")
                .arg(std::env::current_exe().unwrap())
                .status()
                .unwrap();
            assert!(
                status.success() || success.exists(),
                "hidden model-free launch test failed"
            );
            let _ = std::fs::remove_file(success);
            return;
        }
        let driver = std::thread::spawn(move || {
            std::process::Command::new("python3")
                .arg(root.join("crates/argui-runtime/tests/launch/close.py"))
                .current_dir(root)
                .status()
                .unwrap()
        });
        argui_runtime::run(
            WindowConfig {
                title: "Argui model-free launch test".into(),
                ..Default::default()
            },
            Default::default(),
            |_| {},
        )
        .unwrap();
        assert!(driver.join().unwrap().success());
        std::fs::write(
            std::env::var_os("ARGUI_MODEL_FREE_TEST_SUCCESS").unwrap(),
            "passed",
        )
        .unwrap();
    }
}

fn main() {
    let enabled = std::env::var_os("ARGUI_NATIVE_TESTS").is_some();
    if std::env::args().any(|argument| argument == "--list") {
        if enabled
            && cfg!(not(target_arch = "wasm32"))
            && !std::env::args().any(|arg| arg == "--ignored")
        {
            println!("native_lifecycle: test");
        }
        return;
    }
    #[cfg(all(feature = "tasks", not(target_arch = "wasm32")))]
    if enabled {
        if std::env::var_os("ARGUI_MODEL_FREE_TEST_CHILD").is_some() {
            model_free::run();
        } else {
            native::run();
            model_free::run();
        }
    }
}
