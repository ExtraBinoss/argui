import assert from 'node:assert/strict'
import { access, readFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import { resolve } from 'node:path'

const website = fileURLToPath(new URL('../', import.meta.url))
const output = resolve(website, '.output/public')
const catalogue = JSON.parse(await readFile(resolve(website, 'app/data/catalogue.json'), 'utf8'))
const docsSource = await readFile(resolve(website, 'app/data/docs.ts'), 'utf8')
const docs = [...docsSource.matchAll(/slug: '([^']+)'/g)].map((match) => match[1])
const origin = (process.env.NUXT_PUBLIC_SITE_URL ?? '').replace(/\/$/, '')
const base = process.env.NUXT_APP_BASE_URL ?? '/'
const routes = [
  '/',
  '/features',
  '/examples',
  '/docs',
  ...docs.map((slug) => `/docs/${slug}`),
  '/components',
  ...catalogue.map((item) => `/components/${item.slug}`),
]
for (const route of routes) {
  const html = await readFile(resolve(output, `.${route}`, 'index.html'), 'utf8')
  assert.match(html, /<html[^>]*lang="en"/, route)
  assert.match(html, /<title>[^<]*Argui[^<]*<\/title>/, route)
  assert.match(html, /<meta name="description" content="[^"]+"/, route)
  assert.equal((html.match(/<h1(?:\s|>)/g) ?? []).length, 1, route)
  assert.ok(!html.includes('undefined component'), route)
  assert.ok(html.includes(`src="${base}argui-icon.png"`), `Missing base path in logo: ${route}`)
  assert.ok(html.includes(`href="${base}favicon.svg"`), `Missing base path in favicon: ${route}`)
  assert.ok(html.includes(`href="${base}components"`), `Missing base path in navigation: ${route}`)
  if (origin) {
    assert.ok(html.includes(`href="${origin}${route}"`), `Missing canonical URL: ${route}`)
    assert.ok(html.includes(`${origin}/social.png`), `Missing social preview: ${route}`)
  } else assert.ok(!html.includes('rel="canonical"'), 'Do not invent a production domain')
}
const legacyStart = await readFile(resolve(output, 'get-started/index.html'), 'utf8')
assert.ok(legacyStart.includes('/docs/start/installation'), 'The legacy start page must redirect')
for (const item of catalogue) {
  const html = await readFile(resolve(output, 'components', item.slug, 'index.html'), 'utf8')
  assert.ok(html.includes(item.source), `Missing server-rendered source: ${item.slug}`)
}
for (const slug of docs) {
  const html = await readFile(resolve(output, 'docs', slug, 'index.html'), 'utf8')
  assert.ok(html.includes('Compiled example'), `Missing live example: ${slug}`)
  assert.ok(
    html.includes('app_examples/docs-examples/src/examples/'),
    `Missing exact source: ${slug}`,
  )
  assert.ok(html.includes('Reference files'), `Missing reference files: ${slug}`)
}
const sitemap = await readFile(resolve(output, 'sitemap.xml'), 'utf8')
assert.equal((sitemap.match(/<loc>/g) ?? []).length, origin ? routes.length : 0)
assert.ok(!sitemap.includes('/gallery/'))
assert.ok(!sitemap.includes('/examples/ai-harness/'))
assert.ok(!sitemap.includes('/examples/gpu-canvas/'))
assert.ok(!sitemap.includes('/examples/docs/'))
const robots = await readFile(resolve(output, 'robots.txt'), 'utf8')
assert.ok(robots.includes(`Disallow: ${base}gallery/`))
assert.ok(robots.includes(`Disallow: ${base}examples/ai-harness/`))
assert.ok(robots.includes(`Disallow: ${base}examples/gpu-canvas/`))
assert.ok(robots.includes(`Disallow: ${base}examples/docs/`))
assert.equal(robots.includes('Sitemap:'), Boolean(origin))
for (const asset of [
  'gallery/index.html',
  'gallery/bridge.js',
  'preview-bridge.js',
  'browser-shortcuts.js',
  'gallery/pkg/argui_widget_gallery_bg.wasm',
  'examples/ai-harness/index.html',
  'examples/ai-harness/pkg/argui_example_ai_harness_bg.wasm',
  'examples/gpu-canvas/index.html',
  'examples/gpu-canvas/pkg/argui_example_gpu_canvas_bg.wasm',
  'examples/docs/index.html',
  'examples/docs/pkg/argui_example_docs_bg.wasm',
  'gallery-preview.webp',
  'social.png',
  '404.html',
])
  await access(resolve(output, asset))
console.log(
  `Verified ${routes.length} prerendered pages, source links, SEO metadata, sitemap, robots and WASM previews.`,
)
