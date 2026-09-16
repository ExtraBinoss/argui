use argui_core::{Point, Rect, Size};
use argui_layout::LayoutEngine;
use argui_paint::{Color, DisplayCommand, GpuCanvasId, QuadStyle};
use argui_text::TextEngine;
use argui_ui::{
    CustomElement, CustomLayoutContext, CustomMeasurement, CustomPaintContext, Element,
    GpuCanvasSpec, TreeUpdate, UiTree,
};
use std::{cell::Cell, rc::Rc};

#[derive(Debug, Default)]
struct Counts {
    created: Cell<usize>,
    dropped: Cell<usize>,
    measured: Cell<usize>,
    painted: Cell<usize>,
}
struct State(Rc<Counts>);
impl Drop for State {
    fn drop(&mut self) {
        self.0.dropped.set(self.0.dropped.get() + 1);
    }
}
#[derive(Debug, Clone)]
struct Tile {
    counts: Rc<Counts>,
    width: f32,
    color: Color,
}
impl CustomElement for Tile {
    type State = State;
    fn prepare(&self, _: &mut State, _: Size) {}
    fn create_state(&self) -> State {
        self.counts.created.set(self.counts.created.get() + 1);
        State(self.counts.clone())
    }
    fn layout_revision(&self) -> u64 {
        u64::from(self.width.to_bits())
    }
    fn paint_revision(&self) -> u64 {
        u64::from(u32::from_le_bytes(self.color.to_srgba8()))
    }
    fn layout(
        &self,
        state: &mut State,
        _: &mut dyn CustomLayoutContext,
    ) -> Result<CustomMeasurement, String> {
        state.0.measured.set(state.0.measured.get() + 1);
        Ok(CustomMeasurement {
            size: Size::new(self.width, 24.0),
            baseline: Some(16.0),
        })
    }
    fn paint(&self, state: &mut State, cx: &mut CustomPaintContext<'_>) {
        state.0.painted.set(state.0.painted.get() + 1);
        cx.quad(
            Rect::new(Point::default(), cx.bounds.size),
            QuadStyle::solid(self.color),
        );
    }
}
fn tile(counts: &Rc<Counts>, width: f32, color: Color) -> Element {
    Element::custom(Tile {
        counts: counts.clone(),
        width,
        color,
    })
    .keyed("tile")
}

#[derive(Debug)]
struct CanvasTile {
    canvas: GpuCanvasId,
}

impl CustomElement for CanvasTile {
    type State = ();

    fn create_state(&self) {}

    fn layout_revision(&self) -> u64 {
        0
    }

    fn paint_revision(&self) -> u64 {
        3
    }

    fn prepare(&self, _: &mut (), _: Size) {}

    fn layout(
        &self,
        _: &mut (),
        _: &mut dyn CustomLayoutContext,
    ) -> Result<CustomMeasurement, String> {
        Ok(CustomMeasurement {
            size: Size::new(80.0, 40.0),
            baseline: None,
        })
    }

    fn paint(&self, _: &mut (), context: &mut CustomPaintContext<'_>) {
        context.gpu_canvas(
            2,
            Rect::new(Point::new(4.0, 6.0), Size::new(30.0, 20.0)),
            GpuCanvasSpec::new(self.canvas).content_revision(9),
        );
        context.gpu_canvas(
            7,
            Rect::new(Point::new(40.0, 6.0), Size::new(30.0, 20.0)),
            GpuCanvasSpec::new(self.canvas).content_revision(10),
        );
    }
}

