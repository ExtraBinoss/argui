import { createComponent, createContext, createMemo, createSignal, onCleanup, useContext, type Accessor, type JSX } from 'solid-js'
import { mergeThemeOverrides, type ThemeRuntime, type ThemeSnapshot, type ThemeValues } from '@argui/host'

interface ThemeContextValue {
  values: Accessor<Readonly<ThemeValues>>
  snapshot: Accessor<ThemeSnapshot<ThemeValues>>
}

const ThemeContext = createContext<ThemeContextValue>()

/** Installs one host-owned theme session at the application root. */
export function ThemeProvider<T extends object>(props: {
  runtime: ThemeRuntime<T>
  children: JSX.Element
}): JSX.Element {
  const [snapshot, setSnapshot] = createSignal(props.runtime.snapshot())
  const unsubscribe = props.runtime.subscribe((next) => setSnapshot(() => next))
  onCleanup(unsubscribe)
  const value: ThemeContextValue = {
    values: () => snapshot().values as ThemeValues,
    snapshot: snapshot as Accessor<ThemeSnapshot<ThemeValues>>,
  }
  return createComponent(ThemeContext.Provider, { value, get children() { return props.children } })
}

/** Overrides only named tokens for a subtree while other tokens follow the root session. */
export function ThemeScope<T extends object>(props: {
  overrides: Partial<T>
  children: JSX.Element
}): JSX.Element {
  const parent = useContext(ThemeContext)
  if (!parent) throw new Error('ThemeScope requires a root ThemeProvider')
  const values = createMemo(() => mergeThemeOverrides(parent.values(), props.overrides as Partial<ThemeValues>))
  const value: ThemeContextValue = { values, snapshot: parent.snapshot }
  return createComponent(ThemeContext.Provider, { value, get children() { return props.children } })
}

/** Reads the coherent root theme snapshot for the current Solid owner. */
export function useThemeSnapshot<T extends object = ThemeValues>(): Accessor<ThemeSnapshot<T>> {
  const context = useContext(ThemeContext)
  if (!context) throw new Error('useThemeSnapshot requires a root ThemeProvider')
  return createMemo(() => {
    const snapshot = context.snapshot()
    const values = context.values()
    return values === snapshot.values ? snapshot as ThemeSnapshot<T> : { ...snapshot, values: values as T }
  })
}

/** Reads inherited token values, including a sparse local override if present. */
export function useTheme<T extends object = ThemeValues>(): Accessor<Readonly<T>> {
  const context = useContext(ThemeContext)
  if (!context) throw new Error('useTheme requires a root ThemeProvider')
  return context.values as Accessor<Readonly<T>>
}
