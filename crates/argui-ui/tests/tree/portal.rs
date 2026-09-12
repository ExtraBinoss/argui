use argui_core::{Point, Rect, Size};
use argui_ui::{Element, OverlaySurface, UiTree, WindowLayer};

#[test]
fn nested_portals_inherit_preferences_but_keep_their_physical_owner() {
    let mut ui = UiTree::new(Element::column([
        Element::column([
            Element::container([])
                .portal(WindowLayer::Popover)
                .keyed("inherited"),
            Element::container([])
                .portal(WindowLayer::Popover)
                .portal_surface(OverlaySurface::InWindow)
                .keyed("internal"),
        ])
        .portal(WindowLayer::Popover)
        .portal_surface(OverlaySurface::PreferNative)
        .keyed("outer"),
        Element::container([])
            .portal(WindowLayer::Popover)
            .keyed("sibling"),
    ]));
    let node = |key| {
        *ui.node_ids()
            .iter()
            .find(|id| ui.key(**id) == Some(key))
            .unwrap()
    };
    let outer = node("outer");
    let inherited = node("inherited");
    let internal = node("internal");
    let sibling = node("sibling");
    assert_eq!(
        ui.portal_surface_preference(inherited),
        OverlaySurface::PreferNative
    );
    assert_eq!(
        ui.portal_surface_preference(internal),
        OverlaySurface::InWindow
    );
    assert_eq!(
        ui.portal_surface_preference(sibling),
        OverlaySurface::InWindow
    );
    let bounds = Rect::new(Point::new(300.0, -40.0), Size::new(220.0, 150.0));
    assert!(ui.set_native_portal(outer, Some(bounds)));
    assert!(!ui.set_native_portal(outer, Some(bounds)));
    assert_eq!(ui.native_portal_owner(inherited), Some(outer));
    assert_eq!(ui.native_portal_owner(internal), Some(outer));
    assert_eq!(ui.native_portal_owner(sibling), None);
    assert!(ui.set_native_portal(inherited, Some(bounds)));
    assert_eq!(ui.native_portal_owner(inherited), Some(inherited));
    assert!(ui.set_native_portal(inherited, None));
    assert_eq!(ui.native_portal_owner(inherited), Some(outer));
    assert!(ui.set_native_portal(outer, None));
    assert_eq!(ui.native_portal_owner(internal), None);
    assert!(!ui.set_native_portal(sibling, Some(Rect::default())));
    assert!(!ui.set_native_portal(ui.node_ids()[0], Some(bounds)));
    for invalid in [
        Rect::new(Point::new(f32::NAN, 0.0), bounds.size),
        Rect::new(Point::new(0.0, f32::INFINITY), bounds.size),
        Rect::new(Point::default(), Size::new(f32::NAN, 2.0)),
        Rect::new(Point::default(), Size::new(2.0, f32::INFINITY)),
        Rect::new(Point::default(), Size::new(-2.0, 1.0)),
    ] {
        assert!(!ui.set_native_portal(sibling, Some(invalid)));
    }
    ui.set_native_portal(outer, Some(bounds));
    ui.update(Element::column([]));
    assert!(!ui.set_native_portal(outer, Some(bounds)));
    assert_eq!(ui.native_portal_bounds(outer), None);
    assert_eq!(ui.native_portal_owner(outer), None);
    assert_eq!(
        ui.portal_surface_preference(outer),
        OverlaySurface::InWindow
    );
}
