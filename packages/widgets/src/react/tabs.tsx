/** @jsxImportSource @argui/react */
import { createContext, createElement, useCallback, useContext, useEffect, useRef, useState, type ReactElement, type ReactNode } from 'react'
import type { Palette } from '../shared/theme'
import {
  surfaceAIdPart, surfaceAKey, type SurfaceATabsActivationMode,
  type SurfaceATabsOrientation, type SurfaceATabsVariant,
} from '../shared/surface-a'

/** Props for a controlled or default-valued tabs root. */
export interface TabsProps {
  id: string
  theme: Palette
  children: ReactNode
  value?: string
  defaultValue?: string
  orientation?: SurfaceATabsOrientation
  activationMode?: SurfaceATabsActivationMode
  label?: string
  onValueChange?: (value: string) => void
}

/** Props for the tab list. */
export interface TabsListProps {
  children: ReactNode
  label?: string
  variant?: SurfaceATabsVariant
}

/** Props for one tab selector. */
export interface TabsTriggerProps {
  value: string
  children: ReactNode
  label?: string
  disabled?: boolean
}

/** Props for one tab panel. */
export interface TabsContentProps {
  value: string
  children: ReactNode
}

interface TabRecord {
  value: string
  disabled: () => boolean
}

interface RootValue {
  id: string
  theme: Palette
  orientation: () => SurfaceATabsOrientation
  activationMode: () => SurfaceATabsActivationMode
  selected: () => string | undefined
  active: () => string | undefined
  setActive: (value: string) => void
  focused: () => boolean
  setFocused: (focused: boolean) => void
  choose: (value: string) => void
  register: (value: string, disabled: () => boolean) => () => void
  records: () => readonly TabRecord[]
}

const RootContext = createContext<RootValue | null>(null)
const ListVariantContext = createContext<SurfaceATabsVariant>('default')

/** Groups related tab controls and panels around shared selection state. */
export function Tabs(props: TabsProps): ReactElement {
  const [uncontrolled, setUncontrolled] = useState(props.value ?? props.defaultValue ?? '')
  const [active, setActive] = useState(props.value ?? props.defaultValue ?? '')
  const [focused, setFocused] = useState(false)
  const records = useRef<TabRecord[]>([])
  const selected = () => (props.value ?? uncontrolled) || undefined
  const selectedRef = useRef(selected())
  selectedRef.current = selected()
  const controlled = useRef(props.value !== undefined)
  controlled.current = props.value !== undefined
  const activeValue = () => active || selected()
  const choose = (value: string) => {
    const changed = selected() !== value
    if (changed && props.value === undefined) setUncontrolled(value)
    setActive(value)
    if (changed) props.onValueChange?.(value)
  }
  const register = useCallback((value: string, disabled: () => boolean) => {
    if (!records.current.some((entry) => entry.value === value)) records.current.push({ value, disabled })
    if (!selectedRef.current && !controlled.current && !disabled()) {
      selectedRef.current = value
      setUncontrolled(value)
      setActive(value)
    }
    return () => {
      records.current = records.current.filter((entry) => entry.value !== value)
      const fallback = records.current.find((entry) => !entry.disabled())?.value ?? ''
      setActive((current) => current === value ? fallback : current)
      if (!controlled.current && selectedRef.current === value) {
        selectedRef.current = fallback || undefined
        setUncontrolled(fallback)
      }
    }
  }, [])
  const context: RootValue = {
    id: props.id, theme: props.theme, orientation: () => props.orientation ?? 'horizontal',
    activationMode: () => props.activationMode ?? 'automatic', selected, active: activeValue,
    setActive, focused: () => focused, setFocused, choose, register,
    records: () => records.current,
  }
  return createElement(RootContext.Provider, { value: context },
    <column width="fill" gap={10} role={props.label ? 'group' : undefined} accessible_name={props.label}>
      {props.children}
    </column>
  ) as ReactElement
}

