use super::*;
use argui::core::{Key, KeyInput, KeyState, Modifiers, Point, PointerEvent, PointerPhase};
use argui::paint::Filter;

#[test]
fn popover_examples_edit_settings_and_dismiss_with_distinct_effect_surfaces() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::popover");
    for (index, key) in ["popover-project", "popover-sharing", "popover-color"]
        .into_iter()
        .enumerate()
    {
        assert!(keyed(&app.render(), &format!("{key}::content")).is_none());
        click(&app, key);
        let root = app.render();
        let panel = keyed(&root, &format!("{key}::content")).unwrap();
        let filters = &panel.layer.as_ref().unwrap().backdrop_filters;
        match index {
            0 => assert!(filters.is_empty()),
            1 => assert!(matches!(filters.as_slice(), [Filter::Blur(14.0)])),
            _ => assert!(
                matches!(filters.as_slice(), [Filter::Effect(effect)] if effect.id.0 == "gallery.overlay.prism")
            ),
        }
        match index {
            0 => {
                dispatch(
                    &app,
                    "popover-name",
                    UiEventKind::TextChanged("Release notes".into()),
                );
                click(&app, "popover-save");
                assert!(contains_text(
                    &app.render(),
                    "Project: Release notes · Accent: Blue"
                ));
                click(&app, key);
            }
            1 => {
                click(&app, "popover-sharing-toggle");
                assert!(contains_text(&app.render(), "Public link enabled"));
                click(&app, "popover-sharing-toggle");
            }
            _ => {
                for (slug, name) in [("rose", "Rose"), ("violet", "Violet"), ("blue", "Blue")] {
                    click(&app, &format!("popover-accent-{slug}"));
                    assert!(contains_text(
                        &app.render(),
                        &format!("Project: Release notes · Accent: {name}")
                    ));
                }
            }
        }
        dispatch(
            &app,
            key,
            UiEventKind::KeyInput(KeyInput {
                key: Key::Escape,
                state: KeyState::Pressed,
                modifiers: Modifiers::default(),
                repeat: false,
                text: None,
            }),
        );
        assert!(keyed(&app.render(), &format!("{key}::content")).is_none());
        click(&app, key);
        dispatch(
            &app,
            &format!("{key}::content"),
            UiEventKind::PointerOutside(PointerEvent::mouse(
                PointerPhase::Pressed,
                Point::default(),
            )),
        );
        assert!(keyed(&app.render(), &format!("{key}::content")).is_none());
        click(&app, key);
        click(&app, key);
        assert!(keyed(&app.render(), &format!("{key}::content")).is_none());
    }
}
