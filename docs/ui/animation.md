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

## Animated text

Enable `widget-animated-text` on `argui` (or `animated-text` on `argui-widgets`).
`AnimatedText` animates only changing Unicode graphemes: `10 → 11` keeps the
first digit still. `TextAnimation::Roll` passes through intermediate digits,
`Slide` moves directly to the next character, and `Fade` crossfades both
characters at the same baseline, without a blank midpoint.
The **Animated text** gallery page demonstrates all three, carries and reversals.

```rust
use argui::{runtime::Entity, widgets::AnimatedText};

let count = Entity::new(AnimatedText::new("count", "10").font_size(32.0));
// In a parent Render implementation: cx.entity(&count)
count.update(|text, cx| {
    text.set_text("11");
    cx.notify();
});
```

The default duration is 420 ms with cubic ease-out and right-aligned character
positions; use `align_end(false)` for labels. `text_style` replaces the default
theme-colored monospace style; `font_size` sizes that default style. The widget
is single-line and aligns grapheme positions, without parsing locale-specific
number formatting. Updates during a transition coalesce into the latest target
for the following transition; toggling a fade back reverses its current opacity
immediately. Mounted entities honor reduced motion automatically;
manual hosts can use `set_reduced_motion`, `advance` and `build`.

Digit substitutions reuse layout; appearing, disappearing and nonnumeric columns
interpolate their measured widths during the transition. This prevents the final
horizontal jump on `100 → 99` and accommodates proportional status labels.
Fades change text alpha directly without allocating effect layers per glyph.
Unchanged columns are shared, completed reels are released, and idle entities
request no frames. One accessible text node exposes the requested value; intermediate
digits are hidden. `unicode-segmentation` is an optional dependency owned by
this widget to keep combining characters and emoji intact.

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
[roadmap](../roadmap.md). The Widget Gallery's
[Motion Lab](../../crates/argui-widget-gallery/src/pages/motion.rs) and
[animation tests](../../crates/argui-animation/tests/) exercise the current API.
