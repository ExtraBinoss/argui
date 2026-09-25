import { createSignal } from '@argui/solid'
import type { JSX } from '@argui/solid/jsx-runtime'
import type { Palette } from '../shared/theme'

/** Props for a named native navigation landmark. */
export interface BreadcrumbProps {
  /** Palette used for link and separator colors. */
  theme: Palette
  /** Accessible name for this navigation landmark. Defaults to Breadcrumb. */
  label?: string
  /** The breadcrumb list and its items. */
  children: JSX.Element
}

/** Exposes breadcrumb contents as a labeled navigation landmark. */
export function Breadcrumb(props: BreadcrumbProps): JSX.Element {
  return <focusScope width="fill" role="navigation" accessible_name={props.label ?? 'Breadcrumb'} focusable={false}>
    {props.children}
  </focusScope>
}

/** Props for the wrapping list of breadcrumb entries. */
export interface BreadcrumbListProps {
  /** Palette used for list typography. */
  theme: Palette
  /** Breadcrumb items and separators. */
  children: JSX.Element
  /** Space between adjacent list entries. Defaults to 7 logical pixels. */
  gap?: number
}

/** Renders a wrapping semantic list of breadcrumb entries. */
export function BreadcrumbList(props: BreadcrumbListProps): JSX.Element {
  return <row width="fill" wrap={true} gap={props.gap ?? 7} align_items="center" role="list">
    {props.children}
  </row>
}

/** Props for an individual breadcrumb list entry. */
export interface BreadcrumbItemProps {
  /** Palette used for entry alignment. */
  theme: Palette
  /** A link, current page, or ellipsis. */
  children: JSX.Element
}

/** Marks a breadcrumb link or current page as one item in the navigation list. */
export function BreadcrumbItem(props: BreadcrumbItemProps): JSX.Element {
  return <row role="list_item" align_items="center" gap={6}>{props.children}</row>
}

/** Props for a keyboard-activated breadcrumb link. */
export interface BreadcrumbLinkProps {
  /** Stable key for the native focus target. */
  id: string
  /** Palette used for idle, focus, and hover colors. */
  theme: Palette
  /** Visible and accessible destination name. */
  label: string
  /** Action that navigates to the destination in the containing application. */
  onNavigate: () => void
  /** Whether the destination cannot be activated. */
  disabled?: boolean
}

/** Renders a native link target whose navigation action belongs to the application. */
export function BreadcrumbLink(props: BreadcrumbLinkProps): JSX.Element {
  const [focused, setFocused] = createSignal(false)
  const [hovered, setHovered] = createSignal(false)
  const activate = () => { if (!props.disabled) props.onNavigate() }
  const color = () => props.disabled ? props.theme.muted : hovered() || focused() ? props.theme.accent : props.theme.foreground
  return <focusScope key={props.id} role="link" accessible_name={props.label} enabled={!props.disabled}
    keyboard_activation="enter" onClick={activate}
    onFocus={() => setFocused(true)} onKey={() => setFocused(true)} onBlur={() => setFocused(false)}>
    <touchArea enabled={!props.disabled} mouse_cursor={props.disabled ? 'not_allowed' : 'pointer'}
      onPointerEnter={() => setHovered(true)} onPointerLeave={() => hovered() && setHovered(false)}>
      <text text={props.label} color={color()} font_size={props.theme.controlFontSize} />
    </touchArea>
  </focusScope>
}

/** Props for the current, non-activatable breadcrumb page. */
export interface BreadcrumbPageProps {
  /** Palette used for current-page text. */
  theme: Palette
  /** Current page name. */
  label: string
}

/** Marks the final breadcrumb as the current page for assistive technology. */
export function BreadcrumbPage(props: BreadcrumbPageProps): JSX.Element {
  return <focusScope role="link" accessible_name={props.label} current="page" enabled={false} focusable={false}>
    <text text={props.label} color={props.theme.foreground} font_size={props.theme.controlFontSize} />
  </focusScope>
}

/** Props for a decorative separator between breadcrumb entries. */
export interface BreadcrumbSeparatorProps {
  /** Palette used for the default chevron glyph. */
  theme: Palette
  /** Custom separator content. */
  children?: JSX.Element
  /** Text glyph used when children are omitted. Defaults to a chevron. */
  text?: string
}

/** Separates breadcrumb entries while remaining hidden from the accessibility tree. */
export function BreadcrumbSeparator(props: BreadcrumbSeparatorProps): JSX.Element {
  return <row accessible_hidden={true} align_items="center">
    {props.children ?? <text text={props.text ?? '›'} color={props.theme.muted} font_size={14} />}
  </row>
}

/** Props for a decorative ellipsis representing omitted breadcrumb entries. */
export interface BreadcrumbEllipsisProps {
  /** Palette used for the ellipsis. */
  theme: Palette
  /** Accessible label for a surrounding action; the glyph itself remains decorative. */
  label?: string
}

/** Renders a decorative ellipsis that callers can place inside their own menu trigger. */
export function BreadcrumbEllipsis(props: BreadcrumbEllipsisProps): JSX.Element {
  return <row width={22} height={28} accessible_hidden={true} align_items="center" justify_content="center"
    tooltip={props.label}>
    <text text="…" color={props.theme.muted} font_size={16} />
  </row>
}
