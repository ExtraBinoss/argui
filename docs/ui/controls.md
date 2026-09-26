# Checkbox, Switch, Tabs, Slider, and Progress

Both `@argui/widgets/solid` and `@argui/widgets/react` export these five
components with matching props. They read colors from the root `WidgetTheme`,
so an application does not pass a theme into each control. Native `id` is
optional; provide it only for relationships or tests.

Checkbox and Switch use `value` and `onValueChange` for controlled boolean
state, or `defaultValue` for local state. A controlled `value` without a
callback is read-only. `accessibleName` is required even if `label` is not
visible. Switch also accepts `size="sm"`.

Tabs takes `items` with stable `{ value, label, content }` entries. Each item
may be disabled. It supports `variant="default" | "line"`, horizontal or
vertical orientation, and the same controlled or local value contract.
Arrow keys, Home, and End select enabled tabs.

Slider is a horizontal, single-thumb control. It accepts `min`, `max`, and
`step`, clamps and snaps values, and handles click, drag, arrow, Page, Home,
End, and accessibility value actions. Its default range is 0–100 with step 1.

Progress is read-only. Pass a number between 0 and `max` (100 by default),
or `null` for indeterminate progress. It exposes the current value and bounds
to native accessibility APIs. The indeterminate segment sweeps through a
native composited transform. `playing={false}` pauses that sweep at its current
phase; the native reduced-motion preference stops frame requests and leaves a
visible segment at the end of the track. For a full-width sweep, pass a numeric
`width` so the translation can be sized to the track.
