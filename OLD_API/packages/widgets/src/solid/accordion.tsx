import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { createContext, onCleanup, onMount, useContext } from 'solid-js'
import type { JSX as SolidJSX } from 'solid-js'
import type { Palette } from '../shared/theme'
import { surfaceAIdPart, surfaceAKey, type SurfaceAAccordionValue } from '../shared/surface-a'

/** Accordion root props, including controlled and default state. */
export interface AccordionProps {
  id: string
  theme: Palette
  children: JSX.Element
  type?: 'single' | 'multiple'
  value?: SurfaceAAccordionValue
  defaultValue?: SurfaceAAccordionValue
  collapsible?: boolean
  disabled?: boolean
  label?: string
  onValueChange?: (value: string | string[]) => void
}

/** Accordion item props. */
export interface AccordionItemProps {
  value: string
  children: JSX.Element
  disabled?: boolean
}

/** Trigger props for an accordion item. */
export interface AccordionTriggerProps {
  children: JSX.Element
  label?: string
}

/** Content props for an accordion item. */
export interface AccordionContentProps {
  children: JSX.Element
}

interface AccordionItemRecord {
  value: string
  disabled: () => boolean
}

interface AccordionRootContext {
  id: string
  theme: Palette
  type: 'single' | 'multiple'
  value: () => SurfaceAAccordionValue
  active: () => string
  setActive: (value: string) => void
  isDisabled: () => boolean
  isOpen: (value: string) => boolean
  setOpen: (value: string, open: boolean) => void
  toggle: (value: string) => void
  register: (value: string, disabled: () => boolean) => () => void
  move: (key: string) => void
}

interface AccordionItemContext {
  value: string
  disabled: () => boolean
  open: () => boolean
}

const RootContext = createContext<AccordionRootContext>()
const ItemContext = createContext<AccordionItemContext>()

/** Renders a themed accordion with single or multiple controlled state. */
export function Accordion(props: AccordionProps): JSX.Element {
  const type = () => props.type ?? 'single'
  const initial = props.value ?? props.defaultValue ?? (type() === 'multiple' ? [] : '')
  const [uncontrolled, setUncontrolled] = createSignal<SurfaceAAccordionValue>(initial)
  const first = Array.isArray(initial) ? initial[0] ?? '' : initial
  const [active, setActive] = createSignal(first)
  const records: AccordionItemRecord[] = []
  const value = () => props.value ?? uncontrolled()
  const isDisabled = () => !!props.disabled
  const isOpen = (item: string) => type() === 'multiple'
    ? Array.isArray(value()) && value().includes(item)
    : value() === item
  const setValue = (next: string | string[]) => {
    if (props.value === undefined) setUncontrolled(next)
    props.onValueChange?.(next)
  }
  const setOpen = (item: string, open: boolean) => {
    if (isDisabled() || records.find((record) => record.value === item)?.disabled()) return
    if (type() === 'multiple') {
      const current = Array.isArray(value()) ? [...value()] as string[] : []
      if (current.includes(item) === open) return
      setValue(open
        ? current.includes(item) ? current : [...current, item]
        : current.filter((entry) => entry !== item))
      return
    }
    if (open) {
      if (value() !== item) setValue(item)
    } else if (value() === item && props.collapsible === true) setValue('')
  }
  const toggle = (item: string) => setOpen(item, !isOpen(item))
  const register = (item: string, disabled: () => boolean) => {
    if (!records.some((record) => record.value === item)) records.push({ value: item, disabled })
    if (!active() && !disabled()) setActive(item)
    return () => {
      const index = records.findIndex((record) => record.value === item)
      if (index >= 0) records.splice(index, 1)
      if (active() === item) setActive(records.find((record) => !record.disabled())?.value ?? '')
    }
  }
  const move = (key: string) => {
    const available = records.filter((record) => !record.disabled())
    if (!available.length) return
    if (key === 'Enter' || key === ' ') {
      const current = available.find((record) => record.value === active())
      if (current) toggle(current.value)
      return
    }
    const currentIndex = available.findIndex((record) => record.value === active())
    let nextIndex = currentIndex
    if (key === 'Home') nextIndex = 0
    else if (key === 'End') nextIndex = available.length - 1
    else if (key === 'ArrowDown' || key === 'ArrowRight') nextIndex = (currentIndex + 1 + available.length) % available.length
    else if (key === 'ArrowUp' || key === 'ArrowLeft') nextIndex = currentIndex < 0
      ? available.length - 1 : (currentIndex - 1 + available.length) % available.length
    else return
    setActive(available[nextIndex]!.value)
  }
  const context: AccordionRootContext = {
    id: props.id, theme: props.theme, type: type(), value, active, setActive,
    isDisabled, isOpen, setOpen, toggle, register, move,
  }
  return <RootContext.Provider value={context}>{(<>
    <focusScope key={`${props.id}-accordion`} role="group" accessible_name={props.label ?? 'Accordion'}
      active_descendant={active() ? `${props.id}-trigger-${surfaceAIdPart(active())}` : undefined}
      focus_on_click={true} onKey={(payload) => {
        const key = surfaceAKey(payload)
        if (key) move(key)
      }} onSemanticAction={(payload) => {
        const item = active()
        if (payload.action === 'click') toggle(item)
        else if (payload.action === 'expand') setOpen(item, true)
        else if (payload.action === 'collapse') setOpen(item, false)
      }}>
      <column width="fill" gap={0}>{props.children}</column>
    </focusScope>
  </>) as unknown as SolidJSX.Element}</RootContext.Provider>
}

