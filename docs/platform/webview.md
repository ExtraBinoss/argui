# WebView integration

`argui-webview` provides retained WebView sessions, a bounded residency pool,
an Argui layout slot, native Wry views, and browser iframes. Enable the
`webview` feature on `argui-runtime` and depend on `argui-webview` for its
session and layout types.

```toml
argui-runtime = { version = "0.4.0", features = ["webview"] }
argui-webview = "0.4.0"
```

| Host | Backend | Status |
| --- | --- | --- |
| Linux Wayland | GTK 3, WebKitGTK 4.1, Tao/Wry | implemented and tested |
| Windows and macOS | Winit/Wry | compiles; native behavior needs platform validation |
| WebAssembly | retained iframe above the canvas | implemented and browser-tested |
| Linux X11 | — | unsupported by the GTK host |

## Linux dependencies

Fedora:

```sh
sudo dnf install pkgconf-pkg-config gtk3-devel webkit2gtk4.1-devel \
  libsoup3-devel javascriptcoregtk4.1-devel
```

Ubuntu or Debian:

```sh
sudo apt install pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev
```

These development packages are needed only when the WebView feature is enabled.

## Retained sessions

Create `WebViewState` outside `render()`, then place its view in the element
tree:

```rust
use argui_ui::Element;
use argui_webview::{WebView, WebViewError, WebViewSource, WebViewState};

/// Creates one retained webpage session and its Argui layout slot.
/// `url` must be an HTTP(S) page; invalid URLs return an error.
pub fn webpage_slot(url: &str) -> Result<(WebViewState, Element), WebViewError> {
    let state = WebViewState::new(WebViewSource::url(url)?);
    let element = WebView::new(&state).build();
    Ok((state, element))
}
```

The runtime reads final layout bounds and mounts the native or browser view above
the canvas. Bounds follow canvas scaling and page movement. Unsupported
transforms or occluding Argui layers hide the native content conservatively.

The pool keeps at most two native views by default. Inactive entries expire
after 30 seconds or are evicted least-recently-used. Visible entries are never
evicted to create capacity. `next_expiry()` is the next event-loop deadline;
do not poll the pool.

Session identity and source survive eviction. Browser DOM state and unsaved
forms do not. `release()` blocks remounting until `load()` or `reload()`.
Close pool entries before destroying their window.

## Content policies

| Source | Default policy |
| --- | --- |
| Email HTML | sanitized `srcdoc`; scripts, forms, frames, downloads, popups, remote resources, and top navigation blocked |
| Webpage | scripts and forms allowed; same-origin access, application IPC, permissions, downloads, and popups blocked |

Blocked links emit `NavigationRequested`. The application decides whether to
open them. Validate URLs and capture model owners weakly in callbacks.

Email HTML is sanitized with Ammonia and a restrictive CSP. Native HTML is served
through a private protocol. Inline styles are removed; this is a safe document
viewer rather than a complete email renderer.

Browser pages may refuse framing through CSP or `X-Frame-Options`. Cross-origin
browser restrictions also prevent dependable title, navigation, and load-error
inspection. Argui does not report a false remote `Loaded` event.

## Explicit webpage options

`WebViewState::with_options` validates immutable permissions:

```rust
use argui_webview::{
    PopupPolicy, WebCompatibility, WebViewError, WebViewOptions, WebViewSource, WebViewState,
};

/// Creates a browser-compatible page session using a trusted relay origin.
/// `url` is the page and `relay_url` is the relay's HTTP(S) origin.
/// Returns an error for an invalid URL or incompatible security policy.
pub fn compatible_page(url: &str, relay_url: &str) -> Result<WebViewState, WebViewError> {
    WebViewState::with_options(
        WebViewSource::url(url)?,
        WebViewOptions::webpage()
            .compatibility(WebCompatibility::compatible(relay_url)?)
            .popups(PopupPolicy::Block)
            .allow_downloads(false),
    )
}
```

Browser `PopupPolicy::Sandboxed` keeps the popup sandboxed; `External` allows
it to escape. Native Wry rejects non-blocking popup policies because inheritance
cannot be guaranteed. Downloads require explicit opt-in and remain subject to
browser or OS policy. Email sessions cannot loosen their restricted policy.

Compatible browser mode uses a trusted relay on a separate HTTPS origin so the
inner page keeps its own origin. It can support origin-dependent APIs such as
`localStorage`, but cannot bypass third-party cookie, authentication, or
framing restrictions.

The included `crates/argui-webview/src/browser/relay/server.py` server is a
development reference. Run it with the actual application, relay, and allowed
website origins; `--help` describes the required options. A production relay must keep
its response CSP, exact HTTP(S) allowlist, message origin checks, and dedicated
origin. It never proxies destination pages or grants Argui IPC.

## Linux surface boundary

The Linux host gives WGPU a Wayland subsurface below GTK's WebKit child. GTK
remains the input owner; the WGPU child has an empty input region. The canvas is
reattached when GTK recreates its parent surface after hide/show.

`gtk_host/canvas.rs` contains the narrow unsafe boundary for borrowed GTK
display/surface handles. The Tao window owns the foreign handles; Argui owns only
its child surface. WGPU retains the child until the swapchain is dropped. This
lifetime needs native lifecycle tests and must not spread into UI or renderer
data types.

## Verification

```sh
cargo nextest run -p argui-webview --all-features
python3 crates/argui-webview/tests/browser/relay/server.py
```

Rust tests in `crates/argui-webview/tests` cover sanitization, navigation,
mounts, options, and cache policy. The browser checks are
`crates/argui-webview/tests/browser/dom.mjs` and
`crates/argui-webview/tests/browser/relay/client.mjs`; both require an installed
Puppeteer module and a Chrome or Chromium executable. Set `PUPPETEER_MODULE`
and `CHROME_PATH` to those actual locations and run each Node script through
the [private Linux display](../contributing/linux-testing.md). The relay client
starts and stops its own local server.
