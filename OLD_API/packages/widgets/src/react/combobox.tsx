/** @jsxImportSource @argui/react */
import { useRef, useState, type ReactElement } from 'react'
import { foundationIKey } from '../shared/foundation-i'
import { InputEditController } from '../shared/input-edit'
import { nextSelectIndex, resolveSelectOptions, type SelectOption } from '../shared/select-options'
import { selectionTint, type Palette } from '../shared/theme'
import { useWidgetIcons } from './assets'

/** Props for a searchable, single-value native combobox. */
export interface ComboboxProps {
  /** Stable native key used by the text input and popup anchor. */
  id: string
  /** Accessible name for the editable control and its option list. */
  label: string
  /** Palette used by the input, result list, and selected option. */
  theme: Palette
  /** Available values, optionally grouped or disabled. */
  options: readonly SelectOption[]
  /** Current selected value; omit for internal state. */
  value?: string
  /** Initial selection used only when `value` is omitted. */
  defaultValue?: string
  /** Called after selecting an option or clearing the selection. */
  onValueChange?: (value: string) => void
  /** Current search query; pair with `onQueryChange` for controlled search. */
  query?: string
  /** Initial search query used only while `query` is omitted. */
  defaultQuery?: string
  /** Called whenever the user changes the search text. */
  onQueryChange?: (query: string) => void
  /** Placeholder shown when no option is selected. */
  placeholder?: string
  /** Placeholder shown while filtering. */
  searchPlaceholder?: string
  /** Message shown when no options match. */
  emptyLabel?: string
  /** Disables editing and option selection. */
  disabled?: boolean
  /** Input and popup width in logical pixels. Defaults to 260. */
  width?: number
  /** Whether to show a keyboard-accessible clear action. */
  clearable?: boolean
  /** Whether to draw the accessible label above the editor. */
  showLabel?: boolean
}

/** Props alias retained to name the React adapter explicitly. */
export type ReactComboboxProps = ComboboxProps

