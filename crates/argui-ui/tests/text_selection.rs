use argui_core::{CaretAffinity, TextPosition};
use argui_ui::{
    ClipboardRequest, DocumentTextPoint, Element, EventHandlerId, EventListener, EventOwnerId,
    EventType, GestureSet, Interaction, Role, SelectionCommand, SelectionGranularity, Semantics,
    TapGesture, TextEditorSpec, TextInputFilter, TextSelectionStyle, UiEventKind, UiTree,
    UserSelect,
};

fn listens_for_selection(element: Element) -> Element {
    element.on(EventListener::new(
        EventType::SelectionChange,
        EventHandlerId::new(EventOwnerId(1), 0),
    ))
}

#[test]
fn inherited_selection_policy_style_and_semantic_expansion_are_public() {
    let style = TextSelectionStyle {
        background: argui_core::Color::srgb(1.0, 0.0, 0.0),
        handle: argui_core::Color::WHITE,
    };
    let mut tree = UiTree::new(listens_for_selection(
        Element::column([
            Element::text("alpha beta"),
            Element::text("atomic").user_select(UserSelect::All),
        ])
        .selection_style(style),
    ));
    let first = tree.node_id_at(1).unwrap();
    let atomic = tree.node_id_at(2).unwrap();
    assert_eq!(tree.resolved_selection_style(first), style);

    let update = tree.begin_document_selection(
        DocumentTextPoint::new(first, TextPosition::new(7, CaretAffinity::After)),
        false,
        SelectionGranularity::Word,
    );
    assert_eq!(tree.selected_document_text().as_deref(), Some("beta"));
    assert!(matches!(
        update.events[0].kind,
        UiEventKind::DocumentSelectionChanged { .. }
    ));

    tree.begin_document_selection(
        DocumentTextPoint::new(atomic, TextPosition::new(3, CaretAffinity::After)),
        false,
        SelectionGranularity::Character,
    );
    assert_eq!(tree.selected_document_text().as_deref(), Some("atomic"));
}

#[test]
fn user_select_all_treats_a_container_as_one_atomic_selection() {
    let mut tree = UiTree::new(Element::column([
        Element::column([Element::text("first"), Element::text("second")])
            .user_select(UserSelect::All),
        Element::text("outside"),
    ]));
    let second = tree.node_id_at(3).unwrap();

    tree.begin_document_selection(
        DocumentTextPoint::new(second, TextPosition::new(2, CaretAffinity::After)),
        false,
        SelectionGranularity::Character,
    );

    assert_eq!(
        tree.selected_document_text().as_deref(),
        Some("first\nsecond")
    );
}

#[test]
fn selection_lifecycle_covers_extension_reverse_ranges_and_release() {
    let mut tree = UiTree::new(listens_for_selection(Element::column([
        Element::text("first"),
        Element::text("middle"),
        Element::text("last"),
    ])));
    let first = tree.node_id_at(1).unwrap();
    let middle = tree.node_id_at(2).unwrap();
    let last = tree.node_id_at(3).unwrap();

    assert!(!tree.has_document_selection());
    assert!(tree.selected_document_text().is_none());
    assert!(tree.release_document_selection().events.is_empty());
    assert!(tree.clear_document_selection().events.is_empty());

    tree.begin_document_selection(
        DocumentTextPoint::new(last, TextPosition::new(4, CaretAffinity::After)),
        false,
        SelectionGranularity::Character,
    );
    let unchanged = tree.begin_document_selection(
        DocumentTextPoint::new(last, TextPosition::new(4, CaretAffinity::After)),
        false,
        SelectionGranularity::Character,
    );
    assert!(unchanged.events.is_empty());
    tree.drag_document_selection(DocumentTextPoint::new(
        first,
        TextPosition::new(2, CaretAffinity::After),
    ));
    assert_eq!(
        tree.selected_document_text().as_deref(),
        Some("rst\nmiddle\nlast")
    );
    assert_eq!(tree.document_selection_range(middle, 6), Some(0..6));
    assert!(tree.document_selection_intersects(1, 2));
    assert!(!tree.document_selection_intersects(4, 1));
    assert!(tree.document_selection_dragging());

    let release = tree.release_document_selection();
    assert!(!tree.document_selection_dragging());
    assert!(!release.paint_changed);
    assert!(release.events.iter().any(|event| matches!(
        event.kind,
        UiEventKind::DocumentSelectionChanged {
            dragging: false,
            ..
        }
    )));
    tree.drag_document_selection(DocumentTextPoint::new(
        first,
        TextPosition::new(2, CaretAffinity::After),
    ));
    assert!(!tree.document_selection_dragging());

    tree.begin_document_selection(
        DocumentTextPoint::new(middle, TextPosition::new(3, CaretAffinity::After)),
        true,
        SelectionGranularity::Line,
    );
    assert_eq!(tree.document_selection().unwrap().anchor.node, last);
    assert!(tree.clear_document_selection().paint_changed);
    assert!(!tree.has_document_selection());
    assert!(!tree.document_selection_intersects(0, usize::MAX));
}

