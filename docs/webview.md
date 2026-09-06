# WebView integration status

`argui-webview` supplies retained sessions, a bounded residency pool, a layout
slot, a desktop Wry backend and a browser iframe backend. Enable its re-export
with `argui`'s `webview` feature.
The runtime mounts these slots automatically from the retained layout. On
Linux, WebView-enabled application launches use GTK/Tao with a dedicated WGPU
Wayland subsurface; the same runtime handles layout, input, animations and
Devtools. The Windows/macOS Wry attachment uses the Winit host and still needs
platform validation. The GTK host currently requires native Wayland, not X11.

Run `GDK_BACKEND=wayland cargo run -p argui-widget-gallery --features webview`
and open **Examples → WebView**, then **Email** or **Webpage**. For the web,
run `./scripts/serve-widget-gallery.sh` and open `/widgets/`; the script enables
the same feature automatically.

## Browser backend

The browser mounts retained iframe elements above the owning canvas. Bounds
follow Argui layout and canvas CSS scaling/offsets; rectangular clipping does
not resize the document. Covered or unsupported transformed surfaces are hidden
conservatively. DOM scroll/resize observers are event-driven and removed on
eviction, as are document listeners. One cancellable timer wakes the runtime at
the next idle eviction; no continuous rendering or polling is added.

Email uses sanitized `srcdoc`, a restrictive CSP, and `sandbox="allow-same-origin"`
without `allow-scripts`, forms, popups, downloads or top navigation. Same-origin
access is granted only so the trusted parent can intercept link clicks; email
scripts remain forbidden both by sandbox and CSP. HTTP/HTTPS links emit
`NavigationRequested` and the gallery opens them in its separate Webpage session.
Context-menu navigation is suppressed for restricted emails. Remote images and
tracking requests are blocked, and referrers are disabled.

The default isolated Webpage mode permits scripts/forms but omits `allow-same-origin`, so embedded
pages cannot access the application's DOM. It exposes no application IPC and
does not grant camera, microphone, geolocation or clipboard permissions. Sites
may refuse embedding via CSP/X-Frame-Options; this cannot be bypassed. Browser
restrictions also prevent reliable inspection of cross-origin navigation,
titles and load errors, so the backend does not emit a false `Loaded` success
for remote pages. Unlike native incognito views, browser frames do not guarantee
an isolated cookie/profile store. Popups and downloads are blocked by default.

## Explicit webpage permissions

`WebViewState::with_options` validates immutable session permissions. Existing
`WebViewState::new` callers retain the restrictive defaults. Email options cannot
enable scripts, compatible origins, popups or downloads.

```rust
use argui::webview::{PopupPolicy, WebCompatibility, WebViewOptions, WebViewSource, WebViewState};

fn webpage(url: &str, trusted_relay_url: &str) -> Result<WebViewState, argui::webview::WebViewError> {
    WebViewState::with_options(
        WebViewSource::url(url)?,
        WebViewOptions::webpage()
            .compatibility(WebCompatibility::compatible(trusted_relay_url)?)
            .popups(PopupPolicy::Block)
            .allow_downloads(false),
    )
}
```

On the browser, `PopupPolicy::Sandboxed` permits new windows that inherit the
sandbox; `External` explicitly permits them to escape it. Browser popup blockers
and user-activation requirements still apply. Downloads require explicit opt-in
and remain subject to browser policy. On native Wry, downloads use the engine's
download destination when enabled. Non-blocking popup policies return
`UnsupportedOptions`: Argui does not currently guarantee policy inheritance for
native popup windows. Wry already preserves website origins, so the compatibility
setting only affects the browser backend.

### Compatible mode and deployment

Compatible mode uses **two frames**: a trusted relay on a distinct origin, then
the website. The inner frame permits scripts/forms and preserves the site's
origin, allowing origin-dependent APIs such as localStorage. This is not a new
browser engine: third-party cookies, authentication and embedding remain subject
to browser and website restrictions. Sites refusing frames still cannot load.

Run the supplied relay, configuring the actual application, relay and target
origins (these values are examples, not engine exceptions):

```sh
python3 crates/argui-webview/src/browser/relay/server.py \
  --app-origin http://127.0.0.1:8081 \
  --public-origin http://127.0.0.1:8082 \
  --port 8082 --allowed-origin https://example.com
```

The gallery script starts this relay automatically. In **WebView → Webpage**,
use **Mode: isolated / compatible** to compare retained sessions. Configure
`ARGUI_WEBVIEW_ALLOWED_ORIGINS` as a space-separated origin list before running
the script, or set `ARGUI_WEBVIEW_RELAY_URL` to an already deployed trusted relay.
The URL is compile-time gallery configuration; without it the mode control is
absent. An invalid or unreachable relay emits an error, never a silent fallback.

For production, host the relay at the root of a dedicated HTTPS origin behind
your HTTP server/reverse proxy. The included Python server is a development and
reference server, not a hardened production service. Preserve its response CSP
and other security headers, and serve `config.js` from trusted deployment
configuration. Configure the application's own CSP `frame-src` to permit the
relay. Do not host user-controlled scripts/content on the relay origin or source
its URL/configuration from untrusted content.

