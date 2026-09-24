use super::*;

/// Selects the requested comparison presentation on desktop or in an Android feature build.
pub(super) fn comparison_mode() -> Option<&'static str> {
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
    let contract_json = include_str!("../../../../../packages/host/src/contract.generated.json");
    let contract: Value = serde_json::from_str(contract_json)?;
    if contract["abiHash"].as_str() != Some(&host.abi_hash().to_string()) {
        return Err("generated JavaScript contract is stale; run bun run generate:jsx".into());
    }
    let source = include_str!("../../../dist/gallery-core.mjs");
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
pub(super) fn run_snapshot_desktop() -> Result<(), Box<dyn std::error::Error>> {
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
pub(super) fn run_snapshot_android(
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
