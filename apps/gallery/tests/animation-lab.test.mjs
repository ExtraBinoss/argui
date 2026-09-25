import assert from 'node:assert/strict'
import { test } from 'node:test'
import contract from '../../../packages/host/src/contract.generated.json' with { type: 'json' }
import { mountGallery } from '../dist/gallery-core.mjs'
import { mountReactGallery } from '../dist/gallery-react-core.mjs'

test('continuous gallery motion uses compositor properties', async () => {
  for (const mount of [mountGallery, mountReactGallery]) {
    const operations = []
    let deliver = () => {}
    const dispose = mount({
      contract: () => contract,
      commit: (batch) => operations.push(...batch),
      subscribe: (callback) => { deliver = callback; return () => {} },
    }, contract.abiHash)
    try {
      const names = new Map(contract.natives.map((native) => [native.id,
        new Map(native.properties.map((property) => [property.id, property.name]))]))
      let types = new Map(operations.filter((operation) => operation.kind === 'create')
        .map((operation) => [`${operation.id.slot}:${operation.id.generation}`, operation.nativeType]))
      const navigation = operations.find((operation) => operation.kind === 'setProperty'
        && operation.value?.value === 'page-animation-lab')
      assert.ok(navigation, 'Animation Lab must be available in the navigation')
      const identity = `${navigation.id.slot}:${navigation.id.generation}`
      const click = contract.natives.find((native) => native.id === types.get(identity))
        .events.find((event) => event.name === 'click')
      const listener = operations.find((operation) => operation.kind === 'setListener'
        && `${operation.id.slot}:${operation.id.generation}` === identity
        && operation.event === click.id)
      assert.ok(listener?.callback)
      deliver({ node: navigation.id, callback: listener.callback })
      for (let attempt = 0; attempt < 100; attempt++) {
        types = new Map(operations.filter((operation) => operation.kind === 'create')
          .map((operation) => [`${operation.id.slot}:${operation.id.generation}`, operation.nativeType]))
        if (operations.some((operation) => operation.kind === 'setProperty'
          && names.get(types.get(`${operation.id.slot}:${operation.id.generation}`))
            ?.get(operation.property) === 'loop_opacity')) break
        await new Promise((resolve) => setTimeout(resolve, 10))
      }
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
