#![cfg(test)]
use super::*;
use argui_core::Size;
use argui_paint::{ImageAsset, ImageId};
use argui_ui::{UiTree, length};

fn application() -> Application {
    Application::new(
        Default::default(),
        Default::default(),
        Default::default(),
        None,
        None,
        None,
        |_| {},
    )
}

#[test]
fn keyed_bounds_require_a_live_scene_and_completed_layout() {
    let mut app = application();
    assert!(
        app.native_element_bounds("reader")
            .unwrap_err()
            .contains("not mounted")
    );
    app.ui_tree = Some(UiTree::new(
        Element::container([])
            .keyed("reader")
            .width(length(80.0))
            .height(length(40.0)),
    ));
    assert!(
        app.native_element_bounds("reader")
            .unwrap_err()
            .contains("not laid out")
    );
    let layout = app
        .layout_engine
        .compute(
            app.ui_tree.as_mut().unwrap(),
            &mut app.text_engine,
            Size::new(100.0, 100.0),
        )
        .unwrap();
    app.ui_layout = Some(layout);
    assert_eq!(
        app.native_element_bounds("reader").unwrap().size,
        Size::new(80.0, 40.0)
    );
    assert!(
        app.native_element_bounds("missing")
            .unwrap_err()
            .contains("unavailable")
    );
    app.ui_layout.as_mut().unwrap().nodes.clear();
    assert!(
        app.native_element_bounds("reader")
            .unwrap_err()
            .contains("no layout")
    );
}

#[test]
fn replacing_raster_assets_retains_one_image_per_identity() {
    let mut app = application();
    for value in [10, 20, 30] {
        app.register_native_image(ImageAsset::rgba8(ImageId(1), 2, 2, vec![value; 16]).unwrap())
            .unwrap();
        assert_eq!(app.image_assets.len(), 1);
        assert!(app.pending_ui_frame.needs_frame());
    }
    app.register_native_image(ImageAsset::rgba8(ImageId(2), 1, 1, vec![255; 4]).unwrap())
        .unwrap();
    assert_eq!(app.image_assets.len(), 2);
}
