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
Application-facing `UiEvent`s carry both the ID and the optional key.

Hit regions are emitted in paint-tree order and tested in reverse, so the last
interactive element painted wins. Bounds and the resolved rectangular ancestor
clip must both contain the pointer. Future z-index/layer work must reorder paint
and hit-test data together.

Primary-button presses capture their target until release. A release over the
captured target emits `Clicked`; a release outside emits only `Released`.
Focusable controls receive focus on press, and losing window focus clears hover,
press, capture, and focus safely. The runtime restores the exact retained focus
when the window becomes active again.

`VisualState::Focused` reports focus ownership. `VisualState::FocusVisible`
reports keyboard or programmatic focus and stays off for pointer focus. Widgets
can therefore retain accessible keyboard rings without flashing them on click.

## Keyboard and focus

Every physical key transition reaches the focused node as
`UiEventKind::KeyInput`, including modifiers and repeat state. Tab and
Shift-Tab traverse the current scope. Buttons synthesize `Pressed`, `Clicked`
and `Released` from Enter; Space retains the pressed visual until key release
and cancels activation if focus or window ownership changes. Text editing runs
after raw dispatch, so applications can observe Escape, arrows, Home/End and
the edited value without platform-specific handlers.

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
[accessibility and touch](accessibility.md) for the semantic and gesture
contracts.

Ranges use absolute pointer coordinates, so pressing the track moves directly
to that value. Optional `RangeDetents` add velocity-aware magnetic stops: slow
motion settles within a configured tolerance while fast motion remains free.