/** Filters and selects one value using a native text editor and anchored popup. */
export function ReactCombobox(props: ReactComboboxProps): ReactElement {
  const icons = useWidgetIcons()
  const [expanded, setExpanded] = useState(false)
  const [focused, setFocused] = useState(false)
  const [activeIndex, setActiveIndex] = useState(-1)
  const [uncontrolledValue, setUncontrolledValue] = useState(props.defaultValue ?? '')
  const [uncontrolledQuery, setUncontrolledQuery] = useState(props.defaultQuery ?? '')
  const edits = useRef<InputEditController | null>(null)
  const options = resolveSelectOptions(props.options)
  const value = props.value ?? uncontrolledValue
  const query = props.query ?? uncontrolledQuery
  const chosen = options.find((option) => option.value === value)
  const filtered = () => {
    const needle = query.trim().toLocaleLowerCase()
    return options.map((option, index) => ({ option, index })).filter(({ option }) =>
      !needle || option.label.toLocaleLowerCase().includes(needle) || option.value.toLocaleLowerCase().includes(needle))
  }
  const width = Number.isFinite(props.width) && (props.width ?? 0) > 0 ? props.width! : 260
  const setQuery = (next: string) => {
    if (props.query === undefined) setUncontrolledQuery(next)
    props.onQueryChange?.(next)
  }
  const open = () => {
    if (props.disabled) return
    setExpanded(true)
    setQuery('')
    const selected = filtered().find(({ option }) => option.value === value && !option.disabled)
    setActiveIndex(selected?.index ?? nextSelectIndex(options, -1, 1))
  }
  const close = () => { setExpanded(false); setQuery('') }
  const choose = (index: number) => {
    const option = options[index]
    if (!option || option.disabled || props.disabled) return
    if (props.value === undefined) setUncontrolledValue(option.value)
    props.onValueChange?.(option.value)
    close()
  }
  const onKey = (payload: unknown) => {
    const key = foundationIKey(payload)
    if (!key || props.disabled) return
    const matches = filtered()
    if (key === 'Escape') { if (expanded) close() }
    else if (key === 'ArrowDown' || key === 'ArrowUp') {
      if (!expanded) open()
      else {
        const filteredIndex = matches.findIndex(({ index }) => index === activeIndex)
        const next = nextSelectIndex(matches.map(({ option }) => option), filteredIndex < 0
          ? (key === 'ArrowDown' ? -1 : 0) : filteredIndex, key === 'ArrowDown' ? 1 : -1)
        if (next >= 0) setActiveIndex(matches[next]!.index)
      }
    } else if (key === 'Home' || key === 'End') {
      const next = nextSelectIndex(matches.map(({ option }) => option), key === 'Home' ? -1 : 0, key === 'Home' ? 1 : -1)
      if (next >= 0) setActiveIndex(matches[next]!.index)
    } else if (key === 'Enter') {
      if (expanded) choose(activeIndex)
      else open()
    }
  }
  const displayValue = expanded ? query : chosen?.label ?? ''
  edits.current ??= new InputEditController(displayValue)
  const active = () => filtered().find(({ option, index }) => index === activeIndex && !option.disabled)?.index
    ?? filtered().find(({ option }) => !option.disabled)?.index
  return <column width="fill" gap={6}>
    {props.showLabel !== false ? <text text={props.label} color={props.theme.muted} font_size={12} /> : null}
    <focusScope nativeKey={props.id} role="combo_box" accessible_name={props.label}
      accessible_value={chosen?.label ?? ''} controls={`${props.id}-popup`} has_popup="list_box"
      expandable={true} expanded={expanded} active_descendant={expanded && active() !== undefined ? `${props.id}-option-${active()}` : undefined}
      enabled={!props.disabled} onKey={onKey} onClick={() => { if (!expanded) open() }}>
      <rectangle width={width} height={props.theme.inputHeight}
        background={props.disabled ? props.theme.surfaceRaised : props.theme.surface}
        border_color={focused ? props.theme.accent : props.theme.border} border_width={focused ? 2 : 1}
        radius={props.theme.controlRadius} opacity={props.disabled ? 0.55 : 1}>
        <row width="fill" height="fill" min_width={0} gap={7} padding_left={props.theme.controlPadding}
          padding_right={props.theme.controlPadding} align_items="center">
          <container grow={1} min_width={0}>
            <textInput nativeKey={`${props.id}-input`} role="text_input" width="fill" height={24} clip={true}
              value={displayValue} placeholder={expanded ? props.searchPlaceholder ?? props.placeholder ?? 'Search options…' : props.placeholder ?? 'Choose an option…'}
              label={props.label} enabled={!props.disabled} background="#00000000" text_color={props.theme.foreground}
              placeholder_color={props.theme.muted} caret_color={props.theme.accent} selection_color={selectionTint(props.theme.accent)}
              onFocus={() => { setFocused(true); if (!expanded) open() }}
              onBlur={() => setFocused(false)} onEdit={(payload) => edits.current!.apply(payload, displayValue, setQuery)} />
          </container>
          {props.clearable && value ? <focusScope role="button" accessible_name={`Clear ${props.label}`} enabled={!props.disabled}
            keyboard_activation="enter_or_space" onClick={() => {
              if (props.value === undefined) setUncontrolledValue('')
              props.onValueChange?.('')
              setQuery('')
            }}>
            <touchArea enabled={!props.disabled} mouse_cursor="pointer">
              {icons.x ? <svg source={icons.x} color={props.theme.muted} width={16} height={16} />
                : <text text="×" color={props.theme.muted} font_size={18} />}
            </touchArea>
          </focusScope> : null}
          {icons.chevronDown ? <svg source={icons.chevronDown} color={props.theme.muted}
            width={16} height={16} rotation={expanded ? 180 : 0} accessible_hidden={true} />
            : <text text={expanded ? '⌃' : '⌄'} color={props.theme.muted} font_size={14} accessible_hidden={true} />}
        </row>
      </rectangle>
    </focusScope>
    {expanded ? <popupWindow nativeKey={`${props.id}-popup`} anchor={props.id} placement="bottom_start" width={width}
      window_layer="popover" dismiss_policy="outside_pointer_or_escape" restore_focus={true} onDismiss={close}>
      <focusScope role="list_box" accessible_name={`${props.label} options`} multiselectable={false} focusable={false}
        active_descendant={active() !== undefined ? `${props.id}-option-${active()}` : undefined}>
        <rectangle width={width} background={props.theme.surface} border_color={props.theme.border} border_width={1}
          radius={props.theme.overlayRadius} shadow_blur={props.theme.overlayShadowBlur} shadow_offset_y={5}
          shadow_color={props.theme.overlayShadow}>
          <column width="fill" max_height={260} scroll_y={true} gap={2} padding={4}>
            {filtered().length === 0
              ? <row width="fill" height={42} align_items="center" justify_content="center">
                <text text={props.emptyLabel ?? 'No matching options.'} color={props.theme.muted} font_size={13} role="status" live="polite" />
              </row>
              : filtered().map(({ option, index }) => <column key={`${props.id}-option-row-${index}`} width="fill">
                {option.group && (index === 0 || options[index - 1]?.group !== option.group)
                  ? <text text={option.group} color={props.theme.muted} font_size={11} /> : null}
                <focusScope nativeKey={`${props.id}-option-${index}`} role="option" accessible_name={option.label}
                  selected={value === option.value} enabled={!option.disabled} accessible_disabled={option.disabled}
                  position_in_set={filtered().findIndex((item) => item.index === index) + 1} set_size={filtered().length}
                  onClick={() => choose(index)}>
                  <touchArea enabled={!option.disabled} mouse_cursor={option.disabled ? 'not_allowed' : 'pointer'}
                    onPointerEnter={() => { if (!option.disabled) setActiveIndex(index) }}>
                    <rectangle width="fill" height={34} radius={props.theme.controlRadius}
                      background={active() === index ? props.theme.surfaceRaised : props.theme.surface}
                      opacity={option.disabled ? 0.5 : 1}>
                      <row width="fill" height="fill" gap={8} padding_left={props.theme.controlPadding}
                        padding_right={props.theme.controlPadding} align_items="center">
                        <text text={option.label} color={option.disabled ? props.theme.muted : props.theme.foreground}
                          font_size={props.theme.controlFontSize} />
                        <container grow={1} />
                        {value === option.value ? icons.check
                          ? <svg source={icons.check} color={props.theme.foreground} width={16} height={16} accessible_hidden={true} />
                          : <text text="✓" color={props.theme.foreground} font_size={14} accessible_hidden={true} /> : null}
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
