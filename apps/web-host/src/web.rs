use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use argui_platform::WindowConfig;
use argui_render::RendererConfig;
use argui_runtime::{NativeHost, RuntimeEvent, WebHostHandle, WireOperation};
use js_sys::{Function, JSON};
use wasm_bindgen::{JsCast, JsValue, closure::Closure, prelude::wasm_bindgen};

use crate::delivery::event_json;

const CONTRACT_JSON: &str = include_str!("../../../packages/host/src/contract.generated.json");

/// A browser presentation bridge with the same contract, commit, and subscription
/// methods used by the Solid and React adapters on desktop.
#[wasm_bindgen]
pub struct ArguiWebHost {
    runtime: WebHostHandle,
    subscriber: Rc<RefCell<Option<(u64, Function)>>>,
    next_subscription: Cell<u64>,
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
                if let Some(callback) = callback {
                    if let Ok(payload) = JSON::parse(&event_json(&delivery).to_string()) {
                        if let Err(error) = callback.call1(&JsValue::UNDEFINED, &payload) {
                            web_sys::console::error_1(&error);
                        }
                    }
                }
            },
            move |event| report_runtime_event(&target, event),
        )
        .map_err(js_error)?;
        Ok(Self {
            runtime,
            subscriber,
            next_subscription: Cell::new(1),
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
