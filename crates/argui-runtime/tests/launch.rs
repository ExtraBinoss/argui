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
    }
    type Visits = Rc<RefCell<Vec<(bool, usize, MountId)>>>;
    struct Panel {
        data: Entity<Data>,
        main: bool,
        issued: usize,
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
            self.visits
                .borrow_mut()
                .push((self.main, phase, cx.mount_id().unwrap()));
            if self.main && phase > self.issued {
                self.issued = phase;
                eprintln!("native lifecycle phase {phase}");
                if phase < 8 {
                    self.schedule();
                }
                if phase == 7 {
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
            }
            Element::text(format!("phase {phase}"))
        }
        fn layout_changed(&mut self, _: &LayoutSnapshot, _: &mut Context<Self>) {
            if self.main && self.timer.is_none() {
                eprintln!("native lifecycle initial layout");
                self.schedule();
            }
        }
    }
    struct App {
        issued: usize,
        data: Entity<Data>,
        windows: RefCell<HashMap<WindowKey, SingleWindowModel<Panel>>>,
        visits: Visits,
        pending: Rc<RefCell<Vec<TaskHandle>>>,
        closed: Rc<RefCell<Vec<WindowKey>>>,
    }
    impl AppModel for App {
        fn view(&self, key: &WindowKey, environment: WindowEnvironment) -> Option<Element> {
            let mut windows = self.windows.borrow_mut();
            let app = windows.entry(key.clone()).or_insert_with(|| {
                SingleWindowModel::from_entity(self.data.runtime().entity(Panel {
                    data: self.data.clone(),
                    main: key == &WindowKey::main(),
                    issued: 0,
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
            AppUpdate::none().command(command)
        }
        fn event_router(&self, key: &WindowKey) -> Option<argui_runtime::AnyEntity> {
            self.windows.borrow().get(key)?.event_router(key)
        }
        fn update(&mut self, event: &AppEvent) -> AppUpdate {
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
    }
    pub fn run() {
        let runtime = ModelRuntime::default();
        let data = runtime.entity(Data { phase: 0 });
        let service = runtime
            .register_service(String::from("application service"))
            .unwrap();
        let visits = Visits::default();
        let pending = Rc::new(RefCell::new(Vec::new()));
        let closed = Rc::new(RefCell::new(Vec::new()));
        let errors = Rc::new(RefCell::new(Vec::new()));
        let captured = errors.clone();
        let config = ApplicationConfig::new(
            ApplicationIdentity::new(
                ApplicationId::new("dev.argui.lifecycle").unwrap(),
                "Lifecycle integration",
                IconSet::default(),
            ),
            WindowConfig {
                title: "Argui lifecycle check".into(),
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
                data: data.clone(),
                windows: RefCell::default(),
                visits: visits.clone(),
                pending: pending.clone(),
                closed: closed.clone(),
            },
            move |event| match event {
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
        assert_eq!(data.read(|data| data.phase), 8);
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
        native::run();
    }
}
