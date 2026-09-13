use super::{millis, text};
use argui_inspect::FrameRecord;
use argui_ui::{Element, Sides, length, percent};
use argui_widgets::WidgetTheme;

pub(super) fn cpu_stages(frame: &FrameRecord, theme: &WidgetTheme) -> Element {
    let total = frame.total_cpu();
    let mut rows = vec![
        Element::text(format!("CPU stages · {:.2} ms total", millis(total)))
            .text_style(text(12.0, theme.foreground)),
    ];
    for (label, duration) in [
        ("Model", frame.model),
        ("Surface", frame.surface),
        ("Tree", frame.tree),
        ("Layout", frame.layout),
        ("Paint", frame.paint),
        ("Render encoding", frame.render_cpu),
    ] {
        let ratio = if total.is_zero() {
            0.0
        } else {
            duration.as_secs_f64() / total.as_secs_f64()
        };
        rows.push(
            Element::column([
                Element::row([
                    Element::text(label)
                        .text_style(text(11.0, theme.muted_foreground))
                        .grow(1.0),
                    Element::text(format!(
                        "{:.2} ms · {:.1}%",
                        millis(duration),
                        ratio * 100.0
                    ))
                    .text_style(text(11.0, theme.foreground)),
                ])
                .gap(8.0),
                Element::container([Element::container([])
                    .width(percent(ratio as f32))
                    .height(length(3.0))
                    .background(theme.primary)])
                .width(percent(1.0))
                .height(length(3.0))
                .background(theme.muted)
                .semantic_hidden(true),
            ])
            .gap(4.0),
        );
    }
    Element::column(rows)
        .gap(8.0)
        .padding(Sides::length(10.0))
        .background(theme.card)
}

pub(super) fn details(frame: &FrameRecord, theme: &WidgetTheme) -> Element {
    Element::column([
        cpu_stages(frame, theme),
        section(
            "Rendering & memory",
            [
                ("Invalidation", format!("{:?}", frame.update)),
                (
                    "Layers / cached",
                    format!("{} / {}", frame.layers, frame.cached_layers),
                ),
                (
                    "Textures / reused",
                    format!("{} / {}", frame.textures, frame.reused_textures),
                ),
                (
                    "Texture memory",
                    format!("{:.1} MiB", frame.texture_bytes as f64 / 1_048_576.0),
                ),
                ("Offscreen pixels", frame.offscreen_pixels.to_string()),
                (
                    "Vectors cached / hits / rasterized",
                    format!(
                        "{} / {} / {}",
                        frame.vector_atlas_entries,
                        frame.vector_atlas_hits,
                        frame.vector_rasterizations
                    ),
                ),
            ],
            theme,
        ),
        section(
            "Adapter",
            [
                ("Device", frame.adapter.name.clone()),
                ("Backend", frame.adapter.backend.clone()),
                (
                    "GPU timestamps",
                    if frame.adapter.timestamp_queries {
                        "Available"
                    } else {
                        "Unavailable"
                    }
                    .into(),
                ),
            ],
            theme,
        ),
    ])
    .gap(12.0)
}

fn section<const N: usize>(
    title: &str,
    values: [(&str, String); N],
    theme: &WidgetTheme,
) -> Element {
    Element::column(
        std::iter::once(Element::text(title).text_style(text(12.0, theme.foreground))).chain(
            values.into_iter().map(|(label, value)| {
                Element::row([
                    Element::text(label)
                        .text_style(text(11.0, theme.muted_foreground))
                        .width(percent(0.5)),
                    Element::text(value)
                        .text_style(text(11.0, theme.foreground))
                        .grow(1.0)
                        .min_width(length(0.0)),
                ])
                .gap(8.0)
            }),
        ),
    )
    .gap(8.0)
    .padding(Sides::length(10.0))
    .background(theme.card)
}

