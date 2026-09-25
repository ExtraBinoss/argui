import { createSignal, onCleanup, type Accessor } from 'solid-js'
import type { ThemeRuntime, ThemeSnapshot } from '@argui/host'

/** Tracks the coherent host theme snapshot for the current Solid owner. */
export function useThemeSnapshot<T extends object>(runtime: ThemeRuntime<T>): Accessor<ThemeSnapshot<T>> {
  const [snapshot, setSnapshot] = createSignal(runtime.snapshot())
  const unsubscribe = runtime.subscribe((next) => setSnapshot(() => next))
  onCleanup(unsubscribe)
  return snapshot
}

/** Tracks resolved theme values for the current Solid owner. */
export function useTheme<T extends object>(runtime: ThemeRuntime<T>): Accessor<Readonly<T>> {
  const snapshot = useThemeSnapshot(runtime)
  return () => snapshot().values
}
