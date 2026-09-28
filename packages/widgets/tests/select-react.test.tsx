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
function fixture(props: Partial<Pick<SelectProps, 'allowClear' | 'options' | 'defaultValue' | 'width' | 'contentWidth'>> = {}) {
  const environment = globalThis as typeof globalThis & { IS_REACT_ACT_ENVIRONMENT?: boolean }
  const previous = environment.IS_REACT_ACT_ENVIRONMENT
  environment.IS_REACT_ACT_ENVIRONMENT = true
  const { host, runtime, deliver } = nativeControlsFixture()
  const root = createRoot(host)
  const changes: string[] = []
  act(() => root.render(<ThemeProvider runtime={runtime}>
    <Select id="language" label="Language" variant="shadcn" options={props.options ?? options}
      defaultValue={props.defaultValue ?? 'rust'} defaultOpen allowClear={props.allowClear}
      width={props.width} contentWidth={props.contentWidth}
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

/** Checks visible panel alignment after accounting for the native window's shadow frame. */
function expectBelowTrigger(root: NativeNode, initialFocus: string, visibleWidth = 240): void {
  const popup = mountedRole(root, 'listBox')!
  expect(popup).toBeDefined()
  expect(propertyValue(popup, 'initialFocus')).toBe(initialFocus)
  expect(propertyValue(popup, 'anchor')).toBe('language')
  expect(propertyValue(popup, 'placement')).toBe('bottomStart')
  const frame = popup.children[0]!
  const inset = propertyValue(frame, 'padding') as number
  const clientWidth = propertyValue(popup, 'width') as number
  const offset = propertyValue(popup, 'placementOffset') as number
  const crossOffset = propertyValue(popup, 'placementCrossOffset') as number
  expect(inset).toBe(18)
  expect(clientWidth).toBe(visibleWidth + inset * 2)
  expect(propertyValue(frame, 'width')).toBe('100%')
  expect(propertyValue(frame.children[0]!, 'width')).toBe('100%')
  expect(crossOffset + inset).toBe(0)
  expect(offset + inset).toBe(4)
}

/** Returns the sole native measurement target for a real option's label. */
function labelArea(node: NativeNode): NativeNode {
  const entered = pointerListeners(node, 'pointerEnter')
  const left = pointerListeners(node, 'pointerLeave')
  expect(entered).toHaveLength(1)
  expect(left).toHaveLength(1)
  expect(left[0]).toBe(entered[0])
  expect(entered[0]!.type.name).toBe('TouchArea')
  return entered[0]!
}

/** Finds the moving text surface inside the stationary label hit target. */
function marqueeSurface(area: NativeNode): NativeNode {
  expect(area.children).toHaveLength(1)
  const surface = area.children[0]!
  expect(surface.type.name).toBe('Rectangle')
  expect(propertyValue(surface, 'height')).toBe(20)
  return surface
}

/** Confirms leaving a label clears all native loop properties, returning text to rest. */
function expectStoppedMarquee(surface: NativeNode): void {
  for (const name of ['loopMs', 'loopTranslateX', 'loopHold']) {
    expect(propertyValue(surface, name)).toBeUndefined()
  }
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
      expectBelowTrigger(root, 'language-option-device%2F9')
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
    expectBelowTrigger(root, 'language-placeholder')
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
    expectBelowTrigger(root, 'first')
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
    const label = labelArea(python)
    emit(view, label, 'pointerEnter', { kind: 'pointerEnter', width: 300 })
    expect(propertyValue(combo, 'activeDescendant')).toBe('language-option-rust')
    expect(propertyValue(goSurface, 'background')).toBe(goBackground)
    emit(view, combo, 'key', { kind: 'key', key: 'ArrowDown', state: 'pressed', text: null })
    expect(propertyValue(combo, 'activeDescendant')).toBe('language-option-go')
    emit(view, label, 'pointerLeave', { kind: 'pointerLeave' })
    expect(propertyValue(combo, 'activeDescendant')).toBe('language-option-rust')
    expectStoppedMarquee(marqueeSurface(label))

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

test('option labels keep an intrinsic measurement target without a blocking TouchArea ancestor', () => {
  const view = fixture({ allowClear: false })
  try {
    for (const { label } of options) {
      const nativeOption = option(view.root.nativeRoot(), label)!
      const area = labelArea(nativeOption)
      expect(propertyValue(area, 'width')).toBeUndefined()
      expect(propertyValue(area, 'height')).toBeUndefined()
      let ancestor = area.parent
      while (ancestor && ancestor !== nativeOption) {
        expect(ancestor.type.name).not.toBe('TouchArea')
        ancestor = ancestor.parent
      }
      expect(ancestor).toBe(nativeOption)
    }
  } finally {
    view.dispose()
  }
})

test('select marquee stays slow on long labels and resets when hover ends', () => {
  const label = 'A long native input device name'
  const view = fixture({ allowClear: false, width: 120, contentWidth: 200,
    options: [...options, { value: 'long', label }] })
  try {
    const area = labelArea(option(view.root.nativeRoot(), label)!)
    const surface = marqueeSurface(area)
    // The 200 px visible panel reserves 38 px for its border, padding, and check slot.
    const viewport = 162
    expectStoppedMarquee(surface)
    for (const overflow of [1.25, 100.25, 1_000.25, 400_000]) {
      emit(view, area, 'pointerEnter', { kind: 'pointerEnter', width: viewport + overflow })
      const period = propertyValue(surface, 'loopMs') as number
      expect(period).toBe(Math.max(4_000, Math.ceil(overflow / 14 / 0.35 * 1_000)))
      expect(propertyValue(surface, 'loopTranslateX')).toBe(-overflow)
      expect(propertyValue(surface, 'loopHold')).toBe(true)
      // loopHold spends 35% of its period traversing each leg, at no more than 14 px/s.
      const speed = overflow / (period / 1_000 * 0.35)
      expect(speed).toBeLessThanOrEqual(14)
      if (overflow >= 100) expect(speed).toBeGreaterThan(13.99)
      if (overflow >= 1_000) expect(period).toBeGreaterThan(60_000)
      emit(view, area, 'pointerLeave', { kind: 'pointerLeave' })
      expectStoppedMarquee(surface)
    }
    emit(view, area, 'pointerEnter', { kind: 'pointerEnter', width: viewport + 8 })
    expect(propertyValue(surface, 'loopMs')).toBe(4_000)
    expect(propertyValue(surface, 'loopTranslateX')).toBe(-8)
    emit(view, area, 'pointerLeave', { kind: 'pointerLeave' })
    expectStoppedMarquee(surface)
    expect(view.changes).toEqual([])
  } finally {
    view.dispose()
  }
})

test('select labels without measurable overflow do not start a marquee', () => {
  const view = fixture({ allowClear: false, contentWidth: 200 })
  try {
    const area = labelArea(option(view.root.nativeRoot(), 'Go')!)
    const surface = marqueeSurface(area)
    for (const width of [undefined, 0, 160, 162, 163]) {
      emit(view, area, 'pointerEnter', { kind: 'pointerEnter', width })
      expectStoppedMarquee(surface)
      emit(view, area, 'pointerLeave', { kind: 'pointerLeave' })
      expectStoppedMarquee(surface)
    }
  } finally {
    view.dispose()
  }
})

test('visible select panels honor content width or trigger width with the same lower gap', () => {
  for (const contentWidth of [undefined, 310]) {
    for (const defaultValue of ['device/0', 'device/9']) {
      const view = fixture({ allowClear: false, options: devices, defaultValue, width: 180, contentWidth })
      try {
        expectBelowTrigger(view.root.nativeRoot(), `language-option-${encodeURIComponent(defaultValue)}`,
          contentWidth ?? 180)
      } finally {
        view.dispose()
      }
    }
  }
})
