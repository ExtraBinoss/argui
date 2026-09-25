/** @jsxImportSource @argui/react */
import type { AssetRef } from '@argui/host'
import { useRef, useState, type ReactElement, type ReactNode } from 'react'
import type { Palette } from '../shared/theme'

/** Surface treatment for one item in a list or settings panel. */
export type ItemVariant = 'default' | 'outline' | 'muted'

/** Density of an item row. */
export type ItemSize = 'default' | 'sm'

/** Props for a native content row with optional pointer and keyboard activation. */
export interface ItemProps {
  /** Resolved application colors and sizing tokens. */
  theme: Palette
  /** Leading media, text content, and optional trailing actions. */
  children: ReactNode
  /** Optional stable native key for a row that moves in a list. */
  id?: string
  /** Accessible name when the row is interactive or has abbreviated content. */
  label?: string
  /** Row surface treatment. */
  variant?: ItemVariant
  /** Row padding and content spacing. */
  size?: ItemSize
  /** Exposes selection to accessibility APIs and adds an accent border. */
  selected?: boolean
  /** Blocks activation while retaining the row's position in its list. */
  disabled?: boolean
  /** Makes the full row activate on click, Enter, and Space. */
  onClick?: () => void
}

const mobile = (globalThis as { __arguiMobile?: boolean }).__arguiMobile === true

/** Renders a themed list item, adding native button behavior when an action is supplied. */
export function ReactItem(props: ItemProps): ReactElement {
  const [hovered, setHovered] = useState(false)
  const [pressed, setPressed] = useState(false)
  const [focused, setFocused] = useState(false)
  const pointerFocus = useRef(false)
  const variant = props.variant ?? 'default'
  const size = props.size ?? 'default'
  const interactive = props.onClick !== undefined
  const fill = pressed ? props.theme.surfacePressed
    : hovered && interactive ? props.theme.surfaceHover
      : variant === 'muted' ? props.theme.surfaceRaised : '#00000000'
  const border = focused || props.selected ? props.theme.accent
    : variant === 'outline' ? props.theme.border : '#00000000'
  const activate = () => { if (interactive && !props.disabled) props.onClick?.() }
  const content = <touchArea enabled={interactive && !props.disabled} mouse_cursor={props.disabled ? 'not_allowed' : interactive ? 'pointer' : 'default'}
    onPointerEnter={() => { if (!mobile && interactive && !props.disabled) setHovered(true) }}
    onPointerLeave={() => { setHovered(false); setPressed(false) }}
    onPointerDown={() => { if (interactive && !props.disabled) { pointerFocus.current = true; setPressed(true); setFocused(false) } }}
    onPointerUp={() => setPressed(false)} onPointerCancel={() => setPressed(false)}>
    <rectangle width="fill" background={fill} border_color={border}
      border_width={focused || props.selected || variant === 'outline' ? 1 : 0}
      radius={props.theme.controlRadius} opacity={props.disabled ? 0.48 : 1}
      transition_ms={120}>
      <row width="fill" gap={size === 'sm' ? 10 : 16} padding={size === 'sm' ? 12 : 16}
        align_items="center" wrap={true}>{props.children}</row>
    </rectangle>
  </touchArea>
  return <focusScope nativeKey={props.id ? `${props.id}-item` : undefined} role="list_item"
    accessible_name={interactive ? undefined : props.label} selected={props.selected}>
    {interactive ? <focusScope nativeKey={props.id} role="button" accessible_name={props.label}
      enabled={!props.disabled} accessible_disabled={props.disabled} keyboard_activation="enter_or_space"
      onClick={activate}
      onFocus={() => { if (!pointerFocus.current) setFocused(true) }}
      onKey={() => { pointerFocus.current = false; setFocused(true) }}
      onBlur={() => { pointerFocus.current = false; setFocused(false) }}>
      {content}
    </focusScope> : content}
  </focusScope>
}

/** Props for the optional leading visual in an item. */
export interface ItemMediaProps {
  /** Resolved application colors. */
  theme: Palette
  /** Leading visual style. */
  variant?: 'default' | 'icon' | 'image'
  /** Optional preloaded SVG or raster asset. */
  source?: AssetRef
  /** Alternative text for meaningful media; omitted media is decorative. */
  alt?: string
  /** Custom native visual used when no asset is supplied. */
  children?: ReactNode
  /** Width and height of the media frame. */
  size?: number
}

