use std::{
    collections::HashMap,
    sync::{
        Arc, LazyLock, RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState, hotkey::HotKey};

use crate::{
    GlobalShortcut, GlobalShortcutEvent, GlobalShortcutId, GlobalShortcutState,
    global_shortcut::validate_global_shortcuts,
};

/// Thread-safe callback receiving native global-shortcut events.
pub type GlobalShortcutEventHandler = Arc<dyn Fn(GlobalShortcutEvent) + Send + Sync>;
/// Thread-safe callback receiving asynchronous registration failures.
pub type GlobalShortcutErrorHandler = Arc<dyn Fn(String) + Send + Sync>;

/// Owner of the operating-system registrations for one application.
pub struct NativeGlobalShortcuts {
    _backend: NativeGlobalShortcutBackend,
}

enum NativeGlobalShortcutBackend {
    HotKey {
        _registration: HotKeyRegistration,
    },
    #[cfg(target_os = "linux")]
    Portal {
        _session: crate::wayland_global_shortcuts::WaylandGlobalShortcuts,
    },
}

struct HotKeyRegistration {
    manager: GlobalHotKeyManager,
    hotkeys: Vec<HotKey>,
    route: u64,
}

struct ActiveRoute {
    id: u64,
    shortcuts: HashMap<u32, GlobalShortcutId>,
    handler: GlobalShortcutEventHandler,
}

static ACTIVE_ROUTE: LazyLock<RwLock<Option<ActiveRoute>>> = LazyLock::new(|| RwLock::new(None));
static NEXT_ROUTE: AtomicU64 = AtomicU64::new(1);

impl NativeGlobalShortcuts {
    /// Registers `shortcuts` for `application_id` and forwards native state
    /// changes to `handler`.
    ///
    /// The owner must remain alive while registrations are needed. `on_error`
    /// receives failures that occur after asynchronous portal registration has
    /// started.
    ///
    /// # Errors
    /// Returns an error when validation, manager creation, immediate
    /// registration, or portal-worker creation fails.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn new(
        application_id: &str,
        shortcuts: &[GlobalShortcut],
        handler: GlobalShortcutEventHandler,
        on_error: GlobalShortcutErrorHandler,
    ) -> Result<Self, String> {
        validate_global_shortcuts(shortcuts).map_err(|error| error.to_string())?;
        #[cfg(target_os = "linux")]
        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            return crate::wayland_global_shortcuts::WaylandGlobalShortcuts::new(
                application_id,
                shortcuts,
                handler,
                on_error,
            )
            .map(|portal| Self {
                _backend: NativeGlobalShortcutBackend::Portal { _session: portal },
            });
        }
        #[cfg(not(target_os = "linux"))]
        let _ = (application_id, on_error);
        let registration = HotKeyRegistration::new(shortcuts, handler)?;
        Ok(Self {
            _backend: NativeGlobalShortcutBackend::HotKey {
                _registration: registration,
            },
        })
    }
}

impl HotKeyRegistration {
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn new(
        shortcuts: &[GlobalShortcut],
        handler: GlobalShortcutEventHandler,
    ) -> Result<Self, String> {
        let hotkeys = shortcuts
            .iter()
            .map(|shortcut| {
                shortcut
                    .accelerator
                    .parse::<HotKey>()
                    .map(|hotkey| (hotkey, shortcut.id.clone()))
                    .map_err(|error| {
                        format!(
                            "invalid global shortcut '{}': {error}",
                            shortcut.accelerator
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let by_native_id = hotkeys
            .iter()
            .map(|(hotkey, id)| (hotkey.id(), id.clone()))
            .collect::<HashMap<_, _>>();
        if by_native_id.len() != hotkeys.len() {
            return Err("two global shortcuts resolve to the same accelerator".into());
        }
        let manager = GlobalHotKeyManager::new().map_err(|error| error.to_string())?;
        let hotkeys = hotkeys
            .into_iter()
            .map(|(hotkey, _)| hotkey)
            .collect::<Vec<_>>();
        manager
            .register_all(&hotkeys)
            .map_err(|error| error.to_string())?;
        let route = install_event_handler(by_native_id, handler)?;
        Ok(Self {
            manager,
            hotkeys,
            route,
        })
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Drop for HotKeyRegistration {
    fn drop(&mut self) {
        let _ = self.manager.unregister_all(&self.hotkeys);
        if let Ok(mut active) = ACTIVE_ROUTE.write()
            && active.as_ref().is_some_and(|route| route.id == self.route)
        {
            *active = None;
        }
    }
}

/// Installs the process-wide native callback and maps platform IDs back to app IDs.
#[cfg_attr(coverage_nightly, coverage(off))]
fn install_event_handler(
    shortcuts: HashMap<u32, GlobalShortcutId>,
    handler: GlobalShortcutEventHandler,
) -> Result<u64, String> {
    GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
        let Ok(active) = ACTIVE_ROUTE.read() else {
            return;
        };
        let Some(active) = active.as_ref() else {
            return;
        };
        let Some(id) = active.shortcuts.get(&event.id).cloned() else {
            return;
        };
        let state = match event.state {
            HotKeyState::Pressed => GlobalShortcutState::Pressed,
            HotKeyState::Released => GlobalShortcutState::Released,
        };
        (active.handler)(GlobalShortcutEvent {
            id,
            state,
            activation_token: None,
        });
    }));
    let route = NEXT_ROUTE.fetch_add(1, Ordering::Relaxed);
    let mut active = ACTIVE_ROUTE
        .write()
        .map_err(|_| "global shortcut event route is poisoned".to_owned())?;
    if active.is_some() {
        return Err("another global shortcut manager is already active".into());
    }
    *active = Some(ActiveRoute {
        id: route,
        shortcuts,
        handler,
    });
    Ok(route)
}
