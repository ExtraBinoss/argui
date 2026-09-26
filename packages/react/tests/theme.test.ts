import { expect, test } from 'bun:test'
import { act, createElement } from 'react'
import { createThemeRuntime, NativeHost, type NativeBridge, type ThemeWireSnapshot } from '@argui/host'
import { createRoot, ThemeProvider, useTheme } from '../src'

test('React theme hook renders a delivered host revision', () => {
  const actEnvironment = globalThis as typeof globalThis & { IS_REACT_ACT_ENVIRONMENT?: boolean }
  const previousActEnvironment = actEnvironment.IS_REACT_ACT_ENVIRONMENT
  actEnvironment.IS_REACT_ACT_ENVIRONMENT = true
  let deliver: ((snapshot: ThemeWireSnapshot) => void) | undefined
  const bridge: NativeBridge = {
    contract: () => ({ abiHash: 'theme-test', natives: [
      { id: 1, name: 'Container', properties: [], events: [] },
      { id: 2, name: 'Text', properties: [{ id: 3, name: 'text', valueType: 'String', readOnly: false }], events: [] },
    ] }),
    commit: () => {}, subscribe: () => () => {},
    theme: {
      create: () => ({ id: 1, snapshot: {
        revision: 0, variant: 'light', resolvedVariant: 'light', systemScheme: 'light',
        values: { foreground: '#ffffff' }, tokenRevisions: { foreground: 0 },
      } }),
      update: () => { throw new Error('not used') },
      subscribe: (_id, callback) => { deliver = callback; return () => { deliver = undefined } },
      dispose: () => {},
    },
  }
  const runtime = createThemeRuntime<{ foreground: string }>(bridge, {
    tokens: { foreground: { type: 'Color', default: '#ffffff' } },
  })
  const root = createRoot(new NativeHost(bridge, 'theme-test'))
  function ThemedText() { return createElement('text', { text: useTheme<{ foreground: string }>().foreground }) }
  act(() => root.render(createElement(ThemeProvider<{ foreground: string }>, {
    runtime, children: createElement(ThemedText),
  })))
  expect(root.nativeRoot().children[0]?.values.get(3)?.value).toBe('#ffffff')
  act(() => deliver?.({
    revision: 1, variant: 'dark', resolvedVariant: 'dark', systemScheme: 'light',
    values: { foreground: '#000000' }, tokenRevisions: { foreground: 1 },
    change: { tokens: ['foreground'], impact: 'Paint' },
  }))
  expect(root.nativeRoot().children[0]?.values.get(3)?.value).toBe('#000000')
  act(() => root.unmount())
  runtime.dispose()
  actEnvironment.IS_REACT_ACT_ENVIRONMENT = previousActEnvironment
})
