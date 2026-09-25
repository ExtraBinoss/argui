import { expect, test } from 'bun:test'
import { messageScrollOffset } from '../src/shared/chat-components'

test('message scrolling reads the native and web host offset payload', () => {
  expect(messageScrollOffset({ kind: 'scroll', offsetX: 0, offsetY: 128.5 }))
    .toEqual({ x: 0, y: 128.5 })
  expect(messageScrollOffset({ offsetX: Number.NaN, offsetY: 3 })).toBeUndefined()
  expect(messageScrollOffset({ offsetX: 1 })).toBeUndefined()
})
