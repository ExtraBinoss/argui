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

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
use crate::desktop_application::{
    GalleryApp, gallery_config, register_application_services, runtime_service_event,
};
use crate::{
    QuickJsGallery,
    delivery::{coalesce_virtual_windows, event_json},
    effects::registry_from_json,
    hot_reload::{BundleWatcher, Dispatch, ReloadContext, dev_bundle_path, reload_gallery},
    native_metrics::{control_request, forward_profile},
    services::{ServiceChannels, ServiceRegistry, ServiceResponse},
    telemetry::{
        JsCounts, ProfileSummary, mark_commit_for_presentation, observe_profile, report_remaining,
    },
};
use argui_host::Host;
#[cfg(target_os = "android")]
use argui_platform::WindowConfig;
use argui_platform::WindowKey;
use argui_render::{BlurAlgorithm, EffectRegistry, RendererConfig};
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
use argui_runtime::{NativeHostApplicationChannels, run_native_host_application};
use argui_runtime::{
    NativeHostAssets, NativeHostBatch, NativeHostControl, NativeHostDelivery, RuntimeError,
    WireOperation,
};
use argui_ui::UiEventKind;
use serde_json::Value;

#[cfg(target_os = "android")]
const NOTO_SANS: &[u8] = include_bytes!("../../../../assets/fonts/NotoSans-Regular.ttf");

/// Returns the gallery blur mode selected by `ARGUI_GALLERY_BLUR`.
///
/// The default and unrecognized values select automatic mode. This setting
/// lets native gallery runs compare fixed Gaussian and dual-filter rendering.
fn gallery_blur_algorithm() -> BlurAlgorithm {
    match std::env::var("ARGUI_GALLERY_BLUR").as_deref() {
        Ok("gaussian") => BlurAlgorithm::Gaussian,
        Ok("dual") => BlurAlgorithm::DualKawase,
        _ => BlurAlgorithm::Auto,
    }
}

/// Runs the embedded Solid gallery in a desktop native window.
///
/// # Errors
/// Returns an error if the schema, gallery bundle, QuickJS engine, or native window cannot start.
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
pub fn run_desktop() -> Result<(), Box<dyn std::error::Error>> {
    run_desktop_with_services(Arc::new(ServiceRegistry::with_builtins()))
}

