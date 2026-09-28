#![cfg(target_os = "linux")]

use argui_platform::{WindowConfig, prepare_window_presentation};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    platform::x11::EventLoopBuilderExtX11,
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::WindowId,
};
use x11rb::protocol::xproto::{ConnectionExt as _, Gravity};

#[test]
#[ignore = "requires the private X11 display"]
fn transparent_clients_preserve_top_left_backing_without_changing_opaque_clients() {
    assert_eq!(std::env::var("ARGUI_HIDDEN_DISPLAY").as_deref(), Ok("1"));
    struct Check;
    impl ApplicationHandler for Check {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            let window = event_loop
                .create_window(
                    WindowConfig::default()
                        .into_attributes()
                        .with_visible(false),
                )
                .unwrap();
            let xid = match window.window_handle().unwrap().as_raw() {
                RawWindowHandle::Xlib(handle) => handle.window as u32,
                RawWindowHandle::Xcb(handle) => handle.window.get(),
                _ => panic!("X11 window expected"),
            };
            let (connection, _) = x11rb::connect(None).unwrap();
            let gravity = || {
                connection
                    .get_window_attributes(xid)
                    .unwrap()
                    .reply()
                    .unwrap()
                    .bit_gravity
            };
            let original = gravity();
            prepare_window_presentation(&window, false).unwrap();
            assert_eq!(gravity(), original);
            prepare_window_presentation(&window, true).unwrap();
            assert_eq!(gravity(), Gravity::NORTH_WEST);
            prepare_window_presentation(&window, true).unwrap();
            assert_eq!(gravity(), Gravity::NORTH_WEST);
            event_loop.exit();
        }
        fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, _: WindowEvent) {}
    }
    let mut builder = EventLoop::builder();
    builder.with_x11().with_any_thread(true);
    builder.build().unwrap().run_app(&mut Check).unwrap();
}