#[test]
fn custom_paint_gpu_canvases_share_retained_object_and_keep_local_slots() {
    let canvas = GpuCanvasId::fresh();
    let element = Element::custom(CanvasTile { canvas })
        .width(argui_ui::length(80.0))
        .height(argui_ui::length(40.0))
        .paint_opacity(0.5)
        .radius(argui_ui::CornerRadii::all(5.0));
    let mut ui = UiTree::new(element);
    let object = ui.node_ids()[0];
    let output = LayoutEngine::new()
        .compute(&mut ui, &mut TextEngine::new(), Size::new(100.0, 60.0))
        .unwrap();
    let canvases = output
        .display_list
        .commands()
        .iter()
        .filter_map(|command| match command {
            DisplayCommand::GpuCanvas(canvas) => Some(canvas),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(canvases.len(), 2);
    assert_eq!([canvases[0].slot, canvases[1].slot], [2, 7]);
    assert_eq!(canvases[0].object.value, object.get());
    assert_eq!(canvases[1].object, canvases[0].object);
    assert_eq!(canvases[0].bounds.origin, Point::new(4.0, 6.0));
    assert_eq!(canvases[0].opacity, 0.5);
    assert_eq!(canvases[0].radii, argui_ui::CornerRadii::all(5.0));
}
#[test]
fn custom_measurement_paint_invalidation_and_state_lifetime() {
    let counts = Rc::new(Counts::default());
    let mut ui = UiTree::new(tile(&counts, 80.0, Color::WHITE));
    let id = ui.node_ids()[0];
    let mut layout = LayoutEngine::new();
    let mut text = TextEngine::new();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(200.0, 100.0))
        .unwrap();
    assert_eq!(output.nodes[0].bounds.size, Size::new(200.0, 24.0));
    assert_eq!(counts.created.get(), 1);
    let measured = counts.measured.get();
    let initial = layout.custom_stats()[0].phases;
    assert_eq!(initial.layouts, measured as u64);
    assert_eq!(initial.preparations, 1);
    assert_eq!(initial.paints, 1);
    layout.repaint(&ui, &mut output);
    assert_eq!(
        layout.custom_stats()[0].phases,
        initial,
        "idle repaint reuses the fragment"
    );
    assert_eq!(
        ui.update(tile(&counts, 80.0, Color::BLACK)),
        TreeUpdate::Paint
    );
    assert_eq!(ui.node_ids()[0], id);
    layout.repaint(&ui, &mut output);
    assert_eq!(counts.measured.get(), measured);
    let painted = layout.custom_stats()[0].phases;
    assert_eq!(painted.layouts, initial.layouts);
    assert_eq!(painted.preparations, initial.preparations + 1);
    assert_eq!(painted.paints, initial.paints + 1);
    assert!(output.display_list.commands().iter().any(|command| matches!(command, DisplayCommand::Quad(quad) if quad.background == Some(argui_paint::Fill::Solid(Color::BLACK)))));
    assert_eq!(
        ui.update(tile(&counts, 120.0, Color::BLACK)),
        TreeUpdate::Layout
    );
    layout
        .compute(&mut ui, &mut text, Size::new(200.0, 100.0))
        .unwrap();
    assert_eq!(counts.created.get(), 1);
    assert!(counts.measured.get() > measured);
    ui.update(Element::container([]));
    layout
        .compute(&mut ui, &mut text, Size::new(200.0, 100.0))
        .unwrap();
    assert_eq!(counts.dropped.get(), 1);
    assert!(layout.custom_stats().is_empty());
    assert_eq!(
        Rc::strong_count(&counts),
        1,
        "removed properties must be released too"
    );
}
#[test]
fn each_window_owns_its_custom_state_and_invalid_sizes_are_errors() {
    let counts = Rc::new(Counts::default());
    for width in [80.0, f32::NAN, -1.0, f32::INFINITY] {
        let mut ui = UiTree::new(tile(&counts, width, Color::WHITE));
        let mut layout = LayoutEngine::new();
        let result = layout.compute(&mut ui, &mut TextEngine::new(), Size::new(200.0, 100.0));
        assert_eq!(result.is_ok(), width == 80.0);
        if let Err(error) = result {
            assert!(error.to_string().contains("Tile"));
            assert!(
                layout
                    .compute(&mut ui, &mut TextEngine::new(), Size::new(200.0, 100.0))
                    .is_err()
            );
        }
    }
    assert_eq!(counts.created.get(), 4);
    assert_eq!(counts.dropped.get(), 4);
}

#[test]
fn duplicate_custom_keys_are_rejected_before_creating_state() {
    let counts = Rc::new(Counts::default());
    let root = Element::row([
        tile(&counts, 80.0, Color::WHITE),
        tile(&counts, 80.0, Color::BLACK),
    ]);
    let mut ui = UiTree::new(root);
    let error = LayoutEngine::new()
        .compute(&mut ui, &mut TextEngine::new(), Size::new(200.0, 100.0))
        .unwrap_err();
    assert!(error.to_string().contains("custom element"));
    assert_eq!(counts.created.get(), 0);
}

#[test]
fn keyed_reordering_preserves_state_and_repeated_removal_releases_every_instance() {
    let counts = Rc::new(Counts::default());
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    let mut ui = UiTree::new(Element::container([]));
    for cycle in 0..1_000 {
        let children = ["first", "second"].map(|key| tile(&counts, 40.0, Color::WHITE).keyed(key));
        ui.update(Element::row(children.clone()));
        engine
            .compute(&mut ui, &mut text, Size::new(200.0, 100.0))
            .unwrap();
        let first = ui.node_ids()[1];
        let second = ui.node_ids()[2];
        ui.update(Element::row(children.into_iter().rev()));
        engine
            .compute(&mut ui, &mut text, Size::new(200.0, 100.0))
            .unwrap();
        assert_eq!(ui.node_ids()[1], second);
        assert_eq!(ui.node_ids()[2], first);
        assert_eq!(counts.created.get(), (cycle + 1) * 2);
        ui.update(Element::container([]));
        engine
            .compute(&mut ui, &mut text, Size::new(200.0, 100.0))
            .unwrap();
        assert_eq!(counts.created.get(), counts.dropped.get());
        assert!(engine.custom_stats().is_empty());
        assert_eq!(engine.retained_node_count(), 1);
    }
}

#[test]
fn changing_a_sibling_reuses_custom_commands_but_geometry_and_revisions_invalidate_them() {
    use argui_ui::{length, sides};
    let counts = Rc::new(Counts::default());
    let scene = |color, padding, width| {
        Element::row([
            tile(&counts, width, Color::WHITE),
            Element::container([])
                .width(length(10.0))
                .height(length(24.0))
                .background(color),
        ])
        .padding(sides(padding, 0.0))
    };
    let mut ui = UiTree::new(scene(Color::WHITE, 0.0, 80.0));
    let mut layout = LayoutEngine::new();
    let mut text = TextEngine::new();
    let first = layout
        .compute(&mut ui, &mut text, Size::new(240.0, 80.0))
        .unwrap();
    assert_eq!(counts.painted.get(), 1);
    ui.update(scene(Color::BLACK, 0.0, 80.0));
    let second = layout
        .compute(&mut ui, &mut text, Size::new(240.0, 80.0))
        .unwrap();
    assert_eq!(
        counts.painted.get(),
        1,
        "sibling repaint must reuse custom commands"
    );
    assert_ne!(first.display_list, second.display_list);
    ui.update(scene(Color::BLACK, 5.0, 80.0));
    let moved = layout
        .compute(&mut ui, &mut text, Size::new(240.0, 80.0))
        .unwrap();
    assert_eq!(
        counts.painted.get(),
        2,
        "absolute bounds change the recorded commands"
    );
    assert_ne!(moved.display_list, second.display_list);
    ui.update(scene(Color::BLACK, 5.0, 100.0));
    layout
        .compute(&mut ui, &mut text, Size::new(240.0, 80.0))
        .unwrap();
    assert_eq!(counts.painted.get(), 3);
}
