/** @jsxImportSource @argui/react */
import { createContext, createElement, useCallback, useContext, useEffect, useRef, useState, type ReactElement, type ReactNode } from 'react'
import type { Palette } from '../shared/theme'
import { surfaceAIdPart, surfaceAKey, type SurfaceAAccordionValue } from '../shared/surface-a'

/** Accordion root props, including controlled and default state. */
export interface AccordionProps {
  id: string
  theme: Palette
  children: ReactNode
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
  children: ReactNode
  disabled?: boolean
}

/** Trigger props for an accordion item. */
export interface AccordionTriggerProps {
  children: ReactNode
  label?: string
}

/** Content props for an accordion item. */
export interface AccordionContentProps {
  children: ReactNode
}

interface ItemRecord {
  value: string
  disabled: () => boolean
}

interface RootValue {
  id: string
  theme: Palette
  type: 'single' | 'multiple'
  value: () => SurfaceAAccordionValue
  active: () => string
  setActive: (value: string) => void
  focused: () => boolean
  isDisabled: () => boolean
  isOpen: (value: string) => boolean
  setOpen: (value: string, open: boolean) => void
  toggle: (value: string) => void
  register: (value: string, disabled: () => boolean) => () => void
  move: (key: string) => void
}

interface ItemValue {
  value: string
  disabled: () => boolean
  open: () => boolean
}

const RootContext = createContext<RootValue | null>(null)
const ItemContext = createContext<ItemValue | null>(null)

/** Renders a themed accordion with single or multiple controlled state. */
export function Accordion(props: AccordionProps): ReactElement {
  const type = props.type ?? 'single'
  const initial = props.value ?? props.defaultValue ?? (type === 'multiple' ? [] : '')
  const [uncontrolled, setUncontrolled] = useState<SurfaceAAccordionValue>(initial)
  const [active, setActive] = useState<string>(Array.isArray(initial) ? initial[0] ?? '' : typeof initial === 'string' ? initial : '')
  const [focused, setFocused] = useState(false)
  const records = useRef<ItemRecord[]>([])
  const value = () => props.value ?? uncontrolled
  const isDisabled = () => !!props.disabled
  const isOpen = (item: string) => {
    const current = value()
    return type === 'multiple' ? Array.isArray(current) && current.includes(item) : current === item
  }
  const update = (next: string | string[]) => {
    if (props.value === undefined) setUncontrolled(next)
    props.onValueChange?.(next)
  }
  const setOpen = (item: string, open: boolean) => {
    if (isDisabled() || records.current.find((record) => record.value === item)?.disabled()) return
    if (type === 'multiple') {
      const current = Array.isArray(value()) ? [...value()] as string[] : []
      if (current.includes(item) === open) return
      update(open
        ? current.includes(item) ? current : [...current, item]
        : current.filter((entry) => entry !== item))
    } else if (open) {
      if (value() !== item) update(item)
    } else if (value() === item && props.collapsible === true) update('')
  }
  const toggle = (item: string) => setOpen(item, !isOpen(item))
  const register = useCallback((item: string, disabled: () => boolean) => {
    if (!records.current.some((record) => record.value === item)) records.current.push({ value: item, disabled })
    setActive((current: string) => current || (disabled() ? '' : item))
    return () => {
      records.current = records.current.filter((record) => record.value !== item)
      setActive((current: string) => current === item ? records.current.find((record) => !record.disabled())?.value ?? '' : current)
    }
  }, [])
  const move = (key: string) => {
    const available = records.current.filter((record) => !isDisabled() && !record.disabled())
    if (!available.length) return
    if (key === 'Enter' || key === ' ') {
      const current = available.find((record) => record.value === active)
      if (current) toggle(current.value)
      return
    }
    const currentIndex = available.findIndex((record) => record.value === active)
    let nextIndex = currentIndex
    if (key === 'Home') nextIndex = 0
    else if (key === 'End') nextIndex = available.length - 1
    else if (key === 'ArrowDown' || key === 'ArrowRight') nextIndex = (currentIndex + 1 + available.length) % available.length
    else if (key === 'ArrowUp' || key === 'ArrowLeft') nextIndex = currentIndex < 0
      ? available.length - 1 : (currentIndex - 1 + available.length) % available.length
    else return
    setActive(available[nextIndex]!.value)
  }
  const context: RootValue = {
    id: props.id, theme: props.theme, type, value, active: () => active, setActive, focused: () => focused,
    isDisabled, isOpen, setOpen, toggle, register, move,
  }
  return createElement(RootContext.Provider, { value: context },
    <focusScope nativeKey={`${props.id}-accordion`} role="group" accessible_name={props.label ?? 'Accordion'}
      active_descendant={active ? `${props.id}-trigger-${surfaceAIdPart(active)}` : undefined}
      multiselectable={type === 'multiple'} focus_on_click={true} onKey={(payload) => {
        const key = surfaceAKey(payload)
        if (key) move(key)
      }} onFocus={() => setFocused(true)} onBlur={() => setFocused(false)}
      onSemanticAction={(payload) => {
        const current = active
        if (payload.action === 'click') toggle(current)
        else if (payload.action === 'expand') setOpen(current, true)
        else if (payload.action === 'collapse') setOpen(current, false)
      }}>
      <column width="fill" gap={0}>{props.children}</column>
    </focusScope>
  ) as ReactElement
}

