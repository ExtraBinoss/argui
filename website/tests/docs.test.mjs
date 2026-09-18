import assert from 'node:assert/strict'
import { access, readFile } from 'node:fs/promises'
import { resolve } from 'node:path'
import { test } from 'node:test'
import { pathToFileURL } from 'node:url'
import { root } from '../scripts/catalogue.mjs'

const source = await readFile(resolve(root, 'website/app/data/docs.ts'), 'utf8')
const slugs = [...source.matchAll(/slug: '([^']+)'/g)].map((match) => match[1])
const examples = [...source.matchAll(/example: example\('([^']+)', '([^']+)'\)/g)].map((match) => ({
  id: match[1],
  filename: match[2],
}))
const generatedPath = resolve(root, 'website/app/data/doc-example-sources.generated.ts')
const { docExampleSources: generatedExamples } = await import(pathToFileURL(generatedPath).href)
const referencedFiles = [...source.matchAll(/sources: \[([^\]]+)\]/g)].flatMap((match) =>
  [...match[1].matchAll(/'([^']+)'/g)].map((path) => path[1]),
)

test('documentation routes are unique and grouped', () => {
  assert.equal(slugs.length, 22)
  assert.equal(new Set(slugs).size, slugs.length)
  for (const prefix of [
    'start/',
    'essentials/',
    'advanced/',
    'architecture/',
    'technicalities/',
    'platforms/',
  ])
    assert.ok(
      slugs.some((slug) => slug.startsWith(prefix)),
      `Missing ${prefix} guides`,
    )
})

test('every guide runs its own exact Rust source', async () => {
  assert.equal(examples.length, slugs.length)
  assert.equal(new Set(examples.map(({ id }) => id)).size, examples.length)
  for (const { filename } of examples) {
    const path = `app_examples/docs-examples/src/examples/${filename}.rs`
    const rust = await readFile(resolve(root, path), 'utf8')
    assert.equal(generatedExamples[filename], rust, `Stale exact source: ${path}`)
  }
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
