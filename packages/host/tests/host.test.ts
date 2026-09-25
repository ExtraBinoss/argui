import { expect, test } from 'bun:test'
import { NativeHost, type NativeBridge, type NativeContract, type NativeDelivery, type Operation } from '../src'

const contract: NativeContract = {
  abiHash: '42',
  natives: [
    { id: 3, name: 'Column', properties: [{ id: 6, name: 'gap', valueType: 'Float', readOnly: false }], events: [] },
    { id: 4, name: 'Text', properties: [{ id: 9, name: 'text', valueType: 'String', readOnly: false }], events: [] },
    { id: 14, name: 'FocusScope', properties: [], events: [{ id: 1, name: 'click' }] },
    { id: 15, name: 'Image', properties: [{ id: 2, name: 'source', valueType: 'Asset', readOnly: false }], events: [] },
    { id: 16, name: 'TextInput', properties: [{ id: 10, name: 'value', valueType: 'String', readOnly: false }], events: [{ id: 23, name: 'edit' }] },
  ],
}

function recorder(): { bridge: NativeBridge; batches: Operation[][]; deliver: (event: NativeDelivery) => void } {
  const batches: Operation[][] = []
  let sink: (event: NativeDelivery) => void = () => {}
  return {
    bridge: {
      contract: () => contract,
      commit: (batch) => { batches.push([...batch]) },
      subscribe: (next) => { sink = next; return () => { sink = () => {} } },
    },
    batches,
    deliver: (event) => sink(event),
  }
}

test('one Solid-style mount reaches the native bridge as an ordered batch', () => {
  const { bridge, batches } = recorder()
  const host = new NativeHost(bridge, '42')
  const root = host.createElement('column')
  const label = host.createTextNode('Hello')
  host.setProperty(root, 'gap', 12)
  host.insertNode(root, label)
  host.setRoot(root)
  expect(batches).toHaveLength(1)
  expect(batches[0]!.map((operation) => operation.kind)).toEqual([
    'create', 'create', 'setProperty', 'setProperty', 'insert', 'setRoot',
  ])
  expect(batches[0]![0]).toEqual({ kind: 'create', id: { slot: 1, generation: 1 }, nativeType: 3 })
  expect(batches[0]![2]).toEqual({ kind: 'setProperty', id: label.id, property: 9, value: { type: 'String', value: 'Hello' } })
})

test('event callbacks are released with removed native subtrees', () => {
  const { bridge, deliver } = recorder()
  const host = new NativeHost(bridge, '42')
  const root = host.createElement('column')
  const control = host.createElement('focusScope')
  let clicks = 0
  host.setProperty(control, 'onClick', () => { clicks++ })
  host.insertNode(root, control)
  host.setRoot(root)
  deliver({ node: control.id, callback: 1 })
  expect(clicks).toBe(1)
  host.removeNode(root, control)
  host.flush()
  deliver({ node: control.id, callback: 1 })
  expect(clicks).toBe(1)
})

test('native UTF-8 edits update the value cache while external resets still commit', () => {
  const { bridge, batches, deliver } = recorder()
  const host = new NativeHost(bridge, '42')
  const input = host.createElement('textInput')
  host.setProperty(input, 'value', 'aé😊b')
  let received = 0
  host.setProperty(input, 'onEdit', () => {
    received++
    host.setProperty(input, 'value', 'aé🙂b')
  })
  host.setRoot(input)
  deliver({ node: input.id, callback: 1, payload: { kind: 'edit', start: 3, end: 7, text: '🙂' } })
  host.flush()
  expect(received).toBe(1)
  expect(batches).toHaveLength(1)
  expect(input.values.get(10)).toEqual({ type: 'String', value: 'aé🙂b' })
  deliver({ node: input.id, callback: 1, payload: { kind: 'edit', start: 4, end: 7, text: 'x' } })
  expect(input.values.get(10)).toEqual({ type: 'String', value: 'aé🙂b' })
  host.setProperty(input, 'value', 'external reset')
  host.flush()
  expect(batches.at(-1)).toEqual([{ kind: 'setProperty', id: input.id, property: 10,
    value: { type: 'String', value: 'external reset' } }])
})

