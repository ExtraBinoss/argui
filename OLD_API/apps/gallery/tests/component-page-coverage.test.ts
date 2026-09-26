import { expect, test } from 'bun:test'
import { existsSync, readFileSync } from 'node:fs'
import { filteredNavigation, navigationItems, pages } from '../src/gallery-pages'

const catalog = JSON.parse(readFileSync(new URL('../../../components/catalog.json', import.meta.url), 'utf8')) as string[]

test('every installable component has its own Solid and React gallery page', () => {
  const directPages = new Set(['button', 'input-field', 'select', 'popover', 'dialog'])
  for (const name of catalog) {
    const file = name === 'input-field' ? 'input-page.tsx' : `${name}-page.tsx`
    const title = name === 'input-field' ? 'Input' : name.split('-').map((part) => part === 'otp' ? 'OTP' : `${part[0]?.toUpperCase()}${part.slice(1)}`).join(' ')
    expect(pages.includes(title as typeof pages[number])).toBe(true)
    for (const adapter of ['solid', 'react']) {
      expect(existsSync(new URL(`../src/${adapter}/${file}`, import.meta.url))).toBe(true)
      if (!directPages.has(name)) {
        const router = readFileSync(new URL(`../src/${adapter}/widget-pages.tsx`, import.meta.url), 'utf8')
        expect(router.includes(`'./${file.replace('.tsx', '')}'`)).toBe(true)
        expect(router.includes(`case '${title}'`)).toBe(true)
      }
    }
  }
})

test('gallery navigation includes every page in alphabetical order within each section', () => {
  const sections = navigationItems.reduce((result, item) => {
    if (item.kind === 'heading') result.push([])
    else result.at(-1)?.push(item.page)
    return result
  }, [] as string[][])
  expect(sections).toHaveLength(2)
  for (const section of sections) expect(section).toEqual([...section].sort())
  expect(sections.flat().sort()).toEqual([...pages].sort())
  expect(filteredNavigation('tooltip').filter((item) => item.kind === 'page')).toEqual([
    { kind: 'page', page: 'Tooltip' },
  ])
})
