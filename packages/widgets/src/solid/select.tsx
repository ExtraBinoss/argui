import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { SelectProps } from '../shared/types'
import { nextSelectIndex, resolveSelectOptions } from '../shared/select-options'
import { useWidgetIcons } from './assets'

/** Composes an accessible, scrollable native combobox and anchored option popup. */
export function Select(props: SelectProps): JSX.Element {
  const icons = useWidgetIcons()
  const [expanded, setExpanded] = createSignal(false)
  const [activeIndex, setActiveIndex] = createSignal(-1)
  const [focused, setFocused] = createSignal(false)
  const [uncontrolled, setUncontrolled] = createSignal(props.defaultValue ?? '')
  const options = () => resolveSelectOptions(props.options)
  const value = () => props.value ?? uncontrolled()
  const chosen = () => options().find((option) => option.value === value())
  const width = () => Number.isFinite(props.width) && (props.width ?? 0) > 0 ? props.width! : 240
  const open = () => {
    if (props.disabled) return
    const entries = options()
    const selected = entries.findIndex((option) => option.value === value() && !option.disabled)
    setActiveIndex(selected >= 0 ? selected : nextSelectIndex(entries, -1, 1))
    setExpanded(true)
  }
  const choose = (index: number) => {
    const option = options()[index]
    if (!option || option.disabled || props.disabled) return
    if (props.value === undefined) setUncontrolled(option.value)
    props.onChange?.(option.value)
    setExpanded(false)
  }
  const onKey = (payload: unknown) => {
    if (props.disabled) return
    if (typeof payload === 'object' && payload && 'state' in payload && payload.state !== 'pressed') return
    const key = typeof payload === 'object' && payload && 'key' in payload
      ? String(payload.key) : String(payload)
    const entries = options()
    if (key === 'Escape') setExpanded(false)
    else if (key === 'ArrowDown' || key === 'ArrowUp') {
      const next = nextSelectIndex(entries, activeIndex() < 0 ? (key === 'ArrowDown' ? -1 : 0) : activeIndex(),
        key === 'ArrowDown' ? 1 : -1)
      if (next >= 0) { setActiveIndex(next); setExpanded(true) }
    } else if (key === 'Home' || key === 'End') {
      const next = nextSelectIndex(entries, key === 'Home' ? -1 : 0, key === 'Home' ? 1 : -1)
      if (next >= 0) { setActiveIndex(next); setExpanded(true) }
    } else if (key === 'Enter' || key === ' ') {
      if (expanded()) choose(activeIndex())
      else open()
    } else if (key.length === 1 && key !== ' ') {
      const search = key.toLocaleLowerCase()
      const next = entries.findIndex((option) => !option.disabled && option.label.toLocaleLowerCase().startsWith(search))
      if (next >= 0) { setActiveIndex(next); setExpanded(true) }
    }
  }
  return <column gap={8}>
    <text text={props.label} color={props.theme.muted} font_size={12} />
    <focusScope key={props.id} role="combo_box"
      accessible_name={`${props.label}: ${chosen()?.label ?? props.placeholder ?? 'Choose an option'}`}
      accessible_value={chosen()?.label ?? ''} enabled={!props.disabled}
      controls={`${props.id}-popup`}
      active_descendant={expanded() && activeIndex() >= 0 ? `${props.id}-option-${activeIndex()}` : undefined}
      expandable={true} expanded={expanded()} keyboard_activation="none"
      onFocus={() => setFocused(true)} onBlur={() => setFocused(false)}
      onClick={() => { if (!props.disabled) expanded() ? setExpanded(false) : open() }} onKey={onKey}>
      <rectangle width={width()} height={props.theme.inputHeight}
        background={props.disabled ? props.theme.surfaceRaised : props.theme.surface}
        border_color={focused() ? props.theme.accent : props.theme.border}
        border_width={focused() ? 2 : 1} radius={props.theme.controlRadius}
        opacity={props.disabled ? 0.55 : 1}>
        <row width="fill" height="fill" padding={props.theme.controlPadding} align_items="center">
          <text text={chosen()?.label ?? props.placeholder ?? 'Choose an option'}
            color={chosen() ? props.theme.foreground : props.theme.muted}
            font_size={props.theme.controlFontSize} />
          <container grow={1} />
          {icons.chevronDown ? <svg source={icons.chevronDown} color={props.theme.muted}
            width={16} height={16} rotation={expanded() ? 180 : 0} /> : null}
        </row>
      </rectangle>
    </focusScope>
    {expanded() ? <popupWindow key={`${props.id}-popup`} anchor={props.id} placement="bottom_start"
      width={width()} window_layer="popover" dismiss_policy="outside_pointer_or_escape"
      containment="trap" initial_focus="first" restore_focus={true}
      onDismiss={() => setExpanded(false)}>
      <focusScope role="list_box" accessible_name={props.label} focusable={true}
        focus_on_tab_navigation={false} onKey={onKey}
        active_descendant={activeIndex() >= 0 ? `${props.id}-option-${activeIndex()}` : undefined}>
        <rectangle width={width()} background={props.theme.surface}
          border_color={props.theme.border} border_width={1} radius={props.theme.overlayRadius}
          shadow_blur={props.theme.overlayShadowBlur} shadow_offset_y={5} shadow_color={props.theme.overlayShadow}>
          <column width="fill" max_height={280} scroll_y={true} gap={2} padding={4}>
            {options().map((option, index) => <column key={`${props.id}-row-${index}`} width="fill" gap={2}>
              {option.group && (index === 0 || options()[index - 1]?.group !== option.group)
                ? <text text={option.group} color={props.theme.muted} font_size={11} /> : null}
              <focusScope key={`${props.id}-option-${index}`} role="option"
                accessible_name={option.label} selected={value() === option.value}
                enabled={!option.disabled} keyboard_activation="enter_or_space"
                onClick={() => choose(index)}>
                <touchArea enabled={!option.disabled} mouse_cursor={option.disabled ? 'not_allowed' : 'pointer'}
                  onPointerEnter={() => { if (!option.disabled) setActiveIndex(index) }}>
                  <rectangle width="fill" height={34} opacity={option.disabled ? 0.5 : 1}
                    background={index === activeIndex() || option.value === value()
                      ? props.theme.surfaceRaised : props.theme.surface}
                    radius={props.theme.controlRadius}>
                    <row width="fill" height="fill" padding={props.theme.controlPadding} align_items="center">
                      <text text={option.label} color={props.theme.foreground} font_size={props.theme.controlFontSize} />
                    </row>
                  </rectangle>
                </touchArea>
              </focusScope>
            </column>)}
          </column>
        </rectangle>
      </focusScope>
    </popupWindow> : null}
  </column>
}
