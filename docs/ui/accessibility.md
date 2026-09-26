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
  accessibleName="Save"
  accessibleDescription="Save the current document"
  focusable
  keyboardActivation="enterOrSpace"
  onClick={save}
/>
```

The `role` and other keyword values are checked by the TypeScript types and
validated by the native schema. `focusable` adds the primitive to sequential
focus by default; `focusOnTabNavigation={false}` keeps programmatic focus
without adding a Tab stop. `keyboardActivation` routes Enter or Space through
the same `onClick` listener as an accessibility click. `accessibleDisabled`
marks a custom control disabled and stops its interaction.

`focusScope` remains useful when a control consists of several painted children
or needs focus containment, initial focus or restoration. It shares the same
semantic vocabulary. Keep decorative children without roles so the composed
control has one accessible name and one focus target.

## Values, states and relations

Use `accessibleValue` for text values. For a slider or other numeric control,
set `numericValue` and optionally `minimumValue`, `maximumValue` and
`valueStep`. These fields are mutually exclusive with `accessibleValue`.
`selected`, `current`, `checkedState`, `pressedState`, `expanded`, `busy`,
`required`, `readOnly`, `invalid` and `multiselectable` describe controlled
state. `checkedState="mixed"` represents partial selection.
`orientation`, `level`, `positionInSet`, `setSize`, `hasPopup`, `sort` and
`modal` provide structural context. `accessibleHidden` removes a decorative
subtree from the semantic tree.

Relations use the target's native `id` in both Solid and React. `key` belongs
to framework reconciliation and never names an accessibility target:

```tsx
<column>
  <text id="volume-label">Volume</text>
  <rectangle role="slider" labelledBy="volume-label"
    numericValue={42} minimumValue={0} maximumValue={100}
    valueStep={2} focusable canIncrement canDecrement
    onSemanticAction={({ action }) => {
      if (action === 'increment') increaseVolume()
      if (action === 'decrement') decreaseVolume()
    }} />
</column>
```

`labelledBy`, `describedBy`, `controls` and `activeDescendant` resolve in
the retained semantic tree. The first three accept space-separated IDs.
Use stable IDs and keep their targets mounted.
`live="polite"` or `live="assertive"` announces changes where appropriate.

`canIncrement`, `canDecrement`, `canSetValue`, `canExpand`, `canCollapse`
and `canScrollIntoView` advertise platform actions. Handle them with
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
