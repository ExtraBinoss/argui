# Overlays and tooltips

`Popover` presents controlled content anchored to a trigger. Escape and outside
click dismiss it. Events inside nested popovers remain inside; handle the child
before its parent and clear child state when the parent closes.

`TooltipState` opens from hover after 350 ms or immediately from keyboard focus.
It waits 100 ms after pointer exit so the pointer can enter the tooltip. Escape
and trigger activation close it without changing focus.

Forward relevant events to `TooltipState::update`, schedule `advance` at
`next_deadline`, and rebuild when state changes. Reset the state and cancel its
timer when the trigger unmounts. No polling is required.

## Automatic button help

Enable the tooltip widget and wrap the application in `TooltipHost`:

```rust,ignore
let app = TooltipHost::new(MyApplication::default());
let save = Button::new("save", "Save", theme.button())
    .tooltip("Save the current draft")
    .build();
let cancel = Button::new("cancel", "Cancel", theme.outline_button())
    .without_tooltip()
    .build();
```

Buttons use their label as default help. Disabled or busy buttons do not open a
tooltip. A controlled `Tooltip` suppresses the host tooltip for its trigger.
The host dismisses help when a menu, popover, or dialog opens.

`TooltipHost::delay`, `paint`, and `layer` customize the shared behavior.
With text selection, compose `TooltipHost::new(SelectionHost::new(app))`.

## Surfaces and effects

`Popover::layer` and `Tooltip::layer` accept `LayerStyle`: filters, backdrop
filters, mask, shadows, and opacity. Widgets do not register effect presets.

```rust,ignore
let layer = theme.overlay_layer(8.0, 0.0)
    .backdrop(argui_effects::Blur(6.0).filter());
let popover = Popover::new("settings", "Settings", open, trigger, content)
    .paint(PaintStyle::new(QuadStyle::solid(
        theme.popover.with_alpha(0.82),
    )))
    .layer(layer)
    .build(&theme);
```

Backdrop blur needs translucent paint to remain visible. Keep enough opacity for
text contrast in both themes. Calling `layer` replaces the complete layer,
including default shadows. Custom WGSL effects use the same filter API; see
[GPU effects](../rendering/effects.md).

## Feature selection

```toml
argui = { version = "0.3.1", features = [
  "widget-popover",
  "widget-tooltip",
] }
```

`widgets-all` enables the general collection; the updater dialog stays
separate. Composed widgets enable their required building blocks. The facade
uses `widget-*`; direct `argui-widgets` features omit that prefix.

`python3 scripts/check-widget-features.py` checks isolated and full feature
sets.

## Enter and exit

`Presence` is shared by selection toolbars, Select, and Popover. Keep it in the
owning model, call `set_open(open, reduced_motion)`, and pass it to the widget.
It is the source of truth for whether exiting content remains mounted.

Advance it once per application animation callback. Request paint while it
moves; rebuild when exit completes; return `presence.animating()` from
`wants_animation_frame`. Exiting content has no pointer targets, focus scope,
enabled controls, or semantic exposure. Reduced motion settles it immediately.

## Placement

Placement uses logical pixels and final layout bounds:

```rust,ignore
fn layout_changed(&mut self, layout: &LayoutSnapshot, cx: &mut Context<Self>) {
    let Some(anchor) = layout.bounds("menu-button") else {
        return;
    };
    let placed = OverlayPlacement::new(PlacementSide::Bottom)
        .align(OverlayAlign::End)
        .gap(8.0)
        .margin(12.0)
        .place(layout.viewport, anchor, Size::new(360.0, 480.0));
    if self.menu != Some(placed) {
        self.menu = Some(placed);
        cx.notify();
    }
}
```

The calculator tries the preferred side, its opposite, then perpendicular sides,
and clamps cross-axis alignment to the viewport. `PlacedOverlay::max_size`
constrains content that cannot fit. The runtime permits one bounded second
layout pass for this feedback.

Use `Interaction::blocker()` when a surface must stop click-through. Passive
visual overlays can omit it. Scrollable overlays normally use
`ScrollChaining::Contain` so an edge gesture does not scroll content behind.

Native surfaces outside the parent window have separate support and fallback
rules in [native popovers](../platform/native-popovers.md).
