/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { useTheme } from '@argui/react'
import type { WidgetTheme } from '../shared/theme'
import type { BooleanControlOptions } from '../shared/new-controls'

/** Props for a native checkbox with local or controlled state. */
export type CheckboxProps = BooleanControlOptions

/** Renders a focusable checkbox whose checked state is exposed to accessibility. */
export function Checkbox(props: CheckboxProps): ReactElement {
  const theme = useTheme<WidgetTheme>()
  const [localValue, setLocalValue] = useState(props.defaultValue ?? false)
  const checked = props.value ?? localValue
  const readOnly = props.value !== undefined && !props.onValueChange
  const enabled = !props.disabled && !readOnly
  const toggle = () => {
    if (!enabled) return
    if (props.value === undefined) setLocalValue(!checked)
    props.onValueChange?.(!checked)
  }
  return <focusScope id={props.id} role="checkBox" accessibleName={props.accessibleName}
    accessibleDescription={props.description} checkedState={checked ? 'checked' : 'unchecked'}
    enabled={enabled} readOnly={readOnly} keyboardActivation="enterOrSpace" onClick={toggle}
    mouseCursor={enabled ? 'pointer' : 'notAllowed'}
    width={props.width} minWidth={props.minWidth} maxWidth={props.maxWidth}
    grow={props.grow} shrink={props.shrink} alignSelf={props.alignSelf} margin={props.margin}>
    <row gap={8} alignItems="center">
      <rectangle width={16} height={16} radii={4} shrink={0}
        background={checked ? theme.primary : theme.background}
        hoverBackground={checked ? theme.primaryHover : theme.controlHover}
        border={{ width: 1, color: checked ? theme.primary : theme.input }}
        focusBorderColor={theme.focusRing} opacity={enabled ? 1 : 0.5}>
        <row width="100%" height="100%" alignItems="center" justifyContent="center">
          {checked ? <text color={theme.primaryForeground} fontSize={13} weight={700}>✓</text> : null}
        </row>
      </rectangle>
      {props.label ? <text color={theme.text}>{props.label}</text> : null}
    </row>
  </focusScope>
}
