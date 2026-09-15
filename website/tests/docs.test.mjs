import assert from 'node:assert/strict'
import { access, readFile } from 'node:fs/promises'
import { resolve } from 'node:path'
import { test } from 'node:test'
import { root } from '../scripts/catalogue.mjs'

const source = await readFile(resolve(root, 'website/app/data/docs.ts'), 'utf8')
const slugs = [...source.matchAll(/slug: '([^']+)'/g)].map((match) => match[1])
const demos = [...source.matchAll(/demo: '([^']+)'/g)].map((match) => match[1])
const referencedFiles = [...source.matchAll(/sources: \[([^\]]+)\]/g)].flatMap((match) =>
  [...match[1].matchAll(/'([^']+)'/g)].map((path) => path[1]),
)

test('documentation routes are unique and grouped', () => {
  assert.equal(slugs.length, 17)
  assert.equal(new Set(slugs).size, slugs.length)
  for (const prefix of ['start/', 'essentials/', 'advanced/', 'architecture/'])
    assert.ok(
      slugs.some((slug) => slug.startsWith(prefix)),
      `Missing ${prefix} guides`,
    )
})

test('every guide ends in a real gallery page', async () => {
  const catalogue = JSON.parse(
    await readFile(resolve(root, 'website/app/data/catalogue.json'), 'utf8'),
  )
  assert.equal(demos.length, slugs.length)
  for (const demo of demos)
    assert.ok(
      catalogue.some((item) => item.name === demo || item.gallery === demo),
      `Unknown gallery demo: ${demo}`,
    )
})

test('every documentation reference exists', async () => {
  assert.ok(referencedFiles.length > slugs.length * 2)
  for (const path of new Set(referencedFiles)) await access(resolve(root, path))
})

test('every guide has distinct reference files', () => {
  const sourceLists = [...source.matchAll(/sources: \[([^\]]+)\]/g)].map((match) =>
    [...match[1].matchAll(/'([^']+)'/g)].map((path) => path[1]),
  )
  assert.equal(sourceLists.length, slugs.length)
  for (const files of sourceLists) assert.equal(new Set(files).size, files.length)
})
