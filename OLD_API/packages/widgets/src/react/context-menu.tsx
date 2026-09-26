/** @jsxImportSource @argui/react */
import { useState, type ReactElement, type ReactNode } from 'react'
import type { Palette } from '../shared/theme'
import {
  surfaceBKey, type SurfaceMenuCheckbox, type SurfaceMenuController, type SurfaceMenuItem,
  type SurfaceMenuRadio, type SurfaceMenuStateProps,
} from '../shared/surface-b'
import { SurfaceMenuPopup } from './dropdown-menu'

/** Props for a context menu attached to arbitrary trigger content. */
export interface ContextMenuProps extends SurfaceMenuStateProps {
  id: string
  theme: Palette
  label: string
  children: ReactNode
  items: readonly SurfaceMenuItem[]
  menuLabel?: string
  open?: boolean
  defaultOpen?: boolean
  disabled?: boolean
  width?: number
  onOpenChange?: (open: boolean) => void
}

/** Opens a themed menu from a context-menu pointer event or keyboard command. */
export function ContextMenu(props: ContextMenuProps): ReactElement {
  const [uncontrolledOpen, setUncontrolledOpen] = useState(props.defaultOpen ?? false)
  const [internalChecked, setInternalChecked] = useState<Record<string, boolean>>({ ...(props.defaultChecked ?? {}) })
  const [internalRadios, setInternalRadios] = useState<Record<string, string>>({ ...(props.defaultRadioValues ?? {}) })
  const open = props.open ?? uncontrolledOpen
  const setOpen = (next: boolean) => {
    if ((next && props.disabled) || open === next) return
    if (props.open === undefined) setUncontrolledOpen(next)
    props.onOpenChange?.(next)
  }
  const controller: SurfaceMenuController = {
    isChecked: (item) => item.checked ?? props.checked?.[item.id] ?? internalChecked[item.id]
      ?? props.defaultChecked?.[item.id] ?? item.defaultChecked ?? false,
    radioValue: (group) => props.radioValues?.[group] ?? internalRadios[group] ?? props.defaultRadioValues?.[group],
    toggleCheckbox: (item: SurfaceMenuCheckbox) => {
      const checked = item.checked ?? props.checked?.[item.id] ?? internalChecked[item.id]
        ?? props.defaultChecked?.[item.id] ?? item.defaultChecked ?? false
      const next = !checked
      if (item.checked === undefined && props.checked?.[item.id] === undefined) {
        setInternalChecked((current) => ({ ...current, [item.id]: next }))
      }
      item.onCheckedChange?.(next)
      props.onCheckedChange?.(item.id, next)
    },
    chooseRadio: (item: SurfaceMenuRadio) => {
      if (props.radioValues?.[item.group] === undefined) {
        setInternalRadios((current) => ({ ...current, [item.group]: item.value }))
      }
      props.onRadioValueChange?.(item.group, item.value)
    },
    select: (item) => {
      item.onSelect?.()
      props.onSelect?.(item.id)
    },
  }
  const triggerId = `${props.id}-trigger`
  const popupId = `${props.id}-popup`
  const openFromKeyboard = (payload: unknown) => {
    const key = surfaceBKey(payload)
    if (key === 'ContextMenu' || key === 'F10' || key === 'ArrowDown') setOpen(true)
    else if (key === 'Escape') setOpen(false)
  }
  return <column width="fill" gap={0}>
    <focusScope nativeKey={triggerId} role="button" accessible_name={props.label}
      controls={`${popupId}-menu`} has_popup="menu" expanded={open} enabled={!props.disabled}
      accessible_disabled={props.disabled} keyboard_activation="none" onKey={openFromKeyboard}
      onSemanticAction={(payload) => {
        if (payload.action === 'click' || payload.action === 'expand') setOpen(true)
        else if (payload.action === 'collapse') setOpen(false)
      }}>
      <touchArea enabled={!props.disabled} mouse_cursor={props.disabled ? 'not_allowed' : 'context_menu'}
        onContextMenu={() => setOpen(true)}>
        {props.children}
      </touchArea>
    </focusScope>
    {open ? <SurfaceMenuPopup id={popupId} anchor={triggerId} placement="bottom_start"
      label={props.menuLabel ?? props.label} items={props.items} theme={props.theme}
      width={props.width ?? 220} controller={controller} onClose={() => setOpen(false)} /> : null}
  </column>
}
