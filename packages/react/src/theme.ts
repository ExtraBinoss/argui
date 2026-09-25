import { useCallback, useSyncExternalStore } from 'react'
import type { ThemeRuntime, ThemeSnapshot } from '@argui/host'

/** Tracks the coherent host theme snapshot in a React render. */
export function useThemeSnapshot<T extends object>(runtime: ThemeRuntime<T>): ThemeSnapshot<T> {
  const subscribe = useCallback((notify: () => void) => runtime.subscribe(() => notify()), [runtime])
  return useSyncExternalStore(
    subscribe,
    () => runtime.snapshot(),
    () => runtime.snapshot(),
  )
}

/** Tracks resolved theme values in a React render. */
export function useTheme<T extends object>(runtime: ThemeRuntime<T>): Readonly<T> {
  return useThemeSnapshot(runtime).values
}
