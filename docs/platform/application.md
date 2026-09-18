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
argui = { version = "0.3.0", features = ["tray"] }
```

Windows and macOS use their notification area; Linux uses the
StatusNotifierItem protocol. Web reports `TrayUnavailable`.

Tray menus are declarative. After changing labels, checked state, enabled state,
or nesting, return `AppUpdate::tray_changed()`. Built-in items can show, hide,
focus, toggle, or close a `WindowKey`; custom items arrive through
`AppEvent::Tray`.

## Global shortcuts

Enable `global-shortcuts` directly, or use the `desktop` feature profile. A
shortcut has an application-defined ID and a portable accelerator. Modifiers
must precede one physical key:

```rust,ignore
use argui::platform::{ApplicationConfig, GlobalShortcut};

let config = ApplicationConfig::new(identity, window)
    .with_global_shortcut(GlobalShortcut::new("show-search", "CmdOrCtrl+Space"));
```

`CmdOrCtrl` resolves to Command on macOS and Control elsewhere. Other accepted
modifiers are `Shift`, `Alt`, `Ctrl`, and `Super`; keys use physical names such
as `KeyK`, `Space`, `ArrowUp`, or `F12`.

Press and release transitions arrive as `AppEvent::GlobalShortcut`, even while
all application windows are hidden. Returning `AppCommand::FocusWindow` shows,
restores, and focuses the target window. Combine this with
`CloseBehavior::Hide` and a tray `Quit` action for a launcher-style app. The
`spotlight` example demonstrates the complete flow with `CmdOrCtrl+Space`.

Winit cannot directly unmap a Wayland toplevel. On that backend Argui implements
`HideWindow` by minimizing the surface immediately; a compositor-authorized
activation token restores it when `FocusWindow` follows a portal shortcut.

Native registration supports Windows, macOS, Linux X11, and Linux Wayland.
Wayland uses the XDG Global Shortcuts portal: the desktop presents a one-time
approval/configuration dialog, and Argui consumes its activation token when a
shortcut returns `AppCommand::FocusWindow`. Host applications must install a
`.desktop` file whose basename matches
`ApplicationIdentity::linux_application_id()`; packaged applications normally
already satisfy this. A missing portal/backend, a rejected host identity,
registration conflicts, and invalid accelerators emit
`RuntimeEvent::GlobalShortcutsFailed`. Targets without a native implementation
emit `RuntimeEvent::GlobalShortcutsUnavailable`.

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
