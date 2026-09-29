import { expect, test } from 'bun:test'
import { NativeHost, type NativeBridge, type NativeContract, type Operation } from '@argui/host'
import { createSignal } from 'solid-js'
import { createElement, insert, render, useNativeHost } from '../src/renderer'

const contract: NativeContract = {
  abiHash: 'solid-text', natives: [
    { id: 1, name: 'Container', properties: [{ id: 1, name: 'id', valueType: 'String', readOnly: false }], events: [] },
    { id: 2, name: 'Text', properties: [
      { id: 1, name: 'id', valueType: 'String', readOnly: false },
      { id: 2, name: 'text', valueType: 'String', readOnly: false },
    ], events: [] },
  ],
}

test('Solid text children update one native Text value through the renderer', () => {
  const batches: Operation[][] = []
  const bridge: NativeBridge = {
    contract: () => contract,
    commit: (operations) => { batches.push([...operations]) },
    subscribe: () => () => {},
  }
  const host = new NativeHost(bridge, contract.abiHash)
  useNativeHost(host)
  const root = host.createElement('container')
  host.setRoot(root)
  let setLabel!: (value: string) => void
  const dispose = render(() => {
    const [label, update] = createSignal('Bonjour')
    setLabel = update
    const text = createElement('text')
    insert(text, label)
    return text
  }, root)
  host.flush()
  expect(root.children).toHaveLength(1)
  expect(root.children[0]?.values.get(2)?.value).toBe('Bonjour')
  setLabel('Salut')
  host.flush()
  expect(root.children[0]?.values.get(2)?.value).toBe('Salut')
  expect(batches.flat().filter((operation) => operation.kind === 'create' && operation.nativeType === 2)).toHaveLength(1)
  dispose()
})

test('numeric children beside a unit remain valid native strings on mount and update', () => {
  const batches: Operation[][] = []
  const bridge: NativeBridge = {
    contract: () => contract,
    commit: operations => { batches.push([...operations]) },
    subscribe: () => () => {},
  }
  const host = new NativeHost(bridge, contract.abiHash)
  useNativeHost(host)
  const root = host.createElement('container')
  let setSeconds!: (value: number) => void
  const dispose = render(() => {
    const [seconds, update] = createSignal(3)
    setSeconds = update
    const row = createElement('container')
    insert(row, () => [seconds(), ' s'])
    return row
  }, root)
  host.setRoot(root)
  for (const seconds of [3, 10, 0]) {
    setSeconds(seconds)
    host.flush()
    expect(root.children[0]?.children[0]?.values.get(2)?.value).toBe(String(seconds))
    for (const operation of batches.flat()) {
      if (operation.kind === 'setProperty' && operation.value?.type === 'String') {
        expect(typeof operation.value.value).toBe('string')
      }
    }
  }
  dispose(); host.dispose()
})
