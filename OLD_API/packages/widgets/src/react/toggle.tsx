/** @jsxImportSource @argui/react */
import { useRef, useState, type ReactElement } from 'react'
import type { ToggleProps } from '../shared/foundation-g'

const mobile = (globalThis as { __arguiMobile?: boolean }).__arguiMobile === true

/** Renders the same pressed-state button through the React adapter. */
export function ReactToggle(props: ToggleProps): ReactElement {
  const [uncontrolled, setUncontrolled] = useState(props.defaultPressed ?? false)
  const [focused, setFocused] = useState(false)
  const [hovered, setHovered] = useState(false)
  const pointerFocus = useRef(false)
  const pressed = props.pressed ?? uncontrolled
  const disabled = !!props.disabled
  const toggle = () => {
    if (disabled) return
    const next = !pressed
    if (props.pressed === undefined) setUncontrolled(next)
    props.onPressedChange?.(next)
  }
  const size = props.size === 'sm' ? 32 : props.size === 'lg' ? 40 : 36
  const padding = props.size === 'sm' ? 6 : props.size === 'lg' ? 10 : 8
  const fill = pressed ? props.theme.accent : hovered ? props.theme.surfaceHover : '#00000000'
  const color = pressed ? props.theme.accentText : props.theme.foreground
  const border = props.invalid ? props.theme.destructive : focused ? props.theme.accent
    : props.variant === 'outline' ? props.theme.border : '#00000000'
  return <focusScope nativeKey={props.id} role="button" accessible_name={props.label}
    pressed_state={pressed} enabled={!disabled} invalid={!!props.invalid}
    keyboard_activation="enter_or_space" onClick={toggle}
    onFocus={() => { if (!pointerFocus.current) setFocused(true) }}
    onKey={() => { pointerFocus.current = false; setFocused(true) }}
    onBlur={() => { pointerFocus.current = false; setFocused(false) }}>
    <touchArea enabled={!disabled} mouse_cursor={disabled ? 'not_allowed' : 'pointer'}
      onPointerEnter={() => { if (!mobile) setHovered(true) }}
      onPointerLeave={() => setHovered(false)}
      onPointerDown={() => { pointerFocus.current = true; setFocused(false) }}>
      <rectangle min_width={size} height={size} radius={props.theme.controlRadius} background={fill}
        border_color={border} border_width={props.variant === 'outline' || focused || props.invalid ? 1 : 0}
        opacity={disabled ? 0.48 : 1}>
        <row width="fill" height="fill" padding={padding} gap={8} align_items="center" justify_content="center">
          {props.icon ? <svg source={props.icon} color={color} width={16} height={16} accessible_hidden={true} /> : null}
          <text text={props.text ?? props.label} color={color} font_size={props.size === 'sm' ? 12 : props.size === 'lg' ? 15 : 14}
            weight={500} no_wrap={true} />
        </row>
      </rectangle>
    </touchArea>
  </focusScope>
}
