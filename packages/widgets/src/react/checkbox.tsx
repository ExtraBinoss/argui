/** @jsxImportSource @argui/react */
import { useRef, useState, type ReactElement } from 'react'
import type { CheckboxProps, CheckboxValue } from '../shared/foundation-g'

const mobile = (globalThis as { __arguiMobile?: boolean }).__arguiMobile === true

/** Renders the same themed checkbox states through the React adapter. */
export function ReactCheckbox(props: CheckboxProps): ReactElement {
  const [uncontrolled, setUncontrolled] = useState<CheckboxValue>(props.defaultChecked ?? false)
  const [focused, setFocused] = useState(false)
  const [hovered, setHovered] = useState(false)
  const pointerFocus = useRef(false)
  const checked = props.checked !== undefined ? props.checked : uncontrolled
  const disabled = !!props.disabled
  const toggle = () => {
    if (disabled) return
    const next = checked !== true
    if (props.checked === undefined) setUncontrolled(next)
    props.onCheckedChange?.(next)
  }
  const active = checked !== false
  const background = active ? props.theme.accent : hovered ? props.theme.surfaceHover : props.theme.surface
  const border = props.invalid ? props.theme.destructive : focused ? props.theme.accent : active ? props.theme.accent : props.theme.border
  return <focusScope nativeKey={props.id} role="check_box" accessible_name={props.label}
    accessible_description={props.description} checked_state={checked === 'indeterminate' ? 'mixed' : checked ? 'checked' : 'unchecked'}
    enabled={!disabled} required={!!props.required} invalid={!!props.invalid}
    keyboard_activation="enter_or_space" onClick={toggle}
    onFocus={() => { if (!pointerFocus.current) setFocused(true) }}
    onKey={() => { pointerFocus.current = false; setFocused(true) }}
    onBlur={() => { pointerFocus.current = false; setFocused(false) }}>
    <touchArea enabled={!disabled} mouse_cursor={disabled ? 'not_allowed' : 'pointer'}
      onPointerEnter={() => { if (!mobile) setHovered(true) }}
      onPointerLeave={() => setHovered(false)}
      onPointerDown={() => { pointerFocus.current = true; setFocused(false) }}>
      <rectangle width={16} height={16} radius={4} background={background}
        border_color={border} border_width={focused ? 2 : 1} opacity={disabled ? 0.48 : 1}>
        {checked === true ? <text text="✓" width="fill" height="fill" color={props.theme.accentText}
          font_size={12} weight={700} text_align="center" accessible_hidden={true} />
          : checked === 'indeterminate' ? <row width="fill" height="fill" align_items="center" justify_content="center">
            <rectangle width={8} height={2} radius={1} background={props.theme.accentText} accessible_hidden={true} />
          </row> : null}
      </rectangle>
    </touchArea>
  </focusScope>
}
