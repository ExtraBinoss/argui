use argui_core::{Affine2D, Color, Point, PointerButton, PointerEvent, PointerPhase, Rect, Size};
use argui_paint::{ClipChain, Fill};
use argui_schema::{
    NativeElementInput, NativeEventValue, NativeSlotValue, ObservationKind, SchemaError,
    SchemaValue, builtin,
};
use argui_ui::{
    Current, CursorIcon, Element, EventHandler, EventHandlerId, EventOwnerId, FocusContainment,
    FocusPolicy, HitRegion, HitShape, InitialFocus, KeyboardActivation, Role, SemanticValue, Sides,
    TreeUpdate, UiTree, UserSelect,
};

#[test]
fn focus_scope_exposes_widget_root_layout_without_a_wrapper() {
    let scope = builtin::registry()
        .unwrap()
        .construct(
            builtin::FOCUS_SCOPE,
            &NativeElementInput::new()
                .property(
                    builtin::MIN_WIDTH,
                    SchemaValue::Constraint(argui_ui::LengthPercentageAuto::length(0.0)),
                )
                .property(
                    builtin::MAX_WIDTH,
                    SchemaValue::Constraint(argui_ui::LengthPercentageAuto::percent(0.8)),
                )
                .property(builtin::GROW, SchemaValue::Float(1.0))
                .property(builtin::SHRINK, SchemaValue::Float(0.0))
                .property(builtin::ALIGN_SELF, SchemaValue::String("center".into()))
                .property(
                    builtin::MARGIN,
                    SchemaValue::Insets(argui_ui::LayoutInsets::all(6.0)),
                ),
        )
        .unwrap();
    assert_eq!(
        scope.style.min_size.width,
        argui_ui::LengthPercentageAuto::length(0.0)
    );
    assert_eq!(
        scope.style.max_size.width,
        argui_ui::LengthPercentageAuto::percent(0.8)
    );
    assert_eq!(scope.style.flex_grow, 1.0);
    assert_eq!(scope.style.flex_shrink, 0.0);
    assert_eq!(scope.style.align_self, Some(argui_ui::AlignSelf::CENTER));
    assert_eq!(scope.style.margin.left, argui_ui::length(6.0));
}

#[test]
fn focus_scope_exposes_keyboard_focus_without_painting_a_control() {
    let registry = builtin::registry().unwrap();
    let scope = registry
        .construct(
            builtin::FOCUS_SCOPE,
            &NativeElementInput::new()
                .property(
                    builtin::KEYBOARD_ACTIVATION,
                    SchemaValue::String("enterOrSpace".into()),
                )
                .property(builtin::SEMANTIC_ROLE, SchemaValue::String("button".into()))
                .property(
                    builtin::SEMANTIC_LABEL,
                    SchemaValue::String("Run action".into()),
                )
                .slot(NativeSlotValue::new(
                    builtin::CHILDREN,
                    [Element::text("Run action")],
                )),
        )
        .unwrap();
    let interaction = scope.interaction.as_ref().unwrap();
    assert_eq!(interaction.focus_policy, FocusPolicy::TabStop);
    assert!(interaction.focus_on_descendant_press);
    assert_eq!(
        interaction.keyboard_activation,
        KeyboardActivation::EnterOrSpace
    );
    assert_eq!(scope.semantics.as_ref().unwrap().role, Role::Button);
    assert!(scope.paint.quad.background.is_none());
    assert_eq!(scope.children.len(), 1);
    assert_eq!(scope.user_select, UserSelect::None);
    let schema = registry.schema(builtin::FOCUS_SCOPE).unwrap();
    assert_eq!(
        schema
            .properties
            .iter()
            .find(|property| property.id == builtin::HAS_FOCUS)
            .unwrap()
            .observation,
        Some(ObservationKind::Focused)
    );
}

#[test]
fn clickable_combo_scope_prevents_text_selection_without_keyboard_activation() {
    let handler = EventHandler::from_identity(EventHandlerId::new(EventOwnerId(1), 1));
    let scope = builtin::registry()
        .unwrap()
        .construct(
            builtin::FOCUS_SCOPE,
            &NativeElementInput::new()
                .property(
                    builtin::SEMANTIC_ROLE,
                    SchemaValue::String("comboBox".into()),
                )
                .event(NativeEventValue::new(builtin::CLICK, handler))
                .slot(NativeSlotValue::new(
                    builtin::CHILDREN,
                    [Element::text("Choice")],
                )),
        )
        .unwrap();
    assert_eq!(scope.user_select, UserSelect::None);
}

