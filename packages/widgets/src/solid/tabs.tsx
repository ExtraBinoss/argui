import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { createContext, onCleanup, onMount, useContext } from 'solid-js'
import type { JSX as SolidJSX } from 'solid-js'
import type { Palette } from '../shared/theme'
import {
  surfaceAIdPart, surfaceAKey, type SurfaceATabsActivationMode,
  type SurfaceATabsOrientation, type SurfaceATabsVariant,
} from '../shared/surface-a'

/** Props for a controlled or default-valued tabs root. */
export interface TabsProps {
  id: string
  theme: Palette
  children: JSX.Element
  value?: string
  defaultValue?: string
  orientation?: SurfaceATabsOrientation
  activationMode?: SurfaceATabsActivationMode
  label?: string
  onValueChange?: (value: string) => void
}

/** Props for the tab list. */
export interface TabsListProps {
  children: JSX.Element
  label?: string
  variant?: SurfaceATabsVariant
}

/** Props for one tab selector. */
export interface TabsTriggerProps {
  value: string
  children: JSX.Element
  label?: string
  disabled?: boolean
}

/** Props for one tab panel. */
export interface TabsContentProps {
  value: string
  children: JSX.Element
}

interface TabRecord {
  value: string
  disabled: () => boolean
}

interface TabsContext {
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

const RootContext = createContext<TabsContext>()
const ListVariantContext = createContext<() => SurfaceATabsVariant>(() => 'default' as SurfaceATabsVariant)

/** Groups related tab controls and panels around shared selection state. */
export function Tabs(props: TabsProps): JSX.Element {
  const [uncontrolled, setUncontrolled] = createSignal(props.value ?? props.defaultValue ?? '')
  const [active, setActive] = createSignal(props.value ?? props.defaultValue ?? '')
  const [focused, setFocused] = createSignal(false)
  const records: TabRecord[] = []
  const selected = () => (props.value ?? uncontrolled()) || undefined
  const activeValue = () => active() || selected()
  const choose = (value: string) => {
    const changed = selected() !== value
    if (changed && props.value === undefined) setUncontrolled(value)
    setActive(value)
    if (changed) props.onValueChange?.(value)
  }
  const register = (value: string, disabled: () => boolean) => {
    if (!records.some((entry) => entry.value === value)) records.push({ value, disabled })
    if (!selected() && !disabled()) {
      if (props.value === undefined) setUncontrolled(value)
      setActive(value)
    }
    return () => {
      const index = records.findIndex((entry) => entry.value === value)
      if (index >= 0) records.splice(index, 1)
      if (active() === value) setActive(records.find((entry) => !entry.disabled())?.value ?? '')
      if (props.value === undefined && uncontrolled() === value) {
        setUncontrolled(records.find((entry) => !entry.disabled())?.value ?? '')
      }
    }
  }
  const context: TabsContext = {
    id: props.id, theme: props.theme, orientation: () => props.orientation ?? 'horizontal',
    activationMode: () => props.activationMode ?? 'automatic', selected, active: activeValue,
    setActive, focused, setFocused, choose, register, records: () => records,
  }
  return <RootContext.Provider value={context}>{(<>
    <column width="fill" gap={10} role={props.label ? 'group' : undefined} accessible_name={props.label}>
      {props.children}
    </column>
  </>) as unknown as SolidJSX.Element}</RootContext.Provider>
}

/** Draws a horizontal or vertical composite tab list with arrow-key movement. */
export function TabsList(props: TabsListProps): JSX.Element {
  const root = useContext(RootContext)
  if (!root) throw new Error('TabsList must be rendered inside Tabs')
  const orientation = () => root.orientation()
  const variant = () => props.variant ?? 'default'
  const move = (key: string) => {
    const available = root.records().filter((record) => !record.disabled())
    if (!available.length) return
    const current = root.active() ?? root.selected() ?? available[0]!.value
    const index = available.findIndex((record) => record.value === current)
    let next = index
    const forward = orientation() === 'horizontal' ? 'ArrowRight' : 'ArrowDown'
    const backward = orientation() === 'horizontal' ? 'ArrowLeft' : 'ArrowUp'
    if (key === 'Enter' || key === ' ') {
      root.choose(current)
      return
    }
    if (key === 'Home') next = 0
    else if (key === 'End') next = available.length - 1
    else if (key === forward) next = (index + 1 + available.length) % available.length
    else if (key === backward) next = index < 0 ? available.length - 1 : (index - 1 + available.length) % available.length
    else return
    if (next !== index || key === forward || key === backward || key === 'Home' || key === 'End') {
      const value = available[next]!.value
      if (root.activationMode() === 'automatic') root.choose(value)
      else root.setActive(value)
    }
  }
  const id = root.active() ?? root.selected()
  const listId = `${root.id}-tab-list`
  const contents = <>{orientation() === 'horizontal'
    ? <row width="fill" gap={3} padding={variant() === 'default' ? 3 : 0}>{props.children}</row>
    : <column width={144} gap={3} padding={variant() === 'default' ? 3 : 0}>{props.children}</column>}</>
  return <ListVariantContext.Provider value={variant}>{(<focusScope key={listId} role="tab_list" accessible_name={props.label ?? 'Tabs'}
    orientation={orientation()} active_descendant={id ? `${root.id}-tab-${surfaceAIdPart(id)}` : undefined}
    focus_on_click={true} onFocus={() => root.setFocused(true)} onBlur={() => root.setFocused(false)}
    onKey={(payload) => { const key = surfaceAKey(payload); if (key) move(key) }}>
    <rectangle width={orientation() === 'horizontal' ? 'fill' : 152}
      background={variant() === 'default' ? root.theme.surfaceRaised : '#00000000'}
      border_color={variant() === 'default' ? root.theme.border : '#00000000'}
      border_width={variant() === 'default' ? 1 : 0} radius={variant() === 'default' ? root.theme.controlRadius : 0}>
      {contents}
    </rectangle>
  </focusScope>) as unknown as SolidJSX.Element}</ListVariantContext.Provider>
}

/** Provides an accessible tab button associated with its matching panel. */
export function TabsTrigger(props: TabsTriggerProps): JSX.Element {
  const root = useContext(RootContext)
  if (!root) throw new Error('TabsTrigger must be rendered inside TabsList')
  const variant = useContext(ListVariantContext)
  const disabled = () => !!props.disabled
  let unregister = () => {}
  onMount(() => { unregister = root.register(props.value, disabled) })
  onCleanup(() => unregister())
  const selected = () => root.selected() === props.value
  const active = () => root.active() === props.value
  const key = `${root.id}-tab-${surfaceAIdPart(props.value)}`
  const panel = `${root.id}-panel-${surfaceAIdPart(props.value)}`
  const activate = () => { if (!disabled()) root.choose(props.value) }
  return <focusScope key={key} role="tab" accessible_name={props.label} controls={panel}
    selected={selected()} enabled={!disabled()} accessible_disabled={disabled()} focusable={false}
    keyboard_activation="none" onClick={activate}
    onSemanticAction={(payload) => { if (payload.action === 'click') activate() }}>
    <rectangle width="fill" height={36}
      background={variant() === 'default' && selected() ? root.theme.surface : '#00000000'}
      border_color={root.focused() && active() ? root.theme.accent : 'transparent'}
      border_width={root.focused() && active() ? 1 : 0} radius={root.theme.controlRadius}
      opacity={disabled() ? 0.48 : 1}>
      <column width="fill" height="fill" gap={0}>
        <row width="fill" grow={1} padding={10} gap={6} align_items="center" justify_content="center">
          {props.children}
        </row>
        {variantIndicator(root, selected(), variant())}
      </column>
    </rectangle>
  </focusScope>
}

/** Shows its content only while its matching tab is selected. */
export function TabsContent(props: TabsContentProps): JSX.Element {
  const root = useContext(RootContext)
  if (!root) throw new Error('TabsContent must be rendered inside Tabs')
  const selected = () => root.selected() === props.value
  const id = `${root.id}-tab-${surfaceAIdPart(props.value)}`
  return <column key={`${root.id}-panel-${surfaceAIdPart(props.value)}`} width="fill"
    visible={selected()} accessible_hidden={!selected()} role="tab_panel" labelled_by={id} padding={12}>
    {props.children}
  </column>
}

function variantIndicator(root: TabsContext, selected: boolean, variant: SurfaceATabsVariant): JSX.Element | null {
  return variant === 'line'
    ? <rectangle width="fill" height={2} background={selected ? root.theme.accent : '#00000000'} />
    : null
}
