/** JSON-compatible variables accepted by Fluent message patterns. */
export type I18nArgs = Record<string, string | number | boolean>

/** Flat message map for one locale; Rust parses and formats each pattern. */
export type I18nCatalog = Record<string, string>

/** Configuration passed to the native Argui localizer. */
export interface I18nConfig {
  fallback: string
  catalogs: Record<string, I18nCatalog>
  locale?: string
}

/** Effective locale and writing direction selected by the native localizer. */
export interface I18nState {
  locale: string
  rtl: boolean
}

interface NativeI18nBridge {
  load(config: I18nConfig): I18nState
  select(locale: string): I18nState
  tr(id: string, args?: I18nArgs): string
}

interface ArguiBridgeGlobal {
  __arguiBridge?: { i18n?: NativeI18nBridge }
}

let currentState: I18nState | null = null
const listeners = new Set<() => void>()

/** Returns whether the current JavaScript host installed the native i18n bridge. */
export function isI18nBridgeAvailable(): boolean {
  return !!(globalThis as typeof globalThis & ArguiBridgeGlobal).__arguiBridge?.i18n
}

/** Loads catalogs through Rust and notifies framework adapters of the selected locale. */
export function loadI18n(config: I18nConfig): I18nState {
  currentState = nativeI18n().load(config)
  notifySubscribers()
  return currentState
}

/** Formats one message through Rust's Fluent localizer. */
export function tr(id: string, args?: I18nArgs): string {
  return nativeI18n().tr(id, args)
}

/** Changes the requested locale through Rust and notifies framework adapters. */
export function selectLocale(locale: string): I18nState {
  currentState = nativeI18n().select(locale)
  notifySubscribers()
  return currentState
}

/** Returns the last locale state returned by the native localizer. */
export function getI18nState(): I18nState {
  if (!currentState) throw new Error('Call loadI18n before reading the current locale')
  return currentState
}

/** Subscribes to native locale changes and returns a function that removes the listener. */
export function subscribeI18n(listener: () => void): () => void {
  listeners.add(listener)
  return () => listeners.delete(listener)
}

function nativeI18n(): NativeI18nBridge {
  const bridge = (globalThis as typeof globalThis & ArguiBridgeGlobal).__arguiBridge?.i18n
  if (!bridge) throw new Error('The Argui host does not provide the native i18n bridge')
  return bridge
}

function notifySubscribers(): void {
  for (const listener of listeners) listener()
}
