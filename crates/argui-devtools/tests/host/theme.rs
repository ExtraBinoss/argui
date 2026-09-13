use super::*;
use argui_core::{Color, ColorScheme};
use argui_runtime::WindowEnvironment;
use std::cell::Cell;

struct ThemedApp(Rc<Cell<usize>>);
impl Render for ThemedApp {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        self.0.set(self.0.get() + 1);
        let environment = cx.environment();
        let themes = argui_widgets::shadcn(&environment);
        let palette = themes.resolve(environment.color_scheme);
        Element::container([])
            .keyed("themed-app")
            .background(palette.background)
    }
}

fn click(key: &str) -> UiEvent {
    event(
        key,
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    )
}

fn app_color(view: &Element) -> Color {
    let app = super::input::find_element(view, "themed-app").unwrap();
    let Some(argui_paint::Fill::Solid(color)) = app.paint.quad.background else {
        panic!("solid application background");
    };
    color
}

#[test]
fn theme_preview_invalidates_mounted_consumers_only_when_values_change_and_resets_cleanly() {
    let renders = Rc::new(Cell::new(0));
    let host = Entity::new(DevtoolsHost::new(ThemedApp(renders.clone())).open(true))
        .mount()
        .unwrap();
    let original = app_color(&host.render(Default::default()).unwrap());
    let update = |event: UiEvent| change_tools(&host, |tools| tools.update(&event));
    update(click("__devtools-theme"));
    assert!(contains_key(
        &host.render(Default::default()).unwrap(),
        "__devtools-theme-tokens"
    ));
    update(click("__devtools-theme-color-background"));
    update(event(
        "__devtools-color-theme-background::field::0",
        UiEventKind::TextChanged("#cc224488".into()),
    ));
    let next = app_color(&host.render(Default::default()).unwrap());
    assert_eq!(next.to_srgba8(), [204, 34, 68, 136]);
    let before = renders.get();
    update(event(
        "__devtools-color-theme-background::field::0",
        UiEventKind::TextChanged("#xx".into()),
    ));
    assert_eq!(app_color(&host.render(Default::default()).unwrap()), next);
    assert_eq!(
        renders.get(),
        before,
        "invalid input does not rebuild the application"
    );
    update(click("__devtools-theme-copy"));
    let copied = host
        .update(|tools, _| tools.take_clipboard_request())
        .unwrap()
        .unwrap();
    let argui_ui::ClipboardRequest::Write(json) = copied else {
        panic!("export");
    };
    let json: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(json["tokens"]["background"], "#CC224488");
    update(click("__devtools-theme-reset-background"));
    assert_eq!(
        app_color(&host.render(Default::default()).unwrap()),
        original
    );
    update(click("__devtools-theme-dark"));
    assert_ne!(
        app_color(&host.render(Default::default()).unwrap()),
        original
    );
    update(click("__devtools-theme-reset"));
    assert_eq!(
        app_color(&host.render(Default::default()).unwrap()),
        original
    );
    update(click("__devtools-theme-light"));
    update(click("__devtools-theme-system"));
    let view = host
        .render(WindowEnvironment {
            color_scheme: ColorScheme::Dark,
            ..Default::default()
        })
        .unwrap();
    assert_ne!(app_color(&view), original);
}

#[test]
fn theme_blur_fields_preserve_invalid_text_and_export_only_finite_values() {
    let host = Entity::new(populated_host()).mount().unwrap();
    host.render(Default::default()).unwrap();
    change_tools(&host, |tools| tools.update(&click("__devtools-theme")));
    for (input, valid) in [
        ("12.5", true),
        ("NaN", false),
        ("-1", false),
        ("101", false),
    ] {
        change_tools(&host, |tools| {
            tools.update(&event(
                "__devtools-theme-number-overlay-blur",
                UiEventKind::TextChanged(input.into()),
            ))
        });
        let view = host.render(Default::default()).unwrap();
        assert_eq!(contains_text(&view, "Enter a value from 0 to 100"), !valid);
        change_tools(&host, |tools| tools.update(&click("__devtools-theme-copy")));
        let argui_ui::ClipboardRequest::Write(json) = host
            .update(|tools, _| tools.take_clipboard_request())
            .unwrap()
            .unwrap()
        else {
            panic!("export");
        };
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&json).unwrap()["tokens"]["overlay-blur"],
            12.5
        );
    }
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-theme-number-overlay-blur",
            UiEventKind::Blurred,
        ))
    });
    assert!(!contains_text(
        &host.render(Default::default()).unwrap(),
        "Enter a value from 0 to 100"
    ));
}
