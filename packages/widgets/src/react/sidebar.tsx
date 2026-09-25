/** @jsxImportSource @argui/react */
import { useState, type ReactElement } from 'react'
import type { Palette } from '../shared/theme'
import {
  surfaceNavigationIdPart, surfaceNavigationKey, surfaceNavigationSpace,
  surfaceSidebarMove, surfaceSidebarVisibleRows, type SurfaceSidebarItem, type SurfaceSidebarRow,
} from '../shared/surface-navigation-sidebar'

/** Props for a controlled or default-open hierarchical sidebar navigation. */
export interface SidebarProps {
  id: string
  theme: Palette
  items: readonly SurfaceSidebarItem[]
  label?: string
  heading?: string
  open?: boolean
  defaultOpen?: boolean
  collapsible?: 'icon' | 'offcanvas' | 'none'
  selected?: string
  defaultSelected?: string
  expandedItems?: Readonly<Record<string, boolean>>
  defaultExpandedItems?: Readonly<Record<string, boolean>>
  width?: number
  iconWidth?: number
  height?: number
  onOpenChange?: (open: boolean) => void
  onSelectedChange?: (id: string) => void
  onExpandedChange?: (id: string, expanded: boolean) => void
  onSelect?: (id: string) => void
}

/** Renders a collapsible native navigation rail with selectable nested entries. */
export function Sidebar(props: SidebarProps): ReactElement {
  const mode = props.collapsible ?? 'icon'
  const [uncontrolledOpen, setUncontrolledOpen] = useState(props.defaultOpen ?? true)
  const [uncontrolledSelected, setUncontrolledSelected] = useState(props.defaultSelected ?? '')
  const [internalExpanded, setInternalExpanded] = useState<Record<string, boolean>>({ ...(props.defaultExpandedItems ?? {}) })
  const [activeId, setActiveId] = useState(props.items[0]?.id ?? '')
  const open = mode === 'none' ? true : props.open ?? uncontrolledOpen
  const iconOnly = mode === 'icon' && !open
  const contentVisible = mode !== 'offcanvas' || open
  const setOpen = (next: boolean) => {
    if (mode === 'none' || open === next) return
    if (props.open === undefined) setUncontrolledOpen(next)
    props.onOpenChange?.(next)
  }
  const isExpanded = (item: SurfaceSidebarItem) => props.expandedItems?.[item.id]
    ?? internalExpanded[item.id] ?? props.defaultExpandedItems?.[item.id] ?? item.defaultExpanded ?? false
  const expandedMap = () => {
    const map: Record<string, boolean> = {}
    const collect = (items: readonly SurfaceSidebarItem[]) => items.forEach((item) => {
      map[item.id] = isExpanded(item)
      if (item.children?.length) collect(item.children)
    })
    collect(props.items)
    return map
  }
  const visibleRows = (): SurfaceSidebarRow[] => {
    if (!contentVisible) return []
    if (iconOnly) return props.items.map((item) => ({ item, depth: 0, parentId: undefined }))
    return surfaceSidebarVisibleRows(props.items, expandedMap())
  }
  const active = visibleRows().some((row) => row.item.id === activeId && !row.item.disabled)
    ? activeId : visibleRows().find((row) => !row.item.disabled)?.item.id ?? ''
  const selected = props.selected ?? uncontrolledSelected
  const setExpanded = (item: SurfaceSidebarItem, expanded: boolean) => {
    if (props.expandedItems === undefined) {
      setInternalExpanded((current) => ({ ...current, [item.id]: expanded }))
    }
    props.onExpandedChange?.(item.id, expanded)
  }
  const choose = (item: SurfaceSidebarItem) => {
    if (item.disabled) return
    if (props.selected === undefined) setUncontrolledSelected(item.id)
    props.onSelectedChange?.(item.id)
    item.onSelect?.()
    props.onSelect?.(item.id)
  }
  const activate = (item: SurfaceSidebarItem) => {
    if (item.disabled) return
    if (item.children?.length) {
      if (iconOnly) {
        setOpen(true)
        setExpanded(item, true)
      } else setExpanded(item, !isExpanded(item))
    } else choose(item)
  }
  const onKey = (payload: unknown) => {
    const key = surfaceNavigationKey(payload)
    if (!key) return
    const rows = visibleRows()
    const current = rows.find((row) => row.item.id === active)
    const moved = surfaceSidebarMove(rows, active, key)
    if (moved) { setActiveId(moved); return }
    if (key === 'ArrowRight' && current?.item.children?.length) {
      if (iconOnly) { setOpen(true); setExpanded(current.item, true) }
      else if (!isExpanded(current.item)) setExpanded(current.item, true)
      else setActiveId(rows.find((row) => row.parentId === current.item.id && !row.item.disabled)?.item.id ?? active)
    } else if (key === 'ArrowLeft' && current) {
      if (current.item.children?.length && isExpanded(current.item)) setExpanded(current.item, false)
      else if (current.parentId) setActiveId(current.parentId)
    } else if ((key === 'Enter' || surfaceNavigationSpace(key)) && current) activate(current.item)
  }
  const panelWidth = mode === 'offcanvas' && !open ? 42
    : iconOnly ? Math.max(48, props.iconWidth ?? 64) : Math.max(160, props.width ?? 248)
  const renderItem = (item: SurfaceSidebarItem, depth: number): ReactElement => {
    const key = `${props.id}-item-${surfaceNavigationIdPart(item.id)}`
    const group = !!item.children?.length
    const isOpen = group && isExpanded(item) && !iconOnly
    const isSelected = selected === item.id
    const isActive = active === item.id
    const disabled = !!item.disabled
    return <column key={`${key}-branch`} width="fill" gap={2}>
      <focusScope nativeKey={key} role="button" accessible_name={item.label} tooltip={iconOnly ? item.label : undefined}
        controls={group ? `${key}-children` : undefined} expandable={group} expanded={group ? isExpanded(item) : undefined}
        selected={isSelected} enabled={!disabled} accessible_disabled={disabled}
        focusable={false} keyboard_activation="none"
        onClick={() => { if (!disabled) { setActiveId(item.id); activate(item) } }}
        onSemanticAction={(payload) => {
          if (payload.action === 'click' && !disabled) { setActiveId(item.id); activate(item) }
          else if (payload.action === 'expand' && group && !disabled) setExpanded(item, true)
          else if (payload.action === 'collapse' && group && !disabled) setExpanded(item, false)
        }}>
        <touchArea enabled={!disabled} mouse_cursor={disabled ? 'not_allowed' : 'pointer'}
          onPointerEnter={() => { if (!disabled) setActiveId(item.id) }}>
          <rectangle width="fill" height={36} radius={props.theme.controlRadius}
            background={isSelected ? props.theme.accent : isActive ? props.theme.surfaceHover : '#00000000'}
            border_color={isSelected ? props.theme.accent : '#00000000'} border_width={isSelected ? 1 : 0}
            opacity={disabled ? 0.48 : 1}>
            <row width="fill" height="fill" gap={10} padding_left={iconOnly ? 10 : 10 + depth * 12}
              padding_right={9} align_items="center" justify_content={iconOnly ? 'center' : undefined}>
              <text text={iconOnly ? item.icon ?? item.label.slice(0, 1).toUpperCase() : item.icon ?? '•'}
                color={isSelected ? props.theme.accentText : props.theme.accent} font_size={14} />
              {!iconOnly ? <text text={item.label} color={isSelected ? props.theme.accentText : props.theme.foreground}
                font_size={props.theme.controlFontSize} /> : null}
              {!iconOnly ? <container grow={1} /> : null}
              {!iconOnly && item.badge ? <text text={item.badge} color={props.theme.muted} font_size={11} /> : null}
              {!iconOnly && group ? <text text="⌄" color={props.theme.muted} font_size={12}
                rotation={isOpen ? 180 : 0} /> : null}
            </row>
          </rectangle>
        </touchArea>
      </focusScope>
      {isOpen ? <column nativeKey={`${key}-children`} width="fill" gap={2} padding_left={6}
        role="group" accessible_name={`${item.label} links`} labelled_by={key}>
        {item.children!.map((child) => renderItem(child, depth + 1))}
      </column> : null}
    </column>
  }
  const toggleId = `${props.id}-toggle`
  return <focusScope nativeKey={props.id} role="navigation" accessible_name={props.label ?? 'Sidebar'}
    active_descendant={active ? `${props.id}-item-${surfaceNavigationIdPart(active)}` : undefined}
    orientation="vertical" focusable={true} focus_on_tab_navigation={true} onKey={onKey}>
    <rectangle width={panelWidth} height={props.height ?? 360} background={props.theme.surfaceRaised}
      border_color={props.theme.border} border_width={1} radius={props.theme.controlRadius}>
      <column width="fill" height="fill" gap={6} padding={7} scroll_y={contentVisible}>
        {mode !== 'none' ? <focusScope nativeKey={toggleId} role="button"
          accessible_name={open ? 'Collapse sidebar' : 'Expand sidebar'}
          expanded={open} expandable={true} controls={`${props.id}-items`}
          keyboard_activation="enter_or_space" onClick={() => setOpen(!open)}
          onSemanticAction={(payload) => {
            if (payload.action === 'click') setOpen(!open)
            else if (payload.action === 'expand') setOpen(true)
            else if (payload.action === 'collapse') setOpen(false)
          }}>
          <touchArea mouse_cursor="pointer">
            <rectangle width="fill" height={36} radius={props.theme.controlRadius}
              background={props.theme.surface} border_color={props.theme.border} border_width={1}>
              <row width="fill" height="fill" gap={8} padding={9} align_items="center" justify_content={iconOnly || !contentVisible ? 'center' : undefined}>
                <text text={open ? '‹' : '›'} color={props.theme.foreground} font_size={16} />
                {contentVisible && !iconOnly ? <text text={props.heading ?? props.label ?? 'Navigation'}
                  color={props.theme.foreground} font_size={props.theme.controlFontSize} weight={600} /> : null}
              </row>
            </rectangle>
          </touchArea>
        </focusScope> : null}
        {contentVisible ? <column nativeKey={`${props.id}-items`} width="fill" gap={2}>
          {visibleRows().map((row) => row.depth === 0 ? renderItem(row.item, row.depth) : null)}
        </column> : null}
      </column>
    </rectangle>
  </focusScope>
}