The relay enforces an explicit HTTP(S) origin allowlist through both request
validation and **server response CSP**, including redirects. Application and relay
hosts cannot appear in the website list, even with different ports or schemes.
Allow all required destination origins explicitly; wildcards are rejected.
Messages are checked against the exact sending window, origin and protocol.
This relay only embeds pages: it never proxies websites, strips their protection
headers or grants website code an Argui IPC API. Its `accepted` response confirms
navigation configuration, not that the website rendered successfully.

Relay verification:

```sh
python3 crates/argui-webview/tests/browser/relay/server.py
node crates/argui-webview/tests/browser/relay/client.mjs
```

The browser test uses the real relay and locally served websites to check
origin-preserving storage, retained state, reload, rejected messages, allowlisted
navigation and server-CSP rejection of redirects to the application.

Browser DOM integration checks: `node crates/argui-webview/tests/browser/dom.mjs`
with Puppeteer installed, or `PUPPETEER_MODULE` pointing to an existing module
exporting it, and `CHROME_PATH` set to a Chrome executable. Rust tests exercise
the shared sanitizer, mounts, navigation handlers and cache policy.
The end-to-end WASM test is
`node crates/argui-widget-gallery/tests/pages/webview.mjs`, with the same browser
environment and `GALLERY_URL` pointing at the running gallery. It also checks
30-second idle eviction and state preservation after native DOM recreation.

## Retained content

Create `WebViewState` outside `render()`, then build `WebView::new(&state).build()`.
The widget carries an opaque `NativeContent` payload; the runtime resolves its
bounds into `WebViewMount` records. Custom hosts can drive `WebViewPool` directly.
The pool uses the same `WebViewBackend`/`NativeWebView` contract on every OS.
The GTK backend accepts coordinates relative to its registered client container.

The default pool retains at most two native views. Inactive instances expire
after 30 seconds, or are evicted least-recently-used when a slot is needed.
`next_expiry()` is the event-loop deadline; do not poll an idle pool. An overlay
sets `occluded` without removing the mount, retaining the instance without
painting it or allowing it to receive native input. Visible views are never
evicted to make room: exceeding the limit returns `CapacityExceeded`.

Session identity and the latest source survive eviction. Browser DOM, script
state and unsaved forms do not. `release()` prevents remounting until a new
`load()` or `reload()`. Close a host's pool entries before destroying its window.
The counters describe native instances, not total browser-process memory.

## HTML restrictions

HTML is sanitized with Ammonia and served through a private custom protocol
with a response CSP. Scripts, frames, forms and remote resources are disabled.
Inline styles are currently stripped by the sanitizer; this is a conservative
HTML viewer, not a complete email formatting pipeline. Native JavaScript is
also disabled. Browser and HTML sessions cannot change security policy in place.
All native sessions currently use incognito mode; persistent account profiles
and explicitly enabled remote email images are not implemented yet.

Popups and disallowed navigations produce `NavigationRequested`; automatic
downloads are denied unless explicitly enabled for webpages, and new permission
grants are denied. No application IPC handler is
installed. The application must decide whether to open external links.

## Wayland surface ownership and unsafe boundary

The Linux host uses GTK3/Tao for the parent and Wry container, and a distinct
Wayland subsurface for WGPU. Presenting WGPU directly to GTK's surface was tested
and failed with `Explicit Sync only supported on dmabuf buffers`: GTK's SHM
buffers collided with Vulkan's explicit-sync surface state. XWayland and CPU
readback are not used as substitutes.

The platform's `gtk_host/canvas.rs` has a narrowly scoped unsafe allowance:
borrowing GTK's foreign display/surface and implementing the raw window handle.
The retained Tao window owns those handles; the guest Wayland backend must not
disconnect GTK's display or destroy GTK's parent surface. The canvas owns and
destroys only its subsurface and child surface. Its `Arc` is retained by WGPU,
so those proxies outlive the swapchain. Creation, presentation and destruction
are driven on the UI thread, with the parent window retained until after the
renderer is dropped. No `Send`/`Sync` implementations or raw owned pointers are
introduced. The parent-surface lifetime remains explicit in the canvas owner;
this boundary requires native lifecycle testing in addition to the pure tests.

GTK's client allocation determines the child origin and logical size, including
decoration offsets. Scale is applied only to the swapchain dimensions. The
child has an empty input region so GTK remains the sole input dispatcher.
Rounded lower corners are rendered using Argui's existing quad primitive over
a transparent surface, not by changing or copying browser pixels.

GTK child events are handled by WebKit before propagation is stopped, so Tao's
toplevel resize handlers never receive child-local mouse coordinates. Argui
queries pointer coordinates against the toplevel GDK window, clears its hover
when entering native content, and relinquishes keyboard focus while WebKit owns
it. Clicking an Argui control focuses the canvas again.

Blocked links and popup requests emit `NavigationRequested`. Applications can
register `WebViewState::on_navigation_requested` to handle them without polling;
the callback runs outside the session borrow and events remain available through
`drain_events`. Validate the requested URL and capture owners weakly. The gallery
accepts only HTTP/HTTPS and opens these links in its retained Webpage session,
leaving the Email session's restrictive policy unchanged.