#[test]
fn word_selection_ignores_stationary_web_pointer_moves_and_drags_by_words() {
    let mut tree = UiTree::new(Element::text("alpha beta gamma"));
    let text = tree.node_id_at(0).unwrap();
    let point =
        |index| DocumentTextPoint::new(text, TextPosition::new(index, CaretAffinity::After));

    tree.begin_document_selection(point(7), false, SelectionGranularity::Word);
    assert_eq!(tree.selected_document_text().as_deref(), Some("beta"));

    tree.drag_document_selection(point(7));
    assert_eq!(tree.selected_document_text().as_deref(), Some("beta"));

    tree.drag_document_selection(point(13));
    assert_eq!(tree.selected_document_text().as_deref(), Some("beta gamma"));
    tree.release_document_selection();
    tree.drag_document_selection(point(1));
    assert_eq!(tree.selected_document_text().as_deref(), Some("beta gamma"));
}

#[test]
fn disabled_subtrees_can_be_explicitly_reenabled_and_invalid_points_clear() {
    let mut tree = UiTree::new(
        Element::column([
            Element::column([
                Element::text("hidden"),
                Element::text("visible").user_select(UserSelect::Text),
            ])
            .user_select(UserSelect::None),
            Element::text("tail"),
        ])
        .user_select(UserSelect::Text),
    );
    let hidden = tree.node_id_at(2).unwrap();
    let visible = tree.node_id_at(3).unwrap();
    let tail = tree.node_id_at(4).unwrap();
    assert_eq!(tree.resolved_user_select(hidden), UserSelect::None);
    assert_eq!(tree.resolved_user_select(visible), UserSelect::Text);

    tree.select_all_document_text();
    assert_eq!(
        tree.selected_document_text().as_deref(),
        Some("visible\ntail")
    );
    assert_eq!(tree.document_selection_range(hidden, 6), None);

    let update = tree.begin_document_selection(
        DocumentTextPoint::new(hidden, TextPosition::default()),
        false,
        SelectionGranularity::Character,
    );
    assert!(update.paint_changed);
    assert!(tree.document_selection().is_none());

    let untouched =
        tree.drag_document_selection(DocumentTextPoint::new(tail, TextPosition::default()));
    assert!(untouched.events.is_empty());

    let mut transient = UiTree::new(Element::column([Element::text("removed")]));
    let removed = transient.node_id_at(1).unwrap();
    transient.begin_document_selection(
        DocumentTextPoint::new(removed, TextPosition::new(1, CaretAffinity::After)),
        false,
        SelectionGranularity::Character,
    );
    transient.update(Element::container([]));
    assert!(transient.document_selection().is_none());
    assert_eq!(transient.resolved_user_select(removed), UserSelect::None);
    assert_eq!(
        transient.resolved_selection_style(removed),
        TextSelectionStyle::default()
    );
}

#[test]
fn touch_selection_exposes_handles_and_empty_documents_stay_empty() {
    let mut tree = UiTree::new(Element::text("one\ntwo"));
    let text = tree.node_id_at(0).unwrap();
    tree.begin_touch_document_selection(
        DocumentTextPoint::new(text, TextPosition::new(5, CaretAffinity::After)),
        SelectionGranularity::Line,
    );
    assert!(tree.document_selection_handles_visible());
    assert_eq!(tree.selected_document_text().as_deref(), Some("two"));

    let mut empty = UiTree::new(Element::container([]));
    assert!(empty.select_all_document_text().events.is_empty());
    assert!(empty.selected_document_text().is_none());
}

#[test]
fn auto_follows_the_cascade_instead_of_inferring_style_from_interaction() {
    let mut tree = UiTree::new(Element::column([
        Element::container([Element::text("raw interactive text")])
            .interaction(Interaction::default().focus_policy(argui_ui::FocusPolicy::TabStop)),
        Element::container([Element::text("hidden")])
            .interaction(Interaction::default().focus_policy(argui_ui::FocusPolicy::TabStop))
            .user_select(UserSelect::None),
        Element::container([Element::text("reenabled").user_select(UserSelect::Text)])
            .interaction(Interaction::default().focus_policy(argui_ui::FocusPolicy::TabStop))
            .user_select(UserSelect::None),
    ]));
    let raw = tree.node_id_at(2).unwrap();
    let hidden = tree.node_id_at(4).unwrap();
    let reenabled = tree.node_id_at(6).unwrap();
    assert_eq!(tree.resolved_user_select(raw), UserSelect::Text);
    assert_eq!(tree.resolved_user_select(hidden), UserSelect::None);
    assert_eq!(tree.resolved_user_select(reenabled), UserSelect::Text);

    tree.select_all_document_text();
    assert_eq!(
        tree.selected_document_text().as_deref(),
        Some("raw interactive text\nreenabled")
    );
}

