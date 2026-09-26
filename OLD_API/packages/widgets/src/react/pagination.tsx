/** @jsxImportSource @argui/react */
import { createContext, createElement, useContext, useState, type ReactElement, type ReactNode } from 'react'
import type { Palette } from '../shared/theme'

interface PaginationContextValue {
  id(): string
  theme(): Palette
  page(): number
  pageCount(): number
  navigate(page: number): void
}

const PaginationContext = createContext<PaginationContextValue | null>(null)

/** Props for a controlled or uncontrolled pagination navigation landmark. */
export interface PaginationProps {
  /** Stable prefix used to identify page controls. */ id: string
  /** Palette used for page, previous, and next controls. */ theme: Palette
  /** Accessible name for the navigation landmark. Defaults to Pagination. */ label?: string
  /** Total number of pages; values below zero are treated as zero. */ pageCount: number
  /** Current page; omit to keep page state internally. */ page?: number
  /** Initial page used only when `page` is omitted. Defaults to one. */ defaultPage?: number
  /** Called after requesting a different in-range page. */ onPageChange?: (page: number) => void
  /** List of page links and navigation actions. */ children: ReactNode
}

/** Provides clamped page state to the composed pagination controls. */
export function ReactPagination(props: PaginationProps): ReactElement {
  const [uncontrolled, setUncontrolled] = useState(props.defaultPage ?? 1)
  const pageCount = () => Math.max(0, Math.floor(props.pageCount))
  const page = () => {
    const current = props.page ?? uncontrolled
    return pageCount() === 0 ? 1 : Math.min(pageCount(), Math.max(1, Math.floor(current)))
  }
  const navigate = (requested: number) => {
    if (pageCount() === 0) return
    const next = Math.min(pageCount(), Math.max(1, Math.floor(requested)))
    if (next === page()) return
    if (props.page === undefined) setUncontrolled(next)
    props.onPageChange?.(next)
  }
  const context: PaginationContextValue = {
    id: () => props.id, theme: () => props.theme, page, pageCount, navigate,
  }
  return createElement(PaginationContext.Provider, { value: context },
    <focusScope width="fill" role="navigation" accessible_name={props.label ?? 'Pagination'} focusable={false}>
      {props.children}
    </focusScope>) as ReactElement
}

/** Props for the semantic list containing pagination entries. */
export interface PaginationContentProps { /** Page links and action buttons. */ children: ReactNode }

/** Renders pagination entries as a horizontally centered list. */
export function ReactPaginationContent(props: PaginationContentProps): ReactElement {
  return <row width="fill" wrap={true} gap={5} align_items="center" justify_content="center" role="list">
    {props.children}
  </row>
}

/** Props for one semantic list item. */
export interface PaginationItemProps { /** One page link, navigation action, or decorative ellipsis. */ children: ReactNode }

/** Marks a pagination entry as a list item. */
export function ReactPaginationItem(props: PaginationItemProps): ReactElement {
  return <row role="list_item" align_items="center">{props.children}</row>
}

/** Props for an in-range numbered page link. */
export interface PaginationLinkProps {
  /** One-based destination page. */ page: number
  /** Optional accessible name; defaults to the page number. */ label?: string
  /** Size of the page target. Defaults to a square icon-sized control. */ size?: 'icon' | 'default'
  /** Whether the control is disabled. */ disabled?: boolean
  /** Visible page number or custom native content. */ children?: ReactNode
}

/** Renders an accessible page link and updates the enclosing pagination state. */
export function ReactPaginationLink(props: PaginationLinkProps): ReactElement {
  const pagination = useContext(PaginationContext)
  if (!pagination) throw new Error('ReactPaginationLink must be rendered inside ReactPagination')
  const active = pagination.page() === props.page
  return <PageButton pagination={pagination} page={props.page} targetId={`${pagination.id()}-page-${props.page}`}
    label={props.label ?? String(props.page)}
    disabled={props.disabled} active={active} size={props.size ?? 'icon'}>
    {props.children ?? <text text={String(props.page)} color={active ? pagination.theme().foreground : pagination.theme().muted}
      font_size={13} weight={active ? 600 : 400} />}
  </PageButton>
}

interface PageButtonProps {
  pagination: PaginationContextValue
  page: number
  targetId: string
  label: string
  active?: boolean
  disabled?: boolean
  size?: 'icon' | 'default'
  children: ReactNode
}

