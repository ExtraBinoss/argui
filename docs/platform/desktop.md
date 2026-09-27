# Desktop windows and native surfaces

Argui renders the UI inside an operating-system window. The platform crate
creates windows from `WindowConfig`; the runtime owns their lifecycle and input;
the UI crate describes content and anchored overlays. A native JavaScript host
can expose window operations to Solid or React through `ApplicationServices`.
The supplied native CLI host and gallery register those services. A browser
canvas cannot inspect monitors or create a desktop window.

## Create and control windows

`ApplicationConfig::new(identity, main_window)` creates the main window.
`ApplicationConfig::with_window(WindowSpec::new(key, config))` adds an initial
window. From an `AppModel::update`, return an `AppCommand::OpenWindow` to create
one later. The model's `view` must return content for that window key.

```rust
use argui_platform::{CloseBehavior, WindowConfig, WindowKey, WindowLevel, WindowSpec};
use argui_runtime::{AppCommand, AppUpdate};

let inspector = WindowSpec::new(
    WindowKey::new("inspector"),
    WindowConfig {
        title: "Inspector".into(),
        width: 480.0,
        height: 320.0,
        decorations: false,
        resizable: true,
        transparent: true,
        native_shadow: cfg!(any(target_os = "windows", target_os = "macos")),
        level: WindowLevel::Normal,
        close_behavior: CloseBehavior::CloseWindow,
        ..WindowConfig::default()
    },
);
let update = AppUpdate::none().command(AppCommand::OpenWindow(inspector));
```

`width` and `height` are initial **client** dimensions in native logical pixels.
`resizable` controls user resize at creation. `decorations` requests the OS title
bar and border. `transparent` must be chosen at creation; there is no
`setWindowTransparent` service. Transparent pixels reveal the desktop only when
the window compositor supports alpha. A translucent Argui element alone cannot
make an opaque native window transparent. `native_shadow` applies to undecorated
windows on Windows and macOS; it is ignored elsewhere. A GPU-drawn shadow stays
inside the window and is part of its hit area.

For a host-provided TSX view, `ApplicationServices` exposes current window
information and mutations. The native host must register the corresponding
`windows` service; the CLI host and gallery do. Window creation itself is a
host policy: their `windows.openCompanion` service creates a fixed companion
window, while `ApplicationServices` has no generic `openWindow` method.

```tsx solid
import { createSignal } from 'solid-js'
import type { ApplicationServices } from '@argui/host'
import { useTheme } from '@argui/solid'
import { Button, type WidgetTheme } from '@argui/widgets/solid'

export function WindowControls(props: { services: ApplicationServices }) {
  const theme = useTheme<WidgetTheme>()
  const [status, setStatus] = createSignal('')

  const inspectAndResize = async () => {
    try {
      const [info, monitors] = await Promise.all([
        props.services.getWindowInfo(),
        props.services.getMonitors(),
      ])
      await props.services.setWindowSize('main', 900, 600)
      if (info.capabilities.absolutePosition && info.x !== null && info.y !== null) {
        await props.services.setWindowPosition('main', info.x + 20, info.y + 20)
      }
      setStatus(`${monitors.length} monitors; DPI scale ${info.scaleFactor}; UI zoom ${info.uiZoomFactor}`)
    } catch (error) {
      setStatus(String(error))
    }
  }

  return <column gap={8}>
    <Button onClick={() => void inspectAndResize()}>Inspect and resize</Button>
    <text color={theme().text}>{status()}</text>
  </column>
}
```

`setWindowSize` requests a new client size; the OS can apply constraints. Use
`getWindowInfo()` again to read the resulting size. `setWindowPosition` requests
the **outer** top-left position and requires
`info.capabilities.absolutePosition`. `setWindowDecorations`,
`setWindowTitle`, and `setWindowLevel` change the corresponding native
requests. The latter accepts `bottom`, `normal`, or `top`; an accepted
stacking request does not guarantee that every window manager keeps the
window on top.

## Monitors, DPI, and UI zoom

These values use different coordinate systems:

| API | Units |
| --- | --- |
| `getMonitors()` `x`, `y`, `width`, `height` | Physical desktop pixels, with each monitor's native `scaleFactor` |
| `WindowConfig::physical_position` | Initial outer top-left position in physical desktop pixels, where supported |
| `WindowConfig` size, `getWindowInfo()` size, `setWindowSize()` | Native logical client pixels |
| `getWindowInfo()` `x`, `y`, `setWindowPosition()` | Native logical outer screen coordinates, when reported/supported |
| Argui layout and `WindowInputRegion::Exclude` | UI logical units, after application zoom |

`getWindowInfo()` reports native `scaleFactor` and `uiZoomFactor` separately.
One UI unit occupies approximately `scaleFactor × uiZoomFactor` physical pixels.
The runtime applies UI zoom to layout, paint, hit testing, popup placement,
and shaped input regions together. It reinstalls a retained input region after
window resize, scale change, or UI zoom change. An application can read the
current factor from `WindowEnvironment::ui_zoom` or disable the built-in zoom
gestures with `ApplicationConfig::with_ui_zoom(UiZoomConfig::disabled())`.
Do not treat monitor physical coordinates as `setWindowPosition` logical
coordinates, especially across monitors with different DPI. Window position or
visibility may be `null` in TSX when the backend cannot report it.
For Rust applications, `PlatformEvent::Opened` reports the initial
`WindowCapabilities` and physical drawable size; `ScaleFactorChanged` and
`Resized` report later changes.

