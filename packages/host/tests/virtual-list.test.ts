import { expect, test } from 'bun:test'
import { boundedWindow, nativeWindowRange } from '../src/virtual-list'

test('native window ranges are accepted only within the declared collection', () => {
  expect(nativeWindowRange({ kind: 'window', start: 500, end: 523, offset: 20_000, viewportExtent: 400 }, 1_000))
    .toEqual({ start: 500, end: 523, offset: 20_000, viewportExtent: 400 })
  expect(nativeWindowRange({ start: 500, end: 1_001, offset: 0, viewportExtent: 400 }, 1_000)).toBeUndefined()
  expect(nativeWindowRange({ start: -1, end: 10, offset: 0, viewportExtent: 400 }, 1_000)).toBeUndefined()
})

test('a shrinking collection falls back to a bounded initial range', () => {
  expect(boundedWindow({ start: 500, end: 523, offset: 20_000, viewportExtent: 400 }, 6))
    .toEqual({ start: 0, end: 6, offset: 0, viewportExtent: 400 })
})
