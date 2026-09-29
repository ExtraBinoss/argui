//! Intrinsic popup sizing and viewport-constrained text reflow.

use argui_core::Size;
use argui_layout::LayoutEngine;
use argui_ui::{
    Element, FloatingPlacement, Placement, UiTree, WindowLayer, length, percent, sides,
};

/// Builds a floating block whose bounded flex surface measures its text intrinsically.
fn popup(content: &str) -> Element {
    Element::container([Element::row([Element::container([
        Element::text(content).width(percent(1.))
    ])
    .min_width(length(0.))])
    .max_width(length(180.))
    .padding(sides(10., 10.))])
    .keyed("popup")
    .anchored_portal(
        WindowLayer::Popover,
        "anchor",
        FloatingPlacement::new(Placement::BottomStart),
    )
}

#[test]
fn automatic_portals_fit_short_content_and_wrap_long_content() {
    let mut engine = LayoutEngine::new();
    let mut text = super::text_engine();
    let anchor = Element::container([])
        .keyed("anchor")
        .width(length(20.))
        .height(length(20.));
    let mut ui = UiTree::new(Element::column([anchor.clone(), popup("Media")]));
    let mut short_height = 0.;
    for (content, width, wrapped) in [
        ("Media".to_owned(), 800., false),
        (
            "Wrap this longer tooltip description without truncating its remaining text. "
                .repeat(5),
            800.,
            true,
        ),
        (
            "Wrap this description inside a narrow viewport. ".repeat(5),
            120.,
            true,
        ),
    ] {
        ui.update(Element::column([anchor.clone(), popup(&content)]));
        let output = engine
            .compute(&mut ui, &mut text, Size::new(width, 600.))
            .unwrap();
        let portal = output.portals.first().unwrap();
        let surface = output
            .nodes
            .iter()
            .find(|node| node.index == 3)
            .unwrap()
            .bounds;
        assert!(portal.bounds.size.width <= 180. && portal.bounds.size.width <= width - 16.);
        assert!(
            (portal.bounds.size.width - surface.size.width).abs() <= 1.,
            "{portal:?}/{surface:?}"
        );
        if wrapped {
            assert!(
                surface.size.height > short_height * 3.,
                "long hint must reflow: {surface:?}"
            );
        } else {
            assert!(
                surface.size.width < 100.,
                "short hint must shrink: {surface:?}"
            );
            short_height = surface.size.height;
        }
    }
}

#[test]
fn explicit_and_percentage_portal_widths_keep_their_authoring_contract() {
    for (width, expected) in [(length(240.), 240.), (percent(0.5), 400.)] {
        let mut ui = UiTree::new(Element::column([
            Element::container([])
                .keyed("anchor")
                .width(length(20.))
                .height(length(20.)),
            popup("Media").width(width),
        ]));
        let output = LayoutEngine::new()
            .compute(&mut ui, &mut super::text_engine(), Size::new(800., 600.))
            .unwrap();
        assert_eq!(output.portals[0].bounds.size.width, expected);
    }
}
