import { expect, test } from 'bun:test'
import { InputEditController } from '../src/shared/input-edit'
import { InputOffsetIndex } from '../src/shared/input-offset-index'

const bytes = (value: string) => new TextEncoder().encode(value).length

test('sparse UTF-8 index follows insertions, deletions, emoji and distant edits', () => {
  let value = 'مرحبا👩‍🚀e\u0301 ABC'.repeat(200)
  const index = new InputOffsetIndex(value)
  for (let key = 0; key < 100; key++) {
    const point = key % 2 ? [...value].length - 3 : 5 + key
    const units = [...value]
    const start = units.slice(0, point).join('').length
    const end = key % 3 ? start : start + units[point].length
    const edit = { kind: 'edit' as const, start: bytes(value.slice(0, start)), end: bytes(value.slice(0, end)), text: key % 2 ? '🦊' : 'x' }
    expect(index.at(value, edit.start)).toBe(start)
    expect(index.at(value, edit.end)).toBe(end)
    index.update(edit, start, end)
    value = value.slice(0, start) + edit.text + value.slice(end)
    expect(index.byteLength).toBe(bytes(value))
    expect(index.at(value, edit.start + bytes(edit.text))).toBe(start + edit.text.length)
  }
  expect(index.at(value, 1)).toBeUndefined()
})

test('controlled edits survive a burst and partial acknowledgements', () => {
  const authored = 'a'.repeat(1_000_000)
  const controller = new InputEditController(authored)
  const versions: string[] = []
  for (let key = 0; key < 20; key++) {
    expect(controller.apply({ kind: 'edit', start: authored.length + key, end: authored.length + key, text: 'x' }, authored, (value) => versions.push(value))).toBe(true)
  }
  let latest = ''
  expect(controller.apply({ kind: 'edit', start: authored.length + 20, end: authored.length + 20, text: 'x' }, versions[2], (value) => { latest = value })).toBe(true)
  expect(latest).toBe(authored + 'x'.repeat(21))
  expect(controller.apply({ kind: 'edit', start: authored.length + 21, end: authored.length + 21, text: 'x' }, latest, (value) => { latest = value })).toBe(true)
  expect(latest).toBe(authored + 'x'.repeat(22))
})

test('controlled burst beyond 128 edits accepts late partial acknowledgement', () => {
  const authored = 'a'.repeat(1_000_000)
  const controller = new InputEditController(authored)
  let latest = authored
  for (let key = 0; key < 300; key++) {
    expect(controller.apply({ kind: 'edit', start: authored.length + key, end: authored.length + key, text: 'x' }, authored, (value) => { latest = value })).toBe(true)
  }
  expect(controller.pendingCount).toBe(300)
  expect(controller.apply({ kind: 'edit', start: authored.length + 300, end: authored.length + 300, text: 'x' }, authored + 'x'.repeat(150), (value) => { latest = value })).toBe(true)
  expect(latest).toBe(authored + 'x'.repeat(301))
  expect(controller.apply({ kind: 'edit', start: authored.length + 301, end: authored.length + 301, text: 'x' }, latest, (value) => { latest = value })).toBe(true)
  expect(latest).toBe(authored + 'x'.repeat(302))
})

test('controlled Unicode middle burst keeps a late partial echo and later edits', () => {
  const seed = 'م'.repeat(1_000_000)
  const start = new TextEncoder().encode(seed.slice(0, 500_000)).length
  const controller = new InputEditController(seed)
  let latest = seed
  for (let key = 0; key < 300; key++) {
    expect(controller.apply({ kind: 'edit', start: start + key, end: start + key, text: 'x' }, seed, (value) => { latest = value })).toBe(true)
  }
  const partial = seed.slice(0, 500_000) + 'x'.repeat(150) + seed.slice(500_000)
  expect(controller.apply({ kind: 'edit', start: start + 300, end: start + 300, text: 'x' }, partial, (value) => { latest = value })).toBe(true)
  expect(latest).toBe(seed.slice(0, 500_000) + 'x'.repeat(301) + seed.slice(500_000))
})

