use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use argui_core::ColorScheme;
use argui_platform::WindowConfig;
use argui_render::RendererConfig;
use argui_runtime::{NativeHost, RuntimeEvent, ThemeBridge, WebHostHandle, WireOperation};
use js_sys::{Function, JSON};
use wasm_bindgen::{JsCast, JsValue, closure::Closure, prelude::wasm_bindgen};

use crate::delivery::event_json;

const CONTRACT_JSON: &str = include_str!("../../../packages/host/src/contract.generated.json");
/// Callback registrations grouped by application-owned theme session.
type ThemeSubscribers = Rc<RefCell<HashMap<u32, Vec<(u64, Function)>>>>;

/// A browser presentation bridge with the same contract, commit, and subscription
/// methods used by the Solid and React adapters on desktop.
#[wasm_bindgen]
pub struct ArguiWebHost {
    runtime: WebHostHandle,
    subscriber: Rc<RefCell<Option<(u64, Function)>>>,
    next_subscription: Cell<u64>,
    theme: Rc<RefCell<ThemeBridge>>,
    theme_subscribers: ThemeSubscribers,
    next_theme_subscription: Cell<u64>,
    system_query: Option<web_sys::MediaQueryList>,
    system_listener: Option<Closure<dyn FnMut(web_sys::Event)>>,
}

