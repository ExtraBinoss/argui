use argui_inspect::FrameRecord;
use argui_ui::{Element, FlexWrap, Sides, length, percent};
use argui_widgets::{Button, Collapsible, WidgetTheme};

use super::{profiling, text};
use crate::{host::DevtoolsHost, telemetry::DeviceTelemetry};

pub(super) fn panel<A>(
    tools: &DevtoolsHost<A>,
    frame: &FrameRecord,
    theme: &WidgetTheme,
) -> Element {
    let telemetry = &tools.telemetry;
    let snapshot = &telemetry.snapshot;
    let mut rows = vec![
        profiling::cpu_stages(frame, theme),
        title("Process RAM", theme),
    ];
    if let Some(process) = &snapshot.process {
        rows.extend([
            row("Resident (RSS)", bytes(process.resident_bytes), theme),
            row(
                "Private resident",
                optional_bytes(process.private_bytes),
                theme,
            ),
            row(
                "Proportional (PSS)",
                optional_bytes(process.proportional_bytes),
                theme,
            ),
            row("Virtual address space", bytes(process.virtual_bytes), theme),
            row(
                "Process CPU · 100% = one logical CPU",
                process
                    .cpu_percent
                    .map_or("Waiting for next sample".into(), |value| {
                        format!("{value:.1}%")
                    }),
                theme,
            ),
        ]);
        rows.push(note("RSS includes shared resident pages. PSS weights shared pages by their users. Virtual address space is not RAM usage.", theme));
        for category in &process.categories {
            let ratio = if process.resident_bytes == 0 {
                0.0
            } else {
                category.resident_bytes as f64 / process.resident_bytes as f64
            };
            let expanded = telemetry.expanded.contains(&category.name);
            let content = if expanded {
                Element::column(
                    category
                        .details
                        .iter()
                        .map(|(name, value)| row(name, bytes(*value), theme)),
                )
                .gap(8.0)
                .padding(Sides::length(8.0))
            } else {
                Element::container([])
            };
            rows.push(
                Collapsible::new(
                    format!("__devtools-memory-category-{}", category.name),
                    format!(
                        "{} · {} · {:.1}%",
                        category.name,
                        bytes(category.resident_bytes),
                        ratio * 100.0
                    ),
                    expanded,
                    content,
                )
                .build(theme),
            );
        }
        if process.categories.is_empty() {
            rows.push(note(
                "The OS does not provide a resident mapping breakdown here.",
                theme,
            ));
        } else {
            rows.push(note("These categories sum to resident RAM. Anonymous mappings also contain allocator arenas and caches; they are not attributable to individual widgets.", theme));
        }
    } else {
        rows.push(note(
            if cfg!(target_arch = "wasm32") {
                "Process memory and hardware sensors are unavailable in the browser sandbox."
            } else {
                "Waiting for the first process sample…"
            },
            theme,
        ));
    }
    rows.extend([title("Known Argui CPU allocations", theme)]);
    if let Some(memory) = tools.inspector.memory() {
        for (label, value) in [
            ("UI traversal columns", memory.ui_index_bytes),
            ("Layout metadata", memory.layout_metadata_bytes),
            ("Taffy measurement caches", memory.layout_cache_bytes),
            ("Layout geometry", memory.layout_geometry_bytes),
            ("Layout output buffers", memory.layout_output_bytes),
            ("Paint command buffer", memory.paint_command_bytes),
        ] {
            rows.push(row(label, bytes(value as u64), theme));
        }
        rows.push(note(&format!("{} UI nodes · {} layout nodes. These capacity counters exclude shared element data, nested payloads, fonts, images, hash tables and allocator overhead. They are not a decomposition of RSS.", memory.ui_nodes, memory.layout_nodes), theme));
    } else {
        rows.push(note("Waiting for the runtime's capacity snapshot…", theme));
    }
    rows.extend([
        title("Argui GPU allocations", theme),
        row("Texture pool", bytes(frame.texture_bytes.saturating_sub(frame.vector_atlas_bytes).saturating_sub(frame.gpu_canvas_bytes)), theme),
        row("Vector atlas", bytes(frame.vector_atlas_bytes), theme),
        row("GPU canvases", bytes(frame.gpu_canvas_bytes), theme),
        row("Tracked GPU total", bytes(frame.texture_bytes), theme),
        note("Renderer allocations are separate from process RAM and device-wide sensor usage. Driver allocations and other GPU resources are not included in this tracked total.", theme),
        title("Hardware sensors", theme),
    ]);
    if let Some(source) = &telemetry.source {
        rows.push(
            Button::new(
                "__devtools-device-telemetry",
                format!(
                    "{} {source} sensors",
                    if telemetry.devices_enabled {
                        "Disable"
                    } else {
                        "Enable"
                    }
                ),
                theme.outline_button(),
            )
            .build(),
        );
        if telemetry.devices_enabled && snapshot.devices.is_empty() {
            rows.push(note(
                "No hardware sample available. Check the provider status below.",
                theme,
            ));
        }
    } else {
        rows.push(note("Configure a DeviceTelemetryProvider, or enable the all-smi feature and install its executable, to collect GPU sensors.", theme));
    }
    for (index, device) in snapshot.devices.iter().enumerate() {
        let key = if device.identifier.is_empty() {
            index.to_string()
        } else {
            device.identifier.clone()
        };
        rows.push(device_card(
            device,
            &key,
            telemetry.expanded.contains(&key),
            theme,
        ));
    }
    for error in &snapshot.errors {
        rows.push(Element::text(error.as_str()).text_style(text(12.0, theme.destructive)));
    }
    if telemetry.samples > 0 {
        rows.push(note(
            &format!(
                "Sample {} · {:.2} ms collection on a worker · at most once per second",
                telemetry.samples,
                snapshot.collection_time.as_secs_f64() * 1000.0
            ),
            theme,
        ));
    }
    Element::column(rows)
        .gap(10.0)
        .width(percent(1.0))
        .min_width(length(0.0))
}

