/** @jsxImportSource @argui/react */
import { useEffect, useRef, useState, type ReactElement } from 'react'
import type { Palette } from '../shared/theme'
import {
  surfaceBFlatten, surfaceBFocusable, surfaceBIsSpaceKey, surfaceBKey, surfaceBKeyPart, surfaceBMove, surfaceBSearch,
  type SurfaceMenuAction, type SurfaceMenuCheckbox, type SurfaceMenuController, type SurfaceMenuItem,
  type SurfaceMenuRadio, type SurfaceMenuRow, type SurfaceMenuStateProps,
} from '../shared/surface-b'

/** Props for the dropdown root and its built-in trigger. */
export interface DropdownMenuProps extends SurfaceMenuStateProps {
  id: string
  theme: Palette
  triggerLabel: string
  items: readonly SurfaceMenuItem[]
  label?: string
  menuLabel?: string
  open?: boolean
  defaultOpen?: boolean
  disabled?: boolean
  width?: number
  onOpenChange?: (open: boolean) => void
}

/** Shared popup renderer used by dropdown, context-menu, and menubar roots. */
export interface SurfaceMenuPopupProps {
  id: string
  anchor: string
  placement: string
  label: string
  items: readonly SurfaceMenuItem[]
  theme: Palette
  width: number
  controller: SurfaceMenuController
  onClose: () => void
  initialActiveId?: string
  nested?: boolean
}

