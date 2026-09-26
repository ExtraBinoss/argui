# Scroll, stacking, and virtual lists

`ScrollConfig` keeps input policy explicit. Winit's cross-platform convention is
preserved: positive wheel deltas move content toward the pointer, so Argui's
normal polarity subtracts them from the content offset. Wayland and browser
backends already translate their native sign convention before Argui receives
the event. `ScrollPolarity::Inverted` deliberately reverses it; applications may
change that setting at runtime. Axes, logical line size, and a multiplier are
also container-owned configuration.

Scroll offsets are retained by stable `NodeId`. Nested regions are searched from
the deepest hit to the root. `ScrollPropagation::Chain` transfers only the
unconsumed part of each axis to the next ancestor, `Contain` stops chaining while
allowing an explicit elastic edge, and `None` clamps and consumes. Geometry,
nested clips, hit regions, quads, and prepared glyphs translate together without
recomputing Taffy or reshaping text. Hover is recomputed under the stationary
pointer after movement.

Pixel deltas stay pixel precise. `ScrollPhysics::Hybrid` preserves momentum
provided by the platform and starts Argui's exponential continuation only after
native events become quiet. The integral is evaluated over elapsed time, so the
travel distance is stable at 60, 120, and 144 Hz. Line wheels use the configured
logical line size and remain direct. `Native`, `Direct`, and fully configured
`Inertial` policies are available per container. Overscroll is clamped by
default; elastic resistance, limit, spring, and damping are explicit opt-ins.

Sibling paint and hit-test order use stable `z_index` ordering. `Position::Sticky`
is constrained against its nearest scroll viewport during the incremental
translation pass. It therefore stays pinned without a layout pass. Window
layers and portals own floating, popover, modal, and debug content independently
from scroll clipping.

Scrollbar tracks use four independent `Edges` insets. Composite widgets can
therefore reserve any side for resize handles, inline actions, or overlapping
chrome without changing the scroll viewport. Hit testing records the track's
actual position in paint order, so a later sibling painted above it owns the
overlap while the remaining track stays interactive.

In TSX, `scrollView` has no scrollbar by default. Setting `scrollbarSide`
(`"left"` or `"right"`), `scrollbarWidth`, `scrollbarThumbColor`, or
`scrollbarHoverColor` opts into its native vertical scrollbar. The left side
is useful for a sidebar adjacent to the window edge. The thumb changes color
on hover through retained native state; scrolling does not call JavaScript
for each frame. The Rust API exposes the same choice through
`ScrollbarStyle::side(ScrollbarSide::Left)`.

`ScrollbarPartStyle` gives the track and thumb their own base `QuadStyle`,
`StylePatch` values, and `StyleTransition`. Hover and thumb drag feed the same
retained transition registry as ordinary elements, so colors, borders, opacity,
corner radii, and gradient properties animate on the paint-only path. The
scrollbar never asks the application to rebuild its tree for an interaction
frame. Both axes share the same track click, thumb capture, and drag behavior.
`Always`, `Hidden`, and delayed/fading `Auto` visibility are container-owned;
idle auto scrollbars schedule no frames.

`ScrollRequest` targets an exact container offset, a rectangle, or an element.
Each axis independently supports start, center, end, and nearest alignment,
logical margins, instant movement, or a typed tween. Element reveal walks every
scroll ancestor from the inside out. A focused control is revealed automatically,
and direct wheel, touch, or scrollbar input interrupts an active smooth request.

`VirtualList` supports fixed and measured variable-height rows through one API.
It computes a
visible range plus bounded overscan and represents unseen space with two
lightweight spacers. A logical list of one million rows therefore creates only
tens of elements. Wheel movement uses the translation path inside a stable
overscan chunk; crossing a chunk boundary rebuilds only the bounded visible
window. Variable rows are measured from their real layout, feed an `O(log n)`
prefix index, rebuild the virtual window until it settles, and correct the
retained offset to preserve the visible item. Generic keyed scroll anchoring
also preserves the first visible keyed descendant when content above it changes.
`VirtualList::scroll_to` navigates fixed or variable data with the same four
alignment modes. Applications that retain a controlled offset can use
`VList::window_changed` before notifying their model; offsets inside the same
mounted chunk stay on the engine's translation-only scroll path.

## Scroll-driven graphic effects

Enable `argui-effects`' optional `scroll` feature and register its definitions
with `RendererConfig::effects(argui_effects::registry()?)`. The widget itself
does not depend on the effects crate:

```rust
let list = VList::new("results", 32.0, 240.0, offset)
    .effect(argui_effects::EdgeFade::new(20.0).scroll())
    .build(items.len(), theme, |index| row(&items[index]));
```

Any scroll container accepts the same effect through
`ScrollConfig::default().effect(effect)`. `TreeView` uses the configuration on
its `list`; headers and virtual spacers remain part of the same scroll extent.
Nothing is enabled implicitly on application lists.

`EdgeFade` changes content alpha rather than painting the background color over
it. `EdgeShadow` composites a configurable color over the viewport, including
transparent gaps between children. Choose its color from the current theme;
the gallery uses its foreground at 22% alpha on light surfaces and 6% on dark
surfaces to keep both axes visible without a bright halo. Both expose
width in logical pixels, intensity and `[left, top, right, bottom]` strengths.
Their `.filter()` methods also work on ordinary layers without scrolling.
`.scroll_with(threshold, ramp)` controls reveal distance; `.scroll()` uses a
zero threshold and a smooth 12 px reveal. The default fade width is 20 px.
Bands are capped at half the corresponding viewport dimension. Zero width,
zero intensity and disabled edges avoid scroll-effect layers entirely.

Custom WGSL uses the existing effect registry and ABI. Bind an existing named
parameter with `ScrollEffect::new(layer).bind(filter_index, parameter, metric)`.
`ScrollMetric` supplies clamped offsets, normalized progress, remaining distances,
smooth per-edge strengths, logical viewport dimensions, or normalized local
coordinate mappings for transformed viewports. Types are checked when binding;
rebinding the same target replaces its previous source. Offset and remaining
vectors are in logical pixels; viewport-size bindings use `LogicalPixels` and
therefore receive the renderer's DPI scaling automatically.
Use `.when_edges(strengths, threshold, ramp)` to explicitly skip an entire layer
when its selected edges are invisible. Zero-valued bindings alone do not disable
custom shaders, since zero can be a meaningful input to another effect.

Bindings resolve during painting, not through application callbacks or layout.
Effects run in declaration order on the scrolling content, excluding the
container background, border, scrollbar and separately painted portals. Stable
scrollbar gutters are removed from the effect viewport. No overflow means no
extra layer, and idle effects request no animation frames. Active effects still
have an offscreen/filter cost in the existing render graph; custom shaders retain
their usual responsibility for output alpha and any expansion beyond the source.

The former gallery's **Effects → Scroll shadow** demonstration is archived in
`OLD_API/`. The active gallery's `VirtualList` page exercises the v2 viewport
and bounded list window.
