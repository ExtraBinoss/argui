use argui_core::{Affine2D, Color, ColorScheme, Point, Rect, Size};
use argui_paint::{ClipChain, PaintStyle, QuadStyle};
use argui_text::{TextStyle, TextWrap};
use argui_ui::{
    CursorIcon, Element, ElementKind, HitRegion, KeyboardActivation, Role, UiEvent, UiEventKind,
    UiTree, UserSelect, VisualState,
};
use argui_widgets::{
    Button, ButtonAction, ButtonBehavior, ButtonStyle, TablerIcon, WidgetAssets, shadcn,
};

#[test]
fn button_exposes_variants_content_and_busy_state() {
    let theme = shadcn(Color::srgb(0.2, 0.5, 0.9));
    let theme = theme.resolve(ColorScheme::Dark);
    let assets = WidgetAssets::tabler(theme.foreground);
    let button = Button::new("save", "Save", theme.button.clone())
        .leading(assets.icon(TablerIcon::Check, 16.0))
        .trailing(Element::text("⌘S"))
        .build();
    assert_eq!(button.key.as_deref(), Some("save"));
    assert_eq!(button.user_select, UserSelect::None);
    assert_eq!(button.children.len(), 3);
    assert_eq!(button.semantics.as_ref().unwrap().role, Role::Button);
    assert_eq!(
        button.interaction.as_ref().unwrap().keyboard_activation,
        KeyboardActivation::EnterOrSpace
    );

    let loading = Button::new("load", "Loading", theme.button.clone())
        .loading(Element::text("spinner"))
        .build();
    let semantics = loading.semantics.as_ref().unwrap();
    assert!(semantics.state.busy);
    assert!(semantics.state.disabled);
    assert!(!loading.interaction.as_ref().unwrap().enabled);
    assert_eq!(
        loading.interaction.as_ref().unwrap().cursor,
        CursorIcon::Progress
    );

    let disabled = Button::new("disabled", "Disabled", theme.button.clone())
        .enabled(false)
        .build();
    assert_eq!(
        disabled.interaction.as_ref().unwrap().cursor,
        CursorIcon::NotAllowed
    );
}

#[test]
fn button_focus_style_is_explicitly_opt_in() {
    let paint = PaintStyle::new(QuadStyle::solid(Color::srgb(0.0, 0.0, 0.0)));
    let default = ButtonStyle::new(paint.clone(), TextStyle::default());
    assert!(default.focused.is_none());

    let focused =
        ButtonStyle::new(paint, TextStyle::default()).focused(QuadStyle::solid(Color::WHITE));
    assert!(focused.focused.is_some());
}

#[test]
fn button_hover_uses_the_shared_retained_visual_state_path() {
    let resting = QuadStyle::solid(Color::srgb(0.0, 0.0, 0.0));
    let hovered = QuadStyle::solid(Color::WHITE);
    let element = Button::new(
        "save",
        "Save",
        ButtonStyle::new(PaintStyle::new(resting.clone()), TextStyle::default()).hovered(hovered),
    )
    .build();
    let mut tree = UiTree::new(element.clone());
    let node = tree.node_ids()[0];
    tree.pointer_moved(
        Point::new(2.0, 2.0),
        &[HitRegion {
            node,
            bounds: Rect::new(Point::default(), Size::new(20.0, 20.0)),
            transform: Affine2D::IDENTITY,
            clips: ClipChain::default(),
            shape: argui_ui::HitShape::Bounds,
            slop: argui_ui::HitTestStyle::default().slop,
            enabled: true,
            focusable: true,
            cursor: CursorIcon::Pointer,
            gestures: argui_ui::GestureSet::NONE,
            window_drag: None,
        }],
    );
    assert!(tree.visual_states(node).contains(VisualState::Hovered));
    assert!(matches!(
        &element.children[0].kind,
        ElementKind::Text { style, .. } if style.wrap == TextWrap::None
    ));
}

#[test]
fn headless_button_decodes_only_enabled_activation() {
    let tree = UiTree::new(Element::container([]));
    let clicked = UiEvent::new(
        tree.node_ids()[0],
        Some("save".into()),
        UiEventKind::Clicked,
    );
    assert_eq!(
        ButtonBehavior::new("save", "Save").action(&clicked),
        Some(ButtonAction::Activate)
    );
    assert_eq!(
        ButtonBehavior::new("save", "Save")
            .enabled(false)
            .action(&clicked),
        None
    );
}
