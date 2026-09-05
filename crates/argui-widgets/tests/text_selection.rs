use argui_core::{ColorScheme, Point, Rect, Size};
use argui_runtime::{Context, Entity, Render};
use argui_ui::{Element, ElementKind, SelectionCapabilities, UiEventKind, UiTree, UserSelect};
use argui_widgets::{TextSelectionToolbar, shadcn};

struct Editor;

impl Render for Editor {
    fn render(&mut self, _cx: &mut Context<Self>) -> Element {
        Element::text("selectable editor")
    }
}

fn dispatch<A: Render>(app: &Entity<A>, kind: UiEventKind) {
    let mut tree = UiTree::new(app.render());
    let target = tree.node_ids()[0];
    for event in tree.event_deliveries(target, kind) {
        if event.should_dispatch() {
            app.dispatch_event(&event);
        }
    }
}

fn dispatch_key<A: Render>(app: &Entity<A>, key: &str, kind: UiEventKind) {
    let mut tree = UiTree::new(app.render());
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some(key))
        .expect("selection toolbar command should be present");
    for event in tree.event_deliveries(target, kind) {
        if event.should_dispatch() {
            app.dispatch_event(&event);
        }
    }
}

fn has_key(element: &Element, key: &str) -> bool {
    element.key.as_deref() == Some(key) || element.children.iter().any(|child| has_key(child, key))
}

#[test]
fn touch_selection_toolbar_is_a_clamped_non_selectable_widget() {
    let palette = shadcn(argui_core::Color::srgb(0.2, 0.4, 0.8));
    let theme = palette.resolve(ColorScheme::Light);
    let toolbar = TextSelectionToolbar::new(
        "copy",
        Rect::new(Point::new(2.0, 2.0), Size::new(20.0, 20.0)),
        Size::new(200.0, 120.0),
        SelectionCapabilities {
            copy: true,
            select_all: true,
            ..SelectionCapabilities::default()
        },
    )
    .build(theme);

    assert!(matches!(toolbar.kind, ElementKind::Container));
    assert_eq!(toolbar.user_select, UserSelect::None);
    assert!(!toolbar.interaction.as_ref().unwrap().focusable);
    assert_eq!(toolbar.children.len(), 2);
}

#[test]
fn editable_toolbar_keeps_unavailable_commands_visible_but_disabled() {
    let palette = shadcn(argui_core::Color::srgb(0.2, 0.4, 0.8));
    let theme = palette.resolve(ColorScheme::Dark);
    let toolbar = TextSelectionToolbar::new(
        "editor",
        Rect::default(),
        Size::new(500.0, 300.0),
        SelectionCapabilities {
            editable: true,
            paste: true,
            select_all: true,
            ..SelectionCapabilities::default()
        },
    )
    .build(theme);

    assert_eq!(toolbar.children.len(), 4);
    assert!(!toolbar.children[0].interaction.as_ref().unwrap().enabled);
    assert!(!toolbar.children[1].interaction.as_ref().unwrap().enabled);
    assert!(toolbar.children[2].interaction.as_ref().unwrap().enabled);
    assert!(toolbar.children[3].interaction.as_ref().unwrap().enabled);
}

#[test]
fn selection_host_presents_and_removes_the_public_touch_toolbar() {
    let app = Entity::new(argui_widgets::SelectionHost::new(Editor));
    dispatch(
        &app,
        UiEventKind::DocumentSelectionChanged {
            text: Some("selected".into()),
            bounds: Some(Rect::new(Point::new(40.0, 60.0), Size::new(50.0, 18.0))),
            touch: true,
            dragging: false,
        },
    );
    assert!(matches!(app.render().kind, ElementKind::Container));
    dispatch(
        &app,
        UiEventKind::DocumentSelectionChanged {
            text: Some("selected".into()),
            bounds: Some(Rect::new(Point::new(40.0, 60.0), Size::new(50.0, 18.0))),
            touch: true,
            dragging: true,
        },
    );
    assert!(matches!(app.render().kind, ElementKind::Text { .. }));
}

#[test]
fn context_menu_requires_a_command_and_public_toolbar_commands_dismiss_it() {
    let app = Entity::new(argui_widgets::SelectionHost::new(Editor));
    dispatch(
        &app,
        UiEventKind::ContextMenu {
            position: Point::new(20.0, 24.0),
            capabilities: SelectionCapabilities::default(),
        },
    );
    assert!(!has_key(&app.render(), "argui::selection-menu::copy"));

    dispatch(
        &app,
        UiEventKind::ContextMenu {
            position: Point::new(20.0, 24.0),
            capabilities: SelectionCapabilities {
                copy: true,
                ..SelectionCapabilities::default()
            },
        },
    );
    assert!(has_key(&app.render(), "argui::selection-menu::copy"));

    dispatch_key(
        &app,
        "argui::selection-menu::copy",
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );
    assert!(!has_key(&app.render(), "argui::selection-menu::copy"));
}
