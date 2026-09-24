import { useSyncExternalStore } from 'react'
import { getI18nState, selectLocale, subscribeI18n, tr, type I18nArgs, type I18nState } from './index'

/** React snapshot of native locale state with the Rust-backed formatter. */
export interface ReactI18n extends I18nState {
  tr(id: string, args?: I18nArgs): string
  selectLocale(locale: string): I18nState
}

/** Subscribes a React component to locale changes from the native localizer. */
export function useI18n(): ReactI18n {
  const state = useSyncExternalStore(subscribeI18n, getI18nState, getI18nState)
  return { ...state, tr, selectLocale }
}