fn device_card(
    device: &DeviceTelemetry,
    key: &str,
    expanded: bool,
    theme: &WidgetTheme,
) -> Element {
    let detail_rows = if expanded {
        Element::column(
            device
                .details
                .iter()
                .map(|(name, value)| row(name, value.clone(), theme)),
        )
        .gap(6.0)
    } else {
        Element::container([])
    };
    Element::column([
        title(&device.name, theme),
        row("GPU utilization", number(device.utilization_percent, "%"), theme),
        row("Device memory · used / total", format!("{} / {}", optional_bytes(device.used_memory_bytes), optional_bytes(device.total_memory_bytes)), theme),
        row("Temperature", number(device.temperature_celsius, " °C"), theme),
        row("Power", number(device.power_watts, " W"), theme),
        row("Clock", number(device.frequency_mhz, " MHz"), theme),
        Collapsible::new(format!("__devtools-device-details-{key}"), "Driver details", expanded, detail_rows).build(theme),
        note("Device-wide values are reported by the provider. Unsupported all-smi fields may be zero; available sensors depend on the OS, hardware and driver.", theme),
    ]).gap(8.0).padding(Sides::length(10.0)).background(theme.card)
}

fn number(value: Option<f64>, unit: &str) -> String {
    value.map_or("Unavailable".into(), |value| format!("{value:.1}{unit}"))
}

fn optional_bytes(value: Option<u64>) -> String {
    value.map_or("Unavailable".into(), bytes)
}

fn bytes(value: u64) -> String {
    format!("{:.2} MiB", value as f64 / 1_048_576.0)
}

fn title(value: &str, theme: &WidgetTheme) -> Element {
    Element::text(value).text_style(text(13.0, theme.foreground))
}

fn note(value: &str, theme: &WidgetTheme) -> Element {
    Element::text(value).text_style(text(11.0, theme.muted_foreground))
}

fn row(label: &str, value: String, theme: &WidgetTheme) -> Element {
    Element::row([
        Element::text(label)
            .text_style(text(11.0, theme.muted_foreground))
            .grow(1.0)
            .min_width(length(0.0)),
        Element::text(value)
            .text_style(text(11.0, theme.foreground))
            .min_width(length(0.0)),
    ])
    .gap(8.0)
    .flex_wrap(FlexWrap::Wrap)
}
