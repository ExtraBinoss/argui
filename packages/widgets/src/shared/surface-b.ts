/** Common attributes accepted by selectable native menu entries. */
interface SurfaceMenuBase {
  id: string
  disabled?: boolean
  inset?: boolean
  shortcut?: string
  destructive?: boolean
}

/** A plain command that closes its menu after selection. */
export interface SurfaceMenuAction extends SurfaceMenuBase {
  type: 'item'
  label: string
  onSelect?: () => void
}

/** A checkable command that keeps its menu open after toggling. */
export interface SurfaceMenuCheckbox extends SurfaceMenuBase {
  type: 'checkbox'
  label: string
  checked?: boolean
  defaultChecked?: boolean
  onCheckedChange?: (checked: boolean) => void
}

/** One radio choice; the root menu owns the value for each named group. */
export interface SurfaceMenuRadio extends SurfaceMenuBase {
  type: 'radio'
  label: string
  group: string
  value: string
}

/** A submenu which opens in a second anchored native popup. */
export interface SurfaceMenuSubmenu extends SurfaceMenuBase {
  type: 'submenu'
  label: string
  items: readonly SurfaceMenuItem[]
}

/** A non-selectable menu heading. */
export interface SurfaceMenuLabel {
  type: 'label'
  id: string
  label: string
  inset?: boolean
}

/** A non-selectable visual separator. */
export interface SurfaceMenuSeparator {
  type: 'separator'
  id: string
}

/** A labelled or unlabelled group whose children share the parent menu. */
export interface SurfaceMenuGroup {
  type: 'group'
  id: string
  label?: string
  items: readonly SurfaceMenuItem[]
}

/** Typed menu tree shared by dropdown, context and menubar controls. */
export type SurfaceMenuItem = SurfaceMenuAction | SurfaceMenuCheckbox | SurfaceMenuRadio
  | SurfaceMenuSubmenu | SurfaceMenuLabel | SurfaceMenuSeparator | SurfaceMenuGroup

/** Flat rows rendered by one popup; nested submenu items get their own popup. */
export type SurfaceMenuRow = Exclude<SurfaceMenuItem, SurfaceMenuGroup>

/** A selectable row that can become the menu's active keyboard descendant. */
export type SurfaceMenuFocusableItem = SurfaceMenuAction | SurfaceMenuCheckbox | SurfaceMenuRadio | SurfaceMenuSubmenu

/** Shared controlled and default state for checkboxes and radio groups. */
export interface SurfaceMenuStateProps {
  checked?: Readonly<Record<string, boolean>>
  defaultChecked?: Readonly<Record<string, boolean>>
  radioValues?: Readonly<Record<string, string>>
  defaultRadioValues?: Readonly<Record<string, string>>
  onCheckedChange?: (itemId: string, checked: boolean) => void
  onRadioValueChange?: (group: string, value: string) => void
  onSelect?: (itemId: string) => void
}

/** State operations passed from one menu root to its popup and submenus. */
export interface SurfaceMenuController {
  isChecked: (item: SurfaceMenuCheckbox) => boolean
  radioValue: (group: string) => string | undefined
  toggleCheckbox: (item: SurfaceMenuCheckbox) => void
  chooseRadio: (item: SurfaceMenuRadio) => void
  select: (item: SurfaceMenuAction) => void
}

/** One titled menu in a menubar. */
export interface SurfaceMenubarMenu {
  id: string
  label: string
  items: readonly SurfaceMenuItem[]
  disabled?: boolean
}

/** Returns the pressed key name from a native keyboard event payload. */
export function surfaceBKey(payload: unknown): string | undefined {
  if (typeof payload === 'string') return payload
  if (typeof payload !== 'object' || payload === null) return undefined
  const event = payload as Record<string, unknown>
  if (event.state !== undefined && event.state !== 'pressed') return undefined
  return typeof event.key === 'string' ? event.key : undefined
}

/** Encodes an item value as a stable, collision-resistant native key suffix. */
export function surfaceBKeyPart(value: string): string {
  const encoded = Array.from(value, (character) => character.codePointAt(0)!.toString(16)).join('-')
  return encoded || 'empty'
}

/** Recognizes the browser and native spellings commonly used for the Space key. */
export function surfaceBIsSpaceKey(key: string): boolean {
  return key === ' ' || key === 'Space' || key === 'Spacebar'
}

/** Flattens labelled groups while retaining separators and headings as rows. */
export function surfaceBFlatten(items: readonly SurfaceMenuItem[]): SurfaceMenuRow[] {
  const rows: SurfaceMenuRow[] = []
  for (const item of items) {
    if (item.type !== 'group') {
      rows.push(item)
      continue
    }
    if (item.label) rows.push({ type: 'label', id: `${item.id}-label`, label: item.label })
    rows.push(...surfaceBFlatten(item.items))
  }
  return rows
}

/** Returns rows that can receive keyboard focus or selection. */
export function surfaceBFocusable(items: readonly SurfaceMenuItem[]): SurfaceMenuFocusableItem[] {
  return surfaceBFlatten(items).filter((item): item is SurfaceMenuFocusableItem =>
    item.type !== 'label' && item.type !== 'separator' && !item.disabled)
}

/** Resolves an enabled menu item for a navigation key, wrapping at either end. */
export function surfaceBMove(
  items: readonly SurfaceMenuItem[],
  currentId: string | undefined,
  key: string,
): string | undefined {
  const choices = surfaceBFocusable(items)
  if (!choices.length) return undefined
  const index = choices.findIndex((item) => item.id === currentId)
  if (key === 'Home') return choices[0]!.id
  if (key === 'End') return choices[choices.length - 1]!.id
  if (key === 'ArrowDown') return choices[((index + 1) % choices.length)]!.id
  if (key === 'ArrowUp') return choices[(index <= 0 ? choices.length - 1 : index - 1)]!.id
  return undefined
}

/** Finds the next enabled menu entry whose label starts with `prefix`. */
export function surfaceBSearch(
  items: readonly SurfaceMenuItem[],
  currentId: string | undefined,
  prefix: string,
): string | undefined {
  const choices = surfaceBFocusable(items)
  if (!choices.length || !prefix) return undefined
  const start = choices.findIndex((item) => item.id === currentId)
  for (let offset = 1; offset <= choices.length; offset++) {
    const item = choices[(start + offset) % choices.length]!
    if (item.label.toLocaleLowerCase().startsWith(prefix.toLocaleLowerCase())) return item.id
  }
  return undefined
}

/** Returns the next enabled menubar entry for an arrow or edge-navigation key. */
export function surfaceBMoveMenubar(
  menus: readonly SurfaceMenubarMenu[],
  currentId: string | undefined,
  key: string,
): string | undefined {
  const choices = menus.filter((menu) => !menu.disabled)
  if (!choices.length) return undefined
  const index = choices.findIndex((menu) => menu.id === currentId)
  if (key === 'Home') return choices[0]!.id
  if (key === 'End') return choices[choices.length - 1]!.id
  if (key === 'ArrowRight') return choices[(index + 1 + choices.length) % choices.length]!.id
  if (key === 'ArrowLeft') return choices[(index < 0 ? choices.length - 1 : index - 1 + choices.length) % choices.length]!.id
  return undefined
}
