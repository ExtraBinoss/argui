# Application and windows

`ApplicationConfig` owns runtime identity and the initial `WindowConfig`.
Use one stable reverse-DNS identifier in runtime and packaging metadata:

```rust
use argui_platform::{
    ApplicationConfig, ApplicationId, ApplicationIdentity, IconSet, WindowConfig,
};

/// Builds an example configuration using this repository's gallery ID.
/// Returns an error if the application ID is invalid.
pub fn example_config() -> Result<ApplicationConfig, argui_platform::ApplicationIdError> {
    let identity = ApplicationIdentity::new(
        ApplicationId::new("dev.argui.gallery")?,
        "Argui Gallery",
        IconSet::new(),
    );
    Ok(ApplicationConfig::new(identity, WindowConfig::default()))
}
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

```rust
use argui_platform::WindowConfig;

/// Creates a window that accepts keyboard focus immediately on Web.
/// Returns the configured window.
pub fn focused_web_window() -> WindowConfig {
    WindowConfig::default().with_focus_on_launch(true)
}
```

Enable it for a full-page Web application that should accept keyboard input
immediately; leave it disabled for canvases embedded in a scrollable page.

## Accessibility UI zoom

Application-wide UI zoom is enabled by default. `Ctrl + +` and `Ctrl + -` on
Windows, Linux, Android with a hardware keyboard, and WebAssembly increase or
decrease the zoom. macOS and iOS hardware keyboards also accept the native
`Command` equivalents. `Ctrl`/`Command + 0` returns to 100%, and holding that
modifier while scrolling provides continuous zoom. Native trackpad
magnification and a two-finger touch pinch provide pointer and mobile
equivalents.

The runtime combines UI zoom with the host DPI scale at the logical-coordinate
boundary. Layout, shaped text, painting, hit testing, scrolling, IME placement,
safe areas, accessibility bounds, popups, desktop backdrops, and hosted WebView
bounds therefore change together. `WindowEnvironment::ui_zoom` exposes the
current factor when an application wants to display it.

Disable the runtime-owned shortcuts and pinch gesture when an application needs
to reserve them for a product-specific canvas:

```rust
use argui_platform::{ApplicationConfig, UiZoomConfig};

/// Turns off runtime-owned zoom gestures for the given application.
/// Returns the updated configuration.
pub fn without_ui_zoom(config: ApplicationConfig) -> ApplicationConfig {
    config.with_ui_zoom(UiZoomConfig::disabled())
}
```

Disabling UI zoom leaves ordinary application keyboard and pinch handling
unchanged.

## Packaging identity

Argui does not package applications. Keep the installed bundle ID, desktop
entry, and icons aligned with `ApplicationIdentity`. If a Linux packager derives
the desktop-file name from the executable instead of the reverse-DNS ID, use
`ApplicationIdentity::with_linux_application_id` with that actual filename,
without the `.desktop` suffix.

## Tray

```toml
argui-platform = { version = "0.4.0", features = ["tray"] }
argui-runtime = { version = "0.4.0", features = ["tray"] }
```

Windows and macOS use their notification area; Linux uses the
StatusNotifierItem protocol. Web reports `TrayUnavailable`.

Tray menus are declarative. After changing labels, checked state, enabled state,
or nesting, return `AppUpdate::tray_changed()`. Built-in items can show, hide,
focus, toggle, or close a `WindowKey`; custom items arrive through
`AppEvent::Tray`.

## Global shortcuts

Enable `global-shortcuts` on `argui-runtime`; it forwards the feature to
`argui-platform`. Declare both crates directly. A shortcut has an
application-defined ID and a portable accelerator. Modifiers must precede one
physical key:

```toml
argui-platform = { version = "0.4.0", features = ["global-shortcuts"] }
argui-runtime = { version = "0.4.0", features = ["global-shortcuts"] }
```

```rust
use argui_platform::{ApplicationConfig, GlobalShortcut};

/// Adds the search launcher shortcut to an application.
/// Returns the updated configuration.
pub fn with_search_shortcut(config: ApplicationConfig) -> ApplicationConfig {
    config.with_global_shortcut(GlobalShortcut::new("show-search", "CmdOrCtrl+Space"))
}
```

`CmdOrCtrl` resolves to Command on macOS and Control elsewhere. Other accepted
modifiers are `Shift`, `Alt`, `Ctrl`, and `Super`; keys use physical names such
as `KeyK`, `Space`, `ArrowUp`, or `F12`.

Press and release transitions arrive as `AppEvent::GlobalShortcut`, even while
all application windows are hidden. Returning `AppCommand::FocusWindow` shows,
restores, and focuses the target window. Combine this with
`CloseBehavior::Hide` and a tray `Quit` action for a launcher-style app.

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

For transparent windows, custom chrome, monitor coordinates, input regions, and
the per-backend limits, use [Desktop windows and native surfaces](desktop.md).
