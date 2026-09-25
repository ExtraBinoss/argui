# Animation

`argui-animation` provides typed timelines, keyframes, motion and physics on
native and WebAssembly. The runtime supplies one monotonic frame clock; an idle
application requests no animation frames.

`argui-ui` binds animation values to typed properties with `Element::bind`.

## Five-minute start

Create a motion, set its target, and bind it to a retained element property:

```rust
use argui_animation::{Duration, Motion, Tween};
use argui_ui::{Element, property};

let opacity = Motion::new(1.0_f32);
opacity.animate_to(0.4, Tween::new(Duration::from_millis(240)));
let panel = Element::container([])
    .opacity(1.0)
    .bind(property::LayerOpacity, opacity);
```

Retargeting starts from the currently presented value; spring drivers also
preserve velocity. Group opacity changes do not change hit testing or semantics,
so disable interaction or hide semantics separately when invisible content
must be inert.

## Curves

The `curves` module includes `LINEAR`, `EASE_IN`, `EASE_OUT`, `EASE_IN_OUT`,
`STANDARD`, `EMPHASIZED`, `ACCELERATE`, `DECELERATE` and `BACK_OUT`. A custom
curve implements one small trait. Input is normalized; output may overshoot.

```rust
use argui_animation::{Curve, Easing};

struct Anticipate;

impl Curve for Anticipate {
    fn sample(&self, progress: f32) -> f32 {
        progress * progress * (2.7 * progress - 1.7)
    }
}

let easing = Easing::curve(Anticipate);
assert!(easing.sample(0.25) < 0.0);
```

Springs intentionally remain separate from curves. A spring carries velocity
and physical state, whereas a curve only transforms normalized progress.

## What can animate implicitly

| Family | Compatible values | Update cost |
| --- | --- | --- |
| Group opacity and transform | scalar opacity, `Transform2D` components | composite |
| Surface paint | solid colors, border colors/widths, corner radii | paint |
| Gradients | matching kind, points and stop topology | paint |
| Layers | masks, shadow components and typed effect parameters | paint/composite |
| Layout | same-unit width/height/min/max, padding, gap, grow/shrink and pixel insets | layout |
| Scroll | `Point` offsets | scroll |

The first transition to a composited value may require one paint to establish
its layer; subsequent opacity and transform frames remain compositor updates.

Explicit `Element::bind` motions have final precedence over implicit style
transitions. This makes it safe to give one property direct application control
while the remaining container values animate implicitly.

## Gallery example

The Solid/React TSX gallery's **Animation Lab** demonstrates native loops,
implicit transitions, composition, spring retargeting and held keyframes. Its
current source is [`animation-lab.tsx`](../../apps/gallery/src/solid/animation-lab.tsx);
gallery build and hot-reload instructions are in the
[gallery README](../../apps/gallery/README.md).

## Ownership and scheduling

`argui-animation` owns interpolation and timing without Winit, Taffy or WGPU.
`argui-ui` binds declarations to stable nodes, the runtime schedules frames,
and layout/paint resolve values before the renderer receives drawing commands
and shader uniforms. The renderer owns no timeline state.

`Render::wants_animation_frame` opts a presentation into scheduling;
`Render::animation_frame` receives the shared `Frame`. Retained property motions
use a compact registry built during reconciliation. A shared motion advances
once even when several properties consume it. Settled motions leave scheduling.
Held caret keyframes publish their next visible-change deadline instead of
keeping a display-linked loop alive. The native event loop sleeps until that
deadline; ordinary input still wakes it immediately. This preserves a single
ordered animation clock without paying for idle 60 Hz redraws.

Paint changes reuse layout and shaped text. Transform and group-opacity changes
reuse layout, shaped text, paint primitives, and GPU uploads through the retained
compositor while updating hit geometry. Layout properties invalidate retained
layout nodes, subject to ancestor dependencies and explicit layout boundaries.

## Timelines and property motions

`Timeline<T>` owns typed `Keyframes<T>` and `Timing`. It supports play, pause,
resume, reverse, seek, restart, finish, cancel, playback-rate changes and
retargeting from the presented value. Timing includes delays, iterations,
direction and fill modes; keyframes support per-keyframe easing and holds.
Easing includes linear, cubic Bézier, steps, piecewise linear and custom closures.

`Motion<T>` retains its value, target, driver and lifecycle across application
rebuilds. `Element::bind` attaches it to a typed property: transforms, paint,
layout dimensions, scroll offsets, layers, shadows, gradients or effect parameters.
Retargeting starts at the presented value; springs also preserve velocity.

```rust
use argui_animation::{Duration, Motion, Tween};
use argui_ui::{Element, property};

let opacity = Motion::new(1.0_f32);
opacity.animate_to(0.4, Tween::new(Duration::from_millis(240)));
let panel = Element::container([])
    .opacity(1.0)
    .bind(property::LayerOpacity, opacity);
```

`Schedule`, `ScheduleBuilder` and `Cue` compose sequences, parallel groups,
dependencies and staggered timing. Typed `Contribution<T>` values resolve replace,
add and accumulate composition in stable `(priority, order)` order.

## Physics

`Spring<T>` supports scalar, color, point, size and rectangle motion through
`MotionValue`. It solves underdamped, critically damped and overdamped systems
analytically; temporal partitioning does not change the trajectory.

`Decay<T>` integrates exponential velocity decay. Scalar `Inertia` follows that
decay until it crosses an optional bound, then creates a bounce spring from the
same value and velocity. Rest-speed and rest-distance thresholds snap the final
value and stop frame requests. See [scroll](scroll.md) for the consumer policy.

## Style transitions

`StylePatch` is a sparse typed property patch. `StyleTransition` selects a tween
or spring and can override it by `PropertyKey`, state entry or state exit.
Focused, hovered, pressed and disabled states compose deterministically.
Compatible values interpolate; incompatible values, such as linear versus radial
gradients, switch discretely at the midpoint.

The retained transition registry handles interaction changes and authored style
changes across rebuilds. It preserves spring velocity and yields to explicit
`Element::bind` motions as the final composition layer. `StateSelector` targets
the element or the nearest named `StateScopeId`; descendants can respond to a
control's state without depending on its widget recipe. Conditions also combine
state and named container queries with `all`, `any` and `not`.

Reduced motion finishes active property motions at their target and prevents
future motions from running across frames while the preference is active.
See [preferences](interaction.md#system-preferences) and
[conditional styling](styling.md) for environment and rule resolution.

Explicit layout-property animation is supported. Captured before/after geometry
transitions for insertion, removal and reordering remain in the
[roadmap](../roadmap.md). The
[animation tests](../../crates/argui-animation/tests/) exercise the current API.
