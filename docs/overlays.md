# Overlay geometry

Overlay placement is renderer-independent and uses logical pixels. The runtime
passes a `LayoutSnapshot` to `UiApp::layout_changed` after Taffy finishes. It
contains the canvas `viewport` and bounds addressable by stable element key:

```rust
fn layout_changed(&mut self, layout: &LayoutSnapshot) -> ViewUpdate {
    let Some(anchor) = layout.bounds("menu-button") else {
        return ViewUpdate::None;
    };
    let placed = OverlayPlacement::new(PlacementSide::Bottom)
        .align(OverlayAlign::End)
        .gap(8.0)
        .margin(12.0)
        .place(layout.viewport, anchor, Size::new(360.0, 480.0));
    if self.menu == Some(placed) {
        return ViewUpdate::None;
    }
    self.menu = Some(placed);
    ViewUpdate::Rebuild
}
```

The calculator tries the preferred side, its opposite, then perpendicular
sides. It clamps cross-axis alignment to the viewport. `PlacedOverlay` contains
the selected side, final bounds, and `max_size`; use the latter to constrain a
scrollable menu or popover when its desired content cannot fit.

Returning `Rebuild` permits one bounded second layout pass. This avoids layout
loops while allowing the selected side and maximum content size to affect the
Rust tree. The same mechanism is a direct lowering target for a future DSL.

Pointer occlusion remains explicit. Apply `Interaction::blocker()` to a popover
surface when it must prevent hover/click-through. Leave it absent for visual
overlays such as passive tooltips that intentionally behave like
`pointer-events: none`.

Scrollable overlays normally use `ScrollChaining::Contain`. At a content edge,
the wheel or touchpad gesture is then consumed instead of scrolling an ancestor
behind the overlay. The default `Auto` policy preserves normal nested chaining.
