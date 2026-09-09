use super::*;
use argui::core::{Key, Point, PointerEvent, PointerPhase};

#[test]
fn action_menus_palette_queries_and_modal_controls_remain_independent() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::actions");
    dispatch(&app, "scope-left", UiEventKind::Focused);
    dispatch(
        &app,
        "app-menu",
        UiEventKind::Pointer(PointerEvent::mouse(PointerPhase::Pressed, Point::default())),
    );
    click(&app, "app-menu");
    assert!(keyed(&app.render(), "app-menu::item::record").is_some());
    keyboard(&app, "app-menu::item::record", Key::ArrowDown);
    keyboard(&app, "app-menu::item::record", Key::Escape);
    keyboard(&app, "app-menu", Key::ArrowDown);
    click(&app, "app-menu::item::record");
    click(&app, "actions-disable");
    assert!(contains_text(&app.render(), "Enable local actions"));
    click(&app, "actions-disable");
    click(&app, "app-palette");
    dispatch(
        &app,
        "app-palette::query",
        UiEventKind::TextChanged("record".into()),
    );
    keyboard(&app, "app-palette::query", Key::Enter);
    click(&app, "app-palette");
    keyboard(&app, "app-palette::query", Key::Escape);
    click(&app, "actions-modal::trigger");
    assert!(keyed(&app.render(), "modal-close").is_some());
    click(&app, "modal-close");
}
