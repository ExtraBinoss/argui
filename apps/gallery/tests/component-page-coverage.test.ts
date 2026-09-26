import { expect, test } from 'bun:test'
import { readdirSync, readFileSync } from 'node:fs'

const expected = ['button', 'button-group', 'input-field', 'popover', 'select', 'virtual-list']

test('gallery contains the paired Solid and React widget pages', () => {
  for (const adapter of ['solid', 'react']) {
    const directory = new URL(`../src/${adapter}/`, import.meta.url)
    const pages = readdirSync(directory)
      .filter((file) => file.endsWith('-page.tsx') && file !== 'animation-page.tsx')
      .map((file) => file.replace(/-page\.tsx$/, ''))
      .sort()
    expect(pages).toEqual(expected)
    expect(readdirSync(directory)).toContain('animation-page.tsx')
    expect(readdirSync(directory)).toContain('layout-scenarios.tsx')
  }
})

test('each framework exports the same widgets and active sources avoid the archive', () => {
  const widgets = ['button', 'button-group', 'input-field', 'select', 'popover', 'virtual-list']
  for (const adapter of ['solid', 'react']) {
    const index = readFileSync(new URL(`../../../packages/widgets/src/${adapter}/index.ts`, import.meta.url), 'utf8')
    const gallery = readFileSync(new URL(`../src/${adapter}/gallery.tsx`, import.meta.url), 'utf8')
    for (const widget of widgets) expect(index).toContain(`'./${widget}'`)
    expect(gallery).not.toContain('OLD_API')
  }
})
