use argui_core::{Rect, Transform2D, TransformOrigin};
use argui_inspect::{FrameRecord, NodeSnapshot};
use argui_paint::{Border, ClipBehavior, Color, CornerRadii, PaintStyle, QuadStyle, VectorId};
use argui_runtime::UiApp;
use argui_text::{TextColor, TextStyle, TextWrap};
use argui_ui::{
    Align, Button, ButtonStyle, Edges, Element, Inset, Interaction, LayoutStyle, Length,
    ScrollConfig, ScrollbarStyle, TextInput, TextInputStyle, VirtualList,
};

use crate::host::{DevtoolsHost, Tab};

const BG: Color = Color::rgb(0.055, 0.065, 0.085);
const PANEL: Color = Color::rgb(0.075, 0.09, 0.12);
const LINE: Color = Color::rgb(0.18, 0.22, 0.29);
const TEXT: TextColor = TextColor::rgb(0.85, 0.89, 0.95);
const ACCENT: Color = Color::rgb(0.25, 0.72, 0.96);
const RETAINED_TREE_LIMIT: usize = 512;

pub(crate) fn host<A: UiApp>(tools: &DevtoolsHost<A>) -> Element {
    let application = Element::container([tools
        .app
        .view()
        .width(Length::Percent(1.0))
        .height(Length::Percent(1.0))
        .min_height(Length::Px(0.0))])
    .keyed("__devtools-app-root")
    .grow(1.0)
    .shrink(1.0)
    .min_height(Length::Px(0.0))
    .clip(ClipBehavior::Bounds);
    let mut children = vec![application];
    if tools.open || tools.sheet_progress > 0.001 {
        children.push(dock_surface(tools));
    } else {
        children.push(
            toggle_button(false)
                .absolute(Inset::top_left(14.0, 14.0))
                .z_index(30_000),
        );
    }
    if let Some(highlight) = selection_highlight(tools) {
        children.push(highlight);
    }
    if tools.picking {
        children.push(picker_surface(tools));
    }
    Element::column(children)
        .width(Length::Percent(1.0))
        .height(Length::Percent(1.0))
        .clip(ClipBehavior::Bounds)
}

fn dock_surface<A: UiApp>(tools: &DevtoolsHost<A>) -> Element {
    let height = tools.dock_height + 6.0;
    let surface = Element::column([splitter(), dock(tools, tools.dock_height)])
        .keyed("__devtools-surface-content")
        .width(Length::Percent(1.0))
        .height(Length::Px(height))
        .shrink(0.0);
    Element::container([surface])
        .keyed("__devtools-surface")
        .width(Length::Percent(1.0))
        .height(Length::Px(height * tools.sheet_progress))
        .shrink(0.0)
        .clip(ClipBehavior::Bounds)
        .inspectable(false)
}

fn splitter() -> Element {
    Element::container([])
        .keyed("__devtools-splitter")
        .height(Length::Px(6.0))
        .shrink(0.0)
        .background(ACCENT)
        .interaction(
            Interaction::default()
                .hovered(QuadStyle::solid(Color::rgb(0.45, 0.84, 1.0)))
                .pressed(QuadStyle::solid(Color::WHITE)),
        )
        .inspectable(false)
}

fn dock<A: UiApp>(tools: &DevtoolsHost<A>, height: f32) -> Element {
    let body = match tools.tab {
        Tab::Elements => elements_tab(tools),
        Tab::Profiling => profiling_tab(tools),
    };
    Element::column([toolbar(tools), body.grow(1.0).shrink(1.0)])
        .height(Length::Px(height))
        .shrink(0.0)
        .background(BG)
        .border(Border::all(1.0, LINE))
        .clip(ClipBehavior::Bounds)
        .inspectable(false)
}

fn toolbar<A>(tools: &DevtoolsHost<A>) -> Element {
    Element::row([
        toggle_button(true),
        tab_button(
            "__devtools-elements",
            "Elements",
            tools.tab == Tab::Elements,
        ),
        tab_button(
            "__devtools-profiling",
            "Profiling",
            tools.tab == Tab::Profiling,
        ),
        picker_button(tools),
        morph_button(tools),
    ])
    .height(Length::Px(42.0))
    .shrink(0.0)
    .gap(6.0)
    .padding(Edges::symmetric(8.0, 6.0))
    .background(PANEL)
    .align(Align::Center)
}

