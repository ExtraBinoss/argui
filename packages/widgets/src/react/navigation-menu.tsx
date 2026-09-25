/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import type { Palette } from '../shared/theme'
import {
  surfaceNavigationIdPart, surfaceNavigationKey, surfaceNavigationMove, surfaceNavigationSpace,
  type SurfaceNavigationItem, type SurfaceNavigationLink,
} from '../shared/surface-navigation-sidebar'

/** Props for a native navigation bar with direct links and anchored link panels. */
export interface NavigationMenuProps {
  id: string
  theme: Palette
  items: readonly SurfaceNavigationItem[]
  label?: string
  orientation?: 'horizontal' | 'vertical'
  value?: string | null
  defaultValue?: string
  selected?: string
  defaultSelected?: string
  panelWidth?: number
  onValueChange?: (openValue: string | undefined) => void
  onSelectedChange?: (selectedId: string) => void
  onSelect?: (id: string, href?: string) => void
}

/** Shows an accessible navigation list and anchored panels of related links. */
export function NavigationMenu(props: NavigationMenuProps): ReactElement {
  const [uncontrolledOpen, setUncontrolledOpen] = useState<string | undefined>(props.defaultValue)
  const [uncontrolledActive, setUncontrolledActive] = useState(props.defaultValue
    ?? props.items.find((item) => !item.disabled)?.id ?? '')
  const [uncontrolledSelected, setUncontrolledSelected] = useState(props.defaultSelected ?? '')
  const controlledOpen = props.value !== undefined
  const openValue = controlledOpen ? props.value ?? undefined : uncontrolledOpen
  const orientation = props.orientation ?? 'horizontal'
  const requested = openValue ?? uncontrolledActive
  const active = props.items.some((item) => item.id === requested && !item.disabled)
    ? requested : props.items.find((item) => !item.disabled)?.id ?? ''
  const setOpen = (value: string | undefined) => {
    if (openValue === value) return
    if (!controlledOpen) setUncontrolledOpen(value)
    props.onValueChange?.(value)
  }
  const setActive = (value: string) => {
    if (props.items.some((item) => item.id === value && !item.disabled)) setUncontrolledActive(value)
  }
  const selected = props.selected ?? uncontrolledSelected
  const choose = (link: Pick<SurfaceNavigationLink, 'id' | 'href'>) => {
    if (props.selected === undefined) setUncontrolledSelected(link.id)
    props.onSelectedChange?.(link.id)
    props.onSelect?.(link.id, link.href)
    setOpen(undefined)
  }
  const onKey = (payload: unknown) => {
    const key = surfaceNavigationKey(payload)
    if (!key) return
    if (key === 'Escape') { setOpen(undefined); return }
    const next = surfaceNavigationMove(props.items, active, key, orientation)
    if (next) {
      setActive(next)
      const item = props.items.find((entry) => entry.id === next)
      if (openValue) setOpen(item?.links?.length ? item.id : undefined)
      return
    }
    if ((key === 'ArrowDown' || key === 'ArrowUp') && orientation === 'horizontal') {
      const item = props.items.find((entry) => entry.id === active)
      if (item?.links?.length) setOpen(item.id)
      return
    }
    if (key === 'ArrowRight' && orientation === 'vertical') {
      const item = props.items.find((entry) => entry.id === active)
      if (item?.links?.length) setOpen(item.id)
      return
    }
    if ((key === 'Enter' || surfaceNavigationSpace(key)) && active) {
      const item = props.items.find((entry) => entry.id === active)
      if (!item || item.disabled) return
      if (item.links?.length) setOpen(openValue === item.id ? undefined : item.id)
      else choose(item)
    }
  }
  const renderEntry = (item: SurfaceNavigationItem) => {
    const triggerId = `${props.id}-entry-${surfaceNavigationIdPart(item.id)}`
    const popupId = `${props.id}-panel-${surfaceNavigationIdPart(item.id)}`
    const hasPanel = !!item.links?.length
    const isOpen = openValue === item.id
    const focused = active === item.id
    const disabled = !!item.disabled
    return <focusScope nativeKey={triggerId} role={hasPanel ? 'button' : 'link'} accessible_name={item.label}
      accessible_description={item.description} current={selected === item.id ? 'page' : undefined}
      selected={selected === item.id} enabled={!disabled} accessible_disabled={disabled}
      controls={hasPanel ? `${popupId}-content` : undefined} has_popup={hasPanel ? 'menu' : undefined}
      expanded={hasPanel ? isOpen : undefined} expandable={hasPanel ? true : undefined}
      focusable={false} keyboard_activation="none"
      onClick={() => {
        if (disabled) return
        setActive(item.id)
        if (hasPanel) setOpen(isOpen ? undefined : item.id)
        else choose(item)
      }}
      onSemanticAction={(payload) => {
        if (disabled) return
        if (payload.action === 'click') {
          setActive(item.id)
          if (hasPanel) setOpen(isOpen ? undefined : item.id)
          else choose(item)
        } else if (payload.action === 'expand' && hasPanel) { setActive(item.id); setOpen(item.id) }
        else if (payload.action === 'collapse' && isOpen) setOpen(undefined)
      }}>
      <touchArea enabled={!disabled} mouse_cursor={disabled ? 'not_allowed' : 'pointer'}
        onPointerEnter={() => {
          if (disabled) return
          setActive(item.id)
          if (hasPanel) setOpen(item.id)
        }}>
        <rectangle width="fit" height={38} radius={props.theme.controlRadius}
          background={isOpen || focused ? props.theme.accent : props.theme.surfaceRaised}
          border_color={focused ? props.theme.accentHover : props.theme.border} border_width={1}
          opacity={disabled ? 0.45 : 1}>
          <row width="fit" height="fill" gap={8} padding_left={12} padding_right={12} align_items="center">
            <text text={item.label} color={isOpen || focused ? props.theme.accentText : props.theme.foreground}
              font_size={props.theme.controlFontSize} />
            {hasPanel ? <text text="⌄" color={isOpen || focused ? props.theme.accentText : props.theme.muted}
              font_size={12} rotation={isOpen ? 180 : 0} /> : null}
          </row>
        </rectangle>
      </touchArea>
    </focusScope>
  }
  return <focusScope nativeKey={props.id} role="navigation" accessible_name={props.label ?? 'Primary navigation'}
    orientation={orientation} focusable={true} focus_on_tab_navigation={true}
    active_descendant={active ? `${props.id}-entry-${surfaceNavigationIdPart(active)}` : undefined}
    onKey={onKey}>
    {orientation === 'horizontal'
      ? <row width="fill" gap={4} wrap={true} align_items="center">{props.items.map(renderEntry)}</row>
      : <column width="fill" gap={4}>{props.items.map(renderEntry)}</column>}
    {props.items.map((item) => item.links?.length && openValue === item.id && !item.disabled
      ? <NavigationMenuPanel key={`${props.id}-panel-${surfaceNavigationIdPart(item.id)}`}
        id={`${props.id}-panel-${surfaceNavigationIdPart(item.id)}`}
        anchor={`${props.id}-entry-${surfaceNavigationIdPart(item.id)}`} item={item}
        placement={orientation === 'horizontal' ? 'bottom_start' : 'right_start'}
        theme={props.theme} width={props.panelWidth ?? 360} selected={selected}
        onSelect={choose} onClose={() => setOpen(undefined)} /> : null)}
  </focusScope>
}