#[test]
fn accessibility_roles_do_not_change_the_user_select_cascade() {
    let mut tree = UiTree::new(
        Element::column([
            Element::text("selectable document text"),
            Element::container([Element::text("aria button label")])
                .interaction(Interaction::default().focus_policy(argui_ui::FocusPolicy::TabStop))
                .semantics(Semantics::new(Role::Button)),
            Element::container([Element::text("widget button label")])
                .interaction(Interaction::default().focus_policy(argui_ui::FocusPolicy::TabStop))
                .semantics(Semantics::new(Role::Button))
                .user_select(UserSelect::None),
        ])
        .interaction(
            Interaction::default()
                .focus_policy(argui_ui::FocusPolicy::TabStop)
                .gestures(GestureSet::EMPTY.tap(TapGesture::default())),
        )
        .semantics(Semantics::new(Role::Window)),
    );
    let text = tree.node_id_at(1).unwrap();
    let aria_button_label = tree.node_id_at(3).unwrap();
    let widget_button_label = tree.node_id_at(5).unwrap();

    assert_eq!(tree.resolved_user_select(text), UserSelect::Text);
    assert_eq!(
        tree.resolved_user_select(aria_button_label),
        UserSelect::Text
    );
    assert_eq!(
        tree.resolved_user_select(widget_button_label),
        UserSelect::None
    );
    tree.select_all_document_text();
    assert_eq!(
        tree.selected_document_text().as_deref(),
        Some("selectable document text\naria button label")
    );
}

#[test]
fn contain_clamps_dragging_and_select_all_to_its_own_scope() {
    let mut tree = UiTree::new(Element::column([
        Element::text("before"),
        Element::column([Element::text("first"), Element::text("second")])
            .user_select(UserSelect::Contain),
        Element::column([Element::text("outside")]).user_select(UserSelect::Contain),
    ]));
    let before = tree.node_id_at(1).unwrap();
    let first = tree.node_id_at(3).unwrap();
    let second = tree.node_id_at(4).unwrap();
    let outside = tree.node_id_at(6).unwrap();
    tree.begin_document_selection(
        DocumentTextPoint::new(first, TextPosition::new(1, CaretAffinity::After)),
        false,
        SelectionGranularity::Character,
    );
    tree.drag_document_selection(DocumentTextPoint::new(
        outside,
        TextPosition::new(3, CaretAffinity::After),
    ));
    assert_eq!(tree.document_selection().unwrap().focus.node, second);

    tree.begin_document_selection(
        DocumentTextPoint::new(first, TextPosition::new(1, CaretAffinity::After)),
        false,
        SelectionGranularity::Character,
    );
    tree.drag_document_selection(DocumentTextPoint::new(
        before,
        TextPosition::new(3, CaretAffinity::After),
    ));
    assert_eq!(tree.document_selection().unwrap().focus.node, first);

    tree.select_all_document_text();
    assert_eq!(
        tree.selected_document_text().as_deref(),
        Some("first\nsecond")
    );
}

#[test]
fn selection_commands_target_the_requested_editor_without_focus() {
    let editor = Element::text_editor(TextEditorSpec {
        value: "editable".into(),
        placeholder: String::new(),
        multiline: false,
        read_only: false,
        filter: TextInputFilter::Any,
        text: argui_text::TextStyle::default(),
        placeholder_text: argui_text::TextStyle::default(),
        selection: argui_core::Color::TRANSPARENT,
        caret: argui_ui::CaretStyle::default(),
    });
    let mut tree = UiTree::new(editor);
    let target = tree.node_id_at(0).unwrap();
    let capabilities = tree.selection_capabilities(target);
    assert!(capabilities.editable);
    assert!(capabilities.paste);
    assert!(!capabilities.copy);

    tree.selection_command(Some(target), SelectionCommand::SelectAll);
    let copied = tree.selection_command(Some(target), SelectionCommand::Copy);
    assert_eq!(
        copied.clipboard,
        Some(ClipboardRequest::Write("editable".into()))
    );
    let paste = tree.selection_command(Some(target), SelectionCommand::Paste);
    assert_eq!(
        paste.clipboard,
        Some(ClipboardRequest::Read {
            target: Some(target)
        })
    );
}

