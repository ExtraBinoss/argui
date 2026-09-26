import { widgetPages } from './widget-pages'

const componentPages = ['Button', 'Input', 'Select', 'Popover', 'Dialog', ...widgetPages] as const
const examplePages = [
  'Media', 'Services', 'Theming', 'Typography', 'Date Picker', 'Data Table', 'Internationalization', 'Accessibility', 'Overlay', 'Animation Lab', 'Damage Control', 'WGSL Lab',
] as const

/** Ordered gallery destinations shared by the Solid and React shells. */
export const pages = [...componentPages, ...examplePages] as const

export type Page = typeof pages[number]

/** A destination or noninteractive section label within the virtual navigation. */
export type NavigationItem = { kind: 'page'; page: Page } | { kind: 'heading'; label: string }

/** Ordered navigation rows, including a distinct Examples section. */
export const navigationItems: readonly NavigationItem[] = [
  { kind: 'heading', label: 'COMPONENTS' },
  ...componentPages.slice().sort().map((page): NavigationItem => ({ kind: 'page', page })),
  { kind: 'heading', label: 'EXAMPLES' },
  ...examplePages.slice().sort().map((page): NavigationItem => ({ kind: 'page', page })),
]

/** Returns the stable key for a navigation row or heading. */
export function navigationKey(item: NavigationItem): string {
  return item.kind === 'page' ? pageKey(item.page) : `section-${item.label.toLowerCase()}`
}

/** Returns the stable native key for one gallery navigation item. */
export function pageKey(page: Page): string {
  return `page-${page.replaceAll(' ', '-').toLowerCase()}`
}

/** Filters gallery destinations while retaining headings for sections with matches. */
export function filteredNavigation(query: string): readonly NavigationItem[] {
  const needle = query.trim().toLocaleLowerCase()
  if (!needle) return navigationItems
  const matches: NavigationItem[] = []
  let heading: NavigationItem | undefined
  for (const item of navigationItems) {
    if (item.kind === 'heading') {
      heading = item
    } else if (item.page.toLocaleLowerCase().includes(needle)) {
      if (heading) matches.push(heading)
      heading = undefined
      matches.push(item)
    }
  }
  return matches
}

/** Gives filtered native lists a stable data version for measurement resets. */
export function navigationVersion(query: string): number {
  let hash = 0
  for (const character of query.trim().toLocaleLowerCase()) {
    hash = (Math.imul(hash, 31) + character.codePointAt(0)!) | 0
  }
  return hash
}
