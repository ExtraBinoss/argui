/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import { useTheme } from '@argui/react'
import type { WidgetTheme } from '../shared/theme'
import type { BooleanControlOptions } from '../shared/new-controls'

/** Props for a native switch with local or controlled state. */
export type SwitchProps = BooleanControlOptions & { size?: 'default' | 'sm' }

/** Renders a switch with one focusable hit target and a themed track and thumb. */
export function Switch(props: SwitchProps): ReactElement {
  const theme = useTheme<WidgetTheme>()
  const [localValue, setLocalValue] = useState(props.defaultValue ?? false)
  const checked = props.value ?? localValue
  const readOnly = props.value !== undefined && !props.onValueChange
  const enabled = !props.disabled && !readOnly
  const small = props.size === 'sm'
  const toggle = () => {
    if (!enabled) return
    if (props.value === undefined) setLocalValue(!checked)
    props.onValueChange?.(!checked)
  }
  return <focusScope id={props.id} role="switch" accessibleName={props.accessibleName}
    accessibleDescription={props.description} checkedState={checked ? 'checked' : 'unchecked'}
    enabled={enabled} readOnly={readOnly} keyboardActivation="enterOrSpace" onClick={toggle}
    mouseCursor={enabled ? 'pointer' : 'notAllowed'}
    width={props.width} minWidth={props.minWidth} maxWidth={props.maxWidth}
    grow={props.grow} shrink={props.shrink} alignSelf={props.alignSelf} margin={props.margin}>
    <row gap={8} alignItems="center">
      <rectangle width={small ? 24 : 32} height={small ? 14 : 18} radii={9} shrink={0}
        padding={1} background={checked ? theme.primary : theme.input}
        hoverBackground={checked ? theme.primaryHover : theme.controlHover}
        focusBorderColor={theme.focusRing} opacity={enabled ? 1 : 0.5}>
        <container width="100%" height="100%" position="relative">
          <container width={small ? 12 : 16} height={small ? 12 : 16} position="absolute"
            inset={{ start: 0, top: 0 }} transform={{ translateX: checked ? (small ? 10 : 14) : 0 }}
            transitionMs={150} transitionTimingFunction="cubic-bezier(0.4, 0, 0.2, 1)">
            <rectangle width="100%" height="100%" radii={8}
              background={checked ? theme.primaryForeground : theme.background} />
          </container>
        </container>
      </rectangle>
      {props.label ? <text color={theme.text}>{props.label}</text> : null}
    </row>
  </focusScope>
}
