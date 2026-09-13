# Animation

`argui::animation` provides typed timelines, keyframes, motion and physics on
native and WASM. The runtime supplies one monotonic frame clock; an idle
application requests no animation frames.

## Ownership and scheduling

`argui-animation` owns interpolation and timing without Winit, Taffy or WGPU.
`argui-ui` binds declarations to stable nodes, the runtime schedules frames,
and layout/paint resolve values before the renderer receives drawing commands
and shader uniforms. The renderer owns no timeline state.

`Render::wants_animation_frame` opts a presentation into scheduling;
`Render::animation_frame` receives the shared `Frame`. Retained property motions
use a compact registry built during reconciliation. A shared motion advances
once even when several properties consume it. Settled motions leave scheduling.

Paint changes reuse layout and shaped text. Transforms update painting and hit
geometry without running Taffy. Layout properties invalidate retained layout
nodes, subject to ancestor dependencies and explicit layout boundaries.

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
use argui::{
    animation::{Duration, Motion, Tween},
    ui::{Element, property},
};

let opacity = Motion::new(1.0_f32);
opacity.animate_to(0.4, Tween::new(Duration::from_millis(240)));
let panel = Element::container([]).bind(property::Opacity, opacity);
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
[roadmap](../roadmap.md). The [state showcase](../../crates/argui-showcase/src/)
and [animation tests](../../crates/argui-animation/tests/) exercise the current API.
