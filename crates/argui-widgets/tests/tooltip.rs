use argui_core::{
    Color, ColorScheme, Key, KeyInput, KeyState, Modifiers, Point, PointerEvent, PointerKind,
    PointerPhase,
};
use argui_paint::{EffectId, EffectInstance, Filter, LayerStyle, PaintStyle, QuadStyle};
use argui_ui::{
    ClickEvent, Element, FloatingPlacement, Placement, Role, UiEvent, UiEventKind, UiTree,
};
use argui_widgets::{Tooltip, TooltipState, shadcn};
use std::time::Duration;

#[path = "tooltip/host.rs"]
mod host;

fn event(key: &str, kind: UiEventKind) -> UiEvent {
    let tree = UiTree::new(Element::container([]));
    UiEvent::new(tree.node_ids()[0], Some(key.into()), kind)
}

fn pointer(key: &str, phase: PointerPhase) -> UiEvent {
    event(
        key,
        UiEventKind::Pointer(PointerEvent::mouse(phase, Point::default())),
    )
}

fn ms(value: u64) -> Duration {
    Duration::from_millis(value)
}

#[test]
fn hover_delay_cancels_fast_passes_and_allows_crossing_into_the_content() {
    let mut state = TooltipState::new("tip");
    assert!(state.update(&pointer("tip", PointerPhase::Entered), ms(0)));
    assert_eq!(state.next_deadline(), Some(ms(350)));
    assert!(!state.advance(ms(349)));
    assert!(state.update(&pointer("tip", PointerPhase::Left), ms(100)));
    assert_eq!(state.next_deadline(), None);
    assert!(!state.advance(ms(400)));
    state.update(&pointer("tip", PointerPhase::Entered), ms(500));
    assert!(state.advance(ms(850)));
    assert!(state.is_open());
    state.update(&pointer("tip", PointerPhase::Left), ms(900));
    assert_eq!(state.next_deadline(), Some(ms(1000)));
    state.update(&pointer("tip::content", PointerPhase::Entered), ms(950));
    assert_eq!(state.next_deadline(), None);
    assert!(!state.advance(ms(1000)));
    assert!(state.is_open());
    state.update(&pointer("tip::content", PointerPhase::Left), ms(1100));
    assert!(state.advance(ms(1200)));
    assert!(!state.is_open());
}

#[test]
fn keyboard_focus_is_immediate_and_escape_suppresses_reopening_until_reentry() {
    let mut state = TooltipState::new("tip");
    state.update(&event("tip", UiEventKind::Focused), ms(0));
    assert!(state.is_open());
    for (key, key_state, expected) in [
        (Key::Enter, KeyState::Pressed, false),
        (Key::Escape, KeyState::Released, false),
        (Key::Escape, KeyState::Pressed, true),
    ] {
        assert_eq!(
            state.update(
                &event(
                    "tip",
                    UiEventKind::KeyInput(KeyInput {
                        key,
                        state: key_state,
                        modifiers: Modifiers::default(),
                        repeat: false,
                        text: None,
                    })
                ),
                ms(10)
            ),
            expected
        );
    }
    assert!(!state.is_open());
    state.update(&pointer("tip", PointerPhase::Entered), ms(20));
    assert_eq!(state.next_deadline(), Some(ms(370)));
    state.update(&pointer("tip", PointerPhase::Left), ms(30));
    state.update(&event("tip", UiEventKind::Blurred), ms(40));
    state.update(&event("tip", UiEventKind::Focused), ms(50));
    assert!(state.is_open());
    state.update(&event("tip", UiEventKind::Blurred), ms(60));
    assert!(state.advance(ms(160)));
}