/// Confirms radio groups retain their dedicated role through schema construction.
#[test]
fn radio_group_scope_exposes_its_native_semantic_role() {
    let scope = builtin::registry()
        .unwrap()
        .construct(
            builtin::FOCUS_SCOPE,
            &NativeElementInput::new()
                .property(
                    builtin::SEMANTIC_ROLE,
                    SchemaValue::String("radioGroup".into()),
                )
                .property(
                    builtin::SEMANTIC_LABEL,
                    SchemaValue::String("Display density".into()),
                ),
        )
        .unwrap();
    assert_eq!(scope.semantics.as_ref().unwrap().role, Role::RadioGroup);
}

#[test]
fn modal_focus_scope_can_target_and_restore_focus() {
    let registry = builtin::registry().unwrap();
    let scope = registry
        .construct(
            builtin::FOCUS_SCOPE,
            &NativeElementInput::new()
                .property(
                    builtin::FOCUS_CONTAINMENT,
                    SchemaValue::String("modal".into()),
                )
                .property(
                    builtin::INITIAL_FOCUS,
                    SchemaValue::String("confirm".into()),
                )
                .property(builtin::RESTORE_FOCUS, SchemaValue::Bool(false))
                .property(builtin::FOCUS_ON_TAB, SchemaValue::Bool(false)),
        )
        .unwrap();
    let focus = scope.focus_scope.as_ref().unwrap();
    assert_eq!(focus.containment, FocusContainment::Modal);
    assert!(matches!(focus.initial, Some(InitialFocus::Target(_))));
    assert!(!focus.restore);
    assert_eq!(
        scope.interaction.as_ref().unwrap().focus_policy,
        FocusPolicy::Programmatic
    );
}

#[test]
fn focus_scope_validates_policies_and_exposes_disabled_semantics() {
    let registry = builtin::registry().unwrap();
    let disabled = registry
        .construct(
            builtin::FOCUS_SCOPE,
            &NativeElementInput::new()
                .property(builtin::ENABLED, SchemaValue::Bool(false))
                .property(
                    builtin::FOCUS_CONTAINMENT,
                    SchemaValue::String("trap".into()),
                )
                .property(builtin::INITIAL_FOCUS, SchemaValue::String("first".into()))
                .property(
                    builtin::KEYBOARD_ACTIVATION,
                    SchemaValue::String("enter".into()),
                )
                .property(builtin::SEMANTIC_ROLE, SchemaValue::String("menu".into()))
                .property(builtin::BUSY, SchemaValue::Bool(true))
                .property(builtin::CHECKED, SchemaValue::Bool(false))
                .property(builtin::SEMANTIC_EXPANDABLE, SchemaValue::Bool(true))
                .property(builtin::EXPANDED, SchemaValue::Bool(true)),
        )
        .unwrap();
    assert_eq!(
        disabled.interaction.as_ref().unwrap().focus_policy,
        FocusPolicy::None
    );
    assert_eq!(
        disabled.focus_scope.as_ref().unwrap().containment,
        FocusContainment::Trap
    );
    assert_eq!(
        disabled.focus_scope.as_ref().unwrap().initial,
        Some(InitialFocus::First)
    );
    assert_eq!(disabled.semantics.as_ref().unwrap().role, Role::Menu);
    let state = &disabled.semantics.as_ref().unwrap().state;
    assert!(state.disabled);
    assert!(state.busy);
    assert_eq!(state.expanded, Some(true));

    for (property, value) in [
        (builtin::FOCUS_CONTAINMENT, "unknown"),
        (builtin::KEYBOARD_ACTIVATION, "spaceOnly"),
        (builtin::SEMANTIC_ROLE, "unknownRole"),
    ] {
        let error = registry
            .construct(
                builtin::FOCUS_SCOPE,
                &NativeElementInput::new()
                    .property(property, SchemaValue::String(value.into()))
                    .property(builtin::SEMANTIC_LABEL, SchemaValue::String("label".into())),
            )
            .unwrap_err();
        assert!(
            matches!(error, SchemaError::InvalidPropertyValue { .. }),
            "{value}"
        );
    }
}

