# Accessibility for native and TSX components

Argui keeps one semantic tree for native windows and the browser. Roles, names,
values, states, relations, focus and actions belong to the UI tree. AccessKit
translates the native tree to the platform accessibility service; the web
adapter translates it to DOM and ARIA. A visual primitive is decorative until
its author gives it semantics. All built-in primitives, including `rectangle`
and `text`, accept the same semantic properties in Solid and React TSX.

## Custom control

```tsx
<rectangle
  role="button"
  accessible_name="Save"
  accessible_description="Save the current document"
  focusable
  keyboard_activation="enter_or_space"
  onClick={save}
/>
```

The `role` and other keyword values are checked by the TypeScript types and
validated by the native schema. `focusable` adds the primitive to sequential
focus by default; `focus_on_tab_navigation={false}` keeps programmatic focus
without adding a Tab stop. `keyboard_activation` routes Enter or Space through
the same `onClick` listener as an accessibility click. `accessible_disabled`
marks a custom control disabled and stops its interaction.

`focusScope` remains useful when a control consists of several painted children
or needs focus containment, initial focus or restoration. It shares the same
semantic vocabulary. Keep decorative children without roles so the composed
control has one accessible name and one focus target.

## Values, states and relations

Use `accessible_value` for text values. For a slider or other numeric control,
set `numeric_value` and optionally `minimum_value`, `maximum_value` and
`value_step`. These fields are mutually exclusive with `accessible_value`.
`selected`, `current`, `checked_state`, `pressed_state`, `expanded`, `busy`,
`required`, `read_only`, `invalid` and `multiselectable` describe controlled
state. `checked_state="mixed"` represents partial selection.
`orientation`, `level`, `position_in_set`, `set_size`, `has_popup`, `sort` and
`modal` provide structural context. `accessible_hidden` removes a decorative
subtree from the semantic tree.

Relations use the target's native `key` in Solid or `nativeKey` in React:

```tsx
<column>
  <text key="volume-label" text="Volume" />
  <rectangle role="slider" labelled_by="volume-label"
    numeric_value={42} minimum_value={0} maximum_value={100}
    value_step={2} focusable can_increment can_decrement
    onSemanticAction={({ action }) => {
      if (action === 'increment') increaseVolume()
      if (action === 'decrement') decreaseVolume()
    }} />
</column>
```

`labelled_by`, `described_by`, `controls` and `active_descendant` resolve in
the retained semantic tree. The first three accept space-separated keys.
Use stable keys and keep their targets mounted.
`live="polite"` or `live="assertive"` announces changes where appropriate.

`can_increment`, `can_decrement`, `can_set_value`, `can_expand`, `can_collapse`
and `can_scroll_into_view` advertise platform actions. Handle them with
`onSemanticAction`; the callback receives `action` and an optional text or
numeric `value`. The accessible state must be updated by the component after
the action, just as it is after a pointer or keyboard event.

## Platform checks

Tests in `argui-schema`, `argui-accessibility` and the framework packages check
the common contract without an OS window. For desktop Linux, use
`./scripts/linux-hidden-display.sh` and inspect the native window with Orca
and AT-SPI on that private display. Repeat the screen-reader workflow on macOS
and Windows before treating either platform as verified. Android and TalkBack
are a separate later validation step.
