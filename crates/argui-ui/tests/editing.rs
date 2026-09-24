use argui_core::{Affine2D, Key, KeyInput, KeyState, Modifiers, Point, Rect, Size};
use argui_paint::ClipChain;
use argui_text::TextStyle;
use argui_ui::{
    CaretStyle, CursorIcon, Element, EventHandlerId, EventListener, EventOwnerId, EventType,
    FocusPolicy, HitRegion, HitShape, Interaction, TextEditorSpec, TextInputFilter, TextPrivacy,
    UiTree,
};

#[path = "text_input/history.rs"]
mod history;
#[path = "text_input/privacy.rs"]
mod privacy;

fn field(value: &str, filter: TextInputFilter) -> Element {
    Element::text_editor(TextEditorSpec {
        value: value.to_owned(),
        placeholder: "Field".to_owned(),
        multiline: false,
        read_only: false,
        filter,
        text: TextStyle::default(),
        placeholder_text: TextStyle::default(),
        selection: argui_ui::Color::WHITE,
        caret: CaretStyle::default(),
    })
    .keyed("field")
    .interaction(
        Interaction::default()
            .focus_policy(FocusPolicy::TabStop)
            .cursor(CursorIcon::Text),
    )
    .on(EventListener::new(
        EventType::Input,
        EventHandlerId::new(EventOwnerId(1), 0),
    ))
}
fn editor(value: &str, filter: TextInputFilter) -> (UiTree, HitRegion) {
    editor_with_privacy(value, filter, TextPrivacy::Public)
}
fn editor_with_privacy(
    value: &str,
    filter: TextInputFilter,
    privacy: TextPrivacy,
) -> (UiTree, HitRegion) {
    let mut tree = UiTree::new(field(value, filter).text_privacy(privacy));
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
fn key(key: Key, text: Option<&str>, modifiers: Modifiers) -> KeyInput {
    KeyInput {
        key,
        text: text.map(str::to_owned),
        modifiers,
        state: KeyState::Pressed,
        repeat: false,
    }
}
fn type_text(tree: &mut UiTree, value: &str) {
    tree.edit_text_input(&key(
        Key::Character(value.into()),
        Some(value),
        Modifiers::default(),
    ));
}

#[test]
fn deferred_replacements_resolve_keys_and_refuse_missing_or_ambiguous_targets() {
    let (mut tree, region) = editor("before", TextInputFilter::Any);
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
        field("a", TextInputFilter::Any),
        field("b", TextInputFilter::Any),
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