fn picker_button<A>(tools: &DevtoolsHost<A>) -> Element {
    icon_label_button(
        "__devtools-picker",
        tools.icons.target,
        if tools.picking {
            "Picking…"
        } else {
            "Select"
        },
    )
    .background(if tools.picking {
        Color::rgb(0.12, 0.32, 0.46)
    } else {
        Color::rgb(0.10, 0.12, 0.16)
    })
}

fn picker_surface<A>(tools: &DevtoolsHost<A>) -> Element {
    Element::container([])
        .keyed("__devtools-picker-surface")
        .absolute(Inset::top_left(
            tools.app_viewport.origin.y,
            tools.app_viewport.origin.x,
        ))
        .width(Length::Px(tools.app_viewport.size.width.max(1.0)))
        .height(Length::Px(tools.app_viewport.size.height.max(1.0)))
        .z_index(19_000)
        .interaction(Interaction::blocker())
        .inspectable(false)
}

fn morph_button<A>(tools: &DevtoolsHost<A>) -> Element {
    Element::row([
        icon_element(tools.icons.chevron, 20.0).vector_progress(tools.morph_progress),
        Element::text("GPU morph").text_style(text(12.0, TEXT)),
    ])
    .keyed("__devtools-morph")
    .height(Length::Px(30.0))
    .padding(Edges::symmetric(8.0, 4.0))
    .gap(5.0)
    .align(Align::Center)
    .background(BG)
    .interaction(Interaction::default().hovered(QuadStyle::solid(Color::rgb(0.15, 0.20, 0.27))))
}

fn elements_tab<A: UiApp>(tools: &DevtoolsHost<A>) -> Element {
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
                tree_row(node, selected == Some(node.id))
            },
        );
        let search = TextInput::new(
            "__devtools-search",
            &tools.search,
            "Filter elements…",
            TextInputStyle::new(
                PaintStyle::new(
                    QuadStyle::solid(Color::rgb(0.045, 0.055, 0.075)).radius(CornerRadii::all(5.0)),
                ),
                text(12.0, TEXT),
            )
            .focused(QuadStyle::solid(Color::rgb(0.08, 0.16, 0.22))),
        )
        .build();
        Element::row([
            Element::column([search, list.grow(1.0).shrink(1.0)])
                .grow(1.0)
                .shrink(1.0)
                .padding(Edges::all(6.0))
                .gap(6.0),
            crate::style::sidebar(selected, &snapshot.nodes, tools),
        ])
        .height(Length::Percent(1.0))
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

fn tree_row(node: &NodeSnapshot, selected: bool) -> Element {
    let identity = node
        .key
        .as_deref()
        .map_or_else(|| node.kind.clone(), |key| format!("{}  #{key}", node.kind));
    let label = node.summary.as_deref().map_or(identity.clone(), |summary| {
        format!("{identity}  —  {summary}")
    });
    Element::text(format!("{}{}", "  ".repeat(node.depth), label))
        .keyed(format!("__devtools-node-{}", node.id.0))
        .padding(Edges::symmetric(8.0, 4.0))
        .background(if selected {
            Color::rgb(0.12, 0.32, 0.46)
        } else {
            BG
        })
        .border(Border::all(0.5, LINE))
        .text_style(text(13.0, if selected { TextColor::WHITE } else { TEXT }))
        .interaction(Interaction::default().hovered(QuadStyle::solid(Color::rgb(0.11, 0.15, 0.21))))
}

