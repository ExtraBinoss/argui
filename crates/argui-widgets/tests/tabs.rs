use argui_core::{Affine2D, Color, ColorScheme, Point, Rect, Size};
use argui_paint::ClipChain;
use argui_ui::{
    CursorIcon, Element, GestureSet, HitRegion, HitShape, HitTestStyle, Role, UiEvent, UiEventKind,
    UiTree, UserSelect,
};
use argui_widgets::{Tab, Tabs, TabsAction, TabsBehavior, TabsPart, shadcn};

fn click(key: &str) -> UiEvent {
    let tree = UiTree::new(Element::container([]));
    UiEvent::new(
        tree.node_ids()[0],
        Some(key.into()),
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    )
}

#[test]
fn tabs_mount_only_the_selected_panel_and_decode_selection() {
    let themes = shadcn(Color::srgb(0.2, 0.5, 0.9));
    let tabs = Tabs::new(
        "settings",
        [
            Tab::new("General", Element::text("general")),
            Tab::new("GPU", Element::text("gpu")),
        ],
        9,
    )
    .build(themes.resolve(ColorScheme::Light));
    assert_eq!(tabs.children.len(), 2);
    assert_eq!(
        tabs.children[0].semantics.as_ref().unwrap().role,
        Role::TabList
    );
    assert!(
        tabs.children[0]
            .children
            .iter()
            .all(|trigger| trigger.user_select == UserSelect::None)
    );
    assert_eq!(
        tabs.children[1].semantics.as_ref().unwrap().role,
        Role::TabPanel
    );
    assert_eq!(tabs.children[1].key.as_deref(), Some("settings::panel::1"));
    let behavior = TabsBehavior::new(
        "settings",
        [("General".into(), true), ("GPU".into(), true)],
        1,
    );
    assert_eq!(
        behavior.action(&click("settings::tab::0")),
        Some(TabsAction::Select(0))
    );
    assert_eq!(behavior.action(&click("other::tab::0")), None);
    assert_eq!(
        behavior.action(&UiEvent::new(
            click("settings::tab::0").target,
            Some("settings::tab::0".into()),
            UiEventKind::Focused,
        )),
        None
    );

    let disabled = TabsBehavior::new("disabled", [("GPU".into(), false)], 0)
        .decorate(TabsPart::Trigger(0), Element::container([]));
    assert_eq!(
        disabled.interaction.as_ref().unwrap().cursor,
        CursorIcon::NotAllowed
    );
}

#[test]
fn active_tab_keeps_its_surface_while_hovered() {
    let themes = shadcn(Color::srgb(0.2, 0.5, 0.9));
    let tabs = Tabs::new(
        "settings",
        [
            Tab::new("General", Element::text("general")),
            Tab::new("GPU", Element::text("gpu")),
        ],
        0,
    )
    .build(themes.resolve(ColorScheme::Dark));
    let resting = tabs.children[0].children[0].paint.quad.background.clone();
    let mut tree = UiTree::new(tabs);
    let active = tree.node_id_at(2).unwrap();
    tree.set_reduced_motion(true);
    tree.pointer_moved(
        Point::new(10.0, 10.0),
        &[HitRegion {
            node: active,
            bounds: Rect::new(Point::default(), Size::new(80.0, 32.0)),
            transform: Affine2D::IDENTITY,
            clips: ClipChain::default(),
            shape: HitShape::Bounds,
            slop: HitTestStyle::default().slop,
            enabled: true,
            focus_policy: argui_ui::FocusPolicy::TabStop,
            cursor: CursorIcon::Pointer,
            gestures: GestureSet::EMPTY,
            window_drag: None,
        }],
    );

    assert_eq!(
        tree.resolved_quad(active, tree.element_at(2).unwrap())
            .background,
        resting
    );
}
