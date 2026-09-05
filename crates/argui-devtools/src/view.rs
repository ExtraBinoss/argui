use argui_inspect::NodeSnapshot;
use argui_paint::{Border, Color, CornerRadii, LayerStyle, PaintStyle, QuadStyle, VectorId};
use argui_text::{TextColor, TextStyle, TextWrap};
use argui_ui::{
    Axes, Element, EventListener, Interaction, LengthPercentageAuto, Overflow, ScrollConfig,
    ScrollbarPartStyle, ScrollbarStyle, Sides, StylePatch, VirtualList, auto, length, percent,
    property, sides,
};
use argui_widgets::{Button, Input, InputStyle, WidgetTheme};

use crate::host::{DevtoolsHost, Tab};

mod dashboard;
mod graph;
mod profiling;
pub(crate) use dashboard::DetailCache;
mod toolbar;
use toolbar::{icon_label_button, small_button, toggle_button, toolbar};

pub(crate) fn host<A>(
    tools: &DevtoolsHost<A>,
    app: Element,
    theme: &WidgetTheme,
    splitter_listener: Option<EventListener>,
) -> Element {
    let application = Element::container([app
        .width(percent(1.0))
        .height(percent(1.0))
        .min_height(length(0.0))])
    .keyed("__devtools-app-root")
    .grow(1.0)
    .shrink(1.0)
    .min_height(length(0.0))
    .min_width(length(0.0))
    .overflow(Axes {
        x: Overflow::Hidden,
        y: Overflow::Hidden,
    });
    let mut toggle = toggle_button(false, theme)
        .absolute(top_right(14.0, 14.0))
        .z_index(30_000);
    if tools.open || tools.sheet_progress > 0.001 {
        if let Some(interaction) = &mut toggle.interaction {
            interaction.enabled = false;
        }
        toggle = toggle.layer(LayerStyle::new(Default::default()).opacity(0.0));
    }
    let mut children = vec![application, toggle];
    if tools.dock_mode != crate::DockMode::Detached {
        children.push(dock_surface(tools, theme, splitter_listener));
    }
    if tools.picking {
        children.push(picker_surface(tools));
    }
    (if tools.dock_mode == crate::DockMode::Right {
        Element::row(children)
    } else {
        Element::column(children)
    })
    .width(percent(1.0))
    .height(percent(1.0))
    .overflow(Axes {
        x: Overflow::Hidden,
        y: Overflow::Hidden,
    })
}

fn dock_surface<A>(
    tools: &DevtoolsHost<A>,
    theme: &WidgetTheme,
    splitter_listener: Option<EventListener>,
) -> Element {
    let extent = tools.dock_extent();
    let height = extent + 6.0;
    let parts = [
        splitter(tools, theme, splitter_listener),
        dock(
            tools,
            if tools.dock_mode == crate::DockMode::Right {
                tools.viewport.size.height.max(1.0)
            } else {
                extent
            },
            theme,
        ),
    ];
    let right = tools.dock_mode == crate::DockMode::Right;
    let surface = if right {
        Element::row(parts)
    } else {
        Element::column(parts)
    }
    .keyed("__devtools-surface-content")
    .width(if right { length(height) } else { percent(1.0) })
    .height(if right { percent(1.0) } else { length(height) })
    .shrink(0.0);
    Element::container([surface])
        .keyed("__devtools-surface")
        .width(if right {
            length(height * tools.sheet_progress)
        } else {
            percent(1.0)
        })
        .height(if right {
            percent(1.0)
        } else {
            length(height * tools.sheet_progress)
        })
        .shrink(0.0)
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Hidden,
        })
        .inspectable(false)
}

fn splitter<A>(
    tools: &DevtoolsHost<A>,
    theme: &WidgetTheme,
    listener: Option<EventListener>,
) -> Element {
    let mut element = tools.splitter.separator(theme).inspectable(false);
    if let Some(listener) = listener {
        element = element.on(listener);
    }
    element
}

pub(crate) fn dock<A>(tools: &DevtoolsHost<A>, height: f32, theme: &WidgetTheme) -> Element {
    let body = match tools.tab {
        Tab::Elements => elements_tab(tools, theme),
        Tab::Profiling => dashboard::panel(tools, theme),
    };
    Element::column([
        toolbar(tools, theme),
        body.grow(1.0)
            .shrink(1.0)
            .min_height(length(0.0))
            .min_width(length(0.0)),
    ])
    .height(length(height))
    .min_width(length(0.0))
    .grow(1.0)
    .shrink(1.0)
    .background(theme.background)
    .border(Border::all(1.0, theme.border))
    .overflow(Axes {
        x: Overflow::Hidden,
        y: Overflow::Hidden,
    })
    .inspectable(false)
}

fn picker_surface<A>(tools: &DevtoolsHost<A>) -> Element {
    Element::container([])
        .keyed("__devtools-picker-surface")
        .absolute(top_left(
            tools.app_viewport.origin.y,
            tools.app_viewport.origin.x,
        ))
        .width(length(tools.app_viewport.size.width.max(1.0)))
        .height(length(tools.app_viewport.size.height.max(1.0)))
        .z_index(19_000)
        .interaction(Interaction::blocker())
        .inspectable(false)
}

