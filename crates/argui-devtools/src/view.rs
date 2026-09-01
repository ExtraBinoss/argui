use argui_core::Transform2D;
use argui_inspect::{FrameRecord, NodeSnapshot};
use argui_paint::{Border, Color, CornerRadii, LayerStyle, PaintStyle, QuadStyle, VectorId};
use argui_text::{TextColor, TextStyle, TextWrap};
use argui_ui::{
    AlignItems, Axes, Element, FlexWrap, Interaction, LayoutStyle, LengthPercentageAuto, Overflow,
    ScrollConfig, ScrollbarPartStyle, ScrollbarStyle, Sides, StateStyle, VirtualList, VisualState,
    auto, length, percent, property, sides,
};
use argui_widgets::{Button, ButtonStyle, Input, InputStyle, WidgetTheme};

use crate::host::{DevtoolsHost, Tab};

const RETAINED_TREE_LIMIT: usize = 512;

pub(crate) fn host<A>(tools: &DevtoolsHost<A>, app: Element, theme: &WidgetTheme) -> Element {
    let application = Element::container([app
        .width(percent(1.0))
        .height(percent(1.0))
        .min_height(length(0.0))])
    .keyed("__devtools-app-root")
    .grow(1.0)
    .shrink(1.0)
    .min_height(length(0.0))
    .overflow(Axes {
        x: Overflow::Hidden,
        y: Overflow::Hidden,
    });
    let mut toggle = toggle_button(false, theme)
        .absolute(top_left(14.0, 14.0))
        .z_index(30_000);
    if tools.open || tools.sheet_progress > 0.001 {
        if let Some(interaction) = &mut toggle.interaction {
            interaction.enabled = false;
        }
        toggle = toggle.layer(LayerStyle::new(Default::default()).opacity(0.0));
    }
    let mut children = vec![application, toggle, dock_surface(tools, theme)];
    if tools.picking {
        children.push(picker_surface(tools));
    }
    Element::column(children)
        .width(percent(1.0))
        .height(percent(1.0))
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Hidden,
        })
}

fn dock_surface<A>(tools: &DevtoolsHost<A>, theme: &WidgetTheme) -> Element {
    let height = tools.dock_height + 6.0;
    let surface = Element::column([splitter(theme), dock(tools, tools.dock_height, theme)])
        .keyed("__devtools-surface-content")
        .width(percent(1.0))
        .height(length(height))
        .shrink(0.0);
    Element::container([surface])
        .keyed("__devtools-surface")
        .width(percent(1.0))
        .height(length(height * tools.sheet_progress))
        .shrink(0.0)
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Hidden,
        })
        .inspectable(false)
}

fn splitter(theme: &WidgetTheme) -> Element {
    Element::container([])
        .keyed("__devtools-splitter")
        .height(length(6.0))
        .shrink(0.0)
        .background(theme.primary)
        .interaction(Interaction::default().cursor(argui_ui::CursorIcon::NsResize))
        .state(
            VisualState::Hovered,
            StateStyle::from_quad(QuadStyle::solid(theme.primary)),
        )
        .state(
            VisualState::Pressed,
            StateStyle::from_quad(QuadStyle::solid(theme.foreground)),
        )
        .inspectable(false)
}

fn dock<A>(tools: &DevtoolsHost<A>, height: f32, theme: &WidgetTheme) -> Element {
    let body = match tools.tab {
        Tab::Elements => elements_tab(tools, theme),
        Tab::Profiling => profiling_tab(tools, theme),
    };
    Element::column([toolbar(tools, theme), body.grow(1.0).shrink(1.0)])
        .height(length(height))
        .shrink(0.0)
        .background(theme.background)
        .border(Border::all(1.0, theme.border))
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Hidden,
        })
        .inspectable(false)
}

fn toolbar<A>(tools: &DevtoolsHost<A>, theme: &WidgetTheme) -> Element {
    Element::row([
        toggle_button(true, theme),
        tab_button(
            "__devtools-elements",
            "Elements",
            tools.tab == Tab::Elements,
            theme,
        ),
        tab_button(
            "__devtools-profiling",
            "Profiling",
            tools.tab == Tab::Profiling,
            theme,
        ),
        picker_button(tools, theme),
        animated_icon_button(tools, theme),
    ])
    .height(length(42.0))
    .shrink(0.0)
    .gap(6.0)
    .padding(sides(8.0, 6.0))
    .background(theme.card)
    .align_items(AlignItems::CENTER)
}

