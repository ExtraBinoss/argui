// Exercise X11 protocol negotiation and region lifetime on a private server.
#[cfg(target_os = "linux")]
fn exercise() {
    use argui_core::{BackdropMaterial, ColorScheme, Point, Rect, Size};
    use argui_paint::{ClipChain, ClipRegion};
    use argui_platform::{
        WindowConfig,
        desktop_backdrop::{BackdropError, NativeBackdrop},
    };
    use std::sync::Arc;
    use winit::{
        application::ApplicationHandler,
        event::WindowEvent,
        event_loop::{ActiveEventLoop, EventLoop},
        platform::x11::EventLoopBuilderExtX11,
        raw_window_handle::{HasWindowHandle, RawWindowHandle},
        window::WindowId,
    };
    use x11rb::{
        connection::Connection,
        protocol::xproto::{AtomEnum, ConnectionExt, PropMode},
        wrapper::ConnectionExt as _,
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    if std::env::var_os("ARGUI_BACKDROP_TEST_CHILD").is_none() {
        assert!(
            std::process::Command::new(root.join("scripts/linux-hidden-display.sh"))
                .env("ARGUI_TEST_BACKEND", "x11")
                .env("ARGUI_BACKDROP_TEST_CHILD", "1")
                .current_dir(root)
                .arg("timeout")
                .arg("25s")
                .arg(std::env::current_exe().unwrap())
                .status()
                .unwrap()
                .success()
        );
        return;
    }
    assert_eq!(std::env::var("ARGUI_HIDDEN_DISPLAY").as_deref(), Ok("1"));
    let runtime = std::path::PathBuf::from(std::env::var("XDG_RUNTIME_DIR").unwrap());
    assert_eq!(runtime.parent(), Some(std::env::temp_dir().as_path()));
    assert!(
        runtime
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("argui-display.")
    );
    assert!(std::env::var_os("WAYLAND_DISPLAY").is_none());
    struct Check;
    impl ApplicationHandler for Check {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            let window = Arc::new(
                event_loop
                    .create_window(
                        WindowConfig::default()
                            .into_attributes()
                            .with_visible(false),
                    )
                    .unwrap(),
            );
            let id = match window.window_handle().unwrap().as_raw() {
                RawWindowHandle::Xlib(handle) => handle.window as u32,
                _ => panic!("private X11 window expected"),
            };
            let (connection, screen) = x11rb::connect(None).unwrap();
            let root = connection.setup().roots[screen].root;
            let atom = connection
                .intern_atom(false, b"_KDE_NET_WM_BLUR_BEHIND_REGION")
                .unwrap()
                .reply()
                .unwrap()
                .atom;
            let property = || {
                connection
                    .get_property(false, id, atom, AtomEnum::ANY, 0, 4096)
                    .unwrap()
                    .reply()
                    .unwrap()
            };
            let mut backdrop = NativeBackdrop::new(window).unwrap();
            let shape = ClipChain::from_regions([ClipRegion::new(
                Rect::new(Point::new(4.0, 8.0), Size::new(120.0, 200.0)),
                Default::default(),
            )]);
            let size = Size::new(800.0, 600.0);
            let update = |backdrop: &mut NativeBackdrop| {
                backdrop
                    .update(
                        std::slice::from_ref(&shape),
                        BackdropMaterial::Sidebar,
                        ColorScheme::Dark,
                        size,
                        2.0,
                    )
                    .unwrap()
            };
            // Atom existence alone must not claim that a compositor implements blur.
            assert!(!update(&mut backdrop));
            assert_eq!(property().type_, 0);
            connection
                .change_property32(PropMode::REPLACE, root, atom, AtomEnum::CARDINAL, &[1])
                .unwrap()
                .check()
                .unwrap();
            let started = std::time::SystemTime::now();
            while !update(&mut backdrop) {
                assert!(started.elapsed().unwrap() < std::time::Duration::from_secs(2));
                std::thread::yield_now();
            }
            assert_eq!(
                property().value32().unwrap().collect::<Vec<_>>(),
                [8, 16, 240, 400]
            );
            assert!(update(&mut backdrop));
            assert_eq!(
                backdrop.update(
                    &[],
                    BackdropMaterial::Glass,
                    ColorScheme::Light,
                    size,
                    f32::NAN
                ),
                Err(BackdropError::InvalidGeometry)
            );
            assert!(
                backdrop
                    .update(&[], BackdropMaterial::Glass, ColorScheme::Light, size, 1.0)
                    .unwrap()
            );
            assert_eq!(property().type_, 0);
            assert!(update(&mut backdrop));
            connection
                .delete_property(root, atom)
                .unwrap()
                .check()
                .unwrap();
            while update(&mut backdrop) {
                assert!(started.elapsed().unwrap() < std::time::Duration::from_secs(2));
                std::thread::yield_now();
            }
            assert_eq!(property().type_, 0);
            drop(backdrop);
            event_loop.exit();
        }
        fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, _: WindowEvent) {}
    }
    EventLoop::builder()
        .with_x11()
        .build()
        .unwrap()
        .run_app(&mut Check)
        .unwrap();
}

fn main() {
    let enabled = std::env::var_os("ARGUI_NATIVE_TESTS").is_some() && cfg!(target_os = "linux");
    if std::env::args().any(|arg| arg == "--list") {
        if enabled && !std::env::args().any(|arg| arg == "--ignored") {
            println!("native_desktop_backdrop: test");
        }
        return;
    }
    #[cfg(target_os = "linux")]
    if enabled {
        exercise();
    }
}
