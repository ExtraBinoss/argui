import { createSignal, onCleanup, type Accessor } from 'solid-js'
import { getI18nState, selectLocale, subscribeI18n, tr, type I18nArgs, type I18nState } from './index'

/** Reactive Solid access to the native locale state and Fluent formatter. */
export interface SolidI18n {
  locale: Accessor<string>
  rtl: Accessor<boolean>
  tr(id: string, args?: I18nArgs): string
  selectLocale(locale: string): I18nState
}

/** Subscribes a Solid owner to locale changes from the native localizer. */
export function useI18n(): SolidI18n {
  const [state, setState] = createSignal(getI18nState())
  onCleanup(subscribeI18n(() => setState(getI18nState())))
  return {
    locale: () => state().locale,
    rtl: () => state().rtl,
    tr: (id, args) => {
      state()
      return tr(id, args)
    },
    selectLocale,
  }
}
