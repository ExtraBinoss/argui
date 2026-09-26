import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { test } from 'node:test'
import { mountGallery as mountSolid } from '../dist/gallery-core.mjs'
import { mountGallery as mountReact } from '../dist/gallery-react-core.mjs'
import { createThemeBridge } from './fixtures/theme-bridge.mjs'

const contract = JSON.parse(readFileSync('packages/host/src/contract.generated.json', 'utf8'))

function bridgeFor(batches) {
  return {
    contract: () => contract,
    commit: (operations) => batches.push(operations),
    subscribe: () => () => {},
    theme: createThemeBridge(),
  }
}

for (const [name, mount] of [['Solid', mountSolid], ['React', mountReact]]) {
  test(`${name} gallery mounts and releases one native widget tree`, () => {
    const batches = []
    const dispose = mount(bridgeFor(batches), contract.abiHash)
    assert(batches[0].some((operation) => operation.kind === 'setRoot'))
    assert(batches[0].some((operation) => operation.kind === 'create'))
    dispose()
    assert(batches.at(-1).some((operation) => operation.kind === 'remove'
      || operation.kind === 'setRoot' && operation.id === null))
  })
}
