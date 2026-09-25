import { describe, expect, test } from 'bun:test'
import { InputEditController, applyInputEdit, inputEdit } from '../src/shared/input-edit'

describe('native input edit payloads', () => {
  test('splices UTF-8 ranges across JavaScript surrogate pairs', () => {
    const value = 'aé😊b'
    expect(applyInputEdit(value, { kind: 'edit', start: 3, end: 7, text: '🙂' })).toBe('aé🙂b')
    expect(applyInputEdit(value, { kind: 'edit', start: 4, end: 7, text: '' })).toBeUndefined()
    expect(inputEdit({ kind: 'edit', start: -1, end: 2, text: 'x' })).toBeUndefined()
  })

  test('retains edit order while controlled properties acknowledge older values', () => {
    const edits = new InputEditController('A')
    const emitted: string[] = []
    const onChange = (value: string) => emitted.push(value)
    expect(edits.apply({ kind: 'edit', start: 1, end: 1, text: 'é' }, 'A', onChange)).toBe(true)
    expect(edits.apply({ kind: 'edit', start: 3, end: 3, text: '😊' }, 'A', onChange)).toBe(true)
    expect(edits.apply({ kind: 'edit', start: 7, end: 7, text: '!' }, 'Aé', onChange)).toBe(true)
    expect(emitted).toEqual(['Aé', 'Aé😊', 'Aé😊!'])
    expect(edits.apply({ kind: 'edit', start: 5, end: 5, text: '?' }, 'reset', onChange)).toBe(true)
    expect(emitted.at(-1)).toBe('reset?')
  })

  test('applies a burst of middle edits to a million-character ASCII field', () => {
    const initial = 'a'.repeat(1_000_000)
    const edits = new InputEditController(initial)
    let value = ''
    let first = ''
    for (let offset = 0; offset < 8; offset++) {
      expect(edits.apply(
        { kind: 'edit', start: 500_000 + offset, end: 500_000 + offset, text: 'x' },
        initial,
        (next) => { value = next; if (offset === 0) first = next },
      )).toBe(true)
    }
    expect(value.slice(499_999, 500_009)).toBe('axxxxxxxxa')
    expect(edits.apply(
      { kind: 'edit', start: 500_008, end: 500_008, text: 'y' }, first,
      (next) => { value = next },
    )).toBe(true)
    expect(value.slice(499_999, 500_010)).toBe('axxxxxxxxya')
  })
})
