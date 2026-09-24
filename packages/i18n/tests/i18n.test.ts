import { afterEach, beforeEach, expect, test } from 'bun:test'
import { readFileSync } from 'node:fs'
import {
  getI18nState,
  isI18nBridgeAvailable,
  loadI18n,
  selectLocale,
  subscribeI18n,
  tr,
  type I18nArgs,
  type I18nConfig,
  type I18nState,
} from '../src'

interface TestI18nBridge {
  load(config: I18nConfig): I18nState
  select(locale: string): I18nState
  tr(id: string, args?: I18nArgs): string
}

const previousBridge = (globalThis as typeof globalThis & {
  __arguiBridge?: { i18n?: TestI18nBridge }
}).__arguiBridge
let calls: unknown[]
let locale: string

beforeEach(() => {
  calls = []
  locale = 'en-US'
  const i18n: TestI18nBridge = {
    load(config) {
      calls.push(['load', config])
      locale = config.locale ?? config.fallback
      return { locale, rtl: false }
    },
    select(requested) {
      calls.push(['select', requested])
      locale = requested
      return { locale, rtl: locale.startsWith('ar') }
    },
    tr(id, args) {
      calls.push(['tr', id, args])
      return `${locale}:${id}:${args?.name ?? ''}`
    },
  }
  ;(globalThis as typeof globalThis & { __arguiBridge?: { i18n?: TestI18nBridge } }).__arguiBridge = { i18n }
})

afterEach(() => {
  const root = globalThis as typeof globalThis & { __arguiBridge?: { i18n?: TestI18nBridge } }
  if (previousBridge) root.__arguiBridge = previousBridge
  else delete root.__arguiBridge
})

test('loads catalogs, formats messages, and changes locale only through the native bridge', () => {
  const config: I18nConfig = {
    fallback: 'en-US',
    catalogs: { 'en-US': { greeting: 'Hello, { $name }!' }, fr: { greeting: 'Bonjour, { $name } !' } },
  }

  expect(isI18nBridgeAvailable()).toBe(true)
  expect(loadI18n(config)).toEqual({ locale: 'en-US', rtl: false })
  expect(tr('greeting', { name: 'Ada' })).toBe('en-US:greeting:Ada')
  expect(selectLocale('fr')).toEqual({ locale: 'fr', rtl: false })
  expect(tr('greeting', { name: 'Ada' })).toBe('fr:greeting:Ada')
  expect(calls).toEqual([
    ['load', config],
    ['tr', 'greeting', { name: 'Ada' }],
    ['select', 'fr'],
    ['tr', 'greeting', { name: 'Ada' }],
  ])
  expect(getI18nState()).toEqual({ locale: 'fr', rtl: false })
})

test('notifies and releases subscribers when the native locale changes', () => {
  loadI18n({ fallback: 'en-US', catalogs: { 'en-US': {}, fr: {} } })
  let notifications = 0
  const unsubscribe = subscribeI18n(() => { notifications += 1 })

  selectLocale('fr')
  expect(notifications).toBe(1)
  expect(getI18nState().locale).toBe('fr')
  unsubscribe()
  selectLocale('en-US')
  expect(notifications).toBe(1)
})

test('the gallery bundles two JSON catalogs and demonstrates both reactive adapters', () => {
  const source = (path: string) => readFileSync(new URL(path, import.meta.url), 'utf8')
  const pages = source('../../../apps/gallery/src/gallery-pages.ts')
  const solidMain = source('../../../apps/gallery/src/main.tsx')
  const reactMain = source('../../../apps/gallery/src/react-main.tsx')
  const solidDemo = source('../../../apps/gallery/src/i18n-page.tsx')
  const reactDemo = source('../../../apps/gallery/src/react-i18n-page.tsx')
  const english = source('../../../apps/gallery/src/i18n/en-US.json')
  const french = source('../../../apps/gallery/src/i18n/fr.json')

  expect(pages).toContain('Internationalization')
  for (const entry of [solidMain, reactMain]) {
    expect(entry).toContain("import enUS from './i18n/en-US.json'")
    expect(entry).toContain("import fr from './i18n/fr.json'")
    expect(entry).toContain('loadI18n(')
  }
  for (const demo of [solidDemo, reactDemo]) {
    expect(demo).toContain('useI18n()')
    expect(demo).toContain("tr('welcome'")
    expect(demo).toContain('selectLocale(')
  }
  expect(english).toContain('Hello, { $name }!')
  expect(french).toContain('Bonjour, { $name } !')
})
