import assert from 'node:assert/strict'
import { readFile, access } from 'node:fs/promises'
import { resolve } from 'node:path'
import { test } from 'node:test'
import { createCatalogue, root } from '../scripts/catalogue.mjs'

test('published catalogue matches the Rust gallery and documented widgets', async () => {
  const current = await createCatalogue()
  const published = JSON.parse(
    await readFile(resolve(root, 'website/app/data/catalogue.json'), 'utf8'),
  )
  assert.deepEqual(published, current, 'Run pnpm catalogue after changing the Rust catalogue.')
  assert.equal(new Set(current.map((item) => item.slug)).size, current.length)
  const navigation = await readFile(
    resolve(root, 'crates/argui-widget-gallery/src/navigation.rs'),
    'utf8',
  )
  const slugs = [
    ...navigation
      .split('pub const fn slug(self)')[1]
      .split('\n    }')[0]
      .matchAll(/=> "([^"]+)"/g),
  ].map((match) => match[1])
  for (const slug of slugs)
    assert.ok(
      current.some((item) => item.slug === slug),
      `Missing gallery page: ${slug}`,
    )
})

test('every exposed widget feature is discoverable with a valid facade flag', async () => {
  const items = await createCatalogue()
  const widgets = await readFile(resolve(root, 'crates/argui-widgets/Cargo.toml'), 'utf8')
  const facade = await readFile(resolve(root, 'crates/argui/Cargo.toml'), 'utf8')
  const features = [
    ...widgets
      .split('[features]')[1]
      .split('[dependencies]')[0]
      .matchAll(/^([\w-]+) = \[/gm),
  ]
    .map((match) => match[1])
    .filter((name) => !['default', 'all'].includes(name))
  for (const feature of features) {
    assert.ok(
      items.some((item) => item.feature === feature),
      `Missing widget feature: ${feature}`,
    )
    assert.ok(facade.includes(`widget-${feature} = `), `Invalid facade feature: ${feature}`)
  }
})

test('all marketing implementation links resolve in the repository', async () => {
  for (const page of ['index.vue', 'features.vue', 'get-started.vue']) {
    const text = await readFile(resolve(root, 'website/app/pages', page), 'utf8')
    for (const match of text.matchAll(/(?:path: |sourceUrl\()'([^']+)'/g))
      await access(resolve(root, match[1]))
  }
})
