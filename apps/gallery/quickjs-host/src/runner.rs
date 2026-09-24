//! Native window and embedded QuickJS process for the shared gallery module.

use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    time::{Duration, Instant},
};

use crate::{
    AnimationSnapshot, QuickJsGallery,
    comparison::navigation_callback,
    delivery::{coalesce_virtual_windows, event_json},
    effects::registry_from_json,
    hot_reload::{BundleWatcher, Dispatch, dev_bundle_path, reload_gallery},
    native_metrics::{control_request, forward_profile},
    telemetry::{
        JsCounts, ProfileSummary, mark_commit_for_presentation, observe_profile, report_remaining,
    },
};
use argui_host::Host;
use argui_platform::{
    ApplicationConfig, ApplicationId, ApplicationIdentity, IconSet, WindowConfig, WindowKey,
};
use argui_render::{EffectRegistry, RendererConfig};
use argui_runtime::{
    NativeHostAssets, NativeHostBatch, NativeHostControl, NativeHostDelivery, RuntimeError,
    WireOperation, run_application, run_native_host,
};
use argui_ui::UiEventKind;
use serde_json::Value;

#[cfg(target_os = "android")]
const NOTO_SANS: &[u8] =
    include_bytes!("../../../../crates/argui-web-demo/assets/fonts/NotoSans-Regular.ttf");

/// Runs the embedded Solid gallery in a desktop native window.
///
/// # Errors
/// Returns an error if the schema, gallery bundle, QuickJS engine, or native window cannot start.
pub fn run_desktop() -> Result<(), Box<dyn std::error::Error>> {
    if comparison_mode() == Some("rust") {
        return run_snapshot_desktop();
    }
    let profiles = Arc::new(Mutex::new(ProfileSummary::default()));
    let observed = Arc::clone(&profiles);
    let result = run_gallery(
        dev_bundle_path(),
        |host, assets, batches, deliveries, effects, profile_sender, profile_enabled| {
            let frames = AtomicU64::new(0);
            run_native_host(
                WindowConfig {
                    title: "Argui Gallery / QuickJS".into(),
                    ..WindowConfig::default()
                },
                RendererConfig::default().profiling(true).effects(effects),
                host,
                assets,
                batches,
                deliveries,
                move |event| {
                    forward_profile(&event, &profile_sender, &profile_enabled, &frames);
                    observe_profile(&observed, &event, "quickjs");
                },
            )
        },
    );
    if comparison_mode().is_some() {
        report_remaining(&profiles, "quickjs");
    }
    result
}

/// Runs the embedded Solid gallery from Android's native activity.
///
/// `android_app` is the activity handle passed by Android. Returns after the
/// activity's event loop exits.
///
/// # Errors
/// Returns an error if the schema, gallery bundle, QuickJS engine, or native window cannot start.
#[cfg(target_os = "android")]
pub fn run_android(
    android_app: argui_android::AndroidApp,
) -> Result<(), Box<dyn std::error::Error>> {
    if comparison_mode() == Some("rust") {
        return run_snapshot_android(android_app);
    }
    let profiles = Arc::new(Mutex::new(ProfileSummary::default()));
    let observed = Arc::clone(&profiles);
    let bundle_path = cfg!(debug_assertions)
        .then(|| {
            android_app
                .internal_data_path()
                .map(|path| path.join("gallery-core.mjs"))
        })
        .flatten();
    let result = run_gallery(
        bundle_path,
        |host, assets, batches, deliveries, effects, profile_sender, profile_enabled| {
            let frames = AtomicU64::new(0);
            let text_engine = argui_text::TextEngine::from_embedded_fonts(
                [NOTO_SANS],
                "Noto Sans",
                "Noto Sans",
                "Noto Sans",
            );
            argui_runtime::run_android_native_host_with_text_engine(
                android_app,
                WindowConfig {
                    title: "Argui Gallery / QuickJS".into(),
                    ..WindowConfig::default()
                },
                RendererConfig::default().profiling(true).effects(effects),
                text_engine,
                host,
                assets,
                batches,
                deliveries,
                move |event| {
                    forward_profile(&event, &profile_sender, &profile_enabled, &frames);
                    observe_profile(&observed, &event, "quickjs");
                },
            )
        },
    );
    if comparison_mode().is_some() {
        report_remaining(&profiles, "quickjs");
    }
    result
}

