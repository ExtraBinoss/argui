# Overlays and tooltips

`Popover` presents controlled anchored content. Its trigger toggles the panel;
Escape and outside clicks dismiss it. Clicks on the title, description, padding
or nested popovers remain inside the panel. Handle nested popover events before
their parent and reset child state when the parent closes.

`TooltipState` provides hover and keyboard help. Its default hover delay is
350 ms, with 100 ms before closing after pointer exit. Moving into the tooltip
keeps it open; keyboard focus opens it immediately. Escape and button activation
close it without moving focus or consuming the button's action. A later hover
can reopen it while the trigger keeps focus. Content has the Tooltip role and
is connected with `described_by`.

Forward events to `TooltipState::update`, including Escape elsewhere in the
window while a tooltip is open. Schedule `advance` at `next_deadline` and rebuild
when state changes. The state itself requires no polling or task runtime. Reset
it and cancel its timer when the trigger is removed or its page is hidden.
The [gallery example](../../crates/argui-widget-gallery/src/pages/tooltip.rs)
uses a `TaskSlot` for owned, cancellable timers.

## Automatic button tooltips

Buttons declare their label as a tooltip by default. Enable the `tooltip`
feature and wrap the application in `TooltipHost` to display them automatically.
The `button` feature remains usable on its own.

```rust
let application = TooltipHost::new(MyApplication::default());
let button = Button::new("save", "Save", theme.button())
    .tooltip("Save the current draft")
    .build();
let quiet = Button::new("cancel", "Cancel", theme.outline_button())
    .without_tooltip()
    .build();
```

`tooltip` includes the controlled widget, state and host, enabling runtime tasks
for deadline scheduling. Custom elements can declare `Element::tooltip(...)`.
Disabled or busy buttons do not open automatic tooltips. A controlled Tooltip
owns its trigger and does not receive a second host tooltip.

The host dismisses help when a menu, popover or dialog appears, including an
opening initiated by code. Empty layers and notifications do not block tooltips.
Customize automatic help through `TooltipHost::delay`, `paint` and `layer`.
With text selection, use `TooltipHost::new(SelectionHost::new(application))`.

## Surfaces and effects

`Popover::layer` and `Tooltip::layer` accept a full `LayerStyle`: content and
backdrop filters, masks, shadows and opacity. Widgets do not depend on effect
presets. Set a translucent paint to reveal a backdrop filter; opaque paint hides
it. `.layer(...)` replaces the complete layer, including default shadows.

```rust
let layer = theme.overlay_layer(8.0, 0.0)
    .backdrop(argui_effects::Blur(6.0).filter());
let popover = Popover::new("settings", "Settings", open, trigger, content)
    .paint(PaintStyle::new(QuadStyle::solid(theme.popover.with_alpha(opacity))))
    .layer(layer)
    .build(&theme);
```

Blur strength and tint are independent. `Blur(6.0)` controls filtering;
`QuadStyle::solid(...)` controls color, and `with_alpha` controls opacity from
transparent `0.0` to opaque `1.0`. Choose enough opacity to keep foreground text
readable in either theme.

Ordinary menus, popovers, dialogs, toasts and tooltips share `theme.popover`,
`theme.popover_border` and `theme.overlay_shadows`. Transparent effects are local
customizations. Custom WGSL effects use the same filter API as presets:

```rust
let filter = Filter::Effect(EffectInstance::new(
    MY_EFFECT,
    [("strength", EffectValue::F32(0.4))],
));
let layer = theme.overlay_layer(8.0, 0.0).backdrop(filter);
let tooltip = Tooltip::new("help", "Preview your changes", open, trigger)
    .layer(layer)
    .build(&theme);
```

Register the definition before launch with
`RendererConfig::effects(argui_effects::registry()?.with_definition(definition)?)`.
See [GPU effects](../rendering/effects.md) and the
[complete example](../../crates/argui-widget-gallery/src/pages/overlay_effects.rs).

## Independent features

`argui` and `argui-widgets` use `default = []`. Select widgets explicitly:

```toml
argui = { git = "https://github.com/ExtraBinoss/argui", features = ["widget-popover", "widget-tooltip"] }
```

`argui/widgets-all` and `argui-widgets/all` enable the general widget collection;
the updater dialog remains separately opt-in. Composed widgets enable their
building blocks: ContextMenu and Menubar use Menu. TextArea has its own
`textarea` feature, independent of Input. The facade uses the `widget-` prefix.

`python3 scripts/check-widget-features.py` checks each feature from an external
consumer and then checks empty/full configurations. The browser scenario
`crates/argui-widget-gallery/tests/pages/overlay_effects.mjs` covers themes,
surface effects, editing, dismissal, timing, focus and accessible relationships.
Run it through the [private Linux display](../contributing/linux-testing.md).

## Overlay geometry

For the optional cross-platform native popup API, automatic overflow, backend
support and web fallback, see [native-popovers.md](../platform/native-popovers.md).

### Retained enter and exit animations

`argui_widgets::Presence` is shared by selection toolbars, Select and Popover.
Keep it in the owning component, call `set_open(open, reduced_motion)` on state
changes, and pass `.presence(&presence)` to the widget builder. Presence is the
source of truth for its open state; do not combine it with a separate `.open()`.

Advance it once in the owner's animation callback with `advance(frame.elapsed)`.
A `true` result means the exit completed and requires rebuilding to unmount;
otherwise request paint only. Return `presence.animating()` from
`wants_animation_frame()`. The gallery's backend Select and the devtools dock
Select implement this pattern. Reduced motion should also finish presence when
the environment changes, not only when the menu opens.

The shared timings are 140 ms in and 100 ms out, with opacity and a 4 px
translation. Exiting overlays are retained visually but have no pointer targets,
focus scope, enabled controls or accessibility exposure. Reopening reverses from
the current progress. No animation frame is required once settled.

### Placement

Overlay placement is renderer-independent and uses logical pixels. The runtime
passes a `LayoutSnapshot` to `Render::layout_changed` after Taffy finishes. It
contains the canvas `viewport` and bounds addressable by stable element key:

```rust
fn layout_changed(&mut self, layout: &LayoutSnapshot, cx: &mut Context<Self>) {
    let Some(anchor) = layout.bounds("menu-button") else {
        return;
    };
    let placed = OverlayPlacement::new(PlacementSide::Bottom)
        .align(OverlayAlign::End)
        .gap(8.0)
        .margin(12.0)
        .place(layout.viewport, anchor, Size::new(360.0, 480.0));
    if self.menu == Some(placed) {
        return;
    }
    self.menu = Some(placed);
    cx.notify();
}
```

The calculator tries the preferred side, its opposite, then perpendicular
sides. It clamps cross-axis alignment to the viewport. `PlacedOverlay` contains
the selected side, final bounds, and `max_size`; use the latter to constrain a
scrollable menu or popover when its desired content cannot fit.

Calling `Context::notify()` permits one bounded second layout pass. This avoids layout
loops while allowing the selected side and maximum content size to affect the
Rust tree. The same mechanism is a direct lowering target for a future DSL.

Pointer occlusion remains explicit. Apply `Interaction::blocker()` to a popover
surface when it must prevent hover/click-through. Leave it absent for visual
overlays such as passive tooltips that intentionally behave like
`pointer-events: none`.

Scrollable overlays normally use `ScrollChaining::Contain`. At a content edge,
the wheel or touchpad gesture is then consumed instead of scrolling an ancestor
behind the overlay. The default `Auto` policy preserves normal nested chaining.
