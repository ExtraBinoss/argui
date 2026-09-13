# Interaction model

Winit events are translated into renderer-independent platform events. The
runtime converts physical pointer coordinates to logical UI coordinates, then
`argui-ui` dispatches them against the latest clipped Taffy geometry.

```text
winit -> argui-platform -> argui-runtime -> argui-ui
                                              |
                                     hit test and state
                                              |
                              repaint display list only
```

Every retained element receives a stable `NodeId`. Unkeyed nodes retain their ID
at the same structural position; keyed siblings retain it when reordered. IDs
are never derived from hashes, and duplicate keys never produce duplicate IDs.
Application-facing `UiEvent`s distinguish the original target from the current
delivery target and expose both stable keys.

Listeners are declared while rendering with `Context::listener` and attached to
any `Element` with `Element::on`. Dispatch follows the DOM order: capture from
the root to the target, target listeners, then bubbling back to the root.
`stop_propagation`, `stop_immediate_propagation`, and `prevent_default` operate
on the shared event control. Passive listeners cannot prevent defaults, and
once listeners remain consumed across retained rebuilds.

Hit regions are emitted in paint-tree order and tested in reverse, so the last
interactive element painted wins. Bounds and the resolved rectangular ancestor
clip must both contain the pointer. Future z-index/layer work must reorder paint
and hit-test data together.

Pointer capture is explicit. A listener can call `Context::capture_pointer` and
`Context::release_pointer`, while a configured gesture can request
`GestureCapture::OnPress`. Captured movement and release stay targeted at the
capturing element even when hover moves elsewhere. Release, cancellation, and
window focus loss always terminate capture and emit `LostPointerCapture`.
Focusable controls receive focus on press, and the runtime restores the exact
retained focus when the window becomes active again.

Tap, pan, pinch, and rotation are typed policies rather than widget behavior.
`PanGesture` configures axis, threshold, capture, and delivery. Immediate
delivery is appropriate for direct manipulation; frame-coalesced delivery keeps
only the newest changed sample until the next presentation frame while never
dropping started, ended, or cancelled phases. Consequently an arbitrary painted
element can drive a splitter, drag surface, scrubber, selection handle, or
resizer without a specialized engine type.

`VisualState::Focused` reports focus ownership. `VisualState::FocusVisible`
reports keyboard or programmatic focus and stays off for pointer focus. Widgets
can therefore retain accessible keyboard rings without flashing them on click.

## Keyboard and focus

Every physical key transition reaches the focused node as
`UiEventKind::KeyInput`, including modifiers and repeat state. Tab and
Shift-Tab traverse the current scope. Keyboard and accessibility activation
synthesize the same `ClickEvent` as pointer activation while preserving its
typed source. Space retains the pressed visual until key release and cancels
activation if focus or window ownership changes. Text editing runs after raw
dispatch, so applications can observe Escape, arrows, Home/End and the edited
value without platform-specific handlers.

`TextInputFilter` applies the same edit policy to keyboard, paste, and IME
commits. `Any`, `Decimal`, and `Arithmetic` let controlled fields reject an
invalid edit before publishing `TextChanged`; widgets select the policy through
`InputKind`.

`Element::focus_scope` declares focus ownership around a retained subtree.
`FocusScope::restoring` remembers the previous target,
`FocusScope::trapped` additionally wraps Tab traversal, and
`FocusScope::modal` also removes background nodes from the active semantic
tree. Initial focus may select the first focusable descendant or one exact
stable key. Nested scopes restore in stack order.

Application state can request the same operation imperatively with
`Context::request_focus(NodeId | key)` or `Context::clear_focus()`. Requests are
resolved after layout against enabled focus regions; missing, duplicate, or
out-of-scope targets perform no implicit fallback.

## Conditional styles

`Interaction` owns behavior only. Visuals are typed property patches attached
with `Element::when`. `StyleCondition` accepts an own or named-scope state, a
named container query, or a nested `all`, `any`, and `not` expression. All
matching rules compose in declaration order, so the last declaration for one
property wins without discarding unrelated properties. `Element::transition`
supplies one tween or spring default plus property- and direction-specific rules.

Paint, transform, scroll, and layout properties retain their exact invalidation
class. An interaction color does not invoke Taffy; an animated width updates the
existing Taffy node. Tracks live beside stable `NodeId`s, so rebuilds retarget
from the presented value without restarting or jumping. First mount snaps,
reduced motion finishes active tracks, and idle state requests no frames.

`Element::state_scope` establishes a stable control boundary and
`StateSelector::scope` resolves the nearest matching boundary. Descendant
interaction contributes hover, focus, and press to that scope; named states
come from the scope root. Nested controls with the same scope identity shadow
their ancestor, which prevents an inner control from accidentally styling an
outer recipe.

`HitTestStyle` is independent from paint. `PointerEvents` can target the box,
its descendants, both, or neither. `HitShape` provides bounds, rounded-rectangle,
and ellipse geometry, and per-edge hit slop can enlarge a touch target without
changing its layout or pixels. Transforms and every ancestor clip are applied
before a target participates in reverse paint-order hit testing.

Named container queries modify typed style or layout values on the same retained
elements. Ordinary flex and grid layouts remain automatic; a query is only for
a deliberate semantic adaptation such as changing a toolbar from a row to a
column. See [responsive styling](styling.md) for convergence and invalidation.

`Button` is a composition helper in `argui-ui`: one interactive container, one
text child, layout style, paint styles, and no renderer-specific widget code.
Button labels default to no wrapping and their container has zero flex shrink,
so a wrapping row moves a complete button instead of truncating its label. Text
ink is clipped by explicit ancestor clips rather than its exact advance box,
which preserves glyph overhangs at the final character. The renderer still
receives only quads and text commands.

Mouse, touch and pen share one `PointerEvent` representation. Touch contacts
retain stable IDs and pressure, and only the primary contact mutates hover,
press and scroll state. Every contact still enters the opt-in gesture arena so
pinch and rotation can be recognized simultaneously. See
[accessibility and touch](#accessibility-and-touch) for the semantic and gesture
contracts.

Ranges use absolute pointer coordinates, so pressing the track moves directly
to that value. Optional `RangeDetents` add velocity-aware magnetic stops: slow
motion settles within a configured tolerance while fast motion remains free.

## Accessibility and touch

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

### Pointer and gesture contract

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

### System preferences

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
