# Native popovers

Argui can present an anchored panel inside its window or on a native surface
that extends beyond the window bounds. Content, placement, scrolling and input
stay in Argui. Platform adapters handle window ownership and the available work
area. Creation or rendering failures fall back to in-window presentation.

[Desktop backdrops](desktop-backdrops.md), which expose and blur the desktop
behind window regions, are a separate feature.

## Enable native surfaces

Choose widgets and native integration independently. `native-popups` enables no
widgets; `widgets-all` does not enable native popovers.

```toml
[dependencies]
argui = { git = "https://github.com/ExtraBinoss/argui", features = [
    "widget-popover", "widget-select", "widget-tooltip", "native-popups"
] }
```

```rust
use argui::ui::{Element, OverlaySurface};
use argui::widgets::Popover;

let panel = Popover::new(
    "sharing",
    "Sharing settings",
    open,
    trigger,
    Element::text("Sharing options"),
)
.surface(OverlaySurface::PreferNative)
.size(320.0, 400.0)
.build(theme);
```

`OverlaySurface::InWindow` keeps the panel inside its host surface.
`OverlaySurface::PreferNative` requests a native surface with automatic fallback.
Without the feature, including on the web, the preference resolves in-window.
It contains no operating-system types.

`Popover`, `Tooltip`, `TooltipHost`, `Select`, `Menu`, `ContextMenu` and
`DatePicker` expose `.surface(...)`. For `Menubar`, configure its menus. Custom
elements use `.portal_surface(...)` after `.anchored_portal(...)` or
`.rect_portal(...)`, or construct `Portal::new(...).surface(...)` directly.

An unspecified preference inherits from the nearest ancestor portal; the root
default is `InWindow`. Nested popovers therefore inherit `PreferNative`. An
explicitly in-window child stays inside its native parent's surface, with that
surface's constraints and scrolling. Only element-anchored and rectangle-anchored
portals are native candidates; modal and fullscreen layers remain in-window.

The gallery's Popover and Tooltip pages expose **Allow outside this window**.
Build with `--all-features` for the native backend; the browser exercises fallback.

## Platform support

The private `PopupBackend` contract and runtime-facing `NativePopup` live in
[argui-platform/src/popup.rs](../../crates/argui-platform/src/popup.rs).
Adapters are selected with `cfg(target_os)`.

| Window backend | Presentation | Adapter |
| --- | --- | --- |
| Linux X11/Xwayland with winit | Native popup window | [linux.rs](../../crates/argui-platform/src/popup/linux.rs) |
| Windows with winit | Native owned window | [windows.rs](../../crates/argui-platform/src/popup/windows.rs) |
| macOS with winit | Attached AppKit window | [macos.rs](../../crates/argui-platform/src/popup/macos.rs) |
| Linux Wayland or GTK host | `InWindow` fallback | Native presentation unavailable |
| Web, other OS or disabled feature | `InWindow` | No native API calls |

**X11:** uses `override_redirect`, EWMH `TOOLTIP`, `POPUP_MENU` and `COMBO`
types, and `WM_TRANSIENT_FOR` for the actual parent, including nested surfaces.
`_NET_WORKAREA` is intersected with the parent's monitor; without a window
manager, monitor bounds are used. The optional `x11rb` dependency belongs to
`argui-platform`. See the [EWMH specification](https://specifications.freedesktop.org/wm/latest/ar01s05.html).

**Windows:** uses a winit owner, `WS_POPUP` and `WS_EX_TOOLWINDOW`, without
`WS_CHILD`. Tooltips add `WS_EX_NOACTIVATE`. `MonitorFromWindow` and
`GetMonitorInfoW.rcWork` supply the work area; placement is shared across OSes.
See [owned windows](https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features#owned-windows).

**macOS:** attaches an undecorated `NSWindow` through
`addChildWindow(..., Above)` and detaches it before destruction.
`NSScreen.visibleFrame` is converted to winit physical pixels. AppKit calls run
on the OS event-loop thread. See [addChildWindow](https://developer.apple.com/documentation/appkit/nswindow/addchildwindow(_:ordered:)).

**Wayland:** winit 0.30.13 does not expose `xdg_popup` creation or the input
context needed for its grab. The GTK host does not supply this capability
either. WebView-enabled applications use GTK on Wayland and winit on X11.
A future adapter must handle `xdg_positioner`, compositor configuration and
`popup_done` behind the same interface. See [xdg-shell](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/blob/main/stable/xdg-shell/xdg-shell.xml).

## Placement and scrolling

Layout keeps the desired size before constraining the portal. `FloatingPlacement`
tries allowed sides, shifts the rectangle and shrinks it when necessary. Its
bounds are the viewport in-window and the parent monitor's work area natively.
Explicit content width and height limits remain effective.

Native geometry uses logical tree coordinates rounded to physical pixels. The
runtime performs at most two update passes to resolve nested anchors and uses
the OS-reported size without oscillating at fractional DPI. Parent movement,
resize and scale changes invalidate geometry. Missing or hidden anchors produce
no panel or hit target. Placement uses the parent's monitor, rather than
searching for a better adjacent monitor.

`Popover`, `Select` and `Tooltip` use `Overflow::Auto` on both axes and contain
scroll propagation. Menus and date pickers reuse these containers. Long tooltips
keep their complete accessible description and can be hovered for wheel input.
Raw portals require their own `ScrollConfig` and overflow policy.

## State, events and rendering

The logical tree and `NodeId`s remain unique. Moving a panel to a native window
does not recreate its model, editor or selection. Layout separates drawing
commands and clips for each surface; glyphs, assets and the GPU device remain
shared. Input maps back to the same tree. IME cursor coordinates belong to the
window hosting the editor.

Clicks inside a nested popover remain inside its ancestor chain. Escape,
outside clicks and focus transfer retain the existing behavior. OS close requests
or focus leaving the group emit `UiEventKind::DismissRequested` through
`EventType::Dismiss`. Controlled widgets must handle it alongside `PointerOutside`
using their behavior/state object. `TooltipHost` handles it automatically.
Native children are destroyed before their parent.

If the backend refuses a surface or loses its GPU surface, the runtime restores
in-window rendering and emits `RuntimeEvent::PopupFallback { node, reason }`,
also available through `WindowRuntimeEvent`. It does not retry every frame;
close and reopen the panel to retry. Disabled features and browser fallback do
not attempt native creation or emit a refusal diagnostic.

Registered content effects work on native surfaces. Separate popup windows
currently use an opaque panel background, or `RendererConfig.clear_color` when
there is no solid panel fill. They do not blur the desktop; overflowing shadows
are clipped to the surface and outer corners remain opaque. In-window popovers
retain their blur and rounded rendering. Per-surface native accessibility
bridges still need validation.

## Validation

Rust tests cover inheritance, overrides, DPI conversion, constrained placement,
missing anchors, separated drawing commands, out-of-viewport hit testing,
scrolling and dismissal. The Linux `native_popups` test checks real X11 windows,
parent/child ownership, editing, selection shortcuts, clicks and wheel input.
See [Linux graphical testing](../contributing/linux-testing.md).

The WebGPU `overlay_effects.mjs` scenario checks the surface preference and
fallback in both themes. Windows and macOS adapters have been cross-compiled;
interaction, IME and multi-monitor behavior still need checks on those systems.
