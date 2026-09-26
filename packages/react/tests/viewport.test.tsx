/** @jsxImportSource @argui/react */
import { expect, test } from 'bun:test'
import { NativeHost, type NativeBridge, type Operation } from '@argui/host'
import contract from '../../host/src/contract.generated.json' with { type: 'json' }
import { createRoot } from '../src'

test('React retains a native viewport and changes only its controls', () => {
  const batches: Operation[][] = []
  const bridge: NativeBridge = {
    contract: () => contract,
    commit: (batch) => { batches.push([...batch]) },
    subscribe: () => () => {},
  }
  const host = new NativeHost(bridge, contract.abiHash)
  const root = createRoot(host)
  root.render(<gpuCanvas canvasId={1} revision={0} width={320} height={180} alt="Preview" />)
  const node = root.nativeRoot().children[0]!
  expect(node.type.name).toBe('GpuCanvas')
  const initial = batches.flat().filter((operation) => operation.kind === 'setProperty')
  expect(initial.some((operation) => operation.value?.type === 'Int' && operation.value.value === 1)).toBe(true)
  const before = batches.length
  root.render(<gpuCanvas canvasId={1} revision={1} width={320} height={180} alt="Preview" />)
  expect(root.nativeRoot().children[0]).toBe(node)
  expect(batches.length).toBe(before + 1)
  expect(batches.at(-1)).toEqual([{
    kind: 'setProperty', id: node.id, property: contract.natives.find((native) => native.name === 'GpuCanvas')!.properties.find((property) => property.name === 'revision')!.id,
    value: { type: 'Int', value: 1 },
  }])
  root.unmount()
})
