import { expect, test } from 'bun:test'
import { foundationJFilter, foundationJNext, type CommandEntry } from '../src/shared/foundation-j'

const entries: readonly CommandEntry[] = [
  { value: 'open', label: 'Open file', disabled: false },
  { value: 'close', label: 'Close file', disabled: true },
  { value: 'save', label: 'Save all', disabled: false },
]

test('command search matches labels and values without changing item order', () => {
  expect(foundationJFilter(entries, 'FILE').map((entry) => entry.value)).toEqual(['open', 'close'])
  expect(foundationJFilter(entries, 'save').map((entry) => entry.value)).toEqual(['save'])
})

test('command navigation skips disabled items and starts at the requested edge', () => {
  expect(foundationJNext(entries, undefined, 1)).toBe('open')
  expect(foundationJNext(entries, undefined, -1)).toBe('save')
  expect(foundationJNext(entries, 'open', 1)).toBe('save')
  expect(foundationJNext(entries, 'save', 1)).toBe('open')
  expect(foundationJNext(entries, 'open', -1)).toBe('save')
})
