/** @jsxImportSource @argui/react */
import { expect, test } from 'bun:test'
import { act } from 'react'
import { createRoot, ThemeProvider } from '@argui/react'
import type { NativeNode } from '@argui/host'
import { Select, type SelectProps } from '../src/react'
import { mountedRole, mountedRoles, nativeControlsFixture } from './fixtures/native-controls'

const options = [{ value: 'rust', label: 'Rust' }, { value: 'go', label: 'Go' }]
const devices = Array.from({ length: 10 }, (_, index) => ({
  value: `device/${index}`, label: `Device ${index + 1}`,
}))

/** Mounts a compact menu and keeps React's test act environment scoped to it. */
function fixture(props: Partial<Pick<SelectProps, 'allowClear' | 'options' | 'defaultValue'>> = {}) {
  const environment = globalThis as typeof globalThis & { IS_REACT_ACT_ENVIRONMENT?: boolean }
  const previous = environment.IS_REACT_ACT_ENVIRONMENT
  environment.IS_REACT_ACT_ENVIRONMENT = true
  const { host, runtime, deliver } = nativeControlsFixture()
  const root = createRoot(host)
  const changes: string[] = []
  act(() => root.render(<ThemeProvider runtime={runtime}>
    <Select id="language" label="Language" variant="shadcn" options={props.options ?? options}
      defaultValue={props.defaultValue ?? 'rust'} defaultOpen allowClear={props.allowClear}
      onValueChange={value => changes.push(value)} />
  </ThemeProvider>))
  return { root, changes, deliver, dispose: () => {
    act(() => root.unmount()); runtime.dispose(); environment.IS_REACT_ACT_ENVIRONMENT = previous
  } }
}

/** Finds an option by its accessible name, independently of its tree layout. */
function option(node: NativeNode, label: string): NativeNode | undefined {
  const property = node.type.properties.find(item => item.name === 'accessibleName')
  if (property && node.values.get(property.id)?.value === label) return node
  for (const child of node.children) { const found = option(child, label); if (found) return found }
  return undefined
}

/** Reads a property authored on a particular native node. */
function propertyValue(node: NativeNode, name: string): unknown {
  const property = node.type.properties.find(item => item.name === name)
  return property ? node.values.get(property.id)?.value : undefined
}

/** Finds the option's painted surface without depending on pointer wrapper layout. */
function optionSurface(node: NativeNode): NativeNode | undefined {
  if (propertyValue(node, 'hoverBackground') !== undefined) return node
  for (const child of node.children) {
    const found = optionSurface(child)
    if (found) return found
  }
  return undefined
}

/** Collects live native listeners for a pointer event within one option. */
function pointerListeners(node: NativeNode, name: string): NativeNode[] {
  const event = node.type.events.find(item => item.name === name)
  const listeners = event && node.listeners.has(event.id) ? [node] : []
  for (const child of node.children) listeners.push(...pointerListeners(child, name))
  return listeners
}

/** Delivers one existing native callback and flushes its React state changes. */
function emit(view: ReturnType<typeof fixture>, node: NativeNode, name: string, payload: unknown): void {
  const event = node.type.events.find(item => item.name === name)
  const callback = event && node.listeners.get(event.id)
  if (callback === undefined) throw new Error(`${node.type.name} has no ${name} listener`)
  act(() => view.deliver({ node: node.id, callback, payload }))
}

/** Checks that the popup receives a focus target and an explicit upward placement. */
function expectAlignedMenu(root: NativeNode, initialFocus: string): void {
  const popup = mountedRole(root, 'listBox')!
  expect(popup).toBeDefined()
  expect(propertyValue(popup, 'initialFocus')).toBe(initialFocus)
  const offset = propertyValue(popup, 'placementOffset')
  expect(typeof offset).toBe('number')
  expect(Number.isFinite(offset as number)).toBe(true)
  expect(offset as number).toBeLessThan(0)
}

test('compact selects keep their clearable placeholder by default', () => {
  const view = fixture()
  expect(mountedRoles(view.root.nativeRoot()).filter(role => role === 'option')).toHaveLength(3)
  const placeholder = option(view.root.nativeRoot(), 'Choose an option')!
  const click = placeholder.type.events.find(event => event.name === 'click')!
  act(() => view.deliver({ node: placeholder.id, callback: placeholder.listeners.get(click.id)!, payload: { kind: 'click' } }))
  expect(view.changes).toEqual([''])
  view.dispose()
})

