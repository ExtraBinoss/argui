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

- Explicit timelines and implicit property transitions.
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

`UiApp::wants_animation_frame` activates runtime scheduling and
`UiApp::animation_frame` receives the one shared `Frame` sampled for that
presentation. Returning to an inactive timeline removes it from the compact
scheduler immediately. The state showcase exercises this exact path on native
and WASM without separate UI code.

### 3. Transitions and orchestration

- Add ergonomic implicit transitions on composed UI elements.
- Add sequence, parallel, dependency, and stagger orchestration.
- Add replace/add/accumulate composition with deterministic priority rules.

### 4. Physics

- Add springs, decay, inertia, velocity preservation, and bounded settling.
- Use the same physics for interactive controls and optional scroll momentum.
- Stop frame requests immediately after the configured rest thresholds.

### 5. Paint and transforms

- Animate colors, opacity, borders, radii, and transform components through the
  paint-only path.
- Add transform-aware painting, clipping, and hit testing.
- Batch compatible animated primitives exactly like static primitives.

### 6. Layout transitions

- Animate explicit layout properties with scoped Taffy invalidation.
- Add before/after geometry transitions for reorders, insertion, removal, and
  responsive state changes.
- Keep hit testing synchronized with the presented geometry.

### 7. Built-in consumers

- Implement caret blink without a permanent redraw loop.
- Add animated hover, press, focus, scroll-to, momentum, and overlay entry/exit.
- Expose reduced-motion behavior as application policy rather than a theme
  assumption.

### 8. Effects and shader parameters

- Connect timelines to layer opacity, filters, backdrop effects, and registered
  custom-effect uniforms.
- Preserve the effects boundary: `argui-paint` describes values and
  `argui-render` executes the necessary WGPU passes.

### 9. Showcase and profiling

- Use one shared Rust showcase on native and WASM for keyframes, transitions,
  springs, interruption, layout changes, scroll, caret, and shader parameters.
- Measure frame CPU time, allocations, active/idle redraws, GPU passes, and
  steady-state memory.
- Keep every coverage metric at or above 85% and every Rust file within the
  600-line limit.

Each stage ends with its focused tests, the complete quality gate, a native
showcase check, and a real `wasm-pack` build before the next stage starts.