test('delayed controlled acknowledgements do not echo old million-character values', () => {
  const { bridge, batches, deliver } = recorder()
  const host = new NativeHost(bridge, '42')
  const input = host.createElement('textInput')
  const initial = 'a'.repeat(1_000_000)
  host.setProperty(input, 'value', initial)
  host.setProperty(input, 'onEdit', () => {})
  host.setRoot(input)

  for (const [offset, text] of Array.from('bcdefghi').entries()) {
    deliver({ node: input.id, callback: 1, payload: {
      kind: 'edit', start: 1_000_000 + offset, end: 1_000_000 + offset, text,
    } })
  }
  host.setProperty(input, 'value', initial + 'b')
  host.flush()
  expect(batches).toHaveLength(1)

  deliver({ node: input.id, callback: 1, payload: { kind: 'edit', start: 1_000_008, end: 1_000_008, text: 'j' } })
  host.setProperty(input, 'value', initial + 'bcdefghij')
  host.flush()
  expect(batches).toHaveLength(1)
  expect(input.values.get(10)).toEqual({ type: 'String', value: initial + 'bcdefghij' })
})

test('repeated schema lookups keep property and event mutations distinct', () => {
  const { bridge, batches, deliver } = recorder()
  const host = new NativeHost(bridge, '42')
  const root = host.createElement('column')
  const first = host.createElement('focusScope')
  const second = host.createElement('focusScope')
  host.setProperty(root, 'gap', 2)
  host.setProperty(root, 'gap', 3)
  let firstClicks = 0
  let secondClicks = 0
  host.setProperty(first, 'onClick', () => { firstClicks++ })
  host.setProperty(second, 'onClick', () => { secondClicks++ })
  host.insertNode(root, first)
  host.insertNode(root, second)
  host.setRoot(root)
  deliver({ node: first.id, callback: 1 })
  deliver({ node: second.id, callback: 2 })
  expect([firstClicks, secondClicks]).toEqual([1, 1])
  expect(batches[0]!.filter((operation) => operation.kind === 'setProperty')).toHaveLength(2)
  expect(batches[0]!.filter((operation) => operation.kind === 'setListener')).toHaveLength(2)
  expect(() => host.setProperty(first, 'gap', 1)).toThrow('has no property gap')
  expect(() => host.setProperty(first, 'gap', 1)).toThrow('has no property gap')
})

test('a schema hash mismatch rejects the bridge before any native mutation', () => {
  const { bridge, batches } = recorder()
  expect(() => new NativeHost(bridge, 'old')).toThrow('schema hash mismatch')
  expect(batches).toHaveLength(0)
})

test('reparenting keeps a node identity and emits one insert', () => {
  const { bridge, batches } = recorder()
  const host = new NativeHost(bridge, '42')
  const root = host.createElement('column')
  const left = host.createElement('column')
  const right = host.createElement('column')
  const child = host.createTextNode('Move me')
  host.insertNode(root, left)
  host.insertNode(root, right)
  host.insertNode(left, child)
  host.setRoot(root)
  host.insertNode(right, child)
  host.flush()
  expect(host.getParentNode(child)).toBe(right)
  expect(batches.at(-1)).toEqual([{ kind: 'insert', parent: right.id, child: child.id, before: null }])
})

test('disposing the presentation clears and removes the native root', () => {
  const { bridge, batches } = recorder()
  const host = new NativeHost(bridge, '42')
  const root = host.createElement('column')
  host.setRoot(root)
  host.dispose()
  expect(batches.at(-1)).toEqual([
    { kind: 'setRoot', id: null },
    { kind: 'remove', id: root.id },
  ])
})

test('an imported media reference becomes a typed native asset value', () => {
  const { bridge, batches } = recorder()
  const host = new NativeHost(bridge, '42')
  const image = host.createElement('image')
  host.setProperty(image, 'source', { kind: 'image', id: 182680722277923 })
  host.setRoot(image)
  expect(batches[0]![1]).toEqual({
    kind: 'setProperty', id: image.id, property: 2,
    value: { type: 'Asset', value: { kind: 'image', id: 182680722277923 } },
  })
  expect(() => host.setProperty(image, 'source', { kind: 'svg', id: Number.MAX_SAFE_INTEGER + 1 })).toThrow('Invalid Asset')
  host.setProperty(image, 'source', { kind: 'image', id: 182680722277923 })
  host.flush()
  expect(batches).toHaveLength(1)
})