fn profiling_tab<A>(tools: &DevtoolsHost<A>) -> Element {
    let frames = &tools.profile_frames;
    let latest = frames.last().copied().unwrap_or_default();
    let bars = frames
        .iter()
        .rev()
        .take(60)
        .rev()
        .map(|frame| frame_bar(*frame));
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
            ),
            small_button("__devtools-clear", "Clear", false),
            small_button("__devtools-refresh", "Refresh", false),
            icon_label_button("__devtools-copy", tools.icons.copy, "Copy trace"),
            metric(format!("frame {:.2} ms", millis(latest.interval))),
            metric(format!("CPU {:.2} ms", millis(latest.total_cpu()))),
            metric(format!("{} passes", latest.passes)),
            metric(format!(
                "{:.1} MiB",
                latest.texture_bytes as f64 / 1_048_576.0
            )),
        ])
        .wrap(argui_ui::Wrap::Wrap)
        .gap(8.0),
        Element::row(bars)
            .height(Length::Px(110.0))
            .align(Align::End)
            .gap(2.0)
            .padding(Edges::all(8.0))
            .background(PANEL),
        details(latest),
        profiling_list(frames, tools.profiling_offset),
    ])
    .padding(Edges::all(12.0))
    .gap(12.0)
}

fn profiling_list(frames: &[FrameRecord], offset: f32) -> Element {
    let ordered = frames.iter().rev().copied().collect::<Vec<_>>();
    profiling_list_config(ordered.len()).build("__devtools-frames", offset, |index| {
        let frame = ordered[index];
        Element::text(format!(
            "#{:03}  {:>6.2} ms · CPU {:>6.2} · {:?} · {} passes · {:.1} MiB",
            frames.len().saturating_sub(index),
            millis(frame.interval),
            millis(frame.total_cpu()),
            frame.update,
            frame.passes,
            frame.texture_bytes as f64 / 1_048_576.0,
        ))
        .padding(Edges::symmetric(7.0, 3.0))
        .background(if index % 2 == 0 { PANEL } else { BG })
        .text_style(text(11.0, TEXT))
    })
}

pub(crate) fn profiling_list_config(item_count: usize) -> VirtualList {
    VirtualList::new(item_count, 27.0, 130.0)
        .scroll_config(ScrollConfig::default().scrollbar(scrollbar()))
}

fn icon_label_button(key: &str, icon: VectorId, label: &str) -> Element {
    Element::row([
        icon_element(icon, 16.0),
        Element::text(label).text_style(text(12.0, TEXT)),
    ])
    .keyed(key)
    .align(Align::Center)
    .gap(6.0)
    .padding(Edges::symmetric(9.0, 6.0))
    .background(Color::rgb(0.10, 0.12, 0.16))
    .radius(CornerRadii::all(5.0))
    .interaction(Interaction::default().hovered(QuadStyle::solid(Color::rgb(0.16, 0.22, 0.30))))
}

fn icon_element(icon: VectorId, size: f32) -> Element {
    Element::vector(icon)
        .width(Length::Px(size))
        .height(Length::Px(size))
        .shrink(0.0)
}

fn frame_bar(frame: FrameRecord) -> Element {
    let milliseconds = millis(frame.interval).max(millis(frame.total_cpu()));
    let missed = milliseconds > 18.0;
    Element::container([])
        .width(Length::Px(5.0))
        .height(Length::Px((milliseconds * 3.0).clamp(2.0, 94.0) as f32))
        .shrink(0.0)
        .background(if missed {
            Color::rgb(1.0, 0.28, 0.24)
        } else {
            ACCENT
        })
        .radius(CornerRadii::all(2.0))
}

fn details(frame: FrameRecord) -> Element {
    Element::column([
        metric(format!(
            "model {:.2} · tree {:.2} · paint {:.2} · render {:.2} ms",
            millis(frame.model),
            millis(frame.tree),
            millis(frame.paint),
            millis(frame.render_cpu)
        )),
        metric(format!(
            "invalidation {:?} · layers {} · offscreen {} px",
            frame.update, frame.layers, frame.offscreen_pixels
        )),
        metric(format!(
            "textures {} · reused this frame {}",
            frame.textures, frame.reused_textures
        )),
    ])
    .gap(8.0)
}

