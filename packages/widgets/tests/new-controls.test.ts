import { expect, test } from 'bun:test'
import { nextEnabledTab, sliderPointerValue, snapValue } from '../src/shared/new-controls'

test('slider value clamps and snaps within a custom range', () => {
  expect(snapValue(-10, 20, 80, 10)).toBe(20)
  expect(snapValue(47, 20, 80, 10)).toBe(50)
  expect(snapValue(110, 20, 80, 10)).toBe(80)
  expect(snapValue(Number.NaN, 20, 80, 10)).toBe(20)
  expect(snapValue(5, 10, 10, 1)).toBe(10)
})

test('pointer positions account for the thumb and snap to steps', () => {
  expect(sliderPointerValue(8, 116, 20, 80, 10)).toBe(20)
  expect(sliderPointerValue(58, 116, 20, 80, 10)).toBe(50)
  expect(sliderPointerValue(108, 116, 20, 80, 10)).toBe(80)
  expect(sliderPointerValue(undefined, 116, 20, 80, 10)).toBeUndefined()
  expect(sliderPointerValue(8, 12, 20, 80, 10)).toBeUndefined()
})

test('tab navigation wraps and skips unavailable items', () => {
  const items = [
    { value: 'a', label: 'A', content: null },
    { value: 'b', label: 'B', content: null, disabled: true },
    { value: 'c', label: 'C', content: null },
  ]
  expect(nextEnabledTab(items, 0, 1)).toBe(2)
  expect(nextEnabledTab(items, 2, 1)).toBe(0)
  expect(nextEnabledTab(items, 0, -1)).toBe(2)
  expect(nextEnabledTab(items, -1, 1)).toBe(0)
  expect(nextEnabledTab(items, 0, -1)).toBe(2)
  expect(nextEnabledTab([{ value: 'x', label: 'X', content: null, disabled: true }], 0, 1)).toBe(-1)
})
