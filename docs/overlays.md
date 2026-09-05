# Overlay geometry

## Retained enter and exit animations

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

## Placement

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