/** Shows an accessible button and an anchored menu of actions and controls. */
export function DropdownMenu(props: DropdownMenuProps): ReactElement {
  const [uncontrolledOpen, setUncontrolledOpen] = useState(props.defaultOpen ?? false)
  const [internalChecked, setInternalChecked] = useState<Record<string, boolean>>({ ...(props.defaultChecked ?? {}) })
  const [internalRadios, setInternalRadios] = useState<Record<string, string>>({ ...(props.defaultRadioValues ?? {}) })
  const [initialActiveId, setInitialActiveId] = useState('')
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
    toggleCheckbox: (item) => {
      const checked = item.checked ?? props.checked?.[item.id] ?? internalChecked[item.id]
        ?? props.defaultChecked?.[item.id] ?? item.defaultChecked ?? false
      const next = !checked
      if (item.checked === undefined && props.checked?.[item.id] === undefined) {
        setInternalChecked((current) => ({ ...current, [item.id]: next }))
      }
      item.onCheckedChange?.(next)
      props.onCheckedChange?.(item.id, next)
    },
    chooseRadio: (item) => {
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
  const menuId = `${props.id}-popup-menu`
  const triggerId = `${props.id}-trigger`
  return <column width="fit" gap={0}>
    <focusScope nativeKey={triggerId} role="button" accessible_name={props.label ?? props.triggerLabel}
      controls={menuId} has_popup="menu" expanded={open} enabled={!props.disabled}
      accessible_disabled={props.disabled} keyboard_activation="enter_or_space"
      onClick={() => {
        if (!open) setInitialActiveId(surfaceBFocusable(props.items)[0]?.id ?? '')
        setOpen(!open)
      }} onKey={(payload) => {
        const key = surfaceBKey(payload)
        if (key === 'ArrowDown') { setInitialActiveId(surfaceBFocusable(props.items)[0]?.id ?? ''); setOpen(true) }
        else if (key === 'ArrowUp') { setInitialActiveId(surfaceBFocusable(props.items).at(-1)?.id ?? ''); setOpen(true) }
        else if (key === 'Escape') setOpen(false)
      }} onSemanticAction={(payload) => {
        if (payload.action === 'click') {
          if (!open) setInitialActiveId(surfaceBFocusable(props.items)[0]?.id ?? '')
          setOpen(!open)
        }
        else if (payload.action === 'expand') { setInitialActiveId(surfaceBFocusable(props.items)[0]?.id ?? ''); setOpen(true) }
        else if (payload.action === 'collapse') setOpen(false)
      }}>
      <touchArea enabled={!props.disabled} mouse_cursor={props.disabled ? 'not_allowed' : 'pointer'}>
        <rectangle width={props.width ?? 176} height={36}
          background={open ? props.theme.surfaceRaised : props.theme.surface}
          border_color={props.theme.border} border_width={1} radius={props.theme.controlRadius}
          opacity={props.disabled ? 0.52 : 1}>
          <row width="fill" height="fill" gap={10} padding={12} align_items="center">
            <text text={props.triggerLabel} color={props.theme.foreground} font_size={props.theme.controlFontSize} />
            <container grow={1} />
            <text text="⌄" color={props.theme.muted} font_size={14} rotation={open ? 180 : 0} />
          </row>
        </rectangle>
      </touchArea>
    </focusScope>
    {open ? <SurfaceMenuPopup id={`${props.id}-popup`} anchor={triggerId} placement="bottom_start"
      label={props.menuLabel ?? props.triggerLabel} items={props.items} theme={props.theme}
      width={props.width ?? 220} controller={controller} onClose={() => setOpen(false)}
      initialActiveId={initialActiveId} /> : null}
  </column>
}

/** Renders one menu popup and recursively anchors any active submenu. */
/** Renders one menu popup and recursively anchors any active submenu. */
export function SurfaceMenuPopup(props: SurfaceMenuPopupProps): ReactElement {
  const rows = surfaceBFlatten(props.items)
  const navigable = surfaceBFocusable(props.items)
  const [activeId, setActiveId] = useState(props.initialActiveId ?? navigable[0]?.id ?? '')
  const [openSubmenu, setOpenSubmenu] = useState('')
  const searchBuffer = useRef('')
  const searchTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined)
  useEffect(() => () => { if (searchTimer.current) clearTimeout(searchTimer.current) }, [])
  const active = navigable.some((item) => item.id === activeId) ? activeId : navigable[0]?.id ?? ''
  const close = () => props.onClose()
  const activate = (item: SurfaceMenuRow) => {
    if (item.type === 'label' || item.type === 'separator') return
    if (item.disabled) return
    if (item.type === 'item') {
      props.controller.select(item)
      close()
    } else if (item.type === 'checkbox') props.controller.toggleCheckbox(item)
    else if (item.type === 'radio') props.controller.chooseRadio(item)
    else if (item.type === 'submenu') setOpenSubmenu(item.id)
  }
  const onKey = (payload: unknown) => {
    const key = surfaceBKey(payload)
    if (!key) return
    if (key === 'Escape') { close(); return }
    if (key === 'ArrowLeft' && props.nested) { close(); return }
    const next = surfaceBMove(props.items, active, key)
    if (next) { setActiveId(next); setOpenSubmenu(''); return }
    const current = navigable.find((item) => item.id === active)
    if (key === 'ArrowRight' && current?.type === 'submenu' && !current.disabled) {
      setOpenSubmenu(current.id)
      return
    }
    if (key === 'Enter' || surfaceBIsSpaceKey(key)) {
      if (current) activate(current)
      return
    }
    if (key.length === 1 && key !== ' ') {
      searchBuffer.current += key.toLocaleLowerCase()
      const match = surfaceBSearch(props.items, active, searchBuffer.current)
      if (match) { setActiveId(match); setOpenSubmenu('') }
      if (searchTimer.current) clearTimeout(searchTimer.current)
      searchTimer.current = setTimeout(() => { searchBuffer.current = '' }, 700)
    }
  }
  const render = (item: SurfaceMenuRow) => renderRow(item, props, active, navigable.length, activate,
    setActiveId, setOpenSubmenu, openSubmenu === item.id)
  return <popupWindow nativeKey={props.id} anchor={props.anchor} placement={props.placement} width={props.width}
    window_layer="popover" dismiss_policy="outside_pointer_or_escape" containment="trap"
    initial_focus="first" restore_focus={true} onDismiss={close}>
    <focusScope nativeKey={`${props.id}-menu`} role="menu" accessible_name={props.label}
      active_descendant={active ? `${props.id}-item-${surfaceBKeyPart(active)}` : undefined}
      focusable={true} focus_on_tab_navigation={false} keyboard_activation="none" onKey={onKey}
      onSemanticAction={(payload) => {
        if (payload.action === 'click') {
          const current = navigable.find((item) => item.id === active)
          if (current) activate(current)
        }
      }}>
      <rectangle width="fill" background={props.theme.surface} border_color={props.theme.border} border_width={1}
        radius={props.theme.overlayRadius} shadow_blur={props.theme.overlayShadowBlur}
        shadow_offset_y={5} shadow_color={props.theme.overlayShadow}>
        <column width="fill" gap={2} padding={4}>{rows.map(render)}</column>
      </rectangle>
    </focusScope>
  </popupWindow>
}

