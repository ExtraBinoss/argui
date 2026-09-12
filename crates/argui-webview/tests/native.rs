#![cfg(target_os = "linux")]

use std::{
    cell::Cell,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    rc::Rc,
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread,
    time::Duration,
};

use argui_core::{Point, Rect, Size};
use argui_webview::{
    NativeWebView, PoolConfig, PopupPolicy, WebViewBackend, WebViewError, WebViewEvent,
    WebViewEventSink, WebViewMount, WebViewOptions, WebViewPolicy, WebViewPool, WebViewSource,
    WebViewState, WryBackend, WryView,
};
use gtk::prelude::*;
use web_time::Instant;

struct LocalPage {
    url: String,
    stop: Sender<()>,
    worker: Option<thread::JoinHandle<()>>,
}

impl LocalPage {
    fn start(body: &'static str) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind loopback test server");
        let address = listener.local_addr().expect("read test server address");
        listener
            .set_nonblocking(true)
            .expect("make test server nonblocking");
        let (stop, stopped) = mpsc::channel();
        let body = body.as_bytes();
        let worker = thread::spawn(move || serve(listener, stopped, body));
        Self {
            url: format!("http://{address}/"),
            stop,
            worker: Some(worker),
        }
    }
}

impl Drop for LocalPage {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn serve(listener: TcpListener, stopped: Receiver<()>, body: &[u8]) {
    loop {
        match stopped.try_recv() {
            Ok(()) | Err(TryRecvError::Disconnected) => return,
            Err(TryRecvError::Empty) => {}
        }
        match listener.accept() {
            Ok((mut stream, _)) => respond(&mut stream, body),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(_) => return,
        }
    }
}

fn respond(stream: &mut TcpStream, body: &[u8]) {
    let _ = stream.set_read_timeout(Some(Duration::from_millis(250)));
    let mut request = [0_u8; 2048];
    let _ = stream.read(&mut request);
    let headers = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(headers.as_bytes());
    let _ = stream.write_all(body);
}

struct ProbingBackend(WryBackend);

impl WebViewBackend for ProbingBackend {
    type View = WryView;

    fn create(
        &mut self,
        host: u64,
        state: &WebViewState,
        events: WebViewEventSink,
    ) -> Result<Self::View, WebViewError> {
        let mut view = self.0.create(host, state, events)?;
        match state.policy() {
            WebViewPolicy::RestrictedHtml => {
                let browser_url = WebViewSource::url("http://127.0.0.1/policy-mismatch")?;
                assert_eq!(view.load(&browser_url), Err(WebViewError::PolicyMismatch));
            }
            WebViewPolicy::Browser => {
                let file_url = WebViewSource::Url(url::Url::parse("file:///private").unwrap());
                assert_eq!(view.load(&file_url), Err(WebViewError::InvalidUrl));
                assert_eq!(
                    view.load(&WebViewSource::html("wrong policy")),
                    Err(WebViewError::PolicyMismatch)
                );
            }
        }
        Ok(view)
    }
}

fn private_display_enabled() -> bool {
    if std::env::var_os("ARGUI_NATIVE_TESTS").is_none() {
        return false;
    }
    assert_eq!(std::env::var("ARGUI_HIDDEN_DISPLAY").as_deref(), Ok("1"));
    let runtime = std::env::var("XDG_RUNTIME_DIR").expect("private runtime directory");
    assert!(
        std::path::Path::new(&runtime)
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("argui-display.")),
        "native webview tests require the private display helper"
    );
    true
}

fn mount(state: &WebViewState, host: u64, bounds: Rect, occluded: bool) -> WebViewMount {
    WebViewMount {
        state: state.clone(),
        host,
        bounds,
        occluded,
    }
}