/// Runs the desktop gallery with application-registered native services.
/// `services` contains the operations available to its TSX components.
/// When `ARGUI_VALIDATE_ONLY` is set, validates `ARGUI_APP_BUNDLE` without
/// creating a window and does not use `services`.
///
/// # Errors
/// Returns an error if the gallery, native window, or JavaScript engine fails.
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
pub fn run_desktop_with_services(
    services: Arc<ServiceRegistry>,
) -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var_os("ARGUI_VALIDATE_ONLY").is_some() {
        return crate::validate::validate_app_bundle();
    }
    let mut config = gallery_config()?;
    if let Ok(title) = std::env::var("ARGUI_APP_TITLE") {
        config.windows[0].window.title = title;
    }
    let (application_sender, application_requests) = mpsc::channel();
    let appearance =
        register_application_services(&services, application_sender, config.tray.clone());
    let profiles = Arc::new(Mutex::new(ProfileSummary::default()));
    let observed = Arc::clone(&profiles);
    let bundle_path = std::env::var_os("ARGUI_APP_BUNDLE")
        .map(PathBuf::from)
        .or_else(dev_bundle_path);
    let wait_for_submitted_gpu_work = bundle_path.is_some();
    let result = run_gallery(
        bundle_path,
        services,
        |host,
         assets,
         batches,
         deliveries,
         effects,
         profile_sender,
         profile_enabled,
         service_sender| {
            let frames = AtomicU64::new(0);
            run_native_host_application(
                config,
                RendererConfig::default()
                    .profiling(true)
                    .blur_algorithm(gallery_blur_algorithm())
                    .wait_for_submitted_gpu_work(wait_for_submitted_gpu_work)
                    .effects(effects),
                host,
                assets,
                NativeHostApplicationChannels {
                    batches,
                    events: deliveries,
                    requests: application_requests,
                },
                GalleryApp::with_appearance(service_sender.clone(), appearance),
                move |event| {
                    forward_profile(&event, &profile_sender, &profile_enabled, &frames);
                    observe_profile(&observed, &event, "quickjs");
                    if let Some(response) = runtime_service_event(&event) {
                        let _ = service_sender.send(response);
                    }
                },
            )
        },
    );
    report_remaining(&profiles, "quickjs");
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
    let profiles = Arc::new(Mutex::new(ProfileSummary::default()));
    let observed = Arc::clone(&profiles);
    let bundle_path = cfg!(debug_assertions)
        .then(|| {
            android_app
                .internal_data_path()
                .map(|path| path.join("gallery-core.mjs"))
        })
        .flatten();
    let wait_for_submitted_gpu_work = bundle_path.is_some();
    let result = run_gallery(
        bundle_path,
        Arc::new(ServiceRegistry::with_builtins()),
        |host,
         assets,
         batches,
         deliveries,
         effects,
         profile_sender,
         profile_enabled,
         _service_sender| {
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
                RendererConfig::default()
                    .profiling(true)
                    .blur_algorithm(gallery_blur_algorithm())
                    .wait_for_submitted_gpu_work(wait_for_submitted_gpu_work)
                    .effects(effects),
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
    report_remaining(&profiles, "quickjs");
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
    services: Arc<ServiceRegistry>,
    launch: impl FnOnce(
        Host,
        NativeHostAssets,
        Receiver<NativeHostBatch>,
        Sender<NativeHostDelivery>,
        EffectRegistry,
        Sender<String>,
        Arc<AtomicBool>,
        Sender<ServiceResponse>,
    ) -> Result<(), RuntimeError>,
) -> Result<(), Box<dyn std::error::Error>> {
    let host = Host::with_builtins()?;
    let contract_json = include_str!("../../../../packages/host/src/contract.generated.json");
    let contract: Value = serde_json::from_str(contract_json)?;
    if contract["abiHash"].as_str() != Some(&host.abi_hash().to_string()) {
        return Err("generated JavaScript contract is stale; run bun run generate:jsx".into());
    }
    let source = match std::env::var_os("ARGUI_APP_BUNDLE") {
        Some(path) => std::fs::read_to_string(&path)
            .map_err(|error| format!("{}: {error}", PathBuf::from(path).display()))?,
        None => include_str!("../../dist/gallery-core.mjs").to_owned(),
    };

    let (wire_sender, wire_receiver) = mpsc::channel();
    let (batch_sender, batches) = mpsc::channel();
    let (errors_sender, errors) = mpsc::channel();
    std::thread::spawn(move || relay_batches(wire_receiver, batch_sender, errors_sender));

    let (deliveries, events) = mpsc::channel();
    let (service_sender, service_responses) = mpsc::channel();
    let runtime_service_sender = service_sender.clone();
    let actor_services = Arc::clone(&services);
    let (profile_sender, profile_events) = mpsc::channel();
    let profile_enabled = Arc::new(AtomicBool::new(false));
    let actor_profile_enabled = Arc::clone(&profile_enabled);
    let (stop_sender, stop) = mpsc::channel();
    let (ready_sender, ready) = mpsc::channel();
    std::thread::spawn(move || {
        let batch_count = Arc::new(AtomicU64::new(0));
        let operation_count = Arc::new(AtomicU64::new(0));
        let mut dispatch = Dispatch::new(1, Arc::clone(&batch_count), Arc::clone(&operation_count));
        let route = Rc::clone(&dispatch);
        let control_sender = wire_sender.clone();
        let control_enabled = Arc::clone(&actor_profile_enabled);
        let request_services = Arc::clone(&actor_services);
        let cancel_services = Arc::clone(&actor_services);
        let request_sender = service_sender.clone();
        let gallery = QuickJsGallery::new_with_services(
            &source,
            contract_json,
            "mountGallery",
            move |json| route.borrow_mut().accept(&json),
            move |json| control_request(&json, &control_sender, &control_enabled),
            move |json| {
                request_services
                    .submit(1, &json, request_sender.clone())
                    .err()
                    .unwrap_or_default()
            },
            move |json| {
                cancel_services
                    .cancel_json(1, &json)
                    .err()
                    .unwrap_or_default()
            },
        );
        let mut gallery = match gallery {
            Ok(gallery) => gallery,
            Err(error) => {
                let _ = ready_sender.send(Err(error));
                return;
            }
        };
        if wire_sender
            .send(RelayBatch {
                operations: Vec::new(),
                controls: vec![NativeHostControl::SetRendererProfiling(false)],
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
        eprintln!(
            "argui-gallery-profile mode=quickjs startup_batches={startup_batches} startup_operations={startup_operations}"
        );
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
                services: service_responses,
            },
            &actor_profile_enabled,
            &counts,
            ServiceChannels {
                registry: &actor_services,
                sender: &service_sender,
            },
        ) {
            eprintln!("{error}");
        }
        if let Err(error) = gallery.dispose() {
            eprintln!("{error}");
        }
        actor_services.cancel_session(dispatch.borrow().generation);
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
        runtime_service_sender,
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
            Ok(Ok(_commit)) => {
                if let Some(ack) = message.acknowledgement {
                    let _ = ack.send(Ok(()));
                }
                if batch_sequence > 1 && operation_count >= 50 {
                    mark_commit_for_presentation();
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
    services: Receiver<ServiceResponse>,
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
    services: ServiceChannels<'_>,
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
                        ReloadContext {
                            sender: reload.sender,
                            counts,
                            profile_enabled,
                            services: ServiceChannels {
                                registry: services.registry,
                                sender: services.sender,
                            },
                        },
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
        deliver_services(gallery, &inbox.services, dispatch.borrow().generation)?;
        let entered = Instant::now();
        let maximum_delay = if profile_enabled.load(Ordering::Relaxed)
            || services.registry.has_pending(dispatch.borrow().generation)
        {
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
            if profile_enabled.load(Ordering::Relaxed)
                && matches!(delivery.kind, UiEventKind::Click(_))
            {
                eprintln!(
                    "argui-gallery-profile click_js_callback_ms={:.3}",
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
        deliver_services(gallery, &inbox.services, dispatch.borrow().generation)?;
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
        if profile_enabled.load(Ordering::Relaxed) && ticks.is_multiple_of(5) {
            counts.report(deliveries, ticks, work, started.elapsed());
            eprintln!("argui-gallery-profile virtual_window_deliveries={window_deliveries}");
        }
    }
    if profile_enabled.load(Ordering::Relaxed) {
        counts.report(deliveries, ticks, work, started.elapsed());
    }
    Ok(())
}

/// Delivers completed requests only to their still-live JavaScript generation.
/// `gallery` receives JSON, `responses` is the native worker channel, and
/// `session` identifies the current mounted application.
///
/// # Errors
/// Returns an error if a response callback fails in QuickJS.
fn deliver_services(
    gallery: &QuickJsGallery,
    responses: &Receiver<ServiceResponse>,
    session: u32,
) -> Result<(), String> {
    for response in responses.try_iter() {
        if response.session == session || response.session == u32::MAX {
            gallery.deliver_service(&response.json().to_string())?;
        }
    }
    Ok(())
}
