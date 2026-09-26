/** @jsxImportSource @argui/react */
import { createContext, createElement, useContext, useState, type ReactElement, type ReactNode } from 'react'
import type { Palette } from '../shared/theme'

/** Props for a collapsible root. */
export interface CollapsibleProps {
  id: string
  theme: Palette
  children: ReactNode
  open?: boolean
  defaultOpen?: boolean
  disabled?: boolean
  label?: string
  onOpenChange?: (open: boolean) => void
}

/** Props for the collapsible trigger. */
export interface CollapsibleTriggerProps {
  children: ReactNode
  label?: string
}

/** Props for the collapsible content region. */
export interface CollapsibleContentProps {
  children: ReactNode
}

interface RootValue {
  id: string
  theme: Palette
  open: () => boolean
  disabled: () => boolean
  setOpen: (open: boolean) => void
  toggle: () => void
}

const RootContext = createContext<RootValue | null>(null)

/** Provides themed controlled or default-open state for trigger and content. */
export function Collapsible(props: CollapsibleProps): ReactElement {
  const [uncontrolled, setUncontrolled] = useState(props.defaultOpen ?? false)
  const open = () => props.open ?? uncontrolled
  const disabled = () => !!props.disabled
  const setOpen = (next: boolean) => {
    if (disabled() || open() === next) return
    if (props.open === undefined) setUncontrolled(next)
    props.onOpenChange?.(next)
  }
  const toggle = () => setOpen(!open())
  const context: RootValue = { id: props.id, theme: props.theme, open, disabled, setOpen, toggle }
  return createElement(RootContext.Provider, { value: context },
    <column width="fill" gap={8} role={props.label ? 'group' : undefined} accessible_name={props.label}>
      {props.children}
    </column>
  ) as ReactElement
}

/** Renders the accessible button that opens or closes a collapsible panel. */
export function CollapsibleTrigger(props: CollapsibleTriggerProps): ReactElement {
  const root = useContext(RootContext)
  if (!root) throw new Error('CollapsibleTrigger must be rendered inside Collapsible')
  return <focusScope nativeKey={`${root.id}-trigger`} role="button" accessible_name={props.label}
    controls={`${root.id}-content`} expanded={root.open()} enabled={!root.disabled()}
    accessible_disabled={root.disabled()} keyboard_activation="enter_or_space" onClick={root.toggle}
    onSemanticAction={(payload) => {
      if (payload.action === 'click') root.toggle()
      else if (payload.action === 'expand') root.setOpen(true)
      else if (payload.action === 'collapse') root.setOpen(false)
    }}>
    <rectangle width="fill" height={42} background={root.open() ? root.theme.surfaceRaised : root.theme.surface}
      border_color={root.theme.border} border_width={1} radius={root.theme.controlRadius}
      opacity={root.disabled() ? 0.52 : 1}>
      <row width="fill" height="fill" gap={10} padding={12} align_items="center">
        {props.children}<container grow={1} />
        <text text="⌄" color={root.theme.muted} font_size={16} rotation={root.open() ? 180 : 0} />
      </row>
    </rectangle>
  </focusScope>
}

/** Keeps the content region mounted while removing it from layout and semantics when closed. */
export function CollapsibleContent(props: CollapsibleContentProps): ReactElement {
  const root = useContext(RootContext)
  if (!root) throw new Error('CollapsibleContent must be rendered inside Collapsible')
  const open = root.open()
  return <column nativeKey={`${root.id}-content`} width="fill" visible={open}
    accessible_hidden={!open} role="group" labelled_by={`${root.id}-trigger`} padding={12}>
    {props.children}
  </column>
}
