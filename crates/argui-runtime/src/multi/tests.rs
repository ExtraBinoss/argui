use std::{cell::RefCell, rc::Rc};

use super::{
    SharedCallback, SharedModel, SharedUpdates, WindowModel, scoped_callback, tray_command,
};
use crate::{
    AppCommand, AppEvent, AppModel, AppUpdate, RuntimeEvent, ViewUpdate, WindowInvalidation,
};
use argui_platform::{PlatformEvent, TrayAction, WindowKey};
use argui_ui::Element;

struct TestModel;

impl AppModel for TestModel {
    fn view(&self, _window: &WindowKey, _environment: crate::WindowEnvironment) -> Option<Element> {
        Some(Element::container(Vec::<Element>::new()))
    }

    fn update(&mut self, _event: &AppEvent) -> AppUpdate {
        AppUpdate::none()
    }
}

fn window_model(key: &WindowKey) -> (WindowModel, SharedUpdates) {
    let model: SharedModel = Rc::new(RefCell::new(Box::new(TestModel)));
    let pending = Rc::new(RefCell::new(Vec::new()));
    (
        WindowModel {
            key: key.clone(),
            model,
            pending: Rc::clone(&pending),
        },
        pending,
    )
}

#[test]
fn built_in_tray_actions_lower_to_runtime_commands() {
    let key = WindowKey::new("settings");
    assert_eq!(
        tray_command(TrayAction::ToggleWindow(key.clone())),
        Some(AppCommand::ToggleWindow(key))
    );
    assert_eq!(tray_command(TrayAction::Quit), Some(AppCommand::Quit));
    assert_eq!(tray_command(TrayAction::Custom("sync".into())), None);
}

#[test]
fn window_updates_extract_the_strongest_local_invalidation() {
    let main = WindowKey::main();
    let other = WindowKey::new("other");
    let (window, pending) = window_model(&main);

    assert_eq!(window.record(AppUpdate::none()), ViewUpdate::None);
    assert!(pending.borrow().is_empty());

    let local = AppUpdate {
        windows: vec![
            WindowInvalidation {
                window: main.clone(),
                update: ViewUpdate::None,
            },
            WindowInvalidation {
                window: main.clone(),
                update: ViewUpdate::Paint,
            },
            WindowInvalidation {
                window: main.clone(),
                update: ViewUpdate::None,
            },
            WindowInvalidation {
                window: main.clone(),
                update: ViewUpdate::Rebuild,
            },
            WindowInvalidation {
                window: main.clone(),
                update: ViewUpdate::Paint,
            },
        ],
        ..AppUpdate::none()
    };
    assert_eq!(window.record(local), ViewUpdate::Rebuild);
    assert!(pending.borrow().is_empty());

    let remote = AppUpdate::none().window(other, ViewUpdate::Paint);
    assert_eq!(window.record(remote), ViewUpdate::None);
    assert_eq!(pending.borrow().len(), 1);
}

#[test]
fn window_updates_forward_commands_and_tray_changes() {
    let main = WindowKey::main();
    let (window, pending) = window_model(&main);
    assert_eq!(
        window.record(AppUpdate::none().command(AppCommand::Quit)),
        ViewUpdate::None
    );
    assert_eq!(
        window.record(AppUpdate::none().tray_changed()),
        ViewUpdate::None
    );
    assert_eq!(pending.borrow().len(), 2);
}

#[test]
fn scoped_callbacks_route_only_platform_events_through_the_model() {
    let key = WindowKey::main();
    let model: SharedModel = Rc::new(RefCell::new(Box::new(TestModel)));
    let pending: SharedUpdates = Rc::new(RefCell::new(Vec::new()));
    let received = Rc::new(RefCell::new(Vec::new()));
    let callback: SharedCallback = Rc::new(RefCell::new(Box::new({
        let received = Rc::clone(&received);
        move |event| received.borrow_mut().push(event)
    })));
    let mut scoped = scoped_callback(key, model, Rc::clone(&pending), callback);

    scoped(RuntimeEvent::RendererReady);
    assert!(pending.borrow().is_empty());
    scoped(RuntimeEvent::Platform(PlatformEvent::Pointer(
        argui_platform::PointerEvent::mouse(
            argui_platform::PointerPhase::Entered,
            argui_core::Point::default(),
        ),
    )));
    assert_eq!(pending.borrow().len(), 1);
    assert_eq!(received.borrow().len(), 2);
}