function PageButton(props: PageButtonProps): ReactElement {
  const [focused, setFocused] = useState(false)
  const [hovered, setHovered] = useState(false)
  const theme = props.pagination.theme()
  const inactive = !!props.disabled || props.page < 1 || props.page > props.pagination.pageCount()
  const activate = () => { if (!inactive) props.pagination.navigate(props.page) }
  const width = props.size === 'default' ? undefined : 36
  return <focusScope nativeKey={props.targetId} role="link" accessible_name={props.label} current={props.active ? 'page' : undefined}
    selected={props.active} enabled={!inactive} accessible_disabled={inactive}
    position_in_set={props.page >= 1 && props.page <= props.pagination.pageCount() ? props.page : undefined}
    set_size={props.pagination.pageCount() || undefined} keyboard_activation="enter" onClick={activate}
    onFocus={() => setFocused(true)} onBlur={() => setFocused(false)}>
    <touchArea enabled={!inactive} mouse_cursor={inactive ? 'not_allowed' : 'pointer'}
      onPointerEnter={() => setHovered(true)} onPointerLeave={() => setHovered(false)}>
      <rectangle width={width} height={36}
        background={props.active ? theme.surfaceRaised : hovered || focused ? theme.surfaceHover : '#00000000'}
        border_color={props.active || focused ? theme.accent : theme.border}
        border_width={props.active || focused ? 1 : 0} radius={theme.controlRadius} opacity={inactive ? 0.48 : 1}>
        <row width="fill" height="fill" gap={6} padding_left={props.size === 'default' ? 10 : 0}
          padding_right={props.size === 'default' ? 10 : 0} align_items="center" justify_content="center">{props.children}</row>
      </rectangle>
    </touchArea>
  </focusScope>
}

/** Props for the previous-page action. */
export interface PaginationPreviousProps { /** Accessible name; defaults to Go to previous page. */ label?: string }

/** Moves to the previous page and disables itself at the first page. */
export function ReactPaginationPrevious(props: PaginationPreviousProps = {}): ReactElement {
  const pagination = useContext(PaginationContext)
  if (!pagination) throw new Error('ReactPaginationPrevious must be rendered inside ReactPagination')
  const page = pagination.page() - 1
  const disabled = pagination.page() <= 1
  const theme = pagination.theme()
  return <PageButton pagination={pagination} page={page} targetId={`${pagination.id()}-previous`}
    label={props.label ?? 'Go to previous page'} disabled={disabled} size="default">
    <text text="‹" color={disabled ? theme.muted : theme.foreground} font_size={18} />
    <text text="Previous" color={disabled ? theme.muted : theme.foreground} font_size={13} />
  </PageButton>
}

/** Props for the next-page action. */
export interface PaginationNextProps { /** Accessible name; defaults to Go to next page. */ label?: string }

/** Moves to the next page and disables itself at the last page. */
export function ReactPaginationNext(props: PaginationNextProps = {}): ReactElement {
  const pagination = useContext(PaginationContext)
  if (!pagination) throw new Error('ReactPaginationNext must be rendered inside ReactPagination')
  const page = pagination.page() + 1
  const disabled = pagination.page() >= pagination.pageCount()
  const theme = pagination.theme()
  return <PageButton pagination={pagination} page={page} targetId={`${pagination.id()}-next`}
    label={props.label ?? 'Go to next page'} disabled={disabled} size="default">
    <text text="Next" color={disabled ? theme.muted : theme.foreground} font_size={13} />
    <text text="›" color={disabled ? theme.muted : theme.foreground} font_size={18} />
  </PageButton>
}

/** Props for a decorative break between separated ranges of pages. */
export interface PaginationEllipsisProps { /** Accessible description for assistive technology; decorative by default. */ label?: string }

/** Renders a noninteractive, accessibility-hidden page-range ellipsis. */
export function ReactPaginationEllipsis(_props: PaginationEllipsisProps = {}): ReactElement {
  const pagination = useContext(PaginationContext)
  if (!pagination) throw new Error('ReactPaginationEllipsis must be rendered inside ReactPagination')
  return <row width={36} height={36} role={_props.label ? 'text' : undefined} accessible_name={_props.label}
    accessible_hidden={!_props.label} align_items="center" justify_content="center">
    <text text="…" color={pagination.theme().muted} font_size={16} />
  </row>
}
