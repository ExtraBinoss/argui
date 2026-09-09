use argui_core::{Affine2D, Key, KeyInput, KeyState, Modifiers, Point, Rect, Size};
use argui_paint::{ClipChain, PaintStyle};
use argui_text::TextStyle;
use argui_ui::{
    CursorIcon, Element, EventHandlerId, EventListener, EventOwnerId, EventType, HitRegion,
    HitShape, UiTree,
};
use argui_widgets::{Input, InputKind, InputStyle};

#[path = "text_input/history.rs"]
mod history;
#[path = "text_input/privacy.rs"]
mod privacy;

fn field(value: &str, kind: InputKind) -> Element {
    Input::new(
        "field",
        value,
        "Field",
        InputStyle::new(PaintStyle::default(), TextStyle::default()),
    )
    .kind(kind)
    .build()
    .on(EventListener::new(
        EventType::Input,
        EventHandlerId::new(EventOwnerId(1), 0),
    ))
}
fn editor(value: &str, kind: InputKind) -> (UiTree, HitRegion) {
    let mut tree = UiTree::new(field(value, kind));
    let region = HitRegion {
        node: tree.node_ids()[0],
        bounds: Rect::new(Point::default(), Size::new(300.0, 40.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        shape: HitShape::Bounds,
        slop: argui_ui::Sides::length(0.0),
        enabled: true,
        focus_policy: argui_ui::FocusPolicy::TabStop,
        cursor: CursorIcon::Text,
        gestures: argui_ui::GestureSet::EMPTY,
        window_drag: None,
    };
    tree.sync_focus(
        std::slice::from_ref(&region),
        Some(argui_ui::FocusRequest::Focus(region.node.into())),
    );
    (tree, region)
}
fn key(key: Key, text: Option<&str>, command: bool, shift: bool) -> KeyInput {
    KeyInput {
        key,
        text: text.map(str::to_owned),
        modifiers: Modifiers {
            control: command,
            shift,
            ..Default::default()
        },
        state: KeyState::Pressed,
        repeat: false,
    }
}
fn type_text(tree: &mut UiTree, value: &str) {
    tree.edit_text_input(&key(
        Key::Character(value.into()),
        Some(value),
        false,
        false,
    ));
}

#[test]
fn deferred_replacements_resolve_keys_and_refuse_missing_or_ambiguous_targets() {
    let (mut tree, region) = editor("before", InputKind::Text);
    assert!(
        tree.apply_command(argui_ui::UiCommand::ReplaceText {
            target: "field".into(),
            value: "after".into()
        })
        .layout_changed
    );
    assert_eq!(tree.text_input_value(region.node), Some("after"));
    tree.undo_text_input(region.node);
    assert_eq!(tree.text_input_value(region.node), Some("before"));
    assert!(
        !tree
            .apply_command(argui_ui::UiCommand::ReplaceText {
                target: "missing".into(),
                value: "x".into()
            })
            .layout_changed
    );
    tree.replace(Element::column([
        field("a", InputKind::Text),
        field("b", InputKind::Text),
    ]));
    assert!(
        !tree
            .apply_command(argui_ui::UiCommand::ReplaceText {
                target: "field".into(),
                value: "x".into()
            })
            .layout_changed
    );
}