/** Adds one accordion item and registers it for arrow-key navigation. */
export function AccordionItem(props: AccordionItemProps): ReactElement {
  const root = useContext(RootContext)
  if (!root) throw new Error('AccordionItem must be rendered inside Accordion')
  const disabled = () => !!props.disabled
  const item: ItemValue = { value: props.value, disabled: () => root.isDisabled() || disabled(), open: () => root.isOpen(props.value) }
  useEffect(() => root.register(props.value, disabled), [root.register, props.value, props.disabled])
  return createElement(ItemContext.Provider, { value: item},
    <column width="fill" gap={0}>
      {props.children}
      <rectangle width="fill" height={1} background={root.theme.border} />
    </column>
  ) as ReactElement
}

/** Renders the accessible button that toggles its surrounding accordion item. */
export function AccordionTrigger(props: AccordionTriggerProps): ReactElement {
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
    <focusScope nativeKey={id} role="button" accessible_name={props.label} controls={panel}
      expanded={item.open()} enabled={!item.disabled()} accessible_disabled={item.disabled()}
      focusable={false} keyboard_activation="none" onClick={activate}
      onSemanticAction={(payload) => { if (payload.action === 'click') activate() }}>
      <rectangle width="fill" height={48} background={item.open() ? root.theme.surfaceRaised : root.theme.surface}
        border_color={root.active() === item.value && root.focused() ? root.theme.accent : root.theme.border}
        border_width={root.active() === item.value && root.focused() ? 1 : 0} radius={root.theme.controlRadius}
        opacity={item.disabled() ? 0.52 : 1}>
        <row width="fill" height="fill" padding={12} gap={12} align_items="center">
          {props.children}<container grow={1} />
          <text text="⌄" color={root.theme.muted} font_size={16} rotation={item.open() ? 180 : 0} />
        </row>
      </rectangle>
    </focusScope>
  </container>
}

/** Keeps an item's labelled panel in the tree and hides it while collapsed. */
export function AccordionContent(props: AccordionContentProps): ReactElement {
  const root = useContext(RootContext)
  const item = useContext(ItemContext)
  if (!root || !item) throw new Error('AccordionContent must be rendered inside AccordionItem')
  const open = item.open()
  const id = `${root.id}-trigger-${surfaceAIdPart(item.value)}`
  return <column nativeKey={`${root.id}-panel-${surfaceAIdPart(item.value)}`} width="fill"
    visible={open} accessible_hidden={!open} role="group" labelled_by={id} padding={12} gap={8}>
    {props.children}
  </column>
}
