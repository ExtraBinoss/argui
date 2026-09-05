# Accessibility and touch

Argui keeps accessibility in a renderer-independent semantic tree. Each
semantic node reuses the retained UI `NodeId`, so focus and assistive-technology
references remain stable across keyed reconciliation. A semantic-only tree
change produces `TreeUpdate::Semantics`: it does not run Taffy, shape text,
record paint commands, or submit a WGPU frame.

Built-in buttons and text inputs publish their role, label, value and actions.
Other elements can attach `Semantics` directly; decorative descendants can use
`semantic_hidden(true)`. Composite controls suppress their visual text children
to avoid duplicate announcements.

Native windows initialize AccessKit from a complete semantic snapshot before
they become visible. AccessKit actions return through the event-loop proxy and
become ordinary `UiEvent`s. The browser adapter maintains real HTML controls
beside each canvas, patches only changed semantic nodes, and routes focus,
clicks and edited values into the same action path.

Modal focus scopes publish an AccessKit modal node on native targets and
`aria-modal` on Web. While a modal is mounted, the semantic snapshot contains
only the window, the ancestor path to that modal, and its subtree, so hidden
background controls cannot receive assistive-technology focus.

## Pointer and gesture contract

`PointerEvent` is shared by mouse, touch and pen. It preserves contact identity,
device kind, phase, pressure, buttons, primary-contact status and a monotonic
timestamp. Winit coordinates are converted to logical UI units once in the
runtime.

`GestureArena` recognizes tap, pan, pinch and rotation. Gesture sets are opt-in
per interactive element. A default pan waits for its configured threshold;
`PanGesture::immediate()` emits `Started` at pointer-down for absolute-position
controls such as ranges. Every pan event carries its current
logical position, delta, total displacement and velocity. Pinch and rotation
can run simultaneously for a contact pair. Velocity uses a bounded 100 ms
history, and cancellation produces explicit terminal events. The primary touch
contact also drags scroll containers directly while all contacts remain
available to the gesture arena.

## System preferences

`SystemPreferences` reports color scheme, reduced motion, and high contrast with the
source of each value. Application overrides win over system values. Native
detection runs outside startup and frame processing; Web uses media queries.
Reduced motion finishes active property motions at their typed target and keeps
future motions from running across frames while the preference remains active.
The final change retains its exact paint, scroll, or layout invalidation class.
High contrast is delivered to the application so its theme can choose the
appropriate palette.

Run the combined keyboard, modal-focus, screen-reader and multitouch laboratory
with:

```sh
cargo run -p argui --example accessibility
```
