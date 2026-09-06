use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    NativeWebView, WebViewBackend, WebViewError, WebViewEvent, WebViewEventSink, WebViewPolicy,
    WebViewSource, WebViewState,
};
use argui_core::Rect;

const DOCUMENT_URL: &str = "argui-content://localhost/document";
const DOCUMENT_CSP: &str = "default-src 'none'; script-src 'none'; style-src 'unsafe-inline'; img-src data:; base-uri 'none'; form-action 'none'; frame-ancestors 'none'";

#[cfg(target_os = "linux")]
type Host = gtk::Fixed;
#[cfg(not(target_os = "linux"))]
type Host = Arc<dyn raw_window_handle::HasWindowHandle + Send + Sync>;

#[derive(Default)]
pub struct WryBackend {
    hosts: HashMap<u64, Host>,
    #[cfg(target_os = "linux")]
    input_waker: Option<std::rc::Rc<dyn Fn()>>,
}

impl WryBackend {
    /// Wake the owning UI loop when a native child takes pointer or keyboard input.
    #[cfg(target_os = "linux")]
    pub fn input_waker(mut self, wake: impl Fn() + 'static) -> Self {
        self.input_waker = Some(std::rc::Rc::new(wake));
        self
    }
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(target_os = "linux")]
    pub fn register_gtk_host(&mut self, id: u64, container: gtk::Fixed) {
        self.hosts.insert(id, container);
    }

    #[cfg(not(target_os = "linux"))]
    pub fn register_host(
        &mut self,
        id: u64,
        window: Arc<impl raw_window_handle::HasWindowHandle + Send + Sync + 'static>,
    ) {
        self.hosts.insert(id, window);
    }
}

pub struct WryView {
    view: wry::WebView,
    policy: WebViewPolicy,
    document: Arc<Mutex<Vec<u8>>>,
    document_revision: u64,
    #[cfg(target_os = "linux")]
    host: gtk::Fixed,
    #[cfg(target_os = "linux")]
    bounds: std::rc::Rc<std::cell::Cell<Rect>>,
    #[cfg(target_os = "linux")]
    allocation_handler: Option<gtk::glib::SignalHandlerId>,
}

impl WebViewBackend for WryBackend {
    type View = WryView;

