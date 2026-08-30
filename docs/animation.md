# Animation plan

Argui will provide one typed animation engine for native and WASM. It must be
complete without imposing CSS semantics or coupling the renderer to UI state.
An idle application performs no animation work and requests no frames.

## Ownership

```text
argui-animation
  time, timelines, keyframes, easing, springs, interpolation
          ↓
argui-ui
  animation declarations attached to stable NodeIds
          ↓
argui-runtime
  monotonic clock, scheduler, frame requests, invalidation
          ↓
argui-layout / argui-paint
  resolved geometry and visual values
          ↓
argui-render
  display data and shader uniforms, with no animation state
```

`argui-animation` will remain independent from Winit, Taffy, WGPU, and the
future DSL. The DSL will lower into the same typed declarations as Rust code.

## Feature scope

- Explicit timelines and retained typed property motions.
- Typed keyframes with offsets, per-keyframe easing, and optional holds.
- Duration, delay, end delay, playback rate, pause, resume, reverse, seek,
  restart, finish, and cancel.
- Finite or infinite iterations; normal, reverse, alternate, and
  alternate-reverse directions; none, forwards, backwards, and both fill modes.
- Linear, cubic Bézier, steps, piecewise-linear, and user-defined easing.
- Springs, decay, inertia, initial velocity, and retargeting without jumps.
- Replace, add, and accumulate composition.
- Serial sequences, parallel groups, dependencies, and staggered children.
- Start, iteration, finish, and cancel events.
- Animatable scalar values, colors, points, sizes, rectangles, edge values,
  corner radii, opacity, borders, and transform components.
- Translate, rotate, scale, skew, transform origins, and composed transforms.
- Paint-only, transform-only, and layout-invalidating animation classes.
- Layout transitions using captured before/after geometry, without changing
  Taffy's ownership of final layout.
- Caret blinking, scroll animation, momentum, overscroll policy, and decay.
- Filter parameters, custom-effect parameters, and stable custom-WGSL uniforms.
- Configurable reduced-motion policy on native and web.

## Performance rules

- Sample the monotonic clock once per frame, not once per animation.
- Keep active animations in a compact scheduler; do not scan the complete UI
  tree each frame.
- Request the next frame only while a timeline, spring, caret timer, or inertial
  motion is active.
- Classify dirty work precisely: paint values do not invoke Taffy or reshape
  text; transforms do not rebuild geometry; layout values invalidate only the
  affected subtree.
- Reuse typed animation storage and bounded temporary buffers. No per-frame
  string parsing, shader compilation, or unbounded cache growth.
- Resolve animation values before display-list generation. WGPU receives final
  values and may update shader uniforms without knowing about timelines.
- Coalesce input-driven retargeting to the presentation frame.

## Delivery stages

### 1. Clock and scheduler — implemented

- Add `argui-animation` with `Time`, `Duration`, a deterministic manual clock,
  typed interpolation, and a compact active-timeline scheduler.
- Connect native and WASM monotonic clocks in `argui-runtime`.
- Prove zero redraw while idle and exact deterministic sampling in tests.

### 2. Timelines and keyframes — implemented

- Add keyframes, property tracks, timing parameters, fill/direction/iteration
  behavior, playback controls, and lifecycle events.
- Implement linear, Bézier, steps, and piecewise-linear easing.
- Support interruption and retargeting without discontinuities.

The public API is available from `argui::animation`. A `Timeline<T>` owns typed
`Keyframes<T>` and a `Timing`; it can play, pause, resume, reverse, seek,
restart, finish, cancel, change playback rate, or retarget from its currently
presented value. Custom easing closures remain platform-independent.

`Render::wants_animation_frame` activates runtime scheduling and
`Render::animation_frame` receives the one shared `Frame` sampled for that
presentation. Returning to an inactive timeline removes it from the compact
scheduler immediately. The state showcase exercises this exact path on native
and WASM without separate UI code.

### 3. Property motions and orchestration — implemented

- Bind retained typed motions to composed UI elements.
- Add sequence, parallel, dependency, and stagger orchestration.
- Add replace/add/accumulate composition with deterministic priority rules.

