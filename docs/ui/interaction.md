# Interaction

Winit events enter through `argui-platform`. The runtime converts pointer
coordinates to logical pixels and dispatches renderer-independent events through
the retained UI tree.

```text
winit -> platform -> runtime -> UI dispatch -> model update
                                      |
                               focus and hit state
```

## Identity and dispatch

Every retained element has a stable `NodeId`. Unkeyed nodes preserve identity
at the same structural position; keyed siblings preserve it when reordered.
Duplicate keys are errors.

Listeners are created with `Context::listener` and attached with
`Element::on`. Dispatch runs capture from root to target, target listeners,
then bubbling to root. `stop_propagation`, `stop_immediate_propagation`, and
`prevent_default` act on the shared event control. Passive listeners cannot
prevent defaults.

Hit regions follow paint order and are tested in reverse. A target must pass its
own hit shape and every ancestor clip. Paint and hit-test ordering change
together.

## Pointer and gestures

Mouse, touch, and pen use one `PointerEvent` with contact ID, device kind,
phase, pressure, buttons, primary-contact state, and timestamp.

`capture_pointer` keeps movement and release targeted at one element outside
its bounds. Release, cancellation, or focus loss ends capture and emits
`LostPointerCapture`.

The opt-in gesture arena recognizes tap, pan, pinch, and rotation. `PanGesture`
sets axis, threshold, capture, and delivery:

- immediate delivery suits ranges and direct manipulation;
- frame-coalesced delivery keeps the newest movement sample per frame;
- started, ended, and cancelled phases are never dropped.

Pinch and rotation can run together. Velocity uses a bounded recent history.
Touch can drive scroll containers while the same contacts remain available to
configured gestures.

## Keyboard and focus

Physical key transitions reach the focused node with modifiers and repeat state.
Tab and Shift-Tab traverse the active scope. Keyboard and accessibility
activation produce the same typed click path as pointer activation.

`FocusScope::restoring` restores the previous target.
`FocusScope::trapped` also wraps Tab traversal.
`FocusScope::modal` removes background nodes from active accessibility.
Initial focus can target the first focusable descendant or a stable key.

`Context::request_focus` and `clear_focus` resolve after layout. Missing,
duplicate, disabled, or out-of-scope targets do not receive an implicit fallback.
`FocusVisible` is active for keyboard or programmatic focus, not pointer focus.

Text editing runs after raw key dispatch. `TextInputFilter` applies the same
policy to keyboard, paste, and IME commits. Details are in
[actions and text editing](editing.md).

## Interaction styles

`Interaction` defines behavior. `Element::when` attaches typed
`StylePatch` values for hover, press, focus, named state, and container
conditions. Matching rules compose in declaration order; a later value replaces
only the same property.

`Element::state_scope` creates a control boundary. Descendant hover, focus,
and press can style that scope. A nested scope with the same identity shadows
its ancestor.

`HitTestStyle` is independent of paint:

- `PointerEvents` selects box, descendants, both, or neither;
- `HitShape` selects bounds, rounded rectangle, or ellipse;
- hit slop enlarges a touch target without changing layout or pixels.

Transforms and clips apply before hit testing. Style properties keep their
invalidation class: color updates paint, transforms update paint and hit
geometry, and dimensions update layout. Transitions retain tracks beside stable
node IDs and stop requesting frames when settled.

Container queries are for deliberate presentation changes that Flexbox or Grid
cannot infer. See [responsive styling](styling.md).

## Accessibility

The semantic tree reuses UI node IDs. A semantic-only update does not run layout,
text shaping, paint recording, or WGPU submission.

Native windows use AccessKit. The Web adapter maintains real HTML controls beside
the canvas and patches only changed semantic nodes. Both send actions, focus,
and edited values through the same UI event path.

Modal scopes expose only the modal's ancestor path and subtree, preventing
assistive technology from focusing background controls. Decorative descendants
can use `semantic_hidden(true)` to avoid duplicate announcements.

## System preferences

`SystemPreferences` reports color scheme, reduced motion, and high contrast,
including each value's source. Application overrides win. Reduced motion settles
active motions at their target while preserving the property's invalidation
class.

Run the combined keyboard, modal, accessibility, and multitouch example:

```sh
cargo run -p argui --example accessibility
```
