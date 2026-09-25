/** @jsxImportSource @argui/react */
import { createContext, createElement, useCallback, useContext, useEffect, useRef, useState,
  type ReactElement, type ReactNode } from 'react'
import type { AssetRef } from '@argui/host'
import { foundationIKey, foundationINextToggleValue, type ToggleGroupChoice } from '../shared/foundation-i'
import type { Palette } from '../shared/theme'
import type { ToggleSize, ToggleVariant } from '../shared/foundation-g'

/** Orientation controls both layout and the arrow keys used to move the active item. */
export type ToggleGroupOrientation = 'horizontal' | 'vertical'

interface ToggleGroupContextValue {
  id: string
  theme(): Palette
  type(): 'single' | 'multiple'
  orientation(): ToggleGroupOrientation
  variant(): ToggleVariant
  size(): ToggleSize
  spacing(): number
  disabled(): boolean
  focused(): boolean
  selected(value: string): boolean
  active(): string | undefined
  setActive(value: string): void
  toggle(value: string): void
  choices(): readonly ToggleGroupChoice[]
  register(value: string, disabled: boolean): () => void
}

const ToggleGroupContext = createContext<ToggleGroupContextValue | null>(null)

/** Props common to the single- and multiple-selection group modes. */
interface ToggleGroupBaseProps {
  /** Stable prefix used to identify the group and its item targets. */
  id: string
  /** Palette shared by every item. */
  theme: Palette
  /** Accessible group name. */
  label: string
  /** Toggle items in their visual and keyboard-navigation order. */
  children: ReactNode
  /** Layout direction and corresponding arrow-key axis. */
  orientation?: ToggleGroupOrientation
  /** Border treatment inherited by items unless overridden. */
  variant?: ToggleVariant
  /** Control size inherited by items unless overridden. */
  size?: ToggleSize
  /** Gap between adjacent items. Defaults to zero. */
  spacing?: number
  /** Disables all items and removes the group from the tab sequence. */
  disabled?: boolean
}

/** Controlled or uncontrolled single-selection toggle group. */
export interface ToggleGroupSingleProps extends ToggleGroupBaseProps {
  type?: 'single'
  value?: string
  defaultValue?: string
  onValueChange?: (value: string) => void
}

/** Controlled or uncontrolled multiple-selection toggle group. */
export interface ToggleGroupMultipleProps extends ToggleGroupBaseProps {
  type: 'multiple'
  value?: readonly string[]
  defaultValue?: readonly string[]
  onValueChange?: (value: readonly string[]) => void
}

/** Props for a single- or multiple-selection group of native toggle buttons. */
export type ToggleGroupProps = ToggleGroupSingleProps | ToggleGroupMultipleProps

/** Combines native toggle items with roving active-descendant keyboard navigation. */
export function ReactToggleGroup(props: ToggleGroupProps): ReactElement {
  const mode = props.type ?? 'single'
  const initial: string | readonly string[] = props.type === 'multiple'
    ? props.defaultValue ?? []
    : props.defaultValue ?? ''
  const [uncontrolled, setUncontrolled] = useState<string | readonly string[]>(initial)
  const [activeValue, setActiveValue] = useState<string | undefined>(undefined)
  const [focused, setFocused] = useState(false)
  const choicesRef = useRef<ToggleGroupChoice[]>([])
  const [, setChoicesRevision] = useState(0)
  const values = (): readonly string[] => {
    const current = props.value !== undefined ? props.value : uncontrolled
    return mode === 'multiple'
      ? Array.isArray(current) ? current : []
      : typeof current === 'string' && current ? [current] : []
  }
  const selected = (value: string) => values().includes(value)
  const available = () => choicesRef.current.filter((choice) => !props.disabled && !choice.disabled)
  const active = () => {
    const items = available()
    const current = activeValue
    if (current && items.some((item) => item.value === current)) return current
    return items.find((item) => selected(item.value))?.value ?? items[0]?.value
  }
  const setActive = (value: string) => {
    if (available().some((item) => item.value === value)) setActiveValue(value)
  }
  const toggle = (value: string) => {
    if (props.disabled || !available().some((item) => item.value === value)) return
    if (props.type === 'multiple') {
      const current = values()
      const next = current.includes(value) ? current.filter((item) => item !== value) : [...current, value]
      if (props.value === undefined) setUncontrolled(next)
      props.onValueChange?.(next)
    } else {
      const current = values()[0]
      const next = current === value ? '' : value
      if (props.value === undefined) setUncontrolled(next)
      props.onValueChange?.(next)
    }
    setActiveValue(value)
  }
  const choices = () => choicesRef.current
  const register = useCallback((value: string, disabled: boolean) => {
    const current = choicesRef.current
    if (current.some((item) => item.value === value)) {
      choicesRef.current = current.map((item) => item.value === value ? { value, disabled } : item)
    } else {
      choicesRef.current = [...current, { value, disabled }]
    }
    setChoicesRevision((revision) => revision + 1)
    return () => {
      choicesRef.current = choicesRef.current.filter((item) => item.value !== value)
      setChoicesRevision((revision) => revision + 1)
    }
  }, [])
  const onKey = (payload: unknown) => {
    const key = foundationIKey(payload)
    if (!key || props.disabled) return
    setFocused(true)
    const horizontal = (props.orientation ?? 'horizontal') === 'horizontal'
    if ((horizontal && key === 'ArrowRight') || (!horizontal && key === 'ArrowDown')) {
      const next = foundationINextToggleValue(available(), active(), 1)
      if (next) setActive(next)
    } else if ((horizontal && key === 'ArrowLeft') || (!horizontal && key === 'ArrowUp')) {
      const next = foundationINextToggleValue(available(), active(), -1)
      if (next) setActive(next)
    } else if (key === 'Home') {
      const first = available()[0]
      if (first) setActive(first.value)
    } else if (key === 'End') {
      const last = available().at(-1)
      if (last) setActive(last.value)
    } else if ((key === ' ' || key === 'Space' || key === 'Enter') && active()) {
      toggle(active()!)
    }
  }
  const orientation = props.orientation ?? 'horizontal'
  const spacing = Number.isFinite(props.spacing) && props.spacing! >= 0 ? props.spacing! : 0
  const enabled = available().length > 0
  const context: ToggleGroupContextValue = {
    id: props.id, theme: () => props.theme, type: () => props.type ?? 'single',
    orientation: () => props.orientation ?? 'horizontal', variant: () => props.variant ?? 'default',
    size: () => props.size ?? 'default', spacing: () => Number.isFinite(props.spacing) && props.spacing! >= 0 ? props.spacing! : 0,
    disabled: () => !!props.disabled, focused: () => focused, selected, active, setActive, toggle, choices, register,
  }
  return createElement(ToggleGroupContext.Provider, { value: context },
    <focusScope nativeKey={props.id} role="group" accessible_name={props.label} orientation={orientation}
      multiselectable={mode === 'multiple'}
      enabled={enabled && !props.disabled} focusable={enabled && !props.disabled}
      focus_on_tab_navigation={enabled && !props.disabled}
      active_descendant={active() ? `${props.id}-item-${active()}` : undefined}
      onFocus={() => setFocused(true)} onKey={onKey} onBlur={() => setFocused(false)}>
      {orientation === 'vertical'
        ? <column width="fill" gap={spacing}>{props.children}</column>
        : <row width="fill" wrap={spacing > 0} gap={spacing} align_items="center">{props.children}</row>}
    </focusScope>
  ) as ReactElement
}

