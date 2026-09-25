import assert from 'node:assert/strict'
import { test } from 'node:test'
import contract from '../../../packages/host/src/contract.generated.json' with { type: 'json' }
import { mountGallery } from '../dist/gallery-core.mjs'
import { mountReactGallery } from '../dist/gallery-react-core.mjs'

test('continuous gallery motion uses compositor properties', () => {
  for (const mount of [mountGallery, mountReactGallery]) {
    const operations = []
    const dispose = mount({
      contract: () => contract,
      commit: (batch) => operations.push(...batch),
      subscribe: () => () => {},
    }, contract.abiHash)
    try {
      const names = new Map(contract.natives.map((native) => [native.id,
        new Map(native.properties.map((property) => [property.id, property.name]))]))
      const types = new Map(operations.filter((operation) => operation.kind === 'create')
        .map((operation) => [`${operation.id.slot}:${operation.id.generation}`, operation.nativeType]))
      const authored = operations.filter((operation) => operation.kind === 'setProperty')
        .map((operation) => names.get(types.get(`${operation.id.slot}:${operation.id.generation}`))?.get(operation.property))
      assert.ok(authored.includes('loop_opacity'), 'the animation page should retain continuous motion')
      for (const property of ['loop_radius', 'loop_background', 'loop_width', 'loop_gap']) {
        assert.ok(!authored.includes(property), `${property} forces repaint or layout each frame`)
      }
    } finally {
      dispose()
    }
  }
})
