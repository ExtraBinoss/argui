# WebView integration

`argui-webview` provides retained WebView sessions, a bounded residency pool,
an Argui layout slot, native Wry views, and browser iframes. Enable the
`argui/webview` feature.

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

```rust,ignore
let state = WebViewState::new(WebViewSource::url("https://example.com")?);
let element = WebView::new(&state).build();
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

```rust,ignore
let state = WebViewState::with_options(
    WebViewSource::url(url)?,
    WebViewOptions::webpage()
        .compatibility(WebCompatibility::compatible(relay_url)?)
        .popups(PopupPolicy::Block)
        .allow_downloads(false),
)?;
```

Browser `PopupPolicy::Sandboxed` keeps the popup sandboxed; `External` allows
it to escape. Native Wry rejects non-blocking popup policies because inheritance
cannot be guaranteed. Downloads require explicit opt-in and remain subject to
browser or OS policy. Email sessions cannot loosen their restricted policy.

Compatible browser mode uses a trusted relay on a separate HTTPS origin so the
inner page keeps its own origin. It can support origin-dependent APIs such as
`localStorage`, but cannot bypass third-party cookie, authentication, or
framing restrictions.

```sh
python3 crates/argui-webview/src/browser/relay/server.py \
  --app-origin http://127.0.0.1:8081 \
  --public-origin http://127.0.0.1:8082 \
  --port 8082 --allowed-origin https://example.com
```

The included server is a development reference. A production relay must keep
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
python3 crates/argui-webview/tests/browser/relay/server.py
node crates/argui-webview/tests/browser/relay/client.mjs

CHROME_PATH=/path/to/chrome \
PUPPETEER_MODULE=/path/to/puppeteer/puppeteer.js \
node crates/argui-webview/tests/browser/dom.mjs
```

Rust tests in `crates/argui-webview/tests` cover sanitization, navigation,
mounts, options, and cache policy. `tests/browser/dom.mjs` exercises the browser
integration with `CHROME_PATH` and `PUPPETEER_MODULE` set. Run browser and native
checks through the [private Linux display](../contributing/linux-testing.md).
