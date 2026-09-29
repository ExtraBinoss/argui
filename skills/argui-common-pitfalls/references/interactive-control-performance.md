# Interactive control performance pitfalls

Read this when a native splitter, picker, slider, meter, or responsive window becomes
sluggish, or when a typed TSX tree fails native validation.

## Native transition bounds

The adapter accepts `transitionMs` only in the inclusive range 1–60,000 ms.
Zero is invalid, even though the generated JSX property is a `number`.
`transitionTimingFunction` also requires a present timed duration and cannot
accompany `transitionSpring`. Disable both timed properties together:

```tsx both
<container
  transitionMs={animating ? duration : undefined}
  transitionTimingFunction={animating ? 'linear' : undefined}
/>
```

Bound a computed `duration` before it reaches the adapter. A long reader can
advance through segments of at most 60 seconds, retaining its current position
and retargeting from that position. Clamping the duration of a whole-script
animation alone would change the requested playback speed.

A TypeScript pass or bundle build proves neither runtime property ranges nor
successful mounting. Inspect the adapter checks as well. When the user forbids
tests or app launches, respect that limit and report runtime behavior as
unverified instead of claiming a graphical or performance pass.

## Immediate responsive resize

Do not tween an indicator's responsive width, height, or position with the same
duration used to animate selection changes. A 170 ms selection transition can
make the control trail every native resize event. Mount its transition before
the first selection change: attaching the duration only at that change gives
the engine no retained animation origin and skips the first movement. One Solid
pattern is a geometry-keyed `<Show>` around just the indicator. A changed size
then mounts directly at the current selection, while selection changes retain
the existing node and its transition.

Render the latest native dimensions directly. Queued consecutive physical
resize notifications for the same session/window may be collapsed to the newest
size; service replies, visibility and DPI changes remain ordering boundaries.
Debounce geometry persistence after the gesture settles, not the visual layout.
Writing preferences on every resize can also broadcast changes to all scenes.

Guard asynchronous geometry reads with an event revision. A `windowInfo` reply
started before a resize must not overwrite a newer size already applied from
the native event. Refresh DPI/visibility as needed without delaying that event.

Trace presentation as well as layout when a resize appears only after another
click. Frame acquisition `Timeout` must request a retry; `Occluded` must remain
idle until visibility changes. Treating both as a successful skipped frame can
consume the final resize invalidation without ever presenting it. Do not retry
every skipped frame, which would spin while the window is occluded.
An `Outdated`/`Reconfigure` result can occur without any size change. `resize`
then returns false and does not configure the swapchain; call
`reconfigure_surface` in that case before requesting another frame. Apply this
recovery to popup surfaces as well as the main window.

Removing a layout transition must restore the authored Taffy style and report
`Layout`, including a retained-tree revision change when the authored width is
unchanged. Dropping its registry entry alone removes it from the animated-index
list while leaving its interpolated cached width behind. Also settle an active
transition when animation is disabled even if its target value has not changed.
During reconciliation, compare the actual retained Taffy style to the resolved
target: a cached authored style can match the target while Taffy still contains
an animation sample. A description-only equality check then skips restoration.
Cover both removal of the last transition and removal while another remains;
compare cached layout with a fresh engine without injecting pointer input.

For frameless windows, configure native `resizable`, `minimum_size`, and
`maximum_size` where the product needs bounds. Provide reachable edge/corner
press targets with appropriate cursors. Call native resize from pointer-down,
and keep those targets separate from title-bar actions. CSS or preferred TSX
sizes do not configure native window constraints.

## Native split panes and hover handles

For panel resizing, `TouchArea` accepts `resizeTarget`, `resizeAxis`,
`resizeMinimum`, `resizeMaximum`, and optional `resizeTrailing`. The target is
a unique native `id` in the same tree. `horizontal` controls width from pointer
X; `vertical` controls height from pointer Y. Use `resizeTrailing={true}` for a
pane after the divider, so moving left/up increases that pane's size. Bounds
must be finite with `0 <= minimum <= maximum`; calculate them from the usable
workspace while reserving neighbouring panes and gutters.

The Rust engine starts from the measured pane dimension, captures the pointer,
and updates retained layout directly. Subscribe to `onResizeCommit` to store
the final size/proportion once on release. Pointer movement needs no JS `onMoved`
handler, signal update, layout request, or size animation. Keep keyboard resizing
and separator accessibility on an enclosing `focusScope` and send accepted keys
through the same application commit function.

Make the resized pane rigid on its controlled axis with `shrink={0}`. Give
its content a percentage size and let the adjacent pane use `grow={1}` with
`minWidth={0}` or `minHeight={0}` as appropriate. For stacked panes, a growing
upper region and fixed-height lower region follow a native height override
without calculating both heights in JS on every movement. Nested numeric
widths/heights do not follow their parent's live override: use resolvable `%`
or native flex bounds for the content that must respond during the gesture.

