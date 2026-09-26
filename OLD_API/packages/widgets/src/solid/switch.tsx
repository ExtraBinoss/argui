import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { SwitchProps } from '../shared/foundation-e'

const mobile = (globalThis as { __arguiMobile?: boolean }).__arguiMobile === true

/** Provides a themed native switch with controlled and default-checked state. */
export function Switch(props: SwitchProps): JSX.Element {
  const [uncontrolled, setUncontrolled] = createSignal(props.defaultChecked ?? false)
  const [focused, setFocused] = createSignal(false)
  const [hovered, setHovered] = createSignal(false)
  let pointerFocus = false
  const checked = () => props.checked ?? uncontrolled()
  const disabled = () => !!props.disabled
  const toggle = () => {
    if (disabled()) return
    const next = !checked()
    if (props.checked === undefined) setUncontrolled(next)
    props.onCheckedChange?.(next)
  }
  const small = props.size === 'sm'
  const width = small ? 24 : 32
  const height = small ? 14 : 18
  const thumb = small ? 10 : 14

  return <focusScope key={props.id} role="switch" accessible_name={props.label}
    checked_state={checked() ? 'checked' : 'unchecked'} enabled={!disabled()}
    keyboard_activation="enter_or_space" onClick={toggle}
    onFocus={() => { if (!pointerFocus) setFocused(true) }}
    onKey={() => { pointerFocus = false; setFocused(true) }}
    onBlur={() => { pointerFocus = false; setFocused(false) }}>
    <touchArea enabled={!disabled()} mouse_cursor={disabled() ? 'not_allowed' : 'pointer'}
      onPointerEnter={() => { if (!mobile) setHovered(true) }}
      onPointerLeave={() => setHovered(false)}
      onPointerDown={() => { pointerFocus = true; setFocused(false) }}>
      <rectangle width={width} height={height} radius={height / 2}
        background={checked() ? props.theme.accent : hovered() ? props.theme.surfaceHover : props.theme.surfaceRaised}
        border_color={focused() ? props.theme.accent : props.theme.border}
        border_width={focused() ? 2 : 1} opacity={disabled() ? 0.48 : 1}>
        <row width="fill" height="fill" padding={2} align_items="center">
          {checked() ? <container grow={1} /> : null}
          <rectangle width={thumb} height={thumb} radius={thumb / 2}
            background={checked() ? props.theme.accentText : props.theme.surface} />
          {checked() ? null : <container grow={1} />}
        </row>
      </rectangle>
    </touchArea>
  </focusScope>
}
