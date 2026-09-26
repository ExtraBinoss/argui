/** @jsxImportSource @argui/react */
import { expect, test } from 'bun:test'
import { act } from 'react'
import { createRoot, ThemeProvider } from '@argui/react'
import type { NativeNode } from '@argui/host'
import { Checkbox, Progress, Slider, Switch, Tabs } from '../src/react'
import { mountedRole, mountedRoles, nativeControlsFixture } from './fixtures/native-controls'

/** Finds one authored property value within a mounted widget subtree. */
function propertyValue(node: NativeNode, name: string): unknown {
  const property = node.type.properties.find((entry) => entry.name === name)
  const value = property ? node.values.get(property.id)?.value : undefined
  if (value !== undefined) return value
  for (const child of node.children) {
    const found = propertyValue(child, name)
    if (found !== undefined) return found
  }
  return undefined
}

test('React controls mount and their native pointer and motion contracts remain stable', () => {
  const actEnvironment = globalThis as typeof globalThis & { IS_REACT_ACT_ENVIRONMENT?: boolean }
  const previousActEnvironment = actEnvironment.IS_REACT_ACT_ENVIRONMENT
  actEnvironment.IS_REACT_ACT_ENVIRONMENT = true
  const { host, runtime, deliver } = nativeControlsFixture()
  const root = createRoot(host)
  act(() => root.render(<ThemeProvider runtime={runtime}><column>
    <Checkbox accessibleName="Accept" defaultValue={false} />
    <Switch accessibleName="Enable" defaultValue={true} />
    <Tabs accessibleName="Views" items={[{ value: 'one', label: 'One', content: <text>First</text> }]} />
    <Slider accessibleName="Volume" defaultValue={25} />
    <Progress accessibleName="Processing" value={null} width={280} playing={false} />
  </column></ThemeProvider>))
  const roles = mountedRoles(root.nativeRoot())
  for (const role of ['checkBox', 'switch', 'tabList', 'tab', 'tabPanel', 'slider', 'progress']) {
    expect(roles).toContain(role)
  }
  const checkbox = mountedRole(root.nativeRoot(), 'checkBox')!
  const click = checkbox.type.events.find((event) => event.name === 'click')!
  act(() => deliver({ node: checkbox.id, callback: checkbox.listeners.get(click.id)!, payload: { kind: 'click' } }))
  const stateProperty = checkbox.type.properties.find((property) => property.name === 'checkedState')!
  expect(checkbox.values.get(stateProperty.id)?.value).toBe('checked')
  const switchNode = mountedRole(root.nativeRoot(), 'switch')!
  expect(propertyValue(switchNode, 'transitionMs')).toBe(150)
  expect(propertyValue(switchNode, 'transform')).toMatchObject({ translateX: 14 })
  const switchClick = switchNode.type.events.find((event) => event.name === 'click')!
  act(() => deliver({ node: switchNode.id, callback: switchNode.listeners.get(switchClick.id)!,
    payload: { kind: 'click' } }))
  expect(propertyValue(switchNode, 'transform')).toMatchObject({ translateX: 0 })
  const slider = mountedRole(root.nativeRoot(), 'slider')!
  expect(propertyValue(slider, 'accessibleValue')).toBeUndefined()
  const touchArea = slider.children[0]!
  const pointerDown = touchArea.type.events.find((event) => event.name === 'pointerDown')!
  const moved = touchArea.type.events.find((event) => event.name === 'moved')!
  const pointerUp = touchArea.type.events.find((event) => event.name === 'pointerUp')!
  const pointerCancel = touchArea.type.events.find((event) => event.name === 'pointerCancel')!
  const numericValue = slider.type.properties.find((property) => property.name === 'numericValue')!
  act(() => deliver({ node: touchArea.id, callback: touchArea.listeners.get(moved.id)!,
    payload: { kind: 'pointerMove', localX: 108, width: 116 } }))
  expect(slider.values.get(numericValue.id)?.value).toBe(25)
  act(() => deliver({ node: touchArea.id, callback: touchArea.listeners.get(pointerDown.id)!,
    payload: { kind: 'pointerDown', localX: 58, width: 116 } }))
  expect(slider.values.get(numericValue.id)?.value).toBe(50)
  act(() => deliver({ node: touchArea.id, callback: touchArea.listeners.get(moved.id)!,
    payload: { kind: 'pointerMove', localX: 94, width: 116 } }))
  expect(slider.values.get(numericValue.id)?.value).toBe(86)
  act(() => deliver({ node: touchArea.id, callback: touchArea.listeners.get(pointerUp.id)!,
    payload: { kind: 'pointerUp', localX: 108, width: 116 } }))
  expect(slider.values.get(numericValue.id)?.value).toBe(100)
  act(() => deliver({ node: touchArea.id, callback: touchArea.listeners.get(moved.id)!,
    payload: { kind: 'pointerMove', localX: 8, width: 116 } }))
  expect(slider.values.get(numericValue.id)?.value).toBe(100)
  act(() => deliver({ node: touchArea.id, callback: touchArea.listeners.get(pointerDown.id)!,
    payload: { kind: 'pointerDown', localX: 58, width: 116 } }))
  act(() => deliver({ node: touchArea.id, callback: touchArea.listeners.get(pointerCancel.id)!,
    payload: { kind: 'pointerCancel' } }))
  act(() => deliver({ node: touchArea.id, callback: touchArea.listeners.get(moved.id)!,
    payload: { kind: 'pointerMove', localX: 108, width: 116 } }))
  expect(slider.values.get(numericValue.id)?.value).toBe(50)
  const progress = mountedRole(root.nativeRoot(), 'progress')!
  expect(propertyValue(progress, 'busy')).toBe(true)
  expect(propertyValue(progress, 'accessibleValue')).toBe('In progress')
  expect(propertyValue(progress, 'numericValue')).toBeUndefined()
  expect(propertyValue(progress, 'minimumValue')).toBeUndefined()
  expect(propertyValue(progress, 'maximumValue')).toBeUndefined()
  expect(propertyValue(progress, 'loopMs')).toBe(900)
  expect(propertyValue(progress, 'loopTranslateX')).toBe(196)
  expect(propertyValue(progress, 'loopPlaying')).toBe(false)
  act(() => root.unmount())
  runtime.dispose()
  actEnvironment.IS_REACT_ACT_ENVIRONMENT = previousActEnvironment
})

