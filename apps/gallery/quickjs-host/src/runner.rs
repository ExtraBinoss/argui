//! Native window and embedded QuickJS process for the shared gallery module.

#[cfg(any(debug_assertions, feature = "dev-metrics"))]
use std::sync::Mutex;
use std::{
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64},
        mpsc::{self, Receiver, Sender},
    },
};

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
use crate::desktop_application::{
    GalleryApp, gallery_config, register_application_services, runtime_service_event,
};
use crate::{
    QuickJsGallery,
    effects::registry_from_json,
    hot_reload::{BundleWatcher, Dispatch, dev_bundle_path},
    native_metrics::control_request,
    services::{ServiceChannels, ServiceRegistry, ServiceResponse},
    telemetry::JsCounts,
};
#[cfg(any(debug_assertions, feature = "dev-metrics"))]
use crate::{
    native_metrics::forward_profile,
    telemetry::{ProfileSummary, mark_commit_for_presentation, observe_profile, report_remaining},
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
use serde_json::Value;

mod js_loop;
use js_loop::{JsLoopInbox, ReloadControl, run_js_loop};

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
    #[cfg(feature = "automation")]
    if std::env::var_os("ARGUI_AUTOMATION_TEST").is_some() {
        return crate::automation::run();
    }
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
    #[cfg(any(debug_assertions, feature = "dev-metrics"))]
    let profiles = Arc::new(Mutex::new(ProfileSummary::default()));
    #[cfg(any(debug_assertions, feature = "dev-metrics"))]
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
            #[cfg(any(debug_assertions, feature = "dev-metrics"))]
            let frames = AtomicU64::new(0);
            run_native_host_application(
                config,
                RendererConfig::default()
                    .profiling(cfg!(debug_assertions) || cfg!(feature = "dev-metrics"))
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
                    #[cfg(any(debug_assertions, feature = "dev-metrics"))]
                    {
                        forward_profile(&event, &profile_sender, &profile_enabled, &frames);
                        observe_profile(&observed, &event, "quickjs");
                    }
                    #[cfg(not(any(debug_assertions, feature = "dev-metrics")))]
                    let _ = (&profile_sender, &profile_enabled);
                    if let Some(response) = runtime_service_event(&event) {
                        let _ = service_sender.send(response);
                    }
                },
            )
        },
    );
    #[cfg(any(debug_assertions, feature = "dev-metrics"))]
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
    #[cfg(any(debug_assertions, feature = "dev-metrics"))]
    let profiles = Arc::new(Mutex::new(ProfileSummary::default()));
    #[cfg(any(debug_assertions, feature = "dev-metrics"))]
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
            #[cfg(any(debug_assertions, feature = "dev-metrics"))]
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
                    .profiling(cfg!(debug_assertions) || cfg!(feature = "dev-metrics"))
                    .blur_algorithm(gallery_blur_algorithm())
                    .wait_for_submitted_gpu_work(wait_for_submitted_gpu_work)
                    .effects(effects),
                text_engine,
                host,
                assets,
                batches,
                deliveries,
                move |event| {
                    #[cfg(any(debug_assertions, feature = "dev-metrics"))]
                    {
                        forward_profile(&event, &profile_sender, &profile_enabled, &frames);
                        observe_profile(&observed, &event, "quickjs");
                    }
                    #[cfg(not(any(debug_assertions, feature = "dev-metrics")))]
                    let _ = (&profile_sender, &profile_enabled, event);
                },
            )
        },
    );
    #[cfg(any(debug_assertions, feature = "dev-metrics"))]
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
    let (profile_sender, _profile_events) = mpsc::channel();
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
        #[cfg(any(debug_assertions, feature = "dev-metrics"))]
        {
            let (startup_batches, startup_operations) = counts.startup();
            eprintln!(
                "argui-gallery-profile mode=quickjs startup_batches={startup_batches} startup_operations={startup_operations}"
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
                #[cfg(any(debug_assertions, feature = "dev-metrics"))]
                profiles: _profile_events,
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
                    #[cfg(any(debug_assertions, feature = "dev-metrics"))]
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
