import { expect, test } from 'bun:test'
import { createThemeRuntime, mergeThemeOverrides, type NativeBridge, type ThemeDefinition, type ThemeWireSnapshot } from '../src'

interface Colors { foreground: string; spacing: number }

test('local theme overrides inherit untouched tokens and validate token names and types', () => {
  const base: Colors = { foreground: '#ffffff', spacing: 8 }
  const local = mergeThemeOverrides<Colors>(base, { foreground: '#000000' })
  expect(local).toEqual({ foreground: '#000000', spacing: 8 })
  expect(base.foreground).toBe('#ffffff')
  expect(mergeThemeOverrides<Colors>({ ...base, spacing: 10 }, { foreground: '#000000' }).spacing).toBe(10)
  expect(() => mergeThemeOverrides(base, { unknown: 'x' } as Partial<Colors>)).toThrow('Unknown theme token')
  expect(() => mergeThemeOverrides(base, { spacing: 'large' } as unknown as Partial<Colors>)).toThrow('Invalid theme token type')
})

const definition: ThemeDefinition<Colors> = {
  tokens: {
    foreground: { type: 'Color', default: '#ffffff', impact: 'Paint' },
    spacing: { type: 'Length', default: 8, impact: 'Layout' },
  },
  variants: { dark: { foreground: '#000000' } },
  initialVariant: 'dark',
}

function snapshot(revision: number, foreground: string, spacing: number): ThemeWireSnapshot {
  return {
    revision, variant: 'dark', resolvedVariant: 'dark', systemScheme: 'dark',
    values: { foreground, spacing },
    tokenRevisions: { foreground: revision, spacing: spacing === 8 ? 0 : revision },
    change: { tokens: spacing === 8 ? ['foreground'] : ['foreground', 'spacing'], impact: spacing === 8 ? 'Paint' : 'Layout' },
  }
}

test('theme runtime forwards one atomic patch and publishes a coherent host revision once', () => {
  const updates: unknown[] = []
  const disposed: number[] = []
  let deliver: ((snapshot: ThemeWireSnapshot) => void) | undefined
  const bridge: NativeBridge = {
    contract: () => ({ abiHash: '', natives: [] }), commit: () => {}, subscribe: () => () => {},
    theme: {
      create(received) {
        expect(received).toBe(definition)
        return { id: 7, snapshot: snapshot(0, '#000000', 8) }
      },
      update(id, patch) {
        expect(id).toBe(7)
        updates.push(patch)
        const next = snapshot(1, '#ffffff', 12)
        deliver?.(next)
        return next
      },
      subscribe(id, callback) {
        expect(id).toBe(7)
        deliver = callback
        return () => { deliver = undefined }
      },
      dispose(id) { disposed.push(id) },
    },
  }
  const runtime = createThemeRuntime(bridge, definition)
  const notifications: string[][] = []
  const foregroundOnly: string[][] = []
  runtime.subscribe((current, change) => {
    expect(current.values.spacing).toBe(12)
    notifications.push([...change.tokens])
  })
  runtime.subscribe((_current, change) => foregroundOnly.push([...change.tokens]), ['foreground'])
  const patch = { variant: 'dark', overrides: { foreground: '#ffffff', spacing: 12 } }
  expect(runtime.update(patch).values.foreground).toBe('#ffffff')
  expect(updates).toEqual([patch])
  expect(notifications).toEqual([['foreground', 'spacing']])
  expect(foregroundOnly).toEqual([['foreground', 'spacing']])
  runtime.dispose()
  expect(disposed).toEqual([7])
  expect(() => runtime.update({})).toThrow('disposed')
})

test('theme runtime requires the host theme engine', () => {
  const bridge: NativeBridge = {
    contract: () => ({ abiHash: '', natives: [] }), commit: () => {}, subscribe: () => () => {},
  }
  expect(() => createThemeRuntime(bridge, definition)).toThrow('does not expose')
})

test('system appearance deliveries update only subscribers of affected tokens', () => {
  let deliver: ((snapshot: ThemeWireSnapshot) => void) | undefined
  const bridge: NativeBridge = {
    contract: () => ({ abiHash: '', natives: [] }), commit: () => {}, subscribe: () => () => {},
    theme: {
      create: () => ({ id: 1, snapshot: snapshot(0, '#000000', 8) }),
      update: () => snapshot(0, '#000000', 8),
      subscribe: (_id, callback) => { deliver = callback; return () => { deliver = undefined } },
      dispose: () => {},
    },
  }
  const runtime = createThemeRuntime(bridge, definition)
  let colors = 0
  let layout = 0
  runtime.subscribe(() => { colors++ }, ['foreground'])
  runtime.subscribe(() => { layout++ }, ['spacing'])
  deliver?.({
    ...snapshot(1, '#ffffff', 8),
    systemScheme: 'light',
    change: { tokens: ['foreground'], impact: 'Paint' },
  })
  deliver?.(snapshot(1, '#ffffff', 8))
  expect(runtime.snapshot().systemScheme).toBe('light')
  expect(colors).toBe(1)
  expect(layout).toBe(0)
  runtime.dispose()
})
