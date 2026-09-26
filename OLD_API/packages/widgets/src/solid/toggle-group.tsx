import { createSignal } from '@argui/solid'
import type { AssetRef } from '@argui/host'
import type { JSX } from '@argui/solid/jsx-runtime'
import { createContext, onCleanup, useContext, type JSX as SolidJSX } from 'solid-js'
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

const ToggleGroupContext = createContext<ToggleGroupContextValue>()

/** Props common to the single- and multiple-selection group modes. */
interface ToggleGroupBaseProps {
  /** Stable prefix used to identify the group and its item targets. */
  id: string
  /** Palette shared by every item. */
  theme: Palette
  /** Accessible group name. */
  label: string
  /** Toggle items in their visual and keyboard-navigation order. */
  children: JSX.Element
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
export function ToggleGroup(props: ToggleGroupProps): JSX.Element {
  const mode = () => props.type ?? 'single'
  const initial: string | readonly string[] = props.type === 'multiple'
    ? props.defaultValue ?? []
    : props.defaultValue ?? ''
  const [uncontrolled, setUncontrolled] = createSignal<string | readonly string[]>(initial)
  const [choices, setChoices] = createSignal<readonly ToggleGroupChoice[]>([])
  const [activeValue, setActiveValue] = createSignal<string | undefined>(undefined)
  const [focused, setFocused] = createSignal(false)
  const values = (): readonly string[] => {
    if (mode() === 'multiple') {
      const current = props.value !== undefined ? props.value : uncontrolled()
      return Array.isArray(current) ? current : []
    }
    const current = props.value !== undefined ? props.value : uncontrolled()
    return typeof current === 'string' && current ? [current] : []
  }
  const selected = (value: string) => values().includes(value)
  const available = () => choices().filter((choice) => !props.disabled && !choice.disabled)
  const active = () => {
    const items = available()
    const current = activeValue()
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
  const register = (value: string, disabled: boolean) => {
    setChoices((items) => items.some((item) => item.value === value)
      ? items.map((item) => item.value === value ? { value, disabled } : item)
      : [...items, { value, disabled }])
    return () => setChoices((items) => items.filter((item) => item.value !== value))
  }
  const onKey = (payload: unknown) => {
    const key = foundationIKey(payload)
    if (!key || props.disabled) return
    setFocused(true)
    const horizontal = (orientation() === 'horizontal')
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
  const orientation = () => props.orientation ?? 'horizontal'
  const spacing = () => Number.isFinite(props.spacing) && props.spacing! >= 0 ? props.spacing! : 0
  const enabled = () => available().length > 0 && !props.disabled
  const content = (
    <focusScope key={props.id} role="group" accessible_name={props.label} orientation={orientation()}
      multiselectable={mode() === 'multiple'} enabled={enabled()} focusable={enabled()} focus_on_tab_navigation={enabled()}
      active_descendant={active() ? `${props.id}-item-${active()}` : undefined}
      onFocus={() => setFocused(true)} onKey={onKey} onBlur={() => setFocused(false)}>
      {orientation() === 'vertical'
        ? <column width="fill" gap={spacing()}>{props.children}</column>
        : <row width="fill" wrap={spacing() > 0} gap={spacing()} align_items="center">{props.children}</row>}
    </focusScope>
  ) as SolidJSX.Element
  const context: ToggleGroupContextValue = {
    id: props.id, theme: () => props.theme, type: mode, orientation,
    variant: () => props.variant ?? 'default', size: () => props.size ?? 'default', spacing,
    disabled: () => !!props.disabled, focused: () => focused(),
    selected, active, setActive, toggle, choices, register,
  }
  return <ToggleGroupContext.Provider value={context}>{content}</ToggleGroupContext.Provider>
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
  children?: JSX.Element
}

/** Renders an accessible toggle item managed by the surrounding group. */
export function ToggleGroupItem(props: ToggleGroupItemProps): JSX.Element {
  const group = useContext(ToggleGroupContext)
  if (!group) throw new Error('ToggleGroupItem must be rendered inside ToggleGroup')
  const dispose = group.register(props.value, !!props.disabled)
  onCleanup(dispose)
  const [hovered, setHovered] = createSignal(false)
  const [pressed, setPressed] = createSignal(false)
  const selected = () => group.selected(props.value)
  const disabled = () => group.disabled() || !!props.disabled
  const focused = () => group.focused() && group.active() === props.value
  const variant = () => props.variant ?? group.variant()
  const size = () => props.size ?? group.size()
  const dimension = () => size() === 'sm' ? 32 : size() === 'lg' ? 40 : 36
  const padding = () => size() === 'sm' ? 6 : size() === 'lg' ? 10 : 8
  const fill = () => selected() ? group.theme().accent : hovered() ? group.theme().surfaceHover : '#00000000'
  const color = () => selected() ? group.theme().accentText : group.theme().foreground
  const border = () => focused() ? group.theme().accent : variant() === 'outline' ? group.theme().border : '#00000000'
  const index = () => group.choices().findIndex((item) => item.value === props.value)
  const click = () => {
    if (disabled()) return
    group.setActive(props.value)
    group.toggle(props.value)
  }
  return <focusScope key={`${group.id}-item-${props.value}`} role="button" accessible_name={props.label}
    pressed_state={selected()} enabled={!disabled()} accessible_disabled={disabled()}
    position_in_set={index() >= 0 ? index() + 1 : undefined} set_size={group.choices().length || undefined}
    focusable={false} focus_on_tab_navigation={false} keyboard_activation="none"
    onClick={click} onFocus={() => group.setActive(props.value)}>
    <touchArea enabled={!disabled()} mouse_cursor={disabled() ? 'not_allowed' : 'pointer'}
      onPointerEnter={() => setHovered(true)} onPointerLeave={() => { setHovered(false); setPressed(false) }}
      onPointerDown={() => { group.setActive(props.value); setPressed(true) }}
      onPointerUp={() => setPressed(false)} onPointerCancel={() => setPressed(false)}>
      <rectangle min_width={dimension()} height={dimension()} background={pressed() ? group.theme().surfacePressed : fill()}
        border_color={border()} border_width={variant() === 'outline' || focused() ? 1 : 0}
        radius={group.theme().controlRadius} opacity={disabled() ? 0.48 : 1}>
        <row width="fill" height="fill" padding={padding()} gap={8} align_items="center" justify_content="center">
          {props.children ?? <>
            {props.icon ? <svg source={props.icon} color={color()} width={16} height={16} accessible_hidden={true} /> : null}
            <text text={props.text ?? props.label} color={color()}
              font_size={size() === 'sm' ? 12 : size() === 'lg' ? 15 : 14} weight={500} no_wrap={true} />
          </>}
        </row>
      </rectangle>
    </touchArea>
  </focusScope>
}
