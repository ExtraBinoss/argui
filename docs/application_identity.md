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