/** Displays an optional icon, image, or caller-composed native visual beside item content. */
export function ReactItemMedia(props: ItemMediaProps): ReactElement {
  const variant = props.variant ?? 'default'
  const size = Number.isFinite(props.size) && (props.size ?? 0) > 0 ? props.size! : variant === 'image' ? 40 : 32
  const accessible = !!props.alt
  const media = props.source
    ? props.source.kind === 'image'
      ? <image source={props.source} alt={props.alt ?? ''} width="fill" height="fill"
        fit={variant === 'image' ? 'cover' : 'contain'} accessible_hidden={true} />
      : <svg source={props.source} alt={props.alt ?? ''} color={props.theme.foreground}
        width={size} height={size} accessible_hidden={true} />
    : props.children
  return <rectangle width={size} height={size} clip={variant === 'image'}
    radius={variant === 'icon' ? props.theme.controlRadius / 2 : 0}
    background={variant === 'icon' ? props.theme.surfaceRaised : '#00000000'}
    border_color={variant === 'icon' ? props.theme.border : '#00000000'}
    border_width={variant === 'icon' ? 1 : 0}
    role={accessible ? 'image' : undefined} accessible_name={props.alt} accessible_hidden={!accessible}>
    {media}
  </rectangle>
}

/** Props for the text and description column within an item. */
export interface ItemContentProps {
  /** Text elements stacked in the available row space. */
  children: ReactNode
}

/** Provides flexible width and consistent spacing for an item's primary content. */
export function ReactItemContent(props: ItemContentProps): ReactElement {
  return <column grow={1} min_width={0} gap={4}>{props.children}</column>
}

/** Props for an item's main label. */
export interface ItemTitleProps {
  /** Resolved application colors. */
  theme: Palette
  /** Short primary label. */
  text: string
}

/** Renders the primary item label with medium emphasis. */
export function ReactItemTitle(props: ItemTitleProps): ReactElement {
  return <text width="fill" text={props.text} color={props.theme.foreground}
    font_size={props.theme.controlFontSize} weight={500} line_height={1.25} />
}

/** Props for a secondary item description. */
export interface ItemDescriptionProps {
  /** Resolved application colors. */
  theme: Palette
  /** Supporting text for the item. */
  text: string
}

/** Renders muted, two-line supporting text for an item. */
export function ReactItemDescription(props: ItemDescriptionProps): ReactElement {
  return <text width="fill" text={props.text} color={props.theme.muted}
    font_size={13} line_height={1.35} line_clamp={2} />
}

/** Props for a row of trailing item actions or status labels. */
export interface ItemActionsProps {
  /** Action controls or compact status content. */
  children: ReactNode
  /** Space between trailing children. */
  gap?: number
}

/** Arranges trailing item actions in a compact row. */
export function ReactItemActions(props: ItemActionsProps): ReactElement {
  const requestedGap = props.gap ?? 8
  const gap = Number.isFinite(requestedGap) ? Math.max(0, requestedGap) : 8
  return <row shrink={0} gap={gap} align_items="center">{props.children}</row>
}

/** Props for a set of related items. */
export interface ItemGroupProps {
  /** Items and optional separators in the group. */
  children: ReactNode
  /** Accessible name for the list. */
  label?: string
  /** Spacing between adjacent children. */
  gap?: number
}

/** Exposes a vertical set of item rows as a native accessible list. */
export function ReactItemGroup(props: ItemGroupProps): ReactElement {
  const requestedGap = props.gap ?? 0
  const gap = Number.isFinite(requestedGap) ? Math.max(0, requestedGap) : 0
  return <column width="fill" gap={gap} role="list" accessible_name={props.label}>{props.children}</column>
}

/** Props for an optional divider between related items. */
export interface ItemSeparatorProps {
  /** Resolved application colors. */
  theme: Palette
  /** Divider thickness in native layout units. */
  thickness?: number
  /** Whether assistive technology should skip the divider. */
  decorative?: boolean
}

/** Draws a horizontal themed divider between items. */
export function ReactItemSeparator(props: ItemSeparatorProps): ReactElement {
  const requestedThickness = props.thickness ?? 1
  const thickness = Number.isFinite(requestedThickness) && requestedThickness > 0 ? requestedThickness : 1
  const decorative = props.decorative ?? true
  return <rectangle width="fill" height={thickness} background={props.theme.border}
    role={decorative ? undefined : 'separator'} orientation="horizontal"
    accessible_hidden={decorative} />
}

/** Props for a full-width row at the top of an item. */
export interface ItemHeaderProps {
  /** Header labels or media. */
  children: ReactNode
}

/** Arranges an item's leading heading content and trailing metadata. */
export function ReactItemHeader(props: ItemHeaderProps): ReactElement {
  return <row width="fill" gap={8} align_items="center" justify_content="space_between">{props.children}</row>
}

/** Props for supplemental content along the bottom of an item. */
export interface ItemFooterProps {
  /** Footer labels or actions. */
  children: ReactNode
}

/** Arranges an item's footer content across the available row width. */
export function ReactItemFooter(props: ItemFooterProps): ReactElement {
  return <row width="fill" gap={8} align_items="center" justify_content="space_between">{props.children}</row>
}
