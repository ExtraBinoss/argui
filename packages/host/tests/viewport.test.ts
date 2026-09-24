import { expect, test } from 'bun:test'
import { NativeHost, type NativeBridge, type Operation } from '../src'
import contract from '../src/contract.generated.json' with { type: 'json' }

test('Solid-style viewport controls cross the native host without a frame payload', () => {
  const batches: Operation[][] = []
  const bridge: NativeBridge = {
    contract: () => contract,
    commit: (batch) => { batches.push([...batch]) },
    subscribe: () => () => {},
  }
  const host = new NativeHost(bridge, contract.abiHash)
  const viewport = host.createElement('gpuCanvas')
  host.setProperty(viewport, 'canvas_id', 1)
  host.setProperty(viewport, 'resolution_scale', 1.5)
  host.setProperty(viewport, 'alt', 'Preview')
  host.setRoot(viewport)
  const controls = batches.flat().filter((operation) => operation.kind === 'setProperty')
  expect(controls.map((operation) => operation.value?.type)).toEqual(['Int', 'Float', 'String'])
  expect(controls.every((operation) => !('pixels' in operation))).toBe(true)
  expect(() => host.setProperty(viewport, 'canvas_id', 1.5)).toThrow('Invalid Int')
  expect(() => host.setProperty(viewport, 'canvas_id', Number.MAX_SAFE_INTEGER + 1)).toThrow('Invalid Int')
})
