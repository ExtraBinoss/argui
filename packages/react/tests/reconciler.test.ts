import { expect, test } from 'bun:test'
import { createElement } from 'react'
import { NativeHost, type NativeBridge, type NativeContract, type NativeDelivery, type Operation } from '@argui/host'
import { createRoot } from '../src'

const contract: NativeContract = {
  abiHash: 'react-fixture',
  natives: [
    { id: 1, name: 'Container', properties: [], events: [] },
    { id: 3, name: 'Column', properties: [{ id: 6, name: 'gap', valueType: 'Float', readOnly: false }], events: [] },
    { id: 4, name: 'Text', properties: [{ id: 9, name: 'text', valueType: 'String', readOnly: false }], events: [] },
    { id: 14, name: 'FocusScope', properties: [], events: [{ id: 1, name: 'click' }] },
  ],
}

function recorder(): { host: NativeHost; batches: Operation[][]; deliver: (event: NativeDelivery) => void } {
  const batches: Operation[][] = []
  let sink: (event: NativeDelivery) => void = () => {}
  const bridge: NativeBridge = {
    contract: () => contract,
    commit: (batch) => { batches.push([...batch]) },
    subscribe: (next) => { sink = next; return () => { sink = () => {} } },
  }
  return { host: new NativeHost(bridge, contract.abiHash), batches, deliver: (event) => sink(event) }
}

test('React mount emits the same native primitive operations as a Solid-style mount', () => {
  const react = recorder()
  const root = createRoot(react.host)
  root.render(createElement('column', { gap: 12 }, 'Hello'))

  const solid = recorder()
  const solidRoot = solid.host.createElement('container')
  solid.host.setRoot(solidRoot)
  const column = solid.host.createElement('column')
  solid.host.setProperty(column, 'gap', 12)
  const text = solid.host.createTextNode('Hello')
  solid.host.insertNode(column, text)
  solid.host.insertNode(solidRoot, column)
  solid.host.flush()

  expect(react.batches).toEqual(solid.batches)
  root.unmount()
})

test('React updates only changed properties and retains native identities on reorder', () => {
  const { host, batches } = recorder()
  const root = createRoot(host)
  const labels = (order: string[], gap: number) => createElement(
    'column', { gap }, ...order.map((value) => createElement('text', { key: value, text: value })),
  )
  root.render(labels(['A', 'B'], 12))
  const mounted = root.nativeRoot().children[0]!
  const first = mounted.children[0]!
  const second = mounted.children[1]!
  root.render(labels(['B', 'A'], 16))
  expect(root.nativeRoot().children[0]).toBe(mounted)
  expect(mounted.children).toEqual([second, first])
  expect(batches.at(-1)).toEqual([
    { kind: 'setProperty', id: mounted.id, property: 6, value: { type: 'Float', value: 16 } },
    { kind: 'insert', parent: mounted.id, child: first.id, before: null },
  ])
  const count = batches.length
  root.render(labels(['B', 'A'], 16))
  expect(batches).toHaveLength(count)
  root.unmount()
})

test('React unmount removes listeners from the shared host', () => {
  const { host, batches, deliver } = recorder()
  const root = createRoot(host)
  let clicks = 0
  root.render(createElement('focusScope', { onClick: () => { clicks++ } }))
  const control = root.nativeRoot().children[0]!
  const callback = control.listeners.get(1)!
  deliver({ node: control.id, callback })
  expect(clicks).toBe(1)
  root.unmount()
  deliver({ node: control.id, callback })
  expect(clicks).toBe(1)
  expect(batches.some((batch) => batch.some((operation) => operation.kind === 'remove' && operation.id === control.id))).toBe(true)
})

test('a failed React render never creates native primitives', () => {
  const { host, batches } = recorder()
  const root = createRoot(host)
  function Broken(): never { throw new Error('aborted render') }
  expect(() => root.render(createElement(Broken))).toThrow('aborted render')
  expect(batches).toHaveLength(1)
  expect(root.nativeRoot().children).toHaveLength(0)
  root.unmount()
})

test('React replaces event callbacks without retaining stale handlers', () => {
  const { host, deliver } = recorder()
  const root = createRoot(host)
  let first = 0
  let second = 0
  root.render(createElement('focusScope', { onClick: () => { first++ } }))
  const control = root.nativeRoot().children[0]!
  const oldCallback = control.listeners.get(1)!
  root.render(createElement('focusScope', { onClick: () => { second++ } }))
  const newCallback = control.listeners.get(1)!
  expect(newCallback).not.toBe(oldCallback)
  deliver({ node: control.id, callback: oldCallback })
  deliver({ node: control.id, callback: newCallback })
  expect(first).toBe(0)
  expect(second).toBe(1)
  root.unmount()
})
