# Argui animation mental model

Read this when authoring motion, gradients, loaders, or an animation performance
fix. The public TSX tree describes **targets and interactions**. Native state
owns the animation between targets.

```text
Solid / React event or Rust application update
  → schema-typed element description
  → retained UiTree identity + transition / motion registry
  → runtime samples one monotonic clock and coalesces frame requests
  → layout, scroll, paint, or composition update
  → renderer submits reused or changed GPU data
```

An `onClick` may change a signal once. Do not run a JavaScript timer to write a
new transform, opacity, gradient, or list of dots every frame. Use native state
transitions, loops, or a typed `Motion` bound to a stable element. Keep a surface,
its icon, and its label under the same moving compositor ancestor so they cannot
drift apart during a press or pulse. Stable keys and IDs let retained nodes keep
their motion phase and cache identity across application updates.

## Pick the update path

| Animated value | Typical path | What changes |
| --- | --- | --- |
| Transform or group opacity | Composition | Retained layer transform/opacity and interaction geometry; painted content and shaped text are reused. |
| Background, gradient, border, radius, shadow | Paint | Primitive data changes; layout and shaped text are reused. A static gradient inside a moving compositor layer can instead reuse its paint. |
| Scroll offset | Scroll | Native geometry translates without Taffy layout or text reshaping in the usual case. |
| Width, gap, padding, grow | Layout | Size and dependent descendants are recomputed. Use when reflow is the intended effect. |

The first composited frame may paint once to establish a retained layer.
Clipping that exposes unpainted pixels, singular transforms, resize, or an
ancestor effect can require a safe paint fallback. A gradient is GPU shaded,
but changing its stops every frame is still a paint update. GPU execution alone
does not guarantee a smooth frame; count CPU preparation, GPU passes, uploads,
and damaged pixels when evaluating a design.

## Expressive surfaces

- Put a static linear or radial gradient on a bounded rectangle and animate
  that rectangle's transform or group opacity for a moving glow. Clip the
  effect within a stable rounded parent; give the glow enough room that its
  motion does not reveal unpainted edges.
- Keep spinner dots or a ring as stable children of one rotating element. A
  pulse can animate scale and opacity on that element. Reserve its layout size
  once; use layout animation only if nearby content should actually move.
- A click should update application state only at the interaction boundary.
  Native easing or spring motion then runs without rebuilding the TSX tree on
  every frame. Pause or stop loops when their example is inactive.
- Prefer OKLab interpolation and premultiplied alpha for multicolor gradients;
  transparent stops must retain their hue to avoid dark fringes. Use a small,
  bounded number of gradient layers and stops.
- Give active controls a semantic label and keep visual motion consistent with
  their focus and pressed states. Reduced motion must finish at a meaningful
  state instead of leaving a loader half visible.

## Verify the real path

Write a focused test for the native update class or frame scheduling when a
change claims to avoid layout or paint. In the gallery, mount both Solid and
React through the headless host. For visual checks, follow the repository's
private-display procedure or a user's stricter instruction. For a performance
claim, use the inspector or traces to compare frame time, update class, GPU
time, damaged pixels, and idle frame requests on the actual target. A unit test
proving `Composite` is useful but does not measure browser smoothness.

Further details: [animation API](../../../docs/ui/animation.md),
[retained compositor](../../../docs/rendering/compositor.md),
[gradient/color model](../../../docs/rendering/primitives.md), and
[pipeline audit](../../../docs/rendering/animation-performance-review.md).