/** Props for one labeled toggle item. */
export interface ToggleGroupItemProps {
  /** Value selected by activating this item. */
  value: string
  /** Accessible name announced for the item. */
  label: string
  /** Optional visible label; defaults to `label`. */
  text?: string
  /** Optional application-owned icon rendered before the label. */
  icon?: AssetRef
  /** Disables this item without disabling the group. */
  disabled?: boolean
  /** Item-level border override. */
  variant?: ToggleVariant
  /** Item-level size override. */
  size?: ToggleSize
  /** Custom visual content, for example a native icon-and-text composition. */
  children?: ReactNode
}

/** Renders an accessible toggle item managed by the surrounding group. */
export function ReactToggleGroupItem(props: ToggleGroupItemProps): ReactElement {
  const group = useContext(ToggleGroupContext)
  if (!group) throw new Error('ToggleGroupItem must be rendered inside ToggleGroup')
  useEffect(() => group.register(props.value, !!props.disabled), [group.register, props.value, props.disabled])
  const [hovered, setHovered] = useState(false)
  const [pressed, setPressed] = useState(false)
  const selected = group.selected(props.value)
  const disabled = group.disabled() || !!props.disabled
  const focused = group.focused() && group.active() === props.value
  const variant = props.variant ?? group.variant()
  const size = props.size ?? group.size()
  const dimension = size === 'sm' ? 32 : size === 'lg' ? 40 : 36
  const padding = size === 'sm' ? 6 : size === 'lg' ? 10 : 8
  const theme = group.theme()
  const fill = selected ? theme.accent : hovered ? theme.surfaceHover : '#00000000'
  const color = selected ? theme.accentText : theme.foreground
  const border = focused ? theme.accent : variant === 'outline' ? theme.border : '#00000000'
  const index = group.choices().findIndex((item) => item.value === props.value)
  const click = () => {
    if (disabled) return
    group.setActive(props.value)
    group.toggle(props.value)
  }
  return <focusScope nativeKey={`${group.id}-item-${props.value}`} role="button" accessible_name={props.label}
    pressed_state={selected} enabled={!disabled} accessible_disabled={disabled}
    position_in_set={index >= 0 ? index + 1 : undefined} set_size={group.choices().length || undefined}
    focusable={false} focus_on_tab_navigation={false} keyboard_activation="none"
    onClick={click} onFocus={() => group.setActive(props.value)}>
    <touchArea enabled={!disabled} mouse_cursor={disabled ? 'not_allowed' : 'pointer'}
      onPointerEnter={() => setHovered(true)} onPointerLeave={() => { setHovered(false); setPressed(false) }}
      onPointerDown={() => { group.setActive(props.value); setPressed(true) }}
      onPointerUp={() => setPressed(false)} onPointerCancel={() => setPressed(false)}>
      <rectangle min_width={dimension} height={dimension} background={pressed ? theme.surfacePressed : fill}
        border_color={border} border_width={variant === 'outline' || focused ? 1 : 0}
        radius={theme.controlRadius} opacity={disabled ? 0.48 : 1}>
        <row width="fill" height="fill" padding={padding} gap={8} align_items="center" justify_content="center">
          {props.children ?? <>
            {props.icon ? <svg source={props.icon} color={color} width={16} height={16} accessible_hidden={true} /> : null}
            <text text={props.text ?? props.label} color={color}
              font_size={size === 'sm' ? 12 : size === 'lg' ? 15 : 14} weight={500} no_wrap={true} />
          </>}
        </row>
      </rectangle>
    </touchArea>
  </focusScope>
}