#[test]
fn focus_scope_exposes_current_page_select_value_and_key_relations() {
    let registry = builtin::registry().unwrap();
    let target = registry
        .construct(
            builtin::FOCUS_SCOPE,
            &NativeElementInput::new()
                .property(builtin::ID, SchemaValue::String("options".into()))
                .property(
                    builtin::SEMANTIC_ROLE,
                    SchemaValue::String("listBox".into()),
                ),
        )
        .unwrap();
    let control = |current: bool| {
        let mut input = NativeElementInput::new()
            .property(builtin::ID, SchemaValue::String("select".into()))
            .property(
                builtin::SEMANTIC_ROLE,
                SchemaValue::String("comboBox".into()),
            )
            .property(
                builtin::SEMANTIC_LABEL,
                SchemaValue::String("Framework".into()),
            )
            .property(
                builtin::SEMANTIC_DESCRIPTION,
                SchemaValue::String("Choose a framework".into()),
            )
            .property(builtin::SEMANTIC_VALUE, SchemaValue::String("Solid".into()))
            .property(builtin::SEMANTIC_EXPANDABLE, SchemaValue::Bool(true))
            .property(builtin::EXPANDED, SchemaValue::Bool(true))
            .property(
                builtin::SEMANTIC_CONTROLS,
                SchemaValue::String("options".into()),
            );
        if current {
            input = input.property(
                builtin::SEMANTIC_CURRENT,
                SchemaValue::String("page".into()),
            );
        }
        registry.construct(builtin::FOCUS_SCOPE, &input).unwrap()
    };
    let mut tree = UiTree::new(Element::column([control(true), target.clone()]));
    let semantic = tree.semantic_tree(&[], 1.0);
    let select = semantic
        .nodes
        .iter()
        .find(|node| node.semantics.role == Role::ComboBox)
        .unwrap();
    let options = semantic
        .nodes
        .iter()
        .find(|node| node.semantics.role == Role::ListBox)
        .unwrap();
    assert_eq!(select.semantics.state.current, Some(Current::Page));
    assert_eq!(
        select.semantics.description.as_deref(),
        Some("Choose a framework")
    );
    assert_eq!(
        select.semantics.value,
        Some(SemanticValue::Text("Solid".into()))
    );
    assert_eq!(select.semantics.relations.controls, vec![options.id]);
    assert_eq!(
        tree.update(Element::column([control(false), target])),
        TreeUpdate::Semantics
    );
}

#[test]
fn rectangle_uses_focus_scope_hover_and_press_colors_for_buttons() {
    let registry = builtin::registry().unwrap();
    let base = Fill::Solid(Color::BLACK);
    let hover = Fill::Solid(Color::srgb(0.2, 0.4, 0.6));
    let pressed = Fill::Solid(Color::WHITE);
    let rectangle = registry
        .construct(
            builtin::RECTANGLE,
            &NativeElementInput::new()
                .property(builtin::BACKGROUND, SchemaValue::Brush(base.clone()))
                .property(builtin::HOVER_BACKGROUND, SchemaValue::Brush(hover.clone()))
                .property(
                    builtin::PRESSED_BACKGROUND,
                    SchemaValue::Brush(pressed.clone()),
                ),
        )
        .unwrap();
    let scope = registry
        .construct(
            builtin::FOCUS_SCOPE,
            &NativeElementInput::new().slot(NativeSlotValue::new(builtin::CHILDREN, [rectangle])),
        )
        .unwrap();
    let mut tree = UiTree::new(scope);
    let scope_node = tree.node_ids()[0];
    let rectangle_node = tree.node_ids()[1];
    let region = HitRegion {
        node: scope_node,
        bounds: Rect::new(Point::default(), Size::new(100.0, 40.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        shape: HitShape::Bounds,
        slop: Sides::default(),
        enabled: true,
        focus_policy: FocusPolicy::TabStop,
        cursor: CursorIcon::Default,
        gestures: Default::default(),
        window_drag: None,
    };
    let color = |tree: &UiTree| {
        tree.resolved_quad(rectangle_node, tree.element_for(rectangle_node).unwrap())
            .background
    };
    assert_eq!(color(&tree), Some(base));
    tree.pointer_moved(Point::new(10.0, 10.0), std::slice::from_ref(&region));
    assert_eq!(color(&tree), Some(hover));
    tree.pointer_event(
        PointerEvent {
            button: Some(PointerButton::Primary),
            buttons: 1,
            ..PointerEvent::mouse(PointerPhase::Pressed, Point::new(10.0, 10.0))
        },
        &[region],
    );
    assert_eq!(color(&tree), Some(pressed));
}