/** Draws a horizontal or vertical composite tab list with arrow-key movement. */
export function TabsList(props: TabsListProps): ReactElement {
  const root = useContext(RootContext)
  if (!root) throw new Error('TabsList must be rendered inside Tabs')
  const orientation = root.orientation()
  const variant = props.variant ?? 'default'
  const move = (key: string) => {
    const available = root.records().filter((record) => !record.disabled())
    if (!available.length) return
    const current = root.active() ?? root.selected() ?? available[0]!.value
    const index = available.findIndex((record) => record.value === current)
    const forward = orientation === 'horizontal' ? 'ArrowRight' : 'ArrowDown'
    const backward = orientation === 'horizontal' ? 'ArrowLeft' : 'ArrowUp'
    if (key === 'Enter' || key === ' ') {
      root.choose(current)
      return
    }
    let next = index
    if (key === 'Home') next = 0
    else if (key === 'End') next = available.length - 1
    else if (key === forward) next = (index + 1 + available.length) % available.length
    else if (key === backward) next = index < 0 ? available.length - 1 : (index - 1 + available.length) % available.length
    else return
    const value = available[next]!.value
    if (root.activationMode() === 'automatic') root.choose(value)
    else root.setActive(value)
  }
  const active = root.active() ?? root.selected()
  const listId = `${root.id}-tab-list`
  return createElement(ListVariantContext.Provider, { value: variant },
    <focusScope nativeKey={listId} role="tab_list" accessible_name={props.label ?? 'Tabs'}
      orientation={orientation} active_descendant={active ? `${root.id}-tab-${surfaceAIdPart(active)}` : undefined}
      focus_on_click={true} onFocus={() => root.setFocused(true)} onBlur={() => root.setFocused(false)}
      onKey={(payload) => { const key = surfaceAKey(payload); if (key) move(key) }}>
      <rectangle width={orientation === 'horizontal' ? 'fill' : 152}
        background={variant === 'default' ? root.theme.surfaceRaised : '#00000000'}
        border_color={variant === 'default' ? root.theme.border : '#00000000'}
        border_width={variant === 'default' ? 1 : 0} radius={variant === 'default' ? root.theme.controlRadius : 0}>
        {orientation === 'horizontal'
          ? <row width="fill" gap={3} padding={variant === 'default' ? 3 : 0}>{props.children}</row>
          : <column width={144} gap={3} padding={variant === 'default' ? 3 : 0}>{props.children}</column>}
      </rectangle>
    </focusScope>
  ) as ReactElement
}

/** Provides an accessible tab button associated with its matching panel. */
export function TabsTrigger(props: TabsTriggerProps): ReactElement {
  const root = useContext(RootContext)
  const variant = useContext(ListVariantContext)
  if (!root) throw new Error('TabsTrigger must be rendered inside TabsList')
  const disabled = () => !!props.disabled
  useEffect(() => root.register(props.value, disabled), [root.register, props.value, props.disabled])
  const selected = root.selected() === props.value
  const active = root.active() === props.value
  const key = `${root.id}-tab-${surfaceAIdPart(props.value)}`
  const panel = `${root.id}-panel-${surfaceAIdPart(props.value)}`
  const activate = () => { if (!disabled()) root.choose(props.value) }
  return <focusScope nativeKey={key} role="tab" accessible_name={props.label} controls={panel}
    selected={selected} enabled={!disabled()} accessible_disabled={disabled()} focusable={false}
    keyboard_activation="none" onClick={activate}
    onSemanticAction={(payload) => { if (payload.action === 'click') activate() }}>
    <rectangle width="fill" height={36}
      background={variant === 'default' && selected ? root.theme.surface : '#00000000'}
      border_color={root.focused() && active ? root.theme.accent : 'transparent'}
      border_width={root.focused() && active ? 1 : 0} radius={root.theme.controlRadius}
      opacity={disabled() ? 0.48 : 1}>
      <column width="fill" height="fill" gap={0}>
        <row width="fill" grow={1} padding={10} gap={6} align_items="center" justify_content="center">
          {props.children}
        </row>
        {variant === 'line' ? <rectangle width="fill" height={2}
          background={selected ? root.theme.accent : '#00000000'} /> : null}
      </column>
    </rectangle>
  </focusScope>
}

/** Shows its content only while its matching tab is selected. */
export function TabsContent(props: TabsContentProps): ReactElement {
  const root = useContext(RootContext)
  if (!root) throw new Error('TabsContent must be rendered inside Tabs')
  const selected = root.selected() === props.value
  const id = `${root.id}-tab-${surfaceAIdPart(props.value)}`
  return <column nativeKey={`${root.id}-panel-${surfaceAIdPart(props.value)}`} width="fill"
    visible={selected} accessible_hidden={!selected} role="tab_panel" labelled_by={id} padding={12}>
    {props.children}
  </column>
}
