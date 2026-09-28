# Native presentation pitfalls

- A full-screen overlay is a separate `WindowKey`, native window, host graph,
  and presentation session. Enlarging the main window replaces the launcher
  and can persist the overlay dimensions as the user's normal window size.
  Route unsolicited events and service responses to their owning session;
  generation IDs isolate native trees while each JS session has local identity.
- Hiding an auxiliary window retains its host graph. Reopening must rebuild
  from that graph even when the window has no Rust model. Use named focus and
  visibility requests, and check repeated open/close cycles. Mount the native
  scene before showing it; prepare expensive windows after the main window's
  first frame when first-use latency matters.
- Custom chrome needs native `decorations: false`; a painted title bar alone
  leaves OS decorations present. `DragWindow` and `ResizeWindow` require an
  active pointer press. Use separate edge targets, excluding title-bar buttons.
  Set `minimum_size` and `maximum_size` on `WindowConfig`; use actual resize
  events to adapt labels and layout. Persist the normal window's physical
  position separately from its logical client size. Migrate obsolete saved
  dimensions once instead of resetting every manual resize.
- Rounded custom window corners need transparent native and renderer surfaces
  plus a clipped rounded root. Root clipping does not round OS decorations.
  On X11, verify an actual compositor and inspect RGBA corner pixels as well
  as the composed screen. An opaque screenshot from a server without a
  compositor cannot establish that native transparency works on a desktop.
- Install embedded fonts in each scene's `TextEngine` before its first mount.
  Shared image and SVG assets do not transfer an application's custom font
  engine to an independent auxiliary window.
- QuickJS is a JavaScript engine, with no automatic browser or Node globals.
  Inspect the concrete bootstrap before using `console`, `TextEncoder`,
  timers, or cancellation APIs. Supply required host APIs explicitly and test
  rejected promises and timer callbacks in the real QuickJS host. Even passing
  `.catch(console.error)` reads `console` immediately and can abort startup.
- Cargo unifies features across the consuming application. ASHPD accepts
  exactly one async backend; do not combine `async-io` and `tokio`. Its Tokio
  backend needs an owned, persistent executor for connection background tasks.
  A `pollster::block_on` call does not create that executor. Native shortcut
  selection must agree with window backend selection when both `DISPLAY` and
  `WAYLAND_DISPLAY` exist; an X11 window should use X11 shortcut registration.

Build scripts should compile the checked-out dependency revision. Reapplying
a separately maintained patch over an evolving checkout makes subsequent
launches fail with patch conflicts; commit library fixes in their repository
and advance the application's submodule revision.
