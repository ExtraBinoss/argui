# Desktop windows and screen overlays

Argui keeps window creation and native input routing at the platform boundary.
`WindowConfig` describes the window's transparency, physical screen position,
stacking level, initial size, and decoration. `WindowCapabilities` reports what
the current backend can actually do. `ApplicationServices` exposes monitor
geometry, window information, level changes, and pointer input policy to TSX.
Native failures reject the service request; an unsupported shaped region never
silently becomes a window that blocks every click.
`transparentCompositing` reports whether translucent window pixels can show
the desktop underneath. On X11 it requires an active compositing manager.

## Coordinates and input

`getMonitors()` returns physical desktop coordinates and native DPI scale.
`WindowConfig::physical_position` uses those physical coordinates. Window size
and `setWindowPosition` remain in native logical pixels. `getWindowInfo()`
reports the native scale and the current UI zoom separately.

`setWindowInputRegion(window, region)` accepts:

- `{ mode: 'full' }`: the whole window receives pointer input;
- `{ mode: 'passThrough' }`: the whole window lets pointer input reach windows
  below it;
- `{ mode: 'exclude', rect }`: only the UI-logical rectangle lets input through.

The runtime retains the input policy and reinstalls its physical shape after a
window resize, display scale change, or UI zoom change. For `exclude`, use the
same UI-logical rectangle for the transparent paint hole and native input.
During a drag that crosses the hole, temporarily use `full`, then restore
`exclude` at release so the active drag does not lose its pointer stream.
Window alpha and UI `pointerEvents` do not alter the OS input region.

## Backend behavior

| Backend | Whole-window pass-through | Rectangular input hole | Absolute position and top level |
| --- | --- | --- | --- |
| X11 | Yes | Yes, X Shape input region | Supported by the window manager |
| Windows, macOS | Yes | Unsupported in this implementation | Available as OS hints |
| Wayland | Yes | Not exposed by Argui | No portable absolute placement or top-level guarantee |
| Web, Android, iOS | Unsupported | Unsupported | Unsupported for desktop overlays |

On Wayland the protocol can define a surface input region. That alone does
not create a portable, monitor-sized overlay above other applications. The
gallery screen-spotlight service therefore returns an explicit error unless
it runs on X11 with shaped input regions and an active compositor. `setWindowLevel` is an OS
request, not a guarantee that every window manager will keep it above all
other windows.

## Try the X11 spotlight

Open **Examples → Screen spotlight** in the native Solid or React gallery on
X11 and press **Open spotlight**. The monitor-sized transparent window dims
the screen around a clear rectangle. Drag on the dim area to change the
rectangle. Click inside it to reach the underlying window. Right-click the
dim area or press Escape while the overlay is focused to close. **Pass through all clicks** exercises the
whole-window policy; **Close spotlight** remains reachable in the gallery.

This is a fake recording selection: it does not capture or record the screen.
The visual hole and X11 input hole are both derived from the same retained
rectangle in the gallery model.
