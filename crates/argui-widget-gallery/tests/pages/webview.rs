use argui::{
    runtime::Entity,
    ui::{ClickEvent, Element, UiEventKind, UiTree},
};
use argui_widget_gallery::WidgetGallery;

fn click(gallery: &Entity<WidgetGallery>, key: &str) {
    let mut tree = UiTree::new(gallery.render());
    let node = tree
        .node_ids()
        .iter()
        .copied()
        .find(|id| tree.key(*id) == Some(key))
        .expect("control is mounted");
    for event in tree.event_deliveries(node, UiEventKind::Click(ClickEvent::accessibility())) {
        if event.should_dispatch() {
            gallery.dispatch_event(&event);
        }
    }
}

fn native(element: &Element) -> Option<&argui::ui::NativeContent> {
    element
        .native_content
        .as_ref()
        .or_else(|| element.children.iter().find_map(native))
}

#[test]
fn webview_page_is_navigable_with_real_email_and_webpage_tabs() {
    let gallery = Entity::new(WidgetGallery::default());
    click(&gallery, "nav::webview");
    let root = gallery.render();
    let tree = UiTree::new(root.clone());
    let tabs: Vec<_> = tree
        .node_ids()
        .iter()
        .filter_map(|id| tree.key(*id))
        .filter(|key| key.starts_with("webview-tabs"))
        .collect();
    assert!(tabs.iter().any(|key| key.contains("tab::0")));
    assert!(tabs.iter().any(|key| key.contains("tab::1")));
    #[cfg(all(
        feature = "webview",
        any(
            target_arch = "wasm32",
            target_os = "linux",
            target_os = "windows",
            target_os = "macos"
        )
    ))]
    {
        let email_id = native(&root).unwrap().id();
        click(&gallery, "webview-next-email");
        assert_eq!(native(&gallery.render()).unwrap().id(), email_id);
        click(&gallery, "webview-tabs::tab::1");
        assert_ne!(native(&gallery.render()).unwrap().id(), email_id);
        click(&gallery, "webview-tabs::tab::0");
        assert_eq!(native(&gallery.render()).unwrap().id(), email_id);
        click(&gallery, "nav::button");
        assert!(native(&gallery.render()).is_none());
        click(&gallery, "nav::webview");
        assert_eq!(native(&gallery.render()).unwrap().id(), email_id);
    }
    #[cfg(not(all(
        feature = "webview",
        any(
            target_arch = "wasm32",
            target_os = "linux",
            target_os = "windows",
            target_os = "macos"
        )
    )))]
    assert!(native(&root).is_none());
}

#[cfg(all(
    feature = "webview",
    any(
        target_arch = "wasm32",
        target_os = "linux",
        target_os = "windows",
        target_os = "macos"
    )
))]
#[test]
fn email_links_open_in_browser_session_without_relaxing_email_policy() {
    use argui::{
        core::{Point, Rect, Size},
        webview::*,
    };
    use std::{cell::RefCell, rc::Rc, time::Duration};
    struct Backend(Rc<RefCell<Option<WebViewEventSink>>>);
    struct View;
    impl WebViewBackend for Backend {
        type View = View;
        fn create(
            &mut self,
            _: u64,
            _: &WebViewState,
            events: WebViewEventSink,
        ) -> Result<View, WebViewError> {
            *self.0.borrow_mut() = Some(events);
            Ok(View)
        }
    }
    impl NativeWebView for View {
        fn load(&mut self, _: &WebViewSource) -> Result<(), WebViewError> {
            Ok(())
        }
        fn set_bounds(&mut self, _: Rect) -> Result<(), WebViewError> {
            Ok(())
        }
        fn set_visible(&mut self, _: bool) -> Result<(), WebViewError> {
            Ok(())
        }
        fn focus(&mut self) -> Result<(), WebViewError> {
            Ok(())
        }
    }
    let gallery = Entity::new(WidgetGallery::default());
    click(&gallery, "nav::webview");
    let email = native(&gallery.render())
        .unwrap()
        .downcast_ref::<WebViewState>()
        .unwrap()
        .clone();
    let captured = Rc::new(RefCell::new(None));
    let mut pool = WebViewPool::new(Backend(captured.clone()), PoolConfig::default()).unwrap();
    pool.reconcile(
        &[WebViewMount {
            state: email.clone(),
            host: 1,
            bounds: Rect::new(Point::default(), Size::new(100.0, 100.0)),
            occluded: false,
        }],
        Duration::ZERO,
    )
    .unwrap();
    let sink = captured.borrow().clone().unwrap();
    for url in [
        "javascript:alert(1)",
        "file:///etc/passwd",
        "mailto:a@example.com",
        "not a url",
    ] {
        sink.emit(WebViewEvent::NavigationRequested(url.into()));
        assert_eq!(native(&gallery.render()).unwrap().id(), email.id().0);
    }
    sink.emit(WebViewEvent::NavigationRequested(
        "https://example.com/path".into(),
    ));
    let root = gallery.render();
    let webpage = native(&root)
        .unwrap()
        .downcast_ref::<WebViewState>()
        .unwrap();
    assert_ne!(webpage.id(), email.id());
    assert_eq!(
        webpage.source(),
        WebViewSource::url("https://example.com/path").unwrap()
    );
    assert_eq!(email.policy(), WebViewPolicy::RestrictedHtml);
    click(&gallery, "webview-tabs::tab::0");
    assert_eq!(native(&gallery.render()).unwrap().id(), email.id().0);
    drop(gallery);
    sink.emit(WebViewEvent::NavigationRequested(
        "https://example.com/after-close".into(),
    ));
}
