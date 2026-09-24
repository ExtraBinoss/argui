/** @jsxImportSource @argui/react */
import { expect, test } from 'bun:test'
import { NativeHost, type NativeBridge, type NativeDelivery, type Operation } from '@argui/host'
import contract from '../../host/src/contract.generated.json' with { type: 'json' }
import { createRoot, VirtualList } from '../src'

test('React VirtualList mounts only native-requested rows in a million-item collection', async () => {
  const batches: Operation[][] = []
  let deliver: (event: NativeDelivery) => void = () => {}
  const bridge: NativeBridge = {
    contract: () => contract,
    commit: (batch) => { batches.push([...batch]) },
    subscribe: (callback) => { deliver = callback; return () => {} },
  }
  const host = new NativeHost(bridge, contract.abiHash)
  const root = createRoot(host)
  root.render(<VirtualList count={1_000_000} estimate={40} variable={false} dataVersion={3}
    renderItem={(index) => <text text={`Item ${index}`} />} />)
  let list = root.nativeRoot().children[0]!
  expect(list.type.name).toBe('VirtualWindow')
  expect(list.children).toHaveLength(12)
  const dataVersion = list.type.properties.find((property) => property.name === '__data_version')!
  expect(list.values.get(dataVersion.id)?.value).toBe(3)
  expect(list.listeners.has(list.type.events.find((event) => event.name === 'scroll')!.id)).toBe(false)
  const event = list.type.events.find((entry) => entry.name === 'window')!
  deliver({ node: list.id, callback: list.listeners.get(event.id)!, payload: {
    kind: 'window', start: 500_000, end: 500_018, offset: 20_000_000, viewportExtent: 320,
  } })
  await new Promise((resolve) => setTimeout(resolve, 0))
  list = root.nativeRoot().children[0]!
  expect(list.children).toHaveLength(18)
  expect(list.children[0]?.values.get(1)?.value).toContain('500000')
  expect(batches.flat().filter((operation) => operation.kind === 'create').length).toBeLessThan(100)
  root.unmount()
})