/// Starts the JavaScript actor and passes its host channels to `launch`.
///
/// `bundle_path` optionally points to a development bundle watched at runtime.
/// `launch` owns the native UI event loop and returns its runtime result.
/// Returns after the event loop exits and signals the JavaScript actor to stop.
///
/// # Errors
/// Returns an error if the schema, embedded bundle, QuickJS engine, or native window cannot start.
fn run_gallery(
    bundle_path: Option<PathBuf>,
    launch: impl FnOnce(
        Host,
        NativeHostAssets,
        Receiver<NativeHostBatch>,
        Sender<NativeHostDelivery>,
        EffectRegistry,
        Sender<String>,
        Arc<AtomicBool>,
    ) -> Result<(), RuntimeError>,
) -> Result<(), Box<dyn std::error::Error>> {
    let host = Host::with_builtins()?;
    let contract_json = include_str!("../../../../packages/host/src/contract.generated.json");
    let contract: Value = serde_json::from_str(contract_json)?;
    if contract["abiHash"].as_str() != Some(&host.abi_hash().to_string()) {
        return Err("generated JavaScript contract is stale; run bun run generate:jsx".into());
    }
    let source = include_str!("../../dist/gallery-core.mjs");

    let (wire_sender, wire_receiver) = mpsc::channel();
    let (batch_sender, batches) = mpsc::channel();
    let (errors_sender, errors) = mpsc::channel();
    std::thread::spawn(move || relay_batches(wire_receiver, batch_sender, errors_sender));

    let (deliveries, events) = mpsc::channel();
    let (profile_sender, profile_events) = mpsc::channel();
    let profile_enabled = Arc::new(AtomicBool::new(false));
    let actor_profile_enabled = Arc::clone(&profile_enabled);
    let (stop_sender, stop) = mpsc::channel();
    let (ready_sender, ready) = mpsc::channel();
    std::thread::spawn(move || {
        let initial = Rc::new(RefCell::new(Some(Vec::<String>::new())));
        let captured = Rc::clone(&initial);
        let batch_count = Arc::new(AtomicU64::new(0));
        let operation_count = Arc::new(AtomicU64::new(0));
        let mut dispatch = Dispatch::new(1, Arc::clone(&batch_count), Arc::clone(&operation_count));
        let route = Rc::clone(&dispatch);
        let control_sender = wire_sender.clone();
        let control_enabled = Arc::clone(&actor_profile_enabled);
        let gallery = QuickJsGallery::new_with_control(
            source,
            contract_json,
            "mountGallery",
            move |json| {
                if let Some(initial) = captured.borrow_mut().as_mut() {
                    initial.push(json.clone());
                }
                route.borrow_mut().accept(&json)
            },
            move |json| control_request(&json, &control_sender, &control_enabled),
        );
        let mut gallery = match gallery {
            Ok(gallery) => gallery,
            Err(error) => {
                let _ = ready_sender.send(Err(error));
                return;
            }
        };
        if comparison_mode().is_some() {
            let navigation = initial.borrow().as_ref().cloned().unwrap_or_default();
            if let Err(error) = navigation_callback(&navigation)
                .and_then(|callback| gallery.deliver(&callback.to_string()))
            {
                let _ = ready_sender.send(Err(error));
                return;
            }
        }
        *initial.borrow_mut() = None;
        if wire_sender
            .send(RelayBatch {
                operations: Vec::new(),
                controls: vec![NativeHostControl::SetRendererProfiling(
                    comparison_mode().is_some(),
                )],
                acknowledgement: None,
            })
            .is_err()
        {
            let _ = ready_sender.send(Err("native UI thread closed".into()));
            return;
        }
        let first = std::mem::take(&mut dispatch.borrow_mut().pending);
        for operations in first {
            if wire_sender
                .send(RelayBatch {
                    operations,
                    controls: Vec::new(),
                    acknowledgement: None,
                })
                .is_err()
            {
                let _ = ready_sender.send(Err("native UI thread closed".into()));
                return;
            }
        }
        dispatch.borrow_mut().activate(wire_sender.clone());
        let mut reload = ReloadControl {
            watcher: bundle_path.map(BundleWatcher::new),
            contract_json,
            sender: &wire_sender,
        };
        let counts = JsCounts::new(batch_count, operation_count);
        let (startup_batches, startup_operations) = counts.startup();
        if comparison_mode().is_some() {
            eprintln!(
                "argui-comparison mode=quickjs startup_batches={startup_batches} startup_operations={startup_operations}"
            );
        }
        let effects = gallery
            .effect_definitions_json()
            .and_then(|json| registry_from_json(&json));
        let effects = match effects {
            Ok(effects) => effects,
            Err(error) => {
                let _ = ready_sender.send(Err(error));
                return;
            }
        };
        let _ = ready_sender.send(Ok(effects));
        if let Err(error) = run_js_loop(
            &mut gallery,
            &mut dispatch,
            &mut reload,
            JsLoopInbox {
                events,
                profiles: profile_events,
                errors,
                stop,
            },
            &actor_profile_enabled,
            &counts,
        ) {
            eprintln!("{error}");
        }
        if let Err(error) = gallery.dispose() {
            eprintln!("{error}");
        }
    });
    let effects = ready.recv().map_err(|error| error.to_string())??;

    let assets = argui_gallery_assets::load()?;
    let result = launch(
        host,
        assets,
        batches,
        deliveries,
        effects,
        profile_sender,
        profile_enabled,
    );
    let _ = stop_sender.send(());
    result?;
    Ok(())
}

