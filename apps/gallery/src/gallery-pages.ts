/** Ordered gallery destinations shared by the Solid and React shells. */
export const pages = [
  'Button', 'Input', 'Select', 'Popover', 'Media', 'Internationalization', 'Accessibility', 'Overlay', 'Animation Lab', 'Damage Control', 'WGSL Lab',
] as const

export type Page = typeof pages[number]

/** A destination or noninteractive section label within the virtual navigation. */
export type NavigationItem = { kind: 'page'; page: Page } | { kind: 'heading'; label: 'COMPONENTS' | 'EXAMPLES' }

/** Ordered navigation rows, including a distinct Examples section. */
export const navigationItems: readonly NavigationItem[] = [
  { kind: 'heading', label: 'COMPONENTS' },
  { kind: 'page', page: 'Button' },
  { kind: 'page', page: 'Input' },
  { kind: 'page', page: 'Select' },
  { kind: 'page', page: 'Popover' },
  { kind: 'page', page: 'Media' },
  { kind: 'page', page: 'Internationalization' },
  { kind: 'page', page: 'Accessibility' },
  { kind: 'page', page: 'Overlay' },
  { kind: 'page', page: 'Animation Lab' },
  { kind: 'heading', label: 'EXAMPLES' },
  { kind: 'page', page: 'Damage Control' },
  { kind: 'page', page: 'WGSL Lab' },
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