fn elements_tab<A>(tools: &DevtoolsHost<A>, theme: &WidgetTheme) -> Element {
    tools.inspector.with_tree(|snapshot| {
        let selected = tools.inspector.selected();
        let nodes = tools.tree_nodes();
        let selected_key = selected.map(|node| format!("__devtools-node-{}", node.0));
        let list = tools
            .tree_view(&nodes, selected_key.as_deref())
            .build_cached(theme, &mut tools.tree_rows.borrow_mut());
        let search = Input::new(
            "__devtools-search",
            &tools.search,
            "Filter elements…",
            InputStyle::new(
                PaintStyle::new(QuadStyle::solid(theme.card).radius(CornerRadii::all(5.0))),
                text(12.0, theme.foreground),
            )
            .focused(StylePatch::new().set(property::BackgroundColor, theme.muted)),
        )
        .build();
        let tree = Element::column([search, list.grow(1.0).shrink(1.0)])
            .grow(1.0)
            .shrink(1.0)
            .padding(Sides::length(6.0))
            .gap(6.0)
            .min_width(length(0.0))
            .min_height(length(0.0));
        let properties =
            crate::style::sidebar(selected, &snapshot.nodes, tools, theme).min_height(length(0.0));
        if tools.panel_width() < 640.0 {
            if tools.show_properties && selected.is_some() {
                Element::column([
                    small_button("__devtools-back", "← Elements", false, theme),
                    properties.grow(1.0),
                ])
                .min_height(length(0.0))
            } else {
                tree
            }
        } else {
            Element::row([tree, properties]).min_height(length(0.0))
        }
    })
}

pub(crate) fn tree_list_config(item_count: usize, viewport: f32) -> VirtualList {
    VirtualList::fixed(item_count, 28.0, viewport)
        .overscan(8)
        .scroll_config(ScrollConfig::default().scrollbar(scrollbar()))
}

pub(crate) fn matches_query(node: &NodeSnapshot, query: &str) -> bool {
    node.kind.to_lowercase().contains(query)
        || node
            .key
            .as_deref()
            .is_some_and(|key| key.to_lowercase().contains(query))
}

fn profiling_list<A>(tools: &DevtoolsHost<A>, header: Element, theme: &WidgetTheme) -> Element {
    let frames = &tools.profile_frames;
    let columns = frame_columns(
        ["Frame", "Time ms", "CPU ms", "Update", "Passes", "MiB"].map(str::to_owned),
        theme,
    );
    let header = Element::column([header, columns])
        .gap(10.0)
        .padding(Sides::length(8.0))
        .keyed("__devtools-frames-header");
    argui_widgets::VList::new(
        "__devtools-frames",
        28.0,
        tools.profile_extents.frames,
        tools.profiling_offset,
    )
    .build_with_header(
        frames.len(),
        theme,
        header,
        tools.profile_extents.frames_header,
        |index| {
            let position = frames.len() - 1 - index;
            let frame = &frames[position];
            let values = [
                format!("#{}", position + 1),
                format!("{:.2}", millis(frame.interval)),
                format!("{:.2}", millis(frame.total_cpu())),
                format!("{:?}", frame.update),
                frame.passes.to_string(),
                format!("{:.1}", frame.texture_bytes as f64 / 1_048_576.0),
            ];
            Button::new(
                format!("__devtools-frame-{position}"),
                values.join(" · "),
                theme.ghost_button(),
            )
            .content(frame_columns(values, theme))
            .build()
            .padding(sides(8.0, 3.0))
            .background(if index % 2 == 0 {
                theme.card
            } else {
                theme.background
            })
        },
    )
    .height(percent(1.0))
    .min_height(length(0.0))
    .min_width(length(0.0))
}

fn frame_columns(values: [String; 6], theme: &WidgetTheme) -> Element {
    Element::row(
        values
            .into_iter()
            .zip([0.13, 0.19, 0.19, 0.18, 0.13, 0.18])
            .map(|(value, width)| {
                Element::row([Element::text(value).text_style(TextStyle {
                    wrap: TextWrap::None,
                    ..text(11.0, theme.foreground)
                })])
                .width(percent(width))
                .min_width(length(0.0))
                .shrink(0.0)
                .justify_content(argui_ui::JustifyContent::END)
                .padding(sides(4.0, 0.0))
                .overflow(Axes {
                    x: Overflow::Hidden,
                    y: Overflow::Hidden,
                })
            }),
    )
    .width(percent(1.0))
    .min_width(length(0.0))
}

pub(crate) fn profiling_list_config(item_count: usize, viewport: f32) -> VirtualList {
    argui_widgets::VList::new("__devtools-frames", 28.0, viewport, 0.0).config(item_count)
}

fn icon_element(icon: VectorId, size: f32) -> Element {
    Element::vector(icon)
        .width(length(size))
        .height(length(size))
        .shrink(0.0)
}

fn text(size: f32, color: TextColor) -> TextStyle {
    TextStyle {
        font_size: size,
        color,
        wrap: TextWrap::Word,
        ..TextStyle::default()
    }
}

fn scrollbar() -> ScrollbarStyle {
    ScrollbarStyle::new(
        ScrollbarPartStyle::new(QuadStyle::solid(Color::srgba(0.0, 0.0, 0.0, 0.18))),
        ScrollbarPartStyle::new(
            QuadStyle::solid(Color::srgb(0.30, 0.38, 0.48)).radius(CornerRadii::all(4.0)),
        ),
    )
    .width(8.0)
    .insets(argui_ui::Sides::length(3.0))
}

fn millis(duration: std::time::Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}

fn top_left(top: f32, left: f32) -> Sides<LengthPercentageAuto> {
    Sides {
        left: length(left),
        right: auto(),
        top: length(top),
        bottom: auto(),
    }
}

fn top_right(top: f32, right: f32) -> Sides<LengthPercentageAuto> {
    Sides {
        left: auto(),
        right: length(right),
        top: length(top),
        bottom: auto(),
    }
}
