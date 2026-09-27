# Accessibility

Argui maintains accessibility semantics in the retained UI tree. Native
windows lower that tree through AccessKit. The Web backend maintains a DOM
accessibility representation beside the canvas, including real input and
textarea elements for text editing. The Solid and React generated JSX
declarations expose the same semantic properties on built-in primitives.
The higher-level widgets supply roles, focus, names, and state for their
intended use; an authored custom control must supply these deliberately.

## Name and activate a custom control

The following Solid control has one focus target and a description relation.
Its visible text is inside the painted rectangle, while the surrounding
`focusScope` owns click and keyboard activation:

```tsx solid
import { createSignal, useTheme } from '@argui/solid'
import type { WidgetTheme } from '@argui/widgets/solid'

export function SaveAction() {
  const theme = useTheme<WidgetTheme>()
  const [saved, setSaved] = createSignal(false)

  return <column gap={8}>
    <text id="save-help" color={theme().textMuted}>
      Saves the current document in this window.
    </text>
    <focusScope role="button" accessibleName="Save document"
      describedBy="save-help" focusable keyboardActivation="enterOrSpace"
      onClick={() => setSaved(true)}>
      <rectangle width={120} height={36} radii={theme().radius}
        background={theme().primary} hoverBackground={theme().primaryHover}
        focusBorderColor={theme().focusRing}>
        <row width="100%" height="100%" alignItems="center" justifyContent="center">
          <text color={theme().primaryForeground}>Save</text>
        </row>
      </rectangle>
    </focusScope>
    <text role="status" live="polite" color={theme().text}>
      {saved() ? 'Saved' : 'Unsaved changes'}
    </text>
  </column>
}
```

`focusable` makes a primitive eligible for focus. Set
`focusOnTabNavigation={false}` if it needs programmatic focus but should not
be a Tab stop. `keyboardActivation="enterOrSpace"` sends those keys through
the same `onClick` path as pointer and accessibility activation. Set
`accessibleDisabled` for an unavailable custom action, and update its visible
paint as well. Decorative children can use `accessibleHidden` to omit their
subtree from the semantic tree when they would duplicate a control's name.

## States, values, and relations

Use the state that matches the control's behavior:

| Property | Meaning |
| --- | --- |
| `checkedState="unchecked" \| "checked" \| "mixed"` | Checkbox or switch state, including partial selection. |
| `pressedState` | Persistent toggle-button state; a transient pointer press is separate. |
| `selected`, `current`, `expanded`, `busy` | Selection, current location, disclosure, and progress state. |
| `required`, `readOnly`, `invalid`, `accessibleDisabled` | Input and availability state. |
| `live="polite" \| "assertive"` | Announces changed status content at the requested priority. |

`accessibleValue` is a text value. Numeric controls use `numericValue` with
optional `minimumValue`, `maximumValue`, and `valueStep`; the native schema
rejects setting text and numeric values together. The built-in `Slider` uses
numeric values and advertises increment, decrement, and set-value actions.
`Progress` exposes its numeric bounds or a busy indeterminate state.

`labelledBy`, `describedBy`, and `controls` reference stable native `id` values;
the first three accept space-separated IDs. `activeDescendant` names one
native ID. Framework `key` controls reconciliation and is not an
accessibility relation target. Keep relation targets mounted and keep their
IDs stable even when visible labels change. The example above uses
`describedBy="save-help"`; a `Select` uses `controls` and
`activeDescendant` for its popup and option.

`orientation`, `level`, `positionInSet`, `setSize`, `hasPopup`, `sort`,
`modal`, and `multiselectable` supply context to supporting platform
accessibility services. Advertise custom actions with `canIncrement`,
`canDecrement`, `canSetValue`, `canExpand`, `canCollapse`, or
`canScrollIntoView`, then handle `onSemanticAction` and update the state
reported on the node. The payload contains `action` and an optional text or
numeric `value`. The widgets implement their own action handling; application
code normally uses their `onValueChange` callbacks instead.

## Focus and platform verification

`focusScope` is an unpainted focus boundary. It can trap Tab navigation with
`containment="trap"`, or exclude background controls from modal navigation
with `containment="modal"`. `Popover` is nonmodal by default; `Select` traps
focus in its option popup. A modal dialog needs the modal containment contract,
not only a visual overlay. A keyboard or programmatic focus gives the native
focus-visible state; pointer focus need not draw the same ring.

Generated types and schema tests verify the common contract. They do not
replace a screen-reader pass on each target platform. For Linux GUI checks,
use the repository's [private-display procedure](../contributing/linux-testing.md)
and inspect a saved capture as well as AT-SPI/Orca behavior on that display.
Check browser assistive technology against the Web DOM adapter separately.
Windows, macOS, and mobile screen readers require their own validation before
claiming support for a release.
