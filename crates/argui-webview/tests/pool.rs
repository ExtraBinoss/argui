use argui_core::{Point, Rect, Size};
use argui_webview::{
    NativeWebView, PoolConfig, WebViewBackend, WebViewError, WebViewEvent, WebViewEventSink,
    WebViewMount, WebViewPool, WebViewSource, WebViewState,
};
use std::{cell::RefCell, rc::Rc, time::Duration};

#[derive(Default)]
struct Log {
    calls: Vec<String>,
    sinks: Vec<WebViewEventSink>,
    fail_create: bool,
    fail_load: bool,
}
struct Backend(Rc<RefCell<Log>>);
struct View(Rc<RefCell<Log>>);
impl WebViewBackend for Backend {
    type View = View;
    fn create(
        &mut self,
        host: u64,
        _: &WebViewState,
        events: WebViewEventSink,
    ) -> Result<View, WebViewError> {
        let mut log = self.0.borrow_mut();
        if log.fail_create {
            return Err(WebViewError::MissingHost);
        }
        log.calls.push(format!("create:{host}"));
        log.sinks.push(events);
        Ok(View(self.0.clone()))
    }
}
impl NativeWebView for View {
    fn load(&mut self, _: &WebViewSource) -> Result<(), WebViewError> {
        let mut log = self.0.borrow_mut();
        if log.fail_load {
            return Err(WebViewError::Native("load".into()));
        }
        log.calls.push("load".into());
        Ok(())
    }
    fn set_bounds(&mut self, _: Rect) -> Result<(), WebViewError> {
        self.0.borrow_mut().calls.push("bounds".into());
        Ok(())
    }
    fn set_visible(&mut self, visible: bool) -> Result<(), WebViewError> {
        self.0.borrow_mut().calls.push(format!("visible:{visible}"));
        Ok(())
    }
    fn focus(&mut self) -> Result<(), WebViewError> {
        self.0.borrow_mut().calls.push("focus".into());
        Ok(())
    }
}
impl Drop for View {
    fn drop(&mut self) {
        self.0.borrow_mut().calls.push("drop".into());
    }
}
fn mount() -> WebViewMount {
    WebViewMount {
        state: WebViewState::new(WebViewSource::html("hello")),
        host: 1,
        bounds: Rect::new(Point::default(), Size::new(400.0, 300.0)),
        occluded: false,
    }
}
fn setup(capacity: usize) -> (WebViewPool<Backend>, Rc<RefCell<Log>>) {
    let log = Rc::new(RefCell::new(Log::default()));
    (
        WebViewPool::new(
            Backend(log.clone()),
            PoolConfig {
                max_resident: capacity,
                ..PoolConfig::default()
            },
        )
        .unwrap(),
        log,
    )
}

#[test]
fn navigation_callback_is_replaceable_reentrant_and_ignores_stale_instances() {
    let (mut pool, log) = setup(1);
    let slot = mount();
    let seen = Rc::new(RefCell::new(Vec::new()));
    slot.state
        .on_navigation_requested(|_| panic!("replaced handler"));
    let recorded = seen.clone();
    let state = slot.state.clone();
    slot.state.on_navigation_requested(move |url| {
        recorded.borrow_mut().push(url.to_owned());
        assert!(!state.drain_events().is_empty());
        state.reload();
    });
    pool.reconcile(std::slice::from_ref(&slot), Duration::ZERO)
        .unwrap();
    let sink = log.borrow().sinks[0].clone();
    sink.emit(WebViewEvent::Loaded("document".into()));
    assert!(seen.borrow().is_empty());
    sink.emit(WebViewEvent::NavigationRequested(
        "https://example.com".into(),
    ));
    assert_eq!(*seen.borrow(), ["https://example.com"]);
    slot.state.release();
    sink.emit(WebViewEvent::NavigationRequested(
        "https://stale.example".into(),
    ));
    assert_eq!(seen.borrow().len(), 1);
    // Break the deliberate strong capture used to exercise reentrant access.
    slot.state.on_navigation_requested(|_| {});
}

#[test]
fn unchanged_frames_and_identical_sources_make_no_native_calls() {
    let (mut pool, log) = setup(2);
    let mut slot = mount();
    pool.reconcile(std::slice::from_ref(&slot), Duration::ZERO)
        .unwrap();
    assert_eq!(
        log.borrow().calls,
        ["create:1", "bounds", "load", "visible:true"]
    );
    let calls = log.borrow().calls.len();
    slot.state.load(WebViewSource::html("hello")).unwrap();
    for tick in 0..100 {
        pool.reconcile(std::slice::from_ref(&slot), Duration::from_millis(tick))
            .unwrap();
    }
    assert_eq!(log.borrow().calls.len(), calls);
    slot.state
        .load(WebViewSource::html("next message"))
        .unwrap();
    pool.reconcile(std::slice::from_ref(&slot), Duration::from_secs(1))
        .unwrap();
    assert_eq!(pool.stats().created, 1);
    assert_eq!(pool.stats().reused, 1);
    slot.state.focus();
    slot.bounds.size.width += 20.0;
    pool.reconcile(std::slice::from_ref(&slot), Duration::from_secs(2))
        .unwrap();
    assert!(
        log.borrow()
            .calls
            .ends_with(&["bounds".into(), "focus".into()])
    );
    slot.state.reload();
    pool.reconcile(&[slot], Duration::from_secs(3)).unwrap();
    assert_eq!(pool.stats().created, 1);
    assert_eq!(pool.stats().reused, 2);
}

