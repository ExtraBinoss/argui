import { expect, test } from 'bun:test'
import { existsSync, readFileSync } from 'node:fs'
import { pages } from '../src/gallery-pages'

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