pub(super) fn gpu_waterfall(
    frame: &FrameRecord,
    theme: &WidgetTheme,
    viewport: f32,
    offset: f32,
    header_extent: f32,
    effect: Option<argui_ui::ScrollEffect>,
) -> Element {
    let Some(gpu) = &frame.gpu else {
        return Element::text(if frame.adapter.timestamp_queries {
            "GPU results pending"
        } else {
            "GPU timestamps unavailable on this adapter"
        })
        .text_style(text(12.0, theme.muted_foreground));
    };
    let total = gpu.total.as_secs_f64().max(f64::EPSILON);
    let header = Element::column([
        Element::text(format!(
            "GPU timeline · {} passes · {:.3} ms",
            gpu.passes.len(),
            millis(gpu.total)
        ))
        .text_style(text(12.0, theme.foreground)),
        Element::text("Position = start time · width = duration")
            .text_style(text(10.0, theme.muted_foreground)),
        Element::row([
            Element::text("Pass")
                .text_style(text(10.0, theme.muted_foreground))
                .width(percent(0.42)),
            Element::text("ms")
                .text_style(text(10.0, theme.muted_foreground))
                .width(length(46.0))
                .shrink(0.0),
            Element::row([
                Element::text("0").text_style(text(10.0, theme.muted_foreground)),
                Element::text(format!("{:.2} ms", millis(gpu.total)))
                    .text_style(text(10.0, theme.muted_foreground)),
            ])
            .grow(1.0)
            .justify_content(argui_ui::JustifyContent::SPACE_BETWEEN),
        ])
        .gap(8.0),
    ])
    .keyed("__devtools-gpu-header")
    .gap(8.0)
    .padding(Sides::length(10.0));

    let timeline = argui_widgets::VList::new("__devtools-gpu-passes", 28.0, viewport, offset)
        .effects(effect)
        .build_with_header(gpu.passes.len(), theme, header, header_extent, |index| {
            let pass = &gpu.passes[index];
            let start = (pass.start.as_secs_f64() / total).clamp(0.0, 1.0);
            let duration = (pass.duration.as_secs_f64() / total).clamp(0.0, 1.0 - start);
            let label = format!("{:02}  {}", index + 1, pass.label);
            let label_style = argui_text::TextStyle {
                wrap: argui_text::TextWrap::None,
                ..text(11.0, theme.foreground)
            };
            let track = Element::container([Element::container([])
                .absolute(Sides {
                    left: percent(start as f32),
                    right: argui_ui::auto(),
                    top: length(9.0),
                    bottom: argui_ui::auto(),
                })
                .width(percent(duration as f32))
                .height(length(6.0))
                .background(theme.primary)])
            .height(length(24.0))
            .grow(1.0)
            .min_width(length(0.0))
            .background(theme.muted);
            Element::row([
                Element::text(label)
                    .text_style(label_style)
                    .width(percent(0.42))
                    .min_width(length(0.0))
                    .overflow(argui_ui::Axes {
                        x: argui_ui::Overflow::Hidden,
                        y: argui_ui::Overflow::Hidden,
                    }),
                Element::text(format!("{:.3}", millis(pass.duration)))
                    .text_style(text(11.0, theme.foreground))
                    .width(length(46.0))
                    .shrink(0.0),
                track,
            ])
            .keyed(format!("__devtools-gpu-pass-{index}"))
            .padding(argui_ui::sides(10.0, 0.0))
            .height(length(28.0))
            .shrink(0.0)
            .gap(8.0)
            .align_items(argui_ui::AlignItems::CENTER)
            .semantics(
                argui_ui::Semantics::new(argui_ui::Role::Group)
                    .label(&pass.label)
                    .description(format!(
                        "{:.3} ms · {} px · {:?} {:?}",
                        millis(pass.duration),
                        pass.pixels,
                        pass.object_domain,
                        pass.object_id
                    )),
            )
        });
    Element::container([timeline.height(percent(1.0)).min_height(length(0.0))])
        .keyed("__devtools-gpu-timeline")
        .height(percent(1.0))
        .min_height(length(0.0))
}