#[test]
fn lru_evicts_only_inactive_tabs_and_invalidates_their_callbacks() {
    let (mut pool, log) = setup(2);
    let a = mount();
    let b = mount();
    let c = mount();
    pool.reconcile(std::slice::from_ref(&a), Duration::ZERO)
        .unwrap();
    let old = log.borrow().sinks[0].clone();
    pool.reconcile(std::slice::from_ref(&b), Duration::from_secs(1))
        .unwrap();
    pool.reconcile(&[b.clone(), c.clone()], Duration::from_secs(2))
        .unwrap();
    assert_eq!(pool.stats().resident, 2);
    assert_eq!(pool.stats().evicted, 1);
    assert_eq!(a.state.drain_events(), [WebViewEvent::Evicted]);
    old.emit(WebViewEvent::Title("stale".into()));
    assert!(a.state.drain_events().is_empty());
    let calls = log.borrow().calls.len();
    assert_eq!(
        pool.reconcile(&[a.clone(), b, c], Duration::from_secs(3)),
        Err(WebViewError::CapacityExceeded)
    );
    assert_eq!(log.borrow().calls.len(), calls);
    pool.reconcile(std::slice::from_ref(&a), Duration::from_secs(4))
        .unwrap();
    assert_eq!(pool.stats().created, 4);
    pool.close_host(1);
    assert_eq!(pool.stats().resident, 0);
}

#[test]
fn overlay_hiding_is_not_eviction_and_idle_tabs_expire_on_a_deadline() {
    let (mut pool, log) = setup(1);
    let mut slot = mount();
    pool.reconcile(std::slice::from_ref(&slot), Duration::ZERO)
        .unwrap();
    slot.occluded = true;
    pool.reconcile(std::slice::from_ref(&slot), Duration::from_secs(100))
        .unwrap();
    assert_eq!(pool.next_expiry(), None);
    assert_eq!(pool.stats().resident, 1);
    slot.state.focus();
    assert!(!log.borrow().calls.contains(&"focus".into()));
    slot.occluded = false;
    pool.reconcile(std::slice::from_ref(&slot), Duration::from_secs(101))
        .unwrap();
    assert_eq!(pool.stats().created, 1);
    pool.reconcile(&[], Duration::from_secs(102)).unwrap();
    assert_eq!(pool.next_expiry(), Some(Duration::from_secs(132)));
    pool.reconcile(std::slice::from_ref(&slot), Duration::from_secs(110))
        .unwrap();
    assert_eq!(pool.stats().reused, 1);
    pool.reconcile(&[], Duration::from_secs(111)).unwrap();
    pool.reconcile(&[], Duration::from_secs(140)).unwrap();
    assert_eq!(pool.stats().resident, 1);
    pool.reconcile(&[], Duration::from_secs(141)).unwrap();
    assert_eq!(pool.stats().resident, 0);
    assert_eq!(pool.next_expiry(), None);
}

#[test]
fn invalid_mounts_fail_before_mutation_and_zero_sized_slots_do_not_allocate() {
    let (mut pool, log) = setup(1);
    let mut slot = mount();
    assert_eq!(
        pool.reconcile(&[slot.clone(), slot.clone()], Duration::ZERO),
        Err(WebViewError::DuplicateMount)
    );
    for width in [f32::NAN, f32::INFINITY, -1.0] {
        slot.bounds.size.width = width;
        assert_eq!(
            pool.reconcile(std::slice::from_ref(&slot), Duration::ZERO),
            Err(WebViewError::InvalidBounds)
        );
    }
    slot.bounds.size.width = 0.0;
    pool.reconcile(&[slot], Duration::ZERO).unwrap();
    assert!(log.borrow().calls.is_empty());
    assert!(
        WebViewPool::new(
            Backend(log),
            PoolConfig {
                max_resident: 0,
                ..PoolConfig::default()
            }
        )
        .is_err()
    );
}