function renderRow(
  item: SurfaceMenuRow,
  props: SurfaceMenuPopupProps,
  activeId: string,
  size: number,
  activate: (item: SurfaceMenuRow) => void,
  setActive: (value: string) => void,
  setOpenSubmenu: (value: string) => void,
  submenuOpen: boolean,
): ReactElement {
  const theme = props.theme
  if (item.type === 'separator') return <rectangle key={`${props.id}-${surfaceBKeyPart(item.id)}`}
    width="fill" height={1} background={theme.border} />
  if (item.type === 'label') return <row key={`${props.id}-${surfaceBKeyPart(item.id)}`} width="fill" height={26} padding_left={item.inset ? 24 : 8} align_items="center">
    <text text={item.label} color={theme.muted} font_size={12} weight={600} />
  </row>
  const key = `${props.id}-item-${surfaceBKeyPart(item.id)}`
  const focused = item.id === activeId
  const disabled = !!item.disabled
  const isCheck = item.type === 'checkbox' && props.controller.isChecked(item)
  const isRadio = item.type === 'radio' && props.controller.radioValue(item.group) === item.value
  const role = item.type === 'checkbox' ? 'menu_item_check_box' : item.type === 'radio' ? 'menu_item_radio' : 'menu_item'
  const checkedState = item.type === 'checkbox' || item.type === 'radio'
    ? (isCheck || isRadio ? 'checked' : 'unchecked') as 'checked' | 'unchecked' : undefined
  const expand = item.type === 'submenu'
  const panelId = `${props.id}-sub-${surfaceBKeyPart(item.id)}-menu`
  return <column key={key} width="fill" gap={0}>
    <focusScope nativeKey={key} role={role} accessible_name={item.label}
      accessible_description={item.shortcut} enabled={!disabled} accessible_disabled={disabled}
      focusable={false} keyboard_activation="none" checked_state={checkedState}
      expanded={expand ? submenuOpen : undefined} expandable={expand}
      has_popup={expand ? 'menu' : undefined} controls={expand ? panelId : undefined}
      position_in_set={size ? Math.max(1, surfaceBFocusable(props.items).findIndex((entry) => entry.id === item.id) + 1) : undefined}
      set_size={size} onClick={() => activate(item)}
      onSemanticAction={(payload) => { if (payload.action === 'click') activate(item) }}>
      <touchArea enabled={!disabled} mouse_cursor={disabled ? 'not_allowed' : 'pointer'}
        onPointerEnter={() => {
          if (disabled) return
          setActive(item.id)
          setOpenSubmenu(expand ? item.id : '')
        }}>
        <rectangle width="fill" height={32} radius={theme.controlRadius}
          background={focused ? theme.accent : '#00000000'} opacity={disabled ? 0.48 : 1}>
          <row width="fill" height="fill" gap={8} padding_left={item.inset ? 24 : 8}
            padding_right={8} align_items="center">
            <container width={14}>
              <text text={item.type === 'checkbox' && isCheck ? '✓' : item.type === 'radio' && isRadio ? '●' : ''}
                color={focused ? theme.accentText : theme.accent} font_size={14} />
            </container>
            <text text={item.label} color={focused ? theme.accentText : item.destructive ? theme.destructive : theme.foreground}
              font_size={theme.controlFontSize} />
            <container grow={1} />
            {item.shortcut ? <text text={item.shortcut} color={focused ? theme.accentText : theme.muted} font_size={11} /> : null}
            {expand ? <text text="›" color={focused ? theme.accentText : theme.muted} font_size={16} /> : null}
          </row>
        </rectangle>
      </touchArea>
    </focusScope>
    {item.type === 'submenu' && submenuOpen && !disabled
      ? <SurfaceMenuPopup id={`${props.id}-sub-${surfaceBKeyPart(item.id)}`} anchor={key} placement="right_start"
        label={item.label} items={item.items} theme={theme} width={props.width}
        controller={props.controller} onClose={() => setOpenSubmenu('')} nested /> : null}
  </column>
}
