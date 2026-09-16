use argui_core::{Color, Point, Rect, Size};
use argui_layout::LayoutEngine;
use argui_paint::{DisplayCommand, Fill, GpuCanvasId, LayerStyle};
use argui_text::TextEngine;
use argui_ui::{
    Axes, Element, FloatingPlacement, GpuCanvasSpec, Interaction, NodeId, Overflow, OverlaySurface,
    Placement, ScrollConfig, UiTree, WindowLayer, length,
};

fn node(ui: &UiTree, key: &str) -> NodeId {
    *ui.node_ids()
        .iter()
        .find(|node| ui.key(**node) == Some(key))
        .unwrap()
}
fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect::new(Point::new(x, y), Size::new(width, height))
}
fn panel(key: &str, anchor: &str, children: impl IntoIterator<Item = Element>) -> Element {
    Element::column(children)
        .keyed(key)
        .width(length(260.0))
        .height(length(240.0))
        .background(Color::WHITE)
        .interaction(Interaction::blocker())
        .overflow(Axes {
            x: Overflow::Auto,
            y: Overflow::Auto,
        })
        .scroll_config(ScrollConfig::default())
        .anchored_portal(
            WindowLayer::Popover,
            anchor,
            FloatingPlacement::new(Placement::RightStart),
        )
        .portal_surface(OverlaySurface::PreferNative)
}

#[test]
fn native_acceptance_restores_natural_size_splits_paint_and_keeps_global_hits() {
    let mut ui = UiTree::new(Element::column([
        Element::container([])
            .keyed("anchor")
            .width(length(40.0))
            .height(length(30.0))
            .background(Color::BLACK),
        panel(
            "panel",
            "anchor",
            [Element::text("Native text")
                .height(length(300.0))
                .shrink(0.0)],
        )
        .layer(LayerStyle::new(Rect::default()).opacity(0.9)),
    ]));
    let panel = node(&ui, "panel");
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    let viewport = Size::new(160.0, 120.0);
    let internal = engine.compute(&mut ui, &mut text, viewport).unwrap();
    let metadata = internal
        .portals
        .iter()
        .find(|portal| portal.node == panel)
        .unwrap();
    assert_eq!(metadata.desired_size, Size::new(260.0, 240.0));
    assert!(metadata.bounds.size.width < metadata.desired_size.width);
    assert!(internal.native_surfaces.is_empty());
    let bounds = internal
        .native_portal_placement(&ui, panel, rect(-200.0, -100.0, 1000.0, 800.0))
        .unwrap();
    assert_eq!(bounds.size, Size::new(260.0, 240.0));
    assert!((bounds.origin.x + bounds.size.width) > viewport.width);
    ui.set_native_portal(panel, Some(bounds));
    let native = engine.compute(&mut ui, &mut text, viewport).unwrap();
    assert_eq!(native.native_surfaces.len(), 1);
    let surface = &native.native_surfaces[0];
    assert_eq!(surface.node, panel);
    assert_eq!(surface.bounds, bounds);
    assert!(!native.display_list.commands().iter().any(|command| matches!(command, DisplayCommand::Quad(quad) if quad.background == Some(Fill::Solid(Color::WHITE)))));
    assert!(surface.display_list.commands().iter().any(|command| matches!(command, DisplayCommand::Quad(quad) if quad.background == Some(Fill::Solid(Color::WHITE)))));
    assert!(surface.display_list.commands().iter().any(|command| matches!(command, DisplayCommand::Text { transform, .. } if transform.translation == Point::new(-bounds.origin.x, -bounds.origin.y))));
    let outside = Point::new(
        (bounds.origin.x + bounds.size.width) - 10.0,
        bounds.origin.y + 10.0,
    );
    assert!(
        native
            .hit_regions
            .iter()
            .any(|hit| hit.node == panel && hit.contains(outside))
    );
    let scroll = native
        .scroll_regions
        .iter()
        .find(|region| region.node == panel)
        .unwrap();
    assert_eq!(scroll.bounds, bounds);
    assert!(scroll.max_offset.y > 0.0);
    ui.set_native_portal(panel, None);
    let fallback = engine.compute(&mut ui, &mut text, viewport).unwrap();
    assert!(fallback.native_surfaces.is_empty());
    assert_eq!(fallback.portals[0].bounds, internal.portals[0].bounds);
}

