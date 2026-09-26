/** @jsxImportSource @argui/react */
import { useRef, useState, type ReactElement } from 'react'
import {
  foundationGInitialRadioValue,
  foundationGKey,
  foundationGNextRadioValue,
  type RadioGroupProps,
} from '../shared/foundation-g'

/** Renders the same accessible radio group through the React adapter. */
export function ReactRadioGroup(props: RadioGroupProps): ReactElement {
  const [uncontrolled, setUncontrolled] = useState<string | undefined>(() =>
    foundationGInitialRadioValue(props.options, props.defaultValue),
  )
  const [focused, setFocused] = useState(false)
  const pointerFocus = useRef(false)
  const value = props.value !== undefined ? props.value : uncontrolled
  const optionDisabled = (disabled?: boolean) => !!props.disabled || !!disabled
  const active = props.options.find((option) => option.value === value && !optionDisabled(option.disabled))
    ?? props.options.find((option) => !optionDisabled(option.disabled))
  const choose = (next: string) => {
    const option = props.options.find((candidate) => candidate.value === next)
    if (!option || optionDisabled(option.disabled) || value === next) return
    if (props.value === undefined) setUncontrolled(next)
    props.onValueChange?.(next)
  }
  const move = (direction: -1 | 1) => {
    const next = foundationGNextRadioValue(
      props.options.filter((option) => !optionDisabled(option.disabled)), active?.value, direction,
    )
    if (next !== undefined) choose(next)
  }
  const onKey = (payload: unknown) => {
    const key = foundationGKey(payload)
    if (key === undefined) return
    pointerFocus.current = false
    setFocused(true)
    const orientation = props.orientation ?? 'vertical'
    if ((orientation === 'horizontal' && key === 'ArrowRight') || (orientation === 'vertical' && key === 'ArrowDown')) move(1)
    else if ((orientation === 'horizontal' && key === 'ArrowLeft') || (orientation === 'vertical' && key === 'ArrowUp')) move(-1)
    else if (key === 'Home') {
      const first = props.options.find((option) => !optionDisabled(option.disabled))
      if (first) choose(first.value)
    } else if (key === 'End') {
      const last = [...props.options].reverse().find((option) => !optionDisabled(option.disabled))
      if (last) choose(last.value)
    } else if (key === ' ' || key === 'Space') {
      if (active) choose(active.value)
    }
  }
  const renderedOptions = props.options.map((option) => {
    const selected = value === option.value
    const disabled = optionDisabled(option.disabled)
    const focusedItem = focused && active?.value === option.value
    const border = props.invalid ? props.theme.destructive : focusedItem ? props.theme.accent : selected ? props.theme.accent : props.theme.border
    return <focusScope key={`${props.id}-option-${option.value}`} nativeKey={`${props.id}-option-${option.value}`}
        role="radio_button" accessible_name={option.label} accessible_description={option.description}
        selected={selected} checked_state={selected ? 'checked' : 'unchecked'} enabled={!disabled}
        focusable={true} focus_on_tab_navigation={false} keyboard_activation="enter_or_space"
        onClick={() => choose(option.value)}>
      <touchArea enabled={!disabled} mouse_cursor={disabled ? 'not_allowed' : 'pointer'}
          onPointerDown={() => { pointerFocus.current = true; setFocused(false) }}>
        <row width="fill" gap={9} align_items="center" opacity={disabled ? 0.48 : 1}>
          <rectangle width={16} height={16} radius={8} background={props.theme.surface}
            border_color={border} border_width={focusedItem ? 2 : 1}>
            {selected ? <row width="fill" height="fill" align_items="center" justify_content="center">
              <rectangle width={8} height={8} radius={4} background={props.theme.accent} accessible_hidden={true} />
            </row> : null}
          </rectangle>
          <column gap={3}>
            <text text={option.label} color={props.theme.foreground} font_size={props.theme.controlFontSize} />
            {option.description ? <text text={option.description} color={props.theme.muted} font_size={12}
              accessible_hidden={true} /> : null}
          </column>
        </row>
      </touchArea>
    </focusScope>
  })
  const enabled = !props.disabled && props.options.some((option) => !option.disabled)
  const orientation = props.orientation ?? 'vertical'
  return <focusScope nativeKey={props.id} role="radio_group" accessible_name={props.label}
    active_descendant={active ? `${props.id}-option-${active.value}` : undefined}
    orientation={orientation} enabled={enabled} focusable={enabled}
    invalid={!!props.invalid} onFocus={() => { if (!pointerFocus.current) setFocused(true) }}
    onKey={onKey} onBlur={() => { pointerFocus.current = false; setFocused(false) }}>
    {orientation === 'horizontal'
      ? <row width="fill" gap={12}>{renderedOptions}</row>
      : <column width="fill" gap={12}>{renderedOptions}</column>}
  </focusScope>
}
