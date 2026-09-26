import { expect, test } from 'bun:test'
import {
  surfaceEAddMonths, surfaceECalendarCells, surfaceECalendarKeyDate, surfaceECarouselIndex,
  surfaceEDateKey, surfaceEParseDateKey, surfaceEResizeByPixels, surfaceEWithinCalendarBounds,
} from '../src/shared/surface-e'

test('calendar date keys reject invalid days and survive leap-month navigation', () => {
  expect(surfaceEParseDateKey('2024-02-30')).toBeUndefined()
  expect(surfaceEParseDateKey('2024-02-29')?.getHours()).toBe(12)
  expect(surfaceEDateKey(surfaceEAddMonths(new Date(2024, 0, 31, 12), 1))).toBe('2024-02-29')
  expect(surfaceEDateKey(surfaceEAddMonths(new Date(2023, 0, 31, 12), 1))).toBe('2023-02-28')
})

test('calendar rows and keyboard movement follow the selected week start', () => {
  const month = new Date(2026, 8, 1, 12)
  const cells = surfaceECalendarCells(month, 1)
  expect(cells).toHaveLength(42)
  expect(cells[0]?.key).toBe('2026-08-31')
  expect(surfaceEDateKey(surfaceECalendarKeyDate(month, 'Home', 1)!)).toBe('2026-08-31')
  expect(surfaceEDateKey(surfaceECalendarKeyDate(month, 'End', 1)!)).toBe('2026-09-06')
  expect(surfaceEWithinCalendarBounds(month, new Date(2026, 8, 1), new Date(2026, 8, 30))).toBe(true)
  expect(surfaceEWithinCalendarBounds(new Date(2026, 7, 31), new Date(2026, 8, 1), undefined)).toBe(false)
})

test('resizing clamps percentages and carousel navigation honors loop mode', () => {
  expect(surfaceEResizeByPixels(50, 80, 400, 15, 85)).toBe(70)
  expect(surfaceEResizeByPixels(80, 80, 400, 15, 85)).toBe(85)
  expect(surfaceEResizeByPixels(50, Number.NaN, 400, 15, 85)).toBe(50)
  expect(surfaceECarouselIndex(2, 1, 3, true)).toBe(0)
  expect(surfaceECarouselIndex(2, 1, 3, false)).toBe(2)
})
