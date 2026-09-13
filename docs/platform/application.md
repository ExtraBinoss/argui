# Application identity, windows, and tray

`ApplicationConfig` is the single source of runtime identity. Use a stable,
lowercase reverse-DNS identifier; the same identifier must be used by the
installed application bundle.

```rust
use argui::platform::{
    ApplicationConfig, ApplicationId, ApplicationIdentity, IconSet, WindowConfig,
};

let identity = ApplicationIdentity::new(
    ApplicationId::new("com.example.notes")?,
    "Example Notes",
    IconSet::new(),
);
let config = ApplicationConfig::new(identity, WindowConfig::default());
```

The runtime applies this information to Wayland app-id, X11 WM_CLASS, supported
Winit window icons, the browser document title, and browser favicon links. A
tray inherits the application icons unless it supplies an override.

## Renderer loading state

GPU initialization is asynchronous. The runtime's event callback receives
`RuntimeEvent::RendererReady` when its renderer becomes available, or
`RuntimeEvent::RendererFailed(message)` on initialization failure. With multiple
windows, these arrive as `RuntimeEvent::Window` carrying the corresponding
`WindowRuntimeEvent` and window key. `AppModel` also receives
`AppEvent::WindowReady` for its views.

A host can use these events to dismiss its own loading indicator; Argui does
not impose an interface for it. The widget gallery forwards readiness and
failure on the web with `argui:renderer-state`, a `CustomEvent` whose
`detail.state` is `ready` or `error`. The Nuxt preview shows its loader before
fetching WASM and waits for this event before revealing the gallery.
`RendererReady` reports renderer initialization, not a guarantee that the first
frame has already been presented.

## Packaging

Argui does not duplicate an application packager. Keep the values in
`package.metadata.packager` aligned with `ApplicationIdentity`:

```toml
[package.metadata.packager]
product-name = "Example Notes"
identifier = "com.example.notes"
icons = ["icons/32x32.png", "icons/128x128.png", "icons/icon.icns", "icons/icon.ico"]
```

`cargo packager --release` then owns the macOS bundle icon and name, Windows
executable resources, and Linux desktop entry. Runtime window APIs cannot set
those installed-package resources reliably.

Some packagers derive the Linux desktop-file id from the executable rather
than the bundle identifier. In that case, make the runtime id match the
generated `<executable>.desktop` file:

```rust
let identity = ApplicationIdentity::new(
    ApplicationId::new("com.example.notes")?,
    "Example Notes",
    icons,
)
.with_linux_application_id("example-notes");
```

```toml
[package.metadata.packager]
identifier = "com.example.notes"
binaries = [{ path = "example-notes", main = true }]
```

## Optional tray

Enable the portable API with:

```toml
argui = { version = "0.1", features = ["tray"] }
```

Argui uses the native notification area on Windows and macOS and the
freedesktop StatusNotifierItem protocol on Linux. Web builds retain the same
configuration types but report `RuntimeEvent::TrayUnavailable`; browsers do
not expose a system tray.

Tray menus are declarative. After changing checked, enabled, labels, or nested
items in the application model, return `AppUpdate::tray_changed()`. Built-in
actions can show, hide, toggle, focus, or close any `WindowKey`; custom actions
are delivered through `AppEvent::Tray`.

## Multi-window model

`AppModel` owns global state. Each view is addressed by `WindowKey`, while
`AppUpdate` invalidates only the windows whose output changed. Open and close
additional native windows or Web canvases with `AppCommand::OpenWindow` and
`AppCommand::CloseWindow`. WGPU device state is shared across their surfaces.

## Transparent overlays and custom chrome

An overlay uses a transparent, undecorated window whose dimensions fit its
interactive panel. Pixels outside that native rectangle naturally belong to
other applications; `SetWindowMousePassthrough` can additionally disable input
for the entire overlay until the application enables it again.

```rust
use argui::platform::{WindowConfig, WindowLevel};

let overlay = WindowConfig {
    decorations: false,
    transparent: true,
    native_shadow: cfg!(any(target_os = "windows", target_os = "macos")),
    level: WindowLevel::AlwaysOnTop,
    ..WindowConfig::default()
};
```

Transparent windows automatically request premultiplied WGPU surface
composition and clear to transparent black. `Interaction::window_drag` marks
the exact element used for native movement; its
`MoveAndToggleMaximize` behavior maximizes or restores on a double-click.
Interactive children remain normal hit targets, so custom minimize and close
buttons can live inside the drag region.

Undecorated native shadows are available through winit on Windows and macOS.
Set `native_shadow` only for those targets and inspect
`WindowCapabilities::native_shadow` when windows are created dynamically.
X11, Wayland, and Web expose no equivalent portable native shadow. A GPU shadow
must stay inside the native surface; growing that surface also grows its
hit-test rectangle, so compact click-through overlays should omit that shadow
instead of creating an invisible input-blocking margin.

`PlatformEvent::Opened` reports `WindowCapabilities`. Always-on-top is
available through winit on Windows, macOS, and X11, but not on standard
Wayland. Unsupported requests emit `RuntimeEvent::CommandFailed`; Argui does
not silently substitute another window behavior.
