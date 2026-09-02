# Scroll, stacking, and virtual lists

`ScrollConfig` keeps input policy explicit. Winit's cross-platform convention is
preserved: positive wheel deltas move content toward the pointer, so Argui's
normal polarity subtracts them from the content offset. Wayland and browser
backends already translate their native sign convention before Argui receives
the event. `ScrollPolarity::Inverted` deliberately reverses it; applications may
change that setting at runtime. Axes, logical line size, and a multiplier are
also container-owned configuration.

Scroll offsets are retained by stable `NodeId`. Nested regions are searched from
front to back; a region that is already at its boundary lets the delta chain to
an ancestor. Geometry, nested clips, hit regions, quads, and prepared glyphs are
translated together. This path reuses the existing Taffy tree and Cosmic Text
shaping. Hover is recomputed under the stationary pointer after movement.

Sibling paint and hit-test order use stable `z_index` ordering. Absolute
elements use Taffy's layout without contributing to flex flow, which is the
basic overlay mechanism. Portals and focus trapping will build on this ordering
rather than creating a renderer-specific widget.

Scrollbar tracks use four independent `Edges` insets. Composite widgets can
therefore reserve any side for resize handles, inline actions, or overlapping
chrome without changing the scroll viewport. Hit testing records the track's
actual position in paint order, so a later sibling painted above it owns the
overlap while the remaining track stays interactive.

`ScrollbarPartStyle` gives the track and thumb their own base `QuadStyle`,
`StylePatch` values, and `StyleTransition`. Hover and thumb drag feed the same
retained transition registry as ordinary elements, so colors, borders, opacity,
corner radii, and gradient properties animate on the paint-only path. The
scrollbar never asks the application to rebuild its tree for an interaction
frame.

`VirtualList` supports fixed and measured variable-height rows. It computes a
visible range plus bounded overscan and represents unseen space with two
lightweight spacers. A logical list of one million rows therefore creates only
tens of elements. Wheel movement uses the translation path inside a stable
overscan chunk; crossing a chunk boundary rebuilds only the bounded visible
window. Scrollbar track/thumb input, anchor correction after measurements, and
programmatic scroll requests all update the same retained offset.
