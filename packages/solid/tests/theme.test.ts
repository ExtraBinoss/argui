import { expect, test } from 'bun:test'
import { createRoot } from 'solid-js'
import { createThemeRuntime, type NativeBridge, type ThemeWireSnapshot } from '@argui/host'
import { useThemeSnapshot } from '../src/theme'

test('Solid theme accessor follows a host revision and stops at owner cleanup', () => {
  let deliver: ((snapshot: ThemeWireSnapshot) => void) | undefined
  let subscriptions = 0
  const bridge: NativeBridge = {
    contract: () => ({ abiHash: '', natives: [] }), commit: () => {}, subscribe: () => () => {},
    theme: {
      create: () => ({ id: 1, snapshot: {
        revision: 0, variant: 'light', resolvedVariant: 'light', systemScheme: 'light',
        values: { background: '#ffffff' }, tokenRevisions: { background: 0 },
      } }),
      update: () => { throw new Error('not used') },
      subscribe: (_id, callback) => { deliver = callback; subscriptions++; return () => { deliver = undefined; subscriptions-- } },
      dispose: () => {},
    },
  }
  const runtime = createThemeRuntime(bridge, {
    tokens: { background: { type: 'Color', default: '#ffffff' } },
  })
  let read!: ReturnType<typeof useThemeSnapshot<{ background: string }>>
  const dispose = createRoot((cleanup) => {
    read = useThemeSnapshot(runtime)
    return cleanup
  })
  expect(read().values.background).toBe('#ffffff')
  deliver?.({
    revision: 1, variant: 'dark', resolvedVariant: 'dark', systemScheme: 'light',
    values: { background: '#000000' }, tokenRevisions: { background: 1 },
    change: { tokens: ['background'], impact: 'Paint' },
  })
  expect(read().values.background).toBe('#000000')
  dispose()
  expect(subscriptions).toBe(1)
  runtime.dispose()
  expect(subscriptions).toBe(0)
})
