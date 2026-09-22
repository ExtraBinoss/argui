//! Native GTK failure propagation through the real multi-window event loop.

#[cfg(target_os = "linux")]
mod linux {
    use argui_core::Size;
    use argui_platform::{
        ApplicationConfig, ApplicationId, ApplicationIdentity, IconSet, WindowConfig, WindowKey,
    };
    use argui_runtime::{
        AppEvent, AppModel, AppUpdate, RuntimeError, RuntimeEvent, WindowEnvironment,
    };
    use argui_ui::{
        CustomElement, CustomLayoutContext, CustomMeasurement, CustomPaintContext, Element,
    };
    use std::{cell::RefCell, rc::Rc};

    #[derive(Debug)]
    struct FailedLayout;

    impl CustomElement for FailedLayout {
        type State = ();

        fn create_state(&self) {}

        fn layout_revision(&self) -> u64 {
            0
        }

        fn paint_revision(&self) -> u64 {
            0
        }

        fn prepare(&self, _: &mut (), _: Size) {}

        fn layout(
            &self,
            _: &mut (),
            _: &mut dyn CustomLayoutContext,
        ) -> Result<CustomMeasurement, String> {
            Err("intentional native layout failure".into())
        }

        fn paint(&self, _: &mut (), _: &mut CustomPaintContext<'_>) {}
    }

    struct BrokenApp;

    impl AppModel for BrokenApp {
        fn view(&self, _: &WindowKey, _: WindowEnvironment) -> Option<Element> {
            Some(Element::custom(FailedLayout))
        }

        fn update(&mut self, _: &AppEvent) -> AppUpdate {
            AppUpdate::none()
        }
    }

    /// Confirms a custom layout error exits GTK before renderer setup and reaches the caller.
    pub(super) fn run() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let captured = Rc::clone(&events);
        let config = ApplicationConfig::new(
            ApplicationIdentity::new(
                ApplicationId::new("dev.argui.layoutfailure").unwrap(),
                "Layout failure integration",
                IconSet::default(),
            ),
            WindowConfig {
                title: "Argui layout failure".into(),
                ..Default::default()
            },
        );
        let result =
            argui_runtime::run_application(config, Default::default(), BrokenApp, move |event| {
                captured.borrow_mut().push(event);
            });
        assert!(matches!(result, Err(RuntimeError::Layout(_))), "{result:?}");
        assert!(events.borrow().iter().any(|event| matches!(event,
            RuntimeEvent::Window { event: argui_runtime::WindowRuntimeEvent::LayoutFailed(message), .. }
                if message.contains("intentional native layout failure")
        )));
    }
}

fn main() {
    let enabled = std::env::var_os("ARGUI_NATIVE_TESTS").is_some() && cfg!(target_os = "linux");
    if std::env::args().any(|argument| argument == "--list") {
        if enabled && !std::env::args().any(|argument| argument == "--ignored") {
            println!("native_layout_failure: test");
        }
        return;
    }
    #[cfg(target_os = "linux")]
    if enabled {
        linux::run();
    }
}
