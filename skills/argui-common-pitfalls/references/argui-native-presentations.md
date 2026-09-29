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
- Paint a custom client border as the last sibling outside the rounded content
  clip, sharing the surface's bounds and corner radii. Insetting the stroke by
  one UI pixel and reducing its radius leaves a dark background ring outside
  the border, especially visible at antialiased corners. Keep the content inset
  separately. If straight sides fade, inspect ancestor and viewport clips;
  moving the border inside changes the silhouette without resolving clipping.
- Propagate committed preference snapshots as events to every live scene,
  including hidden ones. Wake a parked QuickJS actor on service replies, events,
  native input and shutdown. A 100 ms polling sleep delays every window's theme
  update; a lazy scene also needs a wake when its native window is first opened.
  Guard the initial preference read so it cannot overwrite a newer event.
- A valid suboptimal swapchain frame should still be presented. Defer surface
  reconfiguration to the following frame; discarding an already-presented
  drawable during resize can expose the native backing. Transparent X11 clients
  preserve top-left contents and clear newly exposed pixels via bit gravity
  and a zero-alpha background; do not substitute an opaque black backing.
- A retained transparent target must clear the exact footprint it redraws. If
  separate damage rectangles are combined into one union scissor, clear that
  union too: drawing translucent content over uncleared gaps accumulates alpha
  and leaves rectangular artifacts. Apply this contract to both plain and
  effect roots; compare incremental GPU pixels with a fully cleared reference.
- QuickJS is a JavaScript engine, with no automatic browser or Node globals.
  Inspect the concrete bootstrap before using `console`, `TextEncoder`,
  timers, or cancellation APIs. Supply required host APIs explicitly and test
  rejected promises and timer callbacks in the real QuickJS host. Even passing
  `.catch(console.error)` reads `console` immediately and can abort startup.
- Native copy actions use the host clipboard service, not `navigator.clipboard`.
  On Linux, retain its clipboard handle for the application's lifetime: creating,
  writing and immediately dropping the last handle can lose the selection before
  another process reads it. Serialize reads/writes and verify a copy from an
  independent process in a private display without a clipboard manager. Keep
  original errors visible if copying fails, and ignore late feedback after the
  copied value changes or its component is disposed.
- For bounded multiline error text, pair `lineClamp` with `textOverflow="ellipsis"`
  and clip its layout container. Clamped measurement alone can leave an extra
  shaped line painted beyond the intended height. Copy the original diagnostic,
  never the shortened visual label, and inspect a genuinely long error in the
  native window at its minimum size.
- Linked workspaces must bundle one SolidJS runtime and one `@argui/solid`
  adapter. Keeping symlink paths distinct can duplicate both even when their
  versions match. A widget then reads a different context identity and throws
  `useTheme requires a root ThemeProvider` under a valid provider. In Vite,
  resolve real paths with `preserveSymlinks: false` and deduplicate `solid-js`,
  `@argui/solid`, `@argui/host`, and `@argui/widgets`. Inspect the emitted module
  paths when diagnosing this error; adding another provider does not join
  separate framework runtimes.
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

When Rust compilation follows bundling, retain that invocation's bundles in a
private output directory until staging finishes. Another frontend build can
clean a shared `dist` while Cargo runs. Validate every required scene before
compiling, publish complete files atomically, and clean up only the owning
invocation's temporary directory.