`Motion<T>` owns the presented value, target, driver and lifecycle independently
from application rebuilds. `Element::bind` is the single attachment API; typed
property markers select transforms, paint components, layout dimensions,
scroll offsets, layer/shadow values, gradient geometry/stops, or registered
effect parameters. A target changed during playback starts again from the
currently presented value, while spring retargeting preserves velocity.

The UI tree builds one compact registry when it reconciles. A shared motion is
advanced once even when several properties consume it, static trees allocate no
per-frame animation work, and each changed track emits its exact paint, scroll,
or layout invalidation class.

`Schedule`, `ScheduleBuilder`, and `Cue` describe serial, parallel, dependent,
and staggered timing without owning widget or renderer state. Typed
`Contribution<T>` values resolve replace, add, and accumulate composition in a
stable `(priority, order)` order. Transform, layout, paint, scroll, and effect
properties reuse these primitives.

```rust
use argui::{
    animation::{Duration, Motion, Tween},
    ui::{Element, property},
};

let opacity = Motion::new(1.0_f32);
opacity.animate_to(0.4, Tween::new(Duration::from_millis(240)));
let panel = Element::container([])
    .background(target_color)
    .bind(property::Opacity, opacity);
```

### 4. Physics — implemented

- Add springs, decay, inertia, velocity preservation, and bounded settling.
- Use the same physics for interactive controls and optional scroll momentum.
- Stop frame requests immediately after the configured rest thresholds.

`Spring<T>` supports scalar, color, point, size, and rectangle motion through
`MotionValue`. It solves underdamped, critically damped, and overdamped systems
analytically, so a long frame remains stable and temporal partitioning does not
change the trajectory. Retargeting preserves the current velocity.

`Decay<T>` integrates exponential velocity decay. Scalar `Inertia` uses that
decay until it crosses an optional bound, then creates a bounce spring from the
same presented value and velocity. Configurable rest-speed and rest-distance
thresholds snap the final value and immediately remove the physics consumer
from frame scheduling. The same inertia is ready for the scroll consumer stage;
it is not coupled to scroll direction or platform events.

### 5. Paint and transforms — implemented

- Animate colors, opacity, borders, radii, and transform components through the
  paint-only path.
- Add transform-aware painting, clipping, and hit testing.
- Batch compatible animated primitives exactly like static primitives.

### 6. Layout properties — implemented

- Animate explicit layout properties with scoped Taffy invalidation.
- Keep hit testing synchronized with the presented geometry.

Before/after geometry animation for reorders, insertion, and removal remains a
separate future feature; explicit animated layout properties already update
their existing Taffy nodes without rebuilding the retained tree.

### 7. Built-in consumers — implemented

- Implement caret blink without a permanent redraw loop.
- Add animated hover, press, focus, scroll-to, momentum, and overlay entry/exit.
- Expose reduced-motion behavior as application policy rather than a theme
  assumption.

`StateStyle` is a sparse typed property patch shared by Rust builders and any
future DSL. `StyleTransition` selects a tween or spring globally and can
override it by `PropertyKey`, state entry, or state exit. Focused, hovered,
pressed, and disabled states compose deterministically. Incompatible values,
such as changing a linear gradient into a radial gradient, switch discretely at
the transition midpoint; compatible scalar, color, geometry, transform, layer,
shadow, scroll, layout, and custom-effect values interpolate.

The retained transition registry is keyed by `NodeId`. It handles both
interaction changes and authored style changes across application rebuilds,
preserves spring velocity on retarget, and yields to explicit `Element::bind`
motions as the final composition layer. Descendant state inheritance is
explicit through `Element::inherit_interaction_state`.

### 8. Effects and shader parameters — implemented

- Connect motions to layer opacity, shadows, masks, and registered
  custom-effect uniforms.
- Preserve the effects boundary: `argui-paint` describes values and
  `argui-render` executes the necessary WGPU passes.

### 9. Showcase and profiling

- Use one shared Rust showcase on native and WASM for keyframes, motions,
  springs, interruption, layout changes, scroll, caret, and shader parameters.
- Measure frame CPU time, allocations, active/idle redraws, GPU passes, and
  steady-state memory.
- Keep every coverage metric at or above 85% and every Rust file within the
  600-line limit.

Each stage ends with its focused tests, the complete quality gate, a native
showcase check, and a real `wasm-pack` build before the next stage starts.
