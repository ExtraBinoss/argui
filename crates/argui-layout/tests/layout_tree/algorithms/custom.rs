use argui_core::{Point, Rect, Size};
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{
    CustomConstraints, CustomElement, CustomLayoutContext, CustomMeasurement, CustomPaintContext,
    Element, Sides, UiTree, length,
};

#[derive(Debug)]
struct Stack;
impl CustomElement for Stack {
    type State = ();
    fn create_state(&self) {}
    fn layout_revision(&self) -> u64 {
        0
    }
    fn paint_revision(&self) -> u64 {
        0
    }
    fn prepare(&self, _: &mut (), _: Size) {}
    fn paint(&self, _: &mut (), _: &mut CustomPaintContext<'_>) {}
    fn layout(
        &self,
        _: &mut (),
        cx: &mut dyn CustomLayoutContext,
    ) -> Result<CustomMeasurement, String> {
        let mut height = 0.0;
        let mut width: f32 = 0.0;
        for index in 0..cx.child_count() {
            let measured = cx.measure_child(
                index,
                CustomConstraints {
                    width: cx.known_size().width.or(cx.available().width),
                    height: None,
                },
            )?;
            cx.place_child(index, Rect::new(Point::new(0.0, height), measured.size))?;
            height += measured.size.height + 7.0;
            width = width.max(measured.size.width);
        }
        if cx.child_count() > 0 {
            height -= 7.0;
        }
        Ok(CustomMeasurement {
            size: Size::new(width, height),
            baseline: None,
        })
    }
}

#[test]
fn custom_containers_measure_and_place_nested_standard_children() {
    let nested = Element::custom_container(
        Stack,
        [
            Element::container([])
                .keyed("first")
                .width(length(40.0))
                .height(length(20.0)),
            Element::custom_container(
                Stack,
                [Element::container([])
                    .keyed("inner")
                    .width(length(30.0))
                    .height(length(15.0))],
            )
            .keyed("nested")
            .padding(Sides::length(3.0)),
        ],
    )
    .keyed("outer")
    .padding(Sides::length(10.0));
    let mut ui = UiTree::new(nested);
    let output = LayoutEngine::new()
        .compute(&mut ui, &mut TextEngine::new(), Size::new(300.0, 200.0))
        .unwrap();
    let bounds = |key| {
        output
            .nodes
            .iter()
            .find(|node| ui.key(node.node) == Some(key))
            .unwrap()
            .bounds
    };
    assert_eq!(
        bounds("first"),
        Rect::new(Point::new(10.0, 10.0), Size::new(40.0, 20.0))
    );
    assert_eq!(bounds("nested").origin, Point::new(10.0, 37.0));
    assert_eq!(
        bounds("inner"),
        Rect::new(Point::new(13.0, 40.0), Size::new(30.0, 15.0))
    );
    assert_eq!(bounds("outer").size.height, 68.0);
}

#[derive(Debug)]
struct Invalid(usize);
impl CustomElement for Invalid {
    type State = ();
    fn create_state(&self) {}
    fn layout_revision(&self) -> u64 {
        self.0 as u64
    }
    fn paint_revision(&self) -> u64 {
        0
    }
    fn prepare(&self, _: &mut (), _: Size) {}
    fn paint(&self, _: &mut (), _: &mut CustomPaintContext<'_>) {}
    fn layout(
        &self,
        _: &mut (),
        cx: &mut dyn CustomLayoutContext,
    ) -> Result<CustomMeasurement, String> {
        let bounds = Rect::new(Point::default(), Size::new(10.0, 10.0));
        match self.0 {
            0 => {}
            1 => {
                cx.place_child(2, bounds)?;
            }
            2 => {
                cx.place_child(0, bounds)?;
                cx.place_child(0, bounds)?;
            }
            3 => {
                cx.place_child(0, Rect::new(Point::new(f32::NAN, 0.0), bounds.size))?;
            }
            _ => {
                cx.measure_child(
                    0,
                    CustomConstraints {
                        width: Some(-1.0),
                        height: None,
                    },
                )?;
            }
        }
        Ok(CustomMeasurement {
            size: bounds.size,
            baseline: None,
        })
    }
}

#[test]
fn invalid_child_layouts_fail_repeatedly_and_recover_after_replacement() {
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    let mut ui = UiTree::new(Element::container([]));
    for mode in 0..5 {
        ui.update(Element::custom_container(
            Invalid(mode),
            [Element::text("child")],
        ));
        for _ in 0..2 {
            assert!(
                engine
                    .compute(&mut ui, &mut text, Size::new(200.0, 200.0))
                    .unwrap_err()
                    .to_string()
                    .contains("custom element")
            );
        }
    }
    ui.update(Element::custom_container(Stack, [Element::text("valid")]));
    assert!(
        engine
            .compute(&mut ui, &mut text, Size::new(200.0, 200.0))
            .is_ok()
    );
}

#[derive(Debug)]
struct Overlap;
impl CustomElement for Overlap {
    type State = ();
    fn create_state(&self) {}
    fn layout_revision(&self) -> u64 {
        0
    }
    fn paint_revision(&self) -> u64 {
        0
    }
    fn prepare(&self, _: &mut (), _: Size) {}
    fn paint(&self, _: &mut (), _: &mut CustomPaintContext<'_>) {}
    fn layout(
        &self,
        _: &mut (),
        cx: &mut dyn CustomLayoutContext,
    ) -> Result<CustomMeasurement, String> {
        for index in 0..cx.child_count() {
            cx.place_child(
                index,
                Rect::new(Point::new(20.0, 10.0), Size::new(80.0, 30.0)),
            )?;
        }
        Ok(CustomMeasurement {
            size: Size::new(60.0, 50.0),
            baseline: Some(20.0),
        })
    }
}

fn regions(include_top: bool) -> Element {
    use argui_ui::*;
    let region = |key| {
        Element::custom_region(
            key,
            Interaction::default()
                .focus_policy(argui_ui::FocusPolicy::TabStop)
                .cursor(CursorIcon::EwResize)
                .gestures(
                    GestureSet::EMPTY.pan(
                        PanGesture::default()
                            .immediate()
                            .capture(GestureCapture::OnPress),
                    ),
                ),
            Semantics::new(Role::Slider)
                .label(key)
                .action(SemanticAction::Increment),
        )
        .on(EventListener::new(
            EventType::Gesture,
            EventHandlerId::new(EventOwnerId(1), 0),
        ))
    };
    let mut children = vec![region("under")];
    if include_top {
        children.push(region("top"));
    }
    Element::custom_container(Overlap, children)
        .keyed("extension")
        .width(length(60.0))
        .height(length(50.0))
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Hidden,
        })
        .transform(argui_core::Transform2D::IDENTITY.translate(30.0, 0.0))
}

#[test]
fn regions_share_clipped_hit_testing_capture_focus_and_semantic_lifetime() {
    use argui_core::{PointerEvent, PointerPhase};
    use argui_ui::{CursorIcon, GesturePhase, UiEventKind};
    let mut ui = UiTree::new(regions(true));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    let output = engine
        .compute(&mut ui, &mut text, Size::new(300.0, 200.0))
        .unwrap();
    let target = output
        .hit_regions
        .iter()
        .find(|region| ui.key(region.node) == Some("top"))
        .unwrap()
        .node;
    let point = Point::new(65.0, 20.0);
    assert_eq!(
        output
            .hit_regions
            .iter()
            .rev()
            .find(|region| region.contains(point))
            .unwrap()
            .node,
        target
    );
    assert!(
        output
            .hit_regions
            .iter()
            .all(|region| !region.contains(Point::new(100.0, 20.0)))
    );
    let pressed = PointerEvent::mouse(PointerPhase::Pressed, point);
    ui.pointer_event(pressed, &output.hit_regions);
    ui.focus_pointer_default(pressed.id, &output.hit_regions);
    assert_eq!(ui.focused_node(), Some(target));
    assert_eq!(
        ui.captured_cursor(pressed.id, &output.hit_regions),
        Some(CursorIcon::EwResize)
    );
    let moved = ui.pointer_event(
        PointerEvent::mouse(PointerPhase::Moved, Point::new(280.0, 150.0)),
        &output.hit_regions,
    );
    assert!(
        moved
            .events
            .iter()
            .any(|event| event.target_key() == Some("top")
                && matches!(
                    event.kind,
                    UiEventKind::Gesture(argui_ui::GestureEvent {
                        phase: GesturePhase::Changed,
                        ..
                    })
                ))
    );
    let before = ui.semantic_tree(&output.semantic_bounds, 2.0);
    let semantic = before
        .nodes
        .iter()
        .find(|node| node.id.get() == target.get())
        .unwrap();
    assert_eq!(semantic.semantics.label.as_deref(), Some("top"));
    assert_eq!(semantic.bounds.origin, Point::new(100.0, 20.0));
    assert_eq!(semantic.bounds.size, Size::new(80.0, 60.0));
    ui.update(regions(false));
    let output = engine
        .compute(&mut ui, &mut text, Size::new(300.0, 200.0))
        .unwrap();
    assert!(!ui.node_ids().contains(&target));
    assert_ne!(ui.focused_node(), Some(target));
    assert_eq!(ui.captured_cursor(pressed.id, &output.hit_regions), None);
    let released = ui.pointer_event(
        PointerEvent::mouse(PointerPhase::Released, point),
        &output.hit_regions,
    );
    assert!(
        released
            .events
            .iter()
            .all(|event| event.target_key() != Some("top"))
    );
    let after = ui.semantic_tree(&output.semantic_bounds, 2.0);
    assert!(after.nodes.iter().all(|node| node.id.get() != target.get()));
    assert!(!before.diff(&after).is_empty());
    assert_eq!(engine.retained_node_count(), 2);
}

#[test]
fn custom_child_identity_duplicates_are_rejected() {
    let mut root = regions(true);
    root.children[1].key = root.children[0].key.clone();
    let mut ui = UiTree::new(root);
    let error = LayoutEngine::new()
        .compute(&mut ui, &mut TextEngine::new(), Size::new(300.0, 200.0))
        .unwrap_err();
    assert!(error.to_string().contains("duplicate custom element key"));
}

#[test]
fn custom_scroll_overflow_includes_visible_descendants_but_not_clipped_ones() {
    use argui_ui::{Axes, Overflow, auto};
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    let mut ui = UiTree::new(Element::container([]));
    for (overflow, expected) in [(Overflow::Visible, 160.0), (Overflow::Hidden, 40.0)] {
        let child = Element::container([Element::container([])
            .width(length(200.0))
            .height(length(20.0))
            .absolute(Sides {
                left: length(0.0),
                top: length(0.0),
                right: auto(),
                bottom: auto(),
            })])
        .overflow(Axes {
            x: overflow,
            y: overflow,
        });
        ui.update(
            Element::custom_container(Overlap, [child])
                .width(length(60.0))
                .height(length(50.0))
                .overflow(Axes {
                    x: Overflow::Scroll,
                    y: Overflow::Hidden,
                }),
        );
        let output = engine
            .compute(&mut ui, &mut text, Size::new(300.0, 200.0))
            .unwrap();
        assert_eq!(output.scroll_regions[0].max_offset.x, expected);
        let counts = engine.custom_stats();
        let root = ui.node_ids()[0];
        ui.set_scroll_offset(root, Point::new(30.0, 0.0));
        let mut output = output;
        engine.apply_scroll(&ui, &mut output).unwrap();
        assert_eq!(
            engine.custom_stats()[0].phases.layouts,
            counts[0].phases.layouts
        );
    }
}

#[test]
fn semantic_only_changes_keep_region_identity_and_custom_phase_caches() {
    use argui_ui::{Role, Semantics, TreeUpdate};
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    let mut ui = UiTree::new(regions(true));
    let mut output = engine
        .compute(&mut ui, &mut text, Size::new(300.0, 200.0))
        .unwrap();
    let ids = ui.node_ids().to_vec();
    let stats = engine.custom_stats();
    let mut updated = regions(true);
    updated.children[1].semantics = Some(Semantics::new(Role::Slider).label("Renamed handle"));
    assert_eq!(ui.update(updated), TreeUpdate::Semantics);
    engine.repaint(&ui, &mut output);
    assert_eq!(ui.node_ids(), ids);
    assert_eq!(
        engine.custom_stats()[0].phases.layouts,
        stats[0].phases.layouts
    );
    assert_eq!(
        engine.custom_stats()[0].phases.preparations,
        stats[0].phases.preparations
    );
    assert!(
        ui.semantic_tree(&[], 1.0)
            .nodes
            .iter()
            .any(|node| node.semantics.label.as_deref() == Some("Renamed handle"))
    );
}
