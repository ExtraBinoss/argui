import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import { createContext, useContext } from 'solid-js'
import type { JSX as SolidJSX } from 'solid-js'
import type { Palette } from '../shared/theme'

/** Props for a collapsible root. */
export interface CollapsibleProps {
  id: string
  theme: Palette
  children: JSX.Element
  open?: boolean
  defaultOpen?: boolean
  disabled?: boolean
  label?: string
  onOpenChange?: (open: boolean) => void
}

/** Props for the collapsible trigger. */
export interface CollapsibleTriggerProps {
  children: JSX.Element
  label?: string
}

/** Props for the collapsible content region. */
export interface CollapsibleContentProps {
  children: JSX.Element
}

interface CollapsibleContext {
  id: string
  theme: Palette
  open: () => boolean
  disabled: () => boolean
  setOpen: (open: boolean) => void
  toggle: () => void
}

const RootContext = createContext<CollapsibleContext>()

/** Provides themed controlled or default-open state for trigger and content. */
export function Collapsible(props: CollapsibleProps): JSX.Element {
  const [uncontrolled, setUncontrolled] = createSignal(props.defaultOpen ?? false)
  const open = () => props.open ?? uncontrolled()
  const disabled = () => !!props.disabled
  const setOpen = (next: boolean) => {
    if (disabled() || open() === next) return
    if (props.open === undefined) setUncontrolled(next)
    props.onOpenChange?.(next)
  }
  const toggle = () => setOpen(!open())
  const context: CollapsibleContext = { id: props.id, theme: props.theme, open, disabled, setOpen, toggle }
  return <RootContext.Provider value={context}>{(<>
    <column width="fill" gap={8} role={props.label ? 'group' : undefined} accessible_name={props.label}>
      {props.children}
    </column>
  </>) as unknown as SolidJSX.Element}</RootContext.Provider>
}

/** Renders the accessible button that opens or closes a collapsible panel. */
export function CollapsibleTrigger(props: CollapsibleTriggerProps): JSX.Element {
  const root = useContext(RootContext)
  if (!root) throw new Error('CollapsibleTrigger must be rendered inside Collapsible')
  const id = `${root.id}-trigger`
  return <focusScope key={id} role="button" accessible_name={props.label} controls={`${root.id}-content`}
    expanded={root.open()} enabled={!root.disabled()} accessible_disabled={root.disabled()}
    keyboard_activation="enter_or_space" onClick={root.toggle}
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
export function CollapsibleContent(props: CollapsibleContentProps): JSX.Element {
  const root = useContext(RootContext)
  if (!root) throw new Error('CollapsibleContent must be rendered inside Collapsible')
  return <column key={`${root.id}-content`} width="fill" visible={root.open()}
    accessible_hidden={!root.open()} role="group" labelled_by={`${root.id}-trigger`} padding={12}>
    {props.children}
  </column>
}
