use crate::app::text;
use argui::{
    runtime::{Context, Render},
    ui::{Element, EventType, UiEvent, UiEventKind, length, percent},
    widgets::{Button, Tab, Tabs, TabsAction, TabsBehavior, WidgetTheme, shadcn},
};

pub(crate) struct WebViewDemo {
    selected: usize,
    message: usize,
    #[cfg(feature = "webview")]
    email: argui::webview::WebViewState,
    #[cfg(feature = "webview")]
    webpage: argui::webview::WebViewState,
    #[cfg(all(feature = "webview", target_arch = "wasm32"))]
    compatible: Option<argui::webview::WebViewState>,
    #[cfg(all(feature = "webview", target_arch = "wasm32"))]
    use_compatible: bool,
}

impl Default for WebViewDemo {
    fn default() -> Self {
        Self {
            selected: 0,
            message: 1,
            #[cfg(all(feature = "webview", target_arch = "wasm32"))]
            compatible: option_env!("ARGUI_WEBVIEW_RELAY_URL").map(|relay| {
                use argui::webview::{
                    WebCompatibility, WebViewOptions, WebViewSource, WebViewState,
                };
                WebViewState::with_options(
                    WebViewSource::url("https://example.com").expect("static HTTPS URL"),
                    WebViewOptions::webpage().compatibility(
                        WebCompatibility::compatible(relay).expect("valid ARGUI_WEBVIEW_RELAY_URL"),
                    ),
                )
                .expect("valid webpage policy")
            }),
            #[cfg(all(feature = "webview", target_arch = "wasm32"))]
            use_compatible: false,
            #[cfg(feature = "webview")]
            email: argui::webview::WebViewState::new(argui::webview::WebViewSource::html(
                email_html(1),
            )),
            #[cfg(feature = "webview")]
            webpage: argui::webview::WebViewState::new(
                argui::webview::WebViewSource::url(if cfg!(target_arch = "wasm32") {
                    "https://example.com"
                } else {
                    "https://www.google.com"
                })
                .expect("static HTTPS URL"),
            ),
        }
    }
}

impl WebViewDemo {
    #[cfg(feature = "webview")]
    fn current_webpage(&self) -> &argui::webview::WebViewState {
        #[cfg(target_arch = "wasm32")]
        if self.use_compatible
            && let Some(state) = &self.compatible
        {
            return state;
        }
        &self.webpage
    }

    pub(crate) fn entity() -> argui::runtime::Entity<Self> {
        let entity = argui::runtime::Entity::new(Self::default());
        #[cfg(feature = "webview")]
        entity.read(|demo| {
            for state in [&demo.email, &demo.webpage] {
                let weak = entity.downgrade();
                state.on_navigation_requested(move |url| {
                    let Ok(source) = argui::webview::WebViewSource::url(url) else {
                        return;
                    };
                    if let Some(entity) = weak.upgrade() {
                        entity.update(|demo, cx| {
                            if demo.current_webpage().load(source).is_ok() {
                                demo.selected = 1;
                                cx.notify();
                            }
                        });
                    }
                });
            }
        });
        entity
    }

    fn handle(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        let tabs = TabsBehavior::new(
            "webview-tabs",
            [("Email".into(), true), ("Webpage".into(), true)],
            self.selected,
        );
        if let Some(TabsAction::Select(selected)) = tabs.action(event) {
            self.selected = selected;
            cx.notify();
        }
        if !matches!(event.kind, UiEventKind::Click(_)) {
            return;
        }
        match event.target_key() {
            #[cfg(all(feature = "webview", target_arch = "wasm32"))]
            Some("webview-compatibility") => {
                if self.compatible.is_some() {
                    self.use_compatible = !self.use_compatible;
                    cx.notify();
                }
            }
            Some("webview-next-email") => {
                self.message += 1;
                #[cfg(feature = "webview")]
                self.email
                    .load(argui::webview::WebViewSource::html(email_html(
                        self.message,
                    )))
                    .expect("unchanged HTML policy");
                cx.notify();
            }
            Some("webview-reload") => {
                #[cfg(feature = "webview")]
                if self.selected == 0 {
                    self.email.reload();
                } else {
                    self.current_webpage().reload();
                }
                cx.notify();
            }
            _ => (),
        }
    }

    fn panel(&self, email: bool, theme: &WidgetTheme) -> Element {
        let mut actions =
            vec![Button::new("webview-reload", "Reload", theme.ghost_button()).build()];
        if email {
            actions.push(
                Button::new("webview-next-email", "Next email", theme.ghost_button()).build(),
            );
        }
        #[cfg(all(feature = "webview", target_arch = "wasm32"))]
        if !email && self.compatible.is_some() {
            actions.push(
                Button::new(
                    "webview-compatibility",
                    if self.use_compatible {
                        "Mode: compatible"
                    } else {
                        "Mode: isolated"
                    },
                    theme.ghost_button(),
                )
                .build(),
            );
        }
        Element::column([
            Element::row(actions).gap(8.0),
            text(
                if email {
                    format!(
                        "Message {} · scripts and remote resources blocked",
                        self.message
                    )
                } else {
                    if cfg!(target_arch = "wasm32") { "Webpage · some sites block embedding; browser storage is not a private profile" } else { "Webpage · HTTP/HTTPS navigation · no application IPC" }.into()
                },
                14.0,
                theme.muted_foreground,
                400,
            ),
            self.surface(email, theme),
        ])
        .gap(12.0)
        .width(percent(1.0))
    }

    fn surface(&self, email: bool, theme: &WidgetTheme) -> Element {
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
            let _ = theme;
            argui::webview::WebView::new(if email {
                &self.email
            } else {
                self.current_webpage()
            })
            .build()
            .width(percent(1.0))
            .height(length(380.0))
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
        {
            let _ = email;
            text(if cfg!(target_arch = "wasm32") { "Enable WebView support by rebuilding with --features webview, or use scripts/serve-widget-gallery.sh." } else { "Enable the native example: cargo run -p argui-widget-gallery --features webview" }, 15.0, theme.muted_foreground, 400)
                .width(percent(1.0)).min_height(length(120.0))
        }
    }
}

impl Render for WebViewDemo {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let environment = cx.environment();
        let themes = shadcn(environment.primary);
        let theme = themes.resolve(environment.color_scheme);
        let mut root = Tabs::new(
            "webview-tabs",
            [
                Tab::new("Email", self.panel(true, theme)),
                Tab::new("Webpage", self.panel(false, theme)),
            ],
            self.selected,
        )
        .build(theme);
        for event in [EventType::Click, EventType::Key] {
            root = root.on(cx.listener(event, Self::handle));
        }
        root
    }
}

#[cfg(feature = "webview")]
fn email_html(number: usize) -> String {
    let engine = if cfg!(target_arch = "wasm32") {
        "an isolated browser iframe"
    } else {
        "a native WebView"
    };
    format!(
        "<article><p><strong>ARGUI MAIL</strong> · Message {number}</p><hr><h1>Your inbox is ready</h1><p><strong>From:</strong> Ada &lt;ada@example.com&gt;</p><p>Hello! This email is rendered in {engine}, inside the Argui layout.</p><blockquote>One retained session can display many messages without creating a new browser instance each time.</blockquote><p>The Email and Webpage tabs keep separate security policies. Scripts, frames, forms and remote images are disabled for this message.</p><p><a href='https://example.com'>Open example.com in the Webpage tab</a></p></article>"
    )
}
