use std::{
    cell::RefCell,
    collections::VecDeque,
    rc::{Rc, Weak},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{WebViewError, WebViewOptions, WebViewPolicy, WebViewSource};

type NavigationHandler = Rc<dyn Fn(&str)>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Stable identifier assigned to a retained WebView session.
pub struct WebViewId(
    /// Monotonically allocated session identifier.
    pub u64,
);

#[derive(Clone, Debug, PartialEq)]
/// Lifecycle and navigation event queued by a WebView session.
pub enum WebViewEvent {
    /// A source began loading.
    Loading(String),
    /// A source finished loading.
    Loaded(String),
    /// The document title changed.
    Title(String),
    /// Navigation was blocked and awaits application handling.
    NavigationRequested(String),
    /// The backend reported an error.
    Error(WebViewError),
    /// The pool evicted the session's native view.
    Evicted,
}

#[derive(Clone, Debug)]
/// Cloneable retained session state for a native or browser WebView.
pub struct WebViewState(Rc<RefCell<Session>>);

struct Session {
    id: WebViewId,
    source: WebViewSource,
    options: WebViewOptions,
    revision: u64,
    focus: u64,
    released: bool,
    release_epoch: u64,
    generation: u64,
    events: VecDeque<WebViewEvent>,
    navigation_handler: Option<NavigationHandler>,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("id", &self.id)
            .field("source", &self.source)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
/// Weak, generation-scoped event sender for a WebView session.
pub struct WebViewEventSink {
    session: Weak<RefCell<Session>>,
    generation: u64,
}

impl WebViewState {
    /// Creates a retained session using the restrictive defaults for its source type.
    /// `source` is either an external URL or restricted HTML.
    ///
    /// # Panics
    /// Panics only if the built-in defaults no longer match the source's policy.
    #[must_use]
    pub fn new(source: WebViewSource) -> Self {
        let options = match source.policy() {
            WebViewPolicy::Browser => WebViewOptions::webpage(),
            WebViewPolicy::RestrictedHtml => WebViewOptions::email(),
        };
        Self::with_options(source, options).expect("default policy matches its source")
    }

    /// Creates a retained session with explicit security and browser options.
    /// `source` supplies content; `options` sets session permissions.
    ///
    /// # Errors
    /// Returns an error if the options do not satisfy the source's policy.
    pub fn with_options(
        source: WebViewSource,
        options: WebViewOptions,
    ) -> Result<Self, WebViewError> {
        options.validate(&source)?;
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Ok(Self(Rc::new(RefCell::new(Session {
            id: WebViewId(NEXT.fetch_add(1, Ordering::Relaxed)),
            source,
            options,
            revision: 0,
            focus: 0,
            released: false,
            release_epoch: 0,
            generation: 0,
            events: VecDeque::new(),
            navigation_handler: None,
        }))))
    }

    /// Returns a copy of the session options.
    pub fn options(&self) -> WebViewOptions {
        self.0.borrow().options.clone()
    }

    #[must_use]
    /// Returns the stable identifier assigned to this session.
    pub fn id(&self) -> WebViewId {
        self.0.borrow().id
    }

    #[must_use]
    /// Returns the session's current source.
    pub fn source(&self) -> WebViewSource {
        self.0.borrow().source.clone()
    }

    #[must_use]
    /// Returns the security policy associated with the current source.
    pub fn policy(&self) -> WebViewPolicy {
        self.0.borrow().source.policy()
    }

    /// Replaces the source without changing the session's security policy.
    /// `source` must use the same security policy as the current source.
    ///
    /// # Errors
    /// Returns an error if the new source requires a different policy.
    pub fn load(&self, source: WebViewSource) -> Result<(), WebViewError> {
        let mut session = self.0.borrow_mut();
        if session.source.policy() != source.policy() {
            return Err(WebViewError::PolicyMismatch);
        }
        if session.source != source || session.released {
            session.source = source;
            session.revision += 1;
            session.released = false;
        }
        Ok(())
    }

    /// Requests that the backend reload the current source.
    pub fn reload(&self) {
        let mut session = self.0.borrow_mut();
        session.revision += 1;
        session.released = false;
    }
    /// Requests keyboard focus for the native or browser view.
    pub fn focus(&self) {
        self.0.borrow_mut().focus += 1;
    }
    /// Marks the session released so its backend view can be reclaimed.
    pub fn release(&self) {
        let mut session = self.0.borrow_mut();
        session.released = true;
        session.release_epoch += 1;
        session.generation += 1;
    }

    /// Removes and returns all events currently queued for the session.
    pub fn drain_events(&self) -> Vec<WebViewEvent> {
        self.0.borrow_mut().events.drain(..).collect()
    }

    /// Replaces the application's handler for blocked navigation and popup requests.
    /// The handler must validate the URL before acting; requests remain in the event queue.
    /// Capture application owners weakly to avoid retaining a session through its callback.
    /// `handler` receives a requested navigation URL.
    pub fn on_navigation_requested(&self, handler: impl Fn(&str) + 'static) {
        self.0.borrow_mut().navigation_handler = Some(Rc::new(handler));
    }

    pub(crate) fn revision(&self) -> u64 {
        self.0.borrow().revision
    }
    pub(crate) fn focus_revision(&self) -> u64 {
        self.0.borrow().focus
    }
    pub(crate) fn released(&self) -> bool {
        self.0.borrow().released
    }
    pub(crate) fn release_epoch(&self) -> u64 {
        self.0.borrow().release_epoch
    }

    pub(crate) fn fail(&self, error: WebViewError) {
        self.new_sink().emit(WebViewEvent::Error(error));
    }

    pub(crate) fn new_sink(&self) -> WebViewEventSink {
        let mut session = self.0.borrow_mut();
        session.generation += 1;
        WebViewEventSink {
            session: Rc::downgrade(&self.0),
            generation: session.generation,
        }
    }

    pub(crate) fn evict(&self) {
        self.new_sink().emit(WebViewEvent::Evicted);
    }
}

impl WebViewEventSink {
    /// Queues an event if this sink still belongs to the active session generation.
    pub fn emit(&self, event: WebViewEvent) {
        let Some(session) = self.session.upgrade() else {
            return;
        };
        let mut session = session.borrow_mut();
        if session.generation != self.generation || session.released {
            return;
        }
        if session.events.len() == 128 {
            session.events.pop_front();
        }
        let navigation = match &event {
            WebViewEvent::NavigationRequested(url) => session
                .navigation_handler
                .clone()
                .map(|handler| (handler, url.clone())),
            _ => None,
        };
        session.events.push_back(event);
        drop(session);
        if let Some((handler, url)) = navigation {
            handler(&url);
        }
    }
}
