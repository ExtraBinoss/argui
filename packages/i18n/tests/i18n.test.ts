import { afterEach, beforeEach, expect, test } from 'bun:test'
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