## Native dragging and pointer input

For Rust-rendered custom chrome, attach `Interaction::window_drag` to a visible
title-bar element. `MoveAndToggleMaximize` starts a native move on press and
toggles maximize on a double click. Leave interactive child controls outside
that hit region. The current generated TSX primitive schema has no equivalent
`windowDrag` property; a TSX host can programmatically request a position
change where supported, but that is not a native pointer drag.

```rust
use argui_core::Color;
use argui_ui::{Element, Interaction, WindowDragBehavior, length};

let title_bar = Element::container([])
    .width(length(320.0))
    .height(length(40.0))
    .background(Color::srgb(0.12, 0.16, 0.24))
    .interaction(Interaction::default().window_drag(WindowDragBehavior::MoveAndToggleMaximize));
```

Window alpha and an Argui element's `pointerEvents` affect paint and UI hit
testing, not OS-level pointer routing. `WindowInputRegion` controls whether
other applications receive pointer events:

- `Full`: the complete window receives input.
- `PassThrough`: the complete window lets input reach windows underneath.
- `Exclude(rect)`: only the rectangle lets input through; the rest receives
  input. The rectangle is in Argui UI logical units.

The Rust model can return an input-region command. Paint the clear hole from
the same rectangle used here; changing native input does not paint it. During
a drag across the hole, temporarily install `Full` and restore `Exclude` on
release so the drag keeps its pointer stream.

```rust
use argui_core::{Point, Rect, Size};
use argui_platform::{WindowInputRegion, WindowKey};
use argui_runtime::{AppCommand, AppUpdate};

let hole = Rect::new(Point::new(80.0, 60.0), Size::new(320.0, 200.0));
let update = AppUpdate::none().command(AppCommand::SetWindowInputRegion {
    window: WindowKey::new("overlay"),
    region: WindowInputRegion::Exclude(hole),
});
```

The corresponding TSX service call is
`await services.setWindowInputRegion('overlay', { mode: 'exclude', rect: { x: 80, y: 60, width: 320, height: 200 } })`.
For whole-window routing, use `setWindowMousePassthrough('overlay', true)` and
restore it with `false`. Check `getWindowInfo('overlay').capabilities` before
using either operation; service failures reject the promise. Rust command
failures emit `RuntimeEvent::CommandFailed`.

## Popups beyond the window

An ordinary `popupWindow` remains in its owning window. `allowOutsideWindow`
requests an anchored native surface so a menu, tooltip, or popover can cross
the outer edge. It is a preference: without the runtime's `native-popups`
feature, on WebAssembly, on native Wayland, and on unsupported hosts, the
content stays in-window. The supplied CLI native host currently enables
`window-input-regions` but does **not** enable `native-popups`; enable that
feature in its `argui-runtime` dependency to request detached popups.

```tsx solid
import { useTheme } from '@argui/solid'
import { Popover } from '@argui/widgets/solid'
import type { WidgetTheme } from '@argui/widgets/solid'

export function OutsidePopover() {
  const theme = useTheme<WidgetTheme>()
  return <Popover id="settings-popover" trigger="Settings"
    placement="rightStart" contentWidth={240}
    allowOutsideWindow={true} opaque={true}>
    <text color={theme().text}>Window settings</text>
  </Popover>
}
```

The same surface preference exists in Rust after constructing a portal:

```rust
use argui_ui::{Element, FloatingPlacement, OverlaySurface, Placement, WindowLayer};

let popup = Element::container([Element::text("Window settings")])
    .anchored_portal(
        WindowLayer::Popover,
        "settings-trigger",
        FloatingPlacement::new(Placement::BottomStart),
    )
    .portal_surface(OverlaySurface::PreferNative);
```

The anchor must exist with that stable native `id` in TSX or matching element
key in Rust. Native placement uses the parent monitor's work area; in-window
placement uses the viewport. Parent movement, resize, DPI, and UI zoom update
popup geometry. If native creation fails at runtime, Argui restores in-window
content and emits `RuntimeEvent::PopupFallback`. Detached surfaces use opaque
panel rendering: they cannot blur the desktop, and shadows cannot paint beyond
the popup surface. Use in-window layers for modal and full-screen UI.

## Backend limits

| Backend | Absolute position / non-normal level | Whole-window pass-through | Rectangular input hole | Detached popup |
| --- | --- | --- | --- | --- |
| Windows | Available as OS requests | Available | Unsupported | Owned native window with `native-popups` |
| macOS | Available as OS requests | Available | Unsupported | Attached child window with `native-popups` |
| Linux X11 or Xwayland | Available as window-manager requests | Available | With `window-input-regions` and X Shape | Native popup with `native-popups` and a Winit host |
| Linux Wayland | No portable absolute top-level placement or stacking guarantee | Available, subject to backend result | Unsupported | In-window fallback |
| Web, Android, iOS | No desktop window positioning or overlay contract | Unsupported for desktop overlays | Unsupported | In-window fallback |

On X11, `transparentCompositing` is true only when Argui detects an active
compositing manager; that detection is built with `window-input-regions`.
For a screen-spanning translucent overlay with a click-through hole, require
`backend === 'x11'`, `inputRegions`, `absolutePosition`, and
`transparentCompositing` before opening it. The gallery's **Screen spotlight**
demonstrates this on X11: it draws a dim surface around a shared clear/input
rectangle. It is a selection demonstration and does not capture the screen.