/** Adds one accordion item and registers it for arrow-key navigation. */
export function AccordionItem(props: AccordionItemProps): JSX.Element {
  const root = useContext(RootContext)
  if (!root) throw new Error('AccordionItem must be rendered inside Accordion')
  const disabled = () => root.isDisabled() || !!props.disabled
  const item: AccordionItemContext = {
    value: props.value, disabled, open: () => root.isOpen(props.value),
  }
  let unregister = () => {}
  onMount(() => { unregister = root.register(props.value, disabled) })
  onCleanup(() => unregister())
  return <ItemContext.Provider value={item}>{(<>
    <column width="fill" gap={0}>
      {props.children}
      <rectangle width="fill" height={1} background={root.theme.border} />
    </column>
  </>) as unknown as SolidJSX.Element}</ItemContext.Provider>
}

/** Renders the accessible button that toggles its surrounding accordion item. */
export function AccordionTrigger(props: AccordionTriggerProps): JSX.Element {
  const root = useContext(RootContext)
  const item = useContext(ItemContext)
  if (!root || !item) throw new Error('AccordionTrigger must be rendered inside AccordionItem')
  const id = `${root.id}-trigger-${surfaceAIdPart(item.value)}`
  const panel = `${root.id}-panel-${surfaceAIdPart(item.value)}`
  const activate = () => {
    if (item.disabled()) return
    root.setActive(item.value)
    root.toggle(item.value)
  }
  return <container width="fill" role="heading" level={3}>
    <focusScope key={id} role="button" accessible_name={props.label} controls={panel}
      expanded={item.open()} enabled={!item.disabled()} accessible_disabled={item.disabled()}
      focusable={false} keyboard_activation="none" onClick={activate}
      onSemanticAction={(payload) => { if (payload.action === 'click') activate() }}>
      <rectangle width="fill" height={48}
        background={item.open() ? root.theme.surfaceRaised : root.theme.surface}
        border_color={root.active() === item.value ? root.theme.accent : root.theme.border}
        border_width={root.active() === item.value ? 1 : 0} radius={root.theme.controlRadius}
        opacity={item.disabled() ? 0.52 : 1}>
        <row width="fill" height="fill" padding={12} gap={12} align_items="center">
          {props.children}
          <container grow={1} />
          <text text="⌄" color={root.theme.muted} font_size={16} rotation={item.open() ? 180 : 0} />
        </row>
      </rectangle>
    </focusScope>
  </container>
}

/** Keeps an item's labelled panel in the tree and hides it while collapsed. */
export function AccordionContent(props: AccordionContentProps): JSX.Element {
  const root = useContext(RootContext)
  const item = useContext(ItemContext)
  if (!root || !item) throw new Error('AccordionContent must be rendered inside AccordionItem')
  const open = () => item.open()
  const id = `${root.id}-trigger-${surfaceAIdPart(item.value)}`
  return <column key={`${root.id}-panel-${surfaceAIdPart(item.value)}`} width="fill"
    visible={open()} accessible_hidden={!open()} role="group" labelled_by={id} padding={12} gap={8}>
    {props.children}
  </column>
}
