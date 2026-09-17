# Desktop backdrops

Expose the desktop through selected elements and request native blur behind
them. Apply the API to a sidebar, header, card or in-window floating panel.
Multiple regions can use different tints.

## Enable the feature

`desktop-backdrop` is disabled by default in `argui`, `argui-runtime`,
`argui-platform` and the gallery. It is included by `--all-features` and is
independent of native popovers, widgets and WGSL effects.

```toml
argui = { version = "0.3.0", features = ["desktop-backdrop"] }
```

Configure the window at creation, then choose regions while rendering:

```rust
use argui::{
    core::{BackdropMaterial, Color},
    platform::WindowConfig,
    ui::{DesktopBackdrop, Element, length},
};

let window = WindowConfig {
    desktop_backdrop: Some(BackdropMaterial::Sidebar),
    ..WindowConfig::default()
};
let glass = DesktopBackdrop::new(
    Color::srgba(0.08, 0.09, 0.12, 0.72), // Active tint and opacity.
    Color::srgb(0.08, 0.09, 0.12),       // Fallback without native support.
).inactive_tint(Color::srgba(0.08, 0.09, 0.12, 0.90));

let sidebar = Element::column([Element::text("Navigation")])
    .width(length(260.0))
    .desktop_backdrop(glass);
let content = Element::column([Element::text("Content")])
    .grow(1.0)
    .background(Color::WHITE);
let root = Element::row([sidebar, content]);
```

Ancestors must remain transparent: an opaque background hides the desktop.
Text and controls retain their own opacity. `fallback` is used on unsupported
surfaces, on the web and when high contrast disables blur. It may be translucent
if the application explicitly accepts transparency without blur.

Add or remove `.desktop_backdrop(...)` and notify the model to toggle the effect.
Removing the last region disables native blur. `glass.blur(false)` uses only
`fallback` and `inactive_fallback`; `.blur(true)` requests blur again.
The window's `Glass`, `Sidebar` and `Header` material hints may render identically
on some systems. Different windows can choose different materials.

`WindowEnvironment::desktop_backdrop_available` reports whether the backend and
GPU surface support the request. The compositor can still reduce the effect
according to its preferences and window state. Native errors emit
`RuntimeEvent::DesktopBackdropUnavailable` and use fallback without stopping
the application.

## Controls and limits

- Active and inactive RGBA tints are per element. `.inactive_fallback(color)`
  sets the unsupported inactive background; it defaults to `fallback`.
- Regions follow layout, scrolling, transforms and rounded clips. Native masks
  round inward to logical pixels.
- Alpha controls background visibility, not blur radius. The operating system
  chooses the blur kernel.
- `argui-effects` filters affect Argui content and cannot read other applications.
- Detached native popovers currently use opaque rendering and fallback. In-window
  popovers and independently configured application windows support this API.

## Platform adapters

`argui_platform::desktop_backdrop::NativeBackdrop` receives visible regions and
preferences from the runtime. Shared geometry stays independent of OS APIs;
native handles remain retained until their effects are destroyed.

| Platform | API and behavior |
| --- | --- |
| Linux Wayland | `ext-background-effect-v1` with Blur capability detection; KDE `org_kde_kwin_blur_manager` when the standard extension is absent. Regions apply at the next surface commit. |
| Linux X11 | `_KDE_NET_WM_BLUR_BEHIND_REGION`, only when advertised on the root window. Rectangles use physical pixels. |
| Windows | DWM `DWMWA_SYSTEMBACKDROP_TYPE` / `DWMSBT_TRANSIENTWINDOW`, using Desktop Acrylic on Windows 11 build 22621 onward. WGPU/DX12 owns a `DxgiFromVisual` DirectComposition swapchain to preserve alpha. Opaque UI pixels cover the window material. |
| macOS | `NSVisualEffectView` with `BehindWindow`, below the GPU view and masked to the regions. Popover, Sidebar and HeaderView materials follow window activity. |
| Web or unsupported API | Fallback color. Translucent web fallback reveals the page beneath the canvas, never the desktop behind the browser. |

On Windows, renderer fallback is enabled by default. Startup first tries DirectX 12
with DirectComposition, then DirectX 12 with an opaque HWND surface, and finally
Vulkan. The runtime emits `RuntimeEvent::RendererFallback` when a compatibility
renderer is selected and disables the desktop backdrop for that session. Applications
can require only the preferred DirectX 12 DirectComposition path when diagnosing GPU
or driver failures:

```rust
let renderer = argui::render::RendererConfig::default().renderer_fallback(false);
```

If every enabled configuration fails, `RuntimeEvent::RendererFailed` contains each
attempt and its initialization error.

Compositor support is detected at runtime. The development machine's Mutter 50.4
does not expose either supported Wayland blur protocol and uses fallback.

References: [Wayland protocol](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/blob/main/staging/ext-background-effect/ext-background-effect-v1.xml),
[KDE WindowEffects](https://github.com/KDE/kwindowsystem/blob/master/src/platforms/xcb/kwindoweffects.cpp),
[DWM backdrop types](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwm_systembackdrop_type),
[NSVisualEffectView](https://developer.apple.com/documentation/appkit/nsvisualeffectview).

## Gallery and validation

```sh
cargo run -p argui-widget-gallery --features desktop-backdrop
```

In **Appearance → Sidebar appearance**, **Desktop glass** controls native blur
and is disabled when unavailable. **Accent tint** changes the tint directly.
Changing **Surface opacity** or **Inactive opacity** enables transparency when
blur is inactive. **Allow transparency without blur** controls that mode separately.
Both switches start off; the main content and top bar remain opaque.

Paint and layout tests cover invalidation, clipping, transforms, fallback and
content opacity. The opt-in `native_desktop_backdrop` test verifies X11 requests
and capability removal on a private server; it does not simulate visible blur.
The browser scenario checks controls, alpha and responsive themes. Follow
[Linux graphical testing](../contributing/linux-testing.md) for every GUI launch.
Windows and macOS adapters have been cross-compiled; their appearance still
needs validation on those systems.