#[wasm_bindgen]
impl ArguiWebHost {
    /// Mounts a Rust-rendered canvas into the DOM element named by `parent_id`.
    ///
    /// # Errors
    /// Returns a JavaScript error when the parent is absent, the ABI is stale,
    /// or the browser event loop cannot start.
    #[wasm_bindgen(constructor)]
    pub fn new(parent_id: &str) -> Result<ArguiWebHost, JsValue> {
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| JsValue::from_str("Argui requires a browser document"))?;
        if document.get_element_by_id(parent_id).is_none() {
            return Err(JsValue::from_str(&format!(
                "Argui canvas parent #{parent_id} was not found"
            )));
        }
        let host = NativeHost::with_builtins().map_err(js_error)?;
        let contract: serde_json::Value = serde_json::from_str(CONTRACT_JSON).map_err(js_error)?;
        if contract["abiHash"].as_str() != Some(&host.abi_hash().to_string()) {
            return Err(JsValue::from_str(
                "Argui browser contract is stale; run bun run generate:jsx",
            ));
        }
        let subscriber = Rc::<RefCell<Option<(u64, Function)>>>::default();
        let deliveries = subscriber.clone();
        let target = parent_id.to_owned();
        let runtime = WebHostHandle::start(
            WindowConfig {
                append_to_document: false,
                web_parent_id: Some(parent_id.to_owned()),
                ..WindowConfig::default()
            },
            RendererConfig::default(),
            move |delivery| {
                let callback = deliveries
                    .borrow()
                    .as_ref()
                    .map(|(_, callback)| callback.clone());
                if let Some(callback) = callback
                    && let Ok(payload) = JSON::parse(&event_json(&delivery).to_string())
                    && let Err(error) = callback.call1(&JsValue::UNDEFINED, &payload)
                {
                    web_sys::console::error_1(&error);
                }
            },
            move |event| report_runtime_event(&target, event),
        )
        .map_err(js_error)?;
        let system_query = web_sys::window().and_then(|window| {
            window
                .match_media("(prefers-color-scheme: dark)")
                .ok()
                .flatten()
        });
        let mut theme_bridge = ThemeBridge::default();
        let scheme = if system_query
            .as_ref()
            .is_some_and(web_sys::MediaQueryList::matches)
        {
            ColorScheme::Dark
        } else {
            ColorScheme::Light
        };
        theme_bridge.set_system_scheme(scheme).map_err(js_error)?;
        let theme = Rc::new(RefCell::new(theme_bridge));
        let theme_subscribers: ThemeSubscribers = Rc::default();
        let system_listener = system_query.as_ref().map(|query| {
            let query = query.clone();
            let theme = Rc::clone(&theme);
            let subscribers = Rc::clone(&theme_subscribers);
            Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
                let scheme = if query.matches() {
                    ColorScheme::Dark
                } else {
                    ColorScheme::Light
                };
                let changes = theme.borrow_mut().set_system_scheme(scheme);
                match changes {
                    Ok(changes) => {
                        for (id, json) in changes {
                            let Ok(id) = u32::try_from(id) else { continue };
                            if let Ok(snapshot) = JSON::parse(&json) {
                                let listeners = subscribers.borrow().get(&id).cloned();
                                if let Some(listeners) = listeners {
                                    for (_, listener) in listeners {
                                        if let Err(error) =
                                            listener.call1(&JsValue::UNDEFINED, &snapshot)
                                        {
                                            web_sys::console::error_1(&error);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(error) => web_sys::console::error_1(&JsValue::from_str(&error)),
                }
            })
        });
        if let (Some(query), Some(listener)) = (&system_query, &system_listener) {
            query.add_event_listener_with_callback("change", listener.as_ref().unchecked_ref())?;
        }
        Ok(Self {
            runtime,
            subscriber,
            next_subscription: Cell::new(1),
            theme,
            theme_subscribers,
            next_theme_subscription: Cell::new(1),
            system_query,
            system_listener,
        })
    }

    /// Returns the generated schema contract required by the TypeScript host.
    ///
    /// # Errors
    /// Returns a JavaScript parse error if the embedded contract is malformed.
    pub fn contract(&self) -> Result<JsValue, JsValue> {
        JSON::parse(CONTRACT_JSON)
    }

    /// Validates and queues one atomic presentation transaction.
    /// `operations` is the adapter's array of tagged wire operations.
    ///
    /// # Errors
    /// Returns a JavaScript error for malformed JSON or a rejected transaction.
    pub fn commit(&mut self, operations: JsValue) -> Result<(), JsValue> {
        let json = JSON::stringify(&operations)?
            .as_string()
            .ok_or_else(|| JsValue::from_str("Argui operations could not be serialized"))?;
        let operations: Vec<WireOperation> = serde_json::from_str(&json).map_err(js_error)?;
        self.runtime.commit(operations).map_err(js_error)
    }

    /// Installs `callback` for UI event deliveries and returns its unsubscriber.
    /// Replacing a callback invalidates the previous unsubscriber.
    pub fn subscribe(&self, callback: Function) -> Function {
        let token = self.next_subscription.get();
        self.next_subscription.set(token.wrapping_add(1));
        *self.subscriber.borrow_mut() = Some((token, callback));
        let subscriber = self.subscriber.clone();
        Closure::<dyn FnMut()>::new(move || {
            let is_current = subscriber
                .borrow()
                .as_ref()
                .is_some_and(|(current, _)| *current == token);
            if is_current {
                *subscriber.borrow_mut() = None;
            }
        })
        .into_js_value()
        .unchecked_into()
    }

    /// Creates one app-owned typed theme and returns its host ID and initial snapshot.
    /// `definition` contains token declarations and variants.
    ///
    /// # Errors
    /// Returns a browser exception for an invalid schema or unsupported token value.
    #[wasm_bindgen(js_name = themeCreate)]
    pub fn theme_create(&self, definition: JsValue) -> Result<JsValue, JsValue> {
        let json = js_json(&definition)?;
        let result = self.theme.borrow_mut().create(&json).map_err(js_error)?;
        JSON::parse(&result)
    }

    /// Applies one atomic `patch` to the theme named by `id` and returns its snapshot.
    ///
    /// # Errors
    /// Returns a browser exception for an unknown theme or invalid patch.
    #[wasm_bindgen(js_name = themeUpdate)]
    pub fn theme_update(&self, id: u32, patch: JsValue) -> Result<JsValue, JsValue> {
        let json = js_json(&patch)?;
        let result = self
            .theme
            .borrow_mut()
            .update(u64::from(id), &json)
            .map_err(js_error)?;
        let snapshot = JSON::parse(&result)?;
        let subscribers = self.theme_subscribers.borrow().get(&id).cloned();
        if let Some(subscribers) = subscribers {
            for (_, callback) in subscribers {
                if let Err(error) = callback.call1(&JsValue::UNDEFINED, &snapshot) {
                    web_sys::console::error_1(&error);
                }
            }
        }
        Ok(snapshot)
    }

    /// Subscribes `callback` to snapshot changes for one theme `id`.
    /// Returns an unsubscriber that removes only this registration.
    #[wasm_bindgen(js_name = themeSubscribe)]
    pub fn theme_subscribe(&self, id: u32, callback: Function) -> Function {
        let token = self.next_theme_subscription.get();
        self.next_theme_subscription.set(token.wrapping_add(1));
        self.theme_subscribers
            .borrow_mut()
            .entry(id)
            .or_default()
            .push((token, callback));
        let subscribers = Rc::clone(&self.theme_subscribers);
        Closure::<dyn FnMut()>::new(move || {
            let mut subscribers = subscribers.borrow_mut();
            if let Some(entries) = subscribers.get_mut(&id) {
                entries.retain(|(current, _)| *current != token);
                if entries.is_empty() {
                    subscribers.remove(&id);
                }
            }
        })
        .into_js_value()
        .unchecked_into()
    }

    /// Releases theme `id` and its host-side subscriptions.
    #[wasm_bindgen(js_name = themeDispose)]
    pub fn theme_dispose(&self, id: u32) {
        self.theme_subscribers.borrow_mut().remove(&id);
        self.theme.borrow_mut().dispose(u64::from(id));
    }
}

impl Drop for ArguiWebHost {
    fn drop(&mut self) {
        if let (Some(query), Some(listener)) = (&self.system_query, &self.system_listener) {
            let _ = query
                .remove_event_listener_with_callback("change", listener.as_ref().unchecked_ref());
        }
    }
}

/// Encodes a browser value as JSON for the Rust theme wire parser.
///
/// # Errors
/// Returns an exception when the value cannot be serialized.
fn js_json(value: &JsValue) -> Result<String, JsValue> {
    JSON::stringify(value)?
        .as_string()
        .ok_or_else(|| JsValue::from_str("theme value could not be serialized"))
}

/// Converts a Rust error to a browser exception.
/// `error` is displayed verbatim; the returned value can be thrown by wasm-bindgen.
fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

/// Publishes an asynchronous renderer or platform error on the canvas parent.
/// `parent_id` identifies the mounted host; successful events need no JS work.
fn report_runtime_event(parent_id: &str, event: RuntimeEvent) {
    let message = match event {
        RuntimeEvent::RendererFailed(error)
        | RuntimeEvent::LayoutFailed(error)
        | RuntimeEvent::CommandFailed(error) => error,
        RuntimeEvent::Platform(argui_platform::PlatformEvent::WindowCreationFailed(error)) => error,
        _ => return,
    };
    web_sys::console::error_1(&JsValue::from_str(&format!("Argui: {message}")));
    let Some(target) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(parent_id))
    else {
        return;
    };
    let options = web_sys::CustomEventInit::new();
    options.set_detail(&JsValue::from_str(&message));
    if let Ok(event) = web_sys::CustomEvent::new_with_event_init_dict("argui:error", &options) {
        let _ = target.dispatch_event(&event);
    }
}
