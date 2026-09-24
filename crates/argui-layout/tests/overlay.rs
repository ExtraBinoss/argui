use argui_core::{Point, Size};
use argui_layout::LayoutEngine;
use argui_paint::{Color, DisplayCommand, Fill, QuadStyle};
use argui_text::{TextEngine, TextStyle};
use argui_ui::{
    Axes, CaretStyle, Element, FloatingPlacement, Interaction, Overflow, Placement, ScrollConfig,
    ScrollbarPartStyle, ScrollbarStyle, StylePatch, TextEditorSpec, TextInputFilter, UiTree,
    ViewportPlacement, VisualState, WindowLayer, length,
};

const NOTO_SANS: &[u8] = include_bytes!("../../../assets/fonts/NotoSans-Regular.ttf");

fn text_engine() -> TextEngine {
    TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans")
}

#[test]
fn window_layers_override_document_order_and_local_z_index() {
    let background = Color::srgb(0.1, 0.0, 0.0);
    let content = Color::srgb(0.2, 0.0, 0.0);
    let content_portal = Color::srgb(0.25, 0.0, 0.0);
    let layout_portal = Color::srgb(0.26, 0.0, 0.0);
    let floating = Color::srgb(0.275, 0.0, 0.0);
    let popover = Color::srgb(0.3, 0.0, 0.0);
    let modal = Color::srgb(0.4, 0.0, 0.0);
    let debug = Color::srgb(0.5, 0.0, 0.0);
    let item = |key: &str, color, layer: Option<WindowLayer>, z_index| {
        let element = Element::container([])
            .keyed(key)
            .width(length(40.0))
            .height(length(40.0))
            .background(color)
            .interaction(Interaction::blocker())
            .z_index(z_index);
        layer.map_or(element.clone(), |layer| {
            element.viewport_portal(layer, ViewportPlacement::centered())
        })
    };
    let mut ui = UiTree::new(Element::container([
        item("debug", debug, Some(WindowLayer::Debug), -100),
        item("modal", modal, Some(WindowLayer::Modal), -100),
        item("popover", popover, Some(WindowLayer::Popover), -100),
        item("floating", floating, Some(WindowLayer::Floating), 500),
        item(
            "content-portal",
            content_portal,
            Some(WindowLayer::Content),
            -500,
        ),
        Element::container([])
            .keyed("layout-portal")
            .width(length(40.0))
            .height(length(40.0))
            .background(layout_portal)
            .interaction(Interaction::blocker())
            .portal(WindowLayer::Content),
        item("content", content, None, 100_000),
        item(
            "background",
            background,
            Some(WindowLayer::Background),
            100_000,
        ),
    ]));
    let mut layout = LayoutEngine::new();
    let output = layout
        .compute(&mut ui, &mut text_engine(), Size::new(200.0, 120.0))
        .unwrap();
    let colors = output
        .display_list
        .commands()
        .iter()
        .filter_map(|command| match command {
            DisplayCommand::Quad(quad) => match quad.background {
                Some(Fill::Solid(color)) => Some(color),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(
        colors,
        vec![
            background,
            content,
            content_portal,
            layout_portal,
            floating,
            popover,
            modal,
            debug
        ]
    );
    assert_eq!(
        output
            .portals
            .iter()
            .map(|portal| portal.layer)
            .collect::<Vec<_>>(),
        vec![
            WindowLayer::Debug,
            WindowLayer::Modal,
            WindowLayer::Popover,
            WindowLayer::Floating,
            WindowLayer::Content,
            WindowLayer::Content,
            WindowLayer::Background,
        ]
    );
    assert_eq!(
        output.hit_regions.last().map(|region| region.node),
        ui.node_ids()
            .iter()
            .copied()
            .find(|node| ui.key(*node) == Some("debug"))
    );
}

#[test]
fn constrained_portal_reflows_its_children() {
    let anchor = Element::container([])
        .keyed("edge")
        .width(length(20.0))
        .height(length(20.0));
    let portal = Element::column([
        Element::container([])
            .width(length(180.0))
            .height(length(80.0)),
        Element::container([])
            .width(length(180.0))
            .height(length(80.0)),
    ])
    .width(length(180.0))
    .anchored_portal(
        WindowLayer::Popover,
        "edge",
        FloatingPlacement::new(Placement::BottomStart).viewport_padding(8.0),
    );
    let mut ui = UiTree::new(Element::column([anchor, portal]));
    let output = LayoutEngine::new()
        .compute(&mut ui, &mut text_engine(), Size::new(120.0, 100.0))
        .unwrap();

    assert!(output.nodes[2].bounds.size.width <= 86.0);
    assert!(output.nodes[2].bounds.size.height <= 84.0);
    assert_eq!(output.nodes[2].clip, Some(output.viewport));
}

#[test]
fn nested_popovers_do_not_expand_or_scroll_their_ancestors() {
    let submenu = Element::column([Element::text("First"), Element::text("Second")])
        .keyed("submenu")
        .width(length(288.0))
        .height(length(120.0))
        .anchored_portal(
            WindowLayer::Popover,
            "options",
            FloatingPlacement::new(Placement::RightStart),
        );
    let view = |open: bool| {
        let menu = Element::column(
            [
                Element::text("Copy").height(length(36.0)),
                Element::text("Options")
                    .keyed("options")
                    .height(length(36.0)),
            ]
            .into_iter()
            .chain(open.then(|| submenu.clone())),
        )
        .keyed("menu")
        .width(length(240.0))
        .overflow(Axes {
            x: Overflow::Auto,
            y: Overflow::Auto,
        })
        .anchored_portal(
            WindowLayer::Popover,
            "trigger",
            FloatingPlacement::new(Placement::BottomStart),
        );
        Element::column([
            Element::text("Open").keyed("trigger").height(length(40.0)),
            menu,
        ])
        .keyed("page")
        .width(length(200.0))
        .height(length(50.0))
        .overflow(Axes {
            x: Overflow::Auto,
            y: Overflow::Auto,
        })
    };
    let mut ui = UiTree::new(view(false));
    let mut engine = LayoutEngine::new();
    let mut text = text_engine();
    for open in [false, true, false, true] {
        ui.update(view(open));
        let mut output = engine
            .compute(&mut ui, &mut text, Size::new(800.0, 600.0))
            .unwrap();
        for _ in 0..2 {
            for key in ["page", "menu"] {
                let region = output
                    .scroll_regions
                    .iter()
                    .find(|r| ui.key(r.node) == Some(key))
                    .unwrap();
                assert_eq!(region.max_offset, Point::default(), "{key}, open={open}");
            }
            if open {
                let menu = output
                    .portals
                    .iter()
                    .find(|p| ui.key(p.node) == Some("menu"))
                    .unwrap();
                let submenu = output
                    .portals
                    .iter()
                    .find(|p| ui.key(p.node) == Some("submenu"))
                    .unwrap();
                assert!(submenu.bounds.origin.x >= menu.bounds.origin.x + menu.bounds.size.width);
                assert_eq!(submenu.bounds.size, Size::new(288.0, 120.0));
            }
            engine.apply_scroll(&ui, &mut output).unwrap();
        }
    }
}

#[test]
fn scrollable_overlay_translates_its_input_region_and_scrollbar() {
    let scrollbar = ScrollbarStyle::new(
        ScrollbarPartStyle::new(QuadStyle::solid(Color::srgb(0.1, 0.1, 0.1))),
        ScrollbarPartStyle::new(QuadStyle::solid(Color::WHITE)),
    );
    let anchor = Element::container([])
        .keyed("overlay-anchor")
        .width(length(60.0))
        .height(length(24.0));
    let overlay = Element::column([
        Element::text_editor(TextEditorSpec {
            value: "value".to_owned(),
            placeholder: "placeholder".to_owned(),
            multiline: false,
            read_only: false,
            filter: TextInputFilter::Any,
            text: TextStyle::default(),
            placeholder_text: TextStyle::default(),
            selection: Color::WHITE,
            caret: CaretStyle::default(),
        })
        .keyed("overlay-input")
        .width(length(140.0))
        .interaction(Interaction::default())
        .height(length(28.0))
        .shrink(0.0),
        Element::container([]).height(length(180.0)).shrink(0.0),
    ])
    .keyed("scrollable-overlay")
    .width(length(140.0))
    .height(length(80.0))
    .overflow(Axes {
        x: Overflow::Hidden,
        y: Overflow::Auto,
    })
    .scroll_config(ScrollConfig::default().scrollbar(scrollbar))
    .anchored_portal(
        WindowLayer::Popover,
        "overlay-anchor",
        FloatingPlacement::new(Placement::Bottom).viewport_padding(0.0),
    );
    let missing_anchor = Element::container([])
        .width(length(20.0))
        .height(length(20.0))
        .anchored_portal(
            WindowLayer::Popover,
            "absent-anchor",
            FloatingPlacement::new(Placement::Right),
        );
    let mut ui = UiTree::new(Element::column([anchor, overlay, missing_anchor]));
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(320.0, 240.0))
        .unwrap();

    assert_eq!(output.text_inputs.len(), 1);
    assert_eq!(output.scroll_regions.len(), 1);
    assert!(output.scroll_regions[0].scrollbar.is_some());
    layout.apply_scroll(&ui, &mut output).unwrap();
    assert_eq!(output.text_inputs.len(), 1);
    assert_eq!(output.scroll_regions.len(), 1);
    assert!(output.scroll_regions[0].scrollbar.is_some());
}

#[test]
fn portal_descendants_receive_hover_and_resolve_their_state_style() {
    let hover_color = Color::srgb(0.2, 0.5, 0.9).with_alpha(0.20);
    let option = Element::container([Element::text("Metal")])
        .keyed("backend::option::1")
        .width(length(180.0))
        .height(length(32.0))
        .background(Color::TRANSPARENT)
        .interaction(Interaction::default())
        .when(
            VisualState::Hovered,
            StylePatch::from_quad(QuadStyle::solid(hover_color)),
        );
    let popup = Element::column([option])
        .keyed("backend::popup")
        .width(length(200.0))
        .height(length(48.0))
        .viewport_portal(WindowLayer::Popover, ViewportPlacement::centered());
    let mut ui = UiTree::new(Element::container([popup]));
    let mut layout = LayoutEngine::new();
    let mut output = layout
        .compute(&mut ui, &mut text_engine(), Size::new(480.0, 320.0))
        .unwrap();
    let option = ui
        .node_ids()
        .iter()
        .copied()
        .find(|node| ui.key(*node) == Some("backend::option::1"))
        .unwrap();
    let region = output
        .hit_regions
        .iter()
        .find(|region| region.node == option)
        .unwrap();
    let point = Point::new(
        region.bounds.origin.x + region.bounds.size.width * 0.5,
        region.bounds.origin.y + region.bounds.size.height * 0.5,
    );

    ui.set_reduced_motion(true);
    let update = ui.pointer_moved(point, &output.hit_regions);
    assert!(update.paint_changed);
    assert!(ui.visual_states(option).contains(VisualState::Hovered));
    layout.repaint(&ui, &mut output);
    let index = ui
        .node_ids()
        .iter()
        .position(|node| *node == option)
        .unwrap();
    assert_eq!(
        ui.resolved_quad(option, ui.element_at(index).unwrap())
            .background,
        Some(Fill::Solid(hover_color))
    );
}

#[test]
fn floating_layout_preserves_hidden_ancestors_and_can_return_to_normal_flow() {
    use argui_ui::Display;
    let view = |hidden, floating| {
        let mut surface = Element::container([])
            .keyed("surface")
            .width(length(80.0))
            .height(length(70.0))
            .background(Color::WHITE);
        if floating {
            surface = surface.viewport_portal(WindowLayer::Popover, ViewportPlacement::centered());
        }
        Element::column([surface])
            .keyed("parent")
            .width(length(100.0))
            .height(length(40.0))
            .overflow(Axes {
                x: Overflow::Auto,
                y: Overflow::Auto,
            })
            .display(if hidden {
                Display::None
            } else {
                Display::Block
            })
    };
    let mut ui = UiTree::new(view(false, true));
    let mut engine = LayoutEngine::new();
    let mut text = text_engine();
    for (hidden, floating) in [
        (false, true),
        (true, true),
        (false, true),
        (false, false),
        (false, true),
    ] {
        ui.update(view(hidden, floating));
        let layout = engine
            .compute(&mut ui, &mut text, Size::new(400.0, 300.0))
            .unwrap();
        assert_eq!(
            layout.nodes[1].bounds.size,
            if hidden {
                Size::default()
            } else {
                Size::new(80.0, 70.0)
            }
        );
        if hidden {
            assert!(layout.display_list.commands().is_empty());
        } else {
            assert_eq!(
                layout.scroll_regions[0].max_offset.y,
                if floating { 0.0 } else { 30.0 }
            );
        }
        assert_eq!(engine.retained_node_count(), 2);
    }
}
