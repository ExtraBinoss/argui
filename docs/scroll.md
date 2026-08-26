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

`VirtualList` currently targets fixed-height rows. It computes a visible range
plus bounded overscan and represents the unseen space with two lightweight
spacers. A logical list of one million rows therefore creates only tens of
elements. Wheel movement uses the translation path inside a stable overscan
chunk; crossing a chunk boundary rebuilds only the bounded visible window. Variable-height rows,
scrollbar thumbs, anchor correction, and programmatic `scroll_to` are later
extensions of the same range model.
