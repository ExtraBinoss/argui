use crate::{
    NativeWebView, WebViewBackend, WebViewError, WebViewEvent, WebViewEventSink, WebViewPolicy,
    WebViewSource, WebViewState,
};
use std::{collections::HashMap, rc::Rc};
use wasm_bindgen::{JsCast, prelude::*};

#[wasm_bindgen(module = "/src/browser/dom.js")]
extern "C" {
    type DomView;
    #[wasm_bindgen(catch, js_name = createView)]
    fn create_view(
        canvas: &web_sys::HtmlCanvasElement,
        email: bool,
        callback: &JsValue,
        sandbox: &str,
        relay: Option<String>,
    ) -> Result<DomView, JsValue>;
    #[wasm_bindgen(method, catch)]
    fn load(this: &DomView, value: &str) -> Result<(), JsValue>;
    #[wasm_bindgen(method, catch)]
    fn bounds(this: &DomView, x: f32, y: f32, width: f32, height: f32) -> Result<(), JsValue>;
    #[wasm_bindgen(method, catch)]
    fn visible(this: &DomView, value: bool) -> Result<(), JsValue>;
    #[wasm_bindgen(method, catch)]
    fn focus(this: &DomView) -> Result<(), JsValue>;
    #[wasm_bindgen(method)]
    fn dispose(this: &DomView);
    #[wasm_bindgen(method)]
    fn clip(this: &DomView, x: f32, y: f32, width: f32, height: f32);
    type WakeTimer;
    #[wasm_bindgen(js_name = createWakeTimer)]
    fn create_wake_timer(callback: &JsValue) -> WakeTimer;
    #[wasm_bindgen(method)]
    fn schedule(this: &WakeTimer, milliseconds: f64);
    #[wasm_bindgen(method, js_name = dispose)]
    fn dispose_timer(this: &WakeTimer);
}

/// Browser DOM-backed implementation of the WebView pool backend.
pub struct BrowserBackend {
    hosts: HashMap<u64, web_sys::HtmlCanvasElement>,
    wake: Rc<dyn Fn()>,
    timer: WakeTimer,
    _timer_callback: Closure<dyn Fn()>,
    views: std::cell::RefCell<HashMap<crate::WebViewId, std::rc::Weak<DomView>>>,
}

impl BrowserBackend {
    /// Creates a browser backend and a callback used to wake its host.
    pub fn new(wake: impl Fn() + 'static) -> Self {
        let wake: Rc<dyn Fn()> = Rc::new(wake);
        let callback_wake = wake.clone();
        let timer_callback = Closure::wrap(Box::new(move || callback_wake()) as Box<dyn Fn()>);
        let timer = create_wake_timer(timer_callback.as_ref());
        Self {
            hosts: HashMap::new(),
            wake,
            timer,
            _timer_callback: timer_callback,
            views: Default::default(),
        }
    }
    /// Associates a host ID with its canvas element.
    /// `id` identifies the host and `canvas` is the DOM canvas attached to it.
    pub fn register_host(&mut self, id: u64, canvas: web_sys::HtmlCanvasElement) {
        self.hosts.insert(id, canvas);
    }
    /// Schedule one redraw at the next pool eviction, rather than polling while idle.
    /// Schedules a wake after the optional maintenance delay.
    /// `after` is the delay before the callback is invoked, if any.
    pub fn schedule_maintenance(&self, after: Option<std::time::Duration>) {
        self.timer
            .schedule(after.map_or(-1.0, |duration| duration.as_secs_f64() * 1000.0));
    }
    /// Applies rectangular clipping to browser elements for these mounts.
    pub fn sync_clips(&self, mounts: &[(crate::WebViewMount, Option<argui_core::Rect>)]) {
        let mut views = self.views.borrow_mut();
        views.retain(|_, view| view.strong_count() > 0);
        for (mount, clip) in mounts {
            if let Some(view) = views
                .get(&mount.state.id())
                .and_then(std::rc::Weak::upgrade)
            {
                let clip = clip.unwrap_or(mount.bounds);
                view.clip(
                    clip.origin.x,
                    clip.origin.y,
                    clip.size.width,
                    clip.size.height,
                );
            }
        }
    }
}
impl Drop for BrowserBackend {
    fn drop(&mut self) {
        self.timer.dispose_timer();
    }
}

pub struct BrowserView {
    dom: Rc<DomView>,
    policy: WebViewPolicy,
    _callback: Closure<dyn Fn(String, String)>,
}

impl WebViewBackend for BrowserBackend {
    type View = BrowserView;
    fn create(
        &mut self,
        host: u64,
        state: &WebViewState,
        events: WebViewEventSink,
    ) -> Result<Self::View, WebViewError> {
        let canvas = self.hosts.get(&host).ok_or(WebViewError::MissingHost)?;
        let wake = self.wake.clone();
        let callback = Closure::wrap(Box::new(move |kind: String, value: String| {
            match kind.as_str() {
                "navigation" => events.emit(WebViewEvent::NavigationRequested(value)),
                "loaded" => events.emit(WebViewEvent::Loaded(value)),
                "loading" => events.emit(WebViewEvent::Loading(value)),
                "error" => events.emit(WebViewEvent::Error(WebViewError::Native(value))),
                _ => (),
            }
            wake();
        }) as Box<dyn Fn(String, String)>);
        let options = state.options();
        let relay = match options.compatibility_mode() {
            crate::WebCompatibility::Isolated => None,
            crate::WebCompatibility::Compatible { relay } => Some(relay.to_string()),
        };
        let dom = Rc::new(
            create_view(
                canvas,
                state.policy() == WebViewPolicy::RestrictedHtml,
                callback.as_ref().unchecked_ref(),
                &options.sandbox(),
                relay,
            )
            .map_err(error)?,
        );
        self.views
            .borrow_mut()
            .insert(state.id(), Rc::downgrade(&dom));
        Ok(BrowserView {
            dom,
            policy: state.policy(),
            _callback: callback,
        })
    }
}

impl NativeWebView for BrowserView {
    fn load(&mut self, source: &WebViewSource) -> Result<(), WebViewError> {
        if source.policy() != self.policy {
            return Err(WebViewError::PolicyMismatch);
        }
        let value = match source {
            WebViewSource::Html(html) => crate::email_document(html),
            WebViewSource::Url(url) => {
                WebViewSource::url(url.as_str())?;
                url.to_string()
            }
        };
        self.dom.load(&value).map_err(error)
    }
    fn set_bounds(&mut self, rect: argui_core::Rect) -> Result<(), WebViewError> {
        self.dom
            .bounds(
                rect.origin.x,
                rect.origin.y,
                rect.size.width,
                rect.size.height,
            )
            .map_err(error)
    }
    fn set_visible(&mut self, visible: bool) -> Result<(), WebViewError> {
        self.dom.visible(visible).map_err(error)
    }
    fn focus(&mut self) -> Result<(), WebViewError> {
        self.dom.focus().map_err(error)
    }
}
impl Drop for BrowserView {
    fn drop(&mut self) {
        self.dom.dispose();
    }
}
fn error(value: JsValue) -> WebViewError {
    WebViewError::Native(format!("{value:?}"))
}