fn selection_highlight<A>(tools: &DevtoolsHost<A>) -> Option<Element> {
    let selected = tools
        .picker_hovered
        .filter(|_| tools.picking)
        .or_else(|| tools.inspector.selected());
    let bounds = selected
        .and_then(|selected| tools.inspector.node(selected))
        .map_or_else(Rect::default, |node| node.bounds);
    if selected.is_none() && !tools.picking {
        return None;
    }
    let x = bounds.origin.x;
    let y = bounds.origin.y;
    let width = bounds.size.width.max(0.0);
    let height = bounds.size.height.max(0.0);
    Some(
        Element::container([
            highlight_piece(
                Color::rgba(0.2, 0.72, 1.0, 0.10),
                Transform2D::IDENTITY.translate(x, y).scale(width, height),
            ),
            highlight_piece(
                ACCENT,
                Transform2D::IDENTITY.translate(x, y).scale(width, 2.0),
            ),
            highlight_piece(
                ACCENT,
                Transform2D::IDENTITY
                    .translate(x, y + (height - 2.0).max(0.0))
                    .scale(width, 2.0),
            ),
            highlight_piece(
                ACCENT,
                Transform2D::IDENTITY.translate(x, y).scale(2.0, height),
            ),
            highlight_piece(
                ACCENT,
                Transform2D::IDENTITY
                    .translate(x + (width - 2.0).max(0.0), y)
                    .scale(2.0, height),
            ),
        ])
        .keyed("__devtools-selection-highlight")
        .absolute(Inset::top_left(0.0, 0.0))
        .width(Length::Px(1.0))
        .height(Length::Px(1.0))
        .z_index(20_000)
        .inspectable(false),
    )
}

fn highlight_piece(color: Color, transform: Transform2D) -> Element {
    Element::container([])
        .absolute(Inset::top_left(0.0, 0.0))
        .width(Length::Px(1.0))
        .height(Length::Px(1.0))
        .background(color)
        .transform(transform)
        .transform_origin(TransformOrigin::TOP_LEFT)
        .inspectable(false)
}

fn toggle_button(open: bool) -> Element {
    if open {
        small_button("__devtools-toggle", "×", true).inspectable(false)
    } else {
        Button::new(
            "__devtools-toggle",
            "DevTools",
            ButtonStyle::new(
                PaintStyle::new(QuadStyle::solid(ACCENT)),
                text(13.0, TextColor::rgb(0.02, 0.05, 0.08)),
            )
            .layout(LayoutStyle {
                padding: Edges::symmetric(14.0, 8.0),
                shrink: 0.0,
                ..LayoutStyle::default()
            })
            .hovered(QuadStyle::solid(Color::rgb(0.48, 0.86, 1.0)))
            .pressed(QuadStyle::solid(Color::WHITE)),
        )
        .build()
        .radius(CornerRadii::all(7.0))
        .inspectable(false)
    }
}

fn tab_button(key: &str, label: &str, active: bool) -> Element {
    small_button(key, label, active)
}

fn small_button(key: &str, label: &str, active: bool) -> Element {
    let base = if active {
        Color::rgb(0.12, 0.32, 0.46)
    } else {
        Color::rgb(0.10, 0.12, 0.16)
    };
    Button::new(
        key,
        label,
        ButtonStyle::new(PaintStyle::new(QuadStyle::solid(base)), text(12.0, TEXT))
            .layout(LayoutStyle {
                padding: Edges::symmetric(10.0, 6.0),
                shrink: 0.0,
                ..LayoutStyle::default()
            })
            .hovered(QuadStyle::solid(Color::rgb(0.16, 0.22, 0.30)))
            .pressed(QuadStyle::solid(ACCENT)),
    )
    .build()
    .radius(CornerRadii::all(5.0))
}

fn metric(label: String) -> Element {
    Element::text(label).text_style(text(12.0, TEXT))
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
        QuadStyle::solid(Color::rgba(0.0, 0.0, 0.0, 0.18)),
        QuadStyle::solid(Color::rgb(0.30, 0.38, 0.48)).radius(CornerRadii::all(4.0)),
    )
    .width(8.0)
    .inset(3.0)
}

fn millis(duration: std::time::Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}
