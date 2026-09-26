import { expect, test } from 'bun:test'
import {
  surfaceBFlatten, surfaceBMove, surfaceBMoveMenubar, surfaceBSearch,
  type SurfaceMenuItem,
} from '../src/shared/surface-b'

const items: readonly SurfaceMenuItem[] = [
  { type: 'group', id: 'file', label: 'File', items: [
    { type: 'item', id: 'new', label: 'New' },
    { type: 'item', id: 'open', label: 'Open', disabled: true },
    { type: 'submenu', id: 'recent', label: 'Recent', items: [
      { type: 'item', id: 'one', label: 'One' },
    ] },
  ] },
  { type: 'separator', id: 'divider' },
  { type: 'checkbox', id: 'autosave', label: 'Autosave' },
]

test('menu groups flatten into labelled rows while submenus remain nested', () => {
  expect(surfaceBFlatten(items).map((item) => item.id)).toEqual([
    'file-label', 'new', 'open', 'recent', 'divider', 'autosave',
  ])
})

test('keyboard navigation skips disabled rows, wraps, and searches labels', () => {
  expect(surfaceBMove(items, 'new', 'ArrowDown')).toBe('recent')
  expect(surfaceBMove(items, 'autosave', 'ArrowDown')).toBe('new')
  expect(surfaceBMove(items, 'new', 'ArrowUp')).toBe('autosave')
  expect(surfaceBMove(items, 'recent', 'Home')).toBe('new')
  expect(surfaceBMove(items, 'new', 'End')).toBe('autosave')
  expect(surfaceBSearch(items, 'new', 'au')).toBe('autosave')
  expect(surfaceBSearch(items, 'new', 'op')).toBeUndefined()
})

test('menubar keyboard navigation skips disabled menus and wraps', () => {
  const menus = [
    { id: 'file', label: 'File', items },
    { id: 'edit', label: 'Edit', items, disabled: true },
    { id: 'view', label: 'View', items },
  ]
  expect(surfaceBMoveMenubar(menus, 'file', 'ArrowRight')).toBe('view')
  expect(surfaceBMoveMenubar(menus, 'file', 'ArrowLeft')).toBe('view')
  expect(surfaceBMoveMenubar(menus, 'view', 'ArrowRight')).toBe('file')
})