/// A decoded batch with an optional synchronous response for reload swaps.
pub(crate) struct RelayBatch {
    pub(crate) operations: Vec<WireOperation>,
    pub(crate) controls: Vec<NativeHostControl>,
    pub(crate) acknowledgement: Option<Sender<Result<(), String>>>,
}

/// Sends each decoded JS transaction to the UI thread and reports rejected commits.
fn relay_batches(
    wire: Receiver<RelayBatch>,
    batches: Sender<NativeHostBatch>,
    errors: Sender<String>,
) {
    let mut batch_sequence = 0_u64;
    for message in wire {
        batch_sequence += 1;
        let operation_count = message.operations.len();
        let submitted = Instant::now();
        let (reply, completion) = mpsc::channel();
        if batches
            .send(NativeHostBatch {
                window: WindowKey::main(),
                operations: message.operations,
                controls: message.controls,
                reply,
            })
            .is_err()
        {
            break;
        }
        match completion.recv() {
            Ok(Ok(commit)) => {
                if let Some(ack) = message.acknowledgement {
                    let _ = ack.send(Ok(()));
                }
                if comparison_mode().is_some() {
                    if batch_sequence > 1 && operation_count >= 50 {
                        mark_commit_for_presentation();
                    }
                    eprintln!(
                        "argui-comparison host_batch_operations={operation_count} host_changed_nodes={} host_update={:?} queue_and_commit_ms={:.3}",
                        commit.changed_nodes,
                        commit.update,
                        submitted.elapsed().as_secs_f64() * 1000.0
                    );
                }
            }
            Ok(Err(error)) => {
                if let Some(ack) = message.acknowledgement {
                    let _ = ack.send(Err(error));
                    continue;
                }
                let _ = errors.send(error);
                break;
            }
            Err(_) => break,
        }
    }
}

/// Development bundle state owned by the QuickJS actor.
struct ReloadControl<'a> {
    watcher: Option<BundleWatcher>,
    contract_json: &'a str,
    sender: &'a Sender<RelayBatch>,
}

/// Native channels consumed by the QuickJS event pump.
struct JsLoopInbox {
    events: Receiver<NativeHostDelivery>,
    profiles: Receiver<String>,
    errors: Receiver<String>,
    stop: Receiver<()>,
}

