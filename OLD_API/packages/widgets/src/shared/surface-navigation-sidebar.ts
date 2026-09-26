/** A link rendered inside a navigation menu panel. */
export interface SurfaceNavigationLink {
  id: string
  label: string
  description?: string
  href?: string
  disabled?: boolean
}

/** One top-level navigation destination or a trigger for a link panel. */
export interface SurfaceNavigationItem {
  id: string
  label: string
  description?: string
  href?: string
  links?: readonly SurfaceNavigationLink[]
  disabled?: boolean
}

/** A selectable or expandable entry in a native sidebar navigation tree. */
export interface SurfaceSidebarItem {
  id: string
  label: string
  icon?: string
  badge?: string
  disabled?: boolean
  children?: readonly SurfaceSidebarItem[]
  defaultExpanded?: boolean
  onSelect?: () => void
}

/** A visible sidebar row with its nesting and parent relation. */
export interface SurfaceSidebarRow {
  item: SurfaceSidebarItem
  depth: number
  parentId?: string
}

/** Encodes a user-provided identifier for a stable native key suffix. */
export function surfaceNavigationIdPart(value: string): string {
  const encoded = Array.from(value, (character) => character.codePointAt(0)!.toString(16)).join('-')
  return encoded || 'empty'
}

/** Reads a pressed keyboard key from web or native callback payloads. */
export function surfaceNavigationKey(payload: unknown): string | undefined {
  if (typeof payload === 'string') return payload
  if (typeof payload !== 'object' || payload === null) return undefined
  const event = payload as Record<string, unknown>
  if (event.state !== undefined && event.state !== 'pressed') return undefined
  return typeof event.key === 'string' ? event.key : undefined
}

/** Recognizes the common web and native spellings for the Space key. */
export function surfaceNavigationSpace(key: string): boolean {
  return key === ' ' || key === 'Space' || key === 'Spacebar'
}

/** Moves among enabled top-level navigation items with wrapping arrow keys. */
export function surfaceNavigationMove(
  items: readonly SurfaceNavigationItem[],
  currentId: string | undefined,
  key: string,
  orientation: 'horizontal' | 'vertical',
): string | undefined {
  const entries = items.filter((item) => !item.disabled)
  if (!entries.length) return undefined
  const index = entries.findIndex((item) => item.id === currentId)
  if (key === 'Home') return entries[0]!.id
  if (key === 'End') return entries[entries.length - 1]!.id
  const forward = orientation === 'horizontal' ? 'ArrowRight' : 'ArrowDown'
  const backward = orientation === 'horizontal' ? 'ArrowLeft' : 'ArrowUp'
  if (key === forward) return entries[(index + 1 + entries.length) % entries.length]!.id
  if (key === backward) return entries[(index < 0 ? entries.length - 1 : index - 1 + entries.length) % entries.length]!.id
  return undefined
}

/** Flattens only expanded sidebar branches for roving active-descendant navigation. */
export function surfaceSidebarVisibleRows(
  items: readonly SurfaceSidebarItem[],
  expanded: Readonly<Record<string, boolean>>,
): SurfaceSidebarRow[] {
  const rows: SurfaceSidebarRow[] = []
  const visit = (entries: readonly SurfaceSidebarItem[], depth: number, parentId?: string) => {
    for (const item of entries) {
      rows.push({ item, depth, parentId })
      if (item.children?.length && expanded[item.id]) visit(item.children, depth + 1, item.id)
    }
  }
  visit(items, 0)
  return rows
}

/** Finds an enabled sidebar row for arrow and edge navigation. */
export function surfaceSidebarMove(
  rows: readonly SurfaceSidebarRow[],
  currentId: string | undefined,
  key: string,
): string | undefined {
  const entries = rows.filter((row) => !row.item.disabled)
  if (!entries.length) return undefined
  const index = entries.findIndex((row) => row.item.id === currentId)
  if (key === 'Home') return entries[0]!.item.id
  if (key === 'End') return entries[entries.length - 1]!.item.id
  if (key === 'ArrowDown') return entries[(index + 1 + entries.length) % entries.length]!.item.id
  if (key === 'ArrowUp') return entries[(index < 0 ? entries.length - 1 : index - 1 + entries.length) % entries.length]!.item.id
  return undefined
}