#[test]
fn activation_dismisses_without_eating_button_actions_and_unrelated_events_are_ignored() {
    let mut state = TooltipState::new("tip").delay(Duration::ZERO);
    assert!(!state.update(&pointer("other", PointerPhase::Entered), ms(0)));
    let mut touch = PointerEvent::mouse(PointerPhase::Entered, Point::default());
    touch.kind = PointerKind::Touch;
    assert!(!state.update(&event("tip", UiEventKind::Pointer(touch)), ms(0)));
    state.update(&pointer("tip", PointerPhase::Entered), ms(0));
    assert!(state.is_open());
    assert!(!state.update(&pointer("tip", PointerPhase::Moved), ms(1)));
    state.update(&pointer("tip", PointerPhase::Pressed), ms(2));
    state.update(&event("tip", UiEventKind::Focused), ms(3));
    assert!(!state.is_open());
    state.update(&pointer("tip", PointerPhase::Cancelled), ms(4));
    state.update(&event("tip", UiEventKind::Blurred), ms(5));
    state.update(&event("tip", UiEventKind::Focused), ms(6));
    let click = event("tip", UiEventKind::Click(ClickEvent::accessibility()));
    assert!(state.update(&click, ms(7)));
    assert!(!state.is_open());
    assert!(click.should_dispatch());
}

#[test]
fn hover_reopens_after_click_without_requiring_the_button_to_lose_focus() {
    let mut state = TooltipState::new("tip");
    state.update(&pointer("tip", PointerPhase::Entered), ms(0));
    assert!(state.advance(ms(350)));
    state.update(&pointer("tip", PointerPhase::Pressed), ms(360));
    state.update(&event("tip", UiEventKind::Focused), ms(361));
    state.update(
        &event("tip", UiEventKind::Click(ClickEvent::accessibility())),
        ms(362),
    );
    assert!(!state.is_open());
    state.update(&pointer("tip", PointerPhase::Left), ms(400));
    assert!(!state.advance(ms(1000)));
    assert!(!state.is_open());
    state.update(&pointer("tip", PointerPhase::Entered), ms(1100));
    assert_eq!(state.next_deadline(), Some(ms(1450)));
    assert!(state.advance(ms(1450)));
    assert!(state.is_open());
}

#[test]
fn tooltip_keeps_trigger_semantics_and_uses_a_described_nonfocusable_portal() {
    let themes = shadcn(Color::BLACK);
    let theme = themes.resolve(ColorScheme::Light);
    for open in [false, true] {
        let root = Tooltip::new(
            "tip",
            "Save your changes",
            open,
            argui_widgets::Button::new("tip", "Save", theme.button()).build(),
        )
        .build(theme);
        assert_eq!(root.children.len(), if open { 2 } else { 1 });
        let trigger = &root.children[0];
        let semantics = trigger.semantics.as_ref().unwrap();
        assert_eq!(semantics.label.as_deref(), Some("Save"));
        assert_eq!(semantics.role, Role::Button);
        assert_eq!(semantics.description.as_deref(), Some("Save your changes"));
        assert_eq!(
            trigger.semantic_bindings.described_by.len(),
            usize::from(open)
        );
        let mut ui = UiTree::new(root.clone());
        let output = argui_layout::LayoutEngine::new()
            .compute(
                &mut ui,
                &mut argui_text::TextEngine::new(),
                argui_core::Size::new(400.0, 240.0),
            )
            .unwrap();
        assert!(ui.semantic_diagnostics().is_empty());
        assert_eq!(ui.focused_node(), None);
        if open {
            let panel = &root.children[1];
            assert_eq!(panel.semantics.as_ref().unwrap().role, Role::Tooltip);
            assert!(
                !panel
                    .interaction
                    .as_ref()
                    .unwrap()
                    .focus_policy
                    .is_focusable()
            );
            assert!(panel.focus_scope.is_none());
            assert!(panel.layer.as_ref().unwrap().backdrop_filters.is_empty());
            assert!(
                output
                    .nodes
                    .iter()
                    .all(|node| node.bounds.origin.x.is_finite())
            );
        }
    }
}