    fn create(
        &mut self,
        host: u64,
        state: &WebViewState,
        events: WebViewEventSink,
    ) -> Result<WryView, WebViewError> {
        let host = self.hosts.get(&host).ok_or(WebViewError::MissingHost)?;
        let options = state.options();
        options.validate_native()?;
        let policy = state.policy();
        let document = Arc::new(Mutex::new(Vec::<u8>::new()));
        let body = document.clone();
        let navigation_events = events.clone();
        let title_events = events.clone();
        let popup_events = events.clone();
        #[cfg(target_os = "linux")]
        let navigation_wake = self.input_waker.clone();
        #[cfg(target_os = "linux")]
        let popup_wake = self.input_waker.clone();
        let mut builder = wry::WebViewBuilder::new()
            .with_visible(false)
            .with_incognito(true)
            .with_autoplay(false)
            .with_navigation_handler(move |value| {
                let allowed = policy.allows_navigation(&value);
                if !allowed {
                    navigation_events.emit(WebViewEvent::NavigationRequested(value));
                    #[cfg(target_os = "linux")]
                    if let Some(wake) = &navigation_wake {
                        wake();
                    }
                }
                allowed
            })
            .with_new_window_req_handler(move |url, _| {
                popup_events.emit(WebViewEvent::NavigationRequested(url));
                #[cfg(target_os = "linux")]
                if let Some(wake) = &popup_wake {
                    wake();
                }
                wry::NewWindowResponse::Deny
            })
            .with_download_started_handler(move |url, _| {
                options.downloads_allowed() && WebViewSource::url(&url).is_ok()
            })
            .with_permission_handler(|_| wry::PermissionResponse::Deny)
            .with_document_title_changed_handler(move |title| {
                title_events.emit(WebViewEvent::Title(title))
            })
            .with_on_page_load_handler(move |kind, url| {
                events.emit(match kind {
                    wry::PageLoadEvent::Started => WebViewEvent::Loading(url),
                    wry::PageLoadEvent::Finished => WebViewEvent::Loaded(url),
                })
            });
        if policy == WebViewPolicy::RestrictedHtml {
            builder = builder.with_javascript_disabled().with_custom_protocol(
                "argui-content".into(),
                move |_, request| {
                    let allowed = policy.allows_navigation(&request.uri().to_string())
                        && request.method() == "GET";
                    let content = if allowed {
                        body.lock().expect("document mutex poisoned").clone()
                    } else {
                        Vec::new()
                    };
                    wry::http::Response::builder()
                        .status(if allowed { 200 } else { 404 })
                        .header("Content-Type", "text/html; charset=utf-8")
                        .header("Content-Security-Policy", DOCUMENT_CSP)
                        .header("Cache-Control", "no-store")
                        .header("X-Content-Type-Options", "nosniff")
                        .body(Cow::Owned(content))
                        .expect("static response headers")
                },
            );
        }
        #[cfg(target_os = "linux")]
        let view = {
            use gtk::prelude::*;
            use wry::{WebViewBuilderExtUnix, WebViewExtUnix};
            let view = builder.build_gtk(host).map_err(native_error)?;
            view.webview().set_size_request(1, 1);
            // Run after WebKit's default handler: preserve browser input, but do not
            // bubble child-local coordinates into Tao's toplevel resize/hit testing.
            for signal in [
                "motion-notify-event",
                "button-press-event",
                "button-release-event",
                "scroll-event",
                "enter-notify-event",
                "leave-notify-event",
            ] {
                view.webview()
                    .connect_local(signal, true, |_| Some(true.to_value()));
            }
            if let Some(wake) = self.input_waker.clone() {
                let pointer_wake = wake.clone();
                view.webview().connect_event_after(move |_, event| {
                    if matches!(
                        event.event_type(),
                        gtk::gdk::EventType::EnterNotify
                            | gtk::gdk::EventType::LeaveNotify
                            | gtk::gdk::EventType::ButtonPress
                            | gtk::gdk::EventType::ButtonRelease
                            | gtk::gdk::EventType::FocusChange
                    ) {
                        pointer_wake();
                    }
                });
                view.webview().connect_has_focus_notify(move |_| wake());
            }
            view
        };
        #[cfg(not(target_os = "linux"))]
        let view = builder.build_as_child(host).map_err(native_error)?;
        #[cfg(target_os = "linux")]
        let (bounds, allocation_handler) = {
            use gtk::prelude::*;
            use wry::WebViewExtUnix;
            let bounds = std::rc::Rc::new(std::cell::Cell::new(Rect::default()));
            let current = bounds.clone();
            let widget = view.webview().downgrade();
            let handler = host.connect_size_allocate(move |_, allocation| {
                if let Some(widget) = widget.upgrade() {
                    let bounds = current.get();
                    widget.size_allocate(&gtk::Allocation::new(
                        allocation.x() + bounds.origin.x as i32,
                        allocation.y() + bounds.origin.y as i32,
                        (bounds.size.width as i32).max(1),
                        (bounds.size.height as i32).max(1),
                    ));
                }
            });
            (bounds, Some(handler))
        };
        Ok(WryView {
            view,
            policy,
            document,
            document_revision: 0,
            #[cfg(target_os = "linux")]
            host: host.clone(),
            #[cfg(target_os = "linux")]
            bounds,
            #[cfg(target_os = "linux")]
            allocation_handler,
        })
    }
}

impl NativeWebView for WryView {
    fn load(&mut self, source: &WebViewSource) -> Result<(), WebViewError> {
        if source.policy() != self.policy {
            return Err(WebViewError::PolicyMismatch);
        }
        match source {
            WebViewSource::Url(url) => {
                WebViewSource::url(url.as_str())?;
                self.view.load_url(url.as_str()).map_err(native_error)
            }
            WebViewSource::Html(_) => {
                *self.document.lock().expect("document mutex poisoned") =
                    source.sanitized_html().expect("HTML source").into_bytes();
                self.document_revision += 1;
                self.view
                    .load_url(&format!(
                        "{DOCUMENT_URL}?revision={}",
                        self.document_revision
                    ))
                    .map_err(native_error)
            }
        }
    }

    fn set_bounds(&mut self, bounds: Rect) -> Result<(), WebViewError> {
        let position = bounds.origin;
        #[cfg(target_os = "linux")]
        let position = {
            use gtk::prelude::*;
            self.bounds.set(bounds);
            let allocation = self.host.allocation();
            argui_core::Point::new(
                position.x + allocation.x() as f32,
                position.y + allocation.y() as f32,
            )
        };
        self.view
            .set_bounds(wry::Rect {
                position: wry::dpi::LogicalPosition::new(
                    f64::from(position.x),
                    f64::from(position.y),
                )
                .into(),
                size: wry::dpi::LogicalSize::new(
                    f64::from(bounds.size.width),
                    f64::from(bounds.size.height),
                )
                .into(),
            })
            .map_err(native_error)
    }

    fn set_visible(&mut self, visible: bool) -> Result<(), WebViewError> {
        self.view.set_visible(visible).map_err(native_error)
    }
    fn focus(&mut self) -> Result<(), WebViewError> {
        self.view.focus().map_err(native_error)
    }
}

#[cfg(target_os = "linux")]
impl Drop for WryView {
    fn drop(&mut self) {
        use gtk::prelude::*;
        if let Some(handler) = self.allocation_handler.take() {
            self.host.disconnect(handler);
        }
    }
}

fn native_error(error: wry::Error) -> WebViewError {
    WebViewError::Native(error.to_string())
}