test('required compact selects expose only valid options and still select normally', () => {
  const view = fixture({ allowClear: false })
  expect(mountedRoles(view.root.nativeRoot()).filter(role => role === 'option')).toHaveLength(2)
  expect(option(view.root.nativeRoot(), 'Choose an option')).toBeUndefined()
  const go = option(view.root.nativeRoot(), 'Go')!
  const click = go.type.events.find(event => event.name === 'click')!
  act(() => view.deliver({ node: go.id, callback: go.listeners.get(click.id)!, payload: { kind: 'click' } }))
  expect(view.changes).toEqual(['go'])
  expect(mountedRole(view.root.nativeRoot(), 'comboBox')).toBeDefined()
  view.dispose()
})

for (const allowClear of [true, false]) {
  test(`long ${allowClear ? 'clearable' : 'required'} compact menus focus their selected lower option`, () => {
    const view = fixture({ allowClear, options: devices, defaultValue: 'device/9' })
    try {
      const root = view.root.nativeRoot()
      expect(mountedRoles(root).filter(role => role === 'option')).toHaveLength(allowClear ? 11 : 10)
      const selected = option(root, 'Device 10')!
      expect(propertyValue(selected, 'id')).toBe('language-option-device%2F9')
      expect(propertyValue(selected, 'selected')).toBe(true)
      expectAlignedMenu(root, 'language-option-device%2F9')
    } finally {
      view.dispose()
    }
  })
}

test('empty clearable compact menus focus their Off placeholder', () => {
  const view = fixture({ options: devices, defaultValue: '' })
  try {
    const root = view.root.nativeRoot()
    const placeholder = option(root, 'Choose an option')!
    expect(propertyValue(placeholder, 'selected')).toBe(true)
    expectAlignedMenu(root, 'language-placeholder')
  } finally {
    view.dispose()
  }
})

test('empty required compact menus focus an option without exposing a placeholder', () => {
  const view = fixture({ allowClear: false, defaultValue: '' })
  try {
    const root = view.root.nativeRoot()
    expect(option(root, 'Choose an option')).toBeUndefined()
    expect(propertyValue(option(root, 'Rust')!, 'enabled')).toBe(true)
    expectAlignedMenu(root, 'first')
  } finally {
    view.dispose()
  }
})

test('pointer hover leaves no active option behind and keyboard selection still works', () => {
  const view = fixture({ allowClear: false, options: [...options, { value: 'python', label: 'Python' }] })
  try {
    const root = view.root.nativeRoot()
    const combo = mountedRole(root, 'comboBox')!
    const rust = option(root, 'Rust')!
    const go = option(root, 'Go')!
    const python = option(root, 'Python')!
    const rustSurface = optionSurface(rust)!
    const goSurface = optionSurface(go)!
    const pythonSurface = optionSurface(python)!
    const selectedBackground = propertyValue(rustSurface, 'background')
    const goBackground = propertyValue(goSurface, 'background')
    const pythonBackground = propertyValue(pythonSurface, 'background')
    const hoverBackground = propertyValue(pythonSurface, 'hoverBackground')
    expect(hoverBackground).toBeDefined()
    expect(pythonBackground).not.toBe(selectedBackground)

    emit(view, combo, 'key', { kind: 'key', key: 'ArrowDown', state: 'pressed', text: null })
    expect(propertyValue(combo, 'activeDescendant')).toBe('language-option-go')
    expect(propertyValue(goSurface, 'background')).not.toBe(goBackground)
    const entered = pointerListeners(python, 'pointerEnter')
    const left = pointerListeners(python, 'pointerLeave')
    expect(entered.length).toBeGreaterThan(0)
    expect(left.length).toBeGreaterThan(0)
    for (const node of entered) emit(view, node, 'pointerEnter', { kind: 'pointerEnter', width: 300 })
    for (const node of left) emit(view, node, 'pointerLeave', { kind: 'pointerLeave' })

    expect(propertyValue(pythonSurface, 'background')).toBe(pythonBackground)
    expect(propertyValue(pythonSurface, 'hoverBackground')).toBe(hoverBackground)
    expect(propertyValue(goSurface, 'background')).toBe(goBackground)
    expect(propertyValue(rustSurface, 'background')).toBe(selectedBackground)
    expect(propertyValue(rust, 'selected')).toBe(true)
    expect(propertyValue(python, 'selected')).toBe(false)
    expect(view.changes).toEqual([])

    emit(view, combo, 'key', { kind: 'key', key: 'ArrowDown', state: 'pressed', text: null })
    expect(propertyValue(combo, 'activeDescendant')).toBe('language-option-go')
    expect(propertyValue(goSurface, 'background')).not.toBe(goBackground)
    expect(propertyValue(rust, 'selected')).toBe(true)
    emit(view, combo, 'key', { kind: 'key', key: 'Enter', state: 'pressed', text: null })
    expect(view.changes).toEqual(['go'])
    expect(propertyValue(combo, 'accessibleValue')).toBe('Go')
    expect(mountedRole(view.root.nativeRoot(), 'listBox')).toBeUndefined()
  } finally {
    view.dispose()
  }
})
