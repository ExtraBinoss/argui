/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import type { Palette } from '../shared/theme'
import {
  surfaceBIsSpaceKey, surfaceBKey, surfaceBKeyPart, surfaceBMoveMenubar,
  type SurfaceMenuCheckbox, type SurfaceMenuController, type SurfaceMenuRadio,
  type SurfaceMenuStateProps, type SurfaceMenubarMenu,
} from '../shared/surface-b'
import { SurfaceMenuPopup } from './dropdown-menu'

/** Props for a composite menubar with data-driven menus and popup contents. */
export interface MenubarProps extends SurfaceMenuStateProps {
  id: string
  theme: Palette
  menus: readonly SurfaceMenubarMenu[]
  label?: string
  openMenu?: string | null
  defaultOpenMenu?: string
  width?: number
  onOpenChange?: (menuId: string | undefined) => void
}

/** Renders an accessible menubar with arrow-key menu switching. */
export function Menubar(props: MenubarProps): ReactElement {
  const first = props.menus.find((menu) => !menu.disabled)?.id ?? ''
  const [uncontrolledOpen, setUncontrolledOpen] = useState<string | undefined>(props.defaultOpenMenu)
  const [uncontrolledActive, setUncontrolledActive] = useState(props.defaultOpenMenu ?? first)
  const [internalChecked, setInternalChecked] = useState<Record<string, boolean>>({ ...(props.defaultChecked ?? {}) })
  const [internalRadios, setInternalRadios] = useState<Record<string, string>>({ ...(props.defaultRadioValues ?? {}) })
  const isControlled = props.openMenu !== undefined
  const openMenu = isControlled ? props.openMenu ?? undefined : uncontrolledOpen
  const preferredActive = openMenu ?? uncontrolledActive
  const activeMenu = props.menus.some((menu) => menu.id === preferredActive && !menu.disabled)
    ? preferredActive : first
  const setActive = (menuId: string) => {
    if (props.menus.some((menu) => menu.id === menuId && !menu.disabled)) setUncontrolledActive(menuId)
  }
  const setOpen = (menuId: string | undefined) => {
    if (openMenu === menuId) return
    if (!isControlled) setUncontrolledOpen(menuId)
    props.onOpenChange?.(menuId)
  }
  const toggle = (menuId: string) => {
    setActive(menuId)
    setOpen(openMenu === menuId ? undefined : menuId)
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
  const onKey = (payload: unknown) => {
    const key = surfaceBKey(payload)
    if (!key) return
    if (key === 'Escape') { setOpen(undefined); return }
    const next = surfaceBMoveMenubar(props.menus, activeMenu, key)
    if (next) {
      setActive(next)
      if (openMenu) setOpen(next)
      return
    }
    if ((key === 'ArrowDown' || key === 'ArrowUp' || key === 'Enter' || surfaceBIsSpaceKey(key)) && activeMenu) {
      setOpen(activeMenu)
    }
  }
  return <focusScope nativeKey={props.id} role="menu_bar" accessible_name={props.label ?? 'Menu bar'}
    active_descendant={activeMenu ? `${props.id}-menu-trigger-${surfaceBKeyPart(activeMenu)}` : undefined}
    focus_on_click={true} onKey={onKey}>
    <rectangle width={props.width ?? 'fill'} height={40} background={props.theme.surfaceRaised}
      border_color={props.theme.border} border_width={1} radius={props.theme.controlRadius}>
      <row width="fill" height="fill" gap={2} padding={4} align_items="center">
        {props.menus.map((menu) => {
          const triggerId = `${props.id}-menu-trigger-${surfaceBKeyPart(menu.id)}`
          const popupId = `${props.id}-menu-${surfaceBKeyPart(menu.id)}-popup`
          const isOpen = openMenu === menu.id
          const disabled = !!menu.disabled
          return <focusScope nativeKey={triggerId} role="menu_item" accessible_name={menu.label}
            controls={`${popupId}-menu`} has_popup="menu" expanded={isOpen} enabled={!disabled}
            accessible_disabled={disabled} focusable={false} keyboard_activation="none"
            onClick={() => { if (!disabled) toggle(menu.id) }}
            onSemanticAction={(payload) => {
              if (payload.action === 'click' && !disabled) toggle(menu.id)
              else if (payload.action === 'expand' && !disabled) { setActive(menu.id); setOpen(menu.id) }
              else if (payload.action === 'collapse' && isOpen) setOpen(undefined)
            }}>
            <touchArea enabled={!disabled} mouse_cursor={disabled ? 'not_allowed' : 'pointer'}
              onPointerEnter={() => {
                if (disabled) return
                setActive(menu.id)
                if (openMenu) setOpen(menu.id)
              }}>
              <rectangle width="fit" height="fill" radius={props.theme.controlRadius}
                background={isOpen || activeMenu === menu.id ? props.theme.accent : '#00000000'}
                opacity={disabled ? 0.45 : 1}>
                <row width="fit" height="fill" padding_left={10} padding_right={10} gap={8} align_items="center">
                  <text text={menu.label} color={isOpen || activeMenu === menu.id
                    ? props.theme.accentText : props.theme.foreground} font_size={props.theme.controlFontSize} />
                  <text text="⌄" color={props.theme.muted} font_size={11} rotation={isOpen ? 180 : 0} />
                </row>
              </rectangle>
            </touchArea>
          </focusScope>
        })}
      </row>
    </rectangle>
    {props.menus.map((menu) => openMenu === menu.id && !menu.disabled
      ? <SurfaceMenuPopup id={`${props.id}-menu-${surfaceBKeyPart(menu.id)}-popup`}
        anchor={`${props.id}-menu-trigger-${surfaceBKeyPart(menu.id)}`} placement="bottom_start"
        label={menu.label} items={menu.items} theme={props.theme} width={props.width ?? 220}
        controller={controller} onClose={() => setOpen(undefined)} /> : null)}
  </focusScope>
}
