# Animation

`argui::animation` provides typed timelines, keyframes, motion and physics on
native and WASM. The runtime supplies one monotonic frame clock; an idle
application requests no animation frames.

Argui exposes four layers. Start with the first one and move down only when the
interaction needs more control:

1. `AnimatedOpacity` and `AnimatedContainer` animate target changes implicitly.
2. `Motion<T>` gives application code pause, retarget and spring control.
3. `Timeline<T>` and `Keyframes<T>` describe multi-stage playback.
4. `Spring`, `Decay`, `Inertia`, schedules and composition build physical or
   coordinated systems.

## Five-minute start

Enable `widget-implicit-animation` on the `argui` facade, or
`implicit-animation` when depending on `argui-widgets` directly.

```toml
argui = { version = "0.3", features = ["widget-implicit-animation"] }
```

### Fade a complete subtree

Rebuild the same keyed element with a different target opacity. The first build
is immediate; every later change begins at the value currently on screen.

```rust
use argui::{
    animation::{Duration, curves},
    ui::Element,
    widgets::AnimatedOpacity,
};

fn details(visible: bool) -> Element {
    AnimatedOpacity::new(
        "details-fade",
        if visible { 1.0 } else { 0.0 },
        Element::text("Saved locally"),
    )
    .duration(Duration::from_millis(220))
    .curve(curves::EASE_OUT)
    .build()
}
```

`AnimatedOpacity` uses group opacity, so the element and all descendants fade
together. Opacity does not change hit testing or semantics. Disable interaction
or hide semantics explicitly when invisible content must be inert.

### Animate a container

Compatible values animate together. This example changes layout, paint and
corner geometry without maintaining an animation controller in the model.

```rust
use argui::{
    animation::{Duration, curves},
    core::Color,
    paint::CornerRadii,
    ui::{Element, length},
    widgets::AnimatedContainer,
};

fn project_card(expanded: bool) -> Element {
    AnimatedContainer::new("project-card", [Element::text("Argui")])
        .width(length(if expanded { 320.0 } else { 180.0 }))
        .height(length(if expanded { 150.0 } else { 72.0 }))
        .background(if expanded {
            Color::srgb(0.55, 0.30, 0.96)
        } else {
            Color::srgb(0.18, 0.48, 0.98)
        })
        .radius(CornerRadii::all(if expanded { 28.0 } else { 12.0 }))
        .duration(Duration::from_millis(420))
        .curve(curves::EMPHASIZED)
        .build()
}
```

Pixels interpolate with pixels and percentages with percentages. Switching
between incompatible units, gradient kinds or other discrete representations
snaps safely instead of inventing an ambiguous interpolation. Use
`AnimatedContainer::from_element` or `configure` when row, column, grid or
advanced element configuration is required.

## Curves

The `curves` module includes `LINEAR`, `EASE_IN`, `EASE_OUT`, `EASE_IN_OUT`,
`STANDARD`, `EMPHASIZED`, `ACCELERATE`, `DECELERATE` and `BACK_OUT`. A custom
curve implements one small trait. Input is normalized; output may overshoot.

```rust
use argui::{
    animation::{Curve, Easing},
    ui::Element,
    widgets::AnimatedOpacity,
};

struct Anticipate;

impl Curve for Anticipate {
    fn sample(&self, progress: f32) -> f32 {
        progress * progress * (2.7 * progress - 1.7)
    }
}

let easing = Easing::curve(Anticipate);
assert!(easing.sample(0.25) < 0.0);

let _element = AnimatedOpacity::new("notice", 1.0, Element::text("Ready"))
    .curve(Anticipate)
    .build();
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

## Gallery cookbook

The Widget Gallery's **Animation laboratory** contains twenty live examples:

| Group | Examples |
| --- | --- |
| Implicit | subtree opacity; size; color and radius; padding and gap; composed transform; border and shadow; interrupted retargeting |
| Keyframes | typed multi-property morph; multiple stops; holds; steps; alternate direction; stagger schedule |
| Physics | analytical spring; bounded inertia; velocity-preserving retarget; squash and stretch |
| Composition | additive tracks; a user-defined `Curve`; synchronized layer, glow and compositor properties |

Use **Run all animations** repeatedly while motion is active to see every
retarget continue from its presented value. Enabling the platform reduced-motion
preference snaps all twenty examples to their destination and returns the
application to an idle frame schedule.

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
use argui::{
    animation::{Duration, Motion, Tween},
    ui::{Element, property},
};

let opacity = Motion::new(1.0_f32);
opacity.animate_to(0.4, Tween::new(Duration::from_millis(240)));
let panel = Element::container([])
    .opacity(1.0)
    .bind(property::LayerOpacity, opacity);
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
