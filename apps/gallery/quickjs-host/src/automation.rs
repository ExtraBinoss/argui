//! Windowless QuickJS test execution over the existing native bridge.

use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    error::Error,
    fs,
    io::Read,
    path::{Path, PathBuf},
    rc::Rc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use argui_automation::{
    Action, ActionWindow, Driver, Viewport, frame_diagnostics, summarize_metrics,
};
use argui_core::MetricTrace;
use argui_render::{RendererConfig, SurfaceRenderer};
use argui_runtime::WireOperation;
use image::{ColorType, ImageFormat};
use serde::Serialize;
use serde_json::{Value, json};

use crate::{QuickJsGallery, decode_wire_operations, delivery::event_json};

#[derive(Debug)]
struct Request {
    id: i32,
    method: String,
    target: String,
    args: String,
}

/// Monotonic origin and timeout shared by queued test actions.
#[derive(Clone, Copy)]
struct TestClock {
    started: Instant,
    deadline: Instant,
}

#[derive(Serialize)]
struct Step {
    name: String,
    started_ms: f64,
    duration_ms: f64,
    gap_ms: f64,
    trace_id: usize,
    ok: bool,
    error: Option<String>,
    artifact: Option<String>,
}

/// Runs a bundled TypeScript test, writes `report.json`, and returns its status.
/// The test module imports the real app and mounts it in one QuickJS context.
///
/// # Errors
/// Returns an error for a failed assertion, timeout, build artifact, or render.
pub(crate) fn run() -> Result<(), Box<dyn Error>> {
    if std::env::var_os("ARGUI_AUTOMATION_GATE").is_some() {
        let mut gate = [0_u8; 1];
        std::io::stdin().read_exact(&mut gate)?;
    }
    let started_unix_ms = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs_f64() * 1000.0;
    let source_path = PathBuf::from(std::env::var("ARGUI_AUTOMATION_TEST")?);
    let out = PathBuf::from(std::env::var("ARGUI_AUTOMATION_OUT")?);
    fs::create_dir_all(&out)?;
    let source = fs::read_to_string(&source_path)?;
    let contract = include_str!("../../../../packages/host/src/contract.generated.json");
    let viewport = Viewport::default();
    let driver = Rc::new(RefCell::new(Driver::new(viewport)?));
    let metrics = driver.borrow().metrics();
    let queue = Rc::new(RefCell::new(VecDeque::new()));
    let next_id = Rc::new(Cell::new(1_i32));
    let commits = Rc::clone(&driver);
    let requests = Rc::clone(&queue);
    let ids = Rc::clone(&next_id);
    let started = Instant::now();
    let mut steps = Vec::new();
    let mut adapter: Option<Value> = None;
    let result = (|| -> Result<(), String> {
        let mount = metrics.span("js.mount");
        let gallery = QuickJsGallery::new_automation(
            &source,
            contract,
            move |wire| {
                (|| -> Result<(), String> {
                    let operations = decode_wire_operations(&wire)?
                        .into_iter()
                        .map(WireOperation::into_native)
                        .collect::<Result<Vec<_>, _>>()?;
                    commits.borrow_mut().commit(&operations)
                })()
                .err()
                .unwrap_or_default()
            },
            move |method, target, args| {
                let id = ids.get();
                ids.set(id.saturating_add(1));
                requests.borrow_mut().push_back(Request {
                    id,
                    method,
                    target,
                    args,
                });
                id
            },
        )?;
        let viewport: Viewport = serde_json::from_str(&gallery.automation_viewport()?)
            .map_err(|error| format!("invalid test viewport: {error}"))?;
        driver.borrow_mut().set_viewport(viewport)?;
        gallery.start_automation()?;
        drop(mount);
        let timeout = std::env::var("ARGUI_TEST_TIMEOUT_MS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(30_000)
            .clamp(100, 300_000);
        let deadline = started + Duration::from_millis(timeout);
        let clock = TestClock { started, deadline };
        let mut renderer = None;
        loop {
            if Instant::now() >= deadline {
                return Err(format!(
                    "test timed out after {timeout} ms; check pending app work or increase ARGUI_TEST_TIMEOUT_MS"
                ));
            }
            let request = { queue.borrow_mut().pop_front() };
            if let Some(request) = request {
                let step_started_ms = metrics.now_ms();
                let gap_ms = steps.last().map_or(step_started_ms, |last: &Step| {
                    (step_started_ms - last.started_ms - last.duration_ms).max(0.0)
                });
                let action = metrics.span("automation.action");
                let trace_id = action.id();
                let outcome = execute(
                    &request,
                    &driver,
                    &gallery,
                    &out,
                    &mut renderer,
                    &mut adapter,
                    clock,
                );
                drop(action);
                let error = outcome.as_ref().err().cloned();
                steps.push(Step {
                    name: format!("{} {}", request.method, request.target)
                        .trim()
                        .to_owned(),
                    started_ms: step_started_ms,
                    duration_ms: metrics.now_ms() - step_started_ms,
                    gap_ms,
                    trace_id,
                    ok: error.is_none(),
                    error: error.clone(),
                    artifact: (request.method == "screenshot" && error.is_none())
                        .then(|| {
                            serde_json::from_str::<Value>(&request.args)
                                .ok()
                                .and_then(|args| args["name"].as_str().map(str::to_owned))
                        })
                        .flatten(),
                });
                println!(
                    "argui test: {} {}",
                    request.method,
                    if error.is_some() { "failed" } else { "ok" }
                );
                let _resolve = metrics.span("js.resolve_action");
                gallery.resolve_automation(
                    request.id,
                    error.as_deref().unwrap_or(""),
                    outcome.as_deref().unwrap_or(""),
                )?;
                continue;
            }
            let state: Value = serde_json::from_str(&gallery.automation_state()?)
                .map_err(|error| format!("invalid test state: {error}"))?;
            if state["done"] == true {
                let error = state["error"].as_str().unwrap_or("");
                if error.is_empty() {
                    return Ok(());
                }
                return Err(error.to_owned());
            }
            driver.borrow_mut().advance()?;
            deliver_pending(&driver, &gallery, &metrics)?;
            let elapsed = started.elapsed().as_secs_f64() * 1000.0;
            let tick = metrics.span("js.tick");
            gallery.tick(elapsed)?;
            drop(tick);
            let wait = gallery.next_wake(elapsed)?.min(Duration::from_millis(10));
            if !wait.is_zero() {
                std::thread::sleep(wait);
            }
        }
    })();
    let report = report(&driver.borrow(), &steps, adapter, &result, started_unix_ms);
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    result.map_err(Into::into)
}

/// Executes one queued test action and returns its JSON result.
/// `request` names the typed action; `driver` owns the retained UI; `gallery`
/// receives callback deliveries; `out` bounds screenshot files; `renderer`
/// lazily owns the GPU; `adapter` records its identity; `clock` bounds an
/// explicit wait while JavaScript timers keep running.
///
/// # Errors
/// Returns a target, assertion, GPU, or file error with the failing step named.
fn execute(
    request: &Request,
    driver: &Rc<RefCell<Driver>>,
    gallery: &QuickJsGallery,
    out: &Path,
    renderer: &mut Option<SurfaceRenderer>,
    adapter: &mut Option<Value>,
    clock: TestClock,
) -> Result<String, String> {
    let trace = driver.borrow().metrics();
    let args: Value = serde_json::from_str(&request.args)
        .map_err(|error| format!("{} arguments: {error}", request.method))?;
    let string = |key: &str| {
        args[key]
            .as_str()
            .ok_or_else(|| format!("{} needs {key}", request.method))
    };
    match request.method.as_str() {
        "click" => driver.borrow_mut().act(Action::Click {
            target: request.target.clone(),
            right: args["button"] == "right",
        })?,
        "scroll" => driver.borrow_mut().act(Action::Scroll {
            target: request.target.clone(),
            x: number(&args, "x")?,
            y: number(&args, "y")?,
            lines: args["unit"] == "lines",
        })?,
        "fill" => driver.borrow_mut().act(Action::Fill {
            target: request.target.clone(),
            value: string("value")?.to_owned(),
        })?,
        "key" => driver.borrow_mut().act(Action::Key {
            value: string("key")?.to_owned(),
        })?,
        "drag" => driver.borrow_mut().act(Action::Drag {
            from: request.target.clone(),
            to: string("to")?.to_owned(),
        })?,
        "expectText" => driver.borrow().expect_text(string("text")?)?,
        "wait" => {
            let ms = args["ms"]
                .as_f64()
                .filter(|value| value.is_finite() && (0.0..=300_000.0).contains(value))
                .ok_or("wait needs a finite duration from 0 to 300000 ms")?;
            drive_wait(
                driver,
                gallery,
                &trace,
                clock,
                Duration::from_secs_f64(ms / 1000.0),
            )?;
        }
        "performance" => assert_performance(&driver.borrow(), &args)?,
        "screenshot" => {
            let _capture = trace.span("render.capture");
            let name = string("name")?;
            let path = safe_capture_path(out, name)?;
            let mut owned = driver.borrow_mut();
            let viewport = owned.viewport();
            let (width, height) = viewport.physical_size()?;
            if renderer.is_none() {
                let _init = trace.span("render.adapter_init");
                *renderer = Some(
                    pollster::block_on(SurfaceRenderer::new_offscreen(
                        width,
                        height,
                        RendererConfig::default().profiling(true),
                    ))
                    .map_err(|error| format!("screenshot GPU: {error}"))?,
                );
            }
            let gpu = renderer.as_mut().expect("renderer initialized");
            let began = Instant::now();
            {
                let _prepare = trace.span("render.prepare_text");
                let (layout, text) = owned.scene()?;
                let prepared = text.prepare(&layout.text, viewport.scale);
                drop(_prepare);
                let _submit = trace.span("render.submit_cpu");
                gpu.render_ui(text, &prepared, &layout.display_list, viewport.scale)
                    .map_err(|error| format!("screenshot render: {error}"))?;
            }
            owned.record_render(began.elapsed());
            let _readback = trace.span("render.readback_wait");
            let pixels = gpu
                .read_offscreen_rgba()
                .map_err(|error| format!("screenshot readback: {error}"))?;
            drop(_readback);
            if !pixels
                .as_chunks::<4>()
                .0
                .iter()
                .any(|pixel| pixel != &pixels[..4])
            {
                return Err("screenshot was blank; check the mounted app and GPU renderer".into());
            }
            let _write = trace.span("artifact.png_write");
            image::save_buffer_with_format(
                &path,
                &pixels,
                width,
                height,
                ColorType::Rgba8,
                ImageFormat::Png,
            )
            .map_err(|error| format!("{}: {error}", path.display()))?;
            let profile = gpu.last_profile();
            owned.record_render_profile(&profile);
            *adapter = Some(json!({ "name": profile.adapter.name,
                "backend": profile.adapter.backend, "deviceType": profile.adapter.device_type,
                "timestampQueries": profile.adapter.timestamp_queries }));
        }
        other => return Err(format!("unknown automation action {other:?}")),
    }
    driver.borrow_mut().advance()?;
    deliver_pending(driver, gallery, &trace)?;
    Ok(String::new())
}

/// Advances Argui and JavaScript timers for `duration` without resolving the
/// current test action. `clock` supplies timer time and the overall deadline;
/// `trace` records the wait and nested timer work.
///
/// # Errors
/// Returns a timeout, callback, timer, or scene error.
fn drive_wait(
    driver: &Rc<RefCell<Driver>>,
    gallery: &QuickJsGallery,
    trace: &MetricTrace,
    clock: TestClock,
    duration: Duration,
) -> Result<(), String> {
    let _wait = trace.span("automation.wait");
    let end = Instant::now() + duration;
    while Instant::now() < end {
        if Instant::now() >= clock.deadline {
            return Err("test timed out while waiting; increase ARGUI_TEST_TIMEOUT_MS".into());
        }
        driver.borrow_mut().advance()?;
        deliver_pending(driver, gallery, trace)?;
        let elapsed = clock.started.elapsed().as_secs_f64() * 1000.0;
        let tick = trace.span("js.tick");
        gallery.tick(elapsed)?;
        drop(tick);
        let sleep = gallery
            .next_wake(elapsed)?
            .clamp(Duration::from_millis(1), Duration::from_millis(10))
            .min(end.saturating_duration_since(Instant::now()))
            .min(clock.deadline.saturating_duration_since(Instant::now()));
        if !sleep.is_zero() {
            std::thread::sleep(sleep);
        }
    }
    Ok(())
}

/// Sends queued native UI events into the app's existing callback bridge.
/// `driver` owns the queue, `gallery` is the same QuickJS session, and `trace`
/// relates callback work to its action or idle iteration.
///
/// # Errors
/// Returns the QuickJS callback error if delivery fails.
fn deliver_pending(
    driver: &Rc<RefCell<Driver>>,
    gallery: &QuickJsGallery,
    trace: &MetricTrace,
) -> Result<(), String> {
    let deliveries = driver.borrow_mut().take_deliveries();
    for delivery in deliveries {
        let _callback = trace.span("js.deliver_event");
        gallery.deliver(&event_json(&delivery).to_string())?;
    }
    Ok(())
}

/// Reads a finite numeric action parameter.
/// `args` is the action object and `key` identifies the required field.
///
/// # Errors
/// Returns a message for a missing or non-finite number.
fn number(args: &Value, key: &str) -> Result<f32, String> {
    let value = args[key]
        .as_f64()
        .ok_or_else(|| format!("missing numeric {key}"))? as f32;
    value
        .is_finite()
        .then_some(value)
        .ok_or_else(|| format!("{key} must be finite"))
}

/// Restricts screenshots to a single PNG filename beneath `out`.
///
/// # Errors
/// Returns an error for absolute, nested, hidden, or non-PNG names.
fn safe_capture_path(out: &Path, name: &str) -> Result<PathBuf, String> {
    if name.is_empty()
        || name.starts_with('.')
        || !name.ends_with(".png")
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        || name.contains("..")
    {
        return Err("screenshot name must be a simple .png filename inside --out".into());
    }
    Ok(out.join(name))
}

/// Computes an optional assertion against recorded Argui frame intervals.
/// `budget` contains only thresholds explicitly requested by the TS test.
///
/// # Errors
/// Returns a descriptive failure when a requested threshold is exceeded.
fn assert_performance(driver: &Driver, budget: &Value) -> Result<(), String> {
    let frame_times = driver
        .frame_records()
        .iter()
        .map(|frame| frame.total_cpu().as_secs_f64() * 1000.0)
        .collect::<Vec<_>>();
    if frame_times.is_empty() {
        return Err("no completed frame records for performance assertion".into());
    }
    let summary = frame_summary(&frame_times, frame_budget_ms());
    for (key, measured) in [
        ("p95FrameTimeMsBelow", summary["p95Ms"].as_f64()),
        ("maxFrameTimeMsBelow", summary["maxMs"].as_f64()),
    ] {
        if let Some(limit) = budget[key].as_f64()
            && measured.is_some_and(|value| value >= limit)
        {
            return Err(format!(
                "{key} requested < {limit:.2} ms; measured {:.2} ms",
                measured.unwrap_or_default()
            ));
        }
    }
    if let Some(limit) = budget["framesOverBudgetAtMost"].as_u64()
        && summary["overBudget"]
            .as_u64()
            .is_some_and(|value| value > limit)
    {
        return Err(format!(
            "frames over {:.2} ms exceeded {limit}",
            frame_budget_ms()
        ));
    }
    Ok(())
}

/// Produces frame quantiles and budget count from engine intervals.
/// `intervals` are milliseconds and `budget_ms` is the configured threshold.
fn frame_summary(intervals: &[f64], budget_ms: f64) -> Value {
    let mut sorted = intervals.to_vec();
    sorted.sort_by(f64::total_cmp);
    let percentile = |p: f64| {
        sorted
            .get(((sorted.len() as f64 * p).ceil() as usize).saturating_sub(1))
            .copied()
    };
    json!({ "count": sorted.len(), "budgetMs": budget_ms,
        "p50Ms": percentile(0.50), "p95Ms": percentile(0.95),
        "p99Ms": percentile(0.99), "maxMs": sorted.last(),
        "overBudget": sorted.iter().filter(|&&value| value > budget_ms).count() })
}

/// Reads a positive finite frame budget in milliseconds for reports and assertions.
fn frame_budget_ms() -> f64 {
    std::env::var("ARGUI_FRAME_BUDGET_MS")
        .ok()
        .and_then(|text| text.parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value > 0.0)
        .unwrap_or(16.67)
}

/// Builds a report that preserves missing adapter and frame metrics as null.
/// `driver` supplies raw engine records; `steps`, `adapter`, and `result`
/// describe actions, GPU identity, and final status.
fn report(
    driver: &Driver,
    steps: &[Step],
    adapter: Option<Value>,
    result: &Result<(), String>,
    started_unix_ms: f64,
) -> Value {
    let budget = frame_budget_ms();
    let actions = steps
        .iter()
        .map(|step| ActionWindow {
            started_ms: step.started_ms,
            duration_ms: step.duration_ms,
        })
        .collect::<Vec<_>>();
    let diagnostics = frame_diagnostics(driver.frames(), driver.frame_records(), &actions);
    let mut slow = diagnostics.clone();
    slow.sort_by(|left, right| right.total_cpu_ms.total_cmp(&left.total_cpu_ms));
    slow.truncate(10);
    let frame_times = driver
        .frame_records()
        .iter()
        .map(|frame| frame.total_cpu().as_secs_f64() * 1000.0)
        .collect::<Vec<_>>();
    let intervals = driver
        .frame_records()
        .iter()
        .map(|frame| frame.interval.as_secs_f64() * 1000.0)
        .filter(|value| *value > 0.0)
        .collect::<Vec<_>>();
    json!({ "ok": result.is_ok(), "error": result.as_ref().err(),
        "startedUnixMs": started_unix_ms,
        "viewport": driver.viewport(), "steps": steps,
        "artifacts": steps.iter().filter_map(|step| step.artifact.as_deref()).collect::<Vec<_>>(),
        "adapter": adapter, "frames": driver.frames(),
        "metrics": summarize_metrics(&driver.metrics()),
        "frameDiagnostics": diagnostics,
        "slowFrames": slow,
        "frameMetric": "Argui headless CPU frame time; intervals describe time between committed updates",
        "frameSummary": frame_summary(&frame_times, budget),
        "frameIntervals": frame_summary(&intervals, budget),
        "limitations": ["WebView and OS-owned surfaces are not captured", "GPU adapter required only for screenshots"] })
}
