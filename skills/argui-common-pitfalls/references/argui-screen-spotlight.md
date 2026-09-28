# Monitor spotlight: transparency, dragging and native input

Use the working gallery implementation as the starting point:

- `apps/gallery/src/solid/screen-spotlight-page.tsx` exposes `windows.openSpotlight`.
- `apps/gallery/quickjs-host/src/desktop_application/spotlight.rs` owns the
  actual window, four masks, native pointer handling and X11 input hole.
- The gallery application routes `PlatformEvent::Pointer` to
  `SpotlightState::pointer` and returns its model view for `spotlight`.

The TSX gallery page is a launcher; copying that page alone does not implement
the overlay. Read and adapt its Rust companion before creating another overlay.

## Open at monitor bounds

Inspect window capabilities first. X11 needs absolute positioning,
`window-input-regions`, and an active transparent compositor. Ordinary Wayland
toplevels cannot provide the gallery's positioning and stacking contract.

Create the overlay hidden with the final dimensions:

```rust
let width = f64::from(monitor.width) / monitor.scale_factor;
let height = f64::from(monitor.height) / monitor.scale_factor;
let mut spec = WindowSpec::new(key.clone(), WindowConfig {
    width, height,
    physical_position: Some((monitor.x, monitor.y)),
    decorations: false, resizable: false, transparent: true,
    native_shadow: false,
    level: WindowLevel::AlwaysOnTop,
    ..WindowConfig::default()
});
spec.visible = false;
```

Set state before `OpenWindow`, apply `SetWindowInputRegion`, then show and focus.
Do not open a 640 × 480 Solid scene and resize it to a monitor while it still
paints from the initial JS dimensions. A retained hidden overlay may be reused;
reset geometry, resize/position it, repaint and restore full input before show.

## Four masks, one crop

Keep a single `Rect` in UI logical pixels. Build a `container` with width/height
`percent(1.0)` and four **absolute** children:

| Mask | Insets | Explicit size |
| --- | --- | --- |
| Top | left 0, right 0, top 0 | height crop.y |
| Left | left 0, top crop.y | width crop.x, height crop.height |
| Right | left crop.right, right 0, top crop.y | height crop.height |
| Bottom | left 0, right 0, top crop.bottom, bottom 0 | none |

Use the gallery's `mask` builder with an alpha-bearing `Color::srgba`. The clear
crop has no opaque fill. Paint its border after the masks. With no crop, anchor
one dim mask to all four viewport sides.

`container` uses block layout. Two ordinary siblings each with `height="100%"`
do not overlap: the second is below the viewport. In TSX, both a full-screen
mask and its full-screen `touchArea` must be absolute with all four insets zero.
This mistake makes drag appear broken even though native pan dispatch works.

## Native drag and click-through

Mirror `SpotlightState::pointer`:

1. Primary press: retain the pointer origin and previous crop; set input `Full`.
2. Move: update the crop in Rust and return `ViewUpdate::Rebuild` for this window.
   The native scheduler coalesces redraws. Do not emit each move through QuickJS
   or change the X11 shape for every sample.
3. Release: validate the crop, then apply `WindowInputRegion::Exclude(crop)`.
4. Cancel: restore the previous crop and its input region.

Keep the initiating pointer ID and button with the drag. Other contacts or
button releases must not end it. Disable document text selection on a canvas
that paints labels. Keep `Full` while dragging, including when a delayed preset
request arrives; share the same crop validity check between Record and confirm.

For a movable crop, exclude an interior rectangle inset from its edges, leaving
a native drag band and corner handles. Reject nonpositive interior dimensions.
Test corners before the move band; dim-area presses redraw the crop. A ratio
constraint must preserve the opposite anchor in all four drag directions.

Use the **same UI rectangle** for paint and native input. The runtime converts
input to physical pixels using DPI × UI zoom. Window size is native logical;
window position is physical. Pixel presets divide their dimensions by DPI ×
zoom before changing the UI crop. Reapply input on DPI, resize or zoom changes.

X11 currently supports rectangular excluded regions. On a backend without that
capability, `Full` can still support drawing; do not claim interior click-through.
For a product that fixes zoom, configure `UiZoomConfig::disabled()` once on its
`ApplicationConfig`, including auxiliary windows.

## Rust canvas with Solid controls

A model view and native-host JS root cannot both own the same window. For a
Rust spotlight, exclude its key from native-host scene registration. Put Solid
presets and actions in separate small native windows. Register their keys,
actor generations and fonts, then position them from the settled crop using
physical monitor origin + UI coordinates × DPI × zoom.

Hide controls during the native drag and publish settled geometry on release.
Use a selection revision when async placement completes, so cancelling or
starting another drag cannot reopen stale controls. Do not hold a shared state
mutex while dispatching a synchronous request to the UI thread.

Serialize an entire async controls-placement sequence before reading its revision.
A revision check at Show alone cannot stop two workers from interleaving their
window positions. Hold a separate placement lock, never the model state lock.

On a Wayland desktop, Xwayland only lists X11 clients. Screen UI can use X11
while capture selection uses the Linux portal; this does not make native Wayland
windows enumerable through the X11 client list. Keep that capture fallback
separate from macOS, Windows and a true X11 session's application-owned picker.
