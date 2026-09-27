import { createSignal } from 'solid-js'
import type { JSX } from '@argui/solid/jsx-runtime'
import { useTheme } from '@argui/solid'
import type { AssetRef } from '@argui/host'
import type { WidgetTheme } from '../shared/theme'
import type { BooleanControlOptions } from '../shared/new-controls'

/** Props for a native checkbox with local or controlled state. */
export type CheckboxProps = BooleanControlOptions & { checkedIcon?: AssetRef }

/** Renders a focusable checkbox whose checked state is exposed to accessibility. */
export function Checkbox(props: CheckboxProps): JSX.Element {
  const theme = useTheme<WidgetTheme>()
  const [localValue, setLocalValue] = createSignal(props.defaultValue ?? false)
  const checked = () => props.value ?? localValue()
  const readOnly = () => props.value !== undefined && !props.onValueChange
  const enabled = () => !props.disabled && !readOnly()
  const toggle = () => {
    if (!enabled()) return
    const next = !checked()
    if (props.value === undefined) setLocalValue(next)
    props.onValueChange?.(next)
  }
  return <focusScope id={props.id} role="checkBox" accessibleName={props.accessibleName}
    accessibleDescription={props.description} checkedState={checked() ? 'checked' : 'unchecked'}
    enabled={enabled()} readOnly={readOnly()} keyboardActivation="enterOrSpace" onClick={toggle}
    mouseCursor={enabled() ? 'pointer' : 'notAllowed'}
    width={props.width} minWidth={props.minWidth} maxWidth={props.maxWidth}
    grow={props.grow} shrink={props.shrink} alignSelf={props.alignSelf} margin={props.margin}>
    <row gap={8} alignItems="center">
      <rectangle width={16} height={16} radii={4} shrink={0}
        background={checked() ? theme().primary : theme().background}
        hoverBackground={checked() ? theme().primaryHover : theme().controlHover}
        border={{ width: 1, color: checked() ? theme().primary : theme().input }}
        focusBorderColor={theme().focusRing} opacity={enabled() ? 1 : 0.5}>
        <row width="100%" height="100%" alignItems="center" justifyContent="center">
          {checked() ? props.checkedIcon
            ? <svg source={props.checkedIcon} width={12} height={12} color={theme().primaryForeground} />
            : <text color={theme().primaryForeground} fontSize={13} weight={700}>✓</text> : null}
        </row>
      </rectangle>
      {props.label ? <text color={theme().text}>{props.label}</text> : null}
    </row>
  </focusScope>
}
