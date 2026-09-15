# Application and windows

`ApplicationConfig` owns runtime identity and the initial `WindowConfig`.
Use one stable reverse-DNS identifier in runtime and packaging metadata:

```rust,ignore
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

The runtime applies identity to Wayland app ID, X11 WM_CLASS, Winit icons, Web
document title, favicon links, and default tray icons.

## Renderer readiness

GPU initialization is asynchronous. `RuntimeEvent::RendererReady` reports a
usable renderer; `RendererFailed(message)` reports initialization failure.
Multi-window applications receive the corresponding window event and key.
`AppModel` receives `AppEvent::WindowReady`.

Hosts can dismiss their loader from these events. Renderer readiness does not
guarantee that the first frame has already presented.

## Initial focus

`WindowConfig::focus_on_launch` defaults to `true` on native targets and
`false` on WebAssembly. The Web default keeps an embedded canvas from stealing
page focus and scroll while loading. A click or touch still focuses it.

```rust,ignore
let window = WindowConfig::default().with_focus_on_launch(true);
```

Enable it for a full-page Web application that should accept keyboard input
immediately; leave it disabled for canvases embedded in a scrollable page.

## Packaging identity

Argui does not package applications. Keep a packager's values aligned with
`ApplicationIdentity`:

```toml
[package.metadata.packager]
product-name = "Example Notes"
identifier = "com.example.notes"
icons = [
  "icons/32x32.png",
  "icons/128x128.png",
  "icons/icon.icns",
  "icons/icon.ico",
]
```

`cargo packager --release` owns installed bundle names, desktop entries, icons,
and Windows resources. If a Linux packager derives its desktop-file ID from the
executable, set `with_linux_application_id` to that filename.

## Tray

```toml
argui = { version = "0.2.1", features = ["tray"] }
```

Windows and macOS use their notification area; Linux uses the
StatusNotifierItem protocol. Web reports `TrayUnavailable`.

Tray menus are declarative. After changing labels, checked state, enabled state,
or nesting, return `AppUpdate::tray_changed()`. Built-in items can show, hide,
focus, toggle, or close a `WindowKey`; custom items arrive through
`AppEvent::Tray`.

## Multiple windows

`AppModel` owns shared application state. `WindowKey` addresses each view and
`AppUpdate` invalidates only changed windows. Open and close windows with
`AppCommand::OpenWindow` and `CloseWindow`. Their surfaces share one WGPU
device.

## Transparent windows and custom chrome

```rust,ignore
use argui::platform::{WindowConfig, WindowLevel};

let overlay = WindowConfig {
    decorations: false,
    transparent: true,
    native_shadow: cfg!(any(target_os = "windows", target_os = "macos")),
    level: WindowLevel::AlwaysOnTop,
    ..WindowConfig::default()
};
```

Transparent windows use premultiplied surface composition and clear to
transparent black. `Interaction::window_drag` marks the draggable element;
`MoveAndToggleMaximize` toggles maximize on double-click. Interactive children
remain normal hit targets.

`SetWindowMousePassthrough` changes input for the complete window. Pixels
outside the native window already belong to other applications.

Native undecorated shadows are available on Windows and macOS. A GPU shadow must
fit inside the surface and therefore enlarges its hit rectangle. Always-on-top
works on Windows, macOS, and X11, but standard Wayland may reject it.

`PlatformEvent::Opened` reports `WindowCapabilities`. Unsupported commands
emit `RuntimeEvent::CommandFailed`; applications should adapt from the reported
capabilities rather than assume an OS behavior.
