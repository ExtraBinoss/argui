use argui_inspect::FrameRecord;
use argui_ui::{Axes, Element, FlexWrap, Overflow, ScrollConfig, Sides, length, percent};
use argui_widgets::WidgetTheme;

use super::{
    DevtoolsHost, graph, icon_label_button, millis, profiling, profiling_list, small_button, text,
};

pub(crate) struct DetailCache {
    frame: FrameRecord,
    theme: WidgetTheme,
    panel: usize,
    height: f32,
    header_extent: f32,
    offset: f32,
    element: Element,
}

fn detail_content<A>(tools: &DevtoolsHost<A>, theme: &WidgetTheme) -> Element {
    let height = tools.profile_extents.gpu;
    let mut cache = tools.detail_cache.borrow_mut();
    if cache.as_ref().is_none_or(|cache| {
        cache.frame != tools.profile_details
            || cache.theme != *theme
            || cache.panel != tools.profile_panel
            || cache.header_extent != tools.profile_extents.gpu_header
            || cache.height != height
            || cache.offset != tools.gpu_offset
    }) {
        let element = if tools.profile_panel == 2 {
            profiling::details(&tools.profile_details, theme)
        } else {
            profiling::gpu_waterfall(
                &tools.profile_details,
                theme,
                height,
                tools.gpu_offset,
                tools.profile_extents.gpu_header,
                tools.scroll_effect.clone(),
            )
        };
        *cache = Some(DetailCache {
            frame: tools.profile_details.clone(),
            theme: theme.clone(),
            panel: tools.profile_panel,
            height,
            header_extent: tools.profile_extents.gpu_header,
            offset: tools.gpu_offset,
            element,
        });
    }
    cache
        .as_ref()
        .expect("detail cache was populated")
        .element
        .clone()
}

pub(super) fn panel<A>(tools: &DevtoolsHost<A>, theme: &WidgetTheme) -> Element {
    let empty = FrameRecord::default();
    let frame = tools
        .selected_frame
        .and_then(|index| tools.profile_frames.get(index))
        .or_else(|| tools.profile_frames.last())
        .unwrap_or(&empty);
    let wide = tools.panel_width() >= 760.0;
    let commands = Element::row([
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
        small_button(
            "__devtools-refresh",
            "Live",
            tools.selected_frame.is_none(),
            theme,
        ),
        small_button("__devtools-clear", "Clear", false, theme),
        icon_label_button("__devtools-copy", tools.icons.copy, "Export", theme),
    ])
    .keyed("__devtools-profile-controls")
    .gap(6.0)
    .flex_wrap(FlexWrap::Wrap)
    .padding(Sides::length(8.0))
    .shrink(0.0);
    let overview = || {
        let header = Element::column([
            Element::row([
                stat("Frame", format!("{:.2} ms", millis(frame.interval)), theme),
                stat("CPU", format!("{:.2} ms", millis(frame.total_cpu())), theme),
                stat(
                    "GPU",
                    frame
                        .gpu
                        .as_ref()
                        .map_or_else(|| "—".into(), |gpu| format!("{:.2} ms", millis(gpu.total))),
                    theme,
                ),
            ])
            .gap(6.0),
            graph::graph(&tools.profile_frames, theme),
        ])
        .gap(10.0);
        Element::container([profiling_list(tools, header, theme)])
            .keyed("__devtools-overview")
            .min_width(length(0.0))
            .min_height(length(0.0))
    };
    let details = || {
        let content = detail_content(tools, theme);
        if tools.profile_panel == 2 {
            scroll("__devtools-profile-details-body", content, theme)
        } else {
            Element::container([content])
                .keyed("__devtools-profile-details-body")
                .min_width(length(0.0))
                .min_height(length(0.0))
        }
    };
    let navigation = Element::row([
        small_button(
            "__devtools-profile-overview",
            "Overview",
            tools.profile_panel == 0,
            theme,
        ),
        small_button(
            "__devtools-profile-gpu",
            "GPU passes",
            tools.profile_panel == 1,
            theme,
        ),
        small_button(
            "__devtools-profile-details",
            "Frame details",
            tools.profile_panel == 2,
            theme,
        ),
    ])
    .keyed("__devtools-profile-navigation")
    .gap(4.0)
    .flex_wrap(FlexWrap::Wrap)
    .padding(Sides::length(8.0))
    .shrink(0.0);
    let body = if wide {
        Element::row([
            overview().width(percent(0.46)).shrink(0.0),
            Element::container([])
                .width(length(1.0))
                .background(theme.border)
                .shrink(0.0),
            details().grow(1.0),
        ])
    } else if tools.profile_panel == 0 {
        overview()
    } else {
        details()
    };
    let header = if wide {
        Element::row([
            commands.grow(1.0).shrink(1.0).min_width(length(0.0)),
            navigation,
        ])
        .align_items(argui_ui::AlignItems::CENTER)
    } else {
        Element::column([commands, navigation])
    };
    Element::column([header.shrink(0.0), body.grow(1.0).min_height(length(0.0))])
        .keyed("__devtools-profiling-panel")
        .min_width(length(0.0))
        .min_height(length(0.0))
}

fn scroll(key: &str, child: Element, theme: &WidgetTheme) -> Element {
    Element::column([child.shrink(0.0)])
        .keyed(key)
        .padding(Sides::length(10.0))
        .min_width(length(0.0))
        .min_height(length(0.0))
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        })
        .scroll_config(ScrollConfig::default().scrollbar(theme.scrollbar.clone()))
        .scrollbar_gutter(argui_ui::ScrollbarGutter::Stable)
}

fn stat(label: &str, value: String, theme: &WidgetTheme) -> Element {
    Element::column([
        Element::text(label).text_style(text(10.0, theme.muted_foreground)),
        Element::text(value).text_style(text(16.0, theme.foreground)),
    ])
    .gap(4.0)
    .padding(Sides::length(8.0))
    .background(theme.card)
    .grow(1.0)
    .min_width(length(0.0))
}