test('indeterminate progress pauses and resumes on one retained native node', () => {
  const actEnvironment = globalThis as typeof globalThis & { IS_REACT_ACT_ENVIRONMENT?: boolean }
  const previousActEnvironment = actEnvironment.IS_REACT_ACT_ENVIRONMENT
  actEnvironment.IS_REACT_ACT_ENVIRONMENT = true
  const { host, runtime } = nativeControlsFixture()
  const root = createRoot(host)
  const renderProgress = (value: number | null, playing: boolean) => act(() => root.render(
    <ThemeProvider runtime={runtime}>
      <Progress accessibleName="Processing" value={value} width={280} playing={playing} />
    </ThemeProvider>,
  ))
  renderProgress(50, true)
  const progress = mountedRole(root.nativeRoot(), 'progress')!
  expect(propertyValue(progress, 'numericValue')).toBe(50)
  expect(propertyValue(progress, 'minimumValue')).toBe(0)
  expect(propertyValue(progress, 'maximumValue')).toBe(100)
  expect(propertyValue(progress, 'accessibleValue')).toBeUndefined()
  renderProgress(null, true)
  expect(mountedRole(root.nativeRoot(), 'progress')).toBe(progress)
  expect(propertyValue(progress, 'numericValue')).toBeUndefined()
  expect(propertyValue(progress, 'minimumValue')).toBeUndefined()
  expect(propertyValue(progress, 'maximumValue')).toBeUndefined()
  expect(propertyValue(progress, 'accessibleValue')).toBe('In progress')
  expect(propertyValue(progress, 'loopPlaying')).toBe(true)
  renderProgress(null, false)
  expect(mountedRole(root.nativeRoot(), 'progress')).toBe(progress)
  expect(propertyValue(progress, 'loopPlaying')).toBe(false)
  renderProgress(null, true)
  expect(mountedRole(root.nativeRoot(), 'progress')).toBe(progress)
  expect(propertyValue(progress, 'loopPlaying')).toBe(true)
  act(() => root.unmount())
  runtime.dispose()
  actEnvironment.IS_REACT_ACT_ENVIRONMENT = previousActEnvironment
})
