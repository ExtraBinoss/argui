# Native popovers

Argui overlays normally stay inside their window. The optional
`native-popups` feature lets an anchored panel extend beyond native window
bounds. Content, state, placement, scrolling, and input remain in Argui; the
platform adapter owns the extra surface.

```toml
argui = { version = "0.3.1", features = [
  "widget-popover",
  "widget-select",
  "widget-tooltip",
  "native-popups",
] }
```

```rust,ignore
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

`InWindow` always uses the host surface. `PreferNative` requests a native
surface and falls back in-window when unavailable. Without the feature, and on
Web, it always resolves in-window.

Popover, Tooltip, TooltipHost, Select, Menu, ContextMenu, and DatePicker expose
`.surface(...)`. Custom portals use `.portal_surface(...)`. Surface choice
inherits through nested portals; an explicit `InWindow` child stays inside its
native parent. Modal and full-screen layers remain in-window.

## Platform support

| Host | Result |
| --- | --- |
| Linux X11/Xwayland with Winit | native transient popup |
| Windows with Winit | native owned window |
| macOS with Winit | attached AppKit child window |
| Linux Wayland, including GTK WebView host | in-window fallback |
| Web or feature disabled | in-window fallback |

X11 uses transient EWMH popup types and the monitor work area. Windows uses an
owned `WS_POPUP` tool window; tooltips do not activate it. macOS attaches an
undecorated `NSWindow` and uses `visibleFrame`. Platform calls stay on the OS
event-loop thread.

Winit does not expose the Wayland `xdg_popup` creation and input-grab context
required by this adapter, so native Wayland intentionally falls back.

## Geometry and scrolling

`FloatingPlacement` tries allowed sides, shifts within the work area, and
shrinks when required. In-window placement uses the viewport; native placement
uses the parent monitor's work area.

Coordinates start in logical tree units and round once to physical pixels.
Parent movement, resize, and scale changes invalidate geometry. Missing or hidden
anchors produce no panel or hit target. Placement remains on the parent's
monitor.

Popover, Select, and Tooltip constrain overflow on both axes and contain scroll
propagation. Raw portals must configure their own overflow and `ScrollConfig`.

## State, input, and fallback

Moving a panel to another surface does not recreate its model, editor, selection,
or node IDs. Glyphs, assets, and the WGPU device remain shared. IME position uses
the window that hosts the editor.

Escape, outside click, nested-overlay membership, and focus transfer keep the
in-window behavior. An OS close or focus leaving the popup group emits
`DismissRequested`. Controlled widgets handle it with their state object;
`TooltipHost` handles it automatically.

If native creation or presentation fails, the runtime restores in-window
rendering and emits `PopupFallback { node, reason }`. It retries when the panel
is closed and reopened, not every frame. Normal Web or disabled-feature fallback
does not emit an error.

Effects render on native surfaces, but those surfaces use an opaque panel
background. They do not blur the desktop, and shadows cannot draw outside the
popup surface. In-window overlays retain backdrop filters and normal rounded
composition. [Desktop backdrops](desktop-backdrops.md) are a separate window
capability.

## Verification

Rust tests cover inheritance, placement, DPI rounding, missing anchors,
surface-separated draw commands, input, scrolling, and dismissal.
`argui-runtime`'s `native_popups` test checks real X11 ownership, nested
surfaces, editing, shortcuts, clicks, wheels, and fallback.

```sh
ARGUI_NATIVE_TESTS=1 cargo nextest run \
  -p argui-runtime --all-features --test native_popups
```

Run it through the [private Linux display](../contributing/linux-testing.md).
The browser `overlay_effects.mjs` scenario checks in-window fallback in both
themes. Windows and macOS compile in CI; interaction, IME, accessibility, and
multi-monitor behavior still require platform testing.
