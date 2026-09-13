import assert from 'node:assert/strict'
import { access, readFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import { resolve } from 'node:path'

const website = fileURLToPath(new URL('../', import.meta.url))
const output = resolve(website, '.output/public')
const catalogue = JSON.parse(await readFile(resolve(website, 'app/data/catalogue.json'), 'utf8'))
const origin = (process.env.NUXT_PUBLIC_SITE_URL ?? '').replace(/\/$/, '')
const base = process.env.NUXT_APP_BASE_URL ?? '/'
const routes = [
  '/',
  '/features',
  '/get-started',
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
for (const item of catalogue) {
  const html = await readFile(resolve(output, 'components', item.slug, 'index.html'), 'utf8')
  assert.ok(html.includes(item.source), `Missing server-rendered source: ${item.slug}`)
}
const sitemap = await readFile(resolve(output, 'sitemap.xml'), 'utf8')
assert.equal((sitemap.match(/<loc>/g) ?? []).length, origin ? routes.length : 0)
assert.ok(!sitemap.includes('/gallery/'))
const robots = await readFile(resolve(output, 'robots.txt'), 'utf8')
assert.ok(robots.includes(`Disallow: ${base}gallery/`))
assert.equal(robots.includes('Sitemap:'), Boolean(origin))
for (const asset of [
  'gallery/index.html',
  'gallery/bridge.js',
  'gallery/browser-shortcuts.js',
  'gallery/pkg/argui_widget_gallery_bg.wasm',
  'gallery-preview.webp',
  'social.png',
  '404.html',
])
  await access(resolve(output, asset))
console.log(
  `Verified ${routes.length} prerendered pages, source links, SEO metadata, sitemap, robots and gallery assets.`,
)
