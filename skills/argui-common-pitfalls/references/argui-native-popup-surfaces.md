# Native popup surface pitfalls

`allowOutsideWindow` places popup content in a separate OS window when the
backend supports it. Its renderer and OS window must both support alpha. A
transparent clear on an opaque swapchain, or an alpha swapchain in an opaque
window, still leaves rectangular corner patches around a rounded panel. Set
the native window transparent and use a transparent renderer surface and clear.

The popup's native window fits its content bounds. A shadow requires transparent
padding inside those bounds; otherwise its blur is clipped at square surface
edges. Select reserves theme-sized shadow padding and adjusts its placement and
label viewport accordingly. Popover and Tooltip omit the theme shadow for
`allowOutsideWindow`; add padding and update placement together before enabling
their shadows. Keep `contentWidth` independent of trigger width, and account for
the padding when measuring long labels. Use native scroll shadows and native
hover motion rather than expanding the menu to the longest option.

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