fn wait_for_events(
    states: &[&WebViewState],
    mut ready: impl FnMut(&[Vec<WebViewEvent>]) -> bool,
) -> Vec<Vec<WebViewEvent>> {
    let context = gtk::glib::MainContext::default();
    let deadline = Instant::now() + Duration::from_secs(8);
    let mut collected = vec![Vec::new(); states.len()];
    loop {
        for _ in 0..64 {
            if !context.pending() {
                break;
            }
            context.iteration(false);
        }
        for (state, events) in states.iter().zip(&mut collected) {
            events.extend(state.drain_events());
        }
        if ready(&collected) {
            return collected;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for native webview behavior: {collected:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn wry_backend_enforces_policy_and_runs_the_native_lifecycle() {
    if !private_display_enabled() {
        return;
    }
    gtk::init().expect("initialize GTK on the private display");

    let page = LocalPage::start(
        "<!doctype html><html><head><title>Argui local page</title><script>window.addEventListener('load', () => setTimeout(() => { window.location.href = 'data:text/html,blocked'; }, 1000));</script></head><body>local content</body></html>",
    );
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_default_size(1000, 500);
    let host = gtk::Fixed::new();
    window.add(&host);
    window.show_all();

    let missing_state = WebViewState::new(WebViewSource::html("missing host"));
    let mut missing_pool = WebViewPool::new(WryBackend::new(), PoolConfig::default()).unwrap();
    assert_eq!(
        missing_pool.reconcile(
            &[mount(
                &missing_state,
                404,
                Rect::new(Point::default(), Size::new(100.0, 80.0)),
                false,
            )],
            Duration::ZERO,
        ),
        Err(WebViewError::MissingHost)
    );
    assert!(
        missing_state
            .drain_events()
            .contains(&WebViewEvent::Error(WebViewError::MissingHost))
    );

    let unsupported_state = WebViewState::with_options(
        WebViewSource::url(&page.url).unwrap(),
        WebViewOptions::webpage().popups(PopupPolicy::Sandboxed),
    )
    .unwrap();
    let mut unsupported_backend = WryBackend::new();
    unsupported_backend.register_gtk_host(9, host.clone());
    let mut unsupported_pool =
        WebViewPool::new(unsupported_backend, PoolConfig::default()).unwrap();
    let unsupported =
        WebViewError::UnsupportedOptions("native popup policy inheritance is unavailable".into());
    assert_eq!(
        unsupported_pool.reconcile(
            &[mount(
                &unsupported_state,
                9,
                Rect::new(Point::default(), Size::new(100.0, 80.0)),
                false,
            )],
            Duration::ZERO,
        ),
        Err(unsupported.clone())
    );
    assert!(
        unsupported_state
            .drain_events()
            .contains(&WebViewEvent::Error(unsupported))
    );

    let wake_count = Rc::new(Cell::new(0));
    let wake_observed = wake_count.clone();
    let mut backend = WryBackend::new().input_waker(move || {
        wake_observed.set(wake_observed.get() + 1);
    });
    backend.register_gtk_host(9, host.clone());
    let mut pool = WebViewPool::new(ProbingBackend(backend), PoolConfig::default()).unwrap();
    let email_state = WebViewState::new(WebViewSource::html(
        "<!doctype html><html><body><p>restricted content</p></body></html>",
    ));
    let browser_state = WebViewState::new(WebViewSource::url(&page.url).unwrap());
    let requests = Rc::new(std::cell::RefCell::new(Vec::<String>::new()));
    let observed_requests = requests.clone();
    browser_state.on_navigation_requested(move |url| {
        observed_requests.borrow_mut().push(url.to_owned());
    });
    let email_mount = mount(
        &email_state,
        9,
        Rect::new(Point::new(10.0, 10.0), Size::new(400.0, 180.0)),
        false,
    );
    let browser_mount = mount(
        &browser_state,
        9,
        Rect::new(Point::new(500.0, 10.0), Size::new(400.0, 180.0)),
        false,
    );
    pool.reconcile(
        &[email_mount.clone(), browser_mount.clone()],
        Duration::ZERO,
    )
    .unwrap();

    let loaded = wait_for_events(&[&email_state, &browser_state], |events| {
        let email_loaded = events[0].iter().any(|event| {
            matches!(event, WebViewEvent::Loaded(url) if url.contains("/document?revision=1"))
        });
        let browser_loaded = events[1]
            .iter()
            .any(|event| matches!(event, WebViewEvent::Loaded(url) if url == &page.url));
        let browser_title = events[1].iter().any(
            |event| matches!(event, WebViewEvent::Title(title) if title == "Argui local page"),
        );
        let blocked_navigation = events[1].iter().any(|event| {
            matches!(event, WebViewEvent::NavigationRequested(url) if url.starts_with("data:"))
        });
        email_loaded && browser_loaded && browser_title && blocked_navigation
    });
    assert!(loaded[0].iter().any(
        |event| matches!(event, WebViewEvent::Loading(url) if url.contains("/document?revision=1"))
    ));
    assert!(requests.borrow().iter().any(|url| url.starts_with("data:")));
    assert!(wake_count.get() > 0);
    assert!(pool.contains_pointer(Point::new(20.0, 20.0)));
    assert!(pool.contains_pointer(Point::new(510.0, 20.0)));

    email_state
        .load(WebViewSource::html("<!doctype html><p>updated content</p>"))
        .unwrap();
    email_state.focus();
    let moved_email = mount(
        &email_state,
        9,
        Rect::new(Point::new(20.0, 250.0), Size::new(400.0, 180.0)),
        false,
    );
    let hidden_browser = mount(&browser_state, 9, browser_mount.bounds, true);
    pool.reconcile(
        &[moved_email.clone(), hidden_browser.clone()],
        Duration::from_millis(1),
    )
    .unwrap();
    assert!(!pool.contains_pointer(Point::new(20.0, 20.0)));
    assert!(pool.contains_pointer(Point::new(30.0, 260.0)));
    assert!(!pool.contains_pointer(Point::new(510.0, 20.0)));
    let updated = wait_for_events(&[&email_state], |events| {
        events[0].iter().any(
            |event| matches!(event, WebViewEvent::Loaded(url) if url.contains("/document?revision=2")),
        )
    });
    assert!(updated[0].iter().any(
        |event| matches!(event, WebViewEvent::Loading(url) if url.contains("/document?revision=2"))
    ));

    pool.reconcile(&[moved_email, browser_mount], Duration::from_millis(2))
        .unwrap();
    assert!(pool.contains_pointer(Point::new(510.0, 20.0)));
    pool.close_host(9);
    assert_eq!(pool.stats().resident, 0);
    assert!(email_state.drain_events().contains(&WebViewEvent::Evicted));
    assert!(
        browser_state
            .drain_events()
            .contains(&WebViewEvent::Evicted)
    );

    drop(pool);
    drop(unsupported_pool);
    drop(missing_pool);
    window.close();
    let context = gtk::glib::MainContext::default();
    for _ in 0..64 {
        if !context.pending() {
            break;
        }
        context.iteration(false);
    }
    drop(host);
    drop(window);
    drop(page);
}
