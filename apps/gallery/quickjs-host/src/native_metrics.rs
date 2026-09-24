//! Native renderer controls and real frame profiles for JavaScript examples.

use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc::Sender,
};

use argui_render::{DamageMode, DamageTracking, RenderProfile};
use argui_runtime::{NativeHostControl, RuntimeEvent};
use serde_json::{Value, json};

use crate::runner::RelayBatch;

/// Parses one JavaScript renderer control request.
///
/// `json` must contain a damage-tracking request with a boolean `enabled`
/// field. Returns the native control, or an error for unsupported input.
///
/// # Errors
/// Returns an error for malformed JSON, unknown controls, or missing fields.
pub fn parse_control(json: &str) -> Result<NativeHostControl, String> {
    let request: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
    if request.get("kind").and_then(Value::as_str) == Some("registerSvg") {
        if !cfg!(all(feature = "dev-tabler-icons", debug_assertions)) {
            return Err("development SVG catalogue is disabled".into());
        }
        let id = request
            .get("id")
            .and_then(Value::as_u64)
            .filter(|id| *id > 0)
            .ok_or("registerSvg.id must be a positive integer")?;
        let svg = request
            .get("svg")
            .and_then(Value::as_str)
            .filter(|svg| svg.len() <= 65536)
            .ok_or("registerSvg.svg must be a string under 64 KiB")?;
        let assets = argui_gallery_assets::decode_assets(&[argui_gallery_assets::AssetInput {
            key: "development SVG",
            kind: argui_gallery_assets::AssetKind::Svg,
            id,
            bytes: svg.as_bytes(),
        }])?;
        let vector = assets
            .vectors
            .into_iter()
            .next()
            .ok_or("decoded SVG missing")?;
        return Ok(NativeHostControl::RegisterVector(vector));
    }
    if request.get("kind").and_then(Value::as_str) != Some("damageTracking") {
        return Err("unknown renderer control".into());
    }
    let enabled = request
        .get("enabled")
        .and_then(Value::as_bool)
        .ok_or("damageTracking.enabled must be a boolean")?;
    Ok(NativeHostControl::SetDamageTracking(if enabled {
        DamageTracking::enabled()
    } else {
        DamageTracking::disabled()
    }))
}

/// Encodes a completed native renderer `profile` for the JavaScript dashboard.
///
/// Returns measured CPU and optional GPU milliseconds together with the
/// damage decision and pixel counts. Missing GPU timestamps remain JSON null.
pub fn profile_json(profile: &RenderProfile) -> String {
    json!({
        "cpuMs": profile.cpu_time.as_secs_f64() * 1000.0,
        "gpuMs": profile.gpu.as_ref().map(|gpu| gpu.total.as_secs_f64() * 1000.0),
        "damageMode": match profile.damage.mode {
            DamageMode::Full => "full",
            DamageMode::Seed => "seed",
            DamageMode::Partial => "partial",
            DamageMode::Reused => "reused",
        },
        "regions": profile.damage.regions,
        "damagedPixels": profile.damage.damaged_pixels,
        "viewportPixels": profile.viewport_pixels,
        "retainedBytes": profile.damage.retained_bytes,
    })
    .to_string()
}

/// Sends every eighth measured renderer frame while the dashboard is active.
///
/// `event` is a runtime event; `sender` receives serialized real samples;
/// `enabled` is the page subscription flag and `frames` counts eligible frames.
/// No sample is synthesized when the renderer has not emitted a profile.
pub fn forward_profile(
    event: &RuntimeEvent,
    sender: &Sender<String>,
    enabled: &AtomicBool,
    frames: &AtomicU64,
) {
    if !enabled.load(Ordering::Relaxed) {
        return;
    }
    if let RuntimeEvent::RenderProfile(profile) = event
        && frames.fetch_add(1, Ordering::Relaxed).is_multiple_of(8)
    {
        let _ = sender.send(profile_json(profile));
    }
}

/// Routes one JavaScript profile subscription or damage command.
///
/// `json` identifies the request; `sender` forwards damage changes to the
/// UI thread, and `enabled` gates renderer samples back to JavaScript.
/// Returns an empty string on success or a message for the bridge to throw.
pub(crate) fn control_request(
    json: &str,
    sender: &Sender<RelayBatch>,
    enabled: &AtomicBool,
) -> String {
    if let Ok(value) = serde_json::from_str::<Value>(json)
        && value.get("kind").and_then(Value::as_str) == Some("profile")
    {
        let Some(requested) = value.get("enabled").and_then(Value::as_bool) else {
            return "profile.enabled must be a boolean".into();
        };
        let sent = sender.send(RelayBatch {
            operations: Vec::new(),
            controls: vec![NativeHostControl::SetRendererProfiling(requested)],
            acknowledgement: None,
        });
        if sent.is_err() {
            return "native UI thread closed".into();
        }
        enabled.store(requested, Ordering::Relaxed);
        return String::new();
    }
    let control = match parse_control(json) {
        Ok(control) => control,
        Err(error) => return error,
    };
    sender
        .send(RelayBatch {
            operations: Vec::new(),
            controls: vec![control],
            acknowledgement: None,
        })
        .map_or_else(|_| "native UI thread closed".into(), |()| String::new())
}
