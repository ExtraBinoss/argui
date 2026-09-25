import { expect, test } from 'bun:test'
import { nextSelectIndex, resolveSelectOptions } from '../src/shared/select-options'

test('grouped select navigation skips disabled values and wraps', () => {
  const options = resolveSelectOptions([
    { value: 'stable', group: 'Recommended' },
    { value: 'retired', group: 'Experimental', disabled: true },
    'nightly',
  ])
  expect(options[0]).toEqual({ value: 'stable', label: 'stable', group: 'Recommended', disabled: false })
  expect(nextSelectIndex(options, 0, 1)).toBe(2)
  expect(nextSelectIndex(options, 2, 1)).toBe(0)
  expect(nextSelectIndex(options, 0, -1)).toBe(2)
  expect(nextSelectIndex(options.slice(1, 2), -1, 1)).toBe(-1)
})
