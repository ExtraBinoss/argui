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
            prepare_window_presentation(&window, &WindowConfig::default()).unwrap();
            assert_eq!(gravity(), original);
            let transparent = WindowConfig {
                transparent: true,
                ..Default::default()
            };
            prepare_window_presentation(&window, &transparent).unwrap();
            assert_eq!(gravity(), Gravity::NORTH_WEST);
            prepare_window_presentation(&window, &transparent).unwrap();
            assert_eq!(gravity(), Gravity::NORTH_WEST);
            event_loop.exit();
        }
        fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, _: WindowEvent) {}
    }
    let mut builder = EventLoop::builder();
    builder.with_x11().with_any_thread(true);
    builder.build().unwrap().run_app(&mut Check).unwrap();
}

/// Exercises presentation properties on an actual unmapped private X11 window.
fn presentation_check(config: WindowConfig, check: fn(&winit::window::Window, u32)) {
    assert_eq!(std::env::var("ARGUI_HIDDEN_DISPLAY").as_deref(), Ok("1"));
    struct Check {
        config: WindowConfig,
        check: fn(&winit::window::Window, u32),
    }
    impl ApplicationHandler for Check {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            let window = event_loop
                .create_window(self.config.clone().into_attributes().with_visible(false))
                .unwrap();
            let xid = match window.window_handle().unwrap().as_raw() {
                RawWindowHandle::Xlib(handle) => handle.window as u32,
                RawWindowHandle::Xcb(handle) => handle.window.get(),
                _ => panic!("X11 window expected"),
            };
            prepare_window_presentation(&window, &self.config).unwrap();
            (self.check)(&window, xid);
            event_loop.exit();
        }
        fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, _: WindowEvent) {}
    }
    let mut builder = EventLoop::builder();
    builder.with_x11().with_any_thread(true);
    builder
        .build()
        .unwrap()
        .run_app(&mut Check { config, check })
        .unwrap();
}

/// Reads the complete 32-bit property from a real X11 client.
fn property(xid: u32, name: &[u8]) -> Vec<u32> {
    use x11rb::protocol::xproto::AtomEnum;
    let (connection, _) = x11rb::connect(None).unwrap();
    let atom = connection
        .intern_atom(false, name)
        .unwrap()
        .reply()
        .unwrap()
        .atom;
    connection
        .get_property(false, xid, atom, AtomEnum::ANY, 0, 32)
        .unwrap()
        .reply()
        .unwrap()
        .value32()
        .map(|values| values.collect())
        .unwrap_or_default()
}

#[test]
#[ignore = "requires the private X11 display"]
fn passive_tools_do_not_request_activation_but_remain_focusable() {
    presentation_check(
        WindowConfig {
            focus_on_launch: false,
            skip_taskbar: true,
            ..Default::default()
        },
        |_, xid| {
            use x11rb::protocol::xproto::InputFocus;
            let (connection, _) = x11rb::connect(None).unwrap();
            let skip = connection
                .intern_atom(false, b"_NET_WM_STATE_SKIP_TASKBAR")
                .unwrap()
                .reply()
                .unwrap()
                .atom;
            assert_eq!(property(xid, b"_NET_WM_USER_TIME"), vec![0]);
            assert!(property(xid, b"_NET_WM_STATE").contains(&skip));
            connection.map_window(xid).unwrap().check().unwrap();
            connection
                .set_input_focus(InputFocus::PARENT, xid, x11rb::CURRENT_TIME)
                .unwrap()
                .check()
                .unwrap();
            assert_eq!(
                connection.get_input_focus().unwrap().reply().unwrap().focus,
                xid
            );
        },
    );
}

#[test]
#[ignore = "requires the private X11 display"]
fn remapped_tools_restore_passive_hints_without_losing_stacking_states() {
    presentation_check(
        WindowConfig {
            focus_on_launch: false,
            skip_taskbar: true,
            ..Default::default()
        },
        |window, xid| {
            use x11rb::{
                protocol::xproto::{AtomEnum, PropMode},
                wrapper::ConnectionExt as _,
            };
            let (connection, _) = x11rb::connect(None).unwrap();
            let atom = |name: &[u8]| {
                connection
                    .intern_atom(false, name)
                    .unwrap()
                    .reply()
                    .unwrap()
                    .atom
            };
            let above = atom(b"_NET_WM_STATE_ABOVE");
            let skip = atom(b"_NET_WM_STATE_SKIP_TASKBAR");
            connection
                .change_property32(
                    PropMode::REPLACE,
                    xid,
                    atom(b"_NET_WM_STATE"),
                    AtomEnum::ATOM,
                    &[above],
                )
                .unwrap()
                .check()
                .unwrap();
            connection
                .change_property32(
                    PropMode::REPLACE,
                    xid,
                    atom(b"_NET_WM_USER_TIME"),
                    AtomEnum::CARDINAL,
                    &[1234],
                )
                .unwrap()
                .check()
                .unwrap();
            let config = WindowConfig {
                focus_on_launch: false,
                skip_taskbar: true,
                ..Default::default()
            };
            for _ in 0..2 {
                prepare_window_presentation(window, &config).unwrap();
                assert_eq!(property(xid, b"_NET_WM_STATE"), vec![above, skip]);
                assert_eq!(property(xid, b"_NET_WM_USER_TIME"), vec![0]);
            }
        },
    );
}

#[test]
#[ignore = "requires the private X11 display"]
fn ordinary_windows_keep_their_activation_and_taskbar_policy() {
    presentation_check(WindowConfig::default(), |window, xid| {
        use x11rb::{
            protocol::xproto::{AtomEnum, PropMode},
            wrapper::ConnectionExt as _,
        };
        let (connection, _) = x11rb::connect(None).unwrap();
        let time = connection
            .intern_atom(false, b"_NET_WM_USER_TIME")
            .unwrap()
            .reply()
            .unwrap()
            .atom;
        connection
            .change_property32(PropMode::REPLACE, xid, time, AtomEnum::CARDINAL, &[1234])
            .unwrap()
            .check()
            .unwrap();
        prepare_window_presentation(window, &WindowConfig::default()).unwrap();
        assert_eq!(property(xid, b"_NET_WM_USER_TIME"), vec![1234]);
        assert!(property(xid, b"_NET_WM_STATE").is_empty());
    });
}
