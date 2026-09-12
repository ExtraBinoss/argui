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
            1 => assert!(matches!(filters.as_slice(), [Filter::Blur(6.0)])),
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

#[test]
fn nested_link_options_keep_the_parent_open_and_close_one_level_at_a_time() {
    let app = Entity::new(WidgetGallery::default());
    let parent = "popover-sharing::content";
    let child = "popover-link-options::content";
    let escape = || {
        UiEventKind::KeyInput(KeyInput {
            key: Key::Escape,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            repeat: false,
            text: None,
        })
    };
    click(&app, "nav::popover");
    click(&app, "popover-sharing");
    click(&app, "popover-link-options");
    for expected in ["Downloads allowed", "View only"] {
        click(&app, "popover-downloads-toggle");
        let root = app.render();
        assert!(contains_text(&root, expected));
        assert!(keyed(&root, parent).is_some());
        assert!(keyed(&root, child).is_some());
    }
    dispatch(&app, "popover-downloads-toggle", escape());
    assert!(keyed(&app.render(), child).is_none());
    assert!(keyed(&app.render(), parent).is_some());
    dispatch(&app, "popover-link-options", escape());
    assert!(keyed(&app.render(), parent).is_none());
    click(&app, "popover-sharing");
    click(&app, "popover-link-options");
    click(&app, "popover-link-options");
    assert!(keyed(&app.render(), child).is_none());
    assert!(keyed(&app.render(), parent).is_some());
    click(&app, "popover-link-options");
    dispatch(
        &app,
        child,
        UiEventKind::PointerOutside(PointerEvent::mouse(PointerPhase::Pressed, Point::default())),
    );
    assert!(keyed(&app.render(), child).is_none());
    assert!(keyed(&app.render(), parent).is_some());
    click(&app, "popover-link-options");
    click(&app, "nav::button");
    click(&app, "nav::popover");
    click(&app, "popover-sharing");
    assert!(keyed(&app.render(), child).is_none());
    click(&app, "popover-link-options");
    dispatch(&app, "gallery-search", escape());
    assert!(keyed(&app.render(), child).is_none());
    assert!(keyed(&app.render(), parent).is_some());
    dispatch(&app, "gallery-search", escape());
    assert!(keyed(&app.render(), parent).is_none());
    dispatch(&app, "gallery-search", escape());
}