interface NavigationMenuPanelProps {
  id: string
  anchor: string
  placement: string
  item: SurfaceNavigationItem
  theme: Palette
  width: number
  selected: string
  onSelect: (link: Pick<SurfaceNavigationLink, 'id' | 'href'>) => void
  onClose: () => void
}

/** Renders and keyboard-navigates one link panel inside a native popup window. */
function NavigationMenuPanel(props: NavigationMenuPanelProps): ReactElement {
  const links = props.item.links ?? []
  const [activeId, setActiveId] = useState(links.find((link) => !link.disabled)?.id ?? '')
  const active = links.some((link) => link.id === activeId && !link.disabled)
    ? activeId : links.find((link) => !link.disabled)?.id ?? ''
  const move = (key: string) => {
    const available = links.filter((link) => !link.disabled)
    if (!available.length) return
    const index = available.findIndex((link) => link.id === active)
    let next = index
    if (key === 'Home') next = 0
    else if (key === 'End') next = available.length - 1
    else if (key === 'ArrowDown') next = (index + 1) % available.length
    else if (key === 'ArrowUp') next = (index < 0 ? available.length - 1 : index - 1 + available.length) % available.length
    else return
    setActiveId(available[next]!.id)
  }
  const onKey = (payload: unknown) => {
    const key = surfaceNavigationKey(payload)
    if (!key) return
    if (key === 'Escape' || key === 'ArrowLeft') { props.onClose(); return }
    if (key === 'ArrowDown' || key === 'ArrowUp' || key === 'Home' || key === 'End') {
      move(key)
      return
    }
    if (key === 'Enter' || surfaceNavigationSpace(key)) {
      const link = links.find((entry) => entry.id === active)
      if (link && !link.disabled) props.onSelect(link)
    }
  }
  return <popupWindow nativeKey={props.id} anchor={props.anchor} placement={props.placement} width={props.width}
    window_layer="popover" dismiss_policy="outside_pointer_or_escape" containment="none"
    initial_focus="first" restore_focus={true} onDismiss={props.onClose}>
    <focusScope nativeKey={`${props.id}-content`} role="group" accessible_name={`${props.item.label} links`}
      active_descendant={active ? `${props.id}-link-${surfaceNavigationIdPart(active)}` : undefined}
      focusable={true} focus_on_tab_navigation={false} onKey={onKey}>
      <rectangle width="fill" background={props.theme.surface} border_color={props.theme.border} border_width={1}
        radius={props.theme.overlayRadius} shadow_blur={props.theme.overlayShadowBlur}
        shadow_offset_y={5} shadow_color={props.theme.overlayShadow}>
        <column width="fill" gap={3} padding={8}>
          {links.map((link) => {
            const linkKey = `${props.id}-link-${surfaceNavigationIdPart(link.id)}`
            const focused = active === link.id
            const disabled = !!link.disabled
            return <focusScope nativeKey={linkKey} role="link" accessible_name={link.label}
              accessible_description={link.description} current={props.selected === link.id ? 'page' : undefined}
              selected={props.selected === link.id} enabled={!disabled} accessible_disabled={disabled}
              focusable={false} keyboard_activation="none"
              onClick={() => { if (!disabled) props.onSelect(link) }}
              onSemanticAction={(payload) => { if (payload.action === 'click' && !disabled) props.onSelect(link) }}>
              <touchArea enabled={!disabled} mouse_cursor={disabled ? 'not_allowed' : 'pointer'}
                onPointerEnter={() => { if (!disabled) setActiveId(link.id) }}>
                <rectangle width="fill" radius={props.theme.controlRadius}
                  background={focused ? props.theme.accent : '#00000000'} opacity={disabled ? 0.48 : 1}>
                  <column width="fill" gap={4} padding={10}>
                    <text text={link.label} color={focused ? props.theme.accentText : props.theme.foreground}
                      font_size={props.theme.controlFontSize} weight={600} />
                    {link.description ? <text text={link.description} width="fill"
                      color={focused ? props.theme.accentText : props.theme.muted} font_size={12} /> : null}
                  </column>
                </rectangle>
              </touchArea>
            </focusScope>
          })}
        </column>
      </rectangle>
    </focusScope>
  </popupWindow>
}
