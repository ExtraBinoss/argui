use argui_core::{Point, Rect, Size, Transform2D};
use argui_layout::{LayoutEngine, LayoutOutput};
use argui_text::TextEngine;
use argui_ui::{Element, LayerStyle, NativeContent, Position, UiTree, length};
use argui_webview::{WebView, WebViewSource, WebViewState, resolve_mounts};

fn layout(root: Element) -> (UiTree, LayoutOutput) {
    let mut ui = UiTree::new(root);
    let output = LayoutEngine::new()
        .compute(&mut ui, &mut TextEngine::new(), Size::new(600.0, 500.0))
        .unwrap();
    (ui, output)
}

fn view() -> (WebViewState, Element) {
    let state = WebViewState::new(WebViewSource::html("email"));
    let element = WebView::new(&state)
        .build()
        .width(length(300.0))
        .height(length(200.0));
    (state, element)
}

#[test]
fn native_mount_uses_layout_geometry_and_retained_payload_not_a_key_convention() {
    let (state, view) = view();
    let (ui, output) = layout(Element::column([view.keyed("custom-key")]));
    let mounts = resolve_mounts(&ui, &output, 7);
    assert_eq!(mounts.len(), 1);
    assert_eq!(mounts[0].state.id(), state.id());
    assert_eq!(mounts[0].host, 7);
    assert_eq!(mounts[0].bounds.size, Size::new(300.0, 200.0));
    assert!(!mounts[0].occluded);
    state.load(WebViewSource::html("next")).unwrap();
    assert_eq!(
        resolve_mounts(&ui, &output, 7)[0].state.source(),
        WebViewSource::html("next")
    );
}

#[test]
fn full_clipping_unmounts_and_partial_clipping_hides_without_changing_size() {
    let (_, view) = view();
    let (ui, mut output) = layout(Element::column([view]));
    let native = output
        .nodes
        .iter_mut()
        .find(|node| ui.element_at(node.index).unwrap().native_content.is_some())
        .unwrap();
    native.clip = Some(Rect::new(Point::default(), Size::new(100.0, 100.0)));
    let mounts = resolve_mounts(&ui, &output, 1);
    assert!(mounts[0].occluded);
    let clipped = argui_webview::resolve_clipped_mounts(&ui, &output, 1);
    assert!(!clipped[0].0.occluded);
    assert_eq!(clipped[0].0.bounds.size, Size::new(300.0, 200.0));
    assert_eq!(
        clipped[0].1,
        Some(Rect::new(Point::default(), Size::new(100.0, 100.0)))
    );
    assert_eq!(mounts[0].bounds.size, Size::new(300.0, 200.0));
    let native = output
        .nodes
        .iter_mut()
        .find(|node| ui.element_at(node.index).unwrap().native_content.is_some())
        .unwrap();
    native.clip = Some(Rect::new(Point::new(500.0, 500.0), Size::new(10.0, 10.0)));
    assert!(resolve_mounts(&ui, &output, 1).is_empty());
}

#[test]
fn later_painted_overlays_hide_native_surfaces_but_empty_containers_do_not() {
    for painted in [false, true] {
        let (_, view) = view();
        let overlay = Element::container([])
            .position(Position::Absolute)
            .width(length(50.0))
            .height(length(50.0));
        let overlay = if painted {
            overlay.background(argui_core::Color::WHITE)
        } else {
            overlay
        };
        let (ui, output) = layout(Element::column([view, overlay]));
        assert_eq!(resolve_mounts(&ui, &output, 1)[0].occluded, painted);
    }
}

#[test]
fn transformed_ancestors_are_not_silently_ignored_by_native_surfaces() {
    let (_, view) = view();
    let mut parent = Element::column([view]);
    parent.transform = Transform2D {
        rotation: 0.2,
        ..Transform2D::IDENTITY
    };
    let (ui, output) = layout(parent);
    assert!(resolve_mounts(&ui, &output, 1)[0].occluded);
    let (ui, output) = layout(Element::text("not native"));
    assert!(resolve_mounts(&ui, &output, 1).is_empty());
}

#[test]
fn ancestor_opacity_and_layers_occlude_native_surfaces() {
    let (_, first_view) = view();
    let mut parent = Element::column([first_view]);
    parent.paint.quad.opacity = 0.5;
    let (ui, output) = layout(parent);
    assert!(resolve_mounts(&ui, &output, 1)[0].occluded);

    let (_, second_view) = view();
    let parent = Element::column([second_view]).layer(LayerStyle::new(Rect::default()));
    let (ui, output) = layout(parent);
    assert!(resolve_mounts(&ui, &output, 1)[0].occluded);
}

#[test]
fn non_webview_native_payloads_do_not_become_webview_mounts() {
    let (_, view) = view();
    let foreign = Element::container([])
        .native_content(NativeContent::new(
            42,
            String::from("another native surface"),
        ))
        .position(Position::Absolute)
        .width(length(50.0))
        .height(length(50.0));
    let (ui, output) = layout(Element::column([view, foreign]));
    let mounts = resolve_mounts(&ui, &output, 1);
    assert_eq!(mounts.len(), 1);
    assert!(mounts[0].occluded);
}

#[test]
fn stale_layout_indices_and_foreign_node_ids_are_ignored() {
    let (_, view) = view();
    let overlay = Element::container([])
        .keyed("overlay")
        .position(Position::Absolute)
        .width(length(50.0))
        .height(length(50.0));
    let (ui, mut output) = layout(Element::column([view, overlay]));
    let native_index = output
        .nodes
        .iter()
        .position(|node| {
            ui.element_at(node.index)
                .is_some_and(|element| element.native_content.is_some())
        })
        .unwrap();
    let overlay_index = output
        .nodes
        .iter()
        .position(|node| {
            ui.element_at(node.index)
                .is_some_and(|element| element.key.as_deref() == Some("overlay"))
        })
        .unwrap();

    let foreign_root =
        Element::column((0..ui.node_ids().len() + 1).map(|_| Element::text("foreign tree node")));
    let (_, foreign_output) = layout(foreign_root);
    let foreign_node = foreign_output
        .nodes
        .iter()
        .map(|node| node.node)
        .find(|node| !ui.node_ids().contains(node))
        .unwrap();
    output.nodes[native_index].node = foreign_node;
    output.nodes[overlay_index].index = usize::MAX;

    let mounts = resolve_mounts(&ui, &output, 1);
    assert_eq!(mounts.len(), 1);
    assert!(!mounts[0].occluded);
}

#[test]
fn a_later_surface_clipped_away_does_not_occlude_a_webview() {
    let (_, view) = view();
    let overlay = Element::container([])
        .keyed("overlay")
        .position(Position::Absolute)
        .width(length(50.0))
        .height(length(50.0));
    let (ui, mut output) = layout(Element::column([view, overlay]));
    let overlay = output
        .nodes
        .iter_mut()
        .find(|node| {
            ui.element_at(node.index)
                .is_some_and(|element| element.key.as_deref() == Some("overlay"))
        })
        .unwrap();
    overlay.clip = Some(Rect::new(Point::new(500.0, 500.0), Size::new(5.0, 5.0)));

    assert!(!resolve_mounts(&ui, &output, 1)[0].occluded);
}
