# Native popup surface pitfalls

`allowOutsideWindow` places popup content in a separate OS window when the
backend supports it. Its renderer and OS window must both support alpha. A
transparent clear on an opaque swapchain, or an alpha swapchain in an opaque
window, still leaves rectangular corner patches around a rounded panel. Set
the native window transparent and use a transparent renderer surface and clear.

The popup's native window currently fits the panel bounds. A widget shadow
extends beyond those bounds and gets clipped against the rectangular native
surface. On a transparent popup this leaves straight shadow remnants near the
rounded corners, especially at the bottom. Popover, Select, and Tooltip omit
their theme shadow when requesting `allowOutsideWindow`. If a future design
needs a shadow there, first reserve transparent margin in the native window
and adjust placement, painting, and hit geometry together.

The popup's physical scale is native DPI multiplied by UI zoom. Convert the
usable desktop area back through that combined scale while preserving its
physical origin. Otherwise Ctrl+ and Ctrl+wheel change the UI layout while
the OS popup remains at its old size or position. Exercise placement and
corners at 100%, keyboard zoom, and wheel zoom.

Use `scripts/linux-hidden-display.sh` for Linux GUI checks. Its private X11
mode exercises native popups; Wayland currently takes the in-window fallback.
Inspect a compositor capture for visual shape. Raw X11 window pixels can help
measure alpha, but an X11 root capture need not show the same background that
the desktop compositor places behind a transparent popup.
