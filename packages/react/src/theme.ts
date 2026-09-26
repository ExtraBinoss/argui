import { createContext, createElement, useCallback, useContext, useMemo, useSyncExternalStore, type ReactElement, type ReactNode } from 'react'
import { mergeThemeOverrides, type ThemeRuntime, type ThemeSnapshot, type ThemeValues } from '@argui/host'

interface ThemeContextValue {
  values: Readonly<ThemeValues>
  snapshot: ThemeSnapshot<ThemeValues>
}

const ThemeContext = createContext<ThemeContextValue | null>(null)

/** Installs one host-owned theme session at the application root. */
export function ThemeProvider<T extends object>(props: {
  runtime: ThemeRuntime<T>
  children: ReactNode
}): ReactElement {
  const runtime = props.runtime
  const subscribe = useCallback((notify: () => void) => runtime.subscribe(() => notify()), [runtime])
  const snapshot = useSyncExternalStore(
    subscribe,
    () => runtime.snapshot(),
    () => runtime.snapshot(),
  )
  const context = useMemo(() => ({
    values: snapshot.values as ThemeValues,
    snapshot: snapshot as ThemeSnapshot<ThemeValues>,
  }), [snapshot])
  return createElement(ThemeContext.Provider, { value: context }, props.children)
}

/** Overrides only named tokens for a subtree while other tokens follow the root session. */
export function ThemeScope<T extends object>(props: {
  overrides: Partial<T>
  children: ReactNode
}): ReactElement {
  const parent = useContext(ThemeContext)
  if (!parent) throw new Error('ThemeScope requires a root ThemeProvider')
  const values = useMemo(() => mergeThemeOverrides(parent.values, props.overrides as Partial<ThemeValues>), [parent.values, props.overrides])
  const context = useMemo(() => ({ values, snapshot: parent.snapshot }), [values, parent.snapshot])
  return createElement(ThemeContext.Provider, { value: context }, props.children)
}

/** Reads the coherent root theme snapshot for the current React render. */
export function useThemeSnapshot<T extends object = ThemeValues>(): ThemeSnapshot<T> {
  const context = useContext(ThemeContext)
  if (!context) throw new Error('useThemeSnapshot requires a root ThemeProvider')
  return useMemo(() => context.values === context.snapshot.values
    ? context.snapshot as ThemeSnapshot<T>
    : { ...context.snapshot, values: context.values as T }, [context])
}

/** Reads inherited token values, including a sparse local override if present. */
export function useTheme<T extends object = ThemeValues>(): Readonly<T> {
  const context = useContext(ThemeContext)
  if (!context) throw new Error('useTheme requires a root ThemeProvider')
  return context.values as T
}
