#![cfg(not(target_arch = "wasm32"))]

use argui_devtools::{
    DevtoolsHost,
    telemetry::{DeviceTelemetry, DeviceTelemetryProvider},
};
use argui_runtime::{Context, Render};
use argui_ui::{Element, UiEvent, UiEventKind, UiTree};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use web_time::Instant;

#[cfg(feature = "all-smi")]
#[path = "telemetry/all_smi.rs"]
mod all_smi;

struct App;
impl Render for App {
    fn render(&mut self, _: &mut Context<Self>) -> Element {
        Element::text("Telemetry probe")
    }
}

struct Provider(Arc<AtomicUsize>, bool);
impl DeviceTelemetryProvider for Provider {
    fn name(&self) -> &str {
        "test provider"
    }
    fn sample(&mut self) -> Result<Vec<DeviceTelemetry>, String> {
        self.0.fetch_add(1, Ordering::SeqCst);
        if self.1 {
            return Err("driver unavailable".into());
        }
        std::thread::sleep(Duration::from_millis(20));
        Ok(vec![DeviceTelemetry {
            name: "Test GPU".into(),
            utilization_percent: Some(42.0),
            ..Default::default()
        }])
    }
}

fn click(host: &mut DevtoolsHost<App>, key: &str) {
    host.update(&UiEvent::new(
        UiTree::new(Element::container([])).node_ids()[0],
        Some(key.into()),
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    ));
}

fn tick(host: &mut DevtoolsHost<App>, elapsed: u64) {
    host.animation_frame(argui_animation::Frame {
        now: argui_animation::Time::ZERO,
        elapsed: argui_animation::Duration::from_millis(elapsed),
    });
}

fn wait_sample(host: &mut DevtoolsHost<App>) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while host.telemetry_snapshot().process.is_none() && host.telemetry_snapshot().errors.is_empty()
    {
        tick(host, 1);
        std::thread::sleep(Duration::from_millis(2));
        assert!(Instant::now() < deadline, "bounded telemetry response");
    }
}

#[test]
fn sensors_are_opt_in_bounded_and_stop_when_the_resource_view_is_inactive() {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut host = DevtoolsHost::new(App)
        .open(true)
        .device_telemetry(Provider(calls.clone(), false));
    for _ in 0..4 {
        tick(&mut host, 1000);
    }
    click(&mut host, "__devtools-profiling");
    for _ in 0..4 {
        tick(&mut host, 1000);
    }
    assert!(host.telemetry_snapshot().process.is_none());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    click(&mut host, "__devtools-profile-resources");
    click(&mut host, "__devtools-device-telemetry");
    wait_sample(&mut host);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(host.telemetry_snapshot().devices[0].name, "Test GPU");
    let process = host.telemetry_snapshot().process.as_ref().unwrap();
    assert!(process.resident_bytes > 0);
    assert_eq!(process.cpu_percent, None);
    if !process.categories.is_empty() {
        assert_eq!(
            process
                .categories
                .iter()
                .map(|category| category.resident_bytes)
                .sum::<u64>(),
            process.resident_bytes
        );
        for category in &process.categories {
            assert_eq!(
                category
                    .details
                    .iter()
                    .map(|(_, bytes)| *bytes)
                    .sum::<u64>(),
                category.resident_bytes
            );
        }
    }
    click(&mut host, "__devtools-pause");
    for _ in 0..4 {
        tick(&mut host, 1000);
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    click(&mut host, "__devtools-pause");
    click(&mut host, "__devtools-profile-details");
    for _ in 0..4 {
        tick(&mut host, 1000);
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    click(&mut host, "__devtools-device-telemetry");
    assert!(host.telemetry_snapshot().devices.is_empty());
    click(&mut host, "__devtools-toggle");
    for _ in 0..4 {
        tick(&mut host, 1000);
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn provider_failures_preserve_process_metrics_and_identify_the_failing_source() {
    let mut host = DevtoolsHost::new(App)
        .open(true)
        .device_telemetry(Provider(Arc::default(), true));
    click(&mut host, "__devtools-profiling");
    click(&mut host, "__devtools-profile-resources");
    click(&mut host, "__devtools-device-telemetry");
    wait_sample(&mut host);
    assert!(host.telemetry_snapshot().process.is_some());
    assert!(host.telemetry_snapshot().errors[0].contains("test provider: driver unavailable"));
    let root = argui_runtime::Entity::new(host).render();
    assert!(contains(&root, "driver unavailable"));
}

fn contains(root: &Element, text: &str) -> bool {
    matches!(&root.kind, argui_ui::ElementKind::Text {content, ..} if content.as_str().contains(text))
        || root.children.iter().any(|child| contains(child, text))
}

#[test]
fn resource_panel_exposes_measured_categories_allocator_capacities_and_driver_details() {
    let mut host = DevtoolsHost::new(App)
        .open(true)
        .device_telemetry(Provider(Arc::default(), false));
    click(&mut host, "__devtools-profiling");
    click(&mut host, "__devtools-profile-resources");
    click(&mut host, "__devtools-device-telemetry");
    wait_sample(&mut host);
    let names: Vec<_> = host
        .telemetry_snapshot()
        .process
        .as_ref()
        .unwrap()
        .categories
        .iter()
        .map(|category| category.name.clone())
        .collect();
    for name in &names {
        click(&mut host, &format!("__devtools-memory-category-{name}"));
    }
    click(&mut host, "__devtools-device-details-0");
    host.inspector()
        .publish_memory(argui_inspect::MemorySnapshot {
            ui_nodes: 257,
            layout_cache_bytes: 1024 * 1024,
            ..Default::default()
        });
    let root = argui_runtime::Entity::new(host).render();
    for text in [
        "Resident (RSS)",
        "257 UI nodes",
        "1.00 MiB",
        "Test GPU",
        "42.0%",
        "Unavailable",
        "Driver details",
        "worker",
    ] {
        assert!(contains(&root, text), "{text}");
    }
    for name in names {
        assert!(contains(&root, &name));
    }
}