#[test]
fn internal_child_of_native_popup_clips_to_parent_and_can_be_promoted_separately() {
    let child = panel("child", "child-anchor", [Element::text("Nested")])
        .portal_surface(OverlaySurface::InWindow);
    let mut ui = UiTree::new(Element::column([
        Element::container([])
            .keyed("anchor")
            .width(length(40.0))
            .height(length(30.0)),
        panel(
            "parent",
            "anchor",
            [
                Element::container([])
                    .keyed("child-anchor")
                    .width(length(40.0))
                    .height(length(30.0)),
                child,
            ],
        ),
    ]));
    let parent = node(&ui, "parent");
    let child = node(&ui, "child");
    let parent_bounds = rect(400.0, 40.0, 260.0, 240.0);
    ui.set_native_portal(parent, Some(parent_bounds));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    let viewport = Size::new(160.0, 120.0);
    let output = engine.compute(&mut ui, &mut text, viewport).unwrap();
    assert_eq!(output.native_surfaces.len(), 1);
    let child_bounds = output
        .portals
        .iter()
        .find(|portal| portal.node == child)
        .unwrap()
        .bounds;
    assert!(child_bounds.origin.x >= parent_bounds.origin.x);
    assert!(
        (child_bounds.origin.x + child_bounds.size.width)
            <= (parent_bounds.origin.x + parent_bounds.size.width)
    );
    assert!(
        (child_bounds.origin.y + child_bounds.size.height)
            <= (parent_bounds.origin.y + parent_bounds.size.height)
    );
    let desired = output
        .native_portal_placement(&ui, child, rect(0.0, 0.0, 1600.0, 1000.0))
        .unwrap();
    ui.set_native_portal(child, Some(desired));
    let native = engine.compute(&mut ui, &mut text, viewport).unwrap();
    assert_eq!(native.native_surfaces.len(), 2);
    assert_eq!(
        native
            .portals
            .iter()
            .find(|portal| portal.node == child)
            .unwrap()
            .bounds,
        desired
    );
    assert!(
        native
            .native_surfaces
            .iter()
            .all(|surface| !surface.display_list.is_empty())
    );
    assert!(
        native
            .native_portal_placement(&ui, ui.node_ids()[0], parent_bounds)
            .is_none()
    );
}

#[test]
fn missing_or_hidden_anchors_leave_no_paint_or_hit_targets() {
    for hidden in [false, true] {
        let anchor = if hidden {
            Element::text("Hidden anchor")
                .keyed("anchor")
                .display(argui_ui::Display::None)
        } else {
            Element::text("Different anchor").keyed("other")
        };
        let mut ui = UiTree::new(Element::column([
            anchor,
            panel("panel", "anchor", [Element::text("Unavailable")]),
        ]));
        let panel = node(&ui, "panel");
        let mut engine = LayoutEngine::new();
        let mut text = TextEngine::new();
        let output = engine
            .compute(&mut ui, &mut text, Size::new(400.0, 300.0))
            .unwrap();
        assert!(output.portals.is_empty());
        assert!(output.native_surfaces.is_empty());
        assert!(!output.hit_regions.iter().any(|hit| hit.node == panel));
        assert!(
            output
                .native_portal_placement(&ui, panel, rect(0.0, 0.0, 1000.0, 800.0))
                .is_none()
        );
    }
}

#[test]
fn native_image_vector_and_effect_clips_share_the_same_local_coordinates() {
    use argui_paint::{ImageId, VectorId};
    let mut ui = UiTree::new(Element::column([
        Element::text("Anchor").keyed("anchor"),
        panel(
            "panel",
            "anchor",
            [
                Element::image(ImageId::fresh())
                    .width(length(48.0))
                    .height(length(40.0)),
                Element::vector(VectorId::fresh())
                    .width(length(32.0))
                    .height(length(32.0)),
                Element::gpu_canvas(GpuCanvasSpec::new(GpuCanvasId::fresh()))
                    .width(length(28.0))
                    .height(length(24.0)),
            ],
        )
        .layer(LayerStyle::new(Rect::default()).opacity(0.8)),
    ]));
    let owner = node(&ui, "panel");
    let bounds = rect(-120.0, 300.0, 260.0, 240.0);
    ui.set_native_portal(owner, Some(bounds));
    let output = LayoutEngine::new()
        .compute(&mut ui, &mut TextEngine::new(), Size::new(400.0, 240.0))
        .unwrap();
    let commands = output.native_surfaces[0].display_list.commands();
    let mut kinds = Vec::new();
    for command in commands {
        let (transform, clips) = match command {
            DisplayCommand::Image(image) => {
                kinds.push("image");
                (image.transform, &image.clips)
            }
            DisplayCommand::Vector(vector) => {
                kinds.push("vector");
                (vector.transform, &vector.clips)
            }
            DisplayCommand::GpuCanvas(canvas) => {
                kinds.push("gpu-canvas");
                (canvas.transform, &canvas.clips)
            }
            DisplayCommand::BeginLayer(layer) => {
                assert_eq!(layer.bounds.origin, Point::default());
                continue;
            }
            _ => continue,
        };
        assert_eq!(transform.translation, Point::new(120.0, -300.0));
        let clip = clips.regions().first().unwrap();
        assert_eq!(clip.transform.translation, transform.translation);
        assert_eq!(clip.bounds, bounds);
    }
    assert_eq!(kinds, ["image", "vector", "gpu-canvas"]);
    assert!(output.native_surfaces[0].display_list.validate().is_ok());
}
