import { expect, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import docs from '../app/data/docs-index.json'
import { components, isComponentSlug } from '../app/data/components'
import { searchEntries, type SearchEntry } from '../app/utils/search'

test('published guides include the current entry points and resolve internal links', () => {
  const slugs = new Set(docs.map(page => page.slug))
  for (const slug of ['', 'getting-started', 'cli', 'crates', 'ui', 'ui/controls', 'ui/custom-components', 'platform/desktop', 'architecture']) {
    expect(slugs.has(slug)).toBe(true)
  }
  expect(docs.filter(page => page.section === 'Start').slice(0, 4).map(page => page.slug))
    .toEqual(['', 'getting-started', 'cli', 'crates'])
  expect(docs.some(page => page.slug.startsWith('plans/') || page.slug.startsWith('references/'))).toBe(false)
  for (const entry of docs) {
    const file = resolve(import.meta.dirname, '../app/data/doc-pages', `${entry.slug || 'overview'}.json`)
    const page = JSON.parse(readFileSync(file, 'utf8')) as { html: string }
    for (const [, route] of page.html.matchAll(/href="\/docs(?:\/([^"]*))?"/g)) {
      const slug = (route ?? '').split('#')[0]!
      expect(slugs.has(slug)).toBe(true)
    }
  }
  const example = JSON.parse(readFileSync(resolve(import.meta.dirname, '../app/data/doc-pages/ui/custom-components.json'), 'utf8')) as { html: string }
  expect(example.html).toContain('<!-- argui-example:custom-elements -->')
  expect(example.html).toContain('<!-- argui-example:counter -->')
  expect(example.html).toContain('class="shiki')
  expect(example.html).toContain('code-block-badge-solid">Solid TSX')
  expect(example.html).toContain('code-block-badge-react">React TSX')
  expect(example.html).toContain('code-block-badge-rust">Rust')
  const primitives = JSON.parse(readFileSync(resolve(import.meta.dirname, '../app/data/doc-pages/rendering/primitives.json'), 'utf8')) as { html: string }
  expect(primitives.html).toContain('code-block-badge-both">Solid / React TSX')
  expect(primitives.html).toContain('code-block-badge-rust">Rust')
  const cli = JSON.parse(readFileSync(resolve(import.meta.dirname, '../app/data/doc-pages/cli.json'), 'utf8')) as { html: string }
  expect(cli.html).toContain('install-cli.sh')
  expect(cli.html).toContain('argui list components')
  expect(cli.html).toContain('code-block-badge-code">PowerShell')
  const crates = JSON.parse(readFileSync(resolve(import.meta.dirname, '../app/data/doc-pages/crates.json'), 'utf8')) as { html: string }
  expect(crates.html).toContain('argui-runtime')
  expect(crates.html).toContain('file-picker')
})

test('each advertised component has a page in both TSX adapters', () => {
  const repository = resolve(import.meta.dirname, '../..')
  for (const adapter of ['solid', 'react']) {
    const source = readFileSync(resolve(repository, `apps/gallery/src/${adapter}/gallery.tsx`), 'utf8')
    for (const component of components) {
      expect(source).toContain(`{ id: '${component.slug}',`)
      expect(isComponentSlug(component.slug)).toBe(true)
    }
  }
  expect(isComponentSlug('screen-spotlight')).toBe(false)
})

test('global search covers published guide sections and ranks direct destinations', () => {
  const entries = JSON.parse(readFileSync(resolve(import.meta.dirname, '../public/search-index.json'), 'utf8')) as SearchEntry[]
  expect(entries.length).toBeGreaterThan(docs.length)
  for (const page of docs) {
    const route = page.slug ? `/docs/${page.slug}#` : '/docs#'
    expect(entries.some(entry => entry.href.startsWith(route))).toBe(true)
  }
  for (const entry of entries) {
    expect(entry.href).not.toMatch(/\/(plans|references)\//)
    const [route, anchor] = entry.href.split('#')
    const slug = route === '/docs' ? 'overview' : route!.slice('/docs/'.length)
    const page = JSON.parse(readFileSync(resolve(import.meta.dirname, '../app/data/doc-pages', `${slug}.json`), 'utf8')) as { html: string }
    expect(page.html).toContain(`id="${anchor}"`)
  }
  expect(searchEntries(entries, 'custom components')[0]?.href).toBe('/docs/ui/custom-components#custom-components-and-state')
  expect(searchEntries(entries, 'rectangle').some(hit => hit.href.startsWith('/docs/ui/custom-components#'))).toBe(true)
  const widget: SearchEntry = { title: 'Virtual list', section: 'Components', kind: 'component', href: '/components?component=virtual-list', text: components.find(component => component.slug === 'virtual-list')!.description }
  expect(searchEntries([...entries, widget], 'virtual list')[0]?.href).toBe(widget.href)
})