test('controlled same-length replacements accept a late partial echo', () => {
  const seed = 'a'.repeat(1_000_000)
  const controller = new InputEditController(seed)
  let latest = seed
  let partial = ''
  for (let key = 0; key < 300; key++) {
    const offset = key % 2 === 0 ? 250_000 + Math.floor(key / 2) : 750_000 + Math.floor(key / 2)
    expect(controller.apply({ kind: 'edit', start: offset, end: offset + 1, text: 'x' }, seed, (value) => { latest = value })).toBe(true)
    if (key === 149) partial = latest
  }
  expect(controller.apply({ kind: 'edit', start: 250_000, end: 250_001, text: 'y' }, partial, (value) => { latest = value })).toBe(true)
  expect(latest.length).toBe(seed.length)
  expect(latest.slice(250_000, 250_001)).toBe('y')
  expect(controller.pendingCount).toBe(151)
})

test('controlled noncontiguous Unicode edits replay from a checkpoint', () => {
  const seed = 'م'.repeat(100_000)
  const controller = new InputEditController(seed)
  let latest = seed
  let partial = ''
  for (let key = 0; key < 600; key++) {
    const before = Math.ceil(key / 2)
    const offset = key % 2 === 0 ? 50_000 + before : 150_000 + key
    expect(controller.apply({ kind: 'edit', start: offset, end: offset, text: 'x' }, seed, (value) => { latest = value })).toBe(true)
    if (key === 299) partial = latest
  }
  expect(controller.apply({ kind: 'edit', start: 50_300, end: 50_300, text: 'y' }, partial, (value) => { latest = value })).toBe(true)
  expect(latest.length).toBe(seed.length + 601)
  expect(controller.pendingCount).toBe(301)
})

test('Unicode editor rejects offsets inside a code point and keeps later edits', () => {
  const controller = new InputEditController('a💡z')
  let value = ''
  expect(controller.apply({ kind: 'edit', start: 2, end: 2, text: 'x' }, 'a💡z', (next) => { value = next })).toBe(false)
  expect(controller.apply({ kind: 'edit', start: 5, end: 5, text: 'x' }, 'a💡z', (next) => { value = next })).toBe(true)
  expect(value).toBe('a💡xz')
})

test('long middle burst keeps UTF offsets fast and checkpoint memory bounded', () => {
  let value = 'مرحبا'.repeat(2_000)
  const index = new InputOffsetIndex(value)
  const initial = index.checkpointCount
  const middle = bytes(value) / 2
  for (let key = 0; key < 2_048; key++) {
    const byte = middle + key
    const position = index.at(value, byte)
    expect(position).toBeDefined()
    index.update({ kind: 'edit', start: byte, end: byte, text: 'x' }, position!, position!)
    value = value.slice(0, position) + 'x' + value.slice(position)
  }
  expect(index.at(value, middle + 2_048)).toBe(5_000 + 2_048)
  expect(index.checkpointCount).toBeLessThan(initial + 10)
})

test('uncontrolled callback retains no acknowledgements through 10000 keys and a parent rerender', () => {
  const initial = 'seed'
  const controller = new InputEditController(initial, false)
  let latest = initial
  for (let key = 0; key < 10_000; key++) {
    const offset = initial.length + key
    expect(controller.apply({ kind: 'edit', start: offset, end: offset, text: 'x' }, initial, (value) => { latest = value }, false)).toBe(true)
  }
  expect(controller.pendingCount).toBe(0)
  expect(latest).toBe(initial + 'x'.repeat(10_000))
  expect(controller.apply({ kind: 'edit', start: latest.length, end: latest.length, text: 'y' }, initial, (value) => { latest = value }, false)).toBe(true)
  expect(latest).toBe(initial + 'x'.repeat(10_000) + 'y')
  expect(controller.pendingCount).toBe(0)
})