/// Pumps native events, timer callbacks, and QuickJS microtasks until shutdown.
/// `gallery` is the active script; `dispatch` routes commits; `reload` watches
/// candidate bundles; `inbox` carries native events and shutdown; `profile_enabled`
/// gates profiling samples, and `counts` records JavaScript work. Returns after
/// shutdown or on the first unrecoverable JavaScript/native commit error.
///
/// # Errors
/// Returns an error when JavaScript fails or the native host rejects a batch.
fn run_js_loop(
    gallery: &mut QuickJsGallery,
    dispatch: &mut Rc<RefCell<Dispatch>>,
    reload: &mut ReloadControl<'_>,
    inbox: JsLoopInbox,
    profile_enabled: &Arc<AtomicBool>,
    counts: &JsCounts,
) -> Result<(), String> {
    let started = Instant::now();
    let mut work = Duration::ZERO;
    let mut ticks = 0_u64;
    let mut deliveries = 0_u64;
    let mut window_deliveries = 0_u64;
    loop {
        if inbox.stop.try_recv().is_ok() {
            break;
        }
        if let Ok(error) = inbox.errors.try_recv() {
            return Err(format!("native commit rejected: {error}"));
        }
        if let Some(watcher) = reload.watcher.as_mut() {
            match watcher.changed() {
                Ok(Some(source)) => {
                    if let Err(error) = reload_gallery(
                        gallery,
                        dispatch,
                        &source,
                        reload.contract_json,
                        reload.sender,
                        counts,
                        profile_enabled,
                    ) {
                        eprintln!("argui-hot-reload: {error}; keeping previous scene");
                    } else {
                        eprintln!("argui-hot-reload: bundle applied");
                    }
                }
                Ok(None) => {}
                Err(error) => eprintln!("argui-hot-reload: {error}"),
            }
        }
        let entered = Instant::now();
        let maximum_delay = if profile_enabled.load(Ordering::Relaxed) {
            50
        } else if reload.watcher.is_some() {
            100
        } else {
            1000
        };
        let delay = gallery
            .next_wake(started.elapsed().as_secs_f64() * 1000.0)?
            .min(Duration::from_millis(maximum_delay));
        work += entered.elapsed();
        let burst = inbox
            .events
            .recv_timeout(delay)
            .ok()
            .into_iter()
            .chain(inbox.events.try_iter())
            .collect();
        for delivery in coalesce_virtual_windows(burst) {
            if delivery.callback.node.generation() != dispatch.borrow().generation {
                continue;
            }
            let entered = Instant::now();
            gallery.deliver(&event_json(&delivery).to_string())?;
            if comparison_mode().is_some() && matches!(delivery.kind, UiEventKind::Click(_)) {
                eprintln!(
                    "argui-comparison click_js_callback_ms={:.3}",
                    entered.elapsed().as_secs_f64() * 1000.0
                );
            }
            work += entered.elapsed();
            deliveries += 1;
            window_deliveries += u64::from(matches!(
                delivery.kind,
                UiEventKind::VirtualWindowChanged { .. }
            ));
        }
        let mut latest_profile = None;
        while let Ok(profile) = inbox.profiles.try_recv() {
            latest_profile = Some(profile);
        }
        if profile_enabled.load(Ordering::Relaxed)
            && let Some(profile) = latest_profile
        {
            gallery.deliver_profile(&profile)?;
        }
        let entered = Instant::now();
        gallery.tick(started.elapsed().as_secs_f64() * 1000.0)?;
        work += entered.elapsed();
        ticks += 1;
        if comparison_mode().is_some() && ticks.is_multiple_of(5) {
            counts.report(deliveries, ticks, work, started.elapsed());
            eprintln!("argui-comparison virtual_window_deliveries={window_deliveries}");
        }
    }
    if comparison_mode().is_some() {
        counts.report(deliveries, ticks, work, started.elapsed());
    }
    Ok(())
}

