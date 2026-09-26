import { expect, test } from 'bun:test'
import { NativeHost, type NativeBridge, type NativeContract, type Operation } from '../src'

const contract: NativeContract = {
  abiHash: 'v2-fixture',
  natives: [
    { id: 1, name: 'Container', events: [], properties: [
      { id: 1, name: 'id', valueType: 'String', readOnly: false },
      { id: 2, name: 'width', valueType: 'Dimension', readOnly: false },
      { id: 3, name: 'padding', valueType: 'Insets', readOnly: false },
      { id: 4, name: 'radii', valueType: 'Radii', readOnly: false },
      { id: 5, name: 'border', valueType: 'Border', readOnly: false },
      { id: 6, name: 'shadow', valueType: 'Shadow', readOnly: false },
      { id: 7, name: 'transform', valueType: 'Transform', readOnly: false },
      { id: 8, name: 'placement', valueType: 'String', readOnly: false, allowedValues: ['bottomStart', 'topStart'] },
      { id: 10, name: 'containerRules', valueType: 'ContainerRules', readOnly: false },
      { id: 11, name: 'inset', valueType: 'PositionInsets', readOnly: false },
      { id: 12, name: 'minWidth', valueType: 'Constraint', readOnly: false },
    ] },
    { id: 2, name: 'Text', events: [], properties: [
      { id: 1, name: 'id', valueType: 'String', readOnly: false },
      { id: 9, name: 'text', valueType: 'String', readOnly: false },
    ] },
  ],
}

function recorder(): { host: NativeHost; batches: Operation[][] } {
  const batches: Operation[][] = []
  const bridge: NativeBridge = {
    contract: () => contract,
    commit: (operations) => { batches.push([...operations]) },
    subscribe: () => () => {},
  }
  return { host: new NativeHost(bridge, contract.abiHash), batches }
}

test('automatic IDs are stable while mounted, explicit IDs replace them and collisions fail', () => {
  const { host, batches } = recorder()
  const root = host.createElement('container')
  const child = host.createElement('container')
  host.setProperty(root, 'id', 'screen')
  host.setProperty(child, 'id', 'button')
  host.insertNode(root, child)
  host.setRoot(root)
  expect(root.values.get(1)?.value).toBe('screen')
  expect(child.values.get(1)?.value).toBe('button')
  expect(batches[0]!.filter((op) => op.kind === 'setProperty' && op.property === 1)).toHaveLength(2)
  expect(() => host.setProperty(child, 'id', 'screen')).toThrow('Duplicate native id')
  host.setProperty(child, 'id', null)
  expect(child.values.get(1)?.value).toBe(`argui-${child.id.slot}-${child.id.generation}`)
  const ref = host.handle(child)
  expect(ref.mounted).toBe(true)
  expect(ref.id).toBe(child.values.get(1)?.value as string)
  host.removeNode(root, child)
  expect(ref.mounted).toBe(false)
  expect(ref.hostId).toBeNull()
})

test('layout and paint values reject ambiguous or unsupported wire shapes', () => {
  const { host, batches } = recorder()
  const root = host.createElement('container')
  host.setProperty(root, 'width', '50%')
  host.setProperty(root, 'padding', { top: 8, start: 12 })
  host.setProperty(root, 'radii', { topLeft: 6, bottomRight: 3 })
  host.setProperty(root, 'border', { width: 1, color: '#fff' })
  host.setProperty(root, 'shadow', { blur: 12, color: '#000', offsetY: 2 })
  host.setProperty(root, 'transform', { translateX: 4, rotation: 45 })
  host.setProperty(root, 'placement', 'bottomStart')
  host.setProperty(root, 'containerRules', [{
    scope: 'cards', when: { minWidth: 480 },
    style: { gridColumns: [{ repeat: { count: 'autoFit', tracks: [{ minmax: { min: 160, max: { fr: 1 } } }] } }] },
  }])
  host.setProperty(root, 'inset', { top: 4, start: 8 })
  host.setProperty(root, 'minWidth', '50%')
  host.setRoot(root)
  expect(batches[0]!.filter((op) => op.kind === 'setProperty').map((op) => op.value?.type)).toEqual([
    'String', 'Dimension', 'Insets', 'Radii', 'Border', 'Shadow', 'Transform', 'String', 'ContainerRules',
    'PositionInsets', 'Constraint',
  ])
  expect(() => host.setProperty(root, 'width', 'fill')).toThrow('Invalid Dimension')
  expect(() => host.setProperty(root, 'padding', { left: 1, start: 2 })).toThrow('Invalid Insets')
  expect(() => host.setProperty(root, 'border', { color: '#fff' })).toThrow('Invalid Border')
  expect(() => host.setProperty(root, 'shadow', { blur: 2, color: '#000', extra: 1 })).toThrow('Invalid Shadow')
  expect(() => host.setProperty(root, 'placement', 'bottom_strat')).toThrow('Invalid String')
  expect(() => host.setProperty(root, 'containerRules', [{ scope: 'cards', when: {}, style: { gap: 12 } }])).toThrow('Invalid ContainerRules')
  expect(() => host.setProperty(root, 'inset', { end: 2, right: 3 })).toThrow('Invalid PositionInsets')
  expect(() => host.setProperty(root, 'minWidth', 'fit')).toThrow('Invalid Constraint')
})

test('Solid inline text children update one Text property without native child nodes', () => {
  const { host, batches } = recorder()
  const text = host.createElement('text')
  const child = host.createInlineTextNode('Bonjour')
  host.insertNode(text, child)
  host.setRoot(text)
  expect(text.values.get(9)?.value).toBe('Bonjour')
  expect(batches[0]!.filter((op) => op.kind === 'create')).toHaveLength(1)
  expect(batches[0]!.some((op) => op.kind === 'insert')).toBe(false)
  host.replaceText(child, 'Salut')
  host.flush()
  expect(text.values.get(9)?.value).toBe('Salut')
  expect(() => host.setProperty(text, 'text', 'Conflict')).toThrow('either children or text')
  host.removeNode(text, child)
  expect(text.values.get(9)?.value).toBe('')
})

test('custom native schemas reject non-camelCase public names before mounting', () => {
  const malformed: NativeContract = {
    abiHash: 'bad', natives: [{ id: 1, name: 'Custom', properties: [
      { id: 1, name: 'shadow_color', valueType: 'Color', readOnly: false },
    ], events: [] }],
  }
  const bridge: NativeBridge = {
    contract: () => malformed, commit: () => { throw new Error('should not commit') }, subscribe: () => () => {},
  }
  expect(() => new NativeHost(bridge, 'bad')).toThrow('Invalid public schema property')
})
