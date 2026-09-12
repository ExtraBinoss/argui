use argui_core::Color;
use argui_paint::Fill;
use argui_ui::{DesktopBackdrop, DesktopBackdropState, Element, TreeUpdate, UiTree};

#[test]
fn native_tint_activity_and_fallback_replace_only_the_background() {
    let tint = Color::srgba(0.1, 0.2, 0.3, 0.65);
    let inactive = tint.with_alpha(0.9);
    let backdrop = DesktopBackdrop::new(tint, Color::WHITE).inactive_tint(inactive);
    let element = Element::text("Opaque label").desktop_backdrop(backdrop);
    let mut ui = UiTree::new(element.clone());
    let node = ui.node_ids()[0];
    assert_eq!(
        ui.resolved_quad(node, &element).background,
        Some(Fill::Solid(Color::WHITE))
    );
    assert!(!ui.set_desktop_backdrop_state(DesktopBackdropState::default()));
    for (available, focused, expected) in [
        (true, true, tint),
        (true, false, inactive),
        (false, false, Color::WHITE),
    ] {
        assert!(ui.set_desktop_backdrop_state(DesktopBackdropState { available, focused }));
        let quad = ui.resolved_quad(node, &element);
        assert_eq!(quad.background, Some(Fill::Solid(expected)));
        assert_eq!(quad.opacity, 1.0);
        assert_eq!(ui.node_ids()[0], node);
    }
    assert_eq!(ui.update(Element::text("Opaque label")), TreeUpdate::Paint);
    assert_eq!(ui.resolved_quad(node, ui.root()).background, None);
}

#[test]
fn detached_popup_uses_its_fallback_even_when_the_main_window_has_blur() {
    use argui_core::{Point, Rect, Size};
    use argui_ui::WindowLayer;
    let element = Element::container([])
        .portal(WindowLayer::Popover)
        .desktop_backdrop(DesktopBackdrop::new(Color::TRANSPARENT, Color::WHITE));
    let mut ui = UiTree::new(element.clone());
    let node = ui.node_ids()[0];
    ui.set_desktop_backdrop_state(DesktopBackdropState {
        available: true,
        focused: true,
    });
    assert_eq!(
        ui.resolved_quad(node, &element).background,
        Some(Fill::Solid(Color::TRANSPARENT))
    );
    ui.set_native_portal(
        node,
        Some(Rect::new(Point::new(100.0, 80.0), Size::new(220.0, 150.0))),
    );
    assert_eq!(
        ui.resolved_quad(node, &element).background,
        Some(Fill::Solid(Color::WHITE))
    );
    ui.set_native_portal(node, None);
    assert_eq!(
        ui.resolved_quad(node, &element).background,
        Some(Fill::Solid(Color::TRANSPARENT))
    );
}