A small pill and its enlarged hit target must share their geometry. Put the
painted rectangle **inside** the `TouchArea`, centered by a full-size `row` or
`column`. A default `container` is block layout, so alignment props alone leave
the pill at the start. For a hover-only handle, make its idle background
transparent and use the theme's `primary` as native `hoverBackground`, with a
short color transition. Hovering the enlarged target then reveals the pill
without JS hover signals. Do not add pressed/focus fill if visibility must be
limited to hover; keep keyboard focus semantics independently.

```tsx both
<touchArea
  width={12} height="100%" mouseCursor="ewResize"
  resizeTarget="library-pane" resizeAxis="horizontal"
  resizeMinimum={160} resizeMaximum={maximumLibraryWidth}
  onResizeCommit={event => setLibraryWidth(event.value)}
>
  <row width="100%" height="100%" justifyContent="center" alignItems="center">
    <rectangle width={3} height={32} shrink={0} radii={2}
      background="#00000000" hoverBackground={theme().primary}
      transitionMs={120} transitionTimingFunction="cubic-bezier(0.2, 0, 0, 1)" />
  </row>
</touchArea>
```

Native size overrides survive unrelated producer updates until the authored
dimension acknowledges the commit. Cancellation restores the previous native
or authored size with no commit callback. A changed authored size or handle
configuration interrupts the old drag instead of leaving a provisional override.
When implementing or changing this
engine path, restore the actual cached Taffy style too: removing an override
from a registry alone can leave the last dragged width in the layout cache.

Validate the mounted native tree, not just the TS geometry helper. Compare
pill and hit-target centers on both axes, advance native hover transitions,
and drag all trailing/leading handles through their limits. Count callback
deliveries across a movement burst: zero during movement, one final commit for
a changed size, none on cancellation. Check neighbour reflow and cancellation
against the retained layout cache without replacing the producer tree. These
checks prove geometry and event ownership; presented frame time still needs
a native presentation measurement.

## Popup latency and actor failures

Windowless automation measures tree/layout/paint work; it does not create OS
popup windows. Compare a native input-to-popup-map probe as well, on a private
display. Separate native window creation, GPU renderer initialization, asset
registration, and presentation before blaming the Select's TSX or adding delays.
Repeatedly rebuilding GPU pipelines and registering the app's entire image/SVG
catalog can dominate a small menu. Reuse bounded GPU resources, create a fresh
native owner for each logical popup, and register the scene's referenced asset
versions. Clear retained scene/damage caches when rebinding a surface. Recheck
changed assets and nested menus, not only reopening the same menu.

An inline numeric Solid child such as `<Button>{seconds} s</Button>` reaches
the universal renderer as a number even when its declared input type is string.
Normalize it with `String(value)` at text creation and replacement, or use a
complete string label. A native String wire value containing a number is
rejected. When a hover submenu never appears, inspect the owning actor's stderr
and native commit rejection before changing hit testing or hover delays: that
rejection can stop the entire bar. Test selecting a duration after hover too.

At OS focus return, resume presentation/animation scheduling and request a
fresh frame even if the previous visibility state appears unchanged. A stale
occlusion flag can otherwise keep painting suspended until minimize/restore.
Respect an explicitly hidden/minimized window and keep occluded idle work asleep.
Wayland's Winit host hides by minimizing because `set_visible(false)` has no
effect. Compositor activation can restore that retained surface without a
`ShowWindow` command, and may preserve keyboard focus. Resume its requested
visibility on real focus **or pointer entry**; a restored window can accept
clicks while its retained runtime still refuses to paint. Keep the corresponding
X11/other-backend hidden-window policy intact.
Pointer entry can prove a covered host is available again even when a compositor
omits a new focus event. Recover stale occlusion on that event; clearing occlusion
must also request the first frame, rather than waiting for another input event.
Measure queued commit to render per window and stamp before submitting work.
Stamping after the acknowledgement races with rendering; a global marker can
attribute a hidden scene's prewarm to the next unrelated frame seconds later.
Clear pending markers on hide and rejection. Include swapchain acquisition in
CPU frame timing; starting the profiler after acquisition hides compositor waits.

Reproduce minimizing all windows and restoring only an auxiliary window, then
closing and restoring it separately. If JS commits arrive promptly but multiple
windows stop painting, collect the UI thread stack. A Vulkan Wayland FIFO image
acquisition can wait on an explicit-sync release from a minimized surface and
block the shared event loop. For automatic presentation, prefer supported
tear-free mailbox on actual Linux surfaces, including X11 native popups. Give
active Linux animations their own bounded frame deadlines even when mailbox is
unavailable; a chain of redraw requests is not an independent deadline.
Preserve explicit presentation modes and capabilities. A wake timer alone cannot
fix a blocked acquire call. Mailbox must not turn animations into an unlimited
redraw loop: retain one-shot deadlines, pause hidden work and leave idle windows
without polling. Winit's suspended frame callbacks can also gate remapped
surfaces indefinitely; do not assume a redraw request is a presented frame.
Verify changed page content after a click. Hover can override a selected button's
paint, so an exact selected-color assertion may falsely report a responsive page
as frozen.

