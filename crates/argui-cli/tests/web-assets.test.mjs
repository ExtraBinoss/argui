import assert from 'node:assert/strict'
import { test } from 'node:test'
import { loadWebAssets } from '../assets/templates/assets-web.ts'

const base = 'https://example.test/app/index.html'

function fetcher(files) {
  return async url => files.get(url.pathname) ?? new Response('missing', { status: 404 })
}

test('browser startup loads only release assets from a movable subpath', async () => {
  const files = new Map([
    ['/app/assets.generated.json', Response.json({
      version: 1,
      assets: [{ key: 'mine/star.svg', kind: 'svg', id: 42, path: 'mine/star.svg', bytes: 3 }],
    })],
    ['/app/assets/mine/star.svg', new Response(new Uint8Array([1, 2, 3]))],
  ])
  const assets = await loadWebAssets(base, false, fetcher(files))
  assert.equal(assets.length, 1)
  assert.deepEqual([...assets[0].bytes], [1, 2, 3])
  assert.equal(assets[0].id, 42)
})

test('browser development selects the full-pack manifest', async () => {
  const files = new Map([
    ['/app/assets.dev.generated.json', Response.json({ version: 1, assets: [] })],
  ])
  assert.deepEqual(await loadWebAssets(base, true, fetcher(files)), [])
})

test('browser startup rejects unsafe and stale asset sources', async () => {
  const unsafe = new Map([
    ['/app/assets.generated.json', Response.json({
      version: 1,
      assets: [{ key: 'bad', kind: 'svg', id: 42, path: '../secret.svg', bytes: 3 }],
    })],
  ])
  await assert.rejects(loadWebAssets(base, false, fetcher(unsafe)), /invalid asset entry/)
  const stale = new Map([
    ['/app/assets.generated.json', Response.json({
      version: 1,
      assets: [{ key: 'mine/star.svg', kind: 'svg', id: 42, path: 'mine/star.svg', bytes: 4 }],
    })],
    ['/app/assets/mine/star.svg', new Response(new Uint8Array([1, 2, 3]))],
  ])
  await assert.rejects(loadWebAssets(base, false, fetcher(stale)), /changed after asset generation/)
})