fn picker_button<A>(tools: &DevtoolsHost<A>, theme: &WidgetTheme) -> Element {
    icon_label_button(
        "__devtools-picker",
        tools.icons.target,
        if tools.picking {
            "Picking…"
        } else {
            "Select"
        },
        theme,
    )
    .background(if tools.picking {
        theme.primary
    } else {
        theme.muted
    })
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

fn animated_icon_button<A>(tools: &DevtoolsHost<A>, theme: &WidgetTheme) -> Element {
    Element::row([
        icon_element(tools.icons.chevron, 20.0).transform(
            Transform2D::IDENTITY.rotate(tools.icon_progress * std::f32::consts::FRAC_PI_2),
        ),
        Element::text("GPU transform").text_style(text(12.0, theme.foreground)),
    ])
    .keyed("__devtools-icon-transform")
    .height(length(30.0))
    .padding(sides(8.0, 4.0))
    .gap(5.0)
    .align_items(AlignItems::CENTER)
    .background(theme.background)
    .interaction(Interaction::default())
    .state(
        VisualState::Hovered,
        StateStyle::from_quad(QuadStyle::solid(theme.muted)),
    )
}

fn elements_tab<A>(tools: &DevtoolsHost<A>, theme: &WidgetTheme) -> Element {
    tools.inspector.with_tree(|snapshot| {
        let selected = tools.inspector.selected();
        let viewport = (tools.dock_height - 84.0).max(80.0);
        let query = tools.search.to_lowercase();
        let filtered = (!query.is_empty()).then(|| {
            snapshot
                .nodes
                .iter()
                .enumerate()
                .filter_map(|(index, node)| matches_query(node, &query).then_some(index))
                .collect::<Vec<_>>()
        });
        let item_count = filtered
            .as_ref()
            .map_or(snapshot.nodes.len(), |indices| indices.len());
        let list = tree_list_config(item_count, viewport).build(
            "__devtools-tree",
            tools.tree_offset,
            |index| {
                let node = &snapshot.nodes[filtered.as_ref().map_or(index, |items| items[index])];
                tree_row(node, selected == Some(node.id), theme)
            },
        );
        let search = Input::new(
            "__devtools-search",
            &tools.search,
            "Filter elements…",
            InputStyle::new(
                PaintStyle::new(QuadStyle::solid(theme.card).radius(CornerRadii::all(5.0))),
                text(12.0, theme.foreground),
            )
            .focused(StateStyle::new().set(property::BackgroundColor, theme.muted)),
        )
        .build();
        Element::row([
            Element::column([search, list.grow(1.0).shrink(1.0)])
                .grow(1.0)
                .shrink(1.0)
                .padding(Sides::length(6.0))
                .gap(6.0),
            crate::style::sidebar(selected, &snapshot.nodes, tools, theme),
        ])
        .height(percent(1.0))
    })
}

pub(crate) fn tree_list_config(item_count: usize, viewport: f32) -> VirtualList {
    let overscan = if item_count <= RETAINED_TREE_LIMIT {
        item_count
    } else {
        12
    };
    VirtualList::new(item_count, 28.0, viewport)
        .overscan(overscan)
        .scroll_config(ScrollConfig::default().scrollbar(scrollbar()))
}

pub(crate) fn matches_query(node: &NodeSnapshot, query: &str) -> bool {
    node.kind.to_lowercase().contains(query)
        || node
            .key
            .as_deref()
            .is_some_and(|key| key.to_lowercase().contains(query))
}

fn tree_row(node: &NodeSnapshot, selected: bool, theme: &WidgetTheme) -> Element {
    let identity = node
        .key
        .as_deref()
        .map_or_else(|| node.kind.clone(), |key| format!("{}  #{key}", node.kind));
    let label = node.summary.as_deref().map_or(identity.clone(), |summary| {
        format!("{identity}  —  {summary}")
    });
    Element::text(format!("{}{}", "  ".repeat(node.depth), label))
        .keyed(format!("__devtools-node-{}", node.id.0))
        .padding(sides(8.0, 4.0))
        .background(if selected {
            theme.primary
        } else {
            theme.background
        })
        .border(Border::all(0.5, theme.border))
        .text_style(text(
            13.0,
            if selected {
                theme.primary_foreground
            } else {
                theme.foreground
            },
        ))
        .interaction(Interaction::default())
        .state(
            VisualState::Hovered,
            StateStyle::from_quad(QuadStyle::solid(theme.muted)),
        )
}

fn profiling_tab<A>(tools: &DevtoolsHost<A>, theme: &WidgetTheme) -> Element {
    let frames = &tools.profile_frames;
    let latest = frames.last().cloned().unwrap_or_default();
    let bars = frames
        .iter()
        .rev()
        .take(60)
        .rev()
        .map(|frame| frame_bar(frame, theme));
    Element::column([
        Element::row([
            small_button(
                "__devtools-pause",
                if tools.inspector.paused() {
                    "Resume"
                } else {
                    "Pause"
                },
                tools.inspector.paused(),
                theme,
            ),
            small_button("__devtools-clear", "Clear", false, theme),
            small_button("__devtools-refresh", "Refresh", false, theme),
            icon_label_button("__devtools-copy", tools.icons.copy, "Copy trace", theme),
            metric(format!("frame {:.2} ms", millis(latest.interval)), theme),
            metric(format!("CPU {:.2} ms", millis(latest.total_cpu())), theme),
            metric(
                format!(
                    "GPU {:.2} ms",
                    latest.gpu.as_ref().map_or(0.0, |gpu| millis(gpu.total))
                ),
                theme,
            ),
            metric(format!("{} passes", latest.passes), theme),
            metric(
                format!("{:.1} MiB", latest.texture_bytes as f64 / 1_048_576.0),
                theme,
            ),
        ])
        .flex_wrap(FlexWrap::Wrap)
        .gap(8.0),
        Element::row(bars)
            .height(length(110.0))
            .align_items(AlignItems::END)
            .gap(2.0)
            .padding(Sides::length(8.0))
            .background(theme.card),
        details(&latest, theme),
        gpu_waterfall(&latest, theme),
        profiling_list(frames, tools.profiling_offset, theme),
    ])
    .padding(Sides::length(12.0))
    .gap(12.0)
}

fn profiling_list(frames: &[FrameRecord], offset: f32, theme: &WidgetTheme) -> Element {
    let ordered = frames.iter().rev().cloned().collect::<Vec<_>>();
    profiling_list_config(ordered.len()).build("__devtools-frames", offset, |index| {
        let frame = &ordered[index];
        Element::text(format!(
            "#{:03}  {:>6.2} ms · CPU {:>6.2} · {:?} · {} passes · {:.1} MiB",
            frames.len().saturating_sub(index),
            millis(frame.interval),
            millis(frame.total_cpu()),
            frame.update,
            frame.passes,
            frame.texture_bytes as f64 / 1_048_576.0,
        ))
        .padding(sides(7.0, 3.0))
        .background(if index % 2 == 0 {
            theme.card
        } else {
            theme.background
        })
        .text_style(text(11.0, theme.foreground))
    })
}

pub(crate) fn profiling_list_config(item_count: usize) -> VirtualList {
    VirtualList::new(item_count, 27.0, 130.0)
        .scroll_config(ScrollConfig::default().scrollbar(scrollbar()))
}

fn icon_label_button(key: &str, icon: VectorId, label: &str, theme: &WidgetTheme) -> Element {
    Element::row([
        icon_element(icon, 16.0),
        Element::text(label).text_style(text(12.0, theme.foreground)),
    ])
    .keyed(key)
    .align_items(AlignItems::CENTER)
    .gap(6.0)
    .padding(sides(9.0, 6.0))
    .background(theme.muted)
    .radius(CornerRadii::all(5.0))
    .interaction(Interaction::default())
    .state(
        VisualState::Hovered,
        StateStyle::from_quad(QuadStyle::solid(theme.muted)),
    )
}

fn icon_element(icon: VectorId, size: f32) -> Element {
    Element::vector(icon)
        .width(length(size))
        .height(length(size))
        .shrink(0.0)
}

fn frame_bar(frame: &FrameRecord, theme: &WidgetTheme) -> Element {
    let milliseconds = millis(frame.interval).max(millis(frame.total_cpu()));
    let missed = milliseconds > 18.0;
    Element::container([])
        .width(length(5.0))
        .height(length((milliseconds * 3.0).clamp(2.0, 94.0) as f32))
        .shrink(0.0)
        .background(if missed {
            Color::rgb(1.0, 0.28, 0.24)
        } else {
            theme.primary
        })
        .radius(CornerRadii::all(2.0))
}

fn details(frame: &FrameRecord, theme: &WidgetTheme) -> Element {
    Element::column([
        metric(format!(
            "model {:.2} · surface {:.2} · tree {:.2} · layout {:.2} · paint {:.2} · render {:.2} ms",
            millis(frame.model),
            millis(frame.surface),
            millis(frame.tree),
            millis(frame.layout),
            millis(frame.paint),
            millis(frame.render_cpu)
        ), theme),
        metric(format!(
            "invalidation {:?} · layers {} · cached {} · offscreen {} px · damaged {} px",
            frame.update,
            frame.layers,
            frame.cached_layers,
            frame.offscreen_pixels,
            frame.damaged_pixels,
        ), theme),
        metric(format!(
            "textures {} · reused {} · vectors {} cached / {} hits / {} rasterized · resize events {}",
            frame.textures, frame.reused_textures, frame.vector_atlas_entries,
            frame.vector_atlas_hits, frame.vector_rasterizations,
            frame.resize_events
        ), theme),
        metric(format!(
            "adapter {} · {} · timestamps {} · features {}",
            frame.adapter.name,
            frame.adapter.backend,
            if frame.adapter.timestamp_queries {
                "on"
            } else {
                "unavailable"
            },
            frame.adapter.features,
        ), theme),
    ])
    .gap(8.0)
}

fn gpu_waterfall(frame: &FrameRecord, theme: &WidgetTheme) -> Element {
    let Some(gpu) = &frame.gpu else {
        return metric(
            if frame.adapter.timestamp_queries {
                "GPU results pending".into()
            } else {
                "GPU timestamps unavailable on this adapter".into()
            },
            theme,
        );
    };
    let total = gpu.total.as_secs_f64().max(f64::EPSILON);
    let timeline = gpu.passes.iter().take(12).map(|pass| {
        let start = (pass.start.as_secs_f64() / total).clamp(0.0, 1.0);
        let duration = (pass.duration.as_secs_f64() / total).clamp(0.0, 1.0 - start);
        Element::column([
            Element::row([
                Element::text(pass.label.clone())
                    .text_style(text(11.0, theme.foreground))
                    .grow(1.0),
                Element::text(format!(
                    "{:.3} ms · {} px",
                    millis(pass.duration),
                    pass.pixels
                ))
                .text_style(text(11.0, theme.foreground)),
            ]),
            Element::row([
                Element::container([]).width(percent(start as f32)),
                Element::container([])
                    .height(length(4.0))
                    .width(percent(duration.max(0.002) as f32))
                    .background(theme.primary)
                    .radius(CornerRadii::all(2.0)),
            ]),
        ])
        .gap(3.0)
    });
    let mut ranked = gpu.passes.iter().collect::<Vec<_>>();
    ranked.sort_by_key(|pass| std::cmp::Reverse(pass.duration));
    let ranking = ranked.into_iter().take(6).enumerate().map(|(index, pass)| {
        metric(
            format!(
                "{}. {} · {:.3} ms · {} px",
                index + 1,
                pass.label,
                millis(pass.duration),
                pass.pixels
            ),
            theme,
        )
    });
    Element::column(
        [Element::text("GPU timeline").text_style(text(11.0, theme.muted_foreground))]
            .into_iter()
            .chain(timeline)
            .chain([Element::text("Most expensive passes")
                .text_style(text(11.0, theme.muted_foreground))])
            .chain(ranking),
    )
    .padding(Sides::length(8.0))
    .gap(7.0)
    .background(theme.card)
}

fn toggle_button(open: bool, theme: &WidgetTheme) -> Element {
    if open {
        small_button("__devtools-toggle", "×", true, theme).inspectable(false)
    } else {
        Button::new(
            "__devtools-toggle",
            "DevTools",
            ButtonStyle::new(
                PaintStyle::new(QuadStyle::solid(theme.primary).radius(CornerRadii::all(7.0))),
                text(13.0, theme.primary_foreground),
            )
            .layout(LayoutStyle {
                padding: sides(14.0, 8.0),
                flex_shrink: 0.0,
                ..LayoutStyle::default()
            })
            .hovered(StateStyle::new().set(property::BackgroundColor, theme.primary))
            .pressed(StateStyle::new().set(property::BackgroundColor, theme.foreground)),
        )
        .build()
        .inspectable(false)
    }
}

fn tab_button(key: &str, label: &str, active: bool, theme: &WidgetTheme) -> Element {
    small_button(key, label, active, theme)
}

fn small_button(key: &str, label: &str, active: bool, theme: &WidgetTheme) -> Element {
    let base = if active { theme.primary } else { theme.muted };
    Button::new(
        key,
        label,
        ButtonStyle::new(
            PaintStyle::new(QuadStyle::solid(base).radius(CornerRadii::all(5.0))),
            text(
                12.0,
                if active {
                    theme.primary_foreground
                } else {
                    theme.foreground
                },
            ),
        )
        .layout(LayoutStyle {
            padding: sides(10.0, 6.0),
            flex_shrink: 0.0,
            ..LayoutStyle::default()
        })
        .hovered(StateStyle::new().set(property::BackgroundColor, theme.muted))
        .pressed(StateStyle::new().set(property::BackgroundColor, theme.primary)),
    )
    .build()
}

fn metric(label: String, theme: &WidgetTheme) -> Element {
    Element::text(label).text_style(text(12.0, theme.foreground))
}

fn text(size: f32, color: TextColor) -> TextStyle {
    TextStyle {
        font_size: size,
        color,
        wrap: TextWrap::None,
        ..TextStyle::default()
    }
}

fn scrollbar() -> ScrollbarStyle {
    ScrollbarStyle::new(
        ScrollbarPartStyle::new(QuadStyle::solid(Color::rgba(0.0, 0.0, 0.0, 0.18))),
        ScrollbarPartStyle::new(
            QuadStyle::solid(Color::rgb(0.30, 0.38, 0.48)).radius(CornerRadii::all(4.0)),
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
