#![cfg(all(feature = "native-popups", not(target_arch = "wasm32")))]
use argui_core::{Point, Rect, Size};
use argui_platform::popup::{PopupEnvironment, PopupUnavailable};

#[test]
fn desktop_coordinates_preserve_negative_monitor_origins_and_fractional_scale() {
    let environment = PopupEnvironment::from_physical(
        Point::new(-1700.0, 120.0),
        1.5,
        Rect::new(Point::new(-1920.0, 30.0), Size::new(1920.0, 1050.0)),
    )
    .unwrap();
    assert_eq!(
        environment.work_area.origin,
        Point::new(-220.0 / 1.5, -60.0)
    );
    assert_eq!(environment.work_area.size, Size::new(1280.0, 700.0));
    let bounds = Rect::new(Point::new(100.0, -40.0), Size::new(200.0, 80.0));
    assert_eq!(
        environment.position(bounds),
        winit::dpi::PhysicalPosition::new(-1550, 60)
    );
    assert_eq!(
        environment.size(bounds),
        winit::dpi::PhysicalSize::new(300, 120)
    );
    assert_eq!(
        environment.size(Rect::default()),
        winit::dpi::PhysicalSize::new(1, 1)
    );
    let fractional = Rect::new(Point::new(3.25, -2.25), Size::new(32.75, 21.25));
    let snapped = environment.snap(fractional);
    assert_eq!(environment.snap(snapped), snapped);
    assert_eq!(
        environment.position(snapped),
        environment.position(fractional)
    );
    assert_eq!(environment.size(snapped), environment.size(fractional));
    assert!(PopupEnvironment::from_physical(Point::new(f32::NAN, 0.0), 1.0, bounds).is_none());
    for scale in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        assert!(PopupEnvironment::from_physical(Point::default(), scale, bounds).is_none());
    }
    assert!(PopupEnvironment::from_physical(Point::default(), 1.0, Rect::default()).is_none());
    assert!(
        PopupEnvironment::from_physical(
            Point::default(),
            1.0,
            Rect::new(Point::default(), Size::new(1.0, 0.0))
        )
        .is_none()
    );
    for reason in [
        PopupUnavailable::UnsupportedBackend,
        PopupUnavailable::UnknownGeometry,
        PopupUnavailable::Platform("refused".into()),
    ] {
        assert!(!reason.to_string().is_empty());
    }
}