#[test]
fn release_reopen_move_and_backend_failures_do_not_leak_instances() {
    let (mut pool, log) = setup(1);
    let mut slot = mount();
    log.borrow_mut().fail_create = true;
    assert_eq!(
        pool.reconcile(std::slice::from_ref(&slot), Duration::ZERO),
        Err(WebViewError::MissingHost)
    );
    log.borrow_mut().fail_create = false;
    log.borrow_mut().fail_load = true;
    assert!(
        pool.reconcile(std::slice::from_ref(&slot), Duration::ZERO)
            .is_err()
    );
    assert_eq!(pool.stats().resident, 0);
    assert_eq!(log.borrow().calls.last().unwrap(), "drop");
    log.borrow_mut().fail_load = false;
    pool.reconcile(std::slice::from_ref(&slot), Duration::ZERO)
        .unwrap();
    slot.state.release();
    pool.reconcile(std::slice::from_ref(&slot), Duration::ZERO)
        .unwrap();
    assert_eq!(pool.stats().resident, 0);
    slot.state.load(WebViewSource::html("hello")).unwrap();
    pool.reconcile(std::slice::from_ref(&slot), Duration::ZERO)
        .unwrap();
    slot.host = 2;
    pool.reconcile(std::slice::from_ref(&slot), Duration::ZERO)
        .unwrap();
    assert_eq!(pool.stats().resident, 1);
    pool.close_host(1);
    assert_eq!(pool.stats().resident, 1);
    assert_eq!(
        slot.state
            .load(WebViewSource::url("https://example.com").unwrap()),
        Err(WebViewError::PolicyMismatch)
    );
}

#[test]
fn event_buffer_is_bounded_and_callbacks_do_not_keep_sessions_alive() {
    let (mut pool, log) = setup(1);
    let slot = mount();
    pool.reconcile(std::slice::from_ref(&slot), Duration::ZERO)
        .unwrap();
    let sink = log.borrow().sinks[0].clone();
    for index in 0..200 {
        sink.emit(WebViewEvent::Title(index.to_string()));
    }
    let events = slot.state.drain_events();
    assert_eq!(events.len(), 128);
    assert_eq!(events[0], WebViewEvent::Title("72".into()));
    drop(pool);
    assert_eq!(slot.state.drain_events(), [WebViewEvent::Evicted]);
    sink.emit(WebViewEvent::Title("late".into()));
    assert!(slot.state.drain_events().is_empty());
    drop(slot);
    sink.emit(WebViewEvent::Title("gone".into()));
}

#[test]
fn native_pointer_ownership_tracks_visibility_and_current_bounds() {
    let (mut pool, _) = setup(1);
    let mut slot = mount();
    let inside = Point::new(20.0, 20.0);
    assert!(!pool.contains_pointer(inside));
    pool.reconcile(std::slice::from_ref(&slot), Duration::ZERO)
        .unwrap();
    assert!(pool.contains_pointer(inside));
    assert!(!pool.contains_pointer(Point::new(-1.0, 20.0)));
    assert!(!pool.contains_pointer(Point::new(f32::NAN, 20.0)));
    slot.occluded = true;
    pool.reconcile(std::slice::from_ref(&slot), Duration::ZERO)
        .unwrap();
    assert!(!pool.contains_pointer(inside));
    slot.occluded = false;
    slot.bounds.origin.x = 100.0;
    pool.reconcile(std::slice::from_ref(&slot), Duration::ZERO)
        .unwrap();
    assert!(!pool.contains_pointer(inside));
    assert!(pool.contains_pointer(Point::new(120.0, 20.0)));
    pool.close_host(1);
    assert!(!pool.contains_pointer(Point::new(120.0, 20.0)));
}

#[test]
fn release_then_reload_before_reconciliation_recreates_a_live_instance() {
    let (mut pool, log) = setup(1);
    let slot = mount();
    pool.reconcile(std::slice::from_ref(&slot), Duration::ZERO)
        .unwrap();
    let stale = log.borrow().sinks[0].clone();
    slot.state.release();
    slot.state.reload();
    pool.reconcile(std::slice::from_ref(&slot), Duration::ZERO)
        .unwrap();
    assert_eq!(pool.stats().created, 2);
    assert_eq!(pool.stats().evicted, 1);
    slot.state.drain_events();
    stale.emit(WebViewEvent::Title("old".into()));
    log.borrow().sinks[1].emit(WebViewEvent::Title("new".into()));
    assert_eq!(
        slot.state.drain_events(),
        [WebViewEvent::Title("new".into())]
    );
}

#[test]
fn failed_creation_invalidates_callbacks_before_retry() {
    let (mut pool, log) = setup(1);
    let slot = mount();
    log.borrow_mut().fail_load = true;
    assert!(
        pool.reconcile(std::slice::from_ref(&slot), Duration::ZERO)
            .is_err()
    );
    assert_eq!(
        slot.state.drain_events(),
        [WebViewEvent::Error(WebViewError::Native("load".into()))]
    );
    log.borrow().sinks[0].emit(WebViewEvent::Title("failed view".into()));
    assert!(slot.state.drain_events().is_empty());
    log.borrow_mut().fail_load = false;
    pool.reconcile(std::slice::from_ref(&slot), Duration::ZERO)
        .unwrap();
    log.borrow().sinks[1].emit(WebViewEvent::Title("retry".into()));
    assert_eq!(
        slot.state.drain_events(),
        [WebViewEvent::Title("retry".into())]
    );
}
