# Retained compositor

Argui separates content changes from presentation-only changes. A transform or
group-opacity animation updates a stable `CompositorLayer`; it does not rebuild
layout, reshape text, record primitives, or upload unchanged quad, glyph, image,
and vector buffers.

## Frame flow

```text
animation clock
      |
      v
resolved transform / group opacity
      |
      v
LayoutEngine::composite
  - patch retained layer metadata
  - patch hit, clip, scroll, text, and semantic geometry
      |
      v
SurfaceRenderer::render_composite_notified
  - reuse prepared primitive buffers
  - reuse retained offscreen layer textures
  - submit composition passes
```

The runtime coalesces all changes received before a redraw into one frame. Paint,
scroll, and layout invalidations are stronger than composition and automatically
take their normal paths when both occur together.

## Promotion

An element is promoted when its authored transform or group opacity already
needs composition, or when a transform/group-opacity binding or conditional
style can animate it. Promotion occurs during a normal paint frame so the GPU
surface is warm before the first animated frame. Descendants inherit the nearest
layer's presentation delta without each receiving an offscreen texture.

Adding a compositor-capable property to an unpromoted element deliberately
requests one paint frame to establish the retained content. Later values use
composition. Singular transforms, missing retained identities, resize, DPI, and
content changes also fall back to paint rather than presenting stale pixels. A
move that would expose pixels clipped out of the retained surface or an ancestor
effect layer takes the same safe fallback; fully offscreen layers that remain
offscreen do not force a repaint.

## Threading contract

Timelines are not advanced on a free-running worker. The platform event loop owns
input ordering and samples one monotonic time per frame; this prevents animation,
layout, and hit testing from observing different states. The expensive reusable
work lives behind the compositor boundary and the GPU queue.

Renderer-neutral display commands contain no WGPU, window, or OS handles. That
keeps the scene snapshot suitable for a native render-owner thread without
changing UI semantics, while the same architecture continues to work on
WebAssembly targets where thread availability differs. Surface ownership remains
inside `argui-render`; application and widget code never synchronize GPU state.

## Cache identity

`CompositorId` is derived from the retained UI node identity. The renderer cache
uses a separate profiling domain, so an element's authored effect layer and its
compositor wrapper cannot collide. Cache reuse compares content bounds, child
draws, filters, mask, and content revision while intentionally ignoring current
transform and opacity.

Native popup display lists localize compositor transforms to their surface. The
layout engine converts their presentation deltas back to global coordinates for
hit testing and accessibility, so in-window and native overlays follow the same
contract.