For grouped activation, distinguish requested-open windows from currently
presented ones: minimization remains eligible, whereas hidden prewarmed or closed
scenes do not. On X11, Winit's `set_minimized(false)` sends `_NET_ACTIVE_WINDOW`
even for visible windows. Calling it on each peer creates focus ping-pong and
can undo minimization immediately. Raise visible peers without activation. Some
WMs keep minimized clients mapped, so `MapWindow` alone does not restore them.
Use the WM restoration request only for minimized peers, restore activation to
the chosen source, and restack that source last. Protect the
automatic focus fallback caused by minimization; verify minimize-all, restoration
from either window, focus retention and closed-window exclusion under a real
private window manager. A rootful Xwayland server without a WM cannot validate
these behaviors. Native Wayland has no independent application stacking API.

An idle interval percentile may include one-second gaps without indicating a
slow interaction. Correlate the owning window's queued commit with its first
presentation and, when needed, the native acknowledgement and tree invalidation.
Account for capture-stream latency separately from native transaction timing.

Keep a new native popup hidden until its first submitted GPU frame is ready.
Queue submission is asynchronous: mapping immediately after submission can
expose an uninitialized or stale opaque rectangle before the fade starts. Wait
for readiness outside the UI thread and request a redraw to map on the UI thread.

For transparent native surfaces, keep internal blending in linear light but
premultiply the final **encoded sRGB** color by alpha. An sRGB attachment that
encodes already-premultiplied linear RGB produces overly bright rounded corners
and borders that appear to fade differently from the panel. Apply the final
transfer once, after every internal effect, and retain the opaque direct path.
Verify actual RGBA readback at partial corner alpha and compare panel, border,
and shadow samples through the same fade; a geometry-only test cannot see this.

For X11 capture bars, repeated programmatic focus requests can trigger desktop
attention notifications. Reassert their native window level when remapping,
without refocusing them after every crop edit.
Winit's X11 backend does not apply `WindowAttributes::active`: a retained tool
can still trigger a GNOME “window is ready” banner when mapped. Before mapping,
apply `_NET_WM_USER_TIME = 0` for passive presentation and declare auxiliary
capture controls with `skip_taskbar`. GNOME suppresses attention banners for
skip-taskbar tools. Reapply mapping hints on hide/show, preserve existing
`_NET_WM_STATE` atoms such as `ABOVE`, and keep tools focusable by real input.
An always-on-top request means above ordinary application windows; desktop
panels and system notifications can still occupy a higher compositor layer.

## Color pickers and meters

- Move slider and two-dimensional pad thumbs with compositor transforms over
  stable resolved geometry. Reactive `grow`, percentage width, height or inset
  movement requests layout; a checkerboard and surrounding text then reflow
  with every drag sample. Map pointer positions using the actual control bounds,
  clamp at each edge, and retain keyboard/accessibility behavior.
- An HSV saturation/value pad is a single bilinear GPU fill in `srgb`, with
  top-left white, top-right the pure hue, and both bottom corners black. The TSX
  brush uses `kind: 'bilinear'` and four ordered `corners`. Stacking transparent
  white/black gradients under rounded clipping can leave incorrect edge colors.
- Memoize hue-only and RGB-only brushes separately. Alpha-only movement must
  not rebuild the RGB opacity-track stops. The host compares brush values
  structurally so fresh identical objects do not cause native mutations.
- Suppress unchanged/clamped channel updates and batch related Solid signals.
  Do not rescan or UTF-8-validate an unchanged full script on each color sample.
  Color-only changes to text/editor styles require repaint, not glyph layout;
  content and font-metric changes still require layout.
- A changing hex text field can itself trigger layout on every sample. If live
  numeric text is unnecessary, update it at release/keyboard commit while the
  swatch and actual color preview stay live. Keep essential semantics even when
  redundant visible saturation/brightness percentages are removed.
- A meter reveal should clip a stable gradient without changing the meter's
  layout height on each sample. Keep the underlying SVG independently painted.
  Interpolate the short rise/fall with native transitions instead of adding a
  JS 16 ms interval on top of an existing audio poll. Silence removes the
  transition/reveal immediately; retain the meter nodes with zero opacity so
  each silence/activity boundary does not remount layout. Matched reveal and
  inverse-gradient transforms keep the gradient fixed while its clipping
  boundary moves.
  Adding a timer or a generic "latest input" cache does not fix a layout-heavy
  control; trace its actual call sites before blaming or globally throttling
  pointer, wheel, resize or text-edit events. Preserve release/cancel ordering.
- Keep popup scroll viewports full width, with content padding inside them and
  `scrollbarEndInset={0}` when the scrollbar should touch the viewport ends.
  Reserve blur for surfaces that need it; a live picker popover need not create
  another backdrop pass merely because its floating toolbar is blurred.

Retain native IDs and listeners across ordinary controlled-value changes.
Check the native update classification and changed-node/paint costs before
making a smoothness claim. Layout/composition classification alone is not a
measurement of presented frame time.
