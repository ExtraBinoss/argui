use std::{sync::OnceLock, thread::JoinHandle};

use ashpd::desktop::{
    CreateSessionOptions,
    global_shortcuts::{BindShortcutsOptions, GlobalShortcuts, NewShortcut},
};
use futures_channel::oneshot;
use futures_util::{FutureExt, StreamExt};

use crate::{
    GlobalShortcut, GlobalShortcutEvent, GlobalShortcutId, GlobalShortcutState,
    native_global_shortcuts::{GlobalShortcutErrorHandler, GlobalShortcutEventHandler},
};

/// Active XDG desktop-portal shortcut session for a Wayland application.
pub(crate) struct WaylandGlobalShortcuts {
    stop: Option<oneshot::Sender<()>>,
    worker: Option<JoinHandle<()>>,
}

struct HostRegistration {
    application_id: String,
    result: Result<(), String>,
}

static HOST_REGISTRATION: OnceLock<HostRegistration> = OnceLock::new();

/// Associates this unsandboxed process with `application_id` for Wayland
/// global-shortcut portal calls.
///
/// Call this before any other XDG desktop portal API. Repeating the call with
/// the same ID returns the cached outcome.
///
/// # Errors
/// Returns an error for an invalid ID, a conflicting ID in the same process,
/// or when the desktop portal rejects host registration.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn prepare_wayland_global_shortcuts(application_id: &str) -> Result<(), String> {
    let registration = HOST_REGISTRATION.get_or_init(|| HostRegistration {
        application_id: application_id.to_owned(),
        result: pollster::block_on(register_host_application(application_id)),
    });
    if registration.application_id != application_id {
        return Err(format!(
            "Wayland portal connection is already registered for '{}'",
            registration.application_id
        ));
    }
    registration.result.clone()
}

/// Performs the one-time asynchronous host registration used by the portal.
#[cfg_attr(coverage_nightly, coverage(off))]
async fn register_host_application(application_id: &str) -> Result<(), String> {
    let application_id = ashpd::AppID::try_from(application_id)
        .map_err(|error| format!("invalid portal application id: {error}"))?;
    ashpd::register_host_app(application_id)
        .await
        .map_err(|error| format!("could not register the host application: {error}"))
}

impl WaylandGlobalShortcuts {
    /// Starts portal registration without blocking the application's event loop.
    ///
    /// `application_id` identifies the host application to the portal,
    /// `shortcuts` are presented for approval, `handler` receives activation
    /// changes, and `on_error` receives asynchronous portal failures.
    ///
    /// # Errors
    /// Returns an error when the portal worker thread cannot be created.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(crate) fn new(
        application_id: &str,
        shortcuts: &[GlobalShortcut],
        handler: GlobalShortcutEventHandler,
        on_error: GlobalShortcutErrorHandler,
    ) -> Result<Self, String> {
        let application_id = application_id.to_owned();
        let shortcuts = shortcuts.to_vec();
        let (stop, stopped) = oneshot::channel();
        let worker = std::thread::Builder::new()
            .name("argui-wayland-shortcuts".into())
            .spawn(move || {
                if let Err(error) =
                    pollster::block_on(run_portal(application_id, shortcuts, handler, stopped))
                {
                    on_error(format!("Wayland global shortcuts portal failed: {error}"));
                }
            })
            .map_err(|error| error.to_string())?;
        Ok(Self {
            stop: Some(stop),
            worker: Some(worker),
        })
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Drop for WaylandGlobalShortcuts {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
/// Runs the portal session until the owner requests shutdown.
///
/// `application_id` associates an unsandboxed process with its desktop ID,
/// `shortcuts` contains the validated portable accelerators, `handler`
/// receives portal activations, and `stopped` closes the session cleanly.
async fn run_portal(
    application_id: String,
    shortcuts: Vec<GlobalShortcut>,
    handler: GlobalShortcutEventHandler,
    stopped: oneshot::Receiver<()>,
) -> Result<(), String> {
    prepare_wayland_global_shortcuts(&application_id)?;
    let portal = GlobalShortcuts::new()
        .await
        .map_err(|error| error.to_string())?;
    let session = portal
        .create_session(CreateSessionOptions::default())
        .await
        .map_err(|error| error.to_string())?;
    let requests = shortcuts
        .iter()
        .map(|shortcut| {
            let description = shortcut.id.as_str().replace(['-', '_'], " ");
            let trigger = portal_trigger(&shortcut.accelerator);
            NewShortcut::new(shortcut.id.as_str(), description)
                .preferred_trigger(trigger.as_deref())
        })
        .collect::<Vec<_>>();
    let mut stopped = stopped.fuse();
    let binding = portal
        .bind_shortcuts(&session, &requests, None, BindShortcutsOptions::default())
        .fuse();
    futures_util::pin_mut!(binding);
    let request = futures_util::select! {
        result = binding => result.map_err(|error| error.to_string())?,
        _ = stopped => {
            let _ = session.close().await;
            return Ok(());
        }
    };
    let bound = request.response().map_err(|error| error.to_string())?;
    if !shortcuts.is_empty() && bound.shortcuts().is_empty() {
        return Err("the user did not authorize any requested shortcut".into());
    }
    let mut activated = portal
        .receive_activated()
        .await
        .map_err(|error| error.to_string())?
        .fuse();
    let mut deactivated = portal
        .receive_deactivated()
        .await
        .map_err(|error| error.to_string())?
        .fuse();
    loop {
        futures_util::select! {
            event = activated.next() => {
                let event = event.ok_or("Wayland shortcut activation stream closed")?;
                handler(portal_event(
                    event.shortcut_id(),
                    GlobalShortcutState::Pressed,
                    event.options().get("activation_token").and_then(portal_token),
                ));
            },
            event = deactivated.next() => {
                let event = event.ok_or("Wayland shortcut deactivation stream closed")?;
                handler(portal_event(
                    event.shortcut_id(),
                    GlobalShortcutState::Released,
                    event.options().get("activation_token").and_then(portal_token),
                ));
            },
            _ = stopped => break,
        }
    }
    let _ = session.close().await;
    Ok(())
}

/// Converts a portal signal into Argui's platform-neutral event.
fn portal_event(
    id: &str,
    state: GlobalShortcutState,
    activation_token: Option<String>,
) -> GlobalShortcutEvent {
    GlobalShortcutEvent {
        id: GlobalShortcutId::new(id),
        state,
        activation_token,
    }
}

/// Extracts an XDG activation token from a portal option value.
fn portal_token(value: &ashpd::zbus::zvariant::OwnedValue) -> Option<String> {
    value.downcast_ref::<&str>().ok().map(str::to_owned)
}

/// Converts an Argui accelerator into the XDG shortcuts trigger syntax.
fn portal_trigger(accelerator: &str) -> Option<String> {
    let mut parts = accelerator.split('+').map(str::trim).peekable();
    let mut output = Vec::new();
    while let Some(part) = parts.next() {
        let key = parts.peek().is_none();
        let normalized = match part.to_ascii_lowercase().as_str() {
            "cmdorctrl" | "cmd" | "command" | "ctrl" | "control" => "CTRL".into(),
            "alt" | "option" => "ALT".into(),
            "shift" => "SHIFT".into(),
            "meta" | "super" | "logo" => "LOGO".into(),
            "space" if key => "space".into(),
            "enter" if key => "Return".into(),
            value if key && value.starts_with("key") && value.len() == 4 => value[3..].into(),
            _ => part.to_owned(),
        };
        output.push(normalized);
    }
    (!output.is_empty()).then(|| output.join("+"))
}
