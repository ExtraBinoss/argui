import { expect, test } from 'bun:test'
import {
  surfaceNavigationMove, surfaceSidebarMove, surfaceSidebarVisibleRows,
  type SurfaceNavigationItem, type SurfaceSidebarItem,
} from '../src/shared/surface-navigation-sidebar'

test('navigation menu skips disabled destinations and wraps by orientation', () => {
  const items: readonly SurfaceNavigationItem[] = [
    { id: 'file', label: 'File' },
    { id: 'edit', label: 'Edit', disabled: true },
    { id: 'view', label: 'View' },
  ]
  expect(surfaceNavigationMove(items, 'file', 'ArrowRight', 'horizontal')).toBe('view')
  expect(surfaceNavigationMove(items, 'view', 'ArrowRight', 'horizontal')).toBe('file')
  expect(surfaceNavigationMove(items, 'file', 'ArrowUp', 'vertical')).toBe('view')
  expect(surfaceNavigationMove(items, undefined, 'End', 'vertical')).toBe('view')
})

test('sidebar exposes expanded children with parent links and skips disabled rows', () => {
  const items: readonly SurfaceSidebarItem[] = [
    { id: 'work', label: 'Work', children: [
      { id: 'active', label: 'Active' },
      { id: 'archived', label: 'Archived', disabled: true },
    ] },
    { id: 'settings', label: 'Settings' },
  ]
  expect(surfaceSidebarVisibleRows(items, {}).map((row) => row.item.id)).toEqual(['work', 'settings'])
  const rows = surfaceSidebarVisibleRows(items, { work: true })
  expect(rows.map((row) => [row.item.id, row.depth, row.parentId])).toEqual([
    ['work', 0, undefined], ['active', 1, 'work'], ['archived', 1, 'work'],
    ['settings', 0, undefined],
  ])
  expect(surfaceSidebarMove(rows, 'active', 'ArrowDown')).toBe('settings')
  expect(surfaceSidebarMove(rows, 'work', 'ArrowUp')).toBe('settings')
})
