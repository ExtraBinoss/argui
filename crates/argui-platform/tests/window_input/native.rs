//! Verifies X11 input shapes on the private test display.

#[cfg(target_os = "linux")]
fn exercise() {
    use argui_core::{Point, Rect, Size};
    use argui_platform::{
        WindowConfig, WindowInputRegion, apply_window_input_region, window_capabilities,
    };
    use winit::{
        application::ApplicationHandler,
        event::WindowEvent,
        event_loop::{ActiveEventLoop, EventLoop},
        platform::x11::EventLoopBuilderExtX11,
        raw_window_handle::{HasWindowHandle, RawWindowHandle},
        window::WindowId,
    };
    use x11rb::protocol::shape::{ConnectionExt as _, SK};

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    if std::env::var_os("ARGUI_WINDOW_INPUT_TEST_CHILD").is_none() {
        assert!(
            std::process::Command::new(root.join("scripts/linux-hidden-display.sh"))
                .env("ARGUI_TEST_BACKEND", "x11")
                .env("ARGUI_WINDOW_INPUT_TEST_CHILD", "1")
                .current_dir(root)
                .arg("timeout")
                .arg("25s")
                .arg(std::env::current_exe().unwrap())
                .arg("--exact")
                .arg("native_window_input")
                .arg("--nocapture")
                .status()
                .unwrap()
                .success()
        );
        return;
    }
    assert_eq!(std::env::var("ARGUI_HIDDEN_DISPLAY").as_deref(), Ok("1"));
    assert!(std::env::var_os("WAYLAND_DISPLAY").is_none());
    struct Check;
    impl ApplicationHandler for Check {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            let window = event_loop
                .create_window(
                    WindowConfig {
                        width: 800.0,
                        height: 600.0,
                        ..WindowConfig::default()
                    }
                    .into_attributes()
                    .with_visible(false),
                )
                .unwrap();
            assert!(!window_capabilities(&window).transparent_compositing);
            let xid = match window.window_handle().unwrap().as_raw() {
                RawWindowHandle::Xlib(handle) => handle.window as u32,
                RawWindowHandle::Xcb(handle) => handle.window.get(),
                _ => panic!("X11 window expected"),
            };
            let (connection, _) = x11rb::connect(None).unwrap();
            let shape = || {
                connection
                    .shape_get_rectangles(xid, SK::INPUT)
                    .unwrap()
                    .reply()
                    .unwrap()
                    .rectangles
            };
            let hole = Rect::new(Point::new(100.0, 120.0), Size::new(300.0, 200.0));
            apply_window_input_region(&window, &WindowInputRegion::Exclude(hole), 1.0).unwrap();
            let regions = shape();
            assert_eq!(regions.len(), 4);
            assert!(!regions.iter().any(|rect| {
                let x = i32::from(rect.x);
                let y = i32::from(rect.y);
                x <= 250
                    && 250 < x + i32::from(rect.width)
                    && y <= 200
                    && 200 < y + i32::from(rect.height)
            }));
            apply_window_input_region(&window, &WindowInputRegion::Exclude(hole), 1.5).unwrap();
            let scaled = shape();
            assert_eq!(scaled.len(), 4);
            assert!(scaled.iter().all(|rect| {
                let x = i32::from(rect.x);
                let y = i32::from(rect.y);
                !(x <= 300
                    && 300 < x + i32::from(rect.width)
                    && y <= 300
                    && 300 < y + i32::from(rect.height))
            }));
            apply_window_input_region(&window, &WindowInputRegion::PassThrough, 1.0).unwrap();
            assert!(shape().is_empty());
            apply_window_input_region(&window, &WindowInputRegion::Full, 1.0).unwrap();
            assert_eq!(
                shape()
                    .iter()
                    .map(|rect| u32::from(rect.width) * u32::from(rect.height))
                    .sum::<u32>(),
                800 * 600
            );
            assert!(
                apply_window_input_region(&window, &WindowInputRegion::Exclude(hole), f64::NAN)
                    .is_err()
            );
            event_loop.exit();
        }
        fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, _: WindowEvent) {}
    }
    let mut builder = EventLoop::<()>::with_user_event();
    builder.with_x11();
    builder.with_any_thread(true);
    builder.build().unwrap().run_app(&mut Check).unwrap();
}

#[test]
fn native_window_input() {
    #[cfg(target_os = "linux")]
    exercise();
}
