# Animation

Argui advances typed motions on one runtime frame clock on native platforms
and WebAssembly. A settled motion does not keep requesting frames. A layout
property such as width triggers layout, a background color triggers paint,
and a transform or group opacity can use the retained compositor. The first
composited frame may still need paint to establish its layer; choose the
property for the effect you want, then inspect the presented result.

## TSX: native loops and transitions

The built-in visual primitives expose native loops. The surface and its text
move together here; the moving rectangle still reserves only its original
120 pixel width in layout.

```tsx both
<rectangle width={280} height={56} padding={8} background="#e2e8f0">
  <rectangle width={120} height={40} padding={8} background="#2563eb"
    loopMs={900} loopTranslateX={120} loopPlaying={true}>
    <text color="#ffffff">Moving label</text>
  </rectangle>
</rectangle>
```

`loopMs` is the duration of one leg. Transform targets include
`loopTranslateX`, `loopTranslateY`, and `loopScale`. Other targets include
`loopOpacity`, `loopBackground`, `loopWidth`, `loopRadius`, and `loopGap` on
their supported primitives. Loops alternate to the target and back;
`rotationLoopMs` continuously rotates. `loopPlaying={false}` pauses the
current phase. `loopSteps` makes discrete jump-end steps and is scheduled only
at visible changes. The schema requires a 1–60,000 ms duration and positive
step count. `loopWidth` requires an authored pixel `width`, `loopRadius`
requires `radii`, `loopGap` requires `gap`, and `loopBackground` requires a
solid base background. `rotationLoopMs` cannot be combined with another
transform loop.

For a state change or a changed authored value, use a transition. Hover and
press paint are native on `rectangle`, so JavaScript need not update the tree
for their intermediate frames:

```tsx both
<rectangle width={160} height={44} radii={8}
  background="#e2e8f0" hoverBackground="#bfdbfe"
  transitionMs={160} transitionTimingFunction="cubic-bezier(0.2, 0, 0, 1)">
  <text color="#0f172a">Hover me</text>
</rectangle>
```

`transitionMs` accepts 1–60,000 ms. `transitionTimingFunction` accepts
`linear` or `cubic-bezier(x1, y1, x2, y2)` and requires `transitionMs`.
`transitionSpring` selects the spring driver instead of a timed tween; do
not combine it with `transitionMs` or `transitionTimingFunction`. See the
active [animation scenes](../../apps/gallery/src/solid/animation-page.tsx)
for transform, steps, opacity, color, and width loops.

## Rust: typed property motion

`argui-animation::Motion<T>` retains the presented value and target.
`Element::bind` connects it to a typed `argui-ui::property` selector. Keep the
motion alive across UI rebuilds if the same element should continue its
animation.

```rust
use argui_animation::{Duration, Motion, Tween};
use argui_ui::{Element, property};

let opacity = Motion::new(1.0_f32);
let panel = Element::container([Element::text("Fading panel")])
    .opacity(1.0)
    .bind(property::LayerOpacity, opacity.clone());
opacity.animate_to(0.4, Tween::new(Duration::from_millis(240)));
```

`LayerOpacity` fades the group including descendants. `property::Opacity`
changes the element's painted quad opacity; it is a different property.
Neither one makes invisible content automatically noninteractive or removes
its accessibility semantics. Disable interaction or hide semantics separately
when that behavior is needed.

`Motion::animate_to` retargets from the current presented value.
`Motion::spring_to` also preserves velocity and returns an error for invalid
physics parameters. A direct binding has final precedence over an implicit
style transition for the same property. `Timeline<T>` and `Keyframes<T>` offer
delays, iteration, direction, fill, per-keyframe easing, and holds;
`ScheduleBuilder` composes cues and staggered work. Curves include linear,
cubic Bézier, steps, piecewise linear, and a thread-safe custom function.
Spring, decay, and inertia drivers are separate from easing curves because
they retain velocity.

## What updates

| Property family | Typical update path |
| --- | --- |
| Transform and group opacity | Composition, with hit geometry updated for transforms |
| Solid or compatible gradient paint, border, radius, shadow | Paint or layer update |
| Width, height, minimum/maximum, padding, gap, grow/shrink, pixel insets | Layout |
| Scroll offset | Scroll translation |

Gradient interpolation requires compatible kinds and stop structure;
incompatible values switch discretely. A complete `LayoutStyle` replacement
is also discrete. Reduced-motion preference finishes active property motions
at their targets and prevents subsequent motion from running across frames
while it is active. Hidden retained pages do not schedule their native loops.
Explicit width, gap, and other layout property motions are supported;
insertion, removal, and reordering do not automatically create captured
before/after geometry transitions.

`argui-animation` owns timing and interpolation, `argui-ui` resolves bound
values on stable nodes, and the runtime schedules frames. The renderer does
not own timeline state. The [animation tests](../../crates/argui-ui/tests/tree/animation.rs)
cover compositor, stepped, reduced-motion, and hidden-page scheduling.
Transitions for conditional Rust styles are described in
[styling](styling.md#rust-conditional-styles).