#[test]
fn custom_surface_accepts_registered_effects_and_clamps_width() {
    let themes = shadcn(Color::BLACK);
    let theme = themes.resolve(ColorScheme::Dark);
    let layer = LayerStyle::new(Default::default())
        .backdrop(Filter::Blur(12.0))
        .filter(Filter::Effect(EffectInstance::new(
            EffectId::new("test.tooltip"),
            [("strength", argui_paint::EffectValue::F32(0.5))],
        )));
    let paint = PaintStyle::new(QuadStyle::solid(Color::BLACK));
    let root = Tooltip::new("tip", "Help", true, Element::text("Hover"))
        .placement(FloatingPlacement::new(Placement::BottomEnd))
        .max_width(-1.0)
        .paint(paint.clone())
        .layer(layer.clone())
        .build(theme);
    assert_eq!(root.children[1].layer.as_deref(), Some(&layer));
    assert_eq!(root.children[1].paint, paint);
    assert_eq!(root.children[1].style.max_size.width, argui_ui::length(0.0));
    assert!(
        root.children[0]
            .interaction
            .as_ref()
            .unwrap()
            .focus_policy
            .is_focusable()
    );
}

#[test]
fn long_tooltip_descriptions_wrap_without_clipping_the_last_line() {
    let description = "See the previous versions of this project.";
    let themes = shadcn(Color::BLACK);
    let root = Tooltip::new("tip", description, true, Element::text("History"))
        .max_width(236.0)
        .build(themes.resolve(ColorScheme::Light));
    let mut ui = UiTree::new(root);
    let font = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");
    let mut text = argui_text::TextEngine::from_embedded_fonts(
        [font.as_slice()],
        "Noto Sans",
        "Noto Sans",
        "Noto Sans",
    );
    let output = argui_layout::LayoutEngine::new()
        .compute(&mut ui, &mut text, argui_core::Size::new(400.0, 240.0))
        .unwrap();
    let block = output
        .text
        .blocks()
        .iter()
        .find(|block| block.content.as_str() == description)
        .unwrap();
    let measured = text.measure(description, &block.style, Some(block.bounds.size.width));
    assert!(
        measured.height <= block.bounds.size.height,
        "{measured:?} must fit {:?}",
        block.bounds
    );
    assert!(measured.height > block.style.line_height);
}

#[test]
fn oversized_tooltip_scrolls_in_the_fallback_viewport_and_accepts_native_dismissal() {
    use argui_core::Size;
    use argui_ui::{OverlaySurface, ScrollPropagation};
    let palette = shadcn(Color::WHITE);
    let theme = palette.resolve(ColorScheme::Light);
    let description =
        "A complete description remains available even when this window is tiny. ".repeat(30);
    let mut ui = UiTree::new(
        Tooltip::new("scroll-tip", &description, true, Element::text("Help"))
            .surface(OverlaySurface::PreferNative)
            .build(theme),
    );
    let layout = argui_layout::LayoutEngine::new()
        .compute(
            &mut ui,
            &mut argui_text::TextEngine::new(),
            Size::new(220.0, 140.0),
        )
        .unwrap();
    let panel = layout.portals.first().unwrap().node;
    assert_eq!(
        ui.portal_surface_preference(panel),
        OverlaySurface::PreferNative
    );
    assert_eq!(ui.native_portal_owner(panel), None);
    assert!(layout.native_surfaces.is_empty());
    let scroll = layout
        .scroll_regions
        .iter()
        .find(|region| region.node == panel)
        .unwrap();
    assert!(scroll.max_offset.y > 0.0);
    assert_eq!(scroll.config.propagation, ScrollPropagation::Contain);
    assert!(
        scroll
            .scrollbar
            .as_ref()
            .and_then(|bars| bars.vertical)
            .is_some()
    );
    assert_eq!(
        ui.root().children[0]
            .semantics
            .as_ref()
            .unwrap()
            .description
            .as_deref(),
        Some(description.as_str())
    );
    let mut state = TooltipState::new("scroll-tip").delay(Duration::ZERO);
    state.update(&pointer("scroll-tip", PointerPhase::Entered), ms(0));
    assert!(state.is_open());
    state.update(
        &event("scroll-tip::content", UiEventKind::DismissRequested),
        ms(1),
    );
    assert!(!state.is_open());
}