/// Selects the requested comparison presentation on desktop or in an Android feature build.
fn comparison_mode() -> Option<&'static str> {
    if cfg!(feature = "comparison-rust") {
        Some("rust")
    } else if cfg!(feature = "comparison-quickjs") {
        Some("quickjs")
    } else {
        match std::env::var("ARGUI_GALLERY_COMPARE").ok().as_deref() {
            Some("rust") => Some("rust"),
            Some("quickjs") => Some("quickjs"),
            _ => None,
        }
    }
}

/// Creates the exact Animation Lab snapshot from the same generated bundle as the live host.
/// Returns the frozen model and its decoded media.
///
/// # Errors
/// Returns an error when the bundle, schema, media, or snapshot fails validation.
fn animation_snapshot() -> Result<AnimationSnapshot, Box<dyn std::error::Error>> {
    let host = Host::with_builtins()?;
    let contract_json = include_str!("../../../../packages/host/src/contract.generated.json");
    let contract: Value = serde_json::from_str(contract_json)?;
    if contract["abiHash"].as_str() != Some(&host.abi_hash().to_string()) {
        return Err("generated JavaScript contract is stale; run bun run generate:jsx".into());
    }
    let source = include_str!("../../dist/gallery-core.mjs");
    let assets = argui_gallery_assets::load()?;
    Ok(AnimationSnapshot::build(source, contract_json, assets)?)
}

/// Builds the same window identity and viewport settings for both comparison presentations.
/// Returns a single-window application configuration.
///
/// # Errors
/// Returns an error if the application identifier is invalid.
fn snapshot_config() -> Result<ApplicationConfig, Box<dyn std::error::Error>> {
    Ok(ApplicationConfig::new(
        ApplicationIdentity::new(
            ApplicationId::new("dev.argui.solidgallery")?,
            "Argui Gallery Comparison",
            IconSet::new(),
        ),
        WindowConfig {
            title: "Argui Gallery / Rust snapshot".into(),
            ..WindowConfig::default()
        },
    ))
}

/// Runs the frozen Animation Lab tree as a direct desktop Rust model.
///
/// # Errors
/// Returns an error if snapshot materialization or the native window fails.
fn run_snapshot_desktop() -> Result<(), Box<dyn std::error::Error>> {
    let profiles = Arc::new(Mutex::new(ProfileSummary::default()));
    let observed = Arc::clone(&profiles);
    let snapshot = animation_snapshot()?;
    eprintln!(
        "argui-comparison mode=rust startup_batches={} startup_operations={}",
        snapshot.wire_batches().len(),
        snapshot.wire_batches().iter().map(Vec::len).sum::<usize>()
    );
    let result = run_application(
        snapshot_config()?,
        RendererConfig {
            profiling: true,
            ..RendererConfig::default()
        },
        snapshot,
        move |event| observe_profile(&observed, &event, "rust"),
    );
    report_remaining(&profiles, "rust");
    Ok(result?)
}

/// Runs the frozen Animation Lab tree as a direct Android Rust model.
/// `android_app` supplies the native Activity and its Pixel viewport.
///
/// # Errors
/// Returns an error if snapshot materialization or the native Activity fails.
#[cfg(target_os = "android")]
fn run_snapshot_android(
    android_app: argui_android::AndroidApp,
) -> Result<(), Box<dyn std::error::Error>> {
    let profiles = Arc::new(Mutex::new(ProfileSummary::default()));
    let observed = Arc::clone(&profiles);
    let text_engine = argui_text::TextEngine::from_embedded_fonts(
        [NOTO_SANS],
        "Noto Sans",
        "Noto Sans",
        "Noto Sans",
    );
    let snapshot = animation_snapshot()?;
    eprintln!(
        "argui-comparison mode=rust startup_batches={} startup_operations={}",
        snapshot.wire_batches().len(),
        snapshot.wire_batches().iter().map(Vec::len).sum::<usize>()
    );
    let result = argui_runtime::run_android_application_with_text_engine(
        android_app,
        snapshot_config()?,
        RendererConfig {
            profiling: true,
            ..RendererConfig::default()
        },
        text_engine,
        snapshot,
        move |event| observe_profile(&observed, &event, "rust"),
    );
    report_remaining(&profiles, "rust");
    Ok(result?)
}
