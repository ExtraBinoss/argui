//! Embedded QuickJS execution for the runtime-neutral gallery module.

mod comparison;
mod delivery;
mod effects;
mod hot_reload;
mod native_metrics;
mod runner;
mod telemetry;
mod wire;

pub use comparison::AnimationSnapshot;
pub use delivery::{coalesce_virtual_windows, ui_event_payload};
pub use effects::registry_from_json;
pub use native_metrics::{parse_control, profile_json};
pub use wire::decode_wire_operations;

pub use runner::run_desktop;

#[cfg(target_os = "android")]
argui_android::android_main!(runner::run_android);

use rquickjs::{CaughtError, Context, Function, Module, Object, Runtime};
use std::time::Duration;

/// A gallery session with its own QuickJS runtime and pending-job queue.
pub struct QuickJsGallery {
    runtime: Runtime,
    context: Context,
}

impl QuickJsGallery {
    /// Loads `source` as an ES module and invokes `entry` with the native bridge and `contract_json`.
    /// The `commit` callback forwards native operation batches and returns an error string.
    /// Returns the mounted gallery session.
    ///
    /// # Errors
    /// Returns an error when the script, contract, or initial mount fails.
    pub fn new(
        source: &str,
        contract_json: &str,
        entry: &str,
        commit: impl Fn(String) -> String + 'static,
    ) -> Result<Self, String> {
        Self::new_with_control(source, contract_json, entry, commit, |_| {
            "native controls unavailable".into()
        })
    }

    /// Loads a gallery module with a native control callback in addition to commits.
    ///
    /// `source`, `contract_json`, and `entry` identify the JavaScript module;
    /// `commit` forwards host operations and `control` forwards renderer controls.
    /// Returns the mounted session.
    ///
    /// # Errors
    /// Returns an error when the script, contract, or initial mount fails.
    pub fn new_with_control(
        source: &str,
        contract_json: &str,
        entry: &str,
        commit: impl Fn(String) -> String + 'static,
        control: impl Fn(String) -> String + 'static,
    ) -> Result<Self, String> {
        let runtime = Runtime::new().map_err(js_error)?;
        let context = Context::full(&runtime).map_err(js_error)?;
        context.with(|ctx| {
            let globals = ctx.globals();
            globals
                .set("__arguiMobile", cfg!(target_os = "android"))
                .map_err(js_error)?;
            globals
                .set("__arguiContractJson", contract_json)
                .map_err(js_error)?;
            globals
                .set(
                    "__arguiSend",
                    Function::new(ctx.clone(), commit).map_err(js_error)?,
                )
                .map_err(js_error)?;
            globals
                .set(
                    "__arguiControl",
                    Function::new(ctx.clone(), control).map_err(js_error)?,
                )
                .map_err(js_error)?;
            let _: () = ctx.eval(include_str!("bootstrap.js")).map_err(js_error)?;
            let module =
                Module::declare(ctx.clone(), "gallery-core.mjs", source).map_err(js_error)?;
            let (module, evaluated) = CaughtError::catch(&ctx, module.eval())
                .map_err(|error| format!("QuickJS: {error}"))?;
            CaughtError::catch(&ctx, evaluated.finish::<()>())
                .map_err(|error| format!("QuickJS module: {error}"))?;
            let mount: Function = module.get(entry).map_err(js_error)?;
            let bridge: Object = globals.get("__arguiBridge").map_err(js_error)?;
            let hash: String = ctx
                .eval("JSON.parse(__arguiContractJson).abiHash")
                .map_err(js_error)?;
            let dispose: Function = CaughtError::catch(&ctx, mount.call((bridge, hash)))
                .map_err(|error| format!("QuickJS: {error}"))?;
            globals.set("__arguiDispose", dispose).map_err(js_error)
        })?;
        let gallery = Self { runtime, context };
        gallery.drain_jobs()?;
        Ok(gallery)
    }

    /// Delivers a native callback encoded as JSON to the mounted application.
    ///
    /// # Errors
    /// Returns an error if the callback or its scheduled microtasks fail.
    pub fn deliver(&self, delivery_json: &str) -> Result<(), String> {
        self.context.with(|ctx| {
            let deliver: Function = ctx.globals().get("__arguiDeliver").map_err(js_error)?;
            deliver.call::<_, ()>((delivery_json,)).map_err(js_error)
        })?;
        self.drain_jobs()
    }

    /// Delivers a real renderer profile encoded as JSON to the active subscriber.
    ///
    /// `profile_json` contains the latest native CPU/GPU/damage sample. Returns
    /// after its JavaScript callback and queued microtasks complete.
    ///
    /// # Errors
    /// Returns an error if the profile callback fails.
    pub fn deliver_profile(&self, profile_json: &str) -> Result<(), String> {
        self.context.with(|ctx| {
            let deliver: Function = ctx
                .globals()
                .get("__arguiDeliverProfile")
                .map_err(js_error)?;
            deliver.call::<_, ()>((profile_json,)).map_err(js_error)
        })?;
        self.drain_jobs()
    }

    /// Returns definitions registered by imported JavaScript WGSL modules.
    ///
    /// Returns the JSON array accumulated before the module mounted.
    ///
    /// # Errors
    /// Returns an error if the QuickJS registry cannot be serialized.
    pub fn effect_definitions_json(&self) -> Result<String, String> {
        self.context.with(|ctx| {
            ctx.eval("JSON.stringify(globalThis.__arguiNativeEffects)")
                .map_err(js_error)
        })
    }

    /// Fires due JavaScript interval callbacks at `elapsed_ms` since startup.
    ///
    /// # Errors
    /// Returns an error if a timer or its scheduled microtasks fail.
    pub fn tick(&self, elapsed_ms: f64) -> Result<(), String> {
        self.context.with(|ctx| {
            let tick: Function = ctx.globals().get("__arguiTick").map_err(js_error)?;
            tick.call::<_, ()>((elapsed_ms,)).map_err(js_error)
        })?;
        self.drain_jobs()
    }

    /// Returns a bounded sleep until the next interval; idle sessions sleep for one second.
    ///
    /// # Errors
    /// Returns an error if the timer scheduler cannot be queried.
    pub fn next_wake(&self, elapsed_ms: f64) -> Result<Duration, String> {
        let remaining: Option<f64> = self.context.with(|ctx| {
            let next: Function = ctx.globals().get("__arguiNextTimer").map_err(js_error)?;
            next.call((elapsed_ms,)).map_err(js_error)
        })?;
        let milliseconds = remaining.unwrap_or(1000.0).clamp(0.0, 1000.0);
        Ok(Duration::from_secs_f64(milliseconds / 1000.0))
    }

    /// Unmounts the application and commits disposal of its native root.
    ///
    /// # Errors
    /// Returns an error if the disposer or its scheduled microtasks fail.
    pub fn dispose(&self) -> Result<(), String> {
        self.context.with(|ctx| {
            let dispose: Function = ctx.globals().get("__arguiDispose").map_err(js_error)?;
            dispose.call::<_, ()>(()).map_err(js_error)
        })?;
        self.drain_jobs()
    }

    /// Runs every queued Promise or Solid effect job to a stable boundary.
    fn drain_jobs(&self) -> Result<(), String> {
        while self.runtime.execute_pending_job().map_err(js_error)? {}
        Ok(())
    }
}

/// Converts a QuickJS API error into a message suitable for the native actor.
fn js_error(error: impl std::fmt::Debug) -> String {
    format!("QuickJS: {error:?}")
}