#[test]
fn document_selection_contains_points_and_exposes_static_commands() {
    let mut tree = UiTree::new(Element::column([
        Element::text("alpha"),
        Element::text("beta"),
        Element::text("gamma"),
    ]));
    let first = tree.node_id_at(1).unwrap();
    let middle = tree.node_id_at(2).unwrap();
    let last = tree.node_id_at(3).unwrap();
    let point =
        |node, index| DocumentTextPoint::new(node, TextPosition::new(index, CaretAffinity::After));

    assert!(!tree.document_selection_contains(point(first, 0)));
    assert!(!tree.selection_capabilities(first).copy);
    tree.begin_document_selection(point(last, 3), false, SelectionGranularity::Character);
    tree.drag_document_selection(point(first, 2));
    assert!(tree.document_selection_contains(point(first, 3)));
    assert!(tree.document_selection_contains(point(middle, 2)));
    assert!(!tree.document_selection_contains(point(first, 1)));
    assert!(!tree.document_selection_contains(point(last, 4)));
    let foreign = UiTree::new(Element::column([
        Element::text("foreign-a"),
        Element::text("foreign-b"),
        Element::text("foreign-c"),
        Element::text("foreign-d"),
    ]))
    .node_id_at(4)
    .unwrap();
    assert!(!tree.document_selection_contains(point(foreign, 0)));

    let capabilities = tree.selection_capabilities(middle);
    assert!(!capabilities.editable);
    assert!(capabilities.copy);
    assert!(capabilities.select_all);
    let copied = tree.focused_selection_command(SelectionCommand::Copy);
    assert_eq!(
        copied.clipboard,
        Some(ClipboardRequest::Write("pha\nbeta\ngam".into()))
    );
    assert_eq!(
        tree.selection_command(Some(middle), SelectionCommand::Copy)
            .clipboard,
        copied.clipboard
    );
    assert_eq!(
        tree.selection_command(None, SelectionCommand::Cut),
        Default::default()
    );
    assert_eq!(
        tree.selection_command(None, SelectionCommand::Paste),
        Default::default()
    );
}

#[test]
fn read_only_editor_reports_web_style_command_capabilities() {
    let editor = Element::text_editor(TextEditorSpec {
        value: "locked".into(),
        placeholder: String::new(),
        multiline: false,
        read_only: true,
        filter: TextInputFilter::Any,
        text: argui_text::TextStyle::default(),
        placeholder_text: argui_text::TextStyle::default(),
        selection: argui_core::Color::TRANSPARENT,
        caret: argui_ui::CaretStyle::default(),
    });
    let mut tree = UiTree::new(editor);
    let target = tree.node_id_at(0).unwrap();
    tree.selection_command(Some(target), SelectionCommand::SelectAll);
    let capabilities = tree.selection_capabilities(target);
    assert!(capabilities.editable);
    assert!(capabilities.copy);
    assert!(!capabilities.cut);
    assert!(!capabilities.paste);
    assert!(capabilities.select_all);
}

#[test]
fn selection_policy_keeps_collapsed_ranges_distinct_from_no_selection() {
    let mut tree = UiTree::new(Element::text("single").user_select(UserSelect::Text));
    let node = tree.node_id_at(0).unwrap();
    let update = tree.begin_document_selection(
        DocumentTextPoint::new(node, TextPosition::new(3, CaretAffinity::After)),
        false,
        SelectionGranularity::Character,
    );
    assert!(update.paint_changed);
    assert!(tree.document_selection().is_some());
    assert!(tree.has_document_selection());
    assert!(tree.selected_document_text().is_none());
    assert!(tree.selection_capabilities(node).copy);
    assert!(tree.selection_capabilities(node).select_all);
    assert!(
        tree.selection_command(None, SelectionCommand::Copy)
            .clipboard
            .is_none()
    );

    let released = tree.release_document_selection();
    assert!(!released.paint_changed);
    assert!(released.events.is_empty());
    assert!(tree.clear_document_selection().paint_changed);
    assert!(tree.document_selection().is_none());

    let mut restricted = UiTree::new(Element::text("hidden").user_select(UserSelect::None));
    let restricted_node = restricted.node_id_at(0).unwrap();
    assert!(
        !restricted
            .selection_capabilities(restricted_node)
            .select_all
    );
    assert!(restricted.select_all_document_text().events.is_empty());
    assert!(
        restricted
            .begin_document_selection(
                DocumentTextPoint::new(restricted_node, TextPosition::default()),
                false,
                SelectionGranularity::Word,
            )
            .events
            .is_empty()
    );
    assert!(restricted.document_selection().is_none());
}
