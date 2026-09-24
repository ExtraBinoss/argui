/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import type { SelectProps } from '../shared/types'
import { useWidgetIcons } from './assets'

export type ReactSelectProps = SelectProps

/** Composes a controlled native combobox with an anchored option popup. */
export function ReactSelect(props: ReactSelectProps): ReactElement {
  const icons = useWidgetIcons()
  const [expanded, setExpanded] = useState(false)
  const [activeIndex, setActiveIndex] = useState(0)
  const [focused, setFocused] = useState(false)
  const open = () => {
    setActiveIndex(Math.max(0, props.options.indexOf(props.value)))
    setExpanded(true)
  }
  const choose = (value: string) => {
    props.onChange(value)
    setExpanded(false)
  }
  const onKey = (payload: unknown) => {
    if (typeof payload === 'object' && payload && 'state' in payload && payload.state !== 'pressed') return
    const key = typeof payload === 'object' && payload && 'key' in payload ? String(payload.key) : String(payload)
    if (key === 'Escape') setExpanded(false)
    else if (key === 'ArrowDown' || key === 'ArrowUp') {
      if (props.options.length === 0) return
      const step = key === 'ArrowDown' ? 1 : -1
      setActiveIndex((index) => (index + step + props.options.length) % props.options.length)
      setExpanded(true)
    } else if (key === 'Enter' || key === ' ') {
      if (expanded) {
        const option = props.options[activeIndex]
        if (option !== undefined) choose(option)
      } else open()
    }
  }
  return (
    <column gap={8}>
      <text text={props.label} color={props.theme.muted} font_size={12} />
      <focusScope nativeKey={props.id} role="combo_box"
        accessible_name={`${props.label}: ${props.value}`} accessible_value={props.value}
        controls={`${props.id}-popup`}
        active_descendant={expanded ? `${props.id}-option-${activeIndex}` : undefined}
        expandable={true} expanded={expanded} keyboard_activation="none"
        onFocus={() => setFocused(true)} onBlur={() => setFocused(false)}
        onClick={() => expanded ? setExpanded(false) : open()} onKey={onKey}>
        <rectangle width={240} height={40} background={props.theme.surface}
          border_color={focused ? props.theme.accent : props.theme.border}
          border_width={focused ? 2 : 1} radius={8}>
          <row width="fill" height="fill" padding={10} align_items="center">
            <text text={props.value} color={props.theme.foreground} font_size={14} />
            <container grow={1} />
            {icons.chevronDown ? <svg source={icons.chevronDown} color={props.theme.muted} width={16} height={16} rotation={expanded ? 180 : 0} /> : null}
          </row>
        </rectangle>
      </focusScope>
      {expanded ? (
        <popupWindow nativeKey={`${props.id}-popup`} anchor={props.id} placement="bottom_start"
          width={240} window_layer="popover" dismiss_policy="outside_pointer_or_escape" containment="trap" initial_focus="first" restore_focus={true}
          onDismiss={() => setExpanded(false)}>
          <focusScope role="list_box" accessible_name={props.label} focus_on_tab_navigation={false}>
            <rectangle width={240} background={props.theme.surface}
              border_color={props.theme.border} border_width={1} radius={8}
              shadow_blur={14} shadow_offset_y={5} shadow_color={props.theme.overlayShadow}>
              <column width="fill" gap={2} padding={4}>
                {props.options.map((option, index) => (
                  <focusScope key={option} nativeKey={`${props.id}-option-${index}`} role="option"
                    accessible_name={option} selected={props.value === option}
                    keyboard_activation="enter_or_space" onClick={() => choose(option)}>
                    <touchArea mouse_cursor="pointer" onPointerEnter={() => setActiveIndex(index)}>
                      <rectangle width="fill" height={34}
                        background={index === activeIndex || option === props.value ? props.theme.surfaceRaised : props.theme.surface}
                        radius={5}>
                        <row width="fill" height="fill" padding={10} align_items="center">
                          <text text={option} color={props.theme.foreground} font_size={14} />
                        </row>
                      </rectangle>
                    </touchArea>
                  </focusScope>
                ))}
              </column>
            </rectangle>
          </focusScope>
        </popupWindow>
      ) : null}
    </column>
  )
}
