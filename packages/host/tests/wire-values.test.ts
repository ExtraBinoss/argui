import { expect, test } from 'bun:test'
import { encodeValue, equalValue } from '../src/wire-values'
import type { NativeProperty } from '../src/protocol'

const background: NativeProperty = { id: 1, name: 'background', valueType: 'Brush', readOnly: false }

test('identical fresh gradients do not emit another native mutation', () => {
  const brush = { kind: 'linear', angle: 0, space: 'srgb',
    stops: [{ offset: 0, color: '#12345600' }, { offset: 1, color: '#123456ff' }] }
  const before = encodeValue(background, brush)
  expect(equalValue(before, encodeValue(background, structuredClone(brush)))).toBe(true)
  expect(equalValue(before, encodeValue(background, { ...brush, angle: 90 }))).toBe(false)
  expect(equalValue(before, encodeValue(background, '#123456'))).toBe(false)
})

test('bilinear brushes require exactly four corners and a supported interpolation space', () => {
  const brush = { kind: 'bilinear', space: 'srgb', corners: ['#ffffff', '#ff0000', '#000000', '#000000'] }
  expect(encodeValue(background, brush)).toEqual({ type: 'Brush', value: brush })
  for (const invalid of [
    { ...brush, corners: brush.corners.slice(1) },
    { ...brush, corners: [...brush.corners, '#fff'] },
    { ...brush, corners: ['#fff', null, '#000', '#000'] },
    { ...brush, space: 'hsl' },
    { ...brush, angle: 0 },
  ]) expect(() => encodeValue(background, invalid)).toThrow(TypeError)
  expect(equalValue(encodeValue(background, brush), encodeValue(background, { ...brush, corners: [...brush.corners] }))).toBe(true)
})
