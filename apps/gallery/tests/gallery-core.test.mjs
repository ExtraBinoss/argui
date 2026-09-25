import { test } from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { mountGallery } from '../dist/gallery-core.mjs'
import { createThemeBridge } from './fixtures/theme-bridge.mjs'

const contract = JSON.parse(readFileSync('packages/host/src/contract.generated.json', 'utf8'))

test('runtime-neutral gallery mounts the same native tree without a Bun bootstrap', () => {
  const batches = []
  const bridge = {
    contract: () => contract,
    commit: (operations) => batches.push(operations),
    subscribe: () => () => {},
    theme: createThemeBridge(),
  }
  const dispose = mountGallery(bridge, contract.abiHash)
  assert(batches[0].some((operation) => operation.kind === 'setRoot'))
  assert(batches[0].some((operation) => operation.kind === 'create'))
  dispose()
  assert(batches.at(-1).some((operation) => operation.kind === 'remove'))
})

test('busy loader needs no recurring JavaScript native commits', async () => {
  const batches = []
  const dispose = mountGallery({
    contract: () => contract,
    commit: (operations) => batches.push(operations),
    subscribe: () => () => {},
    theme: createThemeBridge(),
  }, contract.abiHash)
  try {
    const initial = batches.length
    await new Promise((resolve) => setTimeout(resolve, 350))
    assert.equal(batches.length, initial)
  } finally {
    dispose()
  }
})
